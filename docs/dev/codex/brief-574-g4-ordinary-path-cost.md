# brief-574 — erase G4's ordinary-path cost

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

briefs 572 (`d2e0641c`) and 573 (`14f8b46c`) closed group G4: the rule
left-hand-side walk and the held matcher-pattern resolver are flat work stacks
that resume into their work. Both are correct and gated. But calls that never
suspend got dearer. Against the **pre-G4** one-codegen-unit base, from
`.temp/brief-573/perf/against-pre-G4.tsv`:

| Label | Delta, sample 1 / 2 |
| --- | --- |
| `matchq-integer-success` / `-fail` (`MatchQ[5, _Integer]`-shaped; hot) | +2.76% / +2.77%; +2.85% / +2.85% |
| `cases-pattern-child` | +2.51% / +2.35% |
| `g4-rule-blank-constraint` | +6.22% / +6.20% |
| `g4-rule-skew-depth-8` | +4.82% / +4.84% |
| `g4-matchq-skew-depth-8` | +6.27% / +6.28% |
| `g4-matchq-prepared-call` | +4.92% / +4.92% |
| `g4-matchq-pattern-head` / `-child` | +4.21%; +4.01% |
| `g4-cases-pattern-child` | +2.42% / +2.29% |
| `downvalue-definition` | +0.36% / +0.12% |

Several labels got faster (`g4-rule-wrapper` -7.9%, `g4-rule-pattern-child`
-2.9%, `g4-rule-skew-depth-32` -2.3%), and the controls sit at the base.
Allocation counts or bytes rose on the held-pattern labels (e.g.
`g4-matchq-skew-depth-8` 60 -> 63 allocations, 7 312 -> 10 224 bytes).

**Target:** every label of `.temp/brief-573/perf/labels.json` (23) within
+0.05% of the pre-G4 base (a label regresses when **both** samples exceed
+0.05%), every test unchanged and green. A residual is reported with its
attribution to work the flat walks must do that the recursions did not.

## Fixed — not negotiable in this brief

- **Both flat walks and their depth contracts.** No native recursion in
  proportion to depth; `rule_pattern_depth_contract.rs` and the private depth
  tests stay unchanged and green. **No regression is bought back by restoring
  recursion.**
- **Semantics.** `crates/rum-stdlib/tests/rule_pattern_continuations.rs` and
  its children, and every other test, stay exactly as they are.

## Where the director would look first — hypotheses, measure each

1. **`MatchQ[5, _Integer]` costs ~140 more instructions and touches neither
   walk's interesting part** — the pattern is a bare `Blank`. In
   `resolve_held_matcher_pattern`
   (`kernel_data/rewrite_data/cases/pattern_resolution.rs`), the root is
   cloned out of `strip_hold_pattern`, a state with a `Vec` is built, the loop
   is entered, and the now-iterative `contains_pattern_syntax` runs its loop.
   A root that is itself a recognized pattern head, or contains no pattern
   syntax, needs none of that — but **no second resolution algorithm**: the
   same driver, entered without building what it does not need.
2. **Clones made only for the error edge.** The evaluate branches clone the
   expression on the success path (`self.evaluate(expr.clone())`) so a
   suspension can capture it; recover it in the cold capture helper instead.
   brief-566 and brief-569 removed the same shape from `LazyTable` and the
   schema walk.
3. **Vector growth.** The skewed-depth labels reallocate the work stack and
   prefix vectors (bytes 3 256 -> 9 048 for the rule skew); reserve from
   known arity, reuse buffers, or avoid the prefix for untouched arguments.
4. **The rule walk's short-rule transitions** (`definitions/rule_lhs.rs`):
   brief-572 §4 attributes the `Blank` rule's cost to driver self-cost (562),
   value acceptance (234), completion (96) and work-`Vec` drop (58), against a
   recursive helper of 278.

**Attribute before you change**: `valgrind --tool=callgrind` on base and
candidate, each total calibrated against `perf stat` on the same binary.

## Measurement

- **Base:** `.temp/brief-572/base/rum-cli` and `rum-cli-alloc` (pre-G4, one
  codegen unit; hashes in `sha256.txt`); do not rebuild them.
- **Candidates:** the working tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`,
  default and `--features rum-cli/alloc-count` (two arguments), into a fresh
  target directory each under `.temp/brief-574/`; check the hash changed; keep
  every measured binary, its hash and its source. Measure the incoming tree once
  the same way as candidate 00.
- **Panel:** `.temp/brief-573/perf/` — `labels.json`, workloads and checks
  (check each result first). brief-573's scripts are a working harness to copy.
- **Method:** allocation counts first, then two paired samples of `taskset -c
  2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
  `/dev/null`, `(count(2N) - count(N)) / N`, raw output saved before parsing.
  Any candidate you keep, and the final one, runs all 23 labels. No timing.

## Scope

- **Objective:** the ordinary-path shape of `definitions/rule_lhs.rs` and
  `rule_lhs/`, `cases/pattern_resolution.rs` and `pattern_resolution/`, the
  delegating code in `definitions/definitions.rs` and `cases/match_q.rs`, and
  the two iterative helpers (`definition_lhs_contains_pattern_syntax`,
  `contains_pattern_syntax`, `strip_hold_pattern`).
- **Standing duty — always in scope** (`TEMPLATE.md`): replay the fingerprints
  and both certificates, reconcile the ownership inventories, keep
  rum-runtime's frame lists exact. New `UnknownConstruct` rows for code you add
  or change are expected and disclosed; STOP on any other changed or vanished
  existing row, a new `UnknownHostPayload` or `self_reaching` row, or a row in
  code you did not touch.
- **Fence:** no semantic change and no test edit; no change to the matcher
  proper (compile, match, codecs), the normalizer, the evaluator or `Expr`; no
  other suspension group; no dependency, `unsafe`, depth cap or recursion; the
  release profile unchanged. No edits to `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  candidate — what it changed, its attribution, its rows.

## rum / Rust pitfalls (read twice)

1. **One variable per candidate.**
2. **A candidate that makes another label regress is rejected.**
3. **Instruction counts decide.**
4. **Stop-loss is keyed on progress:** the number of regressing labels or the
   largest remaining delta must fall. Stop after two consecutive compiling
   candidates that reduce neither; at most ten measured candidates. **Use the
   budget**: stopping early while candidates still make progress needs a
   stated reason.
5. A second ordinary code path beside the general one needs a falsifier
   proving both agree on every semantic case, or do not add it.
6. `#[inline(always)]` is not a fix by itself (brief-560: +1.17%).

## Verification (MANDATORY — you run it; iterate to green)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-574/tmp …`. Bounds: focused 150 s; a one-unit
release build 900 s; a metric process 150 s; runtime and retention 300 s;
`cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test rule_pattern_continuations
cargo test -p rum-stdlib --lib rule_pattern_depth_contract
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

Plus the retention replay in both feature sets. Paste commands, bounds, exit
statuses and counts verbatim; list native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-574/`. After each candidate write
`.temp/brief-574/PROGRESS.md`: its change, attribution, rows, counters, the
exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads and
  edits within scope, `rg`/`grep`, `python3`, bounded `cargo`, the release
  `rum-cli`, bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind` with
  `callgrind_annotate`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. The attribution for each regressing family, base against incoming.
3. Every candidate measured, with its rows (all 23 for kept candidates), both
   samples.
4. Verification — every command's verbatim result.
5. Progress counters, disclosures, and the attribution of anything left.

---

## Execution report

The +0.05% target is **not met**. The retained tree is **candidate-09**: 10/23 labels regress in both samples; the largest remaining sample is +5.097739%. All ten allowed candidates, including incoming candidate-00, were measured. Tuning stopped at the candidate limit.

Both flat walks and their saved frame layouts remain intact. No test source, matcher, normalizer, evaluator, Expr implementation, dependency, release profile, depth cap, or unsafe code was changed. No git command was run; no network access was used. Scratch, complete logs, measured binaries, source snapshots and raw results are under `.temp/brief-574/`.

### 1. Changed files

| File | Final line ranges | Reason |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs` | 112–117, 119–125 | Peel the root of the named definition classifier before setting up its unchanged iterative DFS. |
| `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 76–81, 146–164 | Reuse the already computed pattern-head selector for policy dispatch; retain the canonical wrapper helper in a debug-only consistency assertion. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 22, 25–26, 34, 38, 48–49, 58, 61, 86–90, 99–100, 103–124, 164–169, 171–177 | Recover evaluate-only raw expressions at cold suspension capture, peel the classifier root, and retain one flat resolver/normalization path. Complete ordinary roots without an acceptance/Done round trip; resumed Done values reach that same completion point. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs` | 36 | Supply no entry-root borrow when resuming the existing saved work stack. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 854 | Replay source positions only; existing closure identities/dispositions remain unchanged. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 261–264, 316–323, 677–683 | Replay existing positions and disclose new UnknownConstruct rows only in touched source. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 2–69 | Install exactly 68 regenerated fingerprint cells; all other contract fields remain unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 57, 66, 103, 147, 151 | Update five evidence positions to the same unchanged work-stack push statement. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 2–3 | Install the exact two certificate cells in each feature set; only fingerprint tokens change. |
| `docs/dev/codex/brief-574-g4-ordinary-path-cost.md` | 172 onward | This required execution report. |

The complete before/after source patch and per-hunk ranges are in `.temp/brief-574/final-source.patch` and `final-source-audit.json`. The initial file snapshot preserves the supplied dirty-tree state. Rejected candidates were restored from those snapshots, never by a repository reset. Both closed runtime frame lists and every test source are byte-identical to the initial snapshot. Final production source equals the retained measured candidate, and the verification export equals live crate files.

### 2. Attribution: pre-G4 against incoming

Callgrind used N=10 and 20 and self-instruction slopes. Before every profile, native `taskset -c 2 perf stat -x, -e cpu_core/instructions/u` ran the identical binary and input file. All totals and calibration percentages are retained in each `candidate-NN/attribution/calibration.tsv`; command lines, stdout/stderr, raw callgrind files and complete `callgrind_annotate` output are adjacent. The short total slopes include startup/interning variability and do not decide acceptance; the full-N paired native panel does.

The family below sums self cost in definition functions, held-pattern resolution and MatchQ functions (including their work-Vec drop symbols). It does not claim to be total program cost.

| Label | Pre-G4 family self/iteration | Incoming family self/iteration | Difference |
| --- | ---: | ---: | ---: |
| `g4-rule-blank-constraint` | 1227 | 1929 | +702 |
| `g4-rule-skew-depth-8` | 11116 | 13110 | +1994 |
| `matchq-integer-success` | 401 | 543 | +142 |
| `matchq-integer-fail` | 400 | 542 | +142 |
| `cases-pattern-child` | 1795 | 2400 | +605 |
| `replaceall-simple` | 1325 | 1325 | +0 |
| `g4-matchq-pattern-child` | 1706 | 2311 | +605 |
| `g4-matchq-pattern-head` | 1774 | 2321 | +547 |
| `g4-cases-pattern-child` | 1795 | 2400 | +605 |
| `g4-matchq-prepared-call` | 2742 | 3251 | +509 |
| `g4-matchq-skew-depth-8` | 7095 | 8729 | +1634 |
| `downvalue-definition` | 4886 | 4914 | +28 |

Bare Blank: the old resolver and classifier cost 57 and 55 self instructions; incoming driver cost is 226, acceptance 25 and empty work-Vec drop 26, with entry-function differences giving a net family increase of 142. Success and failure share this cost. Rule Blank: old recursive helper 278 versus driver 770, finish 96 and work-Vec drop 58; the definition classifier is 66→122. Rule depth-8: old recursive helper 4,483 versus driver 7,619, partly offset by classifier self cost falling 5,867→4,531.

Active held children/Cases: old resolver/classifier 368+287 versus incoming driver/acceptance 932+322 and work-Vec drop 29. Active heads: 301+311 versus 902+251 and drop 29. Prepared calls: 311+272 versus 813+255 and drop 29. Held depth-8: 1,831+4,220 versus 6,184+1,495 and drop 29; Atom clone self-cost increases by 486. These are the explicit destination, prefix, work-stack and transition operations that replaced native recursive control.

Incoming downvalue-definition was additionally profiled even though the fresh candidate-00 panel did not qualify it as a two-sample regression: its definition-classifier family delta is +28. The variable replaceall-simple control has identical family self cost (1,325→1,325); its short-profile differences occur in interning/hash/allocator work, not a changed G4 walk. No sample was discarded or repeated. The 11→10 regression-count reduction includes that variable control and is not claimed as a causal G4 fix.

| Candidate | Calibrated profiles | Min callgrind/native difference | Max difference |
| --- | ---: | ---: | ---: |
| candidate-00 | 48 | +1.275898% | +1.851321% |
| candidate-01 | 12 | +1.364048% | +1.867846% |
| candidate-02 | 12 | +1.330658% | +1.728475% |
| candidate-03 | 12 | +1.292767% | +1.769982% |
| candidate-04 | 20 | +1.337562% | +1.819165% |
| candidate-05 | 24 | +1.342855% | +1.922098% |
| candidate-06 | 20 | +0.776090% | +1.746519% |
| candidate-07 | 40 | +1.310968% | +1.865714% |
| candidate-08 | 16 | +0.785332% | +1.798117% |
| candidate-09 | 40 | +1.254632% | +1.859055% |

### 3. Every measured candidate

Each candidate has a fresh default release target and a separate fresh alloc-count target, both with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`. The alloc feature is passed as the two arguments `--features rum-cli/alloc-count`. Both binaries differ from the base and the preceding measured candidate; retained SHA256 values follow. The fixed base binaries were never rebuilt and their hashes remain:

```text
rum-cli       88d29a74f1726412e82e89717e18ca42cc81f3a727ea088b14d356bc63de43a1
rum-cli-alloc e6e4f9161ed454cb946015ad1e0bd8f012e5e7e23fd678245e8f4d81f6564628
```

Every table is the verbatim reducer output saved as `candidate-NN/perf/rows.tsv`: allocation count/byte slopes first, followed by native instruction slopes and percentage deltas in both samples. Each slope is `(count(2N)-count(N))/N`. A base regression requires both deltas >+0.05%; candidate rejection also conservatively checks >+0.05% in both candidate-versus-parent comparisons. Checks ran first (46 per candidate), then 92 allocation processes, then 92 instruction processes for each of two samples, serially with stdin `/dev/null` and CPU 2 for perf. There were no timing gates.

#### candidate-00 — incoming

Incoming G4 tree, unchanged; measured once against the fixed pre-G4 binaries.

Incoming measurement, not an optimization.

11 regressions; largest remaining delta +6.282014%.

```text
rum-cli 6c71f7bf5fdc4a64010ab235da26a077d31ff97c98fd04b879fb783d76e4c81d
rum-cli-alloc 710643271b5f783dddc5aa057f161529310d1ebd05a7cdb083d0b75237e46f1d
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.045750	999.032000	-0.001376	999.100500	999.094250	-0.000626	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.084000	998.981400	-0.010269	999.125650	999.005900	-0.011985	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22076.334650	21437.032000	-2.895873	22077.615500	21434.488900	-2.913026	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.583250	21139.829550	-2.977494	21787.005150	21140.711800	-2.966417	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.678400	18253.018950	6.216820	17184.650750	18253.316450	6.218722	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26310.186050	24226.759750	-7.918706	26310.361950	24226.791900	-7.919199	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59723.274000	62610.958000	4.835107	59726.968000	62618.780000	4.841719	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217265.846000	212349.513000	-2.262819	217268.585000	212355.564000	-2.261266	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.277350	5286.277000	2.760342	5144.385700	5286.313500	2.758887	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.385450	5416.222300	2.845155	5266.234350	5416.228450	2.848223	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412418.491000	2412582.160000	0.006784	2412447.515000	2412578.042000	0.005411	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39510.508000	40386.324650	2.216668	39576.387200	40386.249400	2.046327	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24152.238000	24150.083600	-0.008920	24144.036250	24150.170700	0.025408	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11381.585750	11449.014700	0.592439	11381.639550	11396.927700	0.134323	True
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.235750	254204.164300	0.002726	254197.278300	254203.770950	0.002554	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.681650	10034.393600	-0.032757	10044.841500	10033.890350	-0.109023	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.698450	49949.626300	-0.002146	49950.676500	49949.340700	-0.002674	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.476300	22969.058600	4.010158	22083.340750	22969.252300	4.011674	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.442500	22064.480450	4.208281	21173.568350	22064.716050	4.208774	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39443.469550	40363.531500	2.332609	39479.911900	40385.579000	2.293995	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.301800	16019.406950	4.919376	15268.611200	16019.494300	4.917822	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52479.510000	55776.280000	6.282014	52480.746000	55774.569000	6.276250	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204435.193000	203196.446000	-0.605936	204433.395000	203199.512000	-0.603562	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-00_total	base_rule_self	candidate-00_rule_self	candidate-00_continuation_self
g4-rule-blank-constraint	17276.7	18138.7	1227.0	1929.0	0
g4-rule-skew-depth-8	60057.2	62216.3	11116.0	13110.0	0
matchq-integer-success	5177.5	5381.0	401.0	543.0	0
matchq-integer-fail	5352.1	5478.0	400.0	542.0	0
cases-pattern-child	39357.4	40688.0	1795.0	2400.0	0
replaceall-simple	11827.8	11708.8	1325.0	1325.0	0
g4-matchq-pattern-child	22499.7	22916.9	1706.0	2311.0	0
g4-matchq-pattern-head	20797.1	22423.3	1774.0	2321.0	0
g4-cases-pattern-child	40065.1	40899.4	1795.0	2400.0	0
g4-matchq-prepared-call	15074.6	16193.0	2742.0	3251.0	0
g4-matchq-skew-depth-8	52273.0	55441.9	7095.0	8729.0	0
downvalue-definition	24164.6	24119.2	4886.0	4914.0	0
```

#### candidate-01 — kept

Move evaluated held-pattern expressions into evaluate; recover raw root/child expressions only on the cold suspension edge. A proposed new helper first failed the unchanged structural envelope and was folded into the existing capture helper before the measured build.

No neighbor regression; largest remaining delta and regression count both fell.

10 regressions; largest remaining delta +6.225429%.

```text
rum-cli 6c1ad4466672ae98fed9ac8ca9dfaa552e1f3d4ce4776475255404271423a886
rum-cli-alloc 1f6f36454a70ab8b32653edc7e146f36926b02de9277a9b0d44d814e94e485db
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.188400	998.950450	-0.023814	998.875300	999.086700	0.021164	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.031800	999.061550	0.002978	999.114200	999.019100	-0.009518	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22079.905800	21440.166300	-2.897383	22076.907300	21436.336550	-2.901542	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21786.874450	21139.969200	-2.969243	21786.423600	21143.914500	-2.949126	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.814700	18254.643200	6.225429	17184.835150	18251.758300	6.208515	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26308.267550	24226.750850	-7.912025	26310.232550	24228.927600	-7.910629	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59723.650000	62605.933000	4.826033	59727.449000	62609.521000	4.825373	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217265.160000	212341.923000	-2.266004	217267.502000	212360.038000	-2.258720	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.261200	5287.333300	2.781198	5144.540050	5287.280100	2.774593	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.352000	5417.263100	2.865572	5266.099900	5417.240550	2.870068	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412444.779000	2412610.245000	0.006859	2412504.843000	2412688.329000	0.007606	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39480.930300	40247.027550	1.940424	39594.332900	40325.191400	1.845866	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.215800	24112.090500	-0.041975	24121.996050	24120.000750	-0.008272	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.726900	11396.974000	-0.094260	11433.312150	11397.123600	-0.316519	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.282600	254204.253950	0.002742	254200.527250	254203.710400	0.001252	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10030.672950	10035.616750	0.049287	10030.731600	10035.313850	0.045682	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49958.036500	49949.339350	-0.017409	49942.190400	49949.738850	0.015114	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.704850	22860.255100	3.516395	22083.623850	22860.135700	3.516234	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.471800	21995.738650	3.883477	21173.697450	21995.879100	3.883033	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39621.895800	40371.827800	1.892721	39497.978750	40278.457400	1.975996	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.309800	15956.193750	4.505305	15268.609000	15956.305250	4.503988	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52483.772000	55301.289000	5.368358	52479.678000	55297.027000	5.368457	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204436.122000	201103.175000	-1.630312	204436.304000	201106.341000	-1.628851	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-01_total	base_rule_self	candidate-01_rule_self	candidate-01_continuation_self
matchq-integer-success	5141.4	5382.4	401.0	544.0	0
g4-matchq-pattern-child	22184.9	23236.1	1706.0	2276.0	0
g4-matchq-skew-depth-8	52493.3	55213.6	7095.0	8569.0	0
```

#### candidate-02 — rejected

Select RuleLhsPolicy from the already computed MatcherPatternHead. Rejected for a new downvalue-definition regression; its unused-helper/import warnings are retained in the build logs.

New downvalue-definition regression in both samples; candidate rejected despite rule-family improvement.

11 regressions; largest remaining delta +5.361136%.

```text
rum-cli cb4c661dd95e2cdd070f88f615e97dd514925725550425fc8708a92364afefc1
rum-cli-alloc 43dbeea5ec761a63114e9aa9f5520d3f7b60a226bf4dbc07668f03f90ffdd4cb
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.898800	999.137750	0.023921	998.929900	998.850500	-0.007949	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.073500	999.017650	-0.005590	999.056450	999.080700	0.002427	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22075.631400	21319.077950	-3.427098	22075.636700	21320.304450	-3.421565	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.644150	21023.813800	-3.510225	21786.940700	21025.152850	-3.496534	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17185.886950	18099.139750	5.313970	17185.596000	18100.766400	5.325218	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26312.367350	23680.905050	-10.000857	26308.148850	23680.904350	-9.986429	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59724.888000	61600.487000	3.140398	59726.089000	61599.545000	3.136746	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217265.364000	208895.244000	-3.852487	217272.836000	208891.375000	-3.857574	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.312850	5287.280850	2.779147	5144.273750	5287.262250	2.779566	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.400700	5409.364150	2.714633	5266.201000	5409.303900	2.717384	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412444.434000	2412568.024000	0.005123	2412541.652000	2412605.733000	0.002656	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39580.590650	40143.422150	1.421989	39477.980950	40254.503800	1.966977	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24121.980150	24150.057550	0.116398	24083.992100	24150.048600	0.274276	True
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.673500	11383.894700	-0.208446	11459.818850	11423.013550	-0.321168	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254200.652650	254204.169850	0.001384	254197.393600	254203.687450	0.002476	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10052.108300	10034.360950	-0.176554	10051.539950	10035.142300	-0.163136	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49941.812000	49949.383200	0.015160	49941.864150	49949.341200	0.014972	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.371400	22860.156200	3.517510	22083.597500	22860.210700	3.516697	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.648700	21995.775150	3.882781	21173.675550	21995.549400	3.881583	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39537.036700	40313.288500	1.963354	39537.824300	40223.588600	1.734451	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.448050	15956.188450	4.504324	15268.417750	15956.249700	4.504933	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52479.885000	55293.403000	5.361136	52480.756000	55290.418000	5.353700	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204435.128000	201098.987000	-1.631882	204434.485000	201093.686000	-1.634166	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-02_total	base_rule_self	candidate-02_rule_self	candidate-02_continuation_self
g4-rule-blank-constraint	17226.5	18036.8	1227.0	1893.0	0
g4-rule-skew-depth-8	59914.6	61606.8	11116.0	12378.0	0
downvalue-definition	24395.5	24319.0	4886.0	4914.0	0
```

#### candidate-03 — kept

Peel the root of definition_lhs_contains_pattern_syntax before setting up its existing iterative DFS. Atom and recognized-root decisions remain the same classifier.

Maximum residual fell; no new base regression or both-sample parent worsening.

10 regressions; largest remaining delta +5.992181%.

```text
rum-cli e6ee5884d536d31dd769f4ff5573fef1b7e30b5f2de083b6fdc0086f69577c56
rum-cli-alloc eaef701d3c6348cbe22a948014f9719cc02f8433bdcf9102d6694c223398989b
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.050850	999.156550	0.010580	999.006050	999.266700	0.026091	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	998.980450	999.112200	0.013188	999.135900	999.026900	-0.010909	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22077.020550	21349.324050	-3.296172	22076.859950	21350.863800	-3.288494	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21786.380050	21050.755750	-3.376533	21861.954100	21056.709200	-3.683316	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.377700	18212.299350	5.981722	17184.253000	18213.964500	5.992181	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26310.494550	24089.834400	-8.440207	26312.570850	24089.767800	-8.447685	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59720.362000	62222.207000	4.189266	59728.582000	62212.828000	4.159225	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217273.451000	210244.652000	-3.235001	217267.732000	210255.720000	-3.227360	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.287000	5287.293050	2.779900	5144.329900	5287.313650	2.779444	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.280850	5417.224200	2.866223	5266.295700	5417.423250	2.869713	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412441.621000	2412579.031000	0.005696	2412444.759000	2412651.300000	0.008561	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39471.978050	40302.468100	2.103999	39546.216900	40260.752100	1.806836	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24099.884450	24129.935400	0.124693	24121.831300	24130.075700	0.034178	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11381.596050	11370.844200	-0.094467	11407.697500	11396.880500	-0.094822	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.353600	254204.293900	0.002730	254200.590500	254204.074700	0.001371	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.701100	10033.967100	-0.037200	10030.627750	10033.907900	0.032701	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49966.706250	49948.931400	-0.035573	49958.142400	49950.009450	-0.016280	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.531450	22860.143700	3.516703	22083.727100	22860.245350	3.516246	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.721800	21995.854250	3.882796	21173.646550	21995.640650	3.882157	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39446.424200	40259.831950	2.062057	39463.366650	40249.615900	1.992352	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.386300	15956.302850	4.505496	15268.468100	15956.342650	4.505197	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52479.572000	55299.094000	5.372609	52485.093000	55300.773000	5.364723	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204434.077000	201105.370000	-1.628254	204431.653000	201107.394000	-1.626098	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-03_total	base_rule_self	candidate-03_rule_self	candidate-03_continuation_self
g4-rule-blank-constraint	17218.4	18356.1	1227.0	1889.0	0
g4-rule-skew-depth-8	59646.7	62341.1	11116.0	12711.0	0
downvalue-definition	24311.0	24425.4	4886.0	4894.0	0
```

#### candidate-04 — rejected

Borrow every held visit from retained root/parent work and carry optional ownership at completion. Rejected: active child, active head and prepared-call instruction counts worsened versus the parent.

Active child, active head and prepared-call labels worsen >0.05% in both parent comparisons.

10 regressions; largest remaining delta +5.987804%.

```text
rum-cli 78086dd11e3bcac94ad6caf0742f238c23b7bf960a2718215a92b50a9fc4341c
rum-cli-alloc 1dd54b2cbc242f5dced434d23be680e8817f5856366c15fb1ed60d3fc57eedf3
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.094800	998.990300	-0.010459	998.974950	998.825150	-0.014995	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.078400	998.996550	-0.008193	998.998300	999.154250	0.015611	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22078.439600	21349.376750	-3.302148	22076.864900	21348.191350	-3.300621	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21787.169800	21053.509700	-3.367395	21788.494300	21051.512450	-3.382436	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17185.395500	18212.368450	5.975847	17185.165300	18214.179400	5.987804	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26310.306450	24087.110550	-8.449905	26308.241100	24089.909350	-8.432079	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59725.301000	62225.882000	4.186804	59725.084000	62213.883000	4.167092	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217275.192000	210244.778000	-3.235719	217268.659000	210256.289000	-3.227511	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.320100	5250.284400	2.059831	5144.384100	5250.223250	2.057373	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.239550	5380.252750	2.164983	5266.222000	5380.329600	2.166783	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412442.808000	2412543.567000	0.004177	2412505.403000	2412605.023000	0.004129	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39431.904850	40298.539000	2.197799	39463.120950	40271.206500	2.047698	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.049600	24290.020300	0.696337	24182.027600	24099.938800	-0.339462	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11446.743700	11396.845000	-0.435920	11394.645600	11370.996500	-0.207546	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254200.644000	254204.018250	0.001327	254196.697500	254203.658950	0.002739	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10052.070500	10033.578200	-0.183965	10052.143100	10033.253500	-0.187916	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49966.500550	49949.673100	-0.033677	49966.759850	49949.288850	-0.034965	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.437650	22884.182200	3.625996	22083.374850	22884.220200	3.626463	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.625850	22012.642550	3.962556	21173.567300	22012.700100	3.963115	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39430.072150	40240.855650	2.056257	39480.119000	40397.012750	2.322419	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.413350	15965.338900	4.564492	15268.393700	15965.364750	4.564796	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52479.522000	55128.569000	5.047773	52483.896000	55128.184000	5.038285	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204433.180000	199985.124000	-2.175799	204432.127000	199985.509000	-2.175107	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-04_total	base_rule_self	candidate-04_rule_self	candidate-04_continuation_self
matchq-integer-success	5068.5	5519.3	401.0	497.0	0
g4-matchq-pattern-child	22302.4	22795.1	1706.0	2281.0	0
g4-matchq-pattern-head	21589.6	21926.5	1774.0	2289.0	0
g4-matchq-prepared-call	15735.8	15579.5	2742.0	3220.0	0
g4-matchq-skew-depth-8	52997.6	55095.9	7095.0	8378.0	0
```

#### candidate-05 — kept

Reuse the cached Rule pattern-head selector on candidate-03. A debug-only equality assertion retains and checks the canonical wrapper helper; release builds have no warning and no assertion cost.

Cached Rule policy reduces the maximum to 5.351806%; no new base regression or qualifying parent worsening; builds have no warnings.

10 regressions; largest remaining delta +5.351806%.

```text
rum-cli 94c2b4806f55adb6e2bbfba39b4b4ae6883cb79189a76c4e68a9e6f04d7ba148
rum-cli-alloc 1a4eddeb79246e9eea1207cd6897f59bc1148bc5ecad5c848431cd2af5f55d85
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.947900	998.991800	0.004395	998.902500	998.878200	-0.002433	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	998.937300	999.145100	0.020802	999.156850	999.091500	-0.006541	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22076.935050	21231.531500	-3.829352	22078.485750	21232.025250	-3.833870	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21786.915550	20938.157000	-3.895726	21785.452200	20937.464350	-3.892450	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17186.337600	18058.420300	5.074279	17185.264850	18058.540800	5.081539	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26310.275600	23541.383950	-10.523993	26308.086250	23543.725500	-10.507647	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59722.035000	61213.755000	2.497772	59723.895000	61200.697000	2.472715	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217269.789000	206789.478000	-4.823639	217264.669000	206791.350000	-4.820535	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.237000	5287.274750	2.780544	5144.180950	5287.444000	2.784954	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.158700	5409.296200	2.718063	5266.373950	5409.108350	2.710297	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412443.082000	2412569.831000	0.005254	2412506.251000	2412569.956000	0.002641	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39567.996000	40253.069950	1.731384	39470.906000	40396.172700	2.344174	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.140200	24190.101350	0.281738	24121.968900	24129.895200	0.032859	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11381.683700	11370.919400	-0.094576	11459.718350	11396.975150	-0.547511	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.052700	254203.455600	0.002519	254197.449650	254204.057050	0.002599	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10038.211250	10034.032300	-0.041630	10052.108250	10035.647250	-0.163757	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49966.828250	49949.132200	-0.035416	49942.215550	49949.394650	0.014375	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.311500	22860.088900	3.517486	22083.475800	22860.081000	3.516680	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.856150	21995.757750	3.881681	21173.675550	21995.659700	3.882104	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39479.456300	40314.306900	2.114646	39488.330500	40248.878300	1.926006	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.213150	15956.220300	4.506141	15268.333650	15956.288400	4.505762	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52479.379000	55287.706000	5.351296	52482.138000	55290.880000	5.351806	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204432.499000	201094.059000	-1.633028	204427.588000	201097.186000	-1.629135	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-05_total	base_rule_self	candidate-05_rule_self	candidate-05_continuation_self
g4-rule-blank-constraint	17227.3	18118.3	1227.0	1853.0	0
g4-rule-skew-depth-8	59885.3	61479.1	11116.0	11979.0	0
downvalue-definition	24260.6	24637.0	4886.0	4894.0	0
matchq-integer-success	5047.6	4995.7	401.0	544.0	0
g4-matchq-pattern-child	22626.9	23013.3	1706.0	2276.0	0
g4-matchq-skew-depth-8	52442.5	55239.5	7095.0	8569.0	0
```

#### candidate-06 — rejected

Keep active Rule work in an Option outside the ancestor vector and materialize it into the existing state at suspension. Rejected: fewer allocations but worse wrapper and skew instruction counts.

The active-record Option removes allocations but worsens wrapper and skewed Rule instruction counts; rejected under the neighbor rule.

10 regressions; largest remaining delta +5.364607%.

```text
rum-cli 191f6b97c0e3776796b39022fae0e09d7fbb7da2040bb9e0e88d3b37ebd92a97
rum-cli-alloc 622bef0135ec41374cf9daa2d16affa73e5803d0fefdc1338462f1250caf59de
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.033200	999.004050	-0.002918	999.175200	998.969600	-0.020577	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.021450	998.853450	-0.016816	998.983600	998.988050	0.000445	False
g4-rule-pattern-child	20000	25.000000	15.000000	1400.000000	920.000000	22078.395150	21254.253200	-3.732798	22079.821600	21251.632850	-3.750885	False
g4-ruledelayed-pattern-child	20000	25.000000	15.000000	1400.000000	920.000000	21786.457900	20955.148550	-3.815716	21788.390450	20960.256350	-3.800804	False
g4-rule-blank-constraint	20000	16.000000	16.000000	1112.000000	920.000000	17185.455100	17636.524150	2.624714	17184.670650	17636.616150	2.629934	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26308.150400	24417.990250	-7.184694	26310.274300	24417.857800	-7.192690	False
g4-rule-skew-depth-8	1000	73.000000	67.000000	3256.000000	5464.000000	59730.545000	62536.517000	4.697717	59724.430000	62543.264000	4.719734	True
g4-rule-skew-depth-32	1000	217.000000	215.000000	9016.000000	21976.000000	217278.411000	213894.481000	-1.557417	217276.986000	213883.451000	-1.561847	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.314300	5287.359050	2.780638	5144.264750	5287.459000	2.783571	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.119650	5409.382550	2.720464	5266.210850	5409.196050	2.715144	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412445.855000	2412627.716000	0.007538	2412541.824000	2412629.611000	0.003639	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39511.064900	40374.089350	2.184260	39494.242250	40192.086800	1.766953	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.087900	24129.973250	0.032689	24122.030250	24205.906100	0.347715	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11446.517050	11383.898650	-0.547052	11407.484750	11397.058200	-0.091401	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.117550	254203.798350	0.002628	254197.700750	254204.284050	0.002590	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.928450	10033.787000	-0.041258	10051.963650	10033.956100	-0.179145	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49941.633650	49949.774300	0.016300	49966.533500	49949.920300	-0.033249	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.399000	22860.301800	3.518040	22083.404300	22860.226850	3.517676	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.585000	21995.677500	3.882633	21173.536700	21995.676050	3.882863	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39590.649000	40206.543650	1.555657	39493.126300	40318.975050	2.091120	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.500000	15956.398500	4.505344	15268.382550	15956.229750	4.505043	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52477.452000	55292.661000	5.364607	52480.899000	55289.182000	5.351057	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204432.405000	201093.953000	-1.633035	204436.259000	201098.385000	-1.632721	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-06_total	base_rule_self	candidate-06_rule_self	candidate-06_continuation_self
g4-rule-blank-constraint	17194.0	17648.0	1227.0	1941.0	0
g4-rule-pattern-child	22527.1	21590.1	2164.0	3239.0	0
g4-rule-wrapper	26227.7	24506.9	3550.0	4951.0	0
g4-rule-skew-depth-8	59973.2	62391.3	11116.0	14237.0	0
g4-rule-skew-depth-32	217415.0	214249.4	80500.0	75185.0	0
```

#### candidate-07 — kept

Peel the held contains_pattern_syntax root before its existing iterative DFS. No alternate resolver or native recursive walk.

Peeling the held classifier root removes about 23 instructions from the bare Blank path and 394 from held depth-8. All 23 rows pass parent-neighbor checks with no new base regression; ten labels remain above target, maximum falls from 5.351806% to 5.097507%.

10 regressions; largest remaining delta +5.097507%.

```text
rum-cli dd7dc7d1780e63ac0ab601ec91ec279286e5b1f7e6212a26c89c4229dca51b58
rum-cli-alloc d75b26151ff7cab670ac65daf29891cdc9f29a09ccd7e2408ad2b58198424a12
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.982000	999.056200	0.007428	999.007650	999.112200	0.010465	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.071000	998.905400	-0.016575	999.123200	999.116450	-0.000676	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22075.567000	21232.213300	-3.820304	22076.308350	21234.006800	-3.815409	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.495100	20936.957100	-3.908200	21786.938850	20938.266950	-3.895324	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17183.831800	18058.563050	5.090432	17184.100900	18060.061700	5.097507	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26314.515850	23548.638800	-10.510841	26314.805200	23548.619250	-10.511900	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59724.590000	61200.809000	2.471711	59723.614000	61208.533000	2.486318	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217275.407000	206788.241000	-4.826670	217265.181000	206791.284000	-4.820789	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.240300	5264.396750	2.335747	5144.073100	5264.327200	2.337721	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.306350	5386.174800	2.276139	5266.278600	5386.411900	2.281180	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412400.174000	2412489.151000	0.003688	2412418.170000	2412551.116000	0.005511	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39489.314700	40208.211500	1.820484	39539.927700	40147.041950	1.535446	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.080150	24130.088600	0.033200	24181.937700	24130.152500	-0.214148	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11459.725950	11370.874550	-0.775336	11381.388900	11448.977400	0.593851	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254201.749900	254204.751450	0.001181	254197.432850	254204.256300	0.002684	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10044.814950	10035.140100	-0.096317	10044.964650	10033.897500	-0.110176	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49941.658300	49949.447000	0.015596	49958.063150	49949.129700	-0.017882	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.532750	22756.249100	3.046235	22083.434100	22756.274350	3.046810	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.613300	21911.761250	3.486169	21173.627650	21911.776200	3.486170	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39463.185400	40150.594750	1.741900	39348.139200	40149.234600	2.035917	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.378000	15881.340300	4.014587	15268.540300	15881.032350	4.011464	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52480.489000	54892.973000	4.596916	52477.397000	54897.302000	4.611328	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204430.357000	199742.051000	-2.293351	204433.490000	199740.732000	-2.295494	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-07_total	base_rule_self	candidate-07_rule_self	candidate-07_continuation_self
g4-rule-blank-constraint	17118.4	18422.5	1227.0	1853.0	0
g4-rule-skew-depth-8	59800.6	61258.1	11116.0	11979.0	0
matchq-integer-success	4996.4	5095.7	401.0	521.0	0
matchq-integer-fail	5467.9	5380.8	400.0	520.0	0
cases-pattern-child	39580.8	40354.5	1795.0	2261.0	0
g4-matchq-pattern-child	22716.8	23054.4	1706.0	2172.0	0
g4-matchq-pattern-head	21239.1	22145.7	1774.0	2207.0	0
g4-cases-pattern-child	39593.3	40616.0	1795.0	2261.0	0
g4-matchq-prepared-call	15098.4	16015.1	2742.0	3148.0	0
g4-matchq-skew-depth-8	52490.7	54720.6	7095.0	8174.0	0
```

#### candidate-08 — rejected

Preallocate one work slot only for a root BlankArguments policy; its evaluate-only operands cannot add structural work to this walk.

Root Blank capacity saves about 98 instructions and 672 allocated bytes, but adds instructions to wrapper and both skewed Rule labels in both samples beyond the parent-neighbor threshold; reject despite a lower measured maximum.

10 regressions; largest remaining delta +4.611105%.

```text
rum-cli da651c13067676248b83f5bb964dff1f112f667f6b61989a9eaa8d33763fba26
rum-cli-alloc 6bb65e71ddc8e8822402996af2e3b94adba4cd3e7fb21f516209d2340d562db0
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.150600	998.983200	-0.016754	999.186000	998.901050	-0.028518	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.045250	999.117700	0.007252	999.017100	999.236050	0.021917	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22077.283900	21242.052150	-3.783218	22079.093800	21244.072450	-3.781955	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21787.083250	20946.870200	-3.856473	21785.699450	20946.849650	-3.850461	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1144.000000	17184.701200	17961.087050	4.517890	17186.188050	17961.000500	4.508344	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26310.137250	23565.834700	-10.430590	26308.186450	23570.605450	-10.405814	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59730.121000	61254.249000	2.551691	59725.810000	61264.188000	2.575734	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217268.786000	207005.337000	-4.723849	217273.381000	206994.350000	-4.730921	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.176250	5264.213350	2.333456	5144.332100	5264.347850	2.332970	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.146750	5386.259350	2.280844	5266.245800	5386.251750	2.278776	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412444.008000	2412547.511000	0.004290	2412442.851000	2412550.758000	0.004473	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39478.872700	40119.053950	1.621579	39423.548600	40207.872750	1.989481	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.245900	24129.893250	0.031702	24181.963100	24129.983100	-0.214954	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.475600	11448.924000	0.363344	11407.715400	11370.967200	-0.322135	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254201.327300	254203.841050	0.000989	254196.733500	254203.876250	0.002810	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10052.020600	10033.736300	-0.181897	10052.376500	10033.366250	-0.189112	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.309850	49949.321650	-0.001978	49958.366950	49949.438500	-0.017872	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.484250	22756.237850	3.046411	22083.481600	22756.080350	3.045710	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.605700	21911.647950	3.485671	21173.789200	21911.524350	3.484190	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39448.701700	40149.147400	1.775586	39529.394450	40150.704950	1.571768	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.492500	15881.228050	4.013072	15268.359750	15881.362950	4.014860	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52480.139000	54896.196000	4.603755	52479.282000	54899.157000	4.611105	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204436.242000	199739.846000	-2.297242	204427.775000	199741.364000	-2.292453	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-08_total	base_rule_self	candidate-08_rule_self	candidate-08_continuation_self
g4-rule-blank-constraint	16989.8	18089.8	1227.0	1937.0	0
g4-rule-wrapper	26639.5	23732.1	3550.0	4103.0	0
g4-rule-skew-depth-8	59507.4	61564.4	11116.0	12037.0	0
g4-rule-skew-depth-32	216991.4	207416.6	80500.0	67517.0	0
```

#### candidate-09 — kept

Complete a held root at the common normalization point without first calling acceptance and looping through Done; resumed Done values share that same point.

Direct held-root completion saves about 63 instructions on bare Blank (now +1.11% success / +1.08% failure versus base), with no new base regression or qualifying parent-neighbor slowdown. The untouched Rule Blank remains the largest residual; its maximum does not fall, so no-progress streak is one. Stop tuning at the ten-candidate limit.

10 regressions; largest remaining delta +5.097739%.

```text
rum-cli 28517da73e9b0aa7ad90e93b12290617125dade935d88a7a2b9a201e41050619
rum-cli-alloc 9b5be2637ef32b33610375292a02aafa6a296083e19411e41e2d667d6a204aec
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.858600	999.156100	0.029784	999.084000	998.887150	-0.019703	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.078650	999.046050	-0.003263	998.586400	999.133650	0.054802	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22076.926450	21231.444150	-3.829710	22076.248400	21231.983950	-3.824311	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21786.216950	20938.118100	-3.892823	21786.327800	20938.184750	-3.893006	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.006950	18060.002700	5.097739	17185.453750	18059.970050	5.088701	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26312.364750	23543.698050	-10.522303	26308.160950	23543.872150	-10.507343	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59723.881000	61196.535000	2.465771	59724.658000	61209.030000	2.485359	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217273.005000	206796.492000	-4.821820	217267.818000	206795.095000	-4.820191	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.510250	5201.447400	1.106755	5144.253900	5201.229450	1.107557	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.152800	5323.224500	1.083746	5266.325650	5323.240850	1.080738	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412442.898000	2412484.068000	0.001707	2412904.010000	2412545.683000	-0.014850	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39585.537250	40095.169050	1.287419	39472.435850	40096.219850	1.580303	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.086600	24130.115600	0.033285	24121.995950	24130.004050	0.033198	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11433.592550	11397.033500	-0.319751	11459.649550	11423.150900	-0.318497	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254200.739650	254203.550550	0.001106	254200.742700	254204.775050	0.001586	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.769450	10033.779950	-0.039745	10030.690650	10033.291400	0.025928	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49942.138300	49949.304450	0.014349	49966.653250	49950.304850	-0.032719	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.440750	22702.292650	2.802335	22083.473050	22702.100600	2.801315	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.495500	21855.584300	3.221427	21173.573050	21855.763000	3.221893	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39471.880050	40118.932900	1.639275	39426.914650	40095.991100	1.697004	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.495250	15825.285950	3.646664	15268.348100	15825.294300	3.647717	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52478.716000	54865.086000	4.547310	52482.509000	54864.032000	4.537746	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204430.965000	199783.091000	-2.273567	204432.775000	199779.824000	-2.276030	False
```

Calibrated self-attribution (all raw profiles retained):

```tsv
label	base_total	candidate-09_total	base_rule_self	candidate-09_rule_self	candidate-09_continuation_self
g4-rule-blank-constraint	17164.2	18163.9	1227.0	1853.0	0
g4-rule-skew-depth-8	59794.4	61078.6	11116.0	11979.0	0
matchq-integer-success	5244.3	5224.6	401.0	458.0	0
matchq-integer-fail	5237.2	5301.1	400.0	457.0	0
cases-pattern-child	39779.9	40166.7	1795.0	2207.0	0
g4-matchq-pattern-child	22312.1	22624.1	1706.0	2118.0	0
g4-matchq-pattern-head	21230.9	21934.6	1774.0	2151.0	0
g4-cases-pattern-child	40132.8	40573.5	1795.0	2207.0	0
g4-matchq-prepared-call	15542.0	16050.3	2742.0	3092.0	0
g4-matchq-skew-depth-8	52544.9	54827.4	7095.0	8142.0	0
```

### 4. Verification: exact commands and verbatim results

All commands below ran serially with the displayed bounds and environment. Final checks use the identical source export at `.temp/brief-574/verification-source`, with generated test/oracle artifacts therefore remaining in brief-574 scratch. Complete unabridged command output is in the named `.temp/brief-574/logs/<label>.txt`; the excerpts below preserve each reported result line exactly. Metadata records exact exit statuses and durations (durations enforce bounds, not performance decisions).

Expected replay failures are retained: stale certificates were first regenerated with the production harness (exit 101), then only fingerprint cells were installed and both feature-set gates rerun to green. Earlier candidate-01 structural-envelope and formatting failures were repaired without test edits before measurement; candidate-06 also needed formatting repair. Candidate-02 warnings are disclosed rather than hidden.

#### candidate-00-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-00/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-00-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 53s
```

#### candidate-00-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-00/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-00-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-01-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **1**. Full output: `.temp/brief-574/logs/candidate-01-format.txt`.

```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:108:
         .clone();
         self.suspend_matcher_pattern(
             state,
[31m-            MatcherPatternAwait::Evaluate { destination, pending_expr },
(B[m[32m+            MatcherPatternAwait::Evaluate {
(B[m[32m+                destination,
(B[m[32m+                pending_expr,
(B[m[32m+            },
(B[m             error,
         )
     }
```

#### candidate-01-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-01-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 11s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
```

#### candidate-01-private-depth

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-01-private-depth.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 43.51s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.67s
```

#### candidate-01-structural

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **101**. Full output: `.temp/brief-574/logs/candidate-01-structural.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 53.30s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::structural_rule_pattern_depth_contract' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
failures:

failures:
    tests::association_depth_contract::rule_pattern_depth_contract::structural_rule_pattern_depth_contract

test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.29s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

#### candidate-01-format-repair

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **1**. Full output: `.temp/brief-574/logs/candidate-01-format-repair.txt`.

```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:105:
                 // supplies an ordinary root only; it never escapes this capture.
                 let pending_expr = match destination {
                     MatcherPatternDestination::Root => root.expect("root entry retains its source"),
[31m-                    MatcherPatternDestination::Head => &state.stack.last().expect("parent").original_head,
(B[m[32m+                    MatcherPatternDestination::Head => {
(B[m[32m+                        &state.stack.last().expect("parent").original_head
(B[m[32m+                    }
(B[m                     MatcherPatternDestination::Argument(index) => {
                         &state.stack.last().expect("parent").original_args[index]
                     }
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:112:
                 }
                 .clone();
[31m-                MatcherPatternAwait::Evaluate { destination, pending_expr }
(B[m[32m+                MatcherPatternAwait::Evaluate {
(B[m[32m+                    destination,
(B[m[32m+                    pending_expr,
(B[m[32m+                }
(B[m             }
         };
         crate::resume_or_suspend!(insert self, MatcherPatternSuspensionFrame { state, pending });
```

#### candidate-01-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-01-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.59s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
```

#### candidate-01-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-01-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.35s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.56s
```

#### candidate-01-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-01-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.57s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.36s
```

#### candidate-01-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-01/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-01-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 49s
```

#### candidate-01-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-01/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-01-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

#### candidate-02-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-02-format.txt`.

Verbatim output: empty.

#### candidate-02-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-02-continuations-final.txt`.

```text
warning: unused import: `definition_lhs_pattern_wrapper_head`
warning: function `definition_lhs_pattern_wrapper_head` is never used
warning: `rum-runtime` (lib) generated 2 warnings (run `cargo fix --lib -p rum-runtime` to apply 1 suggestion)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.70s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
```

#### candidate-02-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-02-private-depth-final.txt`.

```text
warning: unused import: `definition_lhs_pattern_wrapper_head`
warning: function `definition_lhs_pattern_wrapper_head` is never used
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 5 warnings (run `cargo fix --lib -p rum-runtime --tests` to apply 1 suggestion)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.93s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.47s
```

#### candidate-02-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-02-structural-final.txt`.

```text
warning: unused import: `definition_lhs_pattern_wrapper_head`
warning: function `definition_lhs_pattern_wrapper_head` is never used
warning: `rum-runtime` (lib) generated 2 warnings (run `cargo fix --lib -p rum-runtime` to apply 1 suggestion)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.28s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.38s
```

#### candidate-02-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-02-private-rule.txt`.

```text
warning: unused import: `definition_lhs_pattern_wrapper_head`
warning: function `definition_lhs_pattern_wrapper_head` is never used
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 5 warnings (run `cargo fix --lib -p rum-runtime --tests` to apply 1 suggestion)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.77s
```

#### candidate-02-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-02/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-02-build-default.txt`.

```text
warning: unused import: `definition_lhs_pattern_wrapper_head`
warning: function `definition_lhs_pattern_wrapper_head` is never used
warning: `rum-runtime` (lib) generated 2 warnings (run `cargo fix --lib -p rum-runtime` to apply 1 suggestion)
    Finished `release` profile [optimized] target(s) in 1m 49s
```

#### candidate-02-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-02/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-02-build-alloc.txt`.

```text
warning: unused import: `definition_lhs_pattern_wrapper_head`
warning: function `definition_lhs_pattern_wrapper_head` is never used
warning: `rum-runtime` (lib) generated 2 warnings (run `cargo fix --lib -p rum-runtime` to apply 1 suggestion)
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-03-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-03-format.txt`.

Verbatim output: empty.

#### candidate-03-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-03-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.91s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
```

#### candidate-03-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-03-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.96s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.56s
```

#### candidate-03-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-03-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.35s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.36s
```

#### candidate-03-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-03-private-rule.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.67s
```

#### candidate-03-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-03/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-03-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 49s
```

#### candidate-03-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-03/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-03-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 45s
```

#### candidate-04-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-04-format.txt`.

Verbatim output: empty.

#### candidate-04-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-04-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.17s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
```

#### candidate-04-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-04-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.41s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.69s
```

#### candidate-04-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-04-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 30.28s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.36s
```

#### candidate-04-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-04-private-rule.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.83s
```

#### candidate-04-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-04/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-04-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

#### candidate-04-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-04/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-04-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 41s
```

#### interim-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/interim-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 36.73s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.57s
```

#### candidate-05-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-05-format.txt`.

Verbatim output: empty.

#### candidate-05-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-05-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.59s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
```

#### candidate-05-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-05-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.89s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.68s
```

#### candidate-05-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-05-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 30.66s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.34s
```

#### candidate-05-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-05-private-rule.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.94s
```

#### candidate-05-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-05/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-05-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 49s
```

#### candidate-05-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-05/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-05-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 42s
```

#### candidate-06-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **1**. Full output: `.temp/brief-574/logs/candidate-06-format.txt`.

```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs:29:
     // The immutable queued frame remains rooted until this working copy retires.
     let mut state = rule_lhs_working_copy(&frame.state);
     let active = state.stack.pop();
[31m-    let (active, step) = accept_rule_lhs_value(&mut state, active, frame.pending.destination, value);
(B[m[32m+    let (active, step) =
(B[m[32m+        accept_rule_lhs_value(&mut state, active, frame.pending.destination, value);
(B[m     session.drive_rule_lhs(state, active, step)
 }
 
```

#### candidate-06-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-06-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 30.90s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

#### candidate-06-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-06-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.35s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.43s
```

#### candidate-06-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-06-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 39.52s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.30s
```

#### candidate-06-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-06-private-rule.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.81s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.81s
```

#### candidate-06-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-06/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-06-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```

#### candidate-06-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-06/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-06-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

#### candidate-07-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-07-format.txt`.

Verbatim output: empty.

#### candidate-07-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-07-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.69s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
```

#### candidate-07-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-07-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 25.14s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.85s
```

#### candidate-07-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-07-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 41.79s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.35s
```

#### candidate-07-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-07-private-rule.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.78s
```

#### candidate-07-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-07/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-07-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 51s
```

#### candidate-07-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-07/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-07-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

#### candidate-08-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-08-format.txt`.

Verbatim output: empty.

#### candidate-08-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-08-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.12s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
```

#### candidate-08-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-08-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.79s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.55s
```

#### candidate-08-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-08-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.62s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.38s
```

#### candidate-08-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-08-private-rule.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.62s
```

#### candidate-08-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-08/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-08-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 50s
```

#### candidate-08-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-08/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-08-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 45s
```

#### candidate-09-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-09-format.txt`.

Verbatim output: empty.

#### candidate-09-continuations-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-09-continuations-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.38s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
```

#### candidate-09-private-depth-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-09-private-depth-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.60s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.52s
```

#### candidate-09-structural-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-09-structural-final.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 32.52s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.36s
```

#### candidate-09-private-rule

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-09-private-rule.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.74s
```

#### candidate-09-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-09/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-09-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-09-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/candidate-09/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full output: `.temp/brief-574/logs/candidate-09-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 33s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.72s
```

#### fingerprints-replay

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/fingerprints-replay.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.57s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.51s
```

#### certificates-true-replay

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **101**. Full output: `.temp/brief-574/logs/certificates-true-replay.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.87s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
thread 'tests::retention_closure::local::production::local_production_acceptance' panicked at crates/rum-stdlib/src/tests/retention_closure/local/production.rs:98:6:
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 18.03s
```

#### certificates-false-replay

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **101**. Full output: `.temp/brief-574/logs/certificates-false-replay.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 50.46s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
thread 'tests::retention_closure::local::production::local_production_acceptance' panicked at crates/rum-stdlib/src/tests/retention_closure/local/production.rs:98:6:
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1103 filtered out; finished in 18.19s
```

#### final-fmt

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-fmt.txt`.

Verbatim output: empty.

#### final-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 26.19s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
```

#### final-structural

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-structural.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.26s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.36s
```

#### final-runtime

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-runtime -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-runtime.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 05s
thread 'runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs:206:42:
thread 'runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs:203:42:
thread 'runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs:241:5:
thread 'runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs:241:5:
thread 'runtime::accounting_mode::tests::exits::retention_bridge_releases_prefix_on_guest_error_and_panic' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/exits.rs:4:5:
thread 'runtime::accounting_mode::tests::retention_lifetime_normal_early_error_drop_and_unwind' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs:69:26:
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
warning: function `consume` is never used
warning: 1 warning emitted
thread 'runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:393:5:
thread 'runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:393:5:
thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:417:5:
thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:417:5:
thread 'runtime::tests::messages::push_message_rejects_unregistered_builtin_message_tags' panicked at crates/rum-runtime/src/runtime/session/session_helpers.rs:347:25:
thread 'runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback' panicked at crates/rum-runtime/src/runtime/tests/session_memory_maintained.rs:413:5:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_error' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_file' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_script' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_value' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
test result: ok. 1118 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 25.70s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.46s
```

#### final-ownership

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --lib ownership_graph -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-ownership.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 60.36s
```

#### final-retention-default

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-retention-default.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 17.81s
```

#### final-retention-no-default

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-retention-no-default.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.55s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1103 filtered out; finished in 17.41s
```

#### final-workspace

```sh
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-574/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-574/target-verification env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-574/verification-source`; exit: **0**. Full output: `.temp/brief-574/logs/final-workspace.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5m 23s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
thread 'lifecycle_contract_tests::abandoned_poisoned_session_does_not_contaminate_a_fresh_session' panicked at crates/rum-host/src/session.rs:733:24:
thread 'lifecycle_contract_tests::panic_poison_blocks_public_entries_until_reset' panicked at crates/rum-host/src/session.rs:733:24:
thread 'session::tests::panic_is_complete_response_marks_session_poisoned_and_invalid' panicked at crates/rum-host/src/session.rs:832:24:
thread 'tests::session_lifecycle::host_boundary_catches_panic_and_poisons_session' panicked at crates/rum-host/src/session.rs:733:24:
thread 'tests::session_lifecycle::host_session_introspection_survives_poisoned_session' panicked at crates/rum-host/src/session.rs:733:24:
thread 'tests::session_lifecycle::host_session_reset_recovers_poisoned_session' panicked at crates/rum-host/src/session.rs:733:24:
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.42s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.39s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.44s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.12s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.99s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.87s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 296.60s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.32s
thread 'panics_during_timed_eval_or_resume_poison_until_reset' panicked at crates/rum-host/tests/suspended_timeout_contract.rs:258:9:
thread 'panics_during_timed_eval_or_resume_poison_until_reset' panicked at crates/rum-host/tests/suspended_timeout_contract.rs:250:9:
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.04s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
thread 'tests::output_once::host_buffers::synthesized_panic_has_no_buffers_and_preserves_published_prefix' panicked at crates/rum-jupyter/src/tests/output_once/host_buffers.rs:81:13:
thread '<unnamed>' panicked at crates/rum-jupyter/src/tests/output_once/host_buffers.rs:81:13:
thread 'zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:88:31:
thread 'zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:328:9:
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
thread 'terminal::tests::masked_panic_restores_terminal_and_descriptor' panicked at crates/rum-repl/src/terminal/tests.rs:48:9:
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
thread 'runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs:206:42:
thread 'runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs:203:42:
thread 'runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs:241:5:
thread 'runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs:241:5:
thread 'runtime::accounting_mode::tests::exits::retention_bridge_releases_prefix_on_guest_error_and_panic' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/exits.rs:4:5:
thread 'runtime::accounting_mode::tests::retention_lifetime_normal_early_error_drop_and_unwind' panicked at crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs:69:26:
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:263:5:
warning: function `consume` is never used
warning: 1 warning emitted
thread 'runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:393:5:
thread 'runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:393:5:
thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:417:5:
thread 'runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:417:5:
thread 'runtime::tests::messages::push_message_rejects_unregistered_builtin_message_tags' panicked at crates/rum-runtime/src/runtime/session/session_helpers.rs:347:25:
thread 'runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback' panicked at crates/rum-runtime/src/runtime/tests/session_memory_maintained.rs:413:5:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_error' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_file' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_script' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
thread 'runtime::tests::session_requests::batch_boundary::invariant_falsifier_value' panicked at crates/rum-runtime/src/runtime/session/session_requests.rs:608:9:
test result: ok. 1118 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 25.94s
thread 'tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/lazy_depth_contract.rs:105:5:
thread 'tests::association_depth_contract::registry::real_catalog_misdeclaration_names_association_key_flatten_recursive_edge' panicked at crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:608:29:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
thread 'tests::association_depth_contract::schema_depth_contract::schema_root_completeness_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs:149:5:
thread 'tests::association_depth_contract::schema_depth_contract::structural_self_reaching_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs:55:5:
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
expired-C01: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=true
expired-C02: RED stale or omitted instance certificate C02 hash=true
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1057 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1473.28s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.70s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.89s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.36s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.77s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Source/binary audit (150-second bound, exit 0):

```sh
python3 scripts/run-bounded.py --timeout 150 -- python3 .temp/brief-574/audit.py candidate-09 > .temp/brief-574/final-audit-before-report.txt
```

```text
AUDIT PASS: 9 allowed changed files; tests/frame lists/base digests unchanged; measured source and export match; no new unsafe.
```

Metric/result-check validation: {"checks": 460, "metrics": 2760, "attribution_commands": 732}. Every saved process exited zero and passed its saved output check. `.temp/brief-574/metric-command-results.md` expands every metric/check command with its exact exit code and verbatim streams/CSV, and lists every attribution command with complete output files. Every per-process bound is 150 seconds. The result-check commands in that ledger are reconstructed exactly from saved arguments in `measure.py`; raw check streams were saved before validation. All calibration and annotation commands exited zero.

Ignored workspace tests (verbatim). The fingerprint-regeneration test was run explicitly above; other listed ignored tests were not run:

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

Native ZeroMQ skips (verbatim; these are skips, not exercised socket coverage):

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

All mandatory final commands ran. No timing benchmark, external-engine differential test, external-engine output, or candidate beyond 09 was run.

### 5. Counters, standing duty and residuals

| Candidate | Kept? | Regressing labels | Largest residual | Literal no-progress streak |
| --- | --- | ---: | ---: | ---: |
| candidate-00 | True | 11 | +6.282014% | 0 |
| candidate-01 | True | 10 | +6.225429% | 0 |
| candidate-02 | False | 11 | +5.361136% | 0 |
| candidate-03 | True | 10 | +5.992181% | 0 |
| candidate-04 | False | 10 | +5.987804% | 0 |
| candidate-05 | True | 10 | +5.351806% | 0 |
| candidate-06 | False | 10 | +5.364607% | 1 |
| candidate-07 | True | 10 | +5.097507% | 0 |
| candidate-08 | False | 10 | +4.611105% | 0 |
| candidate-09 | True | 10 | +5.097739% | 1 |

Final counter: {"measured_candidates": 10, "optimization_candidates": 9, "kept": "candidate-09", "no_progress_streak": 1, "accepted_no_progress_streak": 2, "regressions": 10, "largest_remaining_delta": 5.097738569059418}. The maximum uses the larger of the two samples on regressing rows. Early checkpoint entries used the smaller sustained value; that reducer definition was corrected without rerunning any sample. Early candidate-02 decision metadata recorded an auxiliary accepted-state streak of one; the later checkpoint clarified the brief’s literal measured-progress streak as zero because its maximum fell, despite rejection. Candidate-04’s maximum fell by only 0.004376 percentage points on an unchanged Rule label; no causal bottleneck improvement is claimed. Rejected candidates never replaced the kept tree.

Ownership reconciliation preserved every existing identity, status and disposition; only source positions moved. No existing row vanished, no new UnknownHostPayload or self-reaching row appeared, and no new row belongs to untouched code.

| Inventory | Before | After | Position moves | New UnknownConstruct | Removed |
| --- | ---: | ---: | ---: | ---: | ---: |
| host_payloads.tsv | 63 | 63 | 0 | 0 | 0 |
| unknown_attributes.tsv | 1465 | 1465 | 0 | 0 | 0 |
| unknown_captures.tsv | 2944 | 2944 | 1 | 0 | 0 |
| unknown_macros.tsv | 3135 | 3139 | 15 | 4 | 0 |
| unknown_storage.tsv | 88 | 88 | 0 | 0 | 0 |

New unknown rows (verbatim):

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs	120	8	unexpanded macro matches! (ends at 120:50)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	78	13	unexpanded macro debug_assert_eq! (ends at 81:14)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	88	16	unexpanded macro matches! (ends at 88:70)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	172	8	unexpanded macro matches! (ends at 172:50)
```

Moved ownership rows (initial inventory row numbers):

| Inventory row | Source | Old position | New position |
| --- | --- | --- | --- |
| unknown_captures.tsv:854 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 176:22 | 210:22 |
| unknown_macros.tsv:261 | `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs` | 124:20 | 136:20 |
| unknown_macros.tsv:262 | `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs` | 141:5 | 153:5 |
| unknown_macros.tsv:263 | `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs` | 152:5 | 164:5 |
| unknown_macros.tsv:315 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 86:16 | 91:16 |
| unknown_macros.tsv:316 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 90:16 | 95:16 |
| unknown_macros.tsv:317 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 112:9 | 117:9 |
| unknown_macros.tsv:318 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 174:12 | 184:12 |
| unknown_macros.tsv:319 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 182:41 | 192:41 |
| unknown_macros.tsv:320 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 185:17 | 195:17 |
| unknown_macros.tsv:321 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 188:20 | 198:20 |
| unknown_macros.tsv:675 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 103:9 | 125:9 |
| unknown_macros.tsv:676 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 113:8 | 135:8 |
| unknown_macros.tsv:677 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 121:44 | 143:44 |
| unknown_macros.tsv:678 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 124:13 | 146:13 |
| unknown_macros.tsv:679 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 155:24 | 189:24 |

Replayed fingerprint cells (all other 18 fields unchanged):

| Row / ID | Before | After |
| --- | --- | --- |
| 2 / B01 | `fnv1a64:5d209f30a83ba1ad` | `fnv1a64:fbfb5a1ca428d593` |
| 3 / B02 | `fnv1a64:bd41ac4e29a8824c` | `fnv1a64:cc1a40bed8415d76` |
| 4 / B03 | `fnv1a64:522ce940b328c6bf` | `fnv1a64:ca985f53f30395d5` |
| 5 / B04 | `fnv1a64:6141da8898bb6b65` | `fnv1a64:0bf46fd10c1d447b` |
| 6 / B05 | `fnv1a64:28012223d2897000` | `fnv1a64:1726a7f33e990cfa` |
| 7 / B06 | `fnv1a64:363cafb7a6dbd599` | `fnv1a64:d2ea7f5bb8322dcf` |
| 8 / B07 | `fnv1a64:7bdd1108d5826950` | `fnv1a64:4f063359caaa428a` |
| 9 / B08 | `fnv1a64:fb6abb37c0721f83` | `fnv1a64:8ce07595dbc5c0e9` |
| 10 / B09 | `fnv1a64:01bc77e85d5e3f51` | `fnv1a64:5b45e5f66e4b2f37` |
| 11 / B10 | `fnv1a64:752d340634b949ae` | `fnv1a64:2c7e19dfa2ede670` |
| 12 / B11 | `fnv1a64:22998be5b2af9165` | `fnv1a64:cd4c212e26116a7b` |
| 13 / B12 | `fnv1a64:c0c8f7e43966d451` | `fnv1a64:1a5265f24a53c437` |
| 14 / I01 | `fnv1a64:3b028a2e14f3982d` | `fnv1a64:d9dd451a10e0cc13` |
| 15 / I02 | `fnv1a64:7f429fb7cabff800` | `fnv1a64:6e68258736cf94fa` |
| 16 / I03 | `fnv1a64:1e83afabd5d5d611` | `fnv1a64:407f6d1f3ce748f7` |
| 17 / I04 | `fnv1a64:3828b1bd8e543eec` | `fnv1a64:80ed0f00b3571b96` |
| 18 / P01 | `fnv1a64:6b7ea1b48f0870b9` | `fnv1a64:32343ffc1241886f` |
| 19 / P02 | `fnv1a64:cda9983f19af249e` | `fnv1a64:5f0638f3473c9a20` |
| 20 / P03 | `fnv1a64:a8bd953d47a5dba7` | `fnv1a64:b261336daa07de6d` |
| 21 / P04 | `fnv1a64:b21cca63ae519e4c` | `fnv1a64:c0f55ed45cea7976` |
| 22 / P05 | `fnv1a64:d0a68d92d0e3aa9d` | `fnv1a64:e71d1e8a3dcbba03` |
| 23 / P06 | `fnv1a64:11481cb4c5966dd1` | `fnv1a64:6ad18ac2d6835db7` |
| 24 / P070 | `fnv1a64:4b95b8f6a40be7d1` | `fnv1a64:a51f2704b4f8d7b7` |
| 25 / P080 | `fnv1a64:2eba9eacd903446e` | `fnv1a64:711b7946fbf95a30` |
| 26 / P090 | `fnv1a64:aa18c5bd3b4adb0c` | `fnv1a64:d75c65721d20e836` |
| 27 / P100 | `fnv1a64:499fd706895239f5` | `fnv1a64:eca95d89a54f588b` |
| 28 / P110 | `fnv1a64:86002ce92ae49c56` | `fnv1a64:56b3442875b44b28` |
| 29 / P120 | `fnv1a64:d4817badde7572a3` | `fnv1a64:1d8eaddf43c18109` |
| 30 / P130 | `fnv1a64:1faa6a6604364350` | `fnv1a64:f2d38cb6f95e1c8a` |
| 31 / P140 | `fnv1a64:05197670c2d3aad4` | `fnv1a64:bc2337b209c42bce` |
| 32 / P150 | `fnv1a64:17608d39642fdaac` | `fnv1a64:2fdfbf69e726ac56` |
| 33 / P071 | `fnv1a64:3b8903138e9d12f0` | `fnv1a64:afbd9459a0364c2a` |
| 34 / P081 | `fnv1a64:71e97c33842551f1` | `fnv1a64:f21b0ec1b2717ed7` |
| 35 / P091 | `fnv1a64:4c49e40d000bb479` | `fnv1a64:6cdf8ae79b55062f` |
| 36 / P101 | `fnv1a64:7e031db660483956` | `fnv1a64:4eb634f5ab17e828` |
| 37 / P111 | `fnv1a64:14f5c00de502eca1` | `fnv1a64:a354a488fa0fdbc7` |
| 38 / P121 | `fnv1a64:d2bcd50a6a5993da` | `fnv1a64:bab6956209b7a72c` |
| 39 / P131 | `fnv1a64:feebfceecf0f8ce5` | `fnv1a64:a99e9237427165fb` |
| 40 / P141 | `fnv1a64:c41117589f8d0759` | `fnv1a64:7fb6365d1f09378f` |
| 41 / P151 | `fnv1a64:80a5cb73a44c04e3` | `fnv1a64:8c6c9bb9a4a2f649` |
| 42 / B13 | `fnv1a64:c44d4340d7789199` | `fnv1a64:60fb12e4e8cee9cf` |
| 43 / R01 | `fnv1a64:9c039952e08cb99c` | `fnv1a64:02363dc725fdc686` |
| 44 / R02 | `fnv1a64:2804020d8afe0a67` | `fnv1a64:f7976dbbf476052d` |
| 45 / R03 | `fnv1a64:e3777e76d52f80c8` | `fnv1a64:7e8f89c1623d40b2` |
| 46 / R040 | `fnv1a64:5005d0c58ae7a63c` | `fnv1a64:c1d35cfd326182a6` |
| 47 / R050 | `fnv1a64:aeb742162492017d` | `fnv1a64:b1c0a7500018db63` |
| 48 / R060 | `fnv1a64:def7a1c0e3a87224` | `fnv1a64:a25c791d5604f7de` |
| 49 / R070 | `fnv1a64:da57444d65c710f1` | `fnv1a64:5a88d6db94133dd7` |
| 50 / R080 | `fnv1a64:ff2d1ab05dd907ec` | `fnv1a64:47f177f382dbe496` |
| 51 / R090 | `fnv1a64:710d43cf80452bb5` | `fnv1a64:a9b8315766f8324b` |
| 52 / R100 | `fnv1a64:17528d935cfe713f` | `fnv1a64:8fbe03a69cd94055` |
| 53 / R110 | `fnv1a64:b82c7ce889193aa6` | `fnv1a64:07235466af3fffb8` |
| 54 / R041 | `fnv1a64:aa86a66aeea05099` | `fnv1a64:4734760efff6a8cf` |
| 55 / R051 | `fnv1a64:383af3d2c878e78c` | `fnv1a64:657e9387aa4ef4b6` |
| 56 / R061 | `fnv1a64:879e75a2c3ef56f3` | `fnv1a64:db1622acba394ad9` |
| 57 / R071 | `fnv1a64:d829cb5bb27c9312` | `fnv1a64:ceb91a602b8be6d4` |
| 58 / R081 | `fnv1a64:058a405b24fe0ef5` | `fnv1a64:a893c6de40fb2d8b` |
| 59 / R091 | `fnv1a64:2650ff71d1f19614` | `fnv1a64:56a4c873fbfd000e` |
| 60 / R101 | `fnv1a64:b0e59df6799d56e4` | `fnv1a64:c570863c36e37e9e` |
| 61 / R111 | `fnv1a64:8fd6669132fe323b` | `fnv1a64:4bd1df2a552db1f1` |
| 62 / R12 | `fnv1a64:de955d010aa0a27b` | `fnv1a64:f5cf92fc681d9a31` |
| 63 / R13 | `fnv1a64:e2cb8e6bed3a73ef` | `fnv1a64:984d126ed4749245` |
| 64 / R14 | `fnv1a64:63f7e57c6602f9f5` | `fnv1a64:07016bff8200188b` |
| 65 / R15 | `fnv1a64:f44a9ffd524c2cc9` | `fnv1a64:80bfe4a3b9d430ff` |
| 66 / O010 | `fnv1a64:25a8482731c4fe53` | `fnv1a64:9e57539145a2b439` |
| 67 / O0200 | `fnv1a64:ffe11845118ad0d4` | `fnv1a64:b6ead986587b51ce` |
| 68 / O011 | `fnv1a64:21805e48f13a36a4` | `fnv1a64:e4e535a56396bc5e` |
| 69 / O0210 | `fnv1a64:ac032fbda06107db` | `fnv1a64:bb900f8d02e50511` |

Certificate cells (complete old/new values retained in the indicated JSON files):

| Row / ID | Feature hash | Column | Before/after record |
| --- | --- | ---: | --- |
| 2 / C01 | true | 5 | `certificate-cell-changes-true.json` |
| 3 / C02 | true | 5 | `certificate-cell-changes-true.json` |
| 2 / C01 | false | 6 | `certificate-cell-changes-false.json` |
| 3 / C02 | false | 6 | `certificate-cell-changes-false.json` |

Evidence-position cells (all other fields unchanged):

| Row / ID | Before evidence | After evidence |
| --- | --- | --- |
| 57 / N:Count | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 66 / N:DeleteCases | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 103 / N:FreeQ | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 147 / N:MatchQ | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 151 / N:MemberQ | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |

All 59 self-reaching rows and 63 host-payload rows remain unchanged. Replay installed 68 fingerprint cells, four certificate cells (C01/C02 × both feature sets) whose non-fingerprint contents are identical, and five evidence-position cells. Detailed before/after cells are in `fingerprint-row-changes.json`, `certificate-cell-changes-{true,false}.json`, `obligation-row-changes.json` and `final-ownership-row-changes.json`; exact generated certificates and acceptance rows are under `.temp/brief-574/certificates/`.

Residual native rows are shown in the retained candidate table above. The final calibrated family slopes are:

```tsv
label	base_total	candidate-09_total	base_rule_self	candidate-09_rule_self	candidate-09_continuation_self
g4-rule-blank-constraint	17164.2	18163.9	1227.0	1853.0	0
g4-rule-skew-depth-8	59794.4	61078.6	11116.0	11979.0	0
matchq-integer-success	5244.3	5224.6	401.0	458.0	0
matchq-integer-fail	5237.2	5301.1	400.0	457.0	0
cases-pattern-child	39779.9	40166.7	1795.0	2207.0	0
g4-matchq-pattern-child	22312.1	22624.1	1706.0	2118.0	0
g4-matchq-pattern-head	21230.9	21934.6	1774.0	2151.0	0
g4-cases-pattern-child	40132.8	40573.5	1795.0	2207.0	0
g4-matchq-prepared-call	15542.0	16050.3	2742.0	3092.0	0
g4-matchq-skew-depth-8	52544.9	54827.4	7095.0	8142.0	0
```

Final driver/acceptance/drop self costs make the remaining flat-walk operations explicit (inclusive callee costs are not included):

| Label | Driver self | Value acceptance self | Work-Vec drop self |
| --- | ---: | ---: | ---: |
| `g4-rule-blank-constraint` | 734 | 0 | 58 |
| `g4-rule-skew-depth-8` | 6887 | 0 | 58 |
| `matchq-integer-success` | 166 | 0 | 26 |
| `matchq-integer-fail` | 166 | 0 | 26 |
| `cases-pattern-child` | 764 | 297 | 29 |
| `g4-matchq-pattern-child` | 764 | 297 | 29 |
| `g4-matchq-pattern-head` | 757 | 226 | 29 |
| `g4-cases-pattern-child` | 764 | 297 | 29 |
| `g4-matchq-prepared-call` | 679 | 230 | 29 |
| `g4-matchq-skew-depth-8` | 5622 | 1470 | 29 |

Rule acceptance is inlined into its driver; zero in that column does not mean no acceptance work. Rule Blank additionally has finish self cost 96 and classifier cost 82 versus base 66; outside this family, `RawVec::grow_one` and `finish_grow` add 88 and 54 self instructions. The active held-child path adds 44/27 in those growth routines and 27 in Atom cloning; held depth-8 adds 138/81 and 243 respectively. These library costs corroborate explicit stack/prefix/owned-value work; small-profile allocator and interning variation is not presented as an exact decomposition of the full-N native delta.

The remaining cost belongs to the present flat implementation’s explicit work records, destination selection, prefix handling, root completion and work-vector cleanup. The Rule Blank residual still includes driver/finish/work-Vec handling absent from the old recursive helper; shallow/deep held residuals include owned visits and parent work management. G4 suspension-frame resume and working-copy functions have zero self slope in these ordinary no-input profiles. This attribution is evidence of the current implementation’s cost, not a proof that further reductions are impossible. The hard candidate limit prevents further tuning here.

Candidate-06 showed why allocation counts do not decide: fewer Rule allocations added 2,258 family self instructions at depth 8 versus candidate-05. Candidate-08 saved 672 allocated bytes and about 98 native instructions on Blank, but its added per-work condition raised the Rule depth-8 family from 11,979 to 12,037 and slowed wrapper/deep neighbors; it was rejected. The retained improvements keep both resumable walks and unchanged semantic/depth contracts.


Post-report source/binary audit — 150-second bound, exit 0:

```sh
python3 scripts/run-bounded.py --timeout 150 -- python3 .temp/brief-574/audit.py candidate-09 > .temp/brief-574/final-audit.txt
```

```text
AUDIT PASS: 10 allowed changed files; tests/frame lists/base digests unchanged; measured source and export match; no new unsafe.
```
