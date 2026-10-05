# brief-569 — erase the schema validator's ordinary-path cost (G3)

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-568 (`70108834`) made the schema validator a flat work stack that
resumes into its walk and cannot overflow the native stack. It is correct and
gated. But a validation that never suspends got dearer on small schemas.
Per-iteration instructions, both arms built with one codegen unit (base
`.temp/brief-568/base/`, candidate `.temp/brief-568/candidate-1/`), from
brief-568's report §4 (the director reproduced the first, second, fourth and
the control):

| Label | Delta, sample 1 / 2 |
| --- | --- |
| `schema-conform-association-defaults-small` | +3.15% / +3.25% (director: +3.27% / +3.27%) |
| `schema-errors-association-defaults-small` | +3.43% / +2.83% |
| `schema-conform-list-small` | +0.76% / +1.02% (director: +1.02% / +1.30%) |
| `schema-errors-list-small` | +0.55% / +0.47% |
| `schema-conform-list-depth-8` / `schema-errors-list-depth-8` | +0.85% / +0.82%; +0.85% / +0.89% |
| `schema-conform-association-index-33` / `-errors-` | +0.52% / +0.49%; +0.55% / +0.57% (director: +0.49%) |
| `schema-conform-association-depth-8` / `-errors-` | +0.26% / +0.27%; +0.35% / +0.35% |

Deeper schemas got faster (list depth 128 -5.2%), and every non-schema
neighbour and control sits at the base (`dot-three-no-input` +0.00%).
Allocation bytes per small validation rose (list-small 1 144 -> 1 960,
association-defaults 3 139 -> 3 977) while counts fell by one.

**Target:** every one of the 27 labels in `.temp/brief-568/perf/labels.json`
within +0.05% of the base (a label regresses when **both** samples exceed
+0.05%), with every test unchanged and green. If an honest effort leaves a
residual, attribute it to the specific work the flat walk must do that the
recursion did not.

## Fixed — not negotiable in this brief

- **The flat walk and its depth contract.** One heap-held work stack, no
  native recursion in proportion to depth; `schema_depth_contract.rs` (the
  recursion scanner per file and combined, no self-reaching schema type, root
  completeness) and the depth tests stay unchanged and green. **No regression
  is bought back by restoring recursion.**
- **Semantics.** `crates/rum-stdlib/tests/schema_continuations.rs` and its
  children, and every other test, stay exactly as they are.

## Where the director would look first — hypotheses, measure each

brief-568 §4's attribution: for the small list, schema self-cost rose 867 ->
1 534 instructions per validation, `Expr::clone` self-cost +168, partly offset
by less formatting; for the 33-key association, the owned `String`-keyed
`HashMap` replaced the old borrowed-key local index.

1. **Build what a flat, non-nested validation does not need lazily**: the work
   stack `Vec` and the path buffer for a schema with no nested container (a
   root container handled before the stack is ever pushed, or the stack and
   path allocated on first descent).
2. **Paths only on errors.** The path text is needed only when an error is
   recorded; a stack of segment references (key index or list index) rendered
   on the first error may cost less than maintaining text on every descent.
3. **Fewer `Expr` clones on the ordinary path.** Share `SharedArgs`/`Expr`
   handles where a work record now clones items it only reads.
4. **The association index without owned `String` keys** — e.g. indices into
   the value entries the state already owns, sorted by key and searched with
   borrowed comparisons. If this removes `AssociationWork.positions`, the
   `RandomState` row in `unknown_storage.tsv` that brief-568 admitted
   disappears: **that one removal is authorized**, from discovery, disclosed.

**Attribute before you change**: `valgrind --tool=callgrind` on base and
candidate, each total calibrated against `perf stat` on the same binary.

## Measurement

- **Base:** `.temp/brief-568/base/rum-cli` and `rum-cli-alloc` (hashes in
  `sha256.txt`); do not rebuild them.
- **Candidates:** the working tree built with
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
  rum-cli/alloc-count`, into a fresh target directory each under
  `.temp/brief-569/`; check the binary's hash changed; keep every measured
  binary, its hash and source. Measure the incoming tree once, the same way,
  as candidate 00.
- **Panel:** `.temp/brief-568/perf/` (27 labels, workloads and checks; check
  each result first). brief-568's and brief-566's scripts are a working
  harness to copy into `.temp/brief-569/`.
- **Method:** allocation counts first, then two paired samples of `taskset -c
  2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
  `/dev/null`, `(count(2N) - count(N)) / N`, raw output saved before parsing.
  Any candidate you keep, and the final one, runs all 27 labels. No timing.

## Scope

- **Objective:** the ordinary-path shape of `crates/rum-stdlib/src/schema.rs`
  and `crates/rum-stdlib/src/schema/`.
- **Standing duty — always in scope** (`TEMPLATE.md`): replay the
  fingerprints and both certificates, reconcile the ownership inventories,
  keep rum-runtime's frame lists exact. New `UnknownConstruct` rows for code
  you add or change are expected and disclosed; the one removal above is
  authorized; STOP on any other changed or vanished existing row, a new
  `UnknownHostPayload` or `self_reaching` row, or a row in code you did not
  touch.
- **Fence:** no semantic change and no test edit; no rum-runtime or rum-core
  production change; no matcher change; no other suspension group; no
  dependency, no `unsafe`, no depth cap, no recursion; the release profile in
  `Cargo.toml` unchanged. No edits to `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  candidate — what it changed, its attribution, its rows.

## rum / Rust pitfalls (read twice)

1. **One variable per candidate.**
2. **A candidate that makes another label regress is rejected**, a deep label
   pushed above the base included.
3. **Instruction counts decide.**
4. **Stop-loss is keyed on progress:** the number of regressing labels or the
   largest remaining delta must fall. Stop after two consecutive compiling
   candidates that reduce neither; at most ten measured candidates.
5. A second ordinary code path kept beside the general one (a "flat-schema
   fast path") is a drift risk: it needs a falsifier proving both paths agree
   on every semantic case, or do not add it.
6. `#[inline(always)]` is not a fix by itself (brief-560: +1.17%).

## Verification (MANDATORY — you run it; iterate to green)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-569/tmp …`. Bounds: a focused test target 150 s; a
one-unit release build 600 s; a metric process 150 s;
`local_production_acceptance` and `cargo test -p rum-runtime` 300 s;
`cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test schema_continuations
cargo test -p rum-stdlib --lib schema_depth_contract
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

Plus the retention replay in both feature sets. Paste commands, bounds, exit
statuses and counts verbatim; list native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-569/`. After each candidate write
`.temp/brief-569/PROGRESS.md`: its change, its attribution, its rows, the
counters, the exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads and
  edits within scope, `rg`/`grep`, `python3`, bounded `cargo`, the release
  `rum-cli`, bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind` with
  `callgrind_annotate`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. The attribution for each regressing family, base against incoming.
3. Every candidate measured, with its rows (all 27 for kept candidates), both
   samples.
4. Verification — every command's verbatim result.
5. Progress counters, disclosures, and the attribution of anything left.

---

## Executor report — brief-569

Retained **candidate-09**. The two-sample instruction gate has **2 / 27 residual labels**; worst recurring delta is **+0.072598%**. The 27-label performance target is not fully met. The flat walk and every existing test remain unchanged in contract. All requested test and retention gates completed; native ZeroMQ skips are itemized below. Performance acceptance is reported separately from test success.

### 1. Changed files and line ranges

- `crates/rum-stdlib/src/schema.rs`: 16, 41, 49, 52–57, 88, 96, 99–104. Borrow evaluated input and argument tail; use the live index-storage specialization.
- `crates/rum-stdlib/src/schema/machine.rs`: 3–6, 9–10, 31–32, 36, after 78 (deletion), 89–95, 103–115. Derive error paths from waiting cursors; preserve the single iterative descent/ascent loop.
- `crates/rum-stdlib/src/schema/continuation.rs`: 5–7, 16–18, 26, 38, 54. Store/resume the same live index representation; no conversion or second validation algorithm.
- `crates/rum-stdlib/src/schema/state.rs`: 24–29, 59, 62, 68, 70, 84, 91, 137–143. Parameterize root-neutral index storage without changing existing test constructors; borrow temporary entry specs.
- `crates/rum-stdlib/src/schema/association.rs`: 2, 5–6, after 9 (deletion), 18–19, 23, 34, 38–41, 43–47, 68–72, 75–80, 95–98, 105–106, 129–145, 149–152. Use sorted indices and borrowed comparisons; preserve value/default order and lazy error paths; collect output from the owned entries buffer.
- `crates/rum-stdlib/src/schema/parse.rs`: 4, 10, 20–21, 23, 79, 88, 112–115, 117, 120, 125, 129–130. Borrow call arguments and temporary entry specs; move owned String schema keys.
- `crates/rum-stdlib/src/schema/format.rs`: 1–5, 38–90, 106. Construct/prefix paths only when reporting errors.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`: 2–69. Replay-derived fingerprint cells only.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`: 2–3. Actual replay-emitted certificate fingerprint cells only.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv`: 49, 233. Relocate the same schema evidence statements; preserve disposition fields.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv`: 2532, 2536–2555. Discovered source-position changes only.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv`: after 80 (deletion). Remove exactly the authorized AssociationWork.positions RandomState row.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv`: 2367–2369, 2373–2385. Discovered new closures in edited schema code and source-position moves.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv`: 1198–1200, 1205–1213. Discovered source-position changes only.

- `docs/dev/codex/brief-569-schema-ordinary-path-cost.md`: 173–2093. Append this five-part report, including every candidate panel and verbatim verification results.

This report is appended after the original instructions. The live/tested byte comparison preceded this reporting-only append; all code and test-data files remain exactly the tested versions. Scratch scripts, source snapshots, binaries, profiles and logs are confined to `.temp/brief-569/`. The source audit compares all initial source/document/script files by SHA-256, also rejects added/removed files, and verifies all tests, runtime/core production, dependency manifests, and other documentation unchanged. Fingerprint keys strip the workspace prefix and hash relative source paths plus bytes (`local/contracts.rs:88–133`); exporting does not change their identity. No runtime frame-list edit was necessary; the runtime gate checks the exact nominal frame lists.

### 2. Base against incoming: attribution

The frozen base was never rebuilt. Both candidate arms use one release codegen unit, fresh target directories, and original source paths. Default and alloc hashes, source manifests and all measured binaries are preserved for every candidate.

- Base `rum-cli` SHA-256: `74db20192d34707bd71a37486140004edb5ee032ea4d291cb88cdca2f4a32009`.
- Base `rum-cli-alloc` SHA-256: `572222df9775a3237f4e6c5754cd793ac8ebb684e243326bd0bf73542f98a124`.

Callgrind totals are individually calibrated against `perf stat` on the same binary and N=10/20 source. Self-cost reductions exclude inclusive call costs and each profile is checked to sum exactly to its total. These short profiles attribute work; the prescribed full-N hardware samples below decide acceptance. No timing result decides acceptance.

| Incoming residual | Base schema self | Incoming schema self | Delta |
| --- | ---: | ---: | ---: |
| `schema-conform-list-small` | 867 | 1534 | +667.0 |
| `schema-conform-association-defaults-small` | 1421 | 2095 | +674.0 |
| `schema-errors-list-small` | 936 | 1596 | +660.0 |
| `schema-errors-association-defaults-small` | 1490 | 2157 | +667.0 |
| `schema-conform-list-depth-8` | 1603 | 3825 | +2222.0 |
| `schema-errors-list-depth-8` | 1672 | 3887 | +2215.0 |
| `schema-conform-association-depth-8` | 5448 | 8461 | +3013.0 |
| `schema-errors-association-depth-8` | 5517 | 8523 | +3006.0 |
| `schema-conform-association-index-33` | 18662.9 | 23192.6 | +4529.7 |
| `schema-errors-association-index-33` | 18699 | 23252.4 | +4553.4 |

List-small adds 168 Expr-clone self instructions, list depth 8 adds 720, and association depth 8 adds 486. The 33-key case adds 980 Expr-clone self instructions plus about 2,700–2,900 aggregate HashMap-insert self instructions. HashMap totals include other maps and are not falsely assigned entirely to the schema index. The flat machine must select a task, retain owned pending values, update a waiting destination, and return the child into an ordered parent output; those operations were implicit in native recursion. Path maintenance and the duplicated owned index keys were avoidable ordinary-path work.

### 3. Every measured candidate

A failure means both deltas exceed +0.05%. A newly failing label rejects a candidate, even when its worst delta improves. Progress is tracked independently against the best observed failure count and worst recurring delta (the largest minimum of the two samples among failing labels). All 27 rows were measured for every candidate, including rejected ones; no sample was selectively repeated.

#### candidate-00 — retained at this step

Incoming source, unchanged. Baseline attribution shows flat record/dispatch work and extra Expr clones; the index also owns String keys.

Failures: 10; worst recurring delta +3.428246%; no-progress streak 0. incoming baseline, unchanged source.

- `rum-cli` SHA-256 `0d91edd3fd77f4f3f454153df3ecd7ee4a67c54d8209c92ab67852bdd5b89a0f`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-00/target-default`.
- `rum-cli-alloc` SHA-256 `9835ac8636b660f83066b043bb0b7acff0df0a138980bdaf7162c15ccd233722`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-00/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | +0.760665 | +1.026280 | 19 → 18 | 1144 → 1960 | FAIL |
| `schema-conform-association-defaults-small` | +3.207178 | +3.252470 | 40.0004 → 40.0004 | 3139.0066 → 3977.0066 | FAIL |
| `schema-errors-list-small` | +0.880415 | +0.356712 | 21 → 20 | 1232 → 2048 | FAIL |
| `schema-errors-association-defaults-small` | +3.652668 | +3.428246 | 39.0001 → 39.0001 | 3161.00165 → 3999.00165 | FAIL |
| `schema-conform-list-depth-8` | +0.825449 | +0.860094 | 124 → 115 | 5297 → 7584 | FAIL |
| `schema-errors-list-depth-8` | +0.889332 | +0.893132 | 126.001 → 117.001 | 9087.8 → 11374.8 | FAIL |
| `schema-conform-list-depth-32` | -1.023769 | -0.988247 | 330 → 277 | 25821 → 34072 | pass |
| `schema-errors-list-depth-32` | -1.825588 | -1.857274 | 332.001 → 279.001 | 26834.712 → 35085.712 | pass |
| `schema-conform-list-depth-128` | -3.880580 | -3.880110 | 1196 → 955 | 152109 → 132376 | pass |
| `schema-errors-list-depth-128` | -5.146950 | -5.134994 | 1198.001 → 957.001 | 153122.712 → 133389.712 | pass |
| `schema-conform-association-depth-8` | +0.234876 | +0.893392 | 386.032 → 378.032 | 36028.528 → 38400.528 | FAIL |
| `schema-errors-association-depth-8` | +0.334199 | +0.360490 | 372.016 → 364.016 | 35852.264 → 38224.264 | FAIL |
| `schema-conform-association-depth-32` | -0.786147 | -0.748884 | 1347.38 → 1295.38 | 170289.048 → 180005.048 | pass |
| `schema-errors-association-depth-32` | -1.255830 | -1.128293 | 1254.001 → 1202.001 | 133640.712 → 143356.712 | pass |
| `schema-conform-association-depth-128` | -1.642297 | -1.642122 | 5101.16 → 4861.16 | 687312.888 → 691844.888 | pass |
| `schema-errors-association-depth-128` | -2.114430 | -2.114241 | 4712.001 → 4472.001 | 534536.712 → 539068.712 | pass |
| `schema-conform-association-index-33` | +0.463016 | +0.498340 | 2843.14 → 2845.14 | 327044.048 → 328876.048 | FAIL |
| `schema-errors-association-index-33` | +0.545788 | +0.538765 | 2810.07 → 2812.07 | 324364.524 → 326196.524 | FAIL |
| `list-literal-small` | +0.015439 | -0.021735 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | +0.244555 | -0.001108 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | +1.037907 | +0.003826 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | -0.004583 | -0.001240 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.000404 | +0.002745 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | +0.011512 | -0.033221 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | +0.001099 | +0.003440 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.185258 | +0.026360 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | +0.015138 | -0.017417 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-00-slopes.tsv`; raw paired results: `candidate-00-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-00/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.232738% to +1.762469%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-00/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1534 |
| `schema-conform-association-defaults-small` | 1421 | 2095 |
| `schema-errors-list-small` | 936 | 1596 |
| `schema-errors-association-defaults-small` | 1490 | 2157 |
| `schema-conform-list-depth-8` | 1603 | 3825 |
| `schema-errors-list-depth-8` | 1672 | 3887 |
| `schema-conform-association-depth-8` | 5448 | 8461 |
| `schema-errors-association-depth-8` | 5517 | 8523 |
| `schema-conform-association-index-33` | 18662.9 | 23192.6 |
| `schema-errors-association-index-33` | 18699 | 23252.4 |
#### candidate-01 — rejected

Handle the active container locally and keep only ancestors in Vec. Rejected: six previously passing deep labels regressed. The saved flat allocation was outweighed by repeated large record movement; the unresolved libc self-cost at list depth 8 rose from the incoming +199.2 to +1579.2 instructions against base.

Failures: 16; worst recurring delta +3.049071%; no-progress streak 0. newly regressing label: reject.

New failures: `schema-conform-association-depth-128`, `schema-conform-association-depth-32`, `schema-conform-list-depth-32`, `schema-errors-association-depth-128`, `schema-errors-association-depth-32`, `schema-errors-list-depth-32`.

- `rum-cli` SHA-256 `65671e9bdc21b0793e4de4968852fef3655fbb74e3e57b3f0033d97bebea6126`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-01/target-default`.
- `rum-cli-alloc` SHA-256 `cb1986d09035e9f04f5f12c50c80686af812b4c81f5e0fe270e44bf0f6651d4f`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-01/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | +1.117609 | +0.912623 | 19 → 17 | 1144 → 1128 | FAIL |
| `schema-conform-association-defaults-small` | +3.214563 | +2.981365 | 40.0004 → 39.0004 | 3139.0066 → 3145.0066 | FAIL |
| `schema-errors-list-small` | +0.209407 | +0.727507 | 21 → 19 | 1232 → 1216 | FAIL |
| `schema-errors-association-defaults-small` | +2.825131 | +3.380964 | 39.0001 → 38.0001 | 3161.00165 → 3167.00165 | FAIL |
| `schema-conform-list-depth-8` | +2.803411 | +2.835086 | 124 → 115 | 5297 → 7584 | FAIL |
| `schema-errors-list-depth-8` | +3.057407 | +3.049071 | 126.001 → 117.001 | 9087.8 → 11374.8 | FAIL |
| `schema-conform-list-depth-32` | +2.317172 | +2.290453 | 330 → 277 | 25821 → 34072 | FAIL |
| `schema-errors-list-depth-32` | +2.187989 | +2.174231 | 332.001 → 279.001 | 26834.712 → 35085.712 | FAIL |
| `schema-conform-list-depth-128` | -0.150119 | -0.144328 | 1196 → 955 | 152109 → 132376 | pass |
| `schema-errors-list-depth-128` | -0.376391 | -0.387779 | 1198.001 → 957.001 | 153122.712 → 133389.712 | pass |
| `schema-conform-association-depth-8` | +1.420430 | +0.275061 | 386.032 → 378.032 | 36028.528 → 38400.528 | FAIL |
| `schema-errors-association-depth-8` | +1.868260 | +1.286964 | 372.016 → 364.016 | 35852.264 → 38224.264 | FAIL |
| `schema-conform-association-depth-32` | +0.666510 | +0.340748 | 1347.38 → 1295.38 | 170289.048 → 180005.048 | FAIL |
| `schema-errors-association-depth-32` | +1.266843 | +1.011862 | 1254.001 → 1202.001 | 133640.712 → 143356.712 | FAIL |
| `schema-conform-association-depth-128` | +0.174500 | +0.181592 | 5101.16 → 4861.16 | 687312.888 → 691844.888 | FAIL |
| `schema-errors-association-depth-128` | +0.403796 | +0.399183 | 4712.001 → 4472.001 | 534536.712 → 539068.712 | FAIL |
| `schema-conform-association-index-33` | +0.378330 | +0.448548 | 2843.14 → 2844.14 | 327044.048 → 328044.048 | FAIL |
| `schema-errors-association-index-33` | +0.544085 | +0.483608 | 2810.07 → 2811.07 | 324364.524 → 325364.524 | FAIL |
| `list-literal-small` | -0.022766 | +0.008128 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | +0.000326 | +0.001903 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | -0.001877 | +0.004265 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | +0.000365 | +0.003049 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.003430 | -0.084698 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | +0.038324 | +0.059516 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | +0.002818 | +0.002894 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.163314 | -0.025726 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | -0.001353 | -0.001768 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-01-slopes.tsv`; raw paired results: `candidate-01-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-01/`.

Attribution: 64 calibrated profiles; Callgrind total offset range +0.214352% to +4.331749%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-01/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1500 |
| `schema-conform-association-defaults-small` | 1421 | 2071 |
| `schema-errors-list-small` | 936 | 1562 |
| `schema-errors-association-defaults-small` | 1490 | 2133 |
| `schema-conform-list-depth-8` | 1603 | 3903 |
| `schema-errors-list-depth-8` | 1672 | 3965 |
| `schema-conform-association-depth-8` | 5448 | 8464 |
| `schema-errors-association-depth-8` | 5517 | 8526 |
| `schema-conform-association-index-33` | 18716.9 | 22701.4 |
| `schema-errors-association-index-33` | 18752.2 | 22760.6 |
| `schema-conform-list-depth-32` | 5347 | 14109 |
| `schema-errors-list-depth-32` | 5416 | 14171 |
| `schema-conform-association-depth-32` | 20784 | 32392 |
| `schema-errors-association-depth-32` | 20853 | 32454 |
| `schema-conform-association-depth-128` | 82128 | 128032 |
| `schema-errors-association-depth-128` | 82197 | 128094 |

Discovery differences against incoming:
- `unknown_macros.tsv`: 7 position moves, 0 additions, 0 removals; 3117 → 3117 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-01-ownership-reconciliation.json`.

#### candidate-02 — rejected

Handle only the root locally; nested containers stay in Vec. Rejected: both list-depth-32 labels newly regressed. More root-selection work left schema self-cost above incoming even though flat allocation bytes fell.

Failures: 12; worst recurring delta +3.190084%; no-progress streak 1. newly regressing label: reject.

New failures: `schema-conform-list-depth-32`, `schema-errors-list-depth-32`.

- `rum-cli` SHA-256 `d7bc3718024e836b87634d2b2f3be2665e7d5ebf025edcfb67970b713fe8d81e`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-02/target-default`.
- `rum-cli-alloc` SHA-256 `9d6ac6b18c4b5519c325ee5d73756700e084e6ade4ecb685d037ee4447c5ea5c`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-02/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | +1.025829 | +0.297581 | 19 → 17 | 1144 → 1128 | FAIL |
| `schema-conform-association-defaults-small` | +3.196409 | +3.060852 | 40.0004 → 39.0004 | 3139.0066 → 3145.0066 | FAIL |
| `schema-errors-list-small` | +0.576630 | +1.136385 | 21 → 19 | 1232 → 1216 | FAIL |
| `schema-errors-association-defaults-small` | +3.190084 | +3.233655 | 39.0001 → 38.0001 | 3161.00165 → 3167.00165 | FAIL |
| `schema-conform-list-depth-8` | +1.923351 | +1.976999 | 124 → 115 | 5297 → 7584 | FAIL |
| `schema-errors-list-depth-8` | +2.221264 | +2.158880 | 126.001 → 117.001 | 9087.8 → 11374.8 | FAIL |
| `schema-conform-list-depth-32` | +0.896319 | +0.868968 | 330 → 277 | 25821 → 34072 | FAIL |
| `schema-errors-list-depth-32` | +0.470181 | +0.480786 | 332.001 → 279.001 | 26834.712 → 35085.712 | FAIL |
| `schema-conform-list-depth-128` | -1.732841 | -1.739993 | 1196 → 955 | 152109 → 132376 | pass |
| `schema-errors-list-depth-128` | -2.414030 | -2.413595 | 1198.001 → 957.001 | 153122.712 → 133389.712 | pass |
| `schema-conform-association-depth-8` | +0.780030 | +0.764347 | 386.032 → 378.032 | 36028.528 → 38400.528 | FAIL |
| `schema-errors-association-depth-8` | +1.158564 | +2.299441 | 372.016 → 364.016 | 35852.264 → 38224.264 | FAIL |
| `schema-conform-association-depth-32` | -0.006470 | -0.008480 | 1347.38 → 1295.38 | 170289.048 → 180005.048 | pass |
| `schema-errors-association-depth-32` | +0.070637 | -0.123481 | 1254.001 → 1202.001 | 133640.712 → 143356.712 | pass |
| `schema-conform-association-depth-128` | -0.860737 | -0.862886 | 5101.16 → 4861.16 | 687312.888 → 691844.888 | pass |
| `schema-errors-association-depth-128` | -1.176815 | -1.039795 | 4712.001 → 4472.001 | 534536.712 → 539068.712 | pass |
| `schema-conform-association-index-33` | +0.576362 | +0.542034 | 2843.14 → 2844.14 | 327044.048 → 328044.048 | FAIL |
| `schema-errors-association-index-33` | +0.607432 | +0.614408 | 2810.07 → 2811.07 | 324364.524 → 325364.524 | FAIL |
| `list-literal-small` | -0.010379 | -0.004454 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | -0.194570 | -0.004848 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | +0.000721 | +0.002191 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | -0.000153 | +0.002969 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.204628 | +0.004586 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | +0.047188 | -0.017185 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | +0.002252 | +0.001421 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.167886 | -0.027885 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | +0.015642 | -0.002555 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-02-slopes.tsv`; raw paired results: `candidate-02-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-02/`.

Attribution: 48 calibrated profiles; Callgrind total offset range +0.272192% to +1.781563%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-02/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1740 |
| `schema-conform-association-defaults-small` | 1421 | 2255 |
| `schema-errors-list-small` | 936 | 1802 |
| `schema-errors-association-defaults-small` | 1490 | 2317 |
| `schema-conform-list-depth-8` | 1603 | 4470 |
| `schema-errors-list-depth-8` | 1672 | 4532 |
| `schema-conform-association-depth-8` | 5448 | 9160 |
| `schema-errors-association-depth-8` | 5517 | 9222 |
| `schema-conform-association-index-33` | 18697.5 | 25558.2 |
| `schema-errors-association-index-33` | 18740.6 | 25607.6 |
| `schema-conform-list-depth-32` | 5347 | 16332 |
| `schema-errors-list-depth-32` | 5416 | 16394 |

Discovery differences against incoming:
- `unknown_macros.tsv`: 7 position moves, 0 additions, 0 removals; 3117 → 3117 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-02-ownership-reconciliation.json`.

#### candidate-03 — retained at this step

Render paths only when recording errors. Retained: every list label passes and failures fall to six. The same iterative cursor stack supplies error paths after normal execution or suspension.

Failures: 6; worst recurring delta +2.746029%; no-progress streak 0. improves retained tree; observed progress tracked separately.

- `rum-cli` SHA-256 `3a1a0c06eacbf5e4ac0275112792ec638615d66397bf91807fe291eb880e7371`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-03/target-default`.
- `rum-cli-alloc` SHA-256 `ad278c2f790f4a4254c95425aa31c87a0b1d797cb2768797a466fc32f69a56c9`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-03/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | -3.091280 | -3.090982 | 19 → 17 | 1144 → 1952 | pass |
| `schema-conform-association-defaults-small` | +3.026824 | +2.699002 | 40.0004 → 39.0004 | 3139.0066 → 3969.0066 | FAIL |
| `schema-errors-list-small` | -3.151799 | -3.173353 | 21 → 19 | 1232 → 2040 | pass |
| `schema-errors-association-defaults-small` | +2.849272 | +2.746029 | 39.0001 → 38.0001 | 3161.00165 → 3991.00165 | FAIL |
| `schema-conform-list-depth-8` | -3.456257 | -3.511921 | 124 → 112 | 5297 → 7528 | pass |
| `schema-errors-list-depth-8` | -3.748391 | -3.722347 | 126.001 → 114.001 | 9087.8 → 11318.8 | pass |
| `schema-conform-list-depth-32` | -7.197556 | -7.191369 | 330 → 272 | 25821 → 33824 | pass |
| `schema-errors-list-depth-32` | -9.382893 | -9.360067 | 332.001 → 274.001 | 26834.712 → 34837.712 | pass |
| `schema-conform-list-depth-128` | -10.454221 | -10.458531 | 1196 → 948 | 152109 → 131360 | pass |
| `schema-errors-list-depth-128` | -13.559992 | -13.558614 | 1198.001 → 950.001 | 153122.712 → 132373.712 | pass |
| `schema-conform-association-depth-8` | +0.477367 | +0.759069 | 386.032 → 376.032 | 36028.528 → 38376.528 | FAIL |
| `schema-errors-association-depth-8` | +0.188283 | +0.191464 | 372.016 → 362.016 | 35852.264 → 38200.264 | FAIL |
| `schema-conform-association-depth-32` | -0.842260 | -0.528638 | 1347.38 → 1291.38 | 170289.048 → 179885.048 | pass |
| `schema-errors-association-depth-32` | -1.363259 | -1.363314 | 1254.001 → 1198.001 | 133640.712 → 143236.712 | pass |
| `schema-conform-association-depth-128` | -1.874924 | -1.873410 | 5101.16 → 4855.16 | 687312.888 → 691340.888 | pass |
| `schema-errors-association-depth-128` | -2.115335 | -2.478231 | 4712.001 → 4466.001 | 534536.712 → 538564.712 | pass |
| `schema-conform-association-index-33` | +0.444023 | +0.439445 | 2843.14 → 2844.14 | 327044.048 → 328868.048 | FAIL |
| `schema-errors-association-index-33` | +0.429959 | +0.604584 | 2810.07 → 2811.07 | 324364.524 → 326188.524 | FAIL |
| `list-literal-small` | -0.001862 | -0.005680 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | -0.065208 | -0.078745 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | +0.004155 | +0.002398 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | +0.000554 | +0.001453 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.003911 | -0.000597 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | +0.099660 | -0.024480 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | -0.000317 | -0.001934 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | +0.051370 | -0.181658 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | -0.034108 | -0.034307 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-03-slopes.tsv`; raw paired results: `candidate-03-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-03/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.245969% to +1.747108%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-03/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1444 |
| `schema-conform-association-defaults-small` | 1421 | 2110 |
| `schema-errors-list-small` | 936 | 1506 |
| `schema-errors-association-defaults-small` | 1490 | 2172 |
| `schema-conform-list-depth-8` | 1603 | 3595 |
| `schema-errors-list-depth-8` | 1672 | 3657 |
| `schema-conform-association-depth-8` | 5448 | 8662 |
| `schema-errors-association-depth-8` | 5517 | 8724 |
| `schema-conform-association-index-33` | 18646 | 23015 |
| `schema-errors-association-index-33` | 18767.7 | 23078.6 |

Discovery differences against incoming:
- `unknown_captures.tsv`: 5 position moves, 4 additions, 0 removals; 2938 → 2942 rows.
- `unknown_macros.tsv`: 16 position moves, 0 additions, 0 removals; 3117 → 3117 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-03-ownership-reconciliation.json`.

#### candidate-04 — retained at this step

Borrow the evaluated input and original argument tail instead of constructing a temporary Vec. Retained: failures fall to four. Value evaluation and argument validation order stay unchanged.

Failures: 4; worst recurring delta +2.142270%; no-progress streak 0. improves retained tree; observed progress tracked separately.

- `rum-cli` SHA-256 `24fcf892f9bbcf7d3304c7f77dc12a7bb72d2fed02323fad19c45e6f33dbd694`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-04/target-default`.
- `rum-cli-alloc` SHA-256 `e93dced1b0094293c367ee84f4e88d1b188771307ffa8320def11075428a870c`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-04/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | -3.270105 | -3.266165 | 19 → 16 | 1144 → 1888 | pass |
| `schema-conform-association-defaults-small` | +1.979792 | +2.159233 | 40.0004 → 38.0004 | 3139.0066 → 3905.0066 | FAIL |
| `schema-errors-list-small` | -4.103989 | -3.987855 | 21 → 18 | 1232 → 1976 | pass |
| `schema-errors-association-defaults-small` | +2.230971 | +2.142270 | 39.0001 → 37.0001 | 3161.00165 → 3927.00165 | FAIL |
| `schema-conform-list-depth-8` | -3.693738 | -3.721940 | 124 → 111 | 5297 → 7464 | pass |
| `schema-errors-list-depth-8` | -4.049031 | -4.005511 | 126.001 → 113.001 | 9087.8 → 11254.8 | pass |
| `schema-conform-list-depth-32` | -7.305140 | -7.299815 | 330 → 271 | 25821 → 33760 | pass |
| `schema-errors-list-depth-32` | -9.483571 | -9.482755 | 332.001 → 273.001 | 26834.712 → 34773.712 | pass |
| `schema-conform-list-depth-128` | -10.491225 | -10.493146 | 1196 → 947 | 152109 → 131296 | pass |
| `schema-errors-list-depth-128` | -13.606512 | -13.600911 | 1198.001 → 949.001 | 153122.712 → 132309.712 | pass |
| `schema-conform-association-depth-8` | +0.803095 | +0.197386 | 386.032 → 375.032 | 36028.528 → 38312.528 | FAIL |
| `schema-errors-association-depth-8` | +0.039461 | +0.134504 | 372.016 → 361.016 | 35852.264 → 38136.264 | pass |
| `schema-conform-association-depth-32` | -0.870933 | -0.865646 | 1347.38 → 1290.38 | 170289.048 → 179821.048 | pass |
| `schema-errors-association-depth-32` | -1.437788 | -1.407436 | 1254.001 → 1197.001 | 133640.712 → 143172.712 | pass |
| `schema-conform-association-depth-128` | -1.680768 | -1.680902 | 5101.16 → 4854.16 | 687312.888 → 691276.888 | pass |
| `schema-errors-association-depth-128` | -2.128980 | -2.129337 | 4712.001 → 4465.001 | 534536.712 → 538500.712 | pass |
| `schema-conform-association-index-33` | -0.129340 | +0.382297 | 2843.14 → 2843.14 | 327044.048 → 328804.048 | pass |
| `schema-errors-association-index-33` | +0.442253 | +0.332486 | 2810.07 → 2810.07 | 324364.524 → 326124.524 | FAIL |
| `list-literal-small` | -0.030344 | -0.022358 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | -0.000915 | -0.002798 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | +0.002612 | +0.001334 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | -0.001689 | -0.005888 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.001462 | +0.003882 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | -0.003230 | +0.006136 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | -0.001715 | -0.001846 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.025541 | -0.161070 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | +0.015658 | +0.015599 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-04-slopes.tsv`; raw paired results: `candidate-04-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-04/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.269705% to +1.719637%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-04/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1382 |
| `schema-conform-association-defaults-small` | 1421 | 2048 |
| `schema-errors-list-small` | 936 | 1444 |
| `schema-errors-association-defaults-small` | 1490 | 2110 |
| `schema-conform-list-depth-8` | 1603 | 3533 |
| `schema-errors-list-depth-8` | 1672 | 3595 |
| `schema-conform-association-depth-8` | 5448 | 8600 |
| `schema-errors-association-depth-8` | 5517 | 8662 |
| `schema-conform-association-index-33` | 18723.3 | 22962.2 |
| `schema-errors-association-index-33` | 18770.3 | 23016 |

Discovery differences against incoming:
- `unknown_attributes.tsv`: 2 position moves, 0 additions, 0 removals; 1453 → 1453 rows.
- `unknown_captures.tsv`: 11 position moves, 4 additions, 0 removals; 2938 → 2942 rows.
- `unknown_macros.tsv`: 23 position moves, 0 additions, 0 removals; 3117 → 3117 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-04-ownership-reconciliation.json`.

#### candidate-05 — rejected

Move the already-owned schema String key into SchemaEntry. Rejected: index-33 conform newly failed both samples, despite a smaller worst delta. Fully restored candidate 04 before the next edit.

Failures: 4; worst recurring delta +1.045777%; no-progress streak 0. newly regressing label: reject.

New failures: `schema-conform-association-index-33`.

- `rum-cli` SHA-256 `c1efa4e8bf6873b785222119311a89a64ece536cc3afbe05dfa49f54916ac931`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-05/target-default`.
- `rum-cli-alloc` SHA-256 `831e45d4a5a6a0eb10639e4a4222cdce2fcf52f3caafbdbaa0db4886f10d382c`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-05/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | -3.741184 | -3.747115 | 19 → 16 | 1144 → 1888 | pass |
| `schema-conform-association-defaults-small` | +1.358412 | +1.045777 | 40.0004 → 36.0004 | 3139.0066 → 3903.0066 | FAIL |
| `schema-errors-list-small` | -3.960244 | -3.797097 | 21 → 18 | 1232 → 1976 | pass |
| `schema-errors-association-defaults-small` | +1.234466 | +0.945205 | 39.0001 → 35.0001 | 3161.00165 → 3925.00165 | FAIL |
| `schema-conform-list-depth-8` | -3.625319 | -3.671102 | 124 → 111 | 5297 → 7464 | pass |
| `schema-errors-list-depth-8` | -3.977494 | -3.936478 | 126.001 → 113.001 | 9087.8 → 11254.8 | pass |
| `schema-conform-list-depth-32` | -7.196522 | -7.196242 | 330 → 271 | 25821 → 33760 | pass |
| `schema-errors-list-depth-32` | -9.358837 | -9.353069 | 332.001 → 273.001 | 26834.712 → 34773.712 | pass |
| `schema-conform-list-depth-128` | -10.376579 | -10.368281 | 1196 → 947 | 152109 → 131296 | pass |
| `schema-errors-list-depth-128` | -13.452674 | -13.454416 | 1198.001 → 949.001 | 153122.712 → 132309.712 | pass |
| `schema-conform-association-depth-8` | +0.011538 | +0.628629 | 386.032 → 367.032 | 36028.528 → 38304.528 | pass |
| `schema-errors-association-depth-8` | -0.131595 | -0.544719 | 372.016 → 353.016 | 35852.264 → 38128.264 | pass |
| `schema-conform-association-depth-32` | -1.653038 | -1.653735 | 1347.38 → 1258.38 | 170289.048 → 179789.048 | pass |
| `schema-errors-association-depth-32` | -2.493775 | -2.094523 | 1254.001 → 1165.001 | 133640.712 → 143140.712 | pass |
| `schema-conform-association-depth-128` | -2.827186 | -2.517315 | 5101.16 → 4726.16 | 687312.888 → 691148.888 | pass |
| `schema-errors-association-depth-128` | -3.288174 | -4.046015 | 4712.001 → 4337.001 | 534536.712 → 538372.712 | pass |
| `schema-conform-association-index-33` | +0.182644 | +0.153996 | 2843.14 → 2810.14 | 327044.048 → 328714.048 | FAIL |
| `schema-errors-association-index-33` | +0.245640 | +0.233876 | 2810.07 → 2777.07 | 324364.524 → 326034.524 | FAIL |
| `list-literal-small` | -0.011010 | -0.011200 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | -0.001948 | -0.478341 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | +0.003261 | -0.003639 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | -0.003276 | +0.000963 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.001893 | +0.001518 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | -0.036382 | -0.075962 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | -0.000097 | -0.000334 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.184816 | -0.018676 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | -0.002612 | +0.013690 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-05-slopes.tsv`; raw paired results: `candidate-05-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-05/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.265438% to +1.714751%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-05/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1397 |
| `schema-conform-association-defaults-small` | 1421 | 1982 |
| `schema-errors-list-small` | 936 | 1459 |
| `schema-errors-association-defaults-small` | 1490 | 2044 |
| `schema-conform-list-depth-8` | 1603 | 3582 |
| `schema-errors-list-depth-8` | 1672 | 3644 |
| `schema-conform-association-depth-8` | 5448 | 8321 |
| `schema-errors-association-depth-8` | 5517 | 8383 |
| `schema-conform-association-index-33` | 18688.4 | 21868 |
| `schema-errors-association-index-33` | 18686.9 | 21927.2 |

Discovery differences against incoming:
- `unknown_attributes.tsv`: 2 position moves, 0 additions, 0 removals; 1453 → 1453 rows.
- `unknown_captures.tsv`: 11 position moves, 4 additions, 0 removals; 2938 → 2942 rows.
- `unknown_macros.tsv`: 23 position moves, 0 additions, 0 removals; 3117 → 3117 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-05-ownership-reconciliation.json`.

#### candidate-06 — retained at this step

Use sorted indices into owned value entries, comparing borrowed String keys. Retained: failures fall to three. Running and suspended state both use Vec<usize>; generic defaults preserve unchanged test constructors with root-neutral HashMap metadata. The authorized RandomState row disappears. Sorting replaces hashing; neither ordering nor the one production/resume algorithm changes.

Failures: 3; worst recurring delta +1.962501%; no-progress streak 0. improves retained tree; observed progress tracked separately.

- `rum-cli` SHA-256 `5e7e79b804922518b1b988b194c7559d9b05392c890fb7d46ee2e6e5b90485a9`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-06/target-default`.
- `rum-cli-alloc` SHA-256 `ab1033e565542bcccea2302e727c27b98106637ac2683cf36a3d48a3e429e41b`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-06/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | -3.787292 | -4.055878 | 19 → 16 | 1144 → 1792 | pass |
| `schema-conform-association-defaults-small` | +1.912372 | +1.839759 | 40.0004 → 38.0004 | 3139.0066 → 3809.0066 | FAIL |
| `schema-errors-list-small` | -3.504019 | -3.838211 | 21 → 18 | 1232 → 1880 | pass |
| `schema-errors-association-defaults-small` | +1.962501 | +2.140564 | 39.0001 → 37.0001 | 3161.00165 → 3831.00165 | FAIL |
| `schema-conform-list-depth-8` | -3.807235 | -3.800241 | 124 → 111 | 5297 → 7176 | pass |
| `schema-errors-list-depth-8` | -4.121908 | -4.122037 | 126.001 → 113.001 | 9087.8 → 10966.8 | pass |
| `schema-conform-list-depth-32` | -7.426061 | -7.415832 | 330 → 271 | 25821 → 32320 | pass |
| `schema-errors-list-depth-32` | -9.620533 | -9.572664 | 332.001 → 273.001 | 26834.712 → 33333.712 | pass |
| `schema-conform-list-depth-128` | -10.524713 | -10.527660 | 1196 → 947 | 152109 → 125248 | pass |
| `schema-errors-list-depth-128` | -13.641943 | -13.631883 | 1198.001 → 949.001 | 153122.712 → 126261.712 | pass |
| `schema-conform-association-depth-8` | -0.008321 | -0.021566 | 386.032 → 375.032 | 36028.528 → 38024.528 | pass |
| `schema-errors-association-depth-8` | +0.571123 | -0.133019 | 372.016 → 361.016 | 35852.264 → 37848.264 | pass |
| `schema-conform-association-depth-32` | -1.456580 | -1.416670 | 1347.38 → 1290.38 | 170289.048 → 178381.048 | pass |
| `schema-errors-association-depth-32` | -0.884869 | -1.684134 | 1254.001 → 1197.001 | 133640.712 → 141732.712 | pass |
| `schema-conform-association-depth-128` | -1.561487 | -1.884456 | 5101.16 → 4854.16 | 687312.888 → 685228.888 | pass |
| `schema-errors-association-depth-128` | -2.263620 | -2.399563 | 4712.001 → 4465.001 | 534536.712 → 532452.712 | pass |
| `schema-conform-association-index-33` | -0.069844 | +0.061350 | 2843.14 → 2810.14 | 327044.048 → 325438.048 | pass |
| `schema-errors-association-index-33` | +0.094026 | +0.144093 | 2810.07 → 2777.07 | 324364.524 → 322758.524 | FAIL |
| `list-literal-small` | +0.010266 | -0.015868 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | +0.478110 | -0.005067 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | +0.003539 | +0.005203 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | +0.002215 | +0.000401 | 2 → 2 | 161 → 161 | pass |
| `table-small` | -0.001272 | -0.328416 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | -0.010528 | +0.032303 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | -0.001739 | -0.000425 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.181991 | -0.043746 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | +0.014942 | -0.002296 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-06-slopes.tsv`; raw paired results: `candidate-06-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-06/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.253750% to +1.736255%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-06/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1388 |
| `schema-conform-association-defaults-small` | 1421 | 2024 |
| `schema-errors-list-small` | 936 | 1450 |
| `schema-errors-association-defaults-small` | 1490 | 2086 |
| `schema-conform-list-depth-8` | 1603 | 3542 |
| `schema-errors-list-depth-8` | 1672 | 3604 |
| `schema-conform-association-depth-8` | 5448 | 8393 |
| `schema-errors-association-depth-8` | 5517 | 8455 |
| `schema-conform-association-index-33` | 18640.7 | 27673 |
| `schema-errors-association-index-33` | 18798.9 | 27735 |

Discovery differences against incoming:
- `unknown_attributes.tsv`: 3 position moves, 0 additions, 0 removals; 1453 → 1453 rows.
- `unknown_captures.tsv`: 8 position moves, 7 additions, 0 removals; 2938 → 2945 rows.
- `unknown_macros.tsv`: 21 position moves, 0 additions, 0 removals; 3117 → 3117 rows.
- `unknown_storage.tsv`: 0 position moves, 0 additions, 1 removals; 89 → 88 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-06-ownership-reconciliation.json`.

#### candidate-07 — retained at this step

Move the already-owned schema String key into SchemaEntry atop candidate 06. This independently retries the key move with sorted index storage; measured separately from the index change. Retained: both index labels now beat base; only the two small-default labels still fail.

Failures: 2; worst recurring delta +1.001966%; no-progress streak 0. improves retained tree; observed progress tracked separately.

- `rum-cli` SHA-256 `3508edcdd944958d798c018441bb1e92b561581fdcc395a6dcb0e439fb68e44f`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-07/target-default`.
- `rum-cli-alloc` SHA-256 `eb1c6e441b11abc0c94ce94bc3c7d733a07188272feb26c3734ed720067b3d7c`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-07/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | -4.149401 | -3.829195 | 19 → 16 | 1144 → 1792 | pass |
| `schema-conform-association-defaults-small` | +1.188024 | +0.873686 | 40.0004 → 36.0004 | 3139.0066 → 3807.0066 | FAIL |
| `schema-errors-list-small` | -4.526373 | -3.869831 | 21 → 18 | 1232 → 1880 | pass |
| `schema-errors-association-defaults-small` | +1.001966 | +1.098107 | 39.0001 → 35.0001 | 3161.00165 → 3829.00165 | FAIL |
| `schema-conform-list-depth-8` | -3.802606 | -3.852774 | 124 → 111 | 5297 → 7176 | pass |
| `schema-errors-list-depth-8` | -4.145633 | -4.176789 | 126.001 → 113.001 | 9087.8 → 10966.8 | pass |
| `schema-conform-list-depth-32` | -7.559062 | -7.498336 | 330 → 271 | 25821 → 32320 | pass |
| `schema-errors-list-depth-32` | -9.702473 | -9.731752 | 332.001 → 273.001 | 26834.712 → 33333.712 | pass |
| `schema-conform-list-depth-128` | -10.626528 | -10.615125 | 1196 → 947 | 152109 → 125248 | pass |
| `schema-errors-list-depth-128` | -13.762832 | -13.757020 | 1198.001 → 949.001 | 153122.712 → 126261.712 | pass |
| `schema-conform-association-depth-8` | -0.966960 | -0.558614 | 386.032 → 367.032 | 36028.528 → 38016.528 | pass |
| `schema-errors-association-depth-8` | -0.754049 | -1.466433 | 372.016 → 353.016 | 35852.264 → 37840.264 | pass |
| `schema-conform-association-depth-32` | -2.191184 | -1.884601 | 1347.38 → 1258.38 | 170289.048 → 178349.048 | pass |
| `schema-errors-association-depth-32` | -2.395687 | -2.798034 | 1254.001 → 1165.001 | 133640.712 → 141700.712 | pass |
| `schema-conform-association-depth-128` | -2.903565 | -2.742508 | 5101.16 → 4726.16 | 687312.888 → 685100.888 | pass |
| `schema-errors-association-depth-128` | -3.587548 | -3.585430 | 4712.001 → 4337.001 | 534536.712 → 532324.712 | pass |
| `schema-conform-association-index-33` | -0.084941 | -0.123774 | 2843.14 → 2777.14 | 327044.048 → 325348.048 | pass |
| `schema-errors-association-index-33` | -0.084134 | -0.097095 | 2810.07 → 2744.07 | 324364.524 → 322668.524 | pass |
| `list-literal-small` | -0.029239 | -0.016530 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | -0.005549 | -0.002264 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | -0.001634 | +0.002580 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | -0.000665 | +0.000956 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.003336 | +0.002522 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | -0.035591 | +0.028984 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | -0.000357 | -0.001545 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.029979 | -0.109273 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | -0.034666 | +0.014592 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-07-slopes.tsv`; raw paired results: `candidate-07-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-07/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.219520% to +1.748867%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-07/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1375 |
| `schema-conform-association-defaults-small` | 1421 | 1947 |
| `schema-errors-list-small` | 936 | 1437 |
| `schema-errors-association-defaults-small` | 1490 | 2009 |
| `schema-conform-list-depth-8` | 1603 | 3500 |
| `schema-errors-list-depth-8` | 1672 | 3562 |
| `schema-conform-association-depth-8` | 5448 | 8087 |
| `schema-errors-association-depth-8` | 5517 | 8149 |
| `schema-conform-association-index-33` | 18649.4 | 26547 |
| `schema-errors-association-index-33` | 18788.5 | 26609 |

Discovery differences against incoming:
- `unknown_attributes.tsv`: 3 position moves, 0 additions, 0 removals; 1453 → 1453 rows.
- `unknown_captures.tsv`: 8 position moves, 7 additions, 0 removals; 2938 → 2945 rows.
- `unknown_macros.tsv`: 21 position moves, 0 additions, 0 removals; 3117 → 3117 rows.
- `unknown_storage.tsv`: 0 position moves, 0 additions, 1 removals; 89 → 88 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-07-ownership-reconciliation.json`.

#### candidate-08 — retained at this step

Borrow temporary EntrySpec expressions while selecting an entry; clone only when constructing the owned child task. No entry reference survives selection or crosses evaluation/suspension. Retained: worst recurring delta falls to +0.732203% with no new failure.

Failures: 2; worst recurring delta +0.732203%; no-progress streak 0. improves retained tree; observed progress tracked separately.

- `rum-cli` SHA-256 `bb5e2db106532689a63d62e4ba339064ee8393db0f1de731f0648700970a080c`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-08/target-default`.
- `rum-cli-alloc` SHA-256 `dd5c44976db12f82b21705873f2190445d9640c951eee328d77fc00be92c7e6d`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-08/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | -3.402863 | -3.812491 | 19 → 16 | 1144 → 1792 | pass |
| `schema-conform-association-defaults-small` | +0.842101 | +0.722896 | 40.0004 → 36.0004 | 3139.0066 → 3807.0066 | FAIL |
| `schema-errors-list-small` | -3.854531 | -3.850629 | 21 → 18 | 1232 → 1880 | pass |
| `schema-errors-association-defaults-small` | +0.732203 | +0.918217 | 39.0001 → 35.0001 | 3161.00165 → 3829.00165 | FAIL |
| `schema-conform-list-depth-8` | -3.829168 | -3.844151 | 124 → 111 | 5297 → 7176 | pass |
| `schema-errors-list-depth-8` | -4.188843 | -4.140433 | 126.001 → 113.001 | 9087.8 → 10966.8 | pass |
| `schema-conform-list-depth-32` | -7.444231 | -7.447539 | 330 → 271 | 25821 → 32320 | pass |
| `schema-errors-list-depth-32` | -9.676001 | -9.656106 | 332.001 → 273.001 | 26834.712 → 33333.712 | pass |
| `schema-conform-list-depth-128` | -10.556462 | -10.570850 | 1196 → 947 | 152109 → 125248 | pass |
| `schema-errors-list-depth-128` | -13.682857 | -13.675294 | 1198.001 → 949.001 | 153122.712 → 126261.712 | pass |
| `schema-conform-association-depth-8` | -0.783038 | -1.111362 | 386.032 → 367.032 | 36028.528 → 38016.528 | pass |
| `schema-errors-association-depth-8` | -0.648052 | -0.248205 | 372.016 → 353.016 | 35852.264 → 37840.264 | pass |
| `schema-conform-association-depth-32` | -1.888074 | -2.202938 | 1347.38 → 1258.38 | 170289.048 → 178349.048 | pass |
| `schema-errors-association-depth-32` | -3.209155 | -3.184117 | 1254.001 → 1165.001 | 133640.712 → 141700.712 | pass |
| `schema-conform-association-depth-128` | -3.378311 | -2.941755 | 5101.16 → 4726.16 | 687312.888 → 685100.888 | pass |
| `schema-errors-association-depth-128` | -3.585560 | -4.024523 | 4712.001 → 4337.001 | 534536.712 → 532324.712 | pass |
| `schema-conform-association-index-33` | -0.160884 | -0.217087 | 2843.14 → 2777.14 | 327044.048 → 325348.048 | pass |
| `schema-errors-association-index-33` | -0.223322 | -0.213009 | 2810.07 → 2744.07 | 324364.524 → 322668.524 | pass |
| `list-literal-small` | +0.018296 | -0.003788 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | -0.005305 | -0.126677 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | -0.001841 | +0.001561 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | -0.002051 | -0.001827 | 2 → 2 | 161 → 161 | pass |
| `table-small` | +0.165415 | +0.006462 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | -0.021172 | +0.016886 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | -0.000381 | -0.000519 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.025403 | -0.038058 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | -0.002382 | -0.001860 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-08-slopes.tsv`; raw paired results: `candidate-08-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-08/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.252154% to +1.753348%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-08/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1381 |
| `schema-conform-association-defaults-small` | 1421 | 1899 |
| `schema-errors-list-small` | 936 | 1443 |
| `schema-errors-association-defaults-small` | 1490 | 1961 |
| `schema-conform-list-depth-8` | 1603 | 3527 |
| `schema-errors-list-depth-8` | 1672 | 3589 |
| `schema-conform-association-depth-8` | 5448 | 7933 |
| `schema-errors-association-depth-8` | 5517 | 7995 |
| `schema-conform-association-index-33` | 18677.3 | 25733 |
| `schema-errors-association-index-33` | 18793.1 | 25795 |

Discovery differences against incoming:
- `unknown_attributes.tsv`: 12 position moves, 0 additions, 0 removals; 1453 → 1453 rows.
- `unknown_captures.tsv`: 8 position moves, 7 additions, 0 removals; 2938 → 2945 rows.
- `unknown_macros.tsv`: 21 position moves, 0 additions, 0 removals; 3117 → 3117 rows.
- `unknown_storage.tsv`: 0 position moves, 0 additions, 1 removals; 89 → 88 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-08-ownership-reconciliation.json`.

#### candidate-09 — retained at this step

Collect association output from the owned entries iterator, allowing standard Vec buffer reuse; keep value order, extras, defaults and error handling unchanged. Retained: allocation measurement confirms one fewer allocation on each small-default validation; worst recurring delta falls to +0.072598% but two labels remain above the strict two-sample threshold. Candidate cap reached.

Failures: 2; worst recurring delta +0.072598%; no-progress streak 0. improves retained tree; observed progress tracked separately.

- `rum-cli` SHA-256 `bd237704d1310623255c9ff7f080191e4cd746218abce628a755d18a060d4cd4`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-09/target-default`.
- `rum-cli-alloc` SHA-256 `3e42ce7fbccae668dd2cb7a565254156768ca19134885c894f283337f9b591fc`; fresh target `/home/remi/dev/rum/.temp/brief-569/candidate-09/target-alloc`.

| Label | Sample 1 Δ% | Sample 2 Δ% | Allocations base → candidate | Bytes base → candidate | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| `schema-conform-list-small` | -3.723968 | -3.197863 | 19 → 16 | 1144 → 1792 | pass |
| `schema-conform-association-defaults-small` | +0.064430 | +0.446684 | 40.0004 → 35.0004 | 3139.0066 → 3551.0066 | FAIL |
| `schema-errors-list-small` | -3.446048 | -3.787260 | 21 → 18 | 1232 → 1880 | pass |
| `schema-errors-association-defaults-small` | +0.630371 | +0.072598 | 39.0001 → 34.0001 | 3161.00165 → 3573.00165 | FAIL |
| `schema-conform-list-depth-8` | -3.687280 | -3.599121 | 124 → 111 | 5297 → 7176 | pass |
| `schema-errors-list-depth-8` | -4.039857 | -3.966717 | 126.001 → 113.001 | 9087.8 → 10966.8 | pass |
| `schema-conform-list-depth-32` | -7.168167 | -7.203322 | 330 → 271 | 25821 → 32320 | pass |
| `schema-errors-list-depth-32` | -9.309453 | -9.309933 | 332.001 → 273.001 | 26834.712 → 33333.712 | pass |
| `schema-conform-list-depth-128` | -10.250909 | -10.248822 | 1196 → 947 | 152109 → 125248 | pass |
| `schema-errors-list-depth-128` | -13.273208 | -13.286436 | 1198.001 → 949.001 | 153122.712 → 126261.712 | pass |
| `schema-conform-association-depth-8` | -1.396562 | -1.354255 | 386.032 → 359.032 | 36028.528 → 35968.528 | pass |
| `schema-errors-association-depth-8` | -1.255611 | -1.379356 | 372.016 → 345.016 | 35852.264 → 35792.264 | pass |
| `schema-conform-association-depth-32` | -3.272255 | -3.426081 | 1347.38 → 1226.38 | 170289.048 → 170157.048 | pass |
| `schema-errors-association-depth-32` | -4.145600 | -4.142894 | 1254.001 → 1133.001 | 133640.712 → 133508.712 | pass |
| `schema-conform-association-depth-128` | -4.268543 | -4.268335 | 5101.16 → 4598.16 | 687312.888 → 652332.888 | pass |
| `schema-errors-association-depth-128` | -4.877116 | -5.084406 | 4712.001 → 4209.001 | 534536.712 → 499556.712 | pass |
| `schema-conform-association-index-33` | -0.315941 | -0.351741 | 2843.14 → 2772.14 | 327044.048 → 317412.048 | pass |
| `schema-errors-association-index-33` | -0.284182 | -0.282947 | 2810.07 → 2739.07 | 324364.524 → 314732.524 | pass |
| `list-literal-small` | -0.004259 | +0.026051 | 0 → 0 | 32 → 32 | pass |
| `association-small-literal-repeat` | +0.000454 | +0.093631 | 23 → 23 | 2176 → 2176 | pass |
| `matchq-integer-success` | +0.001808 | -0.004377 | 1 → 1 | 160 → 160 | pass |
| `matchq-integer-fail` | -0.002071 | +0.000613 | 2 → 2 | 161 → 161 | pass |
| `table-small` | -0.327487 | -0.001571 | 22 → 22 | 977 → 977 | pass |
| `table-iterator-null` | +0.073656 | -0.004496 | 0 → 0 | 32 → 32 | pass |
| `dot-three-no-input` | -0.000501 | -0.000337 | 596 → 596 | 23504 → 23504 | pass |
| `differences-no-input` | -0.024828 | -0.020960 | 13 → 13 | 704 → 704 | pass |
| `take-multispec-small` | -0.017735 | +0.015543 | 109 → 109 | 5856 → 5856 | pass |

Exact unrounded N/2N instruction slopes: `.temp/brief-569/perf/candidate-09-slopes.tsv`; raw paired results: `candidate-09-paired-results.json`; raw stdout/stderr/perf data: `perf/panels/candidate-09/`.

Attribution: 40 calibrated profiles; Callgrind total offset range +0.257792% to +1.737887%. Every perf, Callgrind and annotate command exited 0. Commands/totals and raw profiles are under `attribution/candidate-09/`.

| Attribution label | Base schema self | Candidate schema self |
| --- | ---: | ---: |
| `schema-conform-list-small` | 867 | 1405 |
| `schema-conform-association-defaults-small` | 1421 | 1928 |
| `schema-errors-list-small` | 936 | 1467 |
| `schema-errors-association-defaults-small` | 1490 | 1990 |
| `schema-conform-list-depth-8` | 1603 | 3653 |
| `schema-errors-list-depth-8` | 1672 | 3715 |
| `schema-conform-association-depth-8` | 5448 | 8192 |
| `schema-errors-association-depth-8` | 5517 | 8254 |
| `schema-conform-association-index-33` | 18658.9 | 26127 |
| `schema-errors-association-index-33` | 18729.7 | 26189 |

Discovery differences against incoming:
- `unknown_attributes.tsv`: 12 position moves, 0 additions, 0 removals; 1453 → 1453 rows.
- `unknown_captures.tsv`: 8 position moves, 8 additions, 0 removals; 2938 → 2946 rows.
- `unknown_macros.tsv`: 21 position moves, 0 additions, 0 removals; 3117 → 3117 rows.
- `unknown_storage.tsv`: 0 position moves, 0 additions, 1 removals; 89 → 88 rows.

Exact before/after identities and added rows: `.temp/brief-569/candidate-09-ownership-reconciliation.json`.

### 4. Verification: commands, bounds, exits and verbatim results

All child-process commands are bounded and serial. Common environment is shown in every recorded command. Metric/check processes use stdin `/dev/null`. Output excerpts below are copied verbatim; complete logs remain beside the command metadata under `.temp/brief-569/logs/`. Commands whose successful tools produce no text are marked as such. Failed initial formatting checks and expected stale-certificate replays are retained and disclosed.

#### `candidate-00-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-00/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 45s
```

#### `candidate-00-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-00/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 49s
```

#### `candidate-01-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-01/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 39s
```

#### `candidate-01-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-01/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 43s
```

#### `candidate-01-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 46.73s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.18s
```

#### `candidate-01-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 40; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.45s
```

#### `candidate-01-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-01-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 04s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
```

#### `candidate-02-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-02/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 45s
```

#### `candidate-02-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-02/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 41s
```

#### `candidate-02-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.35s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.17s
```

#### `candidate-02-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 40; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.40s
```

#### `candidate-02-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-02-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.22s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
```

#### `candidate-03-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-03/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 51s
```

#### `candidate-03-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-03/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```

#### `candidate-03-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 35.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.18s
```

#### `candidate-03-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 40; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.55s
```

#### `candidate-03-fmt-fixed`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-03-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 1
```

```text
Diff in /home/remi/dev/rum/crates/rum-stdlib/src/schema/machine.rs:109:
                                 &mut state.errors,
                             );
                             if state.errors.len() != first_error {
[31m-                                prefix_entry_errors(
(B[m[31m-                                    &mut state.errors[first_error..],
(B[m[31m-                                    &state.stack,
(B[m[31m-                                );
(B[m[32m+                                prefix_entry_errors(&mut state.errors[first_error..], &state.stack);
(B[m                             }
                             value
                         }
```

#### `candidate-03-schema-contract`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib tests::schema
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.45s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.11s
```

#### `candidate-03-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.28s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
```

#### `candidate-04-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-04/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### `candidate-04-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-04/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 50s
```

#### `candidate-04-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 32.96s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.23s
```

#### `candidate-04-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 40; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.83s
```

#### `candidate-04-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-04-schema-contract`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib tests::schema
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.50s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.09s
```

#### `candidate-04-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.94s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
```

#### `candidate-05-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-05/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### `candidate-05-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-05/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 50s
```

#### `candidate-05-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.19s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.19s
```

#### `candidate-05-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 40; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.67s
```

#### `candidate-05-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-05-schema-contract`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib tests::schema
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.85s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.10s
```

#### `candidate-05-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.18s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s
```

#### `candidate-06-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-06/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 43s
```

#### `candidate-06-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-06/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### `candidate-06-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 39.44s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.18s
```

#### `candidate-06-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.55s
```

#### `candidate-06-fmt-fixed`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-06-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 1
```

```text
Diff in /home/remi/dev/rum/crates/rum-stdlib/src/schema/continuation.rs:13:
 use std::sync::Arc;
 
 #[derive(FrameRootTrace, PureFrameAbandon)]
[31m-pub(crate) struct SchemaValidationSuspensionFrame<Index = std::collections::HashMap<String, usize>> {
(B[m[32m+pub(crate) struct SchemaValidationSuspensionFrame<Index = std::collections::HashMap<String, usize>>
(B[m[32m+{
(B[m     pub(crate) state: SchemaState<Index>,
     pub(crate) pending: SchemaAwait,
 }
```

#### `candidate-06-schema-contract`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib tests::schema
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.34s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.09s
```

#### `candidate-06-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.07s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
```

#### `candidate-07-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-07/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 41s
```

#### `candidate-07-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-07/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### `candidate-07-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.59s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.18s
```

#### `candidate-07-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.51s
```

#### `candidate-07-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-07-schema-contract`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib tests::schema
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.31s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.09s
```

#### `candidate-07-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.36s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
```

#### `candidate-08-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-08/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 45s
```

#### `candidate-08-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-08/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 49s
```

#### `candidate-08-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 32.27s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.19s
```

#### `candidate-08-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.58s
```

#### `candidate-08-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-08-schema-contract`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib tests::schema
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.86s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.09s
```

#### `candidate-08-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.28s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
```

#### `candidate-09-build-alloc`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-09/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

#### `candidate-09-build-default`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/candidate-09/target-default cargo build --release -p rum-cli
exit: 0
```

```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```

#### `candidate-09-depth`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib schema_depth_contract
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.54s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.18s
```

#### `candidate-09-discovery`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.43s
```

#### `candidate-09-fmt`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo fmt --check
exit: 0
```

(No output.)

#### `candidate-09-schema-contract`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --lib tests::schema
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.11s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.11s
```

#### `candidate-09-semantics`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests cargo test -p rum-stdlib --test schema_continuations
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.94s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
```

#### `final-artifact-audit`

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-tests python3 .temp/brief-569/artifact-audit.py
exit: 0
```

```text
Live/tested source: 1763 files byte-identical.
Archived source files verified: 17630
Measured candidate binaries verified: 20
Frozen base binaries: both hashes unchanged.
Tests, runtime/core production, Cargo manifests and unrelated docs preserved by the scoped source audit.
```

#### `final-certificates-default-green`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.33s
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 18.59s
```

#### `final-certificates-default-replay`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit: 101
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.69s
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 18.09s
```

#### `final-certificates-no-default-green`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.25s
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 19.05s
```

#### `final-certificates-no-default-replay`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit: 101
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.29s
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 18.44s
```

#### `final-fingerprints`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/.temp/brief-569/verification-source/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 0.26s
```

#### `final-fmt`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo fmt --check
exit: 0
```

(No output.)

#### `final-ownership-default`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 60.39s
```

#### `final-ownership-discovery-default`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 28s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 5.59s
```

#### `final-ownership-discovery-no-default`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --no-default-features --lib discovery_report_without_inventory_input -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 48.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 5.66s
```

#### `final-ownership-no-default`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-stdlib --no-default-features --lib ownership_graph -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 61.16s
```

#### `final-runtime`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification cargo test -p rum-runtime
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 07s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 26.28s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.29s
```

#### `final-workspace`

Working directory: `/home/remi/dev/rum/.temp/brief-569/verification-source`.

```text
$ python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-569/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-569/target-verification env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture
exit: 0
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5m 34s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.33s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.08s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.33s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.59s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.02s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.29s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 300.70s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 24.72s
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 43; owning edges: 3685; resolved Drop owners: 18
INDEPENDENT_AUDIT EDGE_N_FRONTIER roots=79 definitions=1049 raw=73894 deduplicated=6242 reason_classes=28 all_spans_read=true
INDEPENDENT_AUDIT EDGE_ALL roots=7214 definitions=7214 raw=47950 deduplicated=47950 reason_classes=50 all_spans_read=true
ORACLE_CONTROLS type=pass resolution=pass negative=pass wrong_claim_caught=true missing_negative_caught=true
ORACLE_TOTAL agreements=8 disagreements=0 techniques=3/3
A1_ORACLE_TOTAL agreements=287 disagreements=0
A1_ORACLE_TOTAL agreements=287 disagreements=0
METHOD_CLAIMS production=3520
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
expired-C01: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=true
expired-C02: RED stale or omitted instance certificate C02 hash=true
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1044 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1438.48s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.29s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.11s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.75s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Metric and expected-result commands are expanded in the saved manifests/scripts rather than hiding the per-process evidence. Each candidate has 27 correct expected-result checks, 108 successful allocation processes, then 216 successful instruction processes (two paired samples, base/candidate, N/2N). Every metric process has a 150-second bound.

- `base`: 27 / 27 expected results correct, all exits 0.
- `candidate-00`: 27 / 27 expected results correct, all exits 0.
- `candidate-01`: 27 / 27 expected results correct, all exits 0.
- `candidate-02`: 27 / 27 expected results correct, all exits 0.
- `candidate-03`: 27 / 27 expected results correct, all exits 0.
- `candidate-04`: 27 / 27 expected results correct, all exits 0.
- `candidate-05`: 27 / 27 expected results correct, all exits 0.
- `candidate-06`: 27 / 27 expected results correct, all exits 0.
- `candidate-07`: 27 / 27 expected results correct, all exits 0.
- `candidate-08`: 27 / 27 expected results correct, all exits 0.
- `candidate-09`: 27 / 27 expected results correct, all exits 0.
- `candidate-00`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-01`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-02`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-03`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-04`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-05`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-06`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-07`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-08`: 108 allocation and 216 instruction processes; all exits 0.
- `candidate-09`: 108 allocation and 216 instruction processes; all exits 0.

Representative exact check, allocation, and instruction records (all others retained in the same raw files):

```text
{"label": "take-multispec-small", "exit": 0, "value": "List[List[List[List[List[List[List[List[1]]]]]]]]", "expected": "List[List[List[List[List[List[List[List[1]]]]]]]]", "stderr": "", "pass": true}
```

```text
{"mode": "alloc", "sample": 1, "label": "schema-conform-list-small", "N": 20000, "build": "base", "exit": 0, "elapsed_seconds": 0.18627633700089063, "count": 381668, "bytes_total": 30769731, "peak_live_bytes": 9133341, "pass": true}
```

```text
{"mode": "instructions", "sample": 1, "label": "schema-conform-list-small", "N": 20000, "build": "base", "exit": 0, "elapsed_seconds": 0.2810294079972664, "instructions": 551463527, "pass": true}
```

Native ZeroMQ lines from the workspace log are skips, even where the Rust test wrapper reports `ok`: 

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Harness-ignored entries from the workspace run (fingerprint regeneration was explicitly run separately above):

```text
test battered_session_recovers_after_repeated_recoverable_limit_trips ... ignored
test cancel_rewrite_loop_stability_10_cycles ... ignored
test runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes ... ignored, release-only profiling gate; run with RUM_PROFILE_EVALUATION_CACHE=1
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ignored, release-only deployment-stack probe; run explicitly with --release
test associations::basic::lookup::tests::lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack drop certificate; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_flatten_round_trip_is_exact_at_physical_height_4097 ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_flatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_drop_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_take_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_keys_depth::keys_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_hit_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_miss_and_resume_drop_complete_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_values_depth::values_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_recursive_armed_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_recursive_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_scan_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_association_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_list_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_all_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_zero_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::held_association_exact_path_depth::associate_to_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::held_association_exact_path_depth::key_drop_from_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::association_assignment_read_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::association_map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::d_nested_sin_completes_at_physical_height_4097_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::merge_nested_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replace_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::replace_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replacepart_deep_path_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::rule_rhs_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::take_multispec_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::with_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::map_indexed_association_depth::map_indexed_association_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::map_indexed_association_depth::map_indexed_association_heads_refuses_in_band_rather_than_terminating ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::merge_native_stack_guard::merge_zero_native_stack_boundary_fails_in_band_and_session_recovers ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... ignored, writes a review copy; does not discharge contracts or install marks
test tests::scan_and_keypath_set_native_stack_guard::keypath_set_zero_native_stack_boundary_preserves_both_lvalue_roots ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::scan_and_keypath_set_native_stack_guard::scan_zero_native_stack_boundary_covers_each_edge_without_callbacks ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::stress_corpus::peano_arithmetic_heavy_multiplication ... ignored
test tests::stress_corpus::rule_based_differentiator_handles_heavy_generated_polynomial ... ignored
test tests::stress_corpus::tokenize_with_ini_grammar_handles_heavy_generated_config ... ignored
```

### 5. Counters, disclosures and residual attribution

10 / 10 candidates measured, incoming included. Final no-progress streak: 0. Performance work stops at the candidate cap. Retained `candidate-09` has 2 failing labels; a green test suite does not make those performance rows pass.

Final ownership reconciliation preserves every old identity except the explicitly authorized RandomState removal; all other differences are source-position changes or new closures in edited schema code. No new UnknownHostPayload or self_reaching row. Exact final additions/removal:

```text
UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	19	18	closure captures: |abort| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	44	40	closure captures: |left , right| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	68	77	closure captures: |abort| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	76	35	closure captures: |index| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	80	22	closure captures: |position| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	133	21	closure captures: |(index , (key , value))| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	32	34	closure captures: |abort| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	95	38	closure captures: |abort| (capture types/modes unresolved)
```

```text
UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	88	27	rum_stdlib::schema::state::AssociationWork.positions/std::option::Option[Own]/arg0/std::collections::HashMap[Own]/arg2: foreign default hasher representation std::collections::hash_map::RandomState
```

Standing-maintenance evidence: `.temp/brief-569/obligation-reconciliation.json`.
Standing-maintenance evidence: `.temp/brief-569/fingerprint-reconciliation.json`.
Standing-maintenance evidence: `.temp/brief-569/certificate-reconciliation.json`.

Generic default HashMap metadata exists to preserve unchanged root-contract constructors. Production and resumed frames specialize the same records to Vec<usize>; both metadata representations are root-neutral and all Expr-bearing fields/derives are shared. The temporary borrowed EntrySpec is never stored in a task or suspended frame. There is no separate flat-schema fast path.

- `schema-conform-association-defaults-small`: +0.064430% / +0.446684%; schema self 1421 → 1928 instructions. The hot profile is the flat `schema_drive` and ordinary completion/drop work; continuation self-cost is 0.
- `schema-errors-association-defaults-small`: +0.630371% / +0.072598%; schema self 1490 → 1990 instructions. The hot profile is the flat `schema_drive` and ordinary completion/drop work; continuation self-cost is 0.

The remaining small-default workload visits an existing entry and an evaluated default. The flat implementation explicitly selects Visit/Default/Next/ChildDone tasks, records and clears a waiting destination, inserts each completed child into its parent output, and owns/drops the heap record. The old recursive validator expressed those control transitions through native calls. The profiles quantify their combined schema cost; inlined machine code does not justify assigning an exact instruction total to each transition. This attribution does not prove every remaining instruction is unavoidable.

Not run: git in any form, network access, third-party-engine comparisons, timing benchmarks, unbounded test/build/metric commands, release-profile changes, new dependencies, new unsafe, or a candidate beyond the cap. No general workspace `--ignored` run was made; only the required fingerprint regeneration target was invoked explicitly with `--ignored`. Native socket execution skipped by the existing sandbox-aware tests is not claimed as exercised. No test was edited and no unrelated worktree change was reverted.

