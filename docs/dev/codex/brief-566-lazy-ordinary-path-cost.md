# brief-566 — erase the lazy ordinary-path cost, measured with one codegen unit

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-565 tried to erase the ordinary-path cost G2 (briefs 563-564) added and
stopped at its stop-loss after two candidates, with the tree restored. Its
attribution (§2 of its report) is good. Its **measurement** was not, and that
was the director's error: the release profile has 16 codegen units, so adding
code anywhere in `rum-stdlib` repartitions them and moves inlining in code the
change never touched. Rebuilt with one codegen unit
(`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`), the director measured:

| Label | Pre-G2 | Now | Delta, sample 1 / 2 |
| --- | ---: | ---: | --- |
| `take-multispec-small` (ordinary list `Take`) | 49 924 | 49 922 | -0.003% / -0.004% |
| `dot-three-no-input`, `differences-no-input`, `table-small`, `do-count-null` (controls) | — | — | all within ±0.02% |
| `lazy-depth-drop-all-one-8` | 48 741 | 49 741 | +2.05% / +2.06% |
| `lazy-range-normal-small` | 53 959 | 54 845 | +1.64% / +1.42% |
| `lazy-range-take-small` | 42 153 | 42 686 | +1.26% / +1.26% |
| `lazy-subsets-take-small` | 25 330 | 25 625 | +1.17% / +0.98% |
| `lazy-foldlist-realize-small` | 112 692 | 113 839 | +1.02% / +1.07% |
| `lazy-table-construct-short` / `-full` | 23 599 / 24 135 | 23 767 / 24 254 | +0.71% / +0.98%; +0.49% / +0.99% |
| `lazy-permutations-take-small` | 28 180 | 28 390 | +0.74% / +0.94% |
| `lazy-table-realize-small` | 117 144 | 118 159 | +0.87% / +0.89% |
| `lazy-select-realize-small` | 114 677 | 115 489 | +0.71% / +0.83% |
| `lazy-depth-map-build-8` | 19 235 | 19 384 | +0.78% / +0.83% |

and `lazy-nestlist-realize-small`, `-depth-take-map-8`, `-part-small`,
`-tuples-take-small`, `-map-realize-small`, `-drop-realize-small`,
`-first-small`, `-transparent-adapters-small`, `-select-scan-small` between
+0.17% and +0.75% (raw rows in `.temp/director/gate-565/perf/`).

So: **G2 costs the ordinary non-lazy path nothing** — brief-565's +0.6% on
`Take` was partition noise, and so were the neighbour regressions that rejected
its candidate 02 (+0.07%, +0.31%, +0.18% in one sample each). **G2 does cost
shallow lazy realizations 0.2-2%**, and that is real. Erase it.

**Target:** every one of the 54 panel labels within +0.05% of the pre-G2 base,
both builds compiled with one codegen unit. A label counts as regressing when
**both** samples exceed +0.05%. Every correctness test stays unchanged and
green. If an honest effort leaves a residual, the report attributes it to the
specific work the flat machine does that the recursive one did not.

## Fixed — not negotiable in this brief

- **The flat representation and its depth contract.** One source plus a stage
  vector under an indexed cursor; no walk recurses in proportion to depth; the
  structural checks (`tests/association_depth_contract/lazy_depth_contract.rs`)
  and the 2 000-stage depth tests stay unchanged and green. **No regression is
  bought back by restoring recursion** — the recursion was a process abort.
- **Semantics.** `crates/rum-stdlib/tests/lazy_pull_continuations.rs` and its
  children, `loop_continuations.rs` and every other test stay exactly as they
  are. The cancellation checkpoint after each callback stays.

## Start here

1. **Re-measure brief-565's candidate 02 under one codegen unit** (source in
   `.temp/brief-565/candidate-02/source/`, diff in `source-change.diff`): it
   moves `LazyTable`'s bound recovery to the suspension edge and brought the
   full constructor to the base before partition noise rejected it.
2. **Attribute on one-unit binaries.** brief-565's callgrind tables were taken
   on 16-unit builds; repeat the attribution for the shallow lazy families on
   the one-unit base and current binaries (`valgrind --tool=callgrind`, each
   total calibrated against `perf stat` on the same binary). brief-565 §2
   already names the moved work: `machine_from_descriptor` and `push_stage`
   replacing `stepper_from_descriptor`, `drive_items` / `drive_pull` layered
   over source pulls, `Vec` drop, `Symbol::is_system_name` calls in descriptor
   parsing. Hypotheses, each to be measured: build per realization what is now
   built per item; a plain index on the ordinary entry rather than a cursor
   enum; a descriptor parse that does not re-test symbol names it has already
   classified; no `Vec` allocation or drop work for a stage-free machine where
   the representation allows it without a second code path that could drift.

## Measurement

- **Base:** `.temp/brief-566/base/rum-cli` and `rum-cli-alloc` — the pre-G2
  source (`ab78df14`'s crates, identical to `f8e73ac4`'s) built by the
  director with one codegen unit; hashes in `.temp/brief-566/base/sha256.txt`.
  Do not rebuild them.
- **Candidates:** build the working tree with
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into a fresh target directory per
  candidate under `.temp/brief-566/`, both default and
  `--features rum-cli/alloc-count`, and check the produced binary's hash
  changed (brief-564 caught a "fresh" build that left a stale binary). Keep
  every measured binary, its hash and its source. Measure the incoming tree
  once, the same way, as candidate 00.
- **Panel:** `.temp/brief-562/performance-panel.json`, 54 labels, workloads
  `.temp/brief-563/perf/<label>-<N>.rum`, result checks
  `.temp/brief-563/perf/<label>-check.rum` (check before measuring).
  `.temp/brief-565/metric-panel.py`, `compare-metrics.py` and `attribute.py`
  are a working harness to copy and adapt.
- **Method:** allocation counts first (`RUM_PROFILE_ALLOC=1`), then two
  samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label
  per process, stdin `/dev/null`, other `RUM_PROFILE_*` cleared,
  `(count(2N) - count(N)) / N`, raw output saved before parsing. An exploratory
  candidate may be screened on the lazy labels plus `take-multispec-small`,
  `dot-three-no-input`, `differences-no-input`, `table-small`; **any candidate
  you keep, and the final one, runs all 54.** No timing.

## Scope

- **Objective:** the ordinary-path shape of `crates/rum-stdlib/src/lazy.rs`,
  `crates/rum-stdlib/src/lazy/`, and the lazy branches of `eval_take`,
  `eval_first`, `eval_normal`.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`): replay
  the retention fingerprints and both certificates, reconcile the ownership
  inventories, keep rum-runtime's frame lists exact. New `UnknownConstruct`
  rows for code you add or change are expected and disclosed; STOP on a
  changed or vanished existing row identity, a new `UnknownHostPayload` or
  `self_reaching` row, or a row in code you did not touch.
- **Fence:** no semantic change and no test edit. No rum-runtime or rum-core
  production change, no `Expr` change, no other brief-548 group, no leases, no
  dependency, no `unsafe`, no depth cap, no recursion, no change to the release
  profile in `Cargo.toml` (one codegen unit is a measurement setting here, set
  by environment). Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier
  briefs or docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  candidate — what it changed, its attribution, its rows.

## rum / Rust pitfalls (read twice)

1. **One variable per candidate.**
2. **A candidate that makes another label regress (both samples above +0.05%)
   is rejected**, a deep label pushed above the base included.
3. **Instruction counts decide.**
4. **Stop-loss is keyed on progress:** the number of regressing labels or the
   largest remaining delta must fall. Stop after two consecutive compiling
   candidates that reduce neither; at most ten measured candidates. Report the
   curve and the attribution of every residual.
5. `#[inline(always)]` is not a fix by itself (brief-560: +1.17% on
   `Differences`). Prefer moving cold work out over forcing hot work in.
6. A second ordinary code path kept alongside the general one (a "shallow
   specialization") is a drift risk; it needs its own falsifier proving both
   paths agree, or do not add it.

## Verification (MANDATORY — you run it; iterate to green)

Serial, every Cargo command through `python3 scripts/run-bounded.py --timeout
<s> -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-566/tmp …`. Bounds: a focused test target 150 s; a
one-unit release build 600 s (the director's took 98 s); a metric process 150 s;
`local_production_acceptance` and `cargo test -p rum-runtime` 300 s;
`cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test lazy_pull_continuations
cargo test -p rum-stdlib --lib lazy
cargo test -p rum-stdlib --test loop_continuations
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

Plus the retention replay in both feature sets. Paste commands, bounds, exit
statuses and counts verbatim; list native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-566/`. After each candidate write
`.temp/brief-566/PROGRESS.md`: its change, its attribution, its rows, the
counters, the exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads and
  edits within scope, `rg`/`grep`, `python3`, bounded `cargo`, the release
  `rum-cli`, bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind`
  with `callgrind_annotate`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. The one-unit attribution for each lazy family, base against incoming.
3. Every candidate measured, with its rows (all 54 for kept candidates), both
   samples.
4. Verification — every command's verbatim result.
5. Progress counters, disclosures, and the attribution of anything left.

---

## Execution report — brief 566

Retained `candidate-06`. The incoming build had 26 regressing labels; the retained build has 6. The largest sample among its still-regressing labels is +0.398470%; the largest replicated delta (the smaller sample for each failing label) is +0.118882%. **The +0.05% target is not fully met.** Stopped at the ten-measured-candidate cap (including incoming 00).

### 1. What changed — files, ranges, and retained state

Only lazy implementation files, replayed inventories, and this appended report are changed outside scratch. Test code remains byte-identical to the entry snapshot; replay-only inventory data changes are listed below. The runtime/core implementation, frame types and closed frame lists, release profile, dependency files and other objective boundaries remain unchanged. No git command or network tool was run; no unsafe, recursion, depth cap, diagnostic instrumentation or second shallow realization algorithm was introduced.

LazyTable borrows raw bounds until evaluation and reconstructs pending/future owned bounds on the error edge. Descriptor parsing classifies symbol context/name once per descriptor visit. Output filling runs inside one indexed loop; ordinary entry supplies an index and carried value, while the resumed bridge decodes its cursor. Every callback cancellation checkpoint remains in place.

| Changed file | Final line ranges | Why |
| --- | --- | --- |
| `crates/rum-stdlib/src/lazy/descriptor.rs` | 5–6, 15–17, 19–148, after 149 (deletion), 175, 177–185 | Classify the symbol and System context once per descriptor visit, then dispatch by its name. |
| `crates/rum-stdlib/src/lazy/machine.rs` | 1–2, 4, 8, 12–19, 124–128, 132, 138 | Fill the output prefix inside one shared indexed machine loop, retaining the callback cancellation checkpoints and exhaustion propagation. |
| `crates/rum-stdlib/src/lazy/realization.rs` | 1, 4, 16, 18–19, 21–28 | Decode resumed cursors before entering the same indexed loop used by ordinary realization; preserve exhaustion handling. |
| `crates/rum-stdlib/src/lazy.rs` | 239–241, 248, 255–259, 265, 272–276, 282, 289, 446 | Borrow raw LazyTable bounds until evaluation and rebuild suspension data only on error; pass a plain index and carried value at ordinary realization entry. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 2196–2199 | Replay-only inventory maintenance; identities, statuses, dispositions and exclusions unchanged. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 2407–2417, 2419 | Replay-only inventory maintenance; identities, statuses, dispositions and exclusions unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 2–69 | Replay-only inventory maintenance; identities, statuses, dispositions and exclusions unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 2–3 | Replay-only inventory maintenance; identities, statuses, dispositions and exclusions unchanged. |
| `docs/dev/codex/brief-566-lazy-ordinary-path-cost.md` | 191–2675 | Append the required execution report. |

The current brief is also changed by this appended report. Scratch source snapshots, raw outputs, commands, hashes, reducers and checkpoints are confined to `.temp/brief-566/`; the existing replay tests additionally write their established generated outputs under `target/brief-544/` and `.temp/brief-546/certificates/`.

Changed ownership rows are listed below as literal before/after discovery rows. Position changes are permitted; no existing identity is removed or reclassified.

`unknown_captures.tsv`:

```text
before rows:
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	357	30	closure captures: |items| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	357	55	closure captures: |item| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	50	40	closure captures: |list| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	55	22	closure captures: |list| (capture types/modes unresolved)
after rows:
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	365	30	closure captures: |items| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	365	55	closure captures: |item| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	55	44	closure captures: |list| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	60	26	closure captures: |list| (capture types/modes unresolved)
```

`unknown_macros.tsv`:

```text
before rows:
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	312	9	unexpanded macro vec! (ends at 312:45)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	314	54	unexpanded macro vec! (ends at 314:70)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	315	50	unexpanded macro vec! (ends at 315:71)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	318	9	unexpanded macro vec! (ends at 318:54)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	344	9	unexpanded macro vec! (ends at 344:69)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	370	45	unexpanded macro vec! (ends at 370:70)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	384	17	unexpanded macro format! (ends at 384:75)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	391	9	unexpanded macro vec! (ends at 391:28)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	396	5	unexpanded macro matches! (ends at 399:6)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	406	9	unexpanded macro vec! (ends at 406:69)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	512	41	unexpanded macro vec! (ends at 512:57)
UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	57	25	unexpanded macro vec! (ends at 57:51)
after rows:
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	320	9	unexpanded macro vec! (ends at 320:45)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	322	54	unexpanded macro vec! (ends at 322:70)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	323	50	unexpanded macro vec! (ends at 323:71)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	326	9	unexpanded macro vec! (ends at 326:54)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	352	9	unexpanded macro vec! (ends at 352:69)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	378	45	unexpanded macro vec! (ends at 378:70)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	392	17	unexpanded macro format! (ends at 392:75)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	399	9	unexpanded macro vec! (ends at 399:28)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	404	5	unexpanded macro matches! (ends at 407:6)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	414	9	unexpanded macro vec! (ends at 414:69)
UnknownConstruct	crates/rum-stdlib/src/lazy.rs	513	41	unexpanded macro vec! (ends at 513:57)
UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	62	29	unexpanded macro vec! (ends at 62:55)
```

Contract fingerprint changes (only column 19; all other cells unchanged):

| TSV line | Contract | Before | After |
| ---: | --- | --- | --- |
| 2 | `B01` | `fnv1a64:97e9f9cbd1620a22` | `fnv1a64:e9fa6f18b50b8566` |
| 3 | `B02` | `fnv1a64:f25d84ddfa27f86b` | `fnv1a64:08ea10fd5b2bc8df` |
| 4 | `B03` | `fnv1a64:b13fcc2d666ada44` | `fnv1a64:a3a6341a1fa04d54` |
| 5 | `B04` | `fnv1a64:8a14ac32dc071b8a` | `fnv1a64:fb22d495d4bf858e` |
| 6 | `B05` | `fnv1a64:aee489e122b84e87` | `fnv1a64:3912d23cc3404fbb` |
| 7 | `B06` | `fnv1a64:07039fcfe2b95ff6` | `fnv1a64:e1923469a3bc125a` |
| 8 | `B07` | `fnv1a64:39cc414d8bf99f57` | `fnv1a64:25ca2d347d7ea4ab` |
| 9 | `B08` | `fnv1a64:fd013cff43ff3340` | `fnv1a64:be0f3ff851165e30` |
| 10 | `B09` | `fnv1a64:b5fec5daa0d638be` | `fnv1a64:d4ab03a29da66342` |
| 11 | `B10` | `fnv1a64:4cf2bbb55f56e3f5` | `fnv1a64:c215ddb20a95c425` |
| 12 | `B11` | `fnv1a64:4b6c5d8ff5fb418a` | `fnv1a64:bc7a85f2eeb3ab8e` |
| 13 | `B12` | `fnv1a64:750b45d67cdecdbe` | `fnv1a64:93b7839e79aef842` |
| 14 | `I01` | `fnv1a64:75cbe4c93e1a00a2` | `fnv1a64:c7dc5a1621c37be6` |
| 15 | `I02` | `fnv1a64:062607751aeed687` | `fnv1a64:90544fd0bb76d7bb` |
| 16 | `I03` | `fnv1a64:59143a5476c7d47e` | `fnv1a64:35190841e354e682` |
| 17 | `I04` | `fnv1a64:da5a41827274130b` | `fnv1a64:c842732cf1f848ff` |
| 18 | `P01` | `fnv1a64:3369404b9f3b7716` | `fnv1a64:a9de46132044a2fa` |
| 19 | `P02` | `fnv1a64:5fe372b4f609f6e5` | `fnv1a64:dfb6787ec3c003b5` |
| 20 | `P03` | `fnv1a64:b3ded0155ee8427c` | `fnv1a64:2e6faac91544fb6c` |
| 21 | `P04` | `fnv1a64:e738a2f37ed1146b` | `fnv1a64:fdc52f12dfd4e4df` |
| 22 | `P05` | `fnv1a64:ccebb694ec228852` | `fnv1a64:5be438d2783f7ff6` |
| 23 | `P06` | `fnv1a64:c58a6aa7090e673e` | `fnv1a64:e436a86f05de91c2` |
| 24 | `P070` | `fnv1a64:ffd806e8e783e13e` | `fnv1a64:1e8444b0e4540bc2` |
| 25 | `P080` | `fnv1a64:8cce6312611ae3b5` | `fnv1a64:4dc8455bce0d9f65` |
| 26 | `P090` | `fnv1a64:7b6eb69c6907482b` | `fnv1a64:f1410000eca1801f` |
| 27 | `P100` | `fnv1a64:ef62faf4b9013a5a` | `fnv1a64:a27742229d6dd2fe` |
| 28 | `P110` | `fnv1a64:d30470bcd0ce4bad` | `fnv1a64:2ce5fd8ef351c8fd` |
| 29 | `P120` | `fnv1a64:9a08e0e47055b060` | `fnv1a64:2f094bde7fe421d0` |
| 30 | `P130` | `fnv1a64:dd999aaabaad7957` | `fnv1a64:c9978691ac327eab` |
| 31 | `P140` | `fnv1a64:dee98ac6e2eb37a3` | `fnv1a64:9871b27cb12dfc37` |
| 32 | `P150` | `fnv1a64:7d11720bcc651fcb` | `fnv1a64:19ceebbfbff20b3f` |
| 33 | `P071` | `fnv1a64:7a4df1ba22772877` | `fnv1a64:3bbd73c95e92ff4b` |
| 34 | `P081` | `fnv1a64:a287d2689d2d54de` | `fnv1a64:b9e296465adeb2e2` |
| 35 | `P091` | `fnv1a64:b764102a340ab8d6` | `fnv1a64:b71f8dab1b16a53a` |
| 36 | `P101` | `fnv1a64:cb07618a0631e8ad` | `fnv1a64:24e8ee5c28b565fd` |
| 37 | `P111` | `fnv1a64:0b1968b1cb139f8e` | `fnv1a64:79d03fa5164c3932` |
| 38 | `P121` | `fnv1a64:b04a386cc58a6eb9` | `fnv1a64:78660d6f80c328b9` |
| 39 | `P131` | `fnv1a64:27bece99125b3d0a` | `fnv1a64:98ccf6fc0b13a70e` |
| 40 | `P141` | `fnv1a64:871d1f48d02641b6` | `fnv1a64:c60ec886b36cd49a` |
| 41 | `P151` | `fnv1a64:d084e309d7d4c1a0` | `fnv1a64:df593dce88e5b690` |
| 42 | `B13` | `fnv1a64:9514335913561bf6` | `fnv1a64:6fa2c7f2d458ce5a` |
| 43 | `R01` | `fnv1a64:d3de88dc4286233b` | `fnv1a64:04101fc5d3e2678f` |
| 44 | `R02` | `fnv1a64:76d15a1c313c463c` | `fnv1a64:fca8756a2a8ea0ac` |
| 45 | `R03` | `fnv1a64:e951b548e2508a5f` | `fnv1a64:64331647b3d8f653` |
| 46 | `R040` | `fnv1a64:4bf6431487ac33db` | `fnv1a64:01505dc33876d0af` |
| 47 | `R050` | `fnv1a64:d898e4a1ffd0c6b2` | `fnv1a64:7c0a1e99082250d6` |
| 48 | `R060` | `fnv1a64:23ce20eee59f4e33` | `fnv1a64:72adf3d4d053f4a7` |
| 49 | `R070` | `fnv1a64:0af59a827ecf13de` | `fnv1a64:22505e603c8071e2` |
| 50 | `R080` | `fnv1a64:a15eaa7541f8dc0b` | `fnv1a64:8f46dc1fc17d11ff` |
| 51 | `R090` | `fnv1a64:91ccad07d88f1c1a` | `fnv1a64:cb0d046b1b79313e` |
| 52 | `R100` | `fnv1a64:76657080104084c4` | `fnv1a64:68cbd86cc975f7d4` |
| 53 | `R110` | `fnv1a64:93d3559dc91e49fd` | `fnv1a64:ba273dc48c74716d` |
| 54 | `R041` | `fnv1a64:7b4d96832a7ddaf6` | `fnv1a64:55dc2b1ceb808d5a` |
| 55 | `R051` | `fnv1a64:0990e4b1f63554ab` | `fnv1a64:7f632e1679cf8c9f` |
| 56 | `R061` | `fnv1a64:9e0078bd7348a0b0` | `fnv1a64:6c7cf7e6bcceaa80` |
| 57 | `R071` | `fnv1a64:be51a4c2d6515321` | `fnv1a64:8cf07a6e72d729c1` |
| 58 | `R081` | `fnv1a64:ab4d644954ad0f5a` | `fnv1a64:5e61ab773919a7fe` |
| 59 | `R091` | `fnv1a64:a3869e5b62575fe3` | `fnv1a64:6561e60324154ef7` |
| 60 | `R101` | `fnv1a64:9031b300d23998f3` | `fnv1a64:6857fda022f0ffe7` |
| 61 | `R111` | `fnv1a64:6ed14120080d9ea8` | `fnv1a64:74580d8ced382a18` |
| 62 | `R12` | `fnv1a64:b88f4711a19256e8` | `fnv1a64:1ac4597763f45bd8` |
| 63 | `R13` | `fnv1a64:27c281b4fae9b534` | `fnv1a64:848d00cc79ce8864` |
| 64 | `R14` | `fnv1a64:09bb096a95b1fa5a` | `fnv1a64:bccf50987a1e92fe` |
| 65 | `R15` | `fnv1a64:290db54f65ab5ea6` | `fnv1a64:30b729ea1c7c56aa` |
| 66 | `O010` | `fnv1a64:310c78b63a6bd890` | `fnv1a64:d3a290ad74d3a760` |
| 67 | `O0200` | `fnv1a64:d9b12c9b31a25da3` | `fnv1a64:93395450ffe52237` |
| 68 | `O011` | `fnv1a64:6656dd76f33112b3` | `fnv1a64:b536b05cdde5b927` |
| 69 | `O0210` | `fnv1a64:bb226e5bbda7a3c8` | `fnv1a64:34f8a6407df52f38` |

Certificate replay cells (identity/status columns unchanged):

`C01`, TSV line 2, column 5, hash feature true, changed=true:

```text
before: instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:111f333865605c91", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:ff9e10ecb6061f55;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:bb226e5bbda7a3c8;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:6656dd76f33112b3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:111f333865605c91", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:a7d91fa29343c40f;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:d9b12c9b31a25da3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:310c78b63a6bd890;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
after:  instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:38253ab66367efa5", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:f359b9f478e116d7;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:34f8a6407df52f38;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:b536b05cdde5b927;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:38253ab66367efa5", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:e109fc5a9003218d;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:93395450ffe52237;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:d3a290ad74d3a760;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```

`C01`, TSV line 2, column 6, hash feature false, changed=true:

```text
before: instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:560fe11c6ec0b786", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:253ca0c58e4549a4;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:bb226e5bbda7a3c8;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:6656dd76f33112b3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:560fe11c6ec0b786", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:ba2daf537b1bc2e2;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:d9b12c9b31a25da3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:310c78b63a6bd890;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
after:  instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:76f9953bbd805a89", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:8acb9d1ecbb26414;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:34f8a6407df52f38;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:b536b05cdde5b927;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:76f9953bbd805a89", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:7aa37f877a395c22;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:93395450ffe52237;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:d3a290ad74d3a760;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```

`C02`, TSV line 3, column 5, hash feature true, changed=true:

```text
before: instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:4ca7ae41382d017f", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:77962e00a4a30170;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:75cbe4c93e1a00a2;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:062607751aeed687;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:da5a41827274130b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:d3de88dc4286233b;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:76d15a1c313c463c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:9514335913561bf6;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:aee489e122b84e87;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
after:  instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:a190d45191f19f6b", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:44920b49eb9fd146;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:c7dc5a1621c37be6;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:90544fd0bb76d7bb;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:c842732cf1f848ff;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:04101fc5d3e2678f;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:fca8756a2a8ea0ac;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:6fa2c7f2d458ce5a;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:3912d23cc3404fbb;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```

`C02`, TSV line 3, column 6, hash feature false, changed=true:

```text
before: instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:f371f7a9c1ee0fd8", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:8bdecfea2e7ba68f;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:75cbe4c93e1a00a2;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:062607751aeed687;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:da5a41827274130b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:d3de88dc4286233b;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:76d15a1c313c463c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:9514335913561bf6;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:aee489e122b84e87;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
after:  instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:163fdcff0fceca07", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:fdf9db253499cc59;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:c7dc5a1621c37be6;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:90544fd0bb76d7bb;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:c842732cf1f848ff;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:04101fc5d3e2678f;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:fca8756a2a8ea0ac;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:6fa2c7f2d458ce5a;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:3912d23cc3404fbb;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```



Ownership inventories retain 7,585 UnknownConstruct, 63 UnknownHostPayload and 59 self-reaching rows in both feature sets, with no new or vanished identities. Four capture positions and twelve macro positions moved within the changed lazy files. Replay updated 68 contract fingerprints and four hash cells for C01/C02; both certificates were then confirmed in default and no-default feature sets. The runtime closed frame lists remain exact and unchanged.

Source audit: 1232 entry snapshot files; the final production bytes match the archived measured `candidate-06` source. The standing replay changes TSV data only after measurement. `lazy.rs` remains in the existing 10–20k warning band; the changed child modules remain below 10k. No organizational cleanup was added.

### 2. One-unit attribution — provided pre-G2 base against incoming

All selected profiles use the preserved one-unit binaries. The supplied base binaries were never rebuilt. Every lazy label was profiled, plus ReplaceAll as a control. N/2N self-cost slopes avoid summing recursive inclusive costs. Parser inlining into `realize`, new stage reservation, indexed descent/ascent, optional result cleanup and context/stage destruction account for the changed lazy work. LazyTable adds three bound clones for suspension recovery; the constructor trials remove 114 clone self instructions and 63 Expr/Atom drop self instructions relative to incoming.

Callgrind and hardware totals are calibrated separately on each binary and source. Raw totals differ by up to about 0.42% in the base/incoming diagnostic set, so attribution does not replace the full-N hardware gate. Small total differences also include symbol interning, cache and allocator effects outside the named lazy functions.

All figures below are Callgrind self instructions per workload iteration, using the N/2N slope. Recursive function suffixes are summed. Parser work in the incoming binary is partly inlined into `realize`; function-name rows are attribution buckets, not independently interchangeable algorithms. Inclusive recursive costs are preserved in the raw tables but are not added here.

| Lazy label | N | Program base → incoming | Δ | Old parser | Old pull → new driver | `realize` base → incoming | New `push_stage` | `Symbol::is_system_name` Δ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `lazy-range-take-small` | 1000 | 42262.121 → 42767.477 | +505.356 | 74.000 | 672.000 → 1329.000 | 377.000 → 197.000 | 0.000 | +0.000 |
| `lazy-range-normal-small` | 1000 | 54152.787 → 55023.589 | +870.802 | 203.000 | 811.000 → 1815.000 | 379.000 → 284.000 | 78.000 | +0.000 |
| `lazy-map-realize-small` | 1000 | 179041.641 → 180149.157 | +1107.516 | 319.000 | 1571.000 → 2471.000 | 379.000 → 355.000 | 220.000 | +89.000 |
| `lazy-select-realize-small` | 1000 | 114572.881 → 115531.583 | +958.702 | 324.000 | 1499.000 → 2075.000 | 247.000 → 361.000 | 227.000 | +103.000 |
| `lazy-select-scan-small` | 1000 | 805520.413 → 806698.499 | +1178.086 | 195.000 | 5156.000 → 6209.000 | 237.000 → 280.000 | 78.000 | +0.000 |
| `lazy-nestlist-realize-small` | 1000 | 145163.350 → 146114.043 | +950.693 | 203.000 | 955.000 → 1991.000 | 379.000 → 289.000 | 78.000 | +0.000 |
| `lazy-foldlist-realize-small` | 1000 | 112960.779 → 114265.618 | +1304.839 | 352.000 | 1598.000 → 2582.000 | 379.000 → 390.000 | 248.000 | +145.000 |
| `lazy-drop-realize-small` | 1000 | 192063.111 → 193022.354 | +959.243 | 453.000 | 1782.000 → 2331.000 | 247.000 → 443.000 | 304.000 | +155.000 |
| `lazy-first-small` | 1000 | 41279.030 → 41560.975 | +281.945 | 190.000 | 179.000 → 311.000 | 135.000 → 274.000 | 78.000 | +0.000 |
| `lazy-part-small` | 1000 | 88785.092 → 89395.288 | +610.196 | 324.000 | 628.000 → 796.000 | 135.000 → 362.000 | 220.000 | +89.000 |
| `lazy-transparent-adapters-small` | 1000 | 182370.678 → 182820.549 | +449.871 | 716.000 | 1535.000 → 2036.000 | 135.000 → 612.000 | 458.000 | +249.000 |
| `lazy-table-construct-short` | 1000 | 23574.853 → 23754.187 | +179.334 | 0.000 | 0.000 → 0.000 | 0.000 → 0.000 | 0.000 | +0.000 |
| `lazy-table-construct-full` | 1000 | 24178.661 → 24291.081 | +112.420 | 0.000 | 0.000 → 0.000 | 0.000 → 0.000 | 0.000 | +0.000 |
| `lazy-table-realize-small` | 1000 | 117370.337 → 118492.837 | +1122.500 | 319.000 | 995.000 → 1592.000 | 283.000 → 355.000 | 220.000 | +89.000 |
| `lazy-permutations-take-small` | 1000 | 28804.469 → 29021.554 | +217.085 | 150.000 | 207.000 → 499.000 | 203.000 → 270.000 | 0.000 | +0.000 |
| `lazy-subsets-take-small` | 1000 | 26096.526 → 26206.581 | +110.055 | 58.000 | 435.000 → 747.000 | 203.000 → 262.000 | 0.000 | +0.000 |
| `lazy-tuples-take-small` | 1000 | 27198.388 → 27376.656 | +178.268 | 128.000 | 515.000 → 795.000 | 203.000 → 326.000 | 0.000 | +0.000 |
| `lazy-depth-map-build-8` | 1000 | 19259.432 → 19433.959 | +174.527 | 1002.000 | 0.000 → 28.000 | 92.000 → 771.000 | 604.000 | +203.000 |
| `lazy-depth-map-first-8` | 1000 | 132119.165 → 131793.336 | -325.829 | 1002.000 | 840.000 → 879.000 | 135.000 → 771.000 | 604.000 | +203.000 |
| `lazy-depth-map-eight-8` | 1000 | 963837.634 → 961475.616 | -2362.018 | 1002.000 | 6748.000 → 6651.000 | 377.000 → 771.000 | 604.000 | +203.000 |
| `lazy-depth-select-restart-8` | 1000 | 181438.903 → 181389.127 | -49.776 | 1042.000 | 2087.000 → 2018.000 | 135.000 → 819.000 | 653.000 | +301.000 |
| `lazy-depth-drop-all-one-8` | 1000 | 48738.577 → 49761.599 | +1023.022 | 1146.000 | 2472.000 → 2781.000 | 135.000 → 907.000 | 744.000 | +532.000 |
| `lazy-depth-take-map-8` | 1000 | 141457.913 → 142421.761 | +963.848 | 2034.000 | 944.000 → 1223.000 | 135.000 → 1419.000 | 1207.000 | +551.000 |
| `lazy-depth-fold-through-source-8` | 1000 | 593250.245 → 592844.691 | -405.554 | 1266.000 | 4688.000 → 5347.000 | 415.000 → 1051.000 | 800.000 | +595.000 |
| `lazy-depth-nest-map-8` | 1000 | 142389.436 → 142734.988 | +345.552 | 1002.000 | 862.000 → 907.000 | 135.000 → 776.000 | 604.000 | +208.000 |
| `lazy-depth-map-build-64` | 1000 | 40678.659 → 39921.227 | -757.432 | 7498.000 | 0.000 → 28.000 | 92.000 → 4747.000 | 4188.000 | +1267.000 |
| `lazy-depth-map-first-64` | 1000 | 955610.993 → 959369.671 | +3758.678 | 7498.000 | 6160.000 → 5471.000 | 135.000 → 4747.000 | 4188.000 | +1267.000 |
| `lazy-depth-map-eight-64` | 1000 | 7432164.998 → 7432771.759 | +606.761 | 7498.000 | 49308.000 → 43387.000 | 377.000 → 4747.000 | 4188.000 | +1267.000 |
| `lazy-depth-select-restart-64` | 1000 | 994574.295 → 990884.188 | -3690.107 | 7818.000 | 14743.000 → 12658.000 | 135.000 → 5131.000 | 4629.000 | +2149.000 |
| `lazy-depth-drop-all-one-64` | 1000 | 351288.000 → 355494.310 | +4206.310 | 8650.000 | 94480.000 → 94201.000 | 135.000 → 5835.000 | 5448.000 | +4228.000 |
| `lazy-depth-take-map-64` | 1000 | 1032186.997 → 1034368.762 | +2181.765 | 15754.000 | 6992.000 → 8223.000 | 135.000 → 9931.000 | 9103.000 | +4247.000 |
| `lazy-depth-fold-through-source-64` | 10 | 33471398.000 → 33466336.700 | -5061.300 | 9610.000 | 248400.000 → 252655.000 | 2331.000 → 6987.000 | 5952.000 | +4795.000 |
| `lazy-depth-nest-map-64` | 1000 | 969833.587 → 968898.762 | -934.825 | 7498.000 | 6182.000 → 5499.000 | 135.000 → 4752.000 | 4188.000 | +1272.000 |
| `lazy-depth-map-build-256` | 1000 | 114230.688 → 109565.581 | -4665.107 | 29770.000 | 0.000 → 28.000 | 92.000 → 18379.000 | 16476.000 | +4915.000 |
| `lazy-depth-map-first-256` | 1000 | 3775702.313 → 3775677.885 | -24.428 | 29770.000 | 24400.000 → 21215.000 | 135.000 → 18379.000 | 16476.000 | +4915.000 |
| `lazy-depth-map-eight-256` | 10 | 29479836.700 → 29404579.900 | -75256.800 | 29770.000 | 195228.000 → 169339.000 | 377.000 → 18379.000 | 16476.000 | +4915.000 |
| `lazy-depth-select-restart-256` | 10 | 3742458.000 → 3725312.200 | -17145.800 | 31050.000 | 58135.000 → 49138.000 | 135.000 → 19915.000 | 18261.000 | +8485.000 |
| `lazy-depth-drop-all-one-256` | 10 | 2388408.100 → 2379446.500 | -8961.600 | 34378.000 | 1409872.000 → 1383769.000 | 135.000 → 22731.000 | 21576.000 | +16900.000 |
| `lazy-depth-take-map-256` | 10 | 4090709.400 → 4091897.200 | +1187.800 | 62794.000 | 27728.000 → 32223.000 | 135.000 → 39115.000 | 36175.000 | +16919.000 |
| `lazy-depth-fold-through-source-256` | 1 | 528137007.000 → 529669039.000 | +1532032.000 | 38218.000 | 3893328.000 → 3909879.000 | 8867.000 → 27339.000 | 23616.000 | +19195.000 |
| `lazy-depth-nest-map-256` | 10 | 3786711.300 → 3772802.100 | -13909.200 | 29770.000 | 24422.000 → 21243.000 | 135.000 → 18384.000 | 16476.000 | +4920.000 |

Every selected Callgrind total is paired with a same-binary, same-source hardware count. These are diagnostic runs; the fixed-N, two-sample acceptance measurements in §3 remain decisive.

| Label | N | Base calibration % at N / 2N | Incoming calibration % at N / 2N |
| --- | ---: | ---: | ---: |
| `lazy-depth-drop-all-one-256` | 10 | +0.186455 / +0.110789 | +0.184479 / +0.110287 |
| `lazy-depth-drop-all-one-64` | 1000 | +0.017664 / +0.009143 | +0.023118 / +0.015839 |
| `lazy-depth-drop-all-one-8` | 1000 | +0.121756 / +0.061739 | +0.163252 / +0.108536 |
| `lazy-depth-fold-through-source-256` | 1 | +0.014734 / +0.008319 | +0.015327 / +0.170194 |
| `lazy-depth-fold-through-source-64` | 10 | +0.022483 / +0.013066 | +0.027123 / +0.014453 |
| `lazy-depth-fold-through-source-8` | 1000 | +0.010392 / +0.000828 | +0.010601 / +0.002766 |
| `lazy-depth-map-build-256` | 1000 | +0.055122 / +0.032305 | +0.078080 / +0.056137 |
| `lazy-depth-map-build-64` | 1000 | +0.183150 / +0.056548 | +0.157044 / +0.141742 |
| `lazy-depth-map-build-8` | 1000 | +0.294788 / +0.207395 | +0.303791 / +0.251296 |
| `lazy-depth-map-eight-256` | 10 | +0.197197 / +0.189414 | +0.015029 / +0.013765 |
| `lazy-depth-map-eight-64` | 1000 | -0.008371 / -0.007827 | -0.007261 / -0.008335 |
| `lazy-depth-map-eight-8` | 1000 | +0.005526 / +0.000596 | +0.010100 / -0.008143 |
| `lazy-depth-map-first-256` | 1000 | +0.180989 / +0.180464 | +0.004117 / +0.181530 |
| `lazy-depth-map-first-64` | 1000 | +0.174219 / -0.167298 | +0.011673 / +0.008622 |
| `lazy-depth-map-first-8` | 1000 | -0.099855 / +0.179953 | +0.057658 / +0.035456 |
| `lazy-depth-nest-map-256` | 10 | +0.282588 / +0.239912 | +0.144456 / +0.075822 |
| `lazy-depth-nest-map-64` | 1000 | -0.161070 / -0.164949 | +0.025607 / +0.022760 |
| `lazy-depth-nest-map-8` | 1000 | +0.047511 / +0.024288 | +0.193152 / +0.175735 |
| `lazy-depth-select-restart-256` | 10 | +0.421946 / +0.391872 | +0.154774 / +0.067965 |
| `lazy-depth-select-restart-64` | 1000 | +0.311235 / +0.308882 | +0.033891 / -0.001852 |
| `lazy-depth-select-restart-8` | 1000 | +0.031414 / +0.031966 | +0.030371 / -0.127607 |
| `lazy-depth-take-map-256` | 10 | +0.114357 / +0.065760 | +0.103415 / +0.063518 |
| `lazy-depth-take-map-64` | 1000 | +0.000288 / +0.004428 | +0.005627 / +0.010463 |
| `lazy-depth-take-map-8` | 1000 | +0.043985 / +0.024033 | +0.054865 / +0.042353 |
| `lazy-drop-realize-small` | 1000 | +0.045040 / -0.026202 | +0.040433 / +0.033036 |
| `lazy-first-small` | 1000 | +0.356978 / +0.102262 | +0.174798 / +0.123426 |
| `lazy-foldlist-realize-small` | 1000 | +0.064911 / +0.031494 | -0.015169 / +0.049814 |
| `lazy-map-realize-small` | 1000 | +0.026890 / -0.049062 | +0.067293 / +0.004703 |
| `lazy-nestlist-realize-small` | 1000 | +0.031021 / -0.011538 | +0.048036 / +0.003337 |
| `lazy-part-small` | 1000 | +0.089283 / +0.044282 | +0.058660 / +0.001624 |
| `lazy-permutations-take-small` | 1000 | +0.264135 / +0.145968 | +0.239515 / +0.162709 |
| `lazy-range-normal-small` | 1000 | +0.092739 / +0.095078 | +0.050426 / +0.085511 |
| `lazy-range-take-small` | 1000 | +0.144804 / +0.081584 | +0.192039 / +0.128395 |
| `lazy-select-realize-small` | 1000 | +0.049625 / +0.020736 | +0.089426 / +0.038968 |
| `lazy-select-scan-small` | 1000 | +0.017900 / +0.000969 | +0.011288 / -0.001660 |
| `lazy-subsets-take-small` | 1000 | +0.254594 / +0.339721 | +0.273430 / +0.167172 |
| `lazy-table-construct-full` | 1000 | +0.318252 / +0.353512 | +0.234959 / +0.216223 |
| `lazy-table-construct-short` | 1000 | +0.328356 / +0.035446 | +0.233759 / +0.124770 |
| `lazy-table-realize-small` | 1000 | +0.047961 / +0.007296 | +0.094369 / +0.041139 |
| `lazy-transparent-adapters-small` | 1000 | +0.056297 / -0.044595 | -0.072393 / +0.039760 |
| `lazy-tuples-take-small` | 1000 | +0.174467 / +0.213632 | +0.250068 / +0.154171 |
| `replaceall-simple` | 1000 | +0.250399 / +0.054559 | +0.414750 / +0.235123 |


Additional control attribution used the exact full-N Scan, ReplaceRepeated and ReplaceAll workloads on base, incoming and candidate 01. For Scan, an unnamed libc routine (`0x00000000001b1220`) changed cost with unchanged calls: five calls/iteration from `Symbol::is_system_name` cost 107→128 self instructions, and one from cached stability cost 22→29. This locates 28 instructions/iteration in comparison callers; it does not identify the libc routine or prove an alignment cause. Other control diagnostics did not reproduce every original gate delta. No acceptance sample was replaced or waived. Commands, raw graphs, calibration and edges are under `.temp/brief-566/control-attribution/`.

### 3. Every measured candidate — changes, attribution, allocations and both instruction samples

Each candidate has a fresh target directory, default and allocation-count binaries, a source snapshot/manifest and distinct binary hashes. All 54 result checks preceded allocation measurement, then exactly two full hardware instruction samples. Each N and 2N ran in a separate process with stdin `/dev/null`, profiling flags cleared except `RUM_PROFILE_ALLOC=1` for allocation runs, and the instruction event pinned to CPU 2. No timing decision was made.

| Build | Default binary SHA-256 | Allocation binary SHA-256 |
| --- | --- | --- |
| `base` | `0da93337f3958c992c938f0df2ae18a12d52011c3e1418db61ddc091475cafea` | `3808e4f35c406d7101d7fcc81c40d6041aaddc8d49bc7579b04a149ca97a8a61` |
| `candidate-00` | `5882842b6cf1232e488c822a8f6a57bbfdcce7afd7212304e839bac0fce50669` | `6676fb5ec94274e45ea5aa46b18bdd144e23a0bb5ece159f2ab8a2406e87548a` |
| `candidate-01` | `da051dc4f82188be50757cd443bae8e072a43f9429298426b970cd15a82801b5` | `94665062adf63708c1d5c0f4b1f3c0dd71b5d94440a66764b54f59f89c4ea86e` |
| `candidate-02` | `4ecc03ec5c2e713aa9e9b352fcfd834b0c60a3bb65a4f36e254e6e8cdebb00c6` | `81e1d8d86ed78e9e4b8d2ca1d95e8f262a2054fe4a94d0e17a5e43c72f263279` |
| `candidate-03` | `feb037cc039d312f19e60785cb6681792b379a71371db8f90a34df6e9647d195` | `8ee232e9c8412699776aa04c3aa752a29e8b1bbf2ba765f95bc8ed25201a0fd2` |
| `candidate-04` | `723f50fe28ae369c394c9d56434a196d25a5bbfdd9067e9c15438f70e25fb283` | `71da30d6f7cc2b88fdef44af49ca477c4be7e6c176e8ff1892b0661d3c02923f` |
| `candidate-05` | `ad4a7a1f095e0d51658bf7065a68b9359bcf9339249c837a30b963501b9d6152` | `4d15cac980bec1b3f966d47808b47f2d10fb7030248b34fa799a17f9638ac8c7` |
| `candidate-06` | `74db20192d34707bd71a37486140004edb5ee032ea4d291cb88cdca2f4a32009` | `572222df9775a3237f4e6c5754cd793ac8ebb684e243326bd0bf73542f98a124` |
| `candidate-07` | `45c55f2fb0e68b14c5e602f7e236906215bb8ca234dfe1a27a1adb1fb43f44d9` | `04b4e3c3f4e9ceb3d4856603c35a117fb954890abfd45d632dd847581af22c8c` |
| `candidate-08` | `a35395d6a8b0192f676269221aa1f3dc2e134d7c9b7059c2160216d004bade90` | `c79d39dcb25de206e37517f41bdb29342c8a1b33d84a68bee641e6d11ded8e65` |
| `candidate-09` | `8e5666d27c9f491f9e6c8953d28b58edb12bfb9ff8c70b0a6ced73cb7d4fc240` | `22d75fba3de94377c2372709a0bb3ab37ec7a07c26bb0da9c09e150cea7ea27a` |

Hashes, source manifests, exact build commands and raw output are preserved under `.temp/brief-566/<build>/`. The fixed workload/check copies match `.temp/brief-563/perf/` byte for byte. All instruction figures below are `(count(2N)-count(N))/N`; displayed percentages are rounded, while regression decisions use the unrounded values and require both samples above +0.05%.

#### candidate-01 attribution against candidate-00

Move LazyTable evaluated-bound recovery to InputRequired; exact replay of brief-565 candidate-02 lazy.rs and table_continuation.rs.

The source parent for this isolated measurement is `candidate-00`; the no-new-regression decision compares with retained `candidate-00`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/table_continuation.rs` | 32, 37–43, 52–67, 106, 121 |
| `crates/rum-stdlib/src/lazy.rs` | 248, 255, 261, 268, 274, 281 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-table-construct-short` | `PROGRAM TOTALS` | 23754.187 | 23551.914 | -202.273 |
| `lazy-table-construct-short` | `<rum_core::expr::Expr as core::clone::Clone>::clone` | 224.000 | 110.000 | -114.000 |
| `lazy-table-construct-short` | `core::ptr::drop_in_place<rum_core::expr::Expr>` | 69.000 | 36.000 | -33.000 |
| `lazy-table-construct-short` | `core::ptr::drop_in_place<rum_core::expr::Atom>` | 102.000 | 72.000 | -30.000 |
| `lazy-table-construct-short` | `rum_core::expr::Symbol::system` | 310.625 | 286.024 | -24.601 |
| `lazy-table-construct-short` | `rum_stdlib::lazy::eval_lazy_table` | 143.000 | 141.000 | -2.000 |
| `lazy-table-construct-short` | `0x00000000001b1220` | 189.977 | 190.447 | +0.470 |
| `lazy-table-construct-short` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 267.931 | 268.376 | +0.445 |
| `lazy-table-construct-full` | `PROGRAM TOTALS` | 24291.081 | 24063.571 | -227.510 |
| `lazy-table-construct-full` | `<rum_core::expr::Expr as core::clone::Clone>::clone` | 300.000 | 186.000 | -114.000 |
| `lazy-table-construct-full` | `rum_core::expr::Symbol::system` | 359.750 | 311.314 | -48.436 |
| `lazy-table-construct-full` | `core::ptr::drop_in_place<rum_core::expr::Expr>` | 69.000 | 36.000 | -33.000 |
| `lazy-table-construct-full` | `core::ptr::drop_in_place<rum_core::expr::Atom>` | 102.000 | 72.000 | -30.000 |
| `lazy-table-construct-full` | `rum_stdlib::lazy::eval_lazy_table` | 145.000 | 143.000 | -2.000 |
| `lazy-table-construct-full` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 432.353 | 430.910 | -1.443 |
| `lazy-table-construct-full` | `rum_core::expr::intern_symbol_text` | 391.386 | 392.115 | +0.729 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-02 attribution against candidate-00

Classify a descriptor symbol once in each descriptor traversal, replacing repeated is_system_symbol_name tests with one context check and a name match.

The source parent for this isolated measurement is `candidate-00`; the no-new-regression decision compares with retained `candidate-00`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/descriptor.rs` | 5–6, 15–17, 19–148, after 149 (deletion), 175, 177–185 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-depth-drop-all-one-8` | `PROGRAM TOTALS` | 49761.599 | 46883.648 | -2877.951 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::Symbol::is_system_name` | 1855.000 | 216.000 | -1639.000 |
| `lazy-depth-drop-all-one-8` | `0x00000000001b1220` | 735.110 | 101.356 | -633.754 |
| `lazy-depth-drop-all-one-8` | `rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name` | 432.000 | 8.000 | -424.000 |
| `lazy-depth-drop-all-one-8` | `rum_stdlib::lazy::realize` | 907.000 | 601.000 | -306.000 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::Symbol::is_system` | 0.000 | 187.000 | +187.000 |
| `lazy-depth-drop-all-one-8` | `rum_stdlib::lazy::descriptor::push_stage` | 744.000 | 681.000 | -63.000 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::Symbol::system` | 72.575 | 74.372 | +1.797 |
| `lazy-depth-map-build-8` | `PROGRAM TOTALS` | 19433.959 | 18129.194 | -1304.765 |
| `lazy-depth-map-build-8` | `rum_core::expr::Symbol::is_system_name` | 1170.000 | 236.000 | -934.000 |
| `lazy-depth-map-build-8` | `0x00000000001b1220` | 534.066 | 229.857 | -304.209 |
| `lazy-depth-map-build-8` | `rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name` | 260.000 | 16.000 | -244.000 |
| `lazy-depth-map-build-8` | `rum_core::expr::Symbol::is_system` | 0.000 | 187.000 | +187.000 |
| `lazy-depth-map-build-8` | `rum_stdlib::lazy::realize` | 771.000 | 649.000 | -122.000 |
| `lazy-depth-map-build-8` | `rum_stdlib::lazy::descriptor::push_stage` | 604.000 | 716.000 | +112.000 |
| `lazy-depth-map-build-8` | `rum_core::expr::intern_symbol_text` | 0.479 | 2.592 | +2.113 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-03 attribution against candidate-00

Classify symbol kind/context once per descriptor traversal but preserve the original ordered if-name comparisons and arity branches.

The source parent for this isolated measurement is `candidate-00`; the no-new-regression decision compares with retained `candidate-00`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/descriptor.rs` | 5–6, 15–20, 29, 40, 50, 67, 76, 90, 104, 118, 132, 174–181, 186, 191 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-depth-drop-all-one-8` | `PROGRAM TOTALS` | 49761.599 | 46900.989 | -2860.610 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::Symbol::is_system_name` | 1855.000 | 216.000 | -1639.000 |
| `lazy-depth-drop-all-one-8` | `0x00000000001b1220` | 735.110 | 101.046 | -634.064 |
| `lazy-depth-drop-all-one-8` | `rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name` | 432.000 | 8.000 | -424.000 |
| `lazy-depth-drop-all-one-8` | `rum_stdlib::lazy::realize` | 907.000 | 601.000 | -306.000 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::Symbol::is_system` | 0.000 | 187.000 | +187.000 |
| `lazy-depth-drop-all-one-8` | `rum_stdlib::lazy::descriptor::push_stage` | 744.000 | 698.000 | -46.000 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::Symbol::system` | 72.575 | 74.696 | +2.121 |
| `lazy-depth-map-build-8` | `PROGRAM TOTALS` | 19433.959 | 18139.569 | -1294.390 |
| `lazy-depth-map-build-8` | `rum_core::expr::Symbol::is_system_name` | 1170.000 | 236.000 | -934.000 |
| `lazy-depth-map-build-8` | `0x00000000001b1220` | 534.066 | 229.935 | -304.131 |
| `lazy-depth-map-build-8` | `rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name` | 260.000 | 16.000 | -244.000 |
| `lazy-depth-map-build-8` | `rum_core::expr::Symbol::is_system` | 0.000 | 187.000 | +187.000 |
| `lazy-depth-map-build-8` | `rum_stdlib::lazy::realize` | 771.000 | 649.000 | -122.000 |
| `lazy-depth-map-build-8` | `rum_stdlib::lazy::descriptor::push_stage` | 604.000 | 719.000 | +115.000 |
| `lazy-depth-map-build-8` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 764.326 | 773.699 | +9.373 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-04 attribution against candidate-02

Fuse output iteration into the indexed machine so the resume cursor is decoded once per realization. The unchanged realization entry delegates once to the same engine for ordinary and resumed work.

The source parent for this isolated measurement is `candidate-02`; the no-new-regression decision compares with retained `candidate-00`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/machine.rs` | 1–2, 8, 12–13, 15–18, 130–134, 138, 144 |
| `crates/rum-stdlib/src/lazy/realization.rs` | 1, 4, 16, 18 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-range-take-small` | `PROGRAM TOTALS` | 42771.587 | 42392.203 | -379.384 |
| `lazy-range-take-small` | `rum_stdlib::lazy::realization::drive_items` | 1329.000 | 987.000 | -342.000 |
| `lazy-range-take-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 80.000 | 10.000 | -70.000 |
| `lazy-range-take-small` | `rum_core::expr::Symbol::system` | 192.030 | 223.136 | +31.106 |
| `lazy-range-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 233.872 | 234.645 | +0.773 |
| `lazy-range-take-small` | `rum_core::expr::intern_symbol_text` | 194.942 | 195.479 | +0.537 |
| `lazy-range-take-small` | `rum_core::expr::intern_text` | 381.706 | 382.218 | +0.512 |
| `lazy-range-take-small` | `0x00000000001b1220` | 306.043 | 305.697 | -0.346 |
| `lazy-range-normal-small` | `PROGRAM TOTALS` | 54747.315 | 54180.230 | -567.085 |
| `lazy-range-normal-small` | `rum_stdlib::lazy::realization::drive_items` | 1815.000 | 1375.000 | -440.000 |
| `lazy-range-normal-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 100.000 | 10.000 | -90.000 |
| `lazy-range-normal-small` | `rum_core::expr::Symbol::system` | 310.808 | 255.826 | -54.982 |
| `lazy-range-normal-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 1128.935 | 1146.303 | +17.368 |
| `lazy-range-normal-small` | `rum_core::expr::intern_symbol_text` | 391.573 | 392.555 | +0.982 |
| `lazy-range-normal-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 368.610 | 368.276 | -0.334 |
| `lazy-range-normal-small` | `0x00000000001b1220` | 357.156 | 356.987 | -0.169 |
| `lazy-foldlist-realize-small` | `PROGRAM TOTALS` | 113805.085 | 113045.036 | -760.049 |
| `lazy-foldlist-realize-small` | `rum_stdlib::lazy::realization::drive_items` | 2582.000 | 2124.000 | -458.000 |
| `lazy-foldlist-realize-small` | `rum_core::expr::Symbol::system` | 1623.701 | 1479.177 | -144.524 |
| `lazy-foldlist-realize-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 100.000 | 10.000 | -90.000 |
| `lazy-foldlist-realize-small` | `rum_core::expr::intern_text` | 1184.126 | 1146.400 | -37.726 |
| `lazy-foldlist-realize-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 2972.039 | 2940.371 | -31.668 |
| `lazy-foldlist-realize-small` | `rum_core::expr::intern_symbol_text` | 587.431 | 588.675 | +1.244 |
| `lazy-foldlist-realize-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 506.130 | 506.843 | +0.713 |
| `lazy-depth-drop-all-one-8` | `rum_stdlib::lazy::realization::drive_items` | 2781.000 | 2596.000 | -185.000 |
| `lazy-depth-drop-all-one-8` | `PROGRAM TOTALS` | 46883.648 | 46699.380 | -184.268 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::Symbol::system` | 74.372 | 76.385 | +2.013 |
| `lazy-depth-drop-all-one-8` | `hashbrown::map::HashMap<K,V,S,A>::insert` | -0.157 | -0.736 | -0.579 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::intern_symbol_text` | -0.653 | -0.935 | -0.282 |
| `lazy-depth-drop-all-one-8` | `rum_core::expr::intern_text` | -0.051 | -0.330 | -0.279 |
| `lazy-depth-drop-all-one-8` | `0x00000000001b1220` | 101.356 | 101.133 | -0.223 |
| `lazy-depth-drop-all-one-8` | `std::sync::poison::once::Once::call_once_force::{{closure}}` | -0.055 | 0.035 | +0.090 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-05 attribution against candidate-04

Keep normalized LazyTable bounds as optional borrows until evaluation; create each owned expression once at evaluate, and reconstruct owned raw bounds only on the error edge for the unchanged suspension helper.

The source parent for this isolated measurement is `candidate-04`; the no-new-regression decision compares with retained `candidate-04`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy.rs` | 239–241, 248, 255–259, 265, 272–276, 282, 289 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-table-construct-short` | `PROGRAM TOTALS` | 23731.152 | 23509.858 | -221.294 |
| `lazy-table-construct-short` | `<rum_core::expr::Expr as core::clone::Clone>::clone` | 224.000 | 110.000 | -114.000 |
| `lazy-table-construct-short` | `core::ptr::drop_in_place<rum_core::expr::Expr>` | 69.000 | 36.000 | -33.000 |
| `lazy-table-construct-short` | `core::ptr::drop_in_place<rum_core::expr::Atom>` | 102.000 | 72.000 | -30.000 |
| `lazy-table-construct-short` | `rum_core::expr::intern_text` | 776.389 | 747.902 | -28.487 |
| `lazy-table-construct-short` | `rum_core::expr::Symbol::system` | 286.896 | 311.150 | +24.254 |
| `lazy-table-construct-short` | `0x00000000001b1220` | 189.914 | 168.935 | -20.979 |
| `lazy-table-construct-short` | `rum_stdlib::lazy::eval_lazy_table` | 143.000 | 123.000 | -20.000 |
| `lazy-table-construct-full` | `PROGRAM TOTALS` | 24293.465 | 24071.504 | -221.961 |
| `lazy-table-construct-full` | `<rum_core::expr::Expr as core::clone::Clone>::clone` | 300.000 | 186.000 | -114.000 |
| `lazy-table-construct-full` | `rum_core::expr::Symbol::system` | 359.486 | 311.484 | -48.002 |
| `lazy-table-construct-full` | `core::ptr::drop_in_place<rum_core::expr::Expr>` | 69.000 | 36.000 | -33.000 |
| `lazy-table-construct-full` | `core::ptr::drop_in_place<rum_core::expr::Atom>` | 102.000 | 72.000 | -30.000 |
| `lazy-table-construct-full` | `rum_stdlib::lazy::eval_lazy_table` | 145.000 | 147.000 | +2.000 |
| `lazy-table-construct-full` | `rum_core::expr::intern_symbol_text` | 393.121 | 392.047 | -1.074 |
| `lazy-table-construct-full` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 268.160 | 269.188 | +1.028 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-06 attribution against candidate-05

Decode DriveCursor on the resumed entry only; ordinary realization supplies a plain starting index and optional carried value directly to the same indexed output loop.

The source parent for this isolated measurement is `candidate-05`; the no-new-regression decision compares with retained `candidate-04`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/machine.rs` | 4, 14–15, after 19 (deletion), 132 |
| `crates/rum-stdlib/src/lazy/realization.rs` | 1, 4, 18–28 |
| `crates/rum-stdlib/src/lazy.rs` | 446 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-range-take-small` | `rum_stdlib::lazy::realization::drive_items` | 987.000 | 0.000 | -987.000 |
| `lazy-range-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 0.000 | 968.000 | +968.000 |
| `lazy-range-take-small` | `rum_core::expr::intern_symbol_text` | 195.774 | 222.905 | +27.131 |
| `lazy-range-take-small` | `PROGRAM TOTALS` | 42333.898 | 42352.442 | +18.544 |
| `lazy-range-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 803.803 | 812.682 | +8.879 |
| `lazy-range-take-small` | `rum_stdlib::lazy::realize` | 211.000 | 212.000 | +1.000 |
| `lazy-range-take-small` | `rum_core::expr::Symbol::system` | 193.465 | 192.947 | -0.518 |
| `lazy-range-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 233.933 | 234.416 | +0.483 |
| `lazy-range-normal-small` | `rum_stdlib::lazy::machine::drive_machine` | 0.000 | 1381.000 | +1381.000 |
| `lazy-range-normal-small` | `rum_stdlib::lazy::realization::drive_items` | 1375.000 | 0.000 | -1375.000 |
| `lazy-range-normal-small` | `PROGRAM TOTALS` | 54209.566 | 54291.399 | +81.833 |
| `lazy-range-normal-small` | `rum_core::expr::Symbol::system` | 311.512 | 358.939 | +47.427 |
| `lazy-range-normal-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 1118.405 | 1146.810 | +28.405 |
| `lazy-range-normal-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 370.098 | 368.963 | -1.135 |
| `lazy-range-normal-small` | `rum_stdlib::lazy::realize` | 262.000 | 263.000 | +1.000 |
| `lazy-range-normal-small` | `rum_core::expr::intern_symbol_text` | 391.266 | 392.142 | +0.876 |
| `lazy-subsets-take-small` | `rum_stdlib::lazy::realization::drive_items` | 613.000 | 0.000 | -613.000 |
| `lazy-subsets-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 0.000 | 613.000 | +613.000 |
| `lazy-subsets-take-small` | `rum_core::expr::Symbol::system` | 393.555 | 418.165 | +24.610 |
| `lazy-subsets-take-small` | `rum_core::expr::intern_text` | 381.589 | 366.572 | -15.017 |
| `lazy-subsets-take-small` | `PROGRAM TOTALS` | 25970.832 | 25978.977 | +8.145 |
| `lazy-subsets-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 930.330 | 927.652 | -2.678 |
| `lazy-subsets-take-small` | `rum_stdlib::lazy::realize` | 265.000 | 266.000 | +1.000 |
| `lazy-subsets-take-small` | `rum_core::expr::intern_symbol_text` | 196.133 | 195.759 | -0.374 |
| `lazy-permutations-take-small` | `rum_stdlib::lazy::realization::drive_items` | 364.000 | 0.000 | -364.000 |
| `lazy-permutations-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 0.000 | 359.000 | +359.000 |
| `lazy-permutations-take-small` | `PROGRAM TOTALS` | 28801.658 | 28830.040 | +28.382 |
| `lazy-permutations-take-small` | `rum_core::expr::Symbol::system` | 401.364 | 419.041 | +17.677 |
| `lazy-permutations-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 905.258 | 913.539 | +8.281 |
| `lazy-permutations-take-small` | `0x00000000001b1220` | 524.262 | 529.956 | +5.694 |
| `lazy-permutations-take-small` | `rum_core::expr::intern_symbol_text` | 195.767 | 197.084 | +1.317 |
| `lazy-permutations-take-small` | `rum_stdlib::lazy::realize` | 274.000 | 275.000 | +1.000 |
| `lazy-map-realize-small` | `rum_stdlib::lazy::machine::drive_machine` | 0.000 | 2021.000 | +2021.000 |
| `lazy-map-realize-small` | `rum_stdlib::lazy::realization::drive_items` | 2007.000 | 0.000 | -2007.000 |
| `lazy-map-realize-small` | `rum_core::expr::Symbol::system` | 1654.842 | 1605.586 | -49.256 |
| `lazy-map-realize-small` | `PROGRAM TOTALS` | 179200.414 | 179156.493 | -43.921 |
| `lazy-map-realize-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 5700.752 | 5691.548 | -9.204 |
| `lazy-map-realize-small` | `rum_core::expr::intern_symbol_text` | 588.322 | 587.248 | -1.074 |
| `lazy-map-realize-small` | `rum_stdlib::lazy::realize` | 316.000 | 317.000 | +1.000 |
| `lazy-map-realize-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 1207.489 | 1207.942 | +0.453 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-07 attribution against candidate-06

Move the optional carried value directly out of the entry slot, then reset the slot after descent; eliminate Option::take and its redundant empty-value cleanup.

The source parent for this isolated measurement is `candidate-06`; the no-new-regression decision compares with retained `candidate-06`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/machine.rs` | 23, 53 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-range-take-small` | `rum_core::expr::intern_symbol_text` | 222.905 | 194.892 | -28.013 |
| `lazy-range-take-small` | `PROGRAM TOTALS` | 42352.442 | 42327.276 | -25.166 |
| `lazy-range-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 968.000 | 988.000 | +20.000 |
| `lazy-range-take-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 10.000 | 0.000 | -10.000 |
| `lazy-range-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 812.682 | 803.751 | -8.931 |
| `lazy-range-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 234.416 | 235.486 | +1.070 |
| `lazy-range-take-small` | `rum_core::expr::intern_text` | 381.805 | 382.477 | +0.672 |
| `lazy-range-take-small` | `rum_core::expr::Symbol::system` | 192.947 | 193.391 | +0.444 |
| `lazy-range-normal-small` | `PROGRAM TOTALS` | 54291.399 | 54230.128 | -61.271 |
| `lazy-range-normal-small` | `rum_core::expr::Symbol::system` | 358.939 | 311.745 | -47.194 |
| `lazy-range-normal-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 1146.810 | 1125.490 | -21.320 |
| `lazy-range-normal-small` | `rum_stdlib::lazy::machine::drive_machine` | 1381.000 | 1399.000 | +18.000 |
| `lazy-range-normal-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 10.000 | 0.000 | -10.000 |
| `lazy-range-normal-small` | `rum_core::expr::intern_symbol_text` | 392.142 | 391.729 | -0.413 |
| `lazy-range-normal-small` | `0x00000000001b1220` | 356.978 | 356.803 | -0.175 |
| `lazy-range-normal-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 368.963 | 368.810 | -0.153 |
| `lazy-map-realize-small` | `PROGRAM TOTALS` | 179156.493 | 179183.894 | +27.401 |
| `lazy-map-realize-small` | `rum_core::expr::Symbol::system` | 1605.586 | 1629.402 | +23.816 |
| `lazy-map-realize-small` | `rum_stdlib::lazy::machine::drive_machine` | 2021.000 | 2039.000 | +18.000 |
| `lazy-map-realize-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 10.000 | 0.000 | -10.000 |
| `lazy-map-realize-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 1207.942 | 1205.288 | -2.654 |
| `lazy-map-realize-small` | `rum_core::expr::intern_text` | 1159.131 | 1157.513 | -1.618 |
| `lazy-map-realize-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 5691.548 | 5690.118 | -1.430 |
| `lazy-map-realize-small` | `rum_core::expr::intern_symbol_text` | 587.248 | 588.268 | +1.020 |
| `lazy-permutations-take-small` | `PROGRAM TOTALS` | 28830.040 | 28809.662 | -20.378 |
| `lazy-permutations-take-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 10.000 | 0.000 | -10.000 |
| `lazy-permutations-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 913.539 | 905.674 | -7.865 |
| `lazy-permutations-take-small` | `rum_core::expr::Symbol::system` | 419.041 | 417.788 | -1.253 |
| `lazy-permutations-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 359.000 | 360.000 | +1.000 |
| `lazy-permutations-take-small` | `rum_core::expr::intern_symbol_text` | 197.084 | 196.287 | -0.797 |
| `lazy-permutations-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 659.311 | 658.571 | -0.740 |
| `lazy-permutations-take-small` | `rum_core::expr::intern_text` | 382.145 | 381.533 | -0.612 |
| `lazy-subsets-take-small` | `rum_core::expr::intern_text` | 366.572 | 381.377 | +14.805 |
| `lazy-subsets-take-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 10.000 | 0.000 | -10.000 |
| `lazy-subsets-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 927.652 | 923.336 | -4.316 |
| `lazy-subsets-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 613.000 | 615.000 | +2.000 |
| `lazy-subsets-take-small` | `PROGRAM TOTALS` | 25978.977 | 25980.971 | +1.994 |
| `lazy-subsets-take-small` | `rum_core::expr::intern_symbol_text` | 195.759 | 196.942 | +1.183 |
| `lazy-subsets-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 656.860 | 655.848 | -1.012 |
| `lazy-subsets-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2` | 1284.727 | 1284.363 | -0.364 |
| `lazy-select-realize-small` | `PROGRAM TOTALS` | 114650.765 | 114786.264 | +135.499 |
| `lazy-select-realize-small` | `rum_core::expr::Symbol::system` | 2174.740 | 2229.029 | +54.289 |
| `lazy-select-realize-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 3391.754 | 3441.557 | +49.803 |
| `lazy-select-realize-small` | `0x00000000001b1220` | 930.874 | 951.802 | +20.928 |
| `lazy-select-realize-small` | `rum_stdlib::lazy::machine::drive_machine` | 1818.000 | 1836.000 | +18.000 |
| `lazy-select-realize-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 10.000 | 0.000 | -10.000 |
| `lazy-select-realize-small` | `rum_core::expr::intern_symbol_text` | 586.136 | 587.845 | +1.709 |
| `lazy-select-realize-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 503.086 | 503.718 | +0.632 |
| `lazy-select-scan-small` | `PROGRAM TOTALS` | 806224.814 | 806317.292 | +92.478 |
| `lazy-select-scan-small` | `rum_stdlib::lazy::machine::drive_machine` | 5893.000 | 5961.000 | +68.000 |
| `lazy-select-scan-small` | `rum_core::expr::Symbol::system` | 7511.405 | 7541.201 | +29.796 |
| `lazy-select-scan-small` | `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 10.000 | 0.000 | -10.000 |
| `lazy-select-scan-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 27274.118 | 27277.901 | +3.783 |
| `lazy-select-scan-small` | `rum_core::expr::intern_symbol_text` | 391.486 | 392.877 | +1.391 |
| `lazy-select-scan-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 8063.216 | 8062.996 | -0.220 |
| `lazy-select-scan-small` | `rum_core::expr::intern_text` | 763.808 | 763.661 | -0.147 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-08 attribution against candidate-06

Normalize the output budget to one numeric limit at realization entry, replacing the per-output enum match with a length comparison.

The source parent for this isolated measurement is `candidate-06`; the no-new-regression decision compares with retained `candidate-06`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/machine.rs` | 17–22, 130 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-range-take-small` | `rum_core::expr::intern_symbol_text` | 222.905 | 195.267 | -27.638 |
| `lazy-range-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 968.000 | 986.000 | +18.000 |
| `lazy-range-take-small` | `PROGRAM TOTALS` | 42352.442 | 42342.652 | -9.790 |
| `lazy-range-take-small` | `rum_core::expr::Symbol::system` | 192.947 | 193.311 | +0.364 |
| `lazy-range-take-small` | `std::sync::poison::once::Once::call_once_force::{{closure}}` | 0.216 | -0.024 | -0.240 |
| `lazy-range-take-small` | `0x00000000001b1220` | 305.935 | 305.759 | -0.176 |
| `lazy-range-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 234.416 | 234.246 | -0.170 |
| `lazy-range-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 812.682 | 812.760 | +0.078 |
| `lazy-range-normal-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 1146.810 | 1117.612 | -29.198 |
| `lazy-range-normal-small` | `PROGRAM TOTALS` | 54291.399 | 54262.596 | -28.803 |
| `lazy-range-normal-small` | `rum_stdlib::lazy::machine::drive_machine` | 1381.000 | 1400.000 | +19.000 |
| `lazy-range-normal-small` | `rum_core::expr::Symbol::system` | 358.939 | 343.001 | -15.938 |
| `lazy-range-normal-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 368.963 | 366.895 | -2.068 |
| `lazy-range-normal-small` | `rum_core::expr::intern_text` | 764.013 | 762.920 | -1.093 |
| `lazy-range-normal-small` | `rum_core::expr::intern_symbol_text` | 392.142 | 392.502 | +0.360 |
| `lazy-range-normal-small` | `std::sync::poison::once::Once::call_once_force::{{closure}}` | -0.042 | 0.122 | +0.164 |
| `lazy-select-scan-small` | `rum_stdlib::lazy::machine::drive_machine` | 5893.000 | 6011.000 | +118.000 |
| `lazy-select-scan-small` | `PROGRAM TOTALS` | 806224.814 | 806338.280 | +113.466 |
| `lazy-select-scan-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 27274.118 | 27268.788 | -5.330 |
| `lazy-select-scan-small` | `rum_core::expr::intern_symbol_text` | 391.486 | 392.447 | +0.961 |
| `lazy-select-scan-small` | `rum_core::expr::Symbol::system` | 7511.405 | 7510.594 | -0.811 |
| `lazy-select-scan-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 8063.216 | 8063.828 | +0.612 |
| `lazy-select-scan-small` | `rum_core::expr::intern_text` | 763.808 | 764.097 | +0.289 |
| `lazy-select-scan-small` | `0x00000000001b1220` | 9075.152 | 9074.954 | -0.198 |
| `lazy-permutations-take-small` | `PROGRAM TOTALS` | 28830.040 | 28827.154 | -2.886 |
| `lazy-permutations-take-small` | `rum_core::expr::Symbol::system` | 419.041 | 417.483 | -1.558 |
| `lazy-permutations-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 359.000 | 358.000 | -1.000 |
| `lazy-permutations-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 913.539 | 914.332 | +0.793 |
| `lazy-permutations-take-small` | `rum_core::expr::intern_symbol_text` | 197.084 | 196.436 | -0.648 |
| `lazy-permutations-take-small` | `rum_core::expr::intern_text` | 382.145 | 381.717 | -0.428 |
| `lazy-permutations-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 659.311 | 658.985 | -0.326 |
| `lazy-permutations-take-small` | `std::sync::poison::once::Once::call_once_force::{{closure}}` | -0.111 | 0.079 | +0.190 |
| `lazy-subsets-take-small` | `PROGRAM TOTALS` | 25978.977 | 25995.230 | +16.253 |
| `lazy-subsets-take-small` | `rum_core::expr::intern_text` | 366.572 | 382.002 | +15.430 |
| `lazy-subsets-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 613.000 | 610.000 | -3.000 |
| `lazy-subsets-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 927.652 | 930.434 | +2.782 |
| `lazy-subsets-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 656.860 | 657.479 | +0.619 |
| `lazy-subsets-take-small` | `rum_core::expr::Symbol::system` | 418.165 | 418.277 | +0.112 |
| `lazy-subsets-take-small` | `std::sync::poison::once::Once::call_once_force::{{closure}}` | 0.036 | 0.141 | +0.105 |
| `lazy-subsets-take-small` | `0x00000000001b1220` | 546.065 | 546.148 | +0.083 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

#### candidate-09 attribution against candidate-06

Borrow the source and the fixed stage slice once per realization, so the indexed driver carries a stable slice pointer/length across pulls and callbacks instead of revisiting Vec metadata.

The source parent for this isolated measurement is `candidate-06`; the no-new-regression decision compares with retained `candidate-06`.

| Changed source file | Candidate line ranges |
| --- | --- |
| `crates/rum-stdlib/src/lazy/machine.rs` | 20–21, 28–29, 31, 34, 37, 58, 117, 134–135 |
| `crates/rum-stdlib/src/lazy/realization.rs` | 25 |

| Label | Function (self cost) | Source parent | Candidate | Δ instructions/iteration |
| --- | --- | ---: | ---: | ---: |
| `lazy-range-take-small` | `rum_core::expr::intern_symbol_text` | 222.905 | 196.005 | -26.900 |
| `lazy-range-take-small` | `PROGRAM TOTALS` | 42352.442 | 42340.986 | -11.456 |
| `lazy-range-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 812.682 | 823.134 | +10.452 |
| `lazy-range-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 968.000 | 972.000 | +4.000 |
| `lazy-range-take-small` | `rum_core::expr::Symbol::system` | 192.947 | 193.577 | +0.630 |
| `lazy-range-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 234.416 | 234.881 | +0.465 |
| `lazy-range-take-small` | `rum_core::expr::intern_text` | 381.805 | 382.043 | +0.238 |
| `lazy-range-take-small` | `0x00000000001b1220` | 305.935 | 306.155 | +0.220 |
| `lazy-range-normal-small` | `PROGRAM TOTALS` | 54291.399 | 54225.407 | -65.992 |
| `lazy-range-normal-small` | `rum_stdlib::lazy::machine::drive_machine` | 1381.000 | 1358.000 | -23.000 |
| `lazy-range-normal-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 1146.810 | 1124.801 | -22.009 |
| `lazy-range-normal-small` | `rum_core::expr::Symbol::system` | 358.939 | 341.862 | -17.077 |
| `lazy-range-normal-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 368.963 | 366.729 | -2.234 |
| `lazy-range-normal-small` | `rum_core::expr::intern_text` | 764.013 | 762.647 | -1.366 |
| `lazy-range-normal-small` | `rum_core::expr::intern_symbol_text` | 392.142 | 391.968 | -0.174 |
| `lazy-range-normal-small` | `0x00000000001b1220` | 356.978 | 356.823 | -0.155 |
| `lazy-select-realize-small` | `rum_core::expr::intern_symbol_text` | 586.136 | 549.328 | -36.808 |
| `lazy-select-realize-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 3391.754 | 3428.128 | +36.374 |
| `lazy-select-realize-small` | `rum_stdlib::lazy::machine::drive_machine` | 1818.000 | 1795.000 | -23.000 |
| `lazy-select-realize-small` | `0x00000000001b1220` | 930.874 | 952.066 | +21.192 |
| `lazy-select-realize-small` | `PROGRAM TOTALS` | 114650.765 | 114631.541 | -19.224 |
| `lazy-select-realize-small` | `rum_core::expr::Symbol::system` | 2174.740 | 2156.581 | -18.159 |
| `lazy-select-realize-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 503.086 | 503.948 | +0.862 |
| `lazy-select-realize-small` | `rum_core::expr::intern_text` | 1145.951 | 1146.172 | +0.221 |
| `lazy-select-scan-small` | `rum_core::expr::Symbol::system` | 7511.405 | 7558.938 | +47.533 |
| `lazy-select-scan-small` | `PROGRAM TOTALS` | 806224.814 | 806249.654 | +24.840 |
| `lazy-select-scan-small` | `rum_stdlib::lazy::machine::drive_machine` | 5893.000 | 5873.000 | -20.000 |
| `lazy-select-scan-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 27274.118 | 27270.452 | -3.666 |
| `lazy-select-scan-small` | `rum_core::expr::intern_symbol_text` | 391.486 | 392.131 | +0.645 |
| `lazy-select-scan-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 8063.216 | 8063.765 | +0.549 |
| `lazy-select-scan-small` | `rum_core::expr::intern_text` | 763.808 | 764.300 | +0.492 |
| `lazy-select-scan-small` | `0x00000000001b1220` | 9075.152 | 9074.715 | -0.437 |
| `lazy-permutations-take-small` | `PROGRAM TOTALS` | 28830.040 | 28804.146 | -25.894 |
| `lazy-permutations-take-small` | `rum_core::expr::intern_text` | 382.145 | 361.992 | -20.153 |
| `lazy-permutations-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 359.000 | 357.000 | -2.000 |
| `lazy-permutations-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 659.311 | 657.448 | -1.863 |
| `lazy-permutations-take-small` | `rum_core::expr::intern_symbol_text` | 197.084 | 195.747 | -1.337 |
| `lazy-permutations-take-small` | `rum_core::expr::Symbol::system` | 419.041 | 418.526 | -0.515 |
| `lazy-permutations-take-small` | `std::sync::poison::once::Once::call_once_force::{{closure}}` | -0.111 | 0.129 | +0.240 |
| `lazy-permutations-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2` | 1776.560 | 1776.339 | -0.221 |
| `lazy-subsets-take-small` | `rum_core::expr::intern_text` | 366.572 | 382.136 | +15.564 |
| `lazy-subsets-take-small` | `PROGRAM TOTALS` | 25978.977 | 25991.752 | +12.775 |
| `lazy-subsets-take-small` | `rum_stdlib::lazy::machine::drive_machine` | 613.000 | 608.000 | -5.000 |
| `lazy-subsets-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 927.652 | 930.109 | +2.457 |
| `lazy-subsets-take-small` | `rum_core::expr::Symbol::system` | 418.165 | 417.201 | -0.964 |
| `lazy-subsets-take-small` | `hashbrown::map::HashMap<K,V,S,A>::insert` | 656.860 | 657.710 | +0.850 |
| `lazy-subsets-take-small` | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2` | 1284.727 | 1284.948 | +0.221 |
| `lazy-subsets-take-small` | `rum_core::expr::intern_symbol_text` | 195.759 | 195.579 | -0.180 |

These self-cost buckets include compiler inlining/movement; program totals and hardware calibration are retained separately, and the acceptance samples remain authoritative.

Candidate diagnostic calibration, each total against hardware on that same binary:

| Binary | Label | N | Callgrind total | Hardware total | Difference % |
| --- | --- | ---: | ---: | ---: | ---: |
| `candidate-01` | `lazy-table-construct-full` | 1000 | 27658726 | 27619129 | +0.143368 |
| `candidate-01` | `lazy-table-construct-full` | 2000 | 51722297 | 51690945 | +0.060653 |
| `candidate-01` | `lazy-table-construct-short` | 1000 | 27191052 | 27130627 | +0.222719 |
| `candidate-01` | `lazy-table-construct-short` | 2000 | 50742966 | 50727358 | +0.030768 |
| `candidate-02` | `lazy-depth-drop-all-one-8` | 1000 | 50777212 | 50690513 | +0.171036 |
| `candidate-02` | `lazy-depth-drop-all-one-8` | 2000 | 97660860 | 97548316 | +0.115373 |
| `candidate-02` | `lazy-depth-map-build-8` | 1000 | 22064973 | 21974043 | +0.413806 |
| `candidate-02` | `lazy-depth-map-build-8` | 2000 | 40194167 | 40083082 | +0.277137 |
| `candidate-02` | `lazy-foldlist-realize-small` | 1000 | 117144125 | 117110467 | +0.028740 |
| `candidate-02` | `lazy-foldlist-realize-small` | 2000 | 230949210 | 230805663 | +0.062194 |
| `candidate-02` | `lazy-range-normal-small` | 1000 | 58339365 | 58268965 | +0.120819 |
| `candidate-02` | `lazy-range-normal-small` | 2000 | 113086680 | 112998450 | +0.078081 |
| `candidate-02` | `lazy-range-take-small` | 1000 | 46342860 | 46264714 | +0.168911 |
| `candidate-02` | `lazy-range-take-small` | 2000 | 89114447 | 89000159 | +0.128413 |
| `candidate-03` | `lazy-depth-drop-all-one-8` | 1000 | 50793631 | 50704378 | +0.176026 |
| `candidate-03` | `lazy-depth-drop-all-one-8` | 2000 | 97694620 | 97585296 | +0.112029 |
| `candidate-03` | `lazy-depth-map-build-8` | 1000 | 22066730 | 21978132 | +0.403119 |
| `candidate-03` | `lazy-depth-map-build-8` | 2000 | 40206299 | 40072341 | +0.334290 |
| `candidate-04` | `lazy-depth-drop-all-one-8` | 1000 | 50591386 | 50504257 | +0.172518 |
| `candidate-04` | `lazy-depth-drop-all-one-8` | 2000 | 97290766 | 97180447 | +0.113520 |
| `candidate-04` | `lazy-foldlist-realize-small` | 1000 | 116591687 | 116472005 | +0.102756 |
| `candidate-04` | `lazy-foldlist-realize-small` | 2000 | 229636723 | 229525260 | +0.048562 |
| `candidate-04` | `lazy-range-normal-small` | 1000 | 57854816 | 57785710 | +0.119590 |
| `candidate-04` | `lazy-range-normal-small` | 2000 | 112035046 | 112011182 | +0.021305 |
| `candidate-04` | `lazy-range-take-small` | 1000 | 45930108 | 45854565 | +0.164745 |
| `candidate-04` | `lazy-range-take-small` | 2000 | 88322311 | 88206542 | +0.131248 |
| `candidate-04` | `lazy-table-construct-full` | 1000 | 27838185 | 27798227 | +0.143743 |
| `candidate-04` | `lazy-table-construct-full` | 2000 | 52131650 | 52019795 | +0.215024 |
| `candidate-04` | `lazy-table-construct-short` | 1000 | 27371954 | 27281728 | +0.330720 |
| `candidate-04` | `lazy-table-construct-short` | 2000 | 51103106 | 51037097 | +0.129335 |
| `candidate-05` | `lazy-map-realize-small` | 1000 | 182552833 | 182432524 | +0.065947 |
| `candidate-05` | `lazy-map-realize-small` | 2000 | 361753247 | 361577021 | +0.048738 |
| `candidate-05` | `lazy-permutations-take-small` | 1000 | 32232154 | 32136138 | +0.298779 |
| `candidate-05` | `lazy-permutations-take-small` | 2000 | 61033812 | 60951498 | +0.135048 |
| `candidate-05` | `lazy-range-normal-small` | 1000 | 57798360 | 57735644 | +0.108626 |
| `candidate-05` | `lazy-range-normal-small` | 2000 | 112007926 | 111958747 | +0.043926 |
| `candidate-05` | `lazy-range-take-small` | 1000 | 45938002 | 45858790 | +0.172730 |
| `candidate-05` | `lazy-range-take-small` | 2000 | 88271900 | 88223082 | +0.055335 |
| `candidate-05` | `lazy-subsets-take-small` | 1000 | 29410148 | 29308457 | +0.346968 |
| `candidate-05` | `lazy-subsets-take-small` | 2000 | 55380980 | 55284205 | +0.175050 |
| `candidate-05` | `lazy-table-construct-full` | 1000 | 27661770 | 27602073 | +0.216277 |
| `candidate-05` | `lazy-table-construct-full` | 2000 | 51733274 | 51667097 | +0.128083 |
| `candidate-05` | `lazy-table-construct-short` | 1000 | 27198608 | 27108420 | +0.332694 |
| `candidate-05` | `lazy-table-construct-short` | 2000 | 50708466 | 50768762 | -0.118766 |
| `candidate-06` | `lazy-map-realize-small` | 1000 | 182569972 | 182446344 | +0.067761 |
| `candidate-06` | `lazy-map-realize-small` | 2000 | 361726465 | 361619736 | +0.029514 |
| `candidate-06` | `lazy-permutations-take-small` | 1000 | 32215809 | 32160536 | +0.171866 |
| `candidate-06` | `lazy-permutations-take-small` | 2000 | 61045849 | 60947973 | +0.160589 |
| `candidate-06` | `lazy-range-normal-small` | 1000 | 57797068 | 57733692 | +0.109773 |
| `candidate-06` | `lazy-range-normal-small` | 2000 | 112088467 | 112019720 | +0.061370 |
| `candidate-06` | `lazy-range-take-small` | 1000 | 45910138 | 45844615 | +0.142924 |
| `candidate-06` | `lazy-range-take-small` | 2000 | 88262580 | 88139158 | +0.140031 |
| `candidate-06` | `lazy-select-realize-small` | 1000 | 118331352 | 118200199 | +0.110958 |
| `candidate-06` | `lazy-select-realize-small` | 2000 | 232982117 | 232875383 | +0.045833 |
| `candidate-06` | `lazy-select-scan-small` | 1000 | 807280484 | 807189219 | +0.011307 |
| `candidate-06` | `lazy-select-scan-small` | 2000 | 1613505298 | 1613546882 | -0.002577 |
| `candidate-06` | `lazy-subsets-take-small` | 1000 | 29404738 | 29311195 | +0.319137 |
| `candidate-06` | `lazy-subsets-take-small` | 2000 | 55383715 | 55287049 | +0.174844 |
| `candidate-07` | `lazy-map-realize-small` | 1000 | 182559135 | 182462168 | +0.053144 |
| `candidate-07` | `lazy-map-realize-small` | 2000 | 361743029 | 361716761 | +0.007262 |
| `candidate-07` | `lazy-permutations-take-small` | 1000 | 32207131 | 32134136 | +0.227157 |
| `candidate-07` | `lazy-permutations-take-small` | 2000 | 61016793 | 60926551 | +0.148116 |
| `candidate-07` | `lazy-range-normal-small` | 1000 | 57807826 | 57765442 | +0.073373 |
| `candidate-07` | `lazy-range-normal-small` | 2000 | 112037954 | 111979773 | +0.051957 |
| `candidate-07` | `lazy-range-take-small` | 1000 | 45929914 | 45897487 | +0.070651 |
| `candidate-07` | `lazy-range-take-small` | 2000 | 88257190 | 88187759 | +0.078731 |
| `candidate-07` | `lazy-select-realize-small` | 1000 | 118244486 | 118339269 | -0.080094 |
| `candidate-07` | `lazy-select-realize-small` | 2000 | 233030750 | 233002482 | +0.012132 |
| `candidate-07` | `lazy-select-scan-small` | 1000 | 807337405 | 807292573 | +0.005553 |
| `candidate-07` | `lazy-select-scan-small` | 2000 | 1613654697 | 1613628551 | +0.001620 |
| `candidate-07` | `lazy-subsets-take-small` | 1000 | 29383910 | 29306103 | +0.265498 |
| `candidate-07` | `lazy-subsets-take-small` | 2000 | 55364881 | 55320142 | +0.080873 |
| `candidate-08` | `lazy-permutations-take-small` | 1000 | 32214802 | 32155836 | +0.183376 |
| `candidate-08` | `lazy-permutations-take-small` | 2000 | 61041956 | 60951881 | +0.147781 |
| `candidate-08` | `lazy-range-normal-small` | 1000 | 57827405 | 57753170 | +0.128538 |
| `candidate-08` | `lazy-range-normal-small` | 2000 | 112090001 | 112059381 | +0.027325 |
| `candidate-08` | `lazy-range-take-small` | 1000 | 45929628 | 45864940 | +0.141040 |
| `candidate-08` | `lazy-range-take-small` | 2000 | 88272280 | 88174906 | +0.110433 |
| `candidate-08` | `lazy-select-scan-small` | 1000 | 807403529 | 807334324 | +0.008572 |
| `candidate-08` | `lazy-select-scan-small` | 2000 | 1613741809 | 1613756419 | -0.000905 |
| `candidate-08` | `lazy-subsets-take-small` | 1000 | 29383505 | 29310339 | +0.249625 |
| `candidate-08` | `lazy-subsets-take-small` | 2000 | 55378735 | 55279927 | +0.178741 |
| `candidate-09` | `lazy-permutations-take-small` | 1000 | 32233817 | 32137810 | +0.298735 |
| `candidate-09` | `lazy-permutations-take-small` | 2000 | 61037963 | 60949807 | +0.144637 |
| `candidate-09` | `lazy-range-normal-small` | 1000 | 57794643 | 57733042 | +0.106700 |
| `candidate-09` | `lazy-range-normal-small` | 2000 | 112020050 | 111907026 | +0.100998 |
| `candidate-09` | `lazy-range-take-small` | 1000 | 45922808 | 45851079 | +0.156439 |
| `candidate-09` | `lazy-range-take-small` | 2000 | 88263794 | 88196146 | +0.076702 |
| `candidate-09` | `lazy-select-realize-small` | 1000 | 118342419 | 118165518 | +0.149706 |
| `candidate-09` | `lazy-select-realize-small` | 2000 | 232973960 | 232826010 | +0.063545 |
| `candidate-09` | `lazy-select-scan-small` | 1000 | 807260705 | 807207997 | +0.006530 |
| `candidate-09` | `lazy-select-scan-small` | 2000 | 1613510359 | 1613480937 | +0.001824 |
| `candidate-09` | `lazy-subsets-take-small` | 1000 | 29381548 | 29309701 | +0.245130 |
| `candidate-09` | `lazy-subsets-take-small` | 2000 | 55373300 | 55282713 | +0.163861 |


Allocation slopes were measured before either instruction sample for each build. Equal row sets are grouped below; every measured candidate is listed in its group. Peak-live byte pairs remain in each candidate’s full JSON/TSV and raw allocation stderr.

Builds: `candidate-00`, `candidate-01`, `candidate-02`, `candidate-03`, `candidate-04`, `candidate-05`, `candidate-06`, `candidate-07`, `candidate-08`, `candidate-09`.

| Label | Base → candidate allocations/iteration | Base → candidate allocated bytes/iteration |
| --- | ---: | ---: |
| `lazy-range-take-small` | 141 → 141 | 2714 → 2714 |
| `lazy-range-normal-small` | 157 → 157 | 3444 → 3388 |
| `lazy-map-realize-small` | 333 → 333 | 13780 → 13740 |
| `lazy-select-realize-small` | 236 → 236 | 6874 → 6834 |
| `lazy-select-scan-small` | 1471 → 1471 | 62849 → 62793 |
| `lazy-nestlist-realize-small` | 197 → 197 | 11372 → 11316 |
| `lazy-foldlist-realize-small` | 216 → 216 | 7644 → 7604 |
| `lazy-drop-realize-small` | 352 → 351 | 14300 → 14204 |
| `lazy-first-small` | 64 → 64 | 2924 → 2868 |
| `lazy-part-small` | 152 → 152 | 6222 → 6182 |
| `lazy-transparent-adapters-small` | 322 → 319 | 13084 → 12876 |
| `lazy-table-construct-short` | 63 → 63 | 1537 → 1537 |
| `lazy-table-construct-full` | 64 → 64 | 1449 → 1449 |
| `lazy-table-realize-small` | 251 → 251 | 9230 → 9190 |
| `lazy-permutations-take-small` | 29 → 29 | 1729 → 1729 |
| `lazy-subsets-take-small` | 29 → 29 | 1417 → 1417 |
| `lazy-tuples-take-small` | 29 → 29 | 1361 → 1361 |
| `scan-no-input` | 33 → 33 | 2536 → 2536 |
| `optionvalue-three-no-input` | 9 → 9 | 1128 → 1128 |
| `replacerepeated-no-input` | 61 → 61 | 7691 → 7691 |
| `dot-three-no-input` | 596 → 596 | 23504 → 23504 |
| `differences-no-input` | 13 → 13 | 704 → 704 |
| `do-count-null` | 0 → 0 | 0 → 0 |
| `do-iterator-null` | 0 → 0 | 32 → 32 |
| `table-iterator-null` | 0 → 0 | 32 → 32 |
| `replaceall-simple` | 5 → 5 | 1352 → 1352 |
| `table-small` | 22 → 22 | 977 → 977 |
| `table-iterator-plus` | 2 → 2 | 72 → 72 |
| `table-iterator-pair-list` | 2 → 2 | 136 → 136 |
| `take-multispec-small` | 109 → 109 | 5856 → 5856 |
| `lazy-depth-map-build-8` | 22.001 → 16.001 | 2255.86 → 1879.86 |
| `lazy-depth-map-first-8` | 194 → 188 | 11552 → 11176 |
| `lazy-depth-map-eight-8` | 1470 → 1464 | 83152.9 → 82776.9 |
| `lazy-depth-select-restart-8` | 333 → 327 | 17356 → 16980 |
| `lazy-depth-drop-all-one-8` | 176 → 170 | 3504 → 3128 |
| `lazy-depth-take-map-8` | 234 → 220 | 12968 → 12144 |
| `lazy-depth-fold-through-source-8` | 749.001 → 743.001 | 58816.9 → 58440.9 |
| `lazy-depth-nest-map-8` | 198 → 192 | 12340 → 11964 |
| `lazy-depth-map-build-64` | 78.001 → 16.001 | 9423.86 → 5911.86 |
| `lazy-depth-map-first-64` | 1426 → 1364 | 88832 → 85320 |
| `lazy-depth-map-eight-64` | 10934 → 10872 | 651217 → 647705 |
| `lazy-depth-select-restart-64` | 1733 → 1671 | 102924 → 99412 |
| `lazy-depth-drop-all-one-64` | 1240 → 1178 | 21032 → 17520 |
| `lazy-depth-take-map-64` | 1746 → 1620 | 100160 → 93064 |
| `lazy-depth-fold-through-source-64` | 43396 → 43334 | 3.40616e+06 → 3.40265e+06 |
| `lazy-depth-nest-map-64` | 1430 → 1368 | 89620 → 86108 |
| `lazy-depth-map-build-256` | 270.001 → 16.001 | 33999.9 → 19735.9 |
| `lazy-depth-map-first-256` | 5650 → 5396 | 353792 → 339528 |
| `lazy-depth-map-eight-256` | 43382 → 43128 | 2.59886e+06 → 2.5846e+06 |
| `lazy-depth-select-restart-256` | 6533 → 6279 | 396300 → 382036 |
| `lazy-depth-drop-all-one-256` | 4888 → 4634 | 81128 → 66864 |
| `lazy-depth-take-map-256` | 6930 → 6420 | 399104 → 370504 |
| `lazy-depth-fold-through-source-256` | 689574 → 689320 | 5.40231e+07 → 5.40089e+07 |
| `lazy-depth-nest-map-256` | 5654 → 5400 | 354580 → 340316 |



#### candidate-00 — reference

Incoming flat G2 source, fresh one-unit builds.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42634.35005 / 42643.52965 | 1.16190 / 1.18138 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54883.63480 / 54868.40990 | 1.61817 / 1.72593 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 179811.63200 / 179957.22845 | 0.63513 / 0.66295 | +0 / -40 | yes |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 115403.66565 / 115401.09935 | 0.77424 / 0.81941 | +0 / -40 | yes |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805748.92315 / 805720.32305 | 0.13792 / 0.13263 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 146118.26715 / 146112.45695 | 0.64940 / 0.64814 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 113857.23955 / 113929.93790 | 1.08977 / 1.09621 | +0 / -40 | yes |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 192865.58990 / 192609.82855 | 0.54596 / 0.43982 | -1 / -96 | yes |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41675.56075 / 41404.94440 | 1.07978 / 0.41681 | +0 / -56 | yes |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88889.23820 / 89003.42045 | 0.56788 / 0.82258 | +0 / -40 | yes |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 183074.96300 / 182959.12160 | 0.46225 / 0.38579 | -3 / -208 | yes |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23754.97120 / 23739.24155 | 0.66075 / 0.60000 | +0 / +0 | yes |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24198.03550 / 24185.03480 | 0.45742 / 0.21299 | +0 / +0 | yes |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 118176.76005 / 118175.62300 | 0.86485 / 0.86919 | +0 / -40 | yes |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28366.34440 / 28390.12830 | 0.77522 / 0.93826 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25569.25375 / 25569.53715 | 0.90533 / 0.93057 | +0 / +0 | yes |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26766.59880 / 26765.97120 | 0.73247 / 0.73183 | +0 / +0 | yes |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29680.13155 / 29680.28980 | -0.00308 / 0.00346 | +0 / +0 | no |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9592.24350 / 9592.32315 | -0.00371 / 0.01478 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39816.79435 / 39777.85370 | 0.09836 / -0.03242 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254198.30755 / 254198.79080 | -0.00067 / -0.00031 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10034.38310 / 10033.82390 | 0.03616 / -0.18102 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 537.04720 / 537.03720 | 0.01149 / 0.01168 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1154.06375 / 1154.03910 | -0.01094 / 0.01620 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 387.01070 / 387.09855 | 0.02352 / 0.00713 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11460.40305 / 11460.29735 | 0.45629 / 0.45759 | +0 / +0 | yes |
| `table-small` | 14634.43665 / 14634.57930 | 14635.18590 / 14611.04140 | 0.00512 / -0.16084 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.21985 / 563.25110 | -0.00274 / -0.01658 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.71760 / 1640.81255 | -0.00416 / -0.00767 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49922.17225 / 49921.43310 | -0.03676 / -0.03693 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 19398.05300 / 19394.04900 | 0.81648 / 0.70164 | -6 / -376 | yes |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 131763.38300 / 131963.69400 | 0.20305 / 0.36347 | -6 / -376 | yes |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 961478.89000 / 961469.88200 | -0.26201 / -0.25773 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 181075.13200 / 181048.41900 | -0.17918 / -0.19495 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 49736.48300 / 49733.99400 | 2.03432 / 2.04611 | -6 / -376 | yes |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 142380.13500 / 142383.35800 | 0.65575 / 0.65520 | -14 / -824 | yes |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 593007.04000 / 593021.15300 | 0.01369 / 0.01601 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 142903.79100 / 142303.43600 | 0.22328 / -0.06365 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 39883.24400 / 39850.28700 | -1.99990 / -2.07285 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 956199.77500 / 957713.40000 | -0.11808 / 0.22213 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7433084.53100 / 7432437.50400 | -0.00714 / -0.00592 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 996652.31700 / 996536.42400 | 0.54124 / 0.89030 | -62 / -3512 | yes |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 355459.13100 / 355457.51100 | 1.18967 / 1.18852 | -62 / -3512 | yes |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1034156.00700 / 1034133.17800 | 0.19967 / 0.19793 | -126 / -7096 | yes |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 34028083.18300 / 34028003.89600 | -0.00834 / -0.01136 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 972014.09800 / 972012.49400 | 0.40028 / 0.21273 | -62 / -3512 | yes |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 109538.37300 / 109511.91100 | -4.10914 / -4.12035 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3775401.00100 / 3762135.61800 | 0.16232 / -0.00412 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29588889.32700 / 29590930.61000 | -0.07206 / -0.07103 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3709828.74500 / 3709827.44200 | -0.13930 / -0.48835 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2379161.48800 / 2379169.37600 | -0.38654 / -0.38638 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 4091985.07900 / 4092097.16700 | 0.04413 / 0.04351 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530672980.13200 / 530693852.71300 | -0.02626 / -0.03084 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3773000.03200 / 3766345.71200 | -0.18173 / -0.35790 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-00/metrics-comparison.tsv`.

#### candidate-01 — rejected

Exact replay of brief-565 candidate-02: LazyTable bound recovery moved to the suspension edge.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42642.45350 / 42690.51170 | 1.18112 / 1.29285 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54792.57095 / 54837.05520 | 1.44956 / 1.66780 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 179815.10575 / 179896.54300 | 0.63708 / 0.62900 | +0 / -40 | yes |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 115637.89915 / 115440.40970 | 0.97878 / 0.85375 | +0 / -40 | yes |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805936.12845 / 806018.63805 | 0.16119 / 0.16970 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 146044.69480 / 146120.32435 | 0.59872 / 0.65356 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 113950.84055 / 113899.72380 | 1.17287 / 1.06940 | +0 / -40 | yes |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 192630.28300 / 192678.03510 | 0.42329 / 0.47539 | -1 / -96 | yes |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41418.61480 / 41418.61275 | 0.45659 / 0.44996 | +0 / -56 | yes |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88804.97415 / 88883.56275 | 0.47255 / 0.68681 | +0 / -40 | yes |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 183104.80530 / 183277.99285 | 0.47862 / 0.56075 | -3 / -208 | yes |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23580.99485 / 23624.12600 | -0.07646 / 0.11217 | +0 / +0 | no |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24029.16845 / 24062.84235 | -0.24362 / -0.29332 | +0 / +0 | no |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 118058.70785 / 118044.18720 | 0.76409 / 0.75700 | +0 / -40 | yes |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28475.74795 / 28389.90130 | 1.16389 / 0.93745 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25598.69900 / 25568.22575 | 1.02153 / 0.92539 | +0 / +0 | yes |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26796.02270 / 26819.73675 | 0.84320 / 0.93418 | +0 / +0 | yes |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29708.30885 / 29708.73850 | 0.09185 / 0.09931 | +0 / +0 | yes |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9593.07475 / 9591.11920 | 0.00495 / 0.00222 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39816.78295 / 39816.86655 | 0.09833 / 0.06562 | +0 / +0 | yes |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254198.52640 / 254198.50085 | -0.00058 / -0.00043 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10035.03765 / 10033.86735 | 0.04269 / -0.18059 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 537.00275 / 536.97565 | 0.00321 / 0.00021 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1153.89850 / 1154.01825 | -0.02526 / 0.01439 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 386.98340 / 386.95640 | 0.01646 / -0.02959 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11382.28180 / 11408.27365 | -0.22849 / 0.00156 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14634.92545 / 14634.18805 | 0.00334 / -0.00267 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.20260 / 563.20540 | -0.00581 / -0.02469 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.64140 / 1640.74310 | -0.00881 / -0.01190 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49922.40620 / 49922.42195 | -0.03629 / -0.03495 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 19383.05600 / 19401.62600 | 0.73853 / 0.74099 | -6 / -376 | yes |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 131747.62900 / 132171.52000 | 0.19107 / 0.52153 | -6 / -376 | yes |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 961522.15700 / 961504.53500 | -0.25752 / -0.25414 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 181123.70100 / 181415.69800 | -0.15240 / 0.00752 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 49739.25900 / 49739.24400 | 2.04002 / 2.05689 | -6 / -376 | yes |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 142390.00400 / 142366.63900 | 0.66273 / 0.64338 | -14 / -824 | yes |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 593020.48300 / 593019.06400 | 0.01596 / 0.01566 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 142505.99500 / 142510.38500 | -0.05571 / 0.08169 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 39849.62900 / 39884.09800 | -2.08249 / -1.98976 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 956201.89400 / 956125.10200 | -0.11786 / 0.05592 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7432909.09500 / 7432848.39900 | -0.00950 / -0.00040 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 990536.42300 / 990652.41500 | -0.07572 / 0.29459 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 355461.65600 / 355463.11400 | 1.19039 / 1.19012 | -62 / -3512 | yes |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1034127.72000 / 1034248.71700 | 0.19693 / 0.20912 | -126 / -7096 | yes |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 34027353.71000 / 34027232.18400 | -0.01049 / -0.01363 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 967020.60000 / 972016.49100 | -0.11550 / 0.21314 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 109500.62000 / 109498.64900 | -4.14219 / -4.13196 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3755771.97700 / 3775575.07700 | -0.35844 / 0.35309 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29586736.35100 / 29589519.19500 | -0.07933 / -0.07580 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3724160.83000 / 3749711.57300 | 0.24648 / 0.58149 | -254 / -14264 | yes |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2379162.89200 / 2379168.18800 | -0.38648 / -0.38643 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 4092389.05100 / 4092135.75100 | 0.05401 / 0.04446 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530692280.05300 / 530684219.17000 | -0.02262 / -0.03265 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3786320.25400 / 3773001.24300 | 0.17067 / -0.18182 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-01/metrics-comparison.tsv`.

#### candidate-02 — rejected

Descriptor symbol classification once per traversal: context checked once, then name dispatch in validation and reservation scan.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42629.33295 / 42626.51420 | 1.14999 / 1.14101 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54628.75640 / 54602.90470 | 1.14626 / 1.23368 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 179320.96515 / 179336.71745 | 0.36052 / 0.31585 | +0 / -40 | yes |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114849.60335 / 114860.02695 | 0.29041 / 0.34670 | +0 / -40 | yes |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805539.33785 / 805623.60545 | 0.11187 / 0.12061 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145813.30430 / 145774.82530 | 0.43934 / 0.41557 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 113229.41440 / 113262.84010 | 0.53234 / 0.50426 | +0 / -40 | yes |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191909.33800 / 191841.01690 | 0.04744 / 0.03891 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41263.15860 / 41264.30065 | 0.07954 / 0.07571 | +0 / -56 | yes |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88332.49795 / 88355.87010 | -0.06201 / 0.08904 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181585.53975 / 181871.65845 | -0.35507 / -0.21088 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23859.10235 / 23838.37235 | 1.10201 / 1.02008 | +0 / +0 | yes |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24243.99450 / 24274.69160 | 0.64822 / 0.58450 | +0 / +0 | yes |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117721.91255 / 117768.82985 | 0.47664 / 0.52197 | +0 / -40 | yes |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28345.24265 / 28344.55435 | 0.70025 / 0.77623 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25490.72650 / 25505.51300 | 0.59543 / 0.67784 | +0 / +0 | yes |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26727.84075 / 26728.43970 | 0.58661 / 0.59059 | +0 / +0 | yes |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29708.11920 / 29712.68850 | 0.09121 / 0.11262 | +0 / +0 | yes |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9593.18125 / 9592.97575 | 0.00606 / 0.02158 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39777.78395 / 39862.80510 | 0.00029 / 0.18107 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254198.36780 / 254198.56430 | -0.00064 / -0.00040 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10033.31890 / 10033.30865 | 0.02555 / -0.18615 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 536.91655 / 537.10765 | -0.01284 / 0.02480 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1154.11270 / 1154.03965 | -0.00670 / 0.01625 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 387.04990 / 386.88620 | 0.03365 / -0.04773 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11408.28635 / 11408.24545 | -0.00054 / 0.00132 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.19650 / 14635.19290 | 0.00519 / 0.00419 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.14165 / 563.19040 | -0.01663 / -0.02735 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.85435 / 1640.80040 | 0.00417 / -0.00841 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49922.49725 / 49921.39705 | -0.03611 / -0.03700 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18099.78500 / 18077.55900 | -5.93094 / -6.13410 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130460.43300 / 130465.70700 | -0.78782 / -0.77580 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 960243.82300 / 960214.17500 | -0.39013 / -0.38800 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179439.90900 / 180064.88400 | -1.08062 / -0.73713 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46856.96100 / 46857.36000 | -3.87301 / -3.85627 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 139030.12900 / 139027.31300 | -1.71254 / -1.71729 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 590071.69500 / 590065.82400 | -0.48137 / -0.48242 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 140879.01500 / 140884.36900 | -1.19676 / -1.06022 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30126.51600 / 30085.42800 | -25.97388 / -26.06878 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 944768.69500 / 944776.95300 | -1.31214 / -1.13163 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7423355.12800 / 7423150.61100 | -0.13802 / -0.13087 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 976377.47000 / 972985.11800 | -1.50406 / -1.49406 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 332367.63300 / 332364.62200 | -5.38386 / -5.38536 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1007781.28200 / 1007640.65600 | -2.35579 / -2.36895 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 34003306.62500 / 34002418.03600 | -0.08115 / -0.08654 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958801.30000 / 958796.49900 | -0.96448 / -1.14981 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 70774.82300 / 70733.30800 | -38.04310 / -38.07171 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3717119.52000 / 3723563.86300 | -1.38390 / -1.02934 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29552294.56400 / 29548957.19900 | -0.19565 / -0.21278 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3666225.22000 / 3652781.30000 | -1.31302 / -2.01855 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2286760.98500 / 2286746.62800 | -4.25527 / -4.25604 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3986219.39400 / 3986350.37400 | -2.54171 / -2.54178 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530579270.67400 / 530625803.19000 | -0.04391 / -0.04366 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3734127.47100 / 3727464.81100 | -1.21014 / -1.38653 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-02/metrics-comparison.tsv`.

#### candidate-03 — rejected

Descriptor symbol kind/context checked once with original ordered if-name comparisons in validation and reservation scan.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42630.41840 / 42624.15470 | 1.15257 / 1.13541 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54674.48630 / 54634.39980 | 1.23093 / 1.29207 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 179398.43670 / 179435.97450 | 0.40388 / 0.37138 | +0 / -40 | yes |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114877.17360 / 114875.09300 | 0.31449 / 0.35987 | +0 / -40 | yes |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805532.62910 / 805645.72255 | 0.11104 / 0.12336 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145812.15395 / 145812.06690 | 0.43854 / 0.44122 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 113192.50005 / 113319.56415 | 0.49957 / 0.55459 | +0 / -40 | yes |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191791.10600 / 191820.30445 | -0.01419 / 0.02811 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41310.86150 / 41264.73485 | 0.19524 / 0.07677 | +0 / -56 | yes |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88352.41365 / 88390.21465 | -0.03947 / 0.12794 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181541.22520 / 181655.81880 | -0.37939 / -0.32930 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23755.78945 / 23706.97005 | 0.66422 / 0.46324 | +0 / +0 | yes |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24228.76240 / 24327.86700 | 0.58499 / 0.80483 | +0 / +0 | yes |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117720.21790 / 117719.58480 | 0.47519 / 0.47994 | +0 / -40 | yes |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28344.80835 / 28344.94340 | 0.69871 / 0.77761 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25505.08770 / 25445.99240 | 0.65211 / 0.44290 | +0 / +0 | yes |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26728.72295 / 26766.00290 | 0.58993 / 0.73195 | +0 / +0 | yes |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29710.80625 / 29708.99385 | 0.10026 / 0.10017 | +0 / +0 | yes |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9592.03910 / 9591.29360 | -0.00584 / 0.00404 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39790.64605 / 39790.77220 | 0.03263 / 0.00004 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254199.06435 / 254198.67730 | -0.00037 / -0.00036 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10035.33010 / 10033.80340 | 0.04560 / -0.18123 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 536.97825 / 536.82710 | -0.00135 / -0.02745 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1154.05340 / 1154.06860 | -0.01184 / 0.01875 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 387.13900 / 386.95815 | 0.05668 / -0.02914 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11408.34860 / 11408.29770 | 0.00000 / 0.00177 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.01410 / 14575.96740 | 0.00395 / -0.40050 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.10310 / 563.12665 | -0.02347 / -0.03867 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.91465 / 1640.73650 | 0.00785 / -0.01230 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49921.69120 / 49922.43070 | -0.03772 / -0.03493 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18104.61500 / 18077.58200 | -5.90584 / -6.13398 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130863.59800 / 130453.51400 | -0.48122 / -0.78508 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 960183.54700 / 960183.17600 | -0.39638 / -0.39121 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179548.90700 / 180084.80800 | -1.02053 / -0.72615 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46873.68500 / 46873.64000 | -3.83870 / -3.82287 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 139052.03100 / 139057.08700 | -1.69706 / -1.69624 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 590071.92200 / 590079.34100 | -0.48133 / -0.48014 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 141093.60200 / 141085.91200 | -1.04627 / -0.91868 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30165.62100 / 30168.87300 | -25.87780 / -25.86373 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 944835.71300 / 948013.20400 | -1.30514 / -0.79296 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7422803.13600 / 7423101.81300 | -0.14545 / -0.13152 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 976198.97300 / 975973.78500 | -1.52207 / -1.19148 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 332551.70500 / 332553.00400 | -5.33146 / -5.33173 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1007892.91700 / 1007887.46600 | -2.34497 / -2.34504 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 34005234.60700 / 34003162.84900 | -0.07549 / -0.08435 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958862.08700 / 962195.87500 | -0.95820 / -0.79934 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 71008.29200 / 71021.54700 | -37.83872 / -37.81935 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3723640.25900 / 3723812.18800 | -1.21091 / -1.02274 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29550118.74500 / 29551331.89600 | -0.20299 / -0.20476 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3653173.67000 / 3679958.25200 | -1.66434 / -1.28956 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2287514.75400 / 2287515.75100 | -4.22371 / -4.22383 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3987604.22300 / 3987098.29000 | -2.50785 / -2.52350 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530615930.15300 / 530659699.96000 | -0.03700 / -0.03727 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3734380.66400 / 3734389.66500 | -1.20344 / -1.20332 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-03/metrics-comparison.tsv`.

#### candidate-04 — kept

Output-loop fusion, measured source parent candidate-02; acceptance compares against retained candidate-00.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42218.48655 / 42256.98330 | 0.17514 / 0.26421 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54096.99155 / 54096.65340 | 0.16168 / 0.29509 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 178844.04965 / 178804.94725 | 0.09360 / 0.01840 | +0 / -40 | no |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114646.78225 / 114593.14120 | 0.11330 / 0.11354 | +0 / -40 | yes |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805373.48690 / 805218.10875 | 0.09126 / 0.07021 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145322.85100 / 145263.12385 | 0.10150 / 0.06309 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 112699.37045 / 112686.41300 | 0.06174 / -0.00724 | +0 / -40 | no |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191447.50045 / 191461.91555 | -0.19332 / -0.15878 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41142.62310 / 41215.57670 | -0.21280 / -0.04245 | +0 / -56 | no |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88280.47800 / 88303.43055 | -0.12086 / 0.02964 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181270.88130 / 181421.46095 | -0.52774 / -0.45789 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23829.05885 / 23794.09100 | 0.97470 / 0.83243 | +0 / +0 | yes |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24396.60495 / 24241.70380 | 1.28178 / 0.44781 | +0 / +0 | yes |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117309.92670 / 117360.94320 | 0.12500 / 0.17382 | +0 / -40 | yes |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28190.23785 / 28189.66645 | 0.14958 / 0.22554 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25399.23795 / 25351.76585 | 0.23439 / 0.07096 | +0 / +0 | yes |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26578.99735 / 26579.62780 | 0.02645 / 0.03054 | +0 / +0 | no |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29684.44260 / 29680.79280 | 0.01144 / 0.00515 | +0 / +0 | no |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9591.27160 / 9593.09905 | -0.01384 / 0.02287 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39777.93480 / 39777.76050 | 0.00067 / -0.03266 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254197.95130 / 254198.38875 | -0.00081 / -0.00047 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10033.76090 / 10035.41230 | 0.02996 / -0.16522 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 537.11665 / 537.06530 | 0.02442 / 0.01691 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1154.06265 / 1154.04535 | -0.01104 / 0.01674 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 387.01580 / 386.99125 | 0.02484 / -0.02059 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11408.25000 / 11382.40410 | -0.00086 / -0.22520 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.11450 / 14610.99480 | 0.00463 / -0.16116 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.08340 / 563.47575 | -0.02697 / 0.02330 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.54635 / 1640.81465 | -0.01460 / -0.00754 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49922.11350 / 49922.00425 | -0.03687 / -0.03579 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18109.04800 / 18081.59400 | -5.88280 / -6.11315 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130389.24500 / 130793.09900 | -0.84195 / -0.52681 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 959585.36800 / 959465.60900 | -0.45843 / -0.46565 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179366.19100 / 179970.69200 | -1.12126 / -0.78906 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46674.45500 / 46671.16500 | -4.24742 / -4.23832 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 138932.64800 / 138934.39100 | -1.78145 / -1.78298 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 589483.37100 / 589441.05500 | -0.58060 / -0.58779 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 141009.76200 / 141014.08700 | -1.10507 / -0.96912 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30112.96200 / 30112.39800 | -26.00719 / -26.00251 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 949300.93400 / 946126.38000 | -0.83871 / -0.99042 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7421430.65200 / 7421084.29100 | -0.16391 / -0.15867 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 975517.99300 / 975572.39600 | -1.59077 / -1.23212 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 327588.17800 / 327587.60200 | -6.74444 / -6.74524 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1007246.55300 / 1007409.48800 | -2.40760 / -2.39135 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 33992377.13600 / 33994956.36900 | -0.11327 / -0.10847 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958839.17900 / 958701.38200 | -0.96057 / -1.15962 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 70791.39100 / 70748.39500 | -38.02859 / -38.05850 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3722401.83400 / 3722575.78800 | -1.24376 / -1.05561 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29544435.42500 / 29543913.49600 | -0.22219 / -0.22981 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3664705.70700 / 3664686.21000 | -1.35392 / -1.69921 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2218617.13700 / 2218633.66500 | -7.10839 / -7.10786 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3984505.75000 / 3985372.04100 | -2.58361 / -2.56570 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530490679.35000 / 530513673.55200 | -0.06060 / -0.06478 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3726651.19000 / 3726652.07000 | -1.40793 / -1.40803 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-04/metrics-comparison.tsv`.

#### candidate-05 — rejected

Borrow normalized LazyTable bounds until evaluation and clone the pending raw values only on the error edge; original continuation helper unchanged.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42214.46705 / 42212.83055 | 0.16561 / 0.15945 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54066.51450 / 54098.45890 | 0.10526 / 0.29844 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 178805.44995 / 178877.93385 | 0.07200 / 0.05922 | +0 / -40 | yes |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114593.80515 / 114605.46175 | 0.06704 / 0.12430 | +0 / -40 | yes |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805199.14270 / 805216.37625 | 0.06959 / 0.07000 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145305.45605 / 145276.47615 | 0.08952 / 0.07228 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 112702.18415 / 112682.89760 | 0.06424 / -0.01036 | +0 / -40 | no |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191509.49350 / 191462.02605 | -0.16101 / -0.15872 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41316.77110 / 41189.09415 | 0.20957 / -0.10668 | +0 / -56 | no |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88273.40855 / 88303.58545 | -0.12886 / 0.02981 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181493.39420 / 181401.93290 | -0.40564 / -0.46861 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23533.51380 / 23556.59665 | -0.27766 / -0.17400 | +0 / +0 | no |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24103.38220 / 24066.73895 | 0.06447 / -0.27718 | +0 / +0 | no |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117208.17705 / 117208.70625 | 0.03816 / 0.04387 | +0 / -40 | no |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28214.23805 / 28274.58585 | 0.23484 / 0.52746 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25351.46520 / 25351.26540 | 0.04586 / 0.06898 | +0 / +0 | no |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26578.80765 / 26626.73180 | 0.02574 / 0.20781 | +0 / +0 | no |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29681.01500 / 29681.03720 | -0.00011 / 0.00598 | +0 / +0 | no |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9591.27680 / 9591.56250 | -0.01379 / 0.00685 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39777.84605 / 39790.86545 | 0.00045 / 0.00028 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254197.90345 / 254198.01220 | -0.00083 / -0.00062 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10033.80580 / 10035.33090 | 0.03041 / -0.16603 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 536.99595 / 536.99770 | 0.00195 / 0.00432 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1153.95945 / 1153.88245 | -0.01998 / 0.00262 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 386.95175 / 387.02285 | 0.00828 / -0.01243 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11408.46270 / 11382.35505 | 0.00100 / -0.22563 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.31295 / 14635.01480 | 0.00599 / 0.00298 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.30570 / 563.40770 | 0.01250 / 0.01122 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.63270 / 1640.68660 | -0.00934 / -0.01534 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49922.18780 / 49921.86035 | -0.03672 / -0.03607 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18091.14200 / 18090.57400 | -5.97586 / -6.06652 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130373.19300 / 130388.15900 | -0.85416 / -0.83478 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 959601.48900 / 959553.85400 | -0.45676 / -0.45650 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179392.09700 / 179348.41900 | -1.10698 / -1.13209 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46674.91300 / 46670.48000 | -4.24649 / -4.23972 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 138937.50800 / 138913.92200 | -1.77802 / -1.79745 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 589501.44400 / 589476.18000 | -0.57755 / -0.58186 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 141003.45900 / 140986.53700 | -1.10949 / -0.98847 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30128.05900 / 30112.10200 | -25.97009 / -26.00323 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 946127.97700 / 946130.03000 | -1.17015 / -0.99003 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7421243.05600 / 7421171.18600 | -0.16644 / -0.15750 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 975352.45900 / 975343.04200 | -1.60746 / -1.25534 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 327592.35700 / 327589.36700 | -6.74325 / -6.74474 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1007262.18700 / 1007378.19200 | -2.40608 / -2.39438 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 33993169.91100 / 33992866.09600 | -0.11094 / -0.11461 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958562.69200 / 961893.27500 | -0.98912 / -0.83054 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 70753.69400 / 70788.73600 | -38.06159 / -38.02319 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3722581.54700 / 3722400.78800 | -1.23899 / -1.06026 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29541494.58600 / 29543563.15100 | -0.23212 / -0.23099 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3652118.11600 / 3665037.19800 | -1.69275 / -1.68980 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2218626.46100 / 2218623.76400 | -7.10800 / -7.10828 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3984268.79900 / 3984907.19600 | -2.58940 / -2.57707 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530464896.67900 / 530434143.97500 | -0.06546 / -0.07976 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3733314.29200 / 3733306.62500 | -1.23165 / -1.23198 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-05/metrics-comparison.tsv`.

#### candidate-06 — kept

Plain index and carried value at ordinary entry, with DriveCursor decoding only in the resumed bridge. One shared indexed loop. Measured source parent candidate-05; retained-gate parent candidate-04.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42194.77490 / 42313.56790 | 0.11888 / 0.39847 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54051.86455 / 54115.11850 | 0.07813 / 0.32933 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 178979.54505 / 178757.74685 | 0.16944 / -0.00801 | +0 / -40 | no |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114594.19415 / 114573.38960 | 0.06738 / 0.09628 | +0 / -40 | yes |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805220.02635 / 805220.43085 | 0.07219 / 0.07050 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145239.76970 / 145318.07370 | 0.04427 / 0.10094 | +0 / -56 | no |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 112826.90445 / 112733.03150 | 0.17497 / 0.03413 | +0 / -40 | no |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191517.79625 / 191449.16835 | -0.15668 / -0.16542 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41207.18705 / 41205.32620 | -0.05621 / -0.06731 | +0 / -56 | no |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88325.42690 / 88443.56490 | -0.07001 / 0.18838 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181399.88920 / 181288.48315 | -0.45695 / -0.53085 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23589.52390 / 23676.04345 | -0.04032 / 0.33218 | +0 / +0 | no |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24198.21450 / 24081.40415 | 0.45817 / -0.21641 | +0 / +0 | no |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117221.33785 / 117173.53290 | 0.04939 / 0.01385 | +0 / -40 | no |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28176.47950 / 28191.84005 | 0.10070 / 0.23327 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25400.34795 / 25352.58395 | 0.23877 / 0.07419 | +0 / +0 | yes |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26580.84280 / 26541.68470 | 0.03340 / -0.11225 | +0 / +0 | no |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29681.06390 / 29680.16530 | 0.00006 / 0.00304 | +0 / +0 | no |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9591.62135 / 9592.08975 | -0.01020 / 0.01234 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39790.80695 / 39790.79935 | 0.03303 / 0.00011 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254198.82340 / 254197.95600 | -0.00046 / -0.00064 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10035.38040 / 10033.35295 | 0.04611 / -0.18571 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 537.06075 / 537.06255 | 0.01401 / 0.01640 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1153.98360 / 1153.95645 | -0.01789 / 0.00903 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 386.94115 / 387.17980 | 0.00554 / 0.02812 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11382.33765 / 11408.20370 | -0.22800 / 0.00095 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.06710 / 14634.97565 | 0.00431 / 0.00271 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.19830 / 563.25165 | -0.00657 / -0.01648 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.76430 / 1640.57470 | -0.00132 / -0.02216 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49921.97460 / 49922.38615 | -0.03715 / -0.03502 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18089.03300 / 18126.74600 | -5.98682 / -5.87870 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130177.01800 / 130380.23700 | -1.00335 / -0.84081 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 959579.18200 / 959627.78200 | -0.45907 / -0.44883 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179978.23900 / 179386.71800 | -0.78386 / -1.11098 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46663.33400 / 46664.11800 | -4.27024 / -4.25278 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 138947.75100 / 138955.80800 | -1.77078 / -1.76784 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 589537.24300 / 589560.36100 | -0.57151 / -0.56767 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 141003.87900 / 141399.82000 | -1.10919 / -0.69823 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30140.14000 / 30121.16400 | -25.94041 / -25.98096 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 949358.73800 / 946024.81700 | -0.83267 / -1.00104 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7421706.80200 / 7421923.32100 | -0.16020 / -0.14738 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 975365.44800 / 973005.79800 | -1.60615 / -1.49197 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 327586.82700 / 327583.48100 | -6.74483 / -6.74641 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1007350.91700 / 1007336.70500 | -2.39749 / -2.39840 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 33997696.93100 / 33996678.32600 | -0.09763 / -0.10341 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958620.38100 / 961944.97300 | -0.98317 / -0.82521 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 70789.60500 / 70761.73500 | -38.03016 / -38.04683 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3722734.08500 / 3722741.38400 | -1.23495 / -1.05120 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29544375.66400 / 29545696.23800 | -0.22239 / -0.22379 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3665519.21200 / 3665190.14800 | -1.33202 / -1.68570 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2218605.55100 / 2218607.23600 | -7.10888 / -7.10897 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3985658.55300 / 3985396.27400 | -2.55542 / -2.56511 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530531366.29300 / 530517098.32300 | -0.05294 / -0.06413 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3733566.01900 / 3746873.97100 | -1.22499 / -0.87304 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-06/metrics-comparison.tsv`.

#### candidate-07 — rejected

Move the carried entry value directly, resetting its optional slot after descent; avoid Option::take in the shared indexed loop.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42242.32700 / 42206.36520 | 0.23171 / 0.14411 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54112.03385 / 54114.42820 | 0.18954 / 0.32805 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 178900.54115 / 178648.03835 | 0.12522 / -0.06937 | +0 / -40 | no |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114549.00060 / 114576.39380 | 0.02792 / 0.09891 | +0 / -40 | no |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805295.34640 / 805380.64570 | 0.08155 / 0.09041 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145350.92525 / 145374.18480 | 0.12084 / 0.13959 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 112644.47500 / 112790.77405 | 0.01300 / 0.08537 | +0 / -40 | no |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191500.93535 / 191469.86605 | -0.16547 / -0.15463 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41262.91030 / 41171.87155 | 0.07894 / -0.14845 | +0 / -56 | no |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88333.26280 / 88294.86825 | -0.06114 / 0.01994 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181223.90555 / 181240.88965 | -0.55352 / -0.55697 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23556.65925 / 23557.97705 | -0.17958 / -0.16815 | +0 / +0 | no |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24092.74045 / 24168.37310 | 0.02030 / 0.14396 | +0 / +0 | no |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117186.41875 / 117223.57675 | 0.01959 / 0.05657 | +0 / -40 | no |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28183.35105 / 28182.73500 | 0.12511 / 0.20089 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25320.24030 / 25344.86500 | -0.07737 / 0.04372 | +0 / +0 | no |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26566.29645 / 26565.97665 | -0.02134 / -0.02083 | +0 / +0 | no |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29681.04740 / 29682.91055 | 0.00000 / 0.01229 | +0 / +0 | no |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9592.24880 / 9592.27125 | -0.00366 / 0.01424 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39777.73695 / 39777.74410 | 0.00017 / -0.03270 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254198.73915 / 254197.85845 | -0.00050 / -0.00068 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10035.15555 / 10033.34540 | 0.04386 / -0.18579 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 536.99670 / 536.84400 | 0.00209 / -0.02430 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1154.12180 / 1154.06960 | -0.00591 / 0.01884 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 387.02275 / 387.12450 | 0.02663 / 0.01383 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11408.31955 / 11460.24880 | -0.00025 / 0.45716 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.08110 / 14673.21000 | 0.00440 / 0.26397 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.34360 / 563.30660 | 0.01923 / -0.00673 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.90060 / 1640.89260 | 0.00699 / -0.00279 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49921.50270 / 49922.54425 | -0.03810 / -0.03470 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18111.39600 / 18103.82300 | -5.87060 / -5.99772 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130369.06700 / 130378.07300 | -0.85730 / -0.84245 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 959624.99400 / 959548.01800 | -0.45432 / -0.45710 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179085.60600 / 179451.39700 | -1.27594 / -1.07532 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46664.20400 / 46665.51100 | -4.26845 / -4.24992 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 138948.38700 / 138943.96000 | -1.77033 / -1.77622 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 589561.85400 / 589527.77400 | -0.56736 / -0.57316 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 140986.80900 / 141007.61100 | -1.12116 / -0.97367 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30134.19100 / 30135.27200 | -25.95502 / -25.94630 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 944588.26000 / 949353.82400 | -1.33098 / -0.65267 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7421896.69300 / 7421097.98400 | -0.15764 / -0.15848 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 975689.36200 / 976125.87000 | -1.57348 / -1.17609 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 327639.17100 / 327642.23200 | -6.72993 / -6.72969 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1007445.51300 / 1007351.61900 | -2.38832 / -2.39696 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 33998757.61300 / 33996965.66600 | -0.09452 / -0.10256 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958614.09400 / 958612.00900 | -0.98382 / -1.16883 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 70745.12000 / 70761.92400 | -38.06910 / -38.04666 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3716527.63100 / 3722731.55600 | -1.39961 / -1.05146 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29544972.45200 / 29543592.59700 | -0.22037 / -0.23089 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3665548.73400 / 3665285.35500 | -1.33123 / -1.68314 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2218859.50900 / 2218851.85900 | -7.09824 / -7.09873 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3986144.92300 / 3985391.68100 | -2.54353 / -2.56522 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530634541.58400 / 530606870.37200 | -0.03350 / -0.04722 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3733550.43700 / 3733552.54900 | -1.22541 / -1.22547 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-07/metrics-comparison.tsv`.

#### candidate-08 — rejected

Normalize the output budget to one numeric limit at entry, replacing its enum match after every emitted item.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42215.85025 / 42213.18175 | 0.16889 / 0.16028 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54123.39830 / 54051.10560 | 0.21058 / 0.21065 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 179036.31085 / 178985.44245 | 0.20121 / 0.11936 | +0 / -40 | yes |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114638.04190 / 114413.16025 | 0.10567 / -0.04370 | +0 / -40 | no |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805241.10110 / 805416.16535 | 0.07481 / 0.09483 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145269.06135 / 145320.11065 | 0.06445 / 0.10234 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 112713.22705 / 112685.85650 | 0.07404 / -0.00773 | +0 / -40 | no |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191692.53720 / 191451.92560 | -0.06558 / -0.16398 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41209.47990 / 41180.35835 | -0.05065 / -0.12787 | +0 / -56 | no |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88312.55700 / 88288.16890 | -0.08457 / 0.01235 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181459.96835 / 181958.44465 | -0.42398 / -0.16326 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23529.03810 / 23558.15575 | -0.29663 / -0.16739 | +0 / +0 | no |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24069.05035 / 24146.10970 | -0.07805 / 0.05171 | +0 / +0 | no |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117196.35730 / 117171.81570 | 0.02807 / 0.01239 | +0 / -40 | no |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28215.55240 / 28191.30265 | 0.23951 / 0.23136 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25379.55815 / 25349.63980 | 0.15672 / 0.06257 | +0 / +0 | yes |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26574.57375 / 26573.91375 | 0.00981 / 0.00904 | +0 / +0 | no |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29681.09120 / 29680.89330 | 0.00015 / 0.00549 | +0 / +0 | no |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9592.22285 / 9591.64530 | -0.00393 / 0.00771 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39790.98035 / 39790.98745 | 0.03347 / 0.00059 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254198.12825 / 254198.19555 | -0.00074 / -0.00055 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10033.34190 / 10033.36970 | 0.02578 / -0.18554 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 537.20990 / 537.03720 | 0.04179 / 0.01168 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1153.96700 / 1153.95140 | -0.01933 / 0.00860 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 387.04640 / 387.11975 | 0.03275 / 0.01261 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11408.31480 / 11460.11245 | -0.00029 / 0.45597 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.00305 / 14634.86630 | 0.00387 / 0.00196 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.23635 / 563.34175 | 0.00019 / -0.00049 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.94840 / 1640.81045 | 0.00990 / -0.00779 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49922.04930 / 49921.75485 | -0.03700 / -0.03629 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18091.37200 / 18097.86600 | -5.97467 / -6.02866 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130388.26000 / 130791.71500 | -0.84270 / -0.52786 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 959675.17900 / 959613.42800 | -0.44911 / -0.45032 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179414.24500 / 179101.31900 | -1.09477 / -1.26831 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46707.15600 / 46707.38200 | -4.18034 / -4.16400 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 138952.74000 / 138952.87700 | -1.76725 / -1.76991 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 589506.65600 / 589494.75200 | -0.57667 / -0.57873 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 141007.78300 / 140806.69400 | -1.10645 / -1.11477 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30122.86500 / 30122.64700 | -25.98285 / -25.97732 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 944599.62800 / 946184.22600 | -1.32980 / -0.98436 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7421849.82700 / 7421833.10800 | -0.15827 / -0.14859 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 975359.87100 / 975576.43000 | -1.60672 / -1.23171 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 327904.69300 / 327904.44700 | -6.65434 / -6.65504 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1007348.59500 / 1007314.00000 | -2.39771 / -2.40060 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 33996691.82400 / 33994497.80600 | -0.10059 / -0.10981 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958617.74700 / 961809.12800 | -0.98344 / -0.83922 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 70778.77600 / 70778.40100 | -38.03964 / -38.03223 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3722646.87600 / 3736089.92400 | -1.23726 / -0.69641 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29546745.71200 / 29545946.81600 | -0.21439 / -0.22294 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3690359.03400 / 3665048.15500 | -0.66339 / -1.68950 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2219895.73500 / 2219884.99100 | -7.05486 / -7.05547 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3985530.29100 / 3985275.80800 | -2.55856 / -2.56805 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530595831.93600 / 530577073.07500 | -0.04079 / -0.05283 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3733556.32100 / 3726897.30100 | -1.22525 / -1.40154 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-08/metrics-comparison.tsv`.

#### candidate-09 — rejected

Borrow source and stage slice once per realization; retain indexed descent/ascent and route exhaustion through the same slice. Final allowed candidate.

| Label | Base instructions 1 / 2 | Candidate instructions 1 / 2 | Delta % 1 / 2 | Allocation count Δ / bytes Δ | Both > +0.05% |
| --- | ---: | ---: | ---: | ---: | :---: |
| `lazy-range-take-small` | 42144.67235 / 42145.63035 | 42176.58015 / 42249.45065 | 0.07571 / 0.24634 | +0 / +0 | yes |
| `lazy-range-normal-small` | 54009.66610 / 53937.48805 | 54080.41880 / 54079.92340 | 0.13100 / 0.26407 | +0 / -56 | yes |
| `lazy-map-realize-small` | 178676.79990 / 178772.05905 | 178783.94485 / 178783.00090 | 0.05997 / 0.00612 | +0 / -40 | no |
| `lazy-select-realize-small` | 114517.03060 / 114463.17885 | 114528.91090 / 114543.16995 | 0.01037 / 0.06988 | +0 / -40 | no |
| `lazy-select-scan-small` | 804639.15505 / 804653.13705 | 805173.54045 / 805146.41980 | 0.06641 / 0.06130 | +0 / -56 | yes |
| `lazy-nestlist-realize-small` | 145175.49415 / 145171.54080 | 145263.13355 / 145306.04975 | 0.06037 / 0.09266 | +0 / -56 | yes |
| `lazy-foldlist-realize-small` | 112629.83625 / 112694.56820 | 112618.55090 / 112444.72180 | -0.01002 / -0.22170 | +0 / -40 | no |
| `lazy-drop-realize-small` | 191818.33295 / 191766.39365 | 191483.61855 / 191423.00840 | -0.17450 / -0.17906 | -1 / -96 | no |
| `lazy-first-small` | 41230.36295 / 41233.08115 | 41210.95410 / 41212.01495 | -0.04707 / -0.05109 | +0 / -56 | no |
| `lazy-part-small` | 88387.30305 / 88277.26915 | 88221.05715 / 88267.01105 | -0.18809 / -0.01162 | +0 / -40 | no |
| `lazy-transparent-adapters-small` | 182232.59530 / 182255.99460 | 181227.77395 / 181558.48420 | -0.55139 / -0.38271 | -3 / -208 | no |
| `lazy-table-construct-short` | 23599.03950 / 23597.65640 | 23510.78155 / 23556.68025 | -0.37399 / -0.17364 | +0 / +0 | no |
| `lazy-table-construct-full` | 24087.85165 / 24133.63140 | 24088.29965 / 24043.44715 | 0.00186 / -0.37369 | +0 / +0 | no |
| `lazy-table-realize-small` | 117163.46920 / 117157.30365 | 117159.58300 / 117136.57915 | -0.00332 / -0.01769 | +0 / -40 | no |
| `lazy-permutations-take-small` | 28148.13485 / 28126.23115 | 28237.94395 / 28190.51800 | 0.31906 / 0.22857 | +0 / +0 | yes |
| `lazy-subsets-take-small` | 25339.84480 / 25333.78935 | 25347.22680 / 25323.40175 | 0.02913 / -0.04100 | +0 / +0 | no |
| `lazy-tuples-take-small` | 26571.96815 / 26571.51225 | 26554.93005 / 26617.96040 | -0.06412 / 0.17480 | +0 / +0 | no |
| `scan-no-input` | 29681.04665 / 29679.26345 | 29680.97615 / 29680.10195 | -0.00024 / 0.00283 | +0 / +0 | no |
| `optionvalue-three-no-input` | 9592.59950 / 9590.90585 | 9592.85835 / 9591.30845 | 0.00270 / 0.00420 | +0 / +0 | no |
| `replacerepeated-no-input` | 39777.66795 / 39790.75430 | 39777.95710 / 39777.68955 | 0.00073 / -0.03283 | +0 / +0 | no |
| `dot-three-no-input` | 254200.00395 / 254199.58635 | 254197.79810 / 254199.32660 | -0.00087 / -0.00010 | +0 / +0 | no |
| `differences-no-input` | 10030.75565 / 10052.02055 | 10035.21640 / 10035.29095 | 0.04447 / -0.16643 | +0 / +0 | no |
| `do-count-null` | 536.98550 / 536.97450 | 537.08010 / 536.96820 | 0.01762 / -0.00117 | +0 / +0 | no |
| `do-iterator-null` | 1154.19005 / 1153.85220 | 1153.90150 / 1153.85570 | -0.02500 / 0.00030 | +0 / +0 | no |
| `table-iterator-null` | 386.91970 / 387.07095 | 387.21405 / 386.95985 | 0.07608 / -0.02870 | +0 / +0 | no |
| `replaceall-simple` | 11408.34830 / 11408.09530 | 11408.25850 / 11408.41370 | -0.00079 / 0.00279 | +0 / +0 | no |
| `table-small` | 14634.43665 / 14634.57930 | 14635.19085 / 14634.94875 | 0.00515 / 0.00252 | +0 / +0 | no |
| `table-iterator-plus` | 563.23530 / 563.34450 | 563.19225 / 563.04670 | -0.00764 / -0.05286 | +0 / +0 | no |
| `table-iterator-pair-list` | 1640.78590 / 1640.93835 | 1640.77375 / 1640.80760 | -0.00074 / -0.00797 | +0 / +0 | no |
| `take-multispec-small` | 49940.52835 / 49939.87580 | 49922.31175 / 49921.37560 | -0.03648 / -0.03704 | +0 / +0 | no |
| `lazy-depth-map-build-8` | 19240.95500 / 19258.92000 | 18119.24100 / 18121.45600 | -5.82982 / -5.90617 | -6 / -376 | no |
| `lazy-depth-map-first-8` | 131496.38400 / 131485.77800 | 130388.61100 / 130583.11700 | -0.84244 / -0.68651 | -6 / -376 | no |
| `lazy-depth-map-eight-8` | 964004.65100 / 963954.28300 | 959539.29700 / 959425.95600 | -0.46321 / -0.46977 | -6 / -376 | no |
| `lazy-depth-select-restart-8` | 181400.15900 / 181402.05800 | 179930.05000 / 179371.46200 | -0.81042 / -1.11939 | -6 / -376 | no |
| `lazy-depth-drop-all-one-8` | 48744.85600 / 48736.78400 | 46529.78300 / 46522.41500 | -4.54422 / -4.54353 | -6 / -376 | no |
| `lazy-depth-take-map-8` | 141452.56000 / 141456.53400 | 138895.71900 / 138900.84200 | -1.80756 / -1.80670 | -14 / -824 | no |
| `lazy-depth-fold-through-source-8` | 592925.88100 / 592926.20300 | 589246.84700 / 589261.29800 | -0.62049 / -0.61810 | -6 / -376 | no |
| `lazy-depth-nest-map-8` | 142585.42400 / 142394.06300 | 141390.01800 / 141393.35300 | -0.83838 / -0.70278 | -6 / -376 | no |
| `lazy-depth-map-build-64` | 40697.14500 / 40693.80800 | 30122.29200 / 30139.06300 | -25.98426 / -25.93698 | -62 / -3512 | no |
| `lazy-depth-map-first-64` | 957330.16800 / 955590.69900 | 945979.82000 / 949310.03400 | -1.18563 / -0.65725 | -62 / -3512 | no |
| `lazy-depth-map-eight-64` | 7433615.27700 / 7432877.81200 | 7420957.19200 / 7420925.76800 | -0.17028 / -0.16080 | -62 / -3512 | no |
| `lazy-depth-select-restart-64` | 991287.04400 / 987742.58000 | 975221.46300 / 972213.51600 | -1.62068 / -1.57218 | -62 / -3512 | no |
| `lazy-depth-drop-all-one-64` | 351280.06200 / 351282.44100 | 319269.32600 / 319268.93400 | -9.11260 / -9.11332 | -62 / -3512 | no |
| `lazy-depth-take-map-64` | 1032095.25400 / 1032090.37400 | 1006968.83300 / 1006978.34800 | -2.43451 / -2.43312 | -126 / -7096 | no |
| `lazy-depth-fold-through-source-64` | 34030922.96600 / 34031869.15900 | 33981254.68000 / 33980698.32800 | -0.14595 / -0.15036 | -62 / -3512 | no |
| `lazy-depth-nest-map-64` | 968138.78900 / 969949.11300 | 958630.83500 / 958495.23600 | -0.98209 / -1.18087 | -62 / -3512 | no |
| `lazy-depth-map-build-256` | 114232.34400 / 114218.09300 | 70787.11700 / 70757.19900 | -38.03233 / -38.05080 | -254 / -14264 | no |
| `lazy-depth-map-first-256` | 3769282.72000 / 3762290.71800 | 3729126.91600 / 3715682.26000 | -1.06534 / -1.23883 | -254 / -14264 | no |
| `lazy-depth-map-eight-256` | 29610225.67800 / 29611964.36500 | 29542065.35600 / 29539824.27100 | -0.23019 / -0.24362 | -254 / -14264 | no |
| `lazy-depth-select-restart-256` | 3715003.90600 / 3728033.43300 | 3664167.76300 / 3664635.61300 | -1.36840 / -1.70057 | -254 / -14264 | no |
| `lazy-depth-drop-all-one-256` | 2388393.52400 / 2388397.67000 | 2087030.28500 / 2087032.58100 | -12.61782 / -12.61788 | -254 / -14264 | no |
| `lazy-depth-take-map-256` | 4090179.99600 / 4090317.40000 | 3983495.03800 / 3983614.30500 | -2.60832 / -2.60868 | -510 / -28600 | no |
| `lazy-depth-fold-through-source-256` | 530812356.05600 / 530857551.50600 | 530300582.50000 / 530254753.96800 | -0.09641 / -0.11355 | -254 / -14264 | no |
| `lazy-depth-nest-map-256` | 3779869.21900 / 3779873.79900 | 3733046.97800 / 3733053.01000 | -1.23873 / -1.23869 | -254 / -14264 | no |

Full raw process output, allocation slopes and peak-live byte pairs: `.temp/brief-566/perf/`; parsed rows: `.temp/brief-566/candidate-09/metrics-comparison.tsv`.



### 4. Verification — commands, bounds, exits and verbatim results

Cargo commands were serial and used the required bounded wrapper/environment. The logs below preserve result lines verbatim; complete stdout/stderr are archived at the named paths. Expected stale-certificate failures during replay are retained, followed by successful confirmations in both feature sets. Nine native ZeroMQ tests skipped because loopback socket binding was not permitted. Their harness results are reported as ok after returning from the skip; those nine transport checks were not executed.

#### candidate-00/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-00/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 38s
```

#### candidate-00/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-00/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 2.05s
```

#### candidate-01-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-01-fmt.log`.

#### candidate-01-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.75s
```

#### candidate-01/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-01/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 38s
```

#### candidate-01/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-01/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.82s
```

#### candidate-02-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-02-format.log`.

#### candidate-02-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-02-fmt.log`.

#### candidate-02-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.42s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.52s
```

#### candidate-02-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.34s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.32s
```

#### candidate-02/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-02/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 39s
```

#### candidate-02/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-02/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.84s
```

#### candidate-03-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-03-format.log`.

#### candidate-03-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-03-fmt.log`.

#### candidate-03-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.68s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.65s
```

#### candidate-03-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 25.83s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.32s
```

#### candidate-03/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-03/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### candidate-03/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-03/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.80s
```

#### candidate-04-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-04-format.log`.

#### candidate-04-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-04-fmt.log`.

#### candidate-04-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.58s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.99s
```

#### candidate-04-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 29.34s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.32s
```

#### candidate-04/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-04/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 42s
```

#### candidate-04/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-04/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.86s
```

#### candidate-05-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-05-format.log`.

#### candidate-05-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-05-fmt.log`.

#### candidate-05-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.71s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.03s
```

#### candidate-05-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.63s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.30s
```

#### candidate-05/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-05/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 39s
```

#### candidate-05/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-05/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.89s
```

#### candidate-06-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-06-format.log`.

#### candidate-06-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-06-fmt.log`.

#### candidate-06-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.89s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.81s
```

#### candidate-06-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.18s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.31s
```

#### candidate-06/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-06/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 43s
```

#### candidate-06/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-06/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.93s
```

#### candidate-07-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-07-format.log`.

#### candidate-07-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-07-fmt.log`.

#### candidate-07-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.72s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.58s
```

#### candidate-07-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.39s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.32s
```

#### candidate-07/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-07/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 53s
```

#### candidate-07/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-07/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.89s
```

#### candidate-08-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-08-format.log`.

#### candidate-08-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-08-fmt.log`.

#### candidate-08-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.68s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.07s
```

#### candidate-08-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.43s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.32s
```

#### candidate-08/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-08/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 41s
```

#### candidate-08/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-08/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.82s
```

#### candidate-09-format

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-09-format.log`.

#### candidate-09-fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/candidate-09-fmt.log`.

#### candidate-09-lazy-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.49s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.71s
```

#### candidate-09-lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.60s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.31s
```

#### candidate-09/build-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-09/target
exit_code=0
    Finished `release` profile [optimized] target(s) in 1m 40s
```

#### candidate-09/build-alloc

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli --target-dir .temp/brief-566/candidate-09/target --features rum-cli/alloc-count
exit_code=0
    Finished `release` profile [optimized] target(s) in 1.87s
```

#### inventory/discovery-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
Production files: 640; test-only files: 408; nominal definitions: 1612; aliases: 40; owning edges: 3660; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 5.79s
```

#### inventory/exact-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib production_inventory_is_exact -- --nocapture
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.45s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7585 rows
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 5.59s
```

#### inventory/discovery-no-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --no-default-features --lib discovery_report_without_inventory_input -- --nocapture
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.43s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
Production files: 640; test-only files: 408; nominal definitions: 1612; aliases: 40; owning edges: 3660; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 5.79s
```

#### inventory/exact-no-default

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --no-default-features --lib production_inventory_is_exact -- --nocapture
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7585 rows
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 5.71s
```

#### inventory/fingerprints

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 0.21s
```

#### inventory/certificates-default-replay

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib local_production_acceptance -- --nocapture
exit_code=101
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.89s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
thread 'tests::retention_closure::local::production::local_production_acceptance' panicked at crates/rum-stdlib/src/tests/retention_closure/local/production.rs:98:6:
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 17.95s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

#### inventory/certificates-no-default-replay

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture
exit_code=101
    Finished `test` profile [unoptimized + debuginfo] target(s) in 43.26s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
thread 'tests::retention_closure::local::production::local_production_acceptance' panicked at crates/rum-stdlib/src/tests/retention_closure/local/production.rs:98:6:
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 18.01s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

#### inventory/certificates-default-confirm

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib local_production_acceptance
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.08s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 17.87s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-44228b396944f9ba)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-ab1526217bb7e597)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-bdeaa5d842f94483)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

#### inventory/certificates-no-default-confirm

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --no-default-features local_production_acceptance
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.91s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 17.65s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-6199ab42789c491a)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-fb613c76a0ef7535)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-e7f8ba5487f6d7b9)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-c6f8c05860b98e46)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

#### final/fmt

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo fmt --check
exit_code=0
```

The command produced no result lines; its complete output is in `.temp/brief-566/final/fmt.log`.

#### final/lazy-pull-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test lazy_pull_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.07s
```

#### final/lazy-lib

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --lib lazy
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.33s
```

#### final/loop-continuations

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-stdlib --test loop_continuations
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-bdeaa5d842f94483)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
```

#### final/runtime

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo test -p rum-runtime
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.11s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.20s
```

#### final/workspace

```text
python3 /home/remi/dev/rum/scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-566/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture
exit_code=0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 34s
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.20s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.24s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.17s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.01s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.70s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
Production files: 640; test-only files: 408; nominal definitions: 1612; aliases: 40; owning edges: 3660; resolved Drop owners: 18
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7585 rows
expired-C01: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=true
expired-C02: RED stale or omitted instance certificate C02 hash=true
test result: ok. 1034 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1135.62s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.90s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.54s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.03s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.87s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```



Raw acceptance-process audit (each count and exit comes from preserved process results):

```text
Exact gate source/check files verified: 162
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "base", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-00", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-01", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-02", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-03", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-04", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-05", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-06", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-07", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-08", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
{"alloc_1_exits": [0], "alloc_1_processes": 108, "build": "candidate-09", "instructions_1_exits": [0], "instructions_1_processes": 108, "instructions_2_exits": [0], "instructions_2_processes": 108, "result_check_exits": [0], "result_checks": 54}
No missing, failed, duplicated or replaced acceptance samples.
```

### 5. Progress counters, disclosures and remaining attribution

| Candidate | Source parent | Gate parent | Failing labels | Largest remaining sample % | Largest replicated % | New regressions | Decision | Consecutive nonprogress |
| --- | --- | --- | ---: | ---: | ---: | --- | --- | ---: |
| `candidate-00` | `provided incoming` | `none` | 26 | +2.046114 | +2.034321 | none | reference | 0 |
| `candidate-01` | `candidate-00` | `candidate-00` | 24 | +2.056886 | +2.040016 | `lazy-depth-select-restart-256`, `replacerepeated-no-input`, `scan-no-input` | rejected | 0 |
| `candidate-02` | `candidate-00` | `candidate-00` | 15 | +1.233681 | +1.146258 | `scan-no-input` | rejected | 0 |
| `candidate-03` | `candidate-00` | `candidate-00` | 15 | +1.292073 | +1.230928 | `scan-no-input` | rejected | 1 |
| `candidate-04` | `candidate-02` | `candidate-00` | 10 | +1.281780 | +0.832433 | none | kept | 0 |
| `candidate-05` | `candidate-04` | `candidate-04` | 7 | +0.527460 | +0.234840 | `lazy-map-realize-small` | rejected | 0 |
| `candidate-06` | `candidate-05` | `candidate-04` | 6 | +0.398470 | +0.118882 | none | kept | 0 |
| `candidate-07` | `candidate-06` | `candidate-06` | 5 | +0.328047 | +0.189536 | `lazy-nestlist-realize-small` | rejected | 0 |
| `candidate-08` | `candidate-06` | `candidate-06` | 7 | +0.239510 | +0.231355 | `lazy-map-realize-small`, `lazy-nestlist-realize-small` | rejected | 0 |
| `candidate-09` | `candidate-06` | `candidate-06` | 5 | +0.319059 | +0.228565 | `lazy-nestlist-realize-small` | rejected | 1 |

Progress and acceptance are separate: a falling failing-label count or largest remaining sample is numerical progress; any newly regressing label still rejects a candidate. Before candidate 02 completed, I corrected the initial scratch checkpoint helper, which had incorrectly treated every rejection as nonprogress. The corrected curve uses the best previously observed count/delta and preserves all samples; candidate 01’s 26→24 reduction is progress even though its new regressions reject it. The correction is recorded in `.temp/brief-566/stop-loss-interpretation.json`.

The following retained-binary attribution uses N=1,000/2,000; the separate full-N Subsets diagnostic follows.

| Residual label | Hardware delta % sample 1 / 2 | Diagnostic program base → retained | Old pull → flat driver self instructions | Old parser+entry → new entry+stage-push self instructions | Extra context / Option cleanup self instructions |
| --- | ---: | ---: | ---: | ---: | ---: |
| `lazy-range-take-small` | +0.118882 / +0.398470 | 42262.121 → 42352.442 | 672.000 → 968.000 | 451.000 → 212.000 | 14.000 / +10.000 |
| `lazy-range-normal-small` | +0.078131 / +0.329327 | 54152.787 → 54291.399 | 811.000 → 1381.000 | 582.000 → 341.000 | 14.000 / +0.000 |
| `lazy-select-realize-small` | +0.067382 / +0.096285 | 114572.881 → 114650.765 | 1499.000 → 1818.000 | 571.000 → 542.000 | 14.000 / +0.000 |
| `lazy-select-scan-small` | +0.072190 / +0.070502 | 805520.413 → 806224.814 | 5156.000 → 5893.000 | 432.000 → 356.000 | 14.000 / +10.000 |
| `lazy-permutations-take-small` | +0.100698 / +0.233266 | 28804.469 → 28830.040 | 207.000 → 359.000 | 353.000 → 275.000 | 14.000 / +10.000 |
| `lazy-subsets-take-small` | +0.238767 / +0.074188 | 26096.526 → 25978.977 | 435.000 → 613.000 | 261.000 → 266.000 | 14.000 / +10.000 |


The residual driver work is the flat machine’s indexed descent/ascent, stage-kind dispatch, and continuation/context cleanup. Range and the combinatorial sources still enter that general loop with an empty stage vector. Normal and Select also construct/drop adapter stages. Select realization performs eight predicates for four even values; the scanning label performs 32 predicates for four values. Each callback retains its required completion cancellation check. The old recursive Select held its branch while retrying its child; the flat loop revisits indexed state on rejection. The self-cost table locates this work while also showing the parser/entry savings that offset it. Symbol/cache/allocator changes and calibration limits remain in program totals; the profiles do not justify assigning every residual instruction to a single source statement.

Additional Subsets diagnostic at the exact acceptance workload sizes, after all Cargo commands finished. This is attribution only; the original two acceptance samples and rejection rules are unchanged. Every process returned exit 0 and stdout `Null`, with stdin `/dev/null`, profile flags cleared and CPU 2. The diagnostic source files match the fixed acceptance sources byte for byte.

| Build | N | Hardware instructions | Callgrind instructions | Callgrind vs hardware % |
| --- | ---: | ---: | ---: | ---: |
| `base` | 20000 | 512852035 | 513117830 | +0.051827 |
| `base` | 40000 | 1019710804 | 1019955090 | +0.023956 |
| `candidate-06` | 20000 | 513096793 | 513354869 | +0.050298 |
| `candidate-06` | 40000 | 1020132072 | 1020550345 | +0.041002 |

| Tool | Base instructions/iteration | Retained instructions/iteration | Delta % |
| --- | ---: | ---: | ---: |
| perf | 25342.938450 | 25351.763950 | +0.034824 |
| callgrind | 25341.863000 | 25359.773800 | +0.070677 |

Largest full-N Subsets self-cost changes (Callgrind slopes):

| Function | Base | Retained | Delta |
| --- | ---: | ---: | ---: |
| `rum_stdlib::lazy::machine::drive_machine` | 0.000000 | 613.000000 | +613.000000 |
| `rum_stdlib::lazy::PullStepper::next` | 435.000000 | 0.000000 | -435.000000 |
| `<T as alloc::slice::hack::ConvertVec>::to_vec` | 142.000000 | 37.000000 | -105.000000 |
| `rum_stdlib::lazy::realize` | 203.000000 | 266.000000 | +63.000000 |
| `rum_stdlib::lazy::stepper_from_descriptor` | 58.000000 | 0.000000 | -58.000000 |
| `rum_core::expr::Symbol::is_system_name` | 425.004750 | 378.004750 | -47.000000 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::PullStepper>` | 39.000000 | 0.000000 | -39.000000 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::state::PullSource>` | 0.000000 | 39.000000 | +39.000000 |
| `0x00000000001b1220` | 572.012450 | 546.010100 | -26.002350 |
| `core::ptr::drop_in_place<rum_stdlib::lazy::state::PullContext>` | 0.000000 | 14.000000 | +14.000000 |
| `rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name` | 28.000000 | 16.000000 | -12.000000 |
| `rum_core::expr::Symbol::is_system` | 0.000000 | 11.000000 | +11.000000 |
| `core::ptr::drop_in_place<core::option::Option<rum_core::expr::Expr>>` | 0.000000 | 10.000000 | +10.000000 |
| `rum_stdlib::structure_ops::sequence::take_drop::eval_take` | 86.000000 | 81.000000 | -5.000000 |
| `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 876.958650 | 872.167050 | -4.791600 |
| `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom'2` | 1231.913900 | 1231.723300 | -0.190600 |
| `hashbrown::map::HashMap<K,V,S,A>::insert` | 553.362750 | 553.223850 | -0.138900 |
| `rum_core::expr::intern_symbol_text` | 195.996800 | 196.055100 | +0.058300 |
| `0x000000000406b128` | 0.000000 | 0.027400 | +0.027400 |
| `0x000000000406a128` | 0.027400 | 0.000000 | -0.027400 |

Verbatim diagnostic commands and results (raw stdout, stderr, instruction CSVs and call graphs are retained alongside each metadata file):

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-20000-perf.csv -- /home/remi/dev/rum/.temp/brief-566/base/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-20000.rum < /dev/null
exit=0
Null
```

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 valgrind --tool=callgrind --callgrind-out-file=/home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-20000-callgrind.out /home/remi/dev/rum/.temp/brief-566/base/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-20000.rum < /dev/null
exit=0
Null
```

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-20000-perf.csv -- /home/remi/dev/rum/.temp/brief-566/candidate-06/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-20000.rum < /dev/null
exit=0
Null
```

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 valgrind --tool=callgrind --callgrind-out-file=/home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-20000-callgrind.out /home/remi/dev/rum/.temp/brief-566/candidate-06/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-20000.rum < /dev/null
exit=0
Null
```

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-40000-perf.csv -- /home/remi/dev/rum/.temp/brief-566/base/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-40000.rum < /dev/null
exit=0
Null
```

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 valgrind --tool=callgrind --callgrind-out-file=/home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-40000-callgrind.out /home/remi/dev/rum/.temp/brief-566/base/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-40000.rum < /dev/null
exit=0
Null
```

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 perf stat -x, -e cpu_core/instructions/u -o /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-40000-perf.csv -- /home/remi/dev/rum/.temp/brief-566/candidate-06/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-40000.rum < /dev/null
exit=0
Null
```

```text
python3 scripts/run-bounded.py --timeout 150 -- taskset -c 2 valgrind --tool=callgrind --callgrind-out-file=/home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-40000-callgrind.out /home/remi/dev/rum/.temp/brief-566/candidate-06/rum-cli --file /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/lazy-subsets-take-small-40000.rum < /dev/null
exit=0
Null
```

Annotation commands (the complete rendered function tables are stored at the matching `.txt` paths):

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=no --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-20000-callgrind.out < /dev/null
exit=0
```

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=yes --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-20000-callgrind.out < /dev/null
exit=0
```

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=no --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-40000-callgrind.out < /dev/null
exit=0
```

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=yes --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/base-40000-callgrind.out < /dev/null
exit=0
```

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=no --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-20000-callgrind.out < /dev/null
exit=0
```

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=yes --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-20000-callgrind.out < /dev/null
exit=0
```

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=no --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-40000-callgrind.out < /dev/null
exit=0
```

```text
python3 scripts/run-bounded.py --timeout 150 -- callgrind_annotate --inclusive=yes --threshold=100 --auto=no /home/remi/dev/rum/.temp/brief-566/residual-fulln/lazy-subsets-take-small/candidate-06-40000-callgrind.out < /dev/null
exit=0
```

No optimization or acceptance decision followed this diagnostic.



The short Subsets diagnostic has a negative total delta despite positive driver overhead. Repeating attribution at the exact N=20,000/40,000 workloads gives +0.070677% in Callgrind and +0.034824% in its hardware calibration, while the original acceptance samples remain +0.238767% and +0.074188%. The driver remains 435→613 self instructions/iteration, with 14 context-drop instructions and 10 additional Option-drop instructions; parser/entry changes add 5, offset by copy/comparison self-cost savings elsewhere. The full-N profile therefore supports a small net extra cost but does not reproduce the full acceptance magnitude in every run. No diagnostic result substitutes for the two recorded gate samples, so Subsets remains one of the six failures.


Failure/disclosure record: one deep map-eight-256 Callgrind 2N diagnostic timed out at 150 s (exit 124). Its incomplete pair was excluded; the deepest remaining diagnostics were rerun at N=10/20, with Fold-through-source at 10/20 for depth 64 and 1/2 for depth 256. Fixed-N acceptance measurements were unchanged. Two attempted profiler-stop signals found no matching process group and delivered no signal; the bounded runner performed the timeout cleanup. A scratch setup snippet’s missing JSON import and a source-audit comparison of insertion order were corrected without changing measured source or data. Initial report assembly also exited 1 with `KeyError: changed`: the bounded command wrapper’s metadata paths overlapped the audit-summary paths. The initial command metadata/logs were preserved, and the read-only audits were rerun under distinct command labels to regenerate their summaries. No acceptance data or measured source was overwritten.

All mandatory Cargo verification commands completed. Native ZeroMQ skips are skips, as recorded explicitly in the workspace transcript; any ignored tests remain ignored. No timing microbench, third-party engine comparison, additional acceptance sample, base rebuild or candidate beyond the cap was run. Every kept/rejected binary, source, raw measurement and decision remains available under `.temp/brief-566/`.

