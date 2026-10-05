# brief-575 — G4's ordinary path, second round: small entries over out-of-line drivers

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-574 measured all ten candidates and kept candidate 09: against the
pre-G4 one-codegen-unit base, 10 of 23 labels still regress, among them the
hot `matchq-integer-success` / `-fail` (director-reproduced on candidate 09:
+1.11% / +1.11%), `g4-rule-blank-constraint` (+5.09%) and
`g4-matchq-skew-depth-8` (+4.53%). Its attribution (§2) names the cost precisely. For
`MatchQ[5, _Integer]` the old recursive resolver and classifier cost 57 + 55
self instructions; the flat driver costs 226, plus 25 for acceptance and 26 to
drop an empty work `Vec` — for a root that is a recognized pattern head, which
the driver never needs to walk. For a short `Blank` rule the old recursive
helper cost 278 and the driver 770, plus finish 96 and work-`Vec` drop 58.

That is the cost of entering a large driver, not of doing work. brief-574
could not attack it because the director's briefs forbade a "second
resolution algorithm". That rule stays — but handling a classification's own
base cases before entering the driver is not a second algorithm: the old
recursive code did exactly that as its first step. **Target:** every label of
`.temp/brief-573/perf/labels.json` within +0.05% of the pre-G4 base (both
samples), every test unchanged and green; a residual is reported with its
attribution.

## The structural direction — measure it, one variable at a time

The traversal-depth line's lesson (`CLAUDE.md`): on the hottest function, move
locals and cold bodies, not control flow — outlining the cold branches cost
nothing and cut the frame 3.17x where an in-loop rework cost 2-7%.

1. **A small entry for each walker** — `resolve_held_matcher_pattern` and
   `evaluate_rule_lhs` — that settles the classification's base cases itself
   and returns: for the held resolver, a root that is a recognized (opaque)
   pattern head and a root with no pattern syntax (evaluate, then normalize);
   for the rule walk, whatever base cases its classifier settles at the root
   (`evaluate_rule_lhs` already returns early for a pattern-free left-hand
   side — check what else is settled at the root). Everything else calls the
   driver, kept out of line (`#[inline(never)]` and, where the attribution
   supports it, `#[cold]` on the capture paths). The entry must not grow a
   second copy of the walk: it decides only what the driver's first step would
   decide.
2. **No work `Vec` until there is a child to push** — the empty-`Vec` drop
   (26 and 58 instructions) disappears when the state is created on first
   descent.
3. **The rule driver's short-rule transitions** — `finish` and acceptance per
   §2 — after 1 and 2 are measured.

**A falsifier is required for 1**: a test that drives every base-case shape
(opaque pattern head, pattern-free root, each rule-root base case) through the
**driver** directly and through the **entry**, and asserts identical results
— so the two can never drift.

## Fixed — not negotiable

Both flat walks and their depth contracts (`rule_pattern_depth_contract.rs`,
the private depth tests); the semantics and every test; capture only on
`InputRequired`; no recursion, no depth cap, no `unsafe`.

## Measurement

Exactly brief-574's: base `.temp/brief-572/base/` (pre-G4, one unit; do not
rebuild); candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directories
under `.temp/brief-575/`, hashes checked; panel `.temp/brief-573/perf/` (23
labels, results checked first); allocations, then two paired samples of
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label per
process, stdin `/dev/null`, `(count(2N) - count(N)) / N`; any kept candidate
and the final one on all 23 labels; callgrind attribution calibrated against
`perf stat`. Measure the incoming tree once as candidate 00. No timing.

## Scope

- **Objective:** the entries and drivers of `definitions/rule_lhs.rs` (and
  `rule_lhs/`) and `cases/pattern_resolution.rs` (and `pattern_resolution/`),
  their callers in `definitions.rs` and `cases/match_q.rs`, the iterative
  classifiers, and the falsifier test (in the walkers' private test modules).
- **Standing duty — always in scope** (`TEMPLATE.md`). New `UnknownConstruct`
  rows for code you add or change are expected and disclosed; STOP on any other
  changed or vanished existing row, a new `UnknownHostPayload` or
  `self_reaching` row, or a row in untouched code.
- **Fence:** no semantic change; no change to existing tests; no matcher,
  normalizer, evaluator or `Expr` change; no other group; no dependency; the
  release profile unchanged. No edits to `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or docs.
- **Disclosure duty:** every change with line ranges and why; every candidate
  with its attribution and rows.

## rum / Rust pitfalls (read twice)

1. One variable per candidate; a candidate that makes another label regress is
   rejected; instruction counts decide.
2. Stop-loss keyed on progress (regressing labels or the largest delta must
   fall); stop after two consecutive compiling candidates that reduce neither;
   at most ten measured candidates; stopping early while progressing needs a
   stated reason.
3. `#[inline(always)]` on the driver is the opposite of the direction here.

## Verification (MANDATORY — you run it; iterate to green)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-575/tmp …`. Bounds: focused 150 s; a one-unit
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

Scratch under `.temp/brief-575/`; `PROGRESS.md` after each candidate.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Nothing outside this repository; no network. Commands: reads and scoped
  edits, `rg`/`grep`, `python3`, bounded `cargo`, the release `rum-cli`,
  bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind`. No probe that
  aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). **No new `unsafe`.**
- Paste real verbatim output; say plainly what did not run.

## Report format (append to this brief)

1. What changed, per file with line ranges.
2. Attribution for each regressing family, base against incoming.
3. Every candidate with its rows (all 23 for kept ones), both samples.
4. Verification, verbatim.
5. Counters, disclosures, attribution of anything left.

---

## Execution report

The +0.05% target is **not met**. Retained tree: **candidate-08**. 8/23 labels exceed +0.05% in both samples; largest sustained-row sample: +4.283887%.

No git command or network operation was run. No unsafe code, recursion, depth cap, external-engine comparison, dependency change or release-profile edit was introduced. Existing tests and both depth contracts are unchanged. Scratch, binaries, source snapshots and unabridged logs are under `.temp/brief-575/`.

### 1. Changes

| File | Final line ranges | Why |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 110 | Reduce measured rule completion overhead; preserve immediate/delayed RHS and suspension behavior. |
| `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 31–44, 47, 49–51, 56, 132–135 | Settle the rule classifier’s root base cases in a small entry and pass remaining work to the existing flat driver; keep root input suspension consistent for the direct-driver falsifier. |
| `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs` | 307–308 | Add the private base-case test module; every incoming test byte is preserved. |
| `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests/base_cases.rs` | 1–121 | Compare direct driver and entry Expr outcomes, prompts, side effects and messages across root base cases, malformed arities, errors, suspension and descent controls. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 16, 24–41, 43–46, 51, 78–82, 84–98, 100–102, 184 | Settle opaque and pattern-free roots before the out-of-line flat driver; retain one descent and normalization policy and create ordinary work storage only for a proved descent. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs` | 366–367 | Add the private base-case test module; every incoming test byte is preserved. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests/base_cases.rs` | 1–98 | Compare direct driver and entry Expr outcomes, prompts, side effects and messages across root base cases, malformed arities, errors, suspension and descent controls. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 403–405, 854 | Replay ownership discovery; positions may move and new UnknownConstruct rows are disclosed below. Existing identities/statuses/dispositions remain unchanged. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 287–298, 312–324, 678–684 | Replay ownership discovery; positions may move and new UnknownConstruct rows are disclosed below. Existing identities/statuses/dispositions remain unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 2–69 | Install the exact regenerated fingerprint cells; all other contract fields remain unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 57, 66, 103, 147, 151, 177, 227–228 | Relocate evidence references to identical source tokens; all other fields remain unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 2–3 | Install the exact regenerated C01/C02 certificate cells for both feature sets; only fingerprint tokens differ. |
| `docs/dev/codex/brief-575-g4-entry-base-cases.md` | 147–2415 | Required execution report. |

`final-source.patch` and `final-source-audit.json` contain every hunk and range against the incoming snapshot. Saved experimental sources, not a repository reset, preserve all supplied work. The final measured crate source equals the live source and verification export; both closed frame lists and every incoming test body are unchanged.

### 2. Pre-G4 against incoming: attribution

Each callgrind profile uses N=10 or 20 and is calibrated against native `taskset -c 2 perf stat -x, -e cpu_core/instructions/u` on the identical binary and input. Only full-N native slopes decide acceptance. Short callgrind totals include startup/interning variation. “Family self” sums definitions, held-pattern resolution, MatchQ and their work-vector drop symbols; it is not total program cost. Raw profiles, command lines and calibration tables are retained for every candidate.

| Label | Pre-G4 family self/iteration | Incoming family self/iteration | Difference |
| --- | ---: | ---: | ---: |
| `g4-rule-blank-constraint` | 1227 | 1853 | +626 |
| `g4-rule-skew-depth-8` | 11116 | 11979 | +863 |
| `matchq-integer-success` | 401 | 458 | +57 |
| `matchq-integer-fail` | 400 | 457 | +57 |
| `cases-pattern-child` | 1795 | 2207 | +412 |
| `g4-matchq-pattern-child` | 1706 | 2118 | +412 |
| `g4-matchq-pattern-head` | 1774 | 2151 | +377 |
| `g4-cases-pattern-child` | 1795 | 2207 | +412 |
| `g4-matchq-prepared-call` | 2742 | 3092 | +350 |
| `g4-matchq-skew-depth-8` | 7095 | 8142 | +1047 |
| `downvalue-definition` | 4886 | 4894 | +8 |
| `replaceall-simple` | 1325 | 1325 | +0 |

Incoming integer MatchQ: old recursive resolver/classifier self costs are 57 + 55; incoming flat driver is 166 plus 26 for empty work-vector drop. The incoming resolution family is 458 versus base 401 (success), and 457 versus 400 (failure). Opaque entry handling removes the root visit clone, driver and empty work-vector cleanup; the full panel, not the short total, measures the resulting saving.

Incoming short Blank rule: recursive helper 278 versus flat driver 734, finish 96 and work-vector drop 58. Definition-classifier self cost is 66 -> 82. At rule depth 8 the recursive helper is 4483 versus flat driver 6887, while classifier cost falls from 5867 to 4132; the explicit work records and transitions still leave a positive total residual.

Held active-child/Cases incoming driver/acceptance costs are 764 + 297, plus 29 for work-vector drop; root resolution entry self is 71. Candidate 01 outlined contains_pattern_syntax after adding the entry call site, raising the affected family by 106 instructions for active child/Cases and 355 at held depth 8. Candidate 02 restores classifier inlining; the remaining short-call family additions are 19 (child/Cases), 13 (head) and 18 (prepared call). Candidate 03's alternate head dispatch raises those short costs further and is rejected. Candidate 04's first-descent reservation saves native instructions in allocation helpers while allocation slopes stay unchanged; family self excludes those generic allocation helpers.

The newly failing candidate-04 controls were profiled, not waived or resampled. Downvalue-definition family self is 4894 in both incoming and candidate 04 (base 4886); ReplaceAll family self is 1325 in base, incoming and candidate 04. Their full native rows still reject candidate 04. Short total-profile variation includes interning/hash/allocator work, so equality of this family is not claimed as equality of every program instruction.

Candidate 05 does not outline the rule inert/policy helpers. Its cached root selector changes the hot visit representation: rule depth-8 driver self rises 6887 -> 7181, the existing classifier stays at 4132, and the separate entry has 98 self instructions. Candidate 06 therefore removes the local selector payload rather than changing a helper's inlining without evidence.

Candidate 07, the first retained optimization, gives integer MatchQ family self 289/288 (success/failure), versus incoming 458/457 and pre-G4 401/400. The opaque root has no resolver driver or empty work-vector drop in the measured family. Rule Blank still has driver 726, entry 116, finish 96 and work-vector cleanup 58 self instructions. Rule depth 8 has driver 6879, classifier 4132, entry 104, finish 96 and cleanup 58. Held active-child has driver 658, acceptance 297 and cleanup 29; held depth 8 has driver 5063, acceptance 1470 and cleanup 29. The latter vectors are populated structural walks, not empty base-case vectors. These profiles identify remaining flat traversal/storage and completion costs; they do not attribute every whole-program instruction to this family. The entry bypass removes the proved no-descent cost.

The isolated held reservation experiment (candidate 02 to 04) reduces RawVec grow_one self 130 -> 86 and finish_grow self 81 -> 54 on the active-child workload, a saving of 71 outside the selected family. Family self rises by 6 there, matching the roughly 65-instruction native improvement. The allocation-helper comparison is saved in candidate-04/attribution/versus-candidate-02-allocation-helpers.tsv. Candidate 07 applies the same first-descent reservation to Rule after its base cases; native Rule Blank improves about 135 instructions over incoming with unchanged allocation slopes.

Candidate 08 inlines finish_rule_lhs without changing completion semantics. The separately attributed 96-instruction finish symbol disappears and the selected family shrinks by 96 per Rule; this is not a claim of 96 whole-program instructions saved because its caller is outside that family. The full native panel saves about 38 instructions for immediate Rule workloads and 54 for the delayed structural label. On Blank, builtin_rule self rises 126 -> 182 while finish_rule_lhs falls 96 -> 0, a net 40 across those symbols. Completion symbol/caller self costs are saved in candidate-08/attribution/completion-versus-candidate-07.tsv. All held-family self counts match candidate 07; the small change in the largest held native sample still counts as nonprogress under the stated stop rule.

Candidate 09 outlines acceptance and is rejected by all six structural Rule neighbors. Compared with candidate 08, its selected family rises by 28 instructions on Blank, 177 on ordinary/delayed pattern-child, 229 on the wrapper, 577 at depth 8 and 1921 at depth 32. The native changes agree with these increases (approximately 28, 177, 229, 577 and 1921). Its held depth-8 family remains exactly 8074, so the tiny held maximum decrease is not attributed to the acceptance edit. Both driver and acceptance self slopes are saved in candidate-09/attribution/driver-acceptance.tsv. Acceptance therefore stays inlined in the final candidate-08 tree.



| Candidate | Calibrated profiles | Minimum CG/native difference | Maximum |
| --- | ---: | ---: | ---: |
| candidate-00 | 48 | +1.312017% | +1.885742% |
| candidate-01 | 36 | +0.861621% | +1.875644% |
| candidate-02 | 28 | +0.827338% | +1.892382% |
| candidate-03 | 28 | +1.291925% | +1.877859% |
| candidate-04 | 36 | +1.248591% | +1.906070% |
| candidate-05 | 24 | +0.749533% | +1.751341% |
| candidate-06 | 20 | +0.770627% | +1.793946% |
| candidate-07 | 52 | +0.839626% | +1.856677% |
| candidate-08 | 56 | +0.815834% | +1.899401% |
| candidate-09 | 28 | +0.810359% | +1.745617% |

### 3. Every measured candidate

Each candidate has fresh default and alloc-count targets with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`. Alloc-count uses the two arguments `--features rum-cli/alloc-count`. All hashes were checked; the fixed pre-G4 base was not rebuilt.

```text
88d29a74f1726412e82e89717e18ca42cc81f3a727ea088b14d356bc63de43a1  rum-cli
e6e4f9161ed454cb946015ad1e0bd8f012e5e7e23fd678245e8f4d81f6564628  rum-cli-alloc
```

Every candidate below ran all 23 labels: 46 result checks first, 92 allocation processes, then two paired sets of 92 native instruction processes. Each process had stdin `/dev/null` and a 150-second bound; native instruction runs used CPU 2. Slopes are `(count(2N)-count(N))/N`. A sustained base regression requires both samples >+0.05%; rejection also checks both candidate-versus-incoming/kept-parent samples. No sample was discarded or remeasured. There was no timing gate.

#### candidate-00 — incoming

Incoming tree, unchanged; 23 labels against fixed pre-G4 base.

Incoming measurement; no optimization.

10 sustained regressions; largest remaining sample +5.089971%.

Progress reference: `none (incoming)`; consecutive nonprogress count: 0.

```text
rum-cli 190024dafb1c8c45584113673fc50d6fffba358924deba107bdd5a3505936daf
rum-cli-alloc 1f685f8d1405fc5b30731b93b6cfeb9f240003618741ade939838ae55813ab4d
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.998950	999.027650	0.002873	999.129050	999.189050	0.006005	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	998.868800	999.207500	0.033908	998.999600	999.157200	0.015776	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22076.225200	21234.002700	-3.815066	22076.326000	21237.274350	-3.800685	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21786.344600	20935.649850	-3.904715	21786.202450	20939.721800	-3.885398	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.454400	18057.818900	5.082294	17183.973600	18058.632800	5.089971	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26314.607400	23546.142000	-10.520641	26310.240600	23543.769500	-10.514807	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59723.060000	61197.511000	2.468814	59730.204000	61206.936000	2.472337	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217267.736000	206785.281000	-4.824672	217280.541000	206795.236000	-4.825699	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.039050	5201.466000	1.116379	5144.253500	5201.083800	1.104734	True
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.292700	5323.170400	1.080033	5266.357400	5323.244150	1.080192	True
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412370.622000	2412379.577000	0.000371	2412538.084000	2412688.234000	0.006224	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39432.412300	40080.077450	1.642469	39554.947700	40093.700600	1.362037	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24121.781050	24130.075700	0.034387	24122.009350	24129.926750	0.032822	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11446.592450	11409.864250	-0.320866	11394.489600	11409.982950	0.135972	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.680650	254203.612750	0.002334	254196.040900	254204.333800	0.003262	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.812250	10035.180950	-0.026214	10051.576650	10033.810450	-0.176750	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.536900	49949.080850	-0.002915	49958.043550	49950.056950	-0.015987	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.215600	22702.022050	2.802157	22083.431750	22702.248950	2.802179	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.698750	21855.997900	3.222390	21173.709950	21855.701400	3.220935	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39497.857050	40082.581950	1.480397	39431.757600	40106.788550	1.711897	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.519650	15825.162450	3.645689	15268.490900	15825.268150	3.646577	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52481.126000	54863.180000	4.538877	52481.042000	54863.369000	4.539405	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204428.118000	199782.165000	-2.272658	204428.286000	199783.429000	-2.272121	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-00_total	base_rule_self	candidate-00_rule_self	candidate-00_continuation_self
g4-rule-blank-constraint	17439.2	18227.1	1227.0	1853.0	0
g4-rule-skew-depth-8	59487.8	61684.1	11116.0	11979.0	0
matchq-integer-success	5045.8	5164.9	401.0	458.0	0
matchq-integer-fail	5303.4	5173.4	400.0	457.0	0
cases-pattern-child	40292.2	40398.6	1795.0	2207.0	0
g4-matchq-pattern-child	22096.6	23102.3	1706.0	2118.0	0
g4-matchq-pattern-head	21092.5	21720.8	1774.0	2151.0	0
g4-cases-pattern-child	39918.3	40338.8	1795.0	2207.0	0
g4-matchq-prepared-call	15197.4	15730.0	2742.0	3092.0	0
g4-matchq-skew-depth-8	52291.9	54935.5	7095.0	8142.0	0
downvalue-definition	24431.5	24055.5	4886.0	4894.0	0
replaceall-simple	10716.1	11706.1	1325.0	1325.0	0
```

#### candidate-01 — rejected

Held entry settles opaque and pattern-free roots before the out-of-line driver; a Structural step carries the already-classified root, with the same descent body. Adds entry/driver falsifier across all opaque heads/arities and ordinary, error, suspension and descent controls.

Integer MatchQ regressions disappear, but seven held-pattern neighbors worsen versus incoming. Attribution and all 23 rows retained; next candidate revises the classification carrier without changing semantics.

8 sustained regressions; largest remaining sample +5.249235%.

Progress reference: `candidate-00`; consecutive nonprogress count: 0.

```text
rum-cli 4632a39103e4b532c65143542f7a3ba9a77999171b8f8222ef8f5d2bd4fef395
rum-cli-alloc 5eea3b46225278039c541f0d20a14b1aceed9220e3f45561e651cd0b304f5df6
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.945400	999.203600	0.025847	998.836200	998.801650	-0.003459	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.045150	999.104050	0.005896	999.293650	999.117350	-0.017642	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22075.337150	21234.693500	-3.808067	22076.987150	21234.636950	-3.815512	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.581800	20938.146550	-3.903123	21786.329850	20935.660250	-3.904603	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17185.761150	18060.098150	5.087566	17184.743250	18059.908750	5.092689	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26308.311500	23543.686800	-10.508560	26308.283500	23543.918350	-10.507585	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59722.607000	61210.022000	2.490539	59727.908000	61218.288000	2.495282	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217272.974000	206798.469000	-4.820896	217272.900000	206796.122000	-4.821944	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.637600	4971.203850	-3.371156	5144.320700	4971.410950	-3.361178	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.485200	5101.216300	-3.138125	5266.196800	5101.307550	-3.131088	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412442.405000	2412264.262000	-0.007384	2412443.191000	2412453.702000	0.000436	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39504.595400	40183.780500	1.719256	39488.103150	40209.309400	1.826389	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24121.922300	24130.128200	0.034018	24092.009900	24130.085500	0.158042	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.612200	11448.678400	0.359989	11407.553350	11384.043400	-0.206091	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.126600	254196.209200	-0.000361	254197.273800	254195.935950	-0.000526	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10052.013450	10033.647600	-0.182708	10044.831100	10033.291400	-0.114882	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.222000	49949.523000	-0.001399	49950.495550	49949.287000	-0.002419	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.455050	22808.108150	3.281430	22083.494800	22808.175600	3.281549	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.708000	21937.542500	3.607467	21173.733900	21937.678850	3.607984	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39496.156450	40257.991250	1.928883	39549.296150	40192.851300	1.627223	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.485100	15912.334800	4.216854	15268.466800	15912.325900	4.216920	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52476.942000	55231.580000	5.249235	52480.456000	55226.594000	5.232687	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204432.301000	200566.556000	-1.890966	204433.892000	200568.012000	-1.891017	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-01_total	base_rule_self	candidate-01_rule_self	candidate-01_continuation_self
matchq-integer-success	5228.6	4680.4	401.0	287.0	0
matchq-integer-fail	5499.4	5050.4	400.0	286.0	0
cases-pattern-child	39585.2	40686.1	1795.0	2313.0	0
g4-matchq-pattern-child	22497.9	22998.7	1706.0	2224.0	0
g4-matchq-pattern-head	21034.2	21803.0	1774.0	2233.0	0
g4-cases-pattern-child	39604.3	40426.3	1795.0	2313.0	0
g4-matchq-prepared-call	15593.1	15735.7	2742.0	3179.0	0
g4-matchq-skew-depth-8	52712.7	55325.6	7095.0	8497.0	0
g4-matchq-skew-depth-32	204471.1	200908.6	53067.0	45835.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 1 | 0 | 0 |
| unknown_macros.tsv | 7 | 0 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |
#### candidate-02 — rejected

Candidate-01 plus one classifier inlining attribute: retain the existing root-peeled iterative contains_pattern_syntax body inside callers, because candidate-01 outlined it and added 106 family instructions on the active-child label and 355 on held depth-8. Driver stays inline(never).

Classifier inlining recovers most driver cost and improves both held depths, but active child, head, Cases and prepared-call neighbors remain above the parent threshold. Four neighboring regressions reject this candidate.

8 sustained regressions; largest remaining sample +5.090793%.

Progress reference: `candidate-01`; consecutive nonprogress count: 0.

```text
rum-cli 1d04b0c6422de923dd4c8a0d2405e79fbd1708325c5854233d8fc97cdd383a9c
rum-cli-alloc fa0d3efe8f5ae73a33e4757e61cf27f94c028e7bd8db61efffa3b213f73f826d
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.097750	999.001650	-0.009619	998.930200	998.948800	0.001862	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.072800	998.969700	-0.010320	999.016200	999.021050	0.000485	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22076.829950	21235.258800	-3.812011	22075.554150	21234.521100	-3.809794	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21786.415700	20939.568050	-3.887044	21786.914600	20935.670300	-3.907136	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17185.285150	18060.152500	5.090793	17184.694350	18057.805300	5.080748	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26314.690900	23548.616400	-10.511522	26312.485250	23541.260950	-10.531975	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59718.262000	61214.547000	2.505574	59725.430000	61211.718000	2.488535	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217284.886000	206788.725000	-4.830599	217262.384000	206780.466000	-4.824543	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.194750	4971.360750	-3.359787	5144.317700	4971.321100	-3.362868	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.482650	5101.156300	-3.139218	5266.254550	5101.319900	-3.131916	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412441.282000	2412324.651000	-0.004835	2412520.146000	2412261.761000	-0.010710	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39538.615950	40118.593250	1.466863	39501.933800	40090.617200	1.490265	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24121.970900	24129.877450	0.032777	24121.940750	24129.985350	0.033350	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.848750	11448.944000	0.360237	11446.578200	11396.988600	-0.433226	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254201.688700	254195.877200	-0.002286	254201.157550	254196.079750	-0.001998	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10030.675600	10035.736550	0.050455	10037.645500	10035.408400	-0.022287	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.123400	49949.516450	-0.001215	49941.767850	49949.159200	0.014800	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.450200	22721.208050	2.887945	22083.493300	22721.104300	2.887274	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.600600	21868.816150	3.283407	21173.780850	21868.846750	3.282673	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39430.123750	40122.225700	1.755262	39575.955250	40171.154000	1.503940	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.520850	15843.241800	3.764091	15268.405600	15843.317400	3.765369	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52478.261000	54801.210000	4.426498	52477.082000	54797.771000	4.422290	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204432.453000	198291.723000	-3.003794	204434.547000	198283.591000	-3.008765	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-02_total	base_rule_self	candidate-02_rule_self	candidate-02_continuation_self
matchq-integer-success	5166.6	5032.6	401.0	287.0	0
g4-matchq-pattern-child	22245.0	22664.0	1706.0	2137.0	0
g4-matchq-pattern-head	21365.0	21720.3	1774.0	2164.0	0
g4-matchq-prepared-call	15280.0	15950.4	2742.0	3110.0	0
g4-matchq-skew-depth-8	52346.0	54755.0	7095.0	8068.0	0
g4-matchq-skew-depth-32	204667.9	198499.9	53067.0	43558.0	0
g4-cases-pattern-child	39740.7	40514.8	1795.0	2226.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 1 | 0 | 0 |
| unknown_macros.tsv | 7 | 0 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |
#### candidate-03 — rejected

Candidate-02 with the Structural root also bypassing the redundant opaque-head classification in the driver. Ordinary Visit values retain both classifier decisions; all descents share one work-push body. Candidate-02 residual family cost versus incoming is +19 child/Cases, +13 head, +18 prepared call.

Skipping the second driver classification changed the short-call dispatch shape and worsened five neighbors. The tiny maximum decrease is on unchanged Rule code and is not claimed as a causal improvement. Next: candidate-02 shape with work storage prepared only for its proven first descent.

8 sustained regressions; largest remaining sample +5.084753%.

Progress reference: `candidate-02`; consecutive nonprogress count: 0.

```text
rum-cli c307cf0ce9d41ebd5364ea870b19f8bc4427e2976dc1f842a303592abfe2e227
rum-cli-alloc cd250b888fde9586c5dcc7c5ead6b34fc521b23fbd60880c511f86a765610936
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.200600	998.789800	-0.041113	999.037500	999.020750	-0.001677	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	998.997400	999.066550	0.006922	999.093700	999.228200	0.013462	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22079.937450	21232.178750	-3.839498	22077.045900	21231.490900	-3.830019	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.435000	20935.523300	-3.914516	21787.626350	20939.522700	-3.892593	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.743000	18058.544800	5.084753	17185.540550	18058.420250	5.079152	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26310.207050	23548.652850	-10.496133	26308.134300	23543.930750	-10.507030	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59724.686000	61204.154000	2.477147	59720.252000	61210.336000	2.495107	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217270.407000	206849.295000	-4.796379	217273.869000	206795.397000	-4.822702	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.086250	4971.428100	-3.356440	5144.138250	4971.394450	-3.358071	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.333350	5101.151200	-3.136568	5266.090200	5101.409450	-3.127192	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412443.679000	2412186.093000	-0.010677	2412444.458000	2412307.880000	-0.005661	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39472.427450	40302.447050	2.102783	39487.985400	40129.269000	1.623997	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24121.822000	24130.030250	0.034028	24092.062750	24129.956850	0.157289	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.395600	11370.810100	-0.320717	11407.703750	11422.985950	0.133964	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254201.136300	254195.737100	-0.002124	254200.937400	254196.112750	-0.001898	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10052.161950	10035.695250	-0.163813	10030.740050	10035.157900	0.044043	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49966.620750	49949.321050	-0.034623	49966.637700	49949.112750	-0.035073	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.592650	22728.233100	2.919092	22083.410900	22728.002800	2.918896	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.411750	21878.661200	3.330826	21173.523500	21878.841500	3.331132	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39687.910600	40129.065300	1.111559	39454.107600	40128.316500	1.708843	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.368950	15845.345350	3.778900	15268.443550	15845.216700	3.777550	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52479.831000	54804.289000	4.429241	52479.569000	54802.324000	4.426018	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204429.870000	198286.309000	-3.005217	204429.647000	198285.793000	-3.005364	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-03_total	base_rule_self	candidate-03_rule_self	candidate-03_continuation_self
matchq-integer-success	5173.4	4907.8	401.0	287.0	0
g4-matchq-pattern-child	22309.8	23146.1	1706.0	2144.0	0
g4-matchq-pattern-head	21353.0	22064.8	1774.0	2174.0	0
g4-matchq-prepared-call	15360.4	16421.6	2742.0	3112.0	0
g4-matchq-skew-depth-8	52711.0	54593.9	7095.0	8070.0	0
cases-pattern-child	39581.6	40611.8	1795.0	2233.0	0
g4-cases-pattern-child	39902.8	40244.0	1795.0	2233.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 1 | 0 | 0 |
| unknown_macros.tsv | 7 | 0 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |
#### candidate-04 — rejected

From candidate-02: initialize the held work vector with four slots only after both entry base cases have returned, when the first descent is proved. Tests exercise the driver with an empty vector independently. This moves the initial growth out of the first push; allocation slopes check its storage cost.

First-descent allocation improves held traversal with unchanged allocation slopes, but downvalue-definition is a new base regression and replaceall-simple worsens versus incoming in both samples. Both controls were attributed; their definition/resolution family self costs equal incoming. No samples were repeated or discarded.

9 sustained regressions; largest remaining sample +5.099267%.

Progress reference: `candidate-03`; consecutive nonprogress count: 1.

```text
rum-cli 93d98e4297ecccf7d22563839a17cb8efa40240fda758fc38fab020d7f2fd23e
rum-cli-alloc 390d803a9022084e651bad9a0cd9b81daf25ce9118af4b6d2360d75b9f50686a
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.126600	999.084800	-0.004184	999.014300	999.085950	0.007172	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.095150	999.121700	0.002657	998.997300	998.862800	-0.013463	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22077.041050	21235.915400	-3.809956	22076.194500	21235.886600	-3.806398	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.151550	20938.250850	-3.900747	21786.080900	20939.527800	-3.885752	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.392200	18060.670300	5.099267	17184.642550	18058.335800	5.084151	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26314.649700	23543.799900	-10.529685	26314.738500	23543.631950	-10.530625	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59724.063000	61202.295000	2.475103	59723.957000	61204.676000	2.479271	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217266.303000	206791.262000	-4.821291	217271.725000	206793.264000	-4.822745	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.223050	4973.380850	-3.321050	5144.350900	4973.357650	-3.323903	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.196300	5103.376300	-3.091795	5266.116200	5103.300150	-3.091767	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412421.174000	2412313.388000	-0.004468	2412394.026000	2412268.823000	-0.005190	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39463.280350	40191.893650	1.846307	39480.035000	40053.911700	1.453587	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24091.968900	24189.899350	0.406486	24031.995000	24129.924550	0.407497	True
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11459.577100	11422.931600	-0.319781	11381.557400	11422.853750	0.362836	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.370350	254196.180550	-0.000468	254196.312050	254195.039650	-0.000501	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.762950	10034.280600	-0.034692	10052.195000	10035.167300	-0.169393	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.397550	49949.409550	-0.001978	49942.292850	49949.410800	0.014252	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.256350	22656.138700	2.594193	22083.290550	22656.244800	2.594515	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.776750	21803.699400	2.975013	21173.731650	21803.769000	2.975561	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39457.814050	40057.833650	1.520661	39481.169800	40208.644050	1.842585	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.459150	15778.133000	3.338083	15268.311950	15778.213100	3.339604	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52480.434000	54735.648000	4.297247	52481.399000	54733.575000	4.291380	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204432.541000	198224.743000	-3.036600	204432.759000	198223.252000	-3.037432	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-04_total	base_rule_self	candidate-04_rule_self	candidate-04_continuation_self
matchq-integer-success	5343.1	4823.1	401.0	289.0	0
g4-matchq-pattern-child	22438.0	22810.5	1706.0	2143.0	0
g4-matchq-pattern-head	20928.9	21744.2	1774.0	2170.0	0
g4-matchq-prepared-call	15170.3	15571.4	2742.0	3116.0	0
g4-matchq-skew-depth-8	52408.4	54707.9	7095.0	8074.0	0
cases-pattern-child	39824.1	40146.4	1795.0	2232.0	0
g4-cases-pattern-child	39879.0	40014.0	1795.0	2232.0	0
downvalue-definition	24360.5	24439.5	4886.0	4894.0	0
replaceall-simple	11270.1	11543.1	1325.0	1325.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 1 | 0 | 0 |
| unknown_macros.tsv | 7 | 0 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |
#### candidate-05 — rejected

From candidate-04: small Rule LHS entry handles atoms, pattern-free calls, inert pattern roots and empty wrappers; a Root visit carries the recognized head to the out-of-line driver. Direct driver root InputRequired returns the evaluator continuation without a redundant work frame. Adds exhaustive registry/arity and inert-root falsifier; both walkers compare Expr outcomes in separate private base-case test modules. Existing tests are unchanged.

Rule entry reduces the Blank residual and overall maximum, but five Rule traversal neighbors worsen versus incoming. Keep all falsifier and continuation evidence; next candidate is chosen from the calibrated helper/driver attribution.

8 sustained regressions; largest remaining sample +4.550376%.

Progress reference: `candidate-04`; consecutive nonprogress count: 0.

```text
rum-cli 9f51c433ed8de05940948ff27dc3d65fb53203766ca1935b9e40a9f91bde531c
rum-cli-alloc 745e3240e06f9c36afa826953afde5753f1a0e33a339251907feb20a1586304e
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.923650	998.921950	-0.000170	999.003850	999.208350	0.020470	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.129200	999.177800	0.004864	998.978550	999.123700	0.014530	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22076.968900	21374.059000	-3.183906	22076.867550	21371.534050	-3.194898	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.346950	21040.814350	-3.430883	21789.196450	21073.181950	-3.286099	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.606050	17930.241250	4.338972	17184.721550	17966.691050	4.550376	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26308.113600	23634.561100	-10.162464	26308.087350	23629.512500	-10.181564	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59727.340000	61566.261000	3.078860	59720.806000	61568.351000	3.093637	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217264.065000	207870.120000	-4.323745	217267.818000	207882.088000	-4.319890	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.252300	4973.623650	-3.316880	5144.167700	4973.258350	-3.322391	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.335000	5095.455600	-3.244750	5266.234400	5095.193750	-3.247874	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412444.212000	2412258.499000	-0.007698	2412447.135000	2412228.793000	-0.009051	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39471.949800	40048.135950	1.459736	39485.601750	40049.576650	1.428305	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24152.064950	24309.823250	0.653188	24122.006900	24129.796200	0.032291	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.626450	11410.106300	0.021739	11407.615150	11422.865950	0.133690	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.329850	254203.904350	0.002586	254197.605950	254204.091550	0.002551	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10044.884700	10035.304550	-0.095373	10052.003200	10033.333850	-0.185728	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.251550	49949.362600	-0.001780	49950.213050	49949.522150	-0.001383	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.545700	22656.202300	2.593137	22083.389650	22656.150250	2.593626	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.652600	21803.736650	2.975793	21173.663800	21803.720300	2.975661	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39496.184400	40104.748150	1.540817	39546.863950	40024.943950	1.208895	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.288100	15778.213250	3.339766	15268.579100	15778.278900	3.338227	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52483.026000	54726.391000	4.274458	52477.372000	54725.612000	4.284208	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204431.407000	198216.847000	-3.039924	204431.773000	198215.053000	-3.040975	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-05_total	base_rule_self	candidate-05_rule_self	candidate-05_continuation_self
g4-rule-blank-constraint	17371.5	18003.9	1227.0	1877.0	0
g4-rule-pattern-child	22334.7	21635.5	2164.0	2873.0	0
g4-ruledelayed-pattern-child	22074.1	21298.4	2084.0	2803.0	0
g4-rule-wrapper	26116.1	23394.2	3550.0	4227.0	0
g4-rule-skew-depth-8	59883.0	61606.7	11116.0	12371.0	0
g4-rule-skew-depth-32	217356.8	208129.8	80500.0	68427.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 1 | 0 | 0 |
| unknown_macros.tsv | 19 | 1 | 0 |
| unknown_storage.tsv | 0 | 1 | 0 |

Candidate-added UnknownConstruct rows, verbatim:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	38	20	unexpanded macro matches! (ends at 41:18)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	21	18	rum_runtime::runtime::definitions::rule_lhs::RuleLhsVisit.Root.head/std::option::Option[Own]/arg0: unresolved/ambiguous type (): MatcherPatternHead
```

#### candidate-06 — rejected

From candidate-05: remove the cached-root payload from RuleLhsVisit and restore the original scalar visit modes and driver selector lookup. Both small entries and falsifiers stay. Attribution rules out helper outlining: depth-8 driver self rose 6887 -> 7181 while classifier self remained 4132; the root payload changed the hot transition representation.

Removing the selector payload recovers most Rule loop cost, but pattern-child, delayed-pattern-child and depth-8 Rule neighbors still regress versus incoming. Relative to consecutive measured candidate 05, neither regression count nor maximum falls; no-progress streak is one.

8 sustained regressions; largest remaining sample +5.059682%.

Progress reference: `candidate-05`; consecutive nonprogress count: 1.

```text
rum-cli 3d95902c338a38e3f874861145d7b38dcea9cbc1f64652e9fc2c464a191d770d
rum-cli-alloc 38cc3606ad8d3d2840588e1358405f6e5074aeb9f1c6c03b5b1195bf3ab47ced
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.084100	999.009900	-0.007427	998.969400	999.079950	0.011066	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.109350	999.034300	-0.007512	999.023200	999.095800	0.007267	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22075.543400	21298.611650	-3.519423	22075.872350	21296.041600	-3.532503	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21789.201700	21003.532900	-3.605771	21788.705850	21003.319200	-3.604559	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17186.129550	18053.281600	5.045651	17184.546050	18054.029350	5.059682	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26314.672000	23557.971800	-10.475906	26310.362100	23555.321800	-10.471313	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59723.604000	61267.772000	2.585524	59723.488000	61279.938000	2.606094	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217266.689000	206847.894000	-4.795395	217263.959000	206847.224000	-4.794507	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.143400	4973.404650	-3.319090	5144.450350	4973.421500	-3.324531	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.082250	5095.236150	-3.244273	5266.259400	5095.224750	-3.247744	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412558.887000	2412243.257000	-0.013083	2412470.890000	2412254.585000	-0.008966	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39516.948550	40049.025950	1.346454	39449.528300	40098.607650	1.645341	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24062.018550	24091.966900	0.124463	24182.187750	24129.984500	-0.215875	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.703450	11422.897550	0.133192	11407.806300	11396.761700	-0.096816	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254197.554750	254203.640000	0.002394	254196.595750	254204.300000	0.003031	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.797100	10035.750150	-0.020392	10052.046750	10035.197300	-0.167622	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49941.783500	49949.328300	0.015107	49942.069750	49949.459100	0.014796	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.281300	22656.164300	2.594193	22083.427850	22656.095450	2.593201	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.767600	21803.599400	2.974585	21173.798050	21803.543650	2.974174	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39488.220250	40145.193800	1.663720	39477.959250	40020.520600	1.374340	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.449100	15778.438550	3.340152	15268.339800	15778.568600	3.341744	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52481.811000	54724.513000	4.273294	52482.476000	54723.772000	4.270561	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204435.852000	198214.950000	-3.042960	204433.198000	198212.938000	-3.042686	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-06_total	base_rule_self	candidate-06_rule_self	candidate-06_continuation_self
g4-rule-blank-constraint	17074.5	18257.8	1227.0	1873.0	0
g4-rule-pattern-child	22345.9	21749.4	2164.0	2799.0	0
g4-rule-wrapper	26293.4	23605.5	3550.0	4121.0	0
g4-rule-skew-depth-8	59808.4	61384.3	11116.0	12069.0	0
g4-rule-skew-depth-32	217190.3	207226.9	80500.0	67405.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 1 | 0 | 0 |
| unknown_macros.tsv | 19 | 1 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |

Candidate-added UnknownConstruct rows, verbatim:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	37	20	unexpanded macro matches! (ends at 40:18)
```

#### candidate-07 — kept

From candidate-06: prepare four Rule work slots only for the proved first descent, after all root base cases return. This mirrors the measured held initialization and avoids the first push growth. No driver control flow, visit representation, classifier or semantic change.

Both small entries plus first-descent work-vector reservation pass all 23 neighbor comparisons: no new fixed-base regression and no sustained increase against the incoming tree. Regressions fall from ten to eight and the maximum falls from 5.089970576% to 4.280695485%. The entry/driver falsifiers and structural/continuation/ownership preflight pass.

8 sustained regressions; largest remaining sample +4.280695%.

Progress reference: `candidate-06`; consecutive nonprogress count: 0.

```text
rum-cli b8b9896dcdc8c230f5087064a079bee90e7c308e36369a2399ddd44fd0a33273
rum-cli-alloc cba6124692108ba1309a31129199f4b92d702a30a288a672c9551aaade1df170
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.061900	999.021300	-0.004064	999.036500	999.035550	-0.000095	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.058850	999.152800	0.009404	999.047550	998.998650	-0.004895	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22079.860100	21162.605050	-4.154261	22075.438500	21163.391400	-4.131502	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21789.986450	20867.561350	-4.233252	21789.794450	20867.472650	-4.232816	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17185.462650	17916.903350	4.256159	17185.255300	17916.669000	4.256054	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26310.330750	23419.469450	-10.987552	26308.042250	23421.864150	-10.970706	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59730.176000	61134.479000	2.351078	59719.998000	61128.515000	2.358535	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217265.996000	206719.550000	-4.854163	217277.198000	206717.153000	-4.860172	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.113750	4973.296050	-3.320644	5144.536150	4973.317950	-3.328156	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.356100	5095.413600	-3.245935	5266.285300	5095.328150	-3.246257	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412397.228000	2412258.023000	-0.005770	2412657.328000	2412259.152000	-0.016504	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39501.188900	40050.508050	1.390640	39480.080150	40049.996600	1.443554	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.004950	24130.037150	0.033298	24121.977550	24206.073400	0.348628	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11394.464050	11396.887850	0.021272	11407.621050	11371.154700	-0.319667	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254196.599400	254203.488450	0.002710	254199.950050	254204.455200	0.001772	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10052.017550	10035.407100	-0.165245	10030.499750	10035.286100	0.047718	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49958.409600	49950.252550	-0.016328	49958.081350	49949.308250	-0.017561	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.472000	22655.908750	2.592150	22083.403150	22655.969000	2.592743	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.599650	21803.489750	2.974884	21173.694800	21803.785800	2.975820	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39693.292600	40144.541550	1.136839	39472.330950	40044.787700	1.450273	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.473800	15778.541450	3.340659	15268.431000	15778.219650	3.338841	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52478.108000	54724.536000	4.280695	52483.687000	54726.967000	4.274242	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204432.409000	198209.428000	-3.044029	204431.222000	198215.186000	-3.040649	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-07_total	base_rule_self	candidate-07_rule_self	candidate-07_continuation_self
matchq-integer-success	5311.1	4733.5	401.0	289.0	0
matchq-integer-fail	5075.0	5323.4	400.0	288.0	0
g4-rule-blank-constraint	17119.0	17662.6	1227.0	1879.0	0
g4-rule-pattern-child	22245.9	21511.1	2164.0	2805.0	0
g4-rule-wrapper	26376.1	23484.2	3550.0	4127.0	0
g4-rule-skew-depth-8	59756.8	61317.2	11116.0	12075.0	0
g4-rule-skew-depth-32	217426.8	207791.2	80500.0	67411.0	0
cases-pattern-child	40094.4	40517.7	1795.0	2232.0	0
g4-matchq-pattern-child	22340.0	22723.4	1706.0	2143.0	0
g4-matchq-pattern-head	21252.7	22170.3	1774.0	2170.0	0
g4-cases-pattern-child	40010.0	40356.2	1795.0	2232.0	0
g4-matchq-prepared-call	15361.7	15809.8	2742.0	3116.0	0
g4-matchq-skew-depth-8	52452.8	54738.3	7095.0	8074.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 1 | 0 | 0 |
| unknown_macros.tsv | 19 | 1 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |

Candidate-added UnknownConstruct rows, verbatim:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	37	20	unexpanded macro matches! (ends at 40:18)
```

#### candidate-08 — kept

From kept candidate-07: inline the small finish_rule_lhs completion helper into its immediate/delayed callers. Calibrated candidate-07 Rule profiles retain the same 96 self instructions in this helper. The driver remains inline(never); no walk or suspension policy changes.

Inlining completion improves every structural Rule label without a sustained neighbor increase or a new base regression. Short Blank improves from about +4.256% to +4.039%, but regression count remains eight and the maximum held depth-8 sample rises slightly; this is one stop-loss nonprogress candidate.

8 sustained regressions; largest remaining sample +4.283887%.

Progress reference: `candidate-07`; consecutive nonprogress count: 1.

```text
rum-cli 86be5519d715462ece77af4ad96300d2d73cb3ab8e8d2b0bdb019662d70cd6bf
rum-cli-alloc 4704630d518097b639f104e12784da2f49933a9895e7f37a739cc1210c071166
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	999.103650	999.119650	0.001601	998.973950	998.851200	-0.012288	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	999.060450	999.015400	-0.004509	999.222900	999.001250	-0.022182	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22077.018150	21125.333650	-4.310747	22076.919150	21120.270900	-4.333251	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.684100	20812.782200	-4.478939	21785.733500	20813.406800	-4.463135	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17184.619400	17878.747550	4.039241	17184.151600	17877.977450	4.037592	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26312.557850	23381.797500	-11.138257	26312.420550	23381.737750	-11.138021	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59721.336000	61091.719000	2.294629	59730.866000	61097.607000	2.288165	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217269.617000	206681.891000	-4.873082	217273.886000	206679.598000	-4.876006	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.403800	4973.251650	-3.326958	5144.219700	4973.305550	-3.322450	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.182450	5095.105200	-3.248601	5266.129950	5095.338650	-3.243203	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412563.027000	2412257.811000	-0.012651	2412473.984000	2412259.030000	-0.008910	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39553.940250	40031.041200	1.206203	39487.267700	40190.166600	1.780065	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24122.118400	24130.090050	0.033047	24121.964800	24206.215300	0.349269	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.721800	11370.922650	-0.322581	11381.479400	11396.849200	0.135042	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254201.341100	254203.930500	0.001019	254197.768100	254203.825900	0.002383	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.775500	10035.645100	-0.021224	10044.915400	10035.408750	-0.094641	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49941.680400	49949.376950	0.015411	49966.583450	49949.672850	-0.033844	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.565200	22656.196900	2.593022	22083.532800	22656.219950	2.593277	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.720500	21803.813850	2.975827	21173.592100	21803.768100	2.976236	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39487.241950	40047.924850	1.419909	39462.319600	40049.268200	1.487365	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.450350	15778.154850	3.338286	15268.379250	15778.278400	3.339576	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52481.695000	54727.926000	4.280028	52479.415000	54727.574000	4.283887	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204433.323000	198213.938000	-3.042256	204430.653000	198214.607000	-3.040662	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-08_total	base_rule_self	candidate-08_rule_self	candidate-08_continuation_self
g4-rule-blank-constraint	16795.5	17946.3	1227.0	1783.0	0
g4-rule-pattern-child	22396.8	21457.7	2164.0	2709.0	0
g4-ruledelayed-pattern-child	22021.7	20789.7	2084.0	2629.0	0
g4-rule-wrapper	26107.2	23386.7	3550.0	4031.0	0
g4-rule-skew-depth-8	59905.0	61126.3	11116.0	11979.0	0
g4-rule-skew-depth-32	217325.5	207317.2	80500.0	67315.0	0
matchq-integer-success	4943.4	4562.7	401.0	289.0	0
matchq-integer-fail	5095.4	5156.5	400.0	288.0	0
cases-pattern-child	39793.9	40274.4	1795.0	2232.0	0
g4-matchq-pattern-child	21782.4	22942.8	1706.0	2143.0	0
g4-matchq-pattern-head	21186.8	21755.1	1774.0	2170.0	0
g4-cases-pattern-child	39745.6	40169.7	1795.0	2232.0	0
g4-matchq-prepared-call	15224.1	15897.1	2742.0	3116.0	0
g4-matchq-skew-depth-8	52396.3	54561.2	7095.0	8074.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 4 | 0 | 0 |
| unknown_macros.tsv | 31 | 1 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |

Candidate-added UnknownConstruct rows, verbatim:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	37	20	unexpanded macro matches! (ends at 40:18)
```

#### candidate-09 — rejected

From kept candidate-08: outline accept_rule_lhs_value with inline(never), moving its work-record mutation and completion locals out of the driver frame. Candidate-08 attribution still has substantial self cost in the driver with acceptance inlined. This is the final slot and the brief’s requested acceptance-transition experiment; no algorithm or capture change.

Outlining acceptance worsens all six structural Rule labels versus kept candidate-08: about +0.15% to +0.98%. The largest fixed-base sample falls only on unchanged held code; that literal counter progress does not waive the six neighbor failures. Stop at ten measured candidates, restore candidate-08, and run standing-duty and final verification.

8 sustained regressions; largest remaining sample +4.280318%.

Progress reference: `candidate-08`; consecutive nonprogress count: 0.

```text
rum-cli 159c96b2767340f72a4a6b062acf56e0f002ced7309d92d9a36d9476742f96fc
rum-cli-alloc f2bfd9f83b81d1f2d850029f6486100ce5a7df2b3d067aa70ee9adac648063b7
```

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.000000	0.000000	32.000000	32.000000	998.934300	999.384550	0.045073	999.075050	998.997800	-0.007732	False
g4-ruledelayed-plain	20000	0.000000	0.000000	32.000000	32.000000	998.934050	999.232600	0.029887	999.052500	998.952500	-0.010009	False
g4-rule-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	22076.269200	21299.811550	-3.517160	22076.102450	21300.980200	-3.511137	False
g4-ruledelayed-pattern-child	20000	25.000000	17.000000	1400.000000	1816.000000	21788.427150	20989.157600	-3.668321	21786.939950	20989.258100	-3.661284	False
g4-rule-blank-constraint	20000	16.000000	18.000000	1112.000000	1816.000000	17186.273550	17906.788600	4.192387	17185.490800	17904.546950	4.184089	True
g4-rule-wrapper	20000	47.000000	29.000000	2488.000000	2296.000000	26314.770400	23608.055950	-10.285913	26310.237200	23610.867150	-10.259771	False
g4-rule-skew-depth-8	1000	73.000000	69.000000	3256.000000	9048.000000	59723.196000	61676.672000	3.270883	59729.073000	61664.332000	3.240062	True
g4-rule-skew-depth-32	1000	217.000000	217.000000	9016.000000	36312.000000	217269.738000	208605.736000	-3.987671	217268.312000	208600.770000	-3.989326	False
matchq-integer-success	20000	1.000000	1.000000	160.000000	160.000000	5144.325600	4973.229050	-3.325928	5144.234550	4973.198900	-3.324803	False
matchq-integer-fail	20000	2.000000	2.000000	161.000000	161.000000	5266.239150	5095.096000	-3.249817	5266.352600	5095.366250	-3.246770	False
cases-wide-no-match	1000	33.001000	33.001000	263279.864000	263279.864000	2412355.287000	2412259.686000	-0.003963	2412421.045000	2412260.293000	-0.006664	False
cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39494.608250	40047.774000	1.400611	39498.008450	40141.478750	1.629121	True
downvalue-definition	20000	29.000000	29.000000	3172.000000	3172.000000	24197.979900	24189.817000	-0.033734	24122.186300	24092.087600	-0.124776	False
replaceall-simple	20000	5.000000	5.000000	1352.000000	1352.000000	11407.676100	11383.948650	-0.207995	11433.466150	11370.833850	-0.547798	False
dot-three-no-input	20000	596.000000	596.000000	23504.000000	23504.000000	254196.612150	254203.888550	0.002863	254197.392650	254204.372700	0.002746	False
differences-no-input	20000	13.000000	13.000000	704.000000	704.000000	10037.623250	10033.748400	-0.038603	10037.828450	10035.168750	-0.026497	False
take-multispec-small	20000	109.000000	109.000000	5856.000000	5856.000000	49950.432000	49949.285300	-0.002296	49942.080850	49949.477650	0.014811	False
g4-matchq-pattern-child	20000	16.000000	17.000000	2428.000000	2844.000000	22083.617750	22656.171750	2.592664	22083.443400	22656.090150	2.593104	True
g4-matchq-pattern-head	20000	16.000000	18.000000	2364.000000	2844.000000	21173.557250	21803.685000	2.976013	21173.717650	21803.742500	2.975504	True
g4-cases-pattern-child	20000	35.000000	36.000000	5744.000000	6160.000000	39479.967100	40170.064250	1.747968	39521.824250	40050.608750	1.337956	True
g4-matchq-prepared-call	20000	8.000000	9.000000	488.000000	904.000000	15268.399300	15778.311400	3.339657	15268.485100	15778.253400	3.338696	True
g4-matchq-skew-depth-8	1000	60.000000	63.000000	7312.000000	10224.000000	52480.840000	54726.891000	4.279754	52478.489000	54724.735000	4.280318	True
g4-matchq-skew-depth-32	1000	222.000000	227.000000	30384.000000	43280.000000	204432.474000	198213.881000	-3.041881	204434.161000	198213.771000	-3.042735	False
```

Calibrated self-attribution:

```tsv
label	base_total	candidate-09_total	base_rule_self	candidate-09_rule_self	candidate-09_continuation_self
g4-rule-blank-constraint	17336.6	17985.8	1227.0	1811.0	0
g4-rule-pattern-child	22237.1	21197.4	2164.0	2886.0	0
g4-ruledelayed-pattern-child	21804.1	21454.5	2084.0	2806.0	0
g4-rule-wrapper	26276.2	23973.3	3550.0	4260.0	0
g4-rule-skew-depth-8	59813.5	61501.7	11116.0	12556.0	0
g4-rule-skew-depth-32	217378.4	209136.3	80500.0	69236.0	0
g4-matchq-skew-depth-8	52277.0	54639.6	7095.0	8074.0	0
```

Ownership replay for this candidate:

| Inventory | Position moves | Added | Removed incoming rows |
| --- | ---: | ---: | ---: |
| host_payloads.tsv | 0 | 0 | 0 |
| unknown_attributes.tsv | 0 | 0 | 0 |
| unknown_captures.tsv | 4 | 0 | 0 |
| unknown_macros.tsv | 31 | 1 | 0 |
| unknown_storage.tsv | 0 | 0 | 0 |

Candidate-added UnknownConstruct rows, verbatim:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	37	20	unexpanded macro matches! (ends at 40:18)
```

### 4. Verification, verbatim

Commands ran serially with the shown bounds, one build job, one test thread, offline Cargo and brief-local TMPDIR. Final verification uses an identical source export under brief-575 scratch so test-generated artifacts stay there. Complete output for every command is in the corresponding `.temp/brief-575/logs/<label>.txt`; the following excerpts preserve result lines verbatim. Durations in metadata enforce bounds and are not performance measurements.

#### candidate-00-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-00/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-00-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-00-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-00/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-00-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-01-fmt

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-01-fmt.txt`.

Verbatim output: empty.

#### candidate-01-held-private

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **101**. Full verbatim output: `.temp/brief-575/logs/candidate-01-held-private.txt`.

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
error[E0277]: can't compare `&pattern_syntax::MatcherPatternHead` with `pattern_syntax::MatcherPatternHead`
   --> crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:414:17
    |
414 |         if head == MatcherPatternHead::OptionsPattern {
    |                 ^^ no implementation for `&pattern_syntax::MatcherPatternHead == pattern_syntax::MatcherPatternHead`

error[E0277]: can't compare `&pattern_syntax::MatcherPatternHead` with `pattern_syntax::MatcherPatternHead`
   --> crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:419:21
    |
419 |             if head == MatcherPatternHead::HoldPattern && arity == 1 {
    |                     ^^ no implementation for `&pattern_syntax::MatcherPatternHead == pattern_syntax::MatcherPatternHead`
For more information about this error, try `rustc --explain E0277`.
error: could not compile `rum-runtime` (lib test) due to 2 previous errors
```

#### candidate-01-held-private-fixed

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **101**. Full verbatim output: `.temp/brief-575/logs/candidate-01-held-private-fixed.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 31.34s
thread 'runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_entry_base_cases_agree_with_driver' panicked at crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs:425:13:
KeyValuePattern[InputString["opaque: "]]: (Ok("KeyValuePattern[\"7\"]"), ["opaque: "], "0", "[]")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_error_resume_runs_no_work ... ok
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_missing_root_falsifier - should panic ... 
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.51s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

#### candidate-01-held-private-qualified

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-01-held-private-qualified.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.44s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.76s
```

#### candidate-01-structural

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **101**. Full verbatim output: `.temp/brief-575/logs/candidate-01-structural.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 25s
thread 'tests::association_depth_contract::rule_pattern_depth_contract::structural_rule_pattern_depth_contract' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
unaudited G4 outbound calls: {"Session::resolve_held_matcher_pattern -> MatcherPatternStep::Structural"}
FAILED
test tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
unaudited G4 outbound calls: {"drive -> moved_to_unscanned_helper"}
ok

failures:

test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.35s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

#### candidate-01-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-01-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.47s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.59s
```

#### candidate-01-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-01-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.11s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.31s
```

#### candidate-01-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-01-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.51s
```

#### candidate-01-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-01/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-01-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

#### candidate-01-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-01/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-01-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### candidate-02-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-02-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.08s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.42s
```

#### candidate-02-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-02-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 27.49s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.37s
```

#### candidate-02-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-02-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.51s
```

#### candidate-02-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-02/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-02-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```

#### candidate-02-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-02/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-02-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-03-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-03-format.txt`.

Verbatim output: empty.

#### candidate-03-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-03-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.56s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.67s
```

#### candidate-03-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-03-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.27s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.37s
```

#### candidate-03-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-03-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.42s
```

#### candidate-03-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-03/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-03-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 49s
```

#### candidate-03-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-03/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-03-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 40s
```

#### candidate-04-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-04-format.txt`.

Verbatim output: empty.

#### candidate-04-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-04-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.17s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.53s
```

#### candidate-04-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-04-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 30.17s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.34s
```

#### candidate-04-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-04-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.44s
```

#### candidate-04-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-04/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-04-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-04-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-04/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-04-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 41s
```

#### candidate-05-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-format.txt`.

Verbatim output: empty.

#### candidate-05-rule-private

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-rule-private.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.63s
test runtime::definitions::rule_lhs::tests::base_cases::rule_lhs_entry_base_cases_agree_with_driver ... RULE_ENTRY_DRIVER_PARITY inert=15 registry=76 ordinary/error/resume/malformed/descent=22 passed
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1110 filtered out; finished in 0.77s
```

#### candidate-05-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 31.79s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

#### candidate-05-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::base_cases::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.80s
```

#### candidate-05-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 31.66s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.37s
```

#### candidate-05-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.49s
```

#### candidate-05-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-05/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 47s
```

#### candidate-05-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-05/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-05-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 41s
```

#### candidate-06-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-format.txt`.

Verbatim output: empty.

#### candidate-06-rule-private

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-rule-private.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.68s
test runtime::definitions::rule_lhs::tests::base_cases::rule_lhs_entry_base_cases_agree_with_driver ... RULE_ENTRY_DRIVER_PARITY inert=15 registry=76 ordinary/error/resume/malformed/descent=22 passed
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1110 filtered out; finished in 0.93s
```

#### candidate-06-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.51s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
```

#### candidate-06-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::base_cases::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.72s
```

#### candidate-06-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.35s
```

#### candidate-06-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.51s
```

#### candidate-06-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-06/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 45s
```

#### candidate-06-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-06/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-06-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 42s
```

#### candidate-07-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-format.txt`.

Verbatim output: empty.

#### candidate-07-rule-private

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-rule-private.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.31s
test runtime::definitions::rule_lhs::tests::base_cases::rule_lhs_entry_base_cases_agree_with_driver ... RULE_ENTRY_DRIVER_PARITY inert=15 registry=76 ordinary/error/resume/malformed/descent=22 passed
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1110 filtered out; finished in 0.64s
```

#### candidate-07-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.19s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

#### candidate-07-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::base_cases::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.80s
```

#### candidate-07-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.38s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.36s
```

#### candidate-07-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.58s
```

#### candidate-07-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-07/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### candidate-07-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-07/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-07-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### candidate-08-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-format.txt`.

Verbatim output: empty.

#### candidate-08-rule-private

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-rule-private.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.75s
test runtime::definitions::rule_lhs::tests::base_cases::rule_lhs_entry_base_cases_agree_with_driver ... RULE_ENTRY_DRIVER_PARITY inert=15 registry=76 ordinary/error/resume/malformed/descent=22 passed
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1110 filtered out; finished in 1.00s
```

#### candidate-08-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.34s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

#### candidate-08-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::base_cases::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.66s
```

#### candidate-08-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.45s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.31s
```

#### candidate-08-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.55s
```

#### candidate-08-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-08/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 45s
```

#### candidate-08-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-08/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-08-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 43s
```

#### candidate-09-format

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo fmt -p rum-runtime < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-format.txt`.

Verbatim output: empty.

#### candidate-09-rule-private

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib rule_lhs -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-rule-private.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.56s
test runtime::definitions::rule_lhs::tests::base_cases::rule_lhs_entry_base_cases_agree_with_driver ... RULE_ENTRY_DRIVER_PARITY inert=15 registry=76 ordinary/error/resume/malformed/descent=22 passed
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1110 filtered out; finished in 0.94s
```

#### candidate-09-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.49s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
```

#### candidate-09-held-final

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-runtime --lib matcher_pattern -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-held-final.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::base_cases::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.40s
```

#### candidate-09-structural-guarded

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-structural-guarded.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.87s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.37s
```

#### candidate-09-ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.55s
```

#### candidate-09-build-default

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-09/target-default cargo build --release -p rum-cli < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-build-default.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 46s
```

#### candidate-09-build-alloc

```sh
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/test-target env CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/candidate-09/target-alloc cargo build --release -p rum-cli --features rum-cli/alloc-count < /dev/null
```

cwd: `/home/remi/dev/rum`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/candidate-09-build-alloc.txt`.

```text
    Finished `release` profile [optimized] target(s) in 1m 44s
```

#### ownership-discovery

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/ownership-discovery.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 31s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 5.50s
```

#### fingerprints-replay

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/fingerprints-replay.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.61s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.50s
```

#### certificates-true-replay

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **101**. Full verbatim output: `.temp/brief-575/logs/certificates-true-replay.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.06s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 17.20s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

#### certificates-false-replay

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **101**. Full verbatim output: `.temp/brief-575/logs/certificates-false-replay.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 49.00s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1103 filtered out; finished in 17.55s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

#### final-fmt

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo fmt --check < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-fmt.txt`.

Verbatim output: empty.

#### final-continuations

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --test rule_pattern_continuations < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-continuations.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 26.29s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

#### final-structural

```sh
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-structural.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.19s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.37s
```

#### final-runtime

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-runtime -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-runtime.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 57.27s
test runtime::definitions::rule_lhs::tests::base_cases::rule_lhs_entry_base_cases_agree_with_driver ... RULE_ENTRY_DRIVER_PARITY inert=15 registry=76 ordinary/error/resume/malformed/descent=22 passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::base_cases::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
warning: function `consume` is never used
warning: 1 warning emitted
test result: ok. 1120 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 25.24s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.18s
```

#### final-ownership

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --lib ownership_graph -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-ownership.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 59.24s
```

#### final-retention-default

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-retention-default.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 17.73s
```

#### final-retention-no-default

```sh
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-retention-no-default.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.93s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1103 filtered out; finished in 17.93s
```

#### final-workspace

```sh
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-575/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-575/target-verification env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture < /dev/null
```

cwd: `/home/remi/dev/rum/.temp/brief-575/verification-source`; exit: **0**. Full verbatim output: `.temp/brief-575/logs/final-workspace.txt`.

```text
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5m 25s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.29s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.40s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.36s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.33s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.26s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.94s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 290.21s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test runtime::definitions::rule_lhs::tests::base_cases::rule_lhs_entry_base_cases_agree_with_driver ... RULE_ENTRY_DRIVER_PARITY inert=15 registry=76 ordinary/error/resume/malformed/descent=22 passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::base_cases::matcher_pattern_entry_base_cases_agree_with_driver ... HELD_ENTRY_DRIVER_PARITY opaque=71 ordinary/held/OptionsPattern/error/resume/descent=15 passed
warning: function `consume` is never used
warning: 1 warning emitted
test result: ok. 1120 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 25.18s
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
expired-C01: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=true
expired-C02: RED stale or omitted instance certificate C02 hash=true
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1057 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1425.70s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.95s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.45s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.63s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Metric/check/attribution command counts: `{"checks": 460, "metrics": 2760, "attribution_commands": 712}`. Every recorded metric process exited zero and passed its output check. `metric-command-results.md` expands all commands, statuses and verbatim streams/CSV; every per-process bound was 150 seconds.

Workspace reported totals: `{"groups": 44, "passed": 3115, "failed": 0, "ignored": 58}`. Native skips below are not exercised coverage even when the harness counts them as successful.

Native ZeroMQ skips, verbatim:

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

Ignored tests, verbatim (the fingerprint regeneration test was separately invoked):

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

All mandatory final gates and both retention feature-set gates ran. No timing benchmark, forbidden external-engine comparison or aborting probe was run.

### 5. Counters, standing duty and residuals

```json
{
  "measured_candidates": 10,
  "optimization_candidates": 9,
  "kept": "candidate-08",
  "progress_reference": "candidate-08",
  "no_progress_streak": 0,
  "accepted_no_progress_streak": 2,
  "regressions": 8,
  "largest_remaining_delta": 4.283887310862755
}
```

Final source audit: `python3 scripts/run-bounded.py --timeout 150 -- python3 .temp/brief-575/audit.py candidate-08`; exit 0. Verbatim output:

```text
AUDIT PASS: 12 allowed changed files; existing tests/frame lists/base digests unchanged; measured source and export match; no new unsafe.
```

The stop-loss counter compares each measured candidate with the immediately preceding measured candidate; retention compares it with the incoming or currently retained tree. These are separate decisions. The helper initially inherited the previous brief’s kept-parent comparison; that accounting was corrected before candidate 06. The unreconciled state is preserved in counters-before-reconciliation.json. The corrected sequence through candidate 08 is 0, 0, 0, 1, 0, 1, 0, 1 for candidates 01–08. A tiny maximum decrease is counted by the literal rule but is not claimed as a causal improvement on unchanged code.

The initial falsifier iteration exposed an unqualified System head in the new tests and a source-contract restriction on a new tuple variant constructor. Both were corrected in new code; no existing test or depth contract was changed. The Structural variant uses named fields and the same descent body. The temporary candidate-05 Root selector introduced one UnknownConstruct storage row; that experimental selector was removed in candidate 06. No incoming storage row was removed. All final new unknown rows remain unknown.

Measured-candidate count is 10 (00 incoming plus 01–09). Stop reason is the hard candidate cap, not an assertion that the target is reached. Candidate 09 is rejected for six sustained neighbor increases; its tiny maximum decrease is on unchanged held code and counts only as literal numerical progress. Final stop-loss streak is zero; the auxiliary accepted-nonprogress field is two and is not used to override the measured-candidate stop rule. Candidate 08 is restored exactly from its measured source snapshot, with acceptance inlining retained and completion inlining added. No additional performance sample or candidate was run after reaching the cap.

All 23 allocation count/byte slopes are identical to incoming for every measured candidate. The pre-G4 differences visible in the full rows remain; in particular, structural work records retain the existing flat-walk storage costs. The target remains unmet on eight labels: Rule Blank and Rule depth 8, both Cases pattern-child labels, and held MatchQ child, head, prepared-call and depth 8. Single-sample excursions on other rows are printed rather than discarded. Those are not counted as sustained regressions, and the strict every-sample target is still reported as unmet.

The retention certificate regeneration first fails on stale C01 fingerprints by design in each feature set, writes the replay artifact, and is then checked again in the final gates. Installer checks permit only exact fingerprint-token substitutions. The 68 local-contract fingerprint cells preserve the other 18 fields, and eight obligation rows relocate only their evidence field to identical source tokens. The complete old/new cells and raw replay outputs are retained.

Final source audit passes against the initial snapshot and the identical verification export. Scope is limited to the two walkers, one completion attribute in definitions.rs, two appended private test-module declarations, their two new base-case files, ownership position/new-unknown replay, and retention replay. Existing parent test files retain their incoming bytes; each new base-case test file is under 5k characters. No other evaluator, matcher, normalizer, Expr, dependency, release profile or frame-list code changed.


Ownership reconciliation preserves every incoming identity, status and disposition; no incoming row vanished, no new UnknownHostPayload/self-reaching row appeared, and no new row belongs to untouched code.

| Inventory | Before | After | Position moves | New UnknownConstruct | Removed |
| --- | ---: | ---: | ---: | ---: | ---: |
| host_payloads.tsv | 63 | 63 | 0 | 0 | 0 |
| unknown_attributes.tsv | 1465 | 1465 | 0 | 0 | 0 |
| unknown_captures.tsv | 2944 | 2944 | 4 | 0 | 0 |
| unknown_macros.tsv | 3139 | 3140 | 31 | 1 | 0 |
| unknown_storage.tsv | 88 | 88 | 0 | 0 | 0 |

New unknown rows, verbatim:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs	37	20	unexpanded macro matches! (ends at 40:18)
```

Every ownership position move (incoming inventory row number):

| Inventory row | Source | Before | After |
| --- | --- | --- | --- |
| unknown_captures.tsv:403 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 438:78 | 439:78 |
| unknown_captures.tsv:404 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 505:35 | 506:35 |
| unknown_captures.tsv:405 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 534:18 | 535:18 |
| unknown_captures.tsv:854 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 210:22 | 232:22 |
| unknown_macros.tsv:287 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 117:62 | 118:62 |
| unknown_macros.tsv:288 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 120:19 | 121:19 |
| unknown_macros.tsv:289 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 160:12 | 161:12 |
| unknown_macros.tsv:290 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 168:13 | 169:13 |
| unknown_macros.tsv:291 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 177:44 | 178:44 |
| unknown_macros.tsv:292 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 358:16 | 359:16 |
| unknown_macros.tsv:293 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 385:52 | 386:52 |
| unknown_macros.tsv:294 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 400:52 | 401:52 |
| unknown_macros.tsv:295 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 461:17 | 462:17 |
| unknown_macros.tsv:296 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 490:5 | 491:5 |
| unknown_macros.tsv:297 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 494:5 | 495:5 |
| unknown_macros.tsv:298 | `crates/rum-runtime/src/runtime/definitions/definitions.rs` | 498:5 | 499:5 |
| unknown_macros.tsv:312 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 50:16 | 67:16 |
| unknown_macros.tsv:313 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 51:20 | 68:20 |
| unknown_macros.tsv:314 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 52:21 | 69:21 |
| unknown_macros.tsv:315 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 67:17 | 84:17 |
| unknown_macros.tsv:316 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 78:13 | 95:13 |
| unknown_macros.tsv:317 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 91:16 | 108:16 |
| unknown_macros.tsv:318 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 95:16 | 112:16 |
| unknown_macros.tsv:319 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 117:9 | 138:9 |
| unknown_macros.tsv:320 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 184:12 | 205:12 |
| unknown_macros.tsv:321 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 192:41 | 213:41 |
| unknown_macros.tsv:322 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 195:17 | 216:17 |
| unknown_macros.tsv:323 | `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | 198:20 | 219:20 |
| unknown_macros.tsv:677 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 88:16 | 109:16 |
| unknown_macros.tsv:678 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 125:9 | 146:9 |
| unknown_macros.tsv:679 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 135:8 | 156:8 |
| unknown_macros.tsv:680 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 143:44 | 164:44 |
| unknown_macros.tsv:681 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 146:13 | 167:13 |
| unknown_macros.tsv:682 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 172:8 | 194:8 |
| unknown_macros.tsv:683 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 189:24 | 211:24 |

Every replayed fingerprint cell (other 18 fields unchanged):

| Row / ID | Before | After |
| --- | --- | --- |
| 2 / B01 | `fnv1a64:fbfb5a1ca428d593` | `fnv1a64:7f5a6fa4e74215d2` |
| 3 / B02 | `fnv1a64:cc1a40bed8415d76` | `fnv1a64:f68abe2b45f174cf` |
| 4 / B03 | `fnv1a64:ca985f53f30395d5` | `fnv1a64:7b32a3066ea58cd4` |
| 5 / B04 | `fnv1a64:0bf46fd10c1d447b` | `fnv1a64:7a814d33ba28613a` |
| 6 / B05 | `fnv1a64:1726a7f33e990cfa` | `fnv1a64:349b2b1cc87c8593` |
| 7 / B06 | `fnv1a64:d2ea7f5bb8322dcf` | `fnv1a64:8017511c808a44ce` |
| 8 / B07 | `fnv1a64:4f063359caaa428a` | `fnv1a64:7ee5dce047873923` |
| 9 / B08 | `fnv1a64:8ce07595dbc5c0e9` | `fnv1a64:bdcecc9fccf6e478` |
| 10 / B09 | `fnv1a64:5b45e5f66e4b2f37` | `fnv1a64:4f761b0c940fced6` |
| 11 / B10 | `fnv1a64:2c7e19dfa2ede670` | `fnv1a64:d19ac76157ed7281` |
| 12 / B11 | `fnv1a64:cd4c212e26116a7b` | `fnv1a64:3bd8fe90d41c873a` |
| 13 / B12 | `fnv1a64:1a5265f24a53c437` | `fnv1a64:0e829b08701863d6` |
| 14 / I01 | `fnv1a64:d9dd451a10e0cc13` | `fnv1a64:5d3c5aa253fa0c52` |
| 15 / I02 | `fnv1a64:6e68258736cf94fa` | `fnv1a64:8bdca8b0c0b30d93` |
| 16 / I03 | `fnv1a64:407f6d1f3ce748f7` | `fnv1a64:38b16abaa2a41596` |
| 17 / I04 | `fnv1a64:80ed0f00b3571b96` | `fnv1a64:d2dd98b7ff3d89ef` |
| 18 / P01 | `fnv1a64:32343ffc1241886f` | `fnv1a64:4529a9d81a87f36e` |
| 19 / P02 | `fnv1a64:5f0638f3473c9a20` | `fnv1a64:45b4dac5d567b3f1` |
| 20 / P03 | `fnv1a64:b261336daa07de6d` | `fnv1a64:15be61d159e9f1ac` |
| 21 / P04 | `fnv1a64:c0f55ed45cea7976` | `fnv1a64:eb65dc40ca9a90cf` |
| 22 / P05 | `fnv1a64:e71d1e8a3dcbba03` | `fnv1a64:1aa3b26d6c10ec42` |
| 23 / P06 | `fnv1a64:6ad18ac2d6835db7` | `fnv1a64:5f01bfd8fc47fd56` |
| 24 / P070 | `fnv1a64:a51f2704b4f8d7b7` | `fnv1a64:994f5c1adabd7756` |
| 25 / P080 | `fnv1a64:711b7946fbf95a30` | `fnv1a64:45ddd5502a8d9e41` |
| 26 / P090 | `fnv1a64:d75c65721d20e836` | `fnv1a64:c067bfb4a974158f` |
| 27 / P100 | `fnv1a64:eca95d89a54f588b` | `fnv1a64:62c4b05704b710ca` |
| 28 / P110 | `fnv1a64:56b3442875b44b28` | `fnv1a64:b0f43dfc87ccdd39` |
| 29 / P120 | `fnv1a64:1d8eaddf43c18109` | `fnv1a64:230a671d054b9118` |
| 30 / P130 | `fnv1a64:f2d38cb6f95e1c8a` | `fnv1a64:22b3363d763b1323` |
| 31 / P140 | `fnv1a64:bc2337b209c42bce` | `fnv1a64:67fa247b39575cc7` |
| 32 / P150 | `fnv1a64:2fdfbf69e726ac56` | `fnv1a64:85c9ce445b919faf` |
| 33 / P071 | `fnv1a64:afbd9459a0364c2a` | `fnv1a64:9724fe996442e043` |
| 34 / P081 | `fnv1a64:f21b0ec1b2717ed7` | `fnv1a64:6053eebf0d601676` |
| 35 / P091 | `fnv1a64:6cdf8ae79b55062f` | `fnv1a64:e81a052200f30b2e` |
| 36 / P101 | `fnv1a64:4eb634f5ab17e828` | `fnv1a64:a8f72ec9bd307a39` |
| 37 / P111 | `fnv1a64:a354a488fa0fdbc7` | `fnv1a64:5b4054b636f226e6` |
| 38 / P121 | `fnv1a64:bab6956209b7a72c` | `fnv1a64:7b8e513ae0d26fdd` |
| 39 / P131 | `fnv1a64:a99e9237427165fb` | `fnv1a64:182b6f99f07c82ba` |
| 40 / P141 | `fnv1a64:7fb6365d1f09378f` | `fnv1a64:90a72bd96d9a558e` |
| 41 / P151 | `fnv1a64:8c6c9bb9a4a2f649` | `fnv1a64:04a896a1fc65b958` |
| 42 / B13 | `fnv1a64:60fb12e4e8cee9cf` | `fnv1a64:0e27e4a5b12700ce` |
| 43 / R01 | `fnv1a64:02363dc725fdc686` | `fnv1a64:4261bf8b2240405f` |
| 44 / R02 | `fnv1a64:f7976dbbf476052d` | `fnv1a64:380a3b5ed586de6c` |
| 45 / R03 | `fnv1a64:7e8f89c1623d40b2` | `fnv1a64:546e6a4b34ebf00b` |
| 46 / R040 | `fnv1a64:c1d35cfd326182a6` | `fnv1a64:a95fa6d2d97e29ff` |
| 47 / R050 | `fnv1a64:b1c0a7500018db63` | `fnv1a64:4974f43229f793a2` |
| 48 / R060 | `fnv1a64:a25c791d5604f7de` | `fnv1a64:e3bcb744cb30ad57` |
| 49 / R070 | `fnv1a64:5a88d6db94133dd7` | `fnv1a64:c8c1b6d8ef01d576` |
| 50 / R080 | `fnv1a64:47f177f382dbe496` | `fnv1a64:99e201aacec252ef` |
| 51 / R090 | `fnv1a64:a9b8315766f8324b` | `fnv1a64:6debb1b33b0caa8a` |
| 52 / R100 | `fnv1a64:8fbe03a69cd94055` | `fnv1a64:40584759187b3754` |
| 53 / R110 | `fnv1a64:07235466af3fffb8` | `fnv1a64:cb53fd78331f6249` |
| 54 / R041 | `fnv1a64:4734760efff6a8cf` | `fnv1a64:f46147cfc84ebfce` |
| 55 / R051 | `fnv1a64:657e9387aa4ef4b6` | `fnv1a64:4e89edca36a2220f` |
| 56 / R061 | `fnv1a64:db1622acba394ad9` | `fnv1a64:84875bdf7d8b1ca8` |
| 57 / R071 | `fnv1a64:ceb91a602b8be6d4` | `fnv1a64:f8cf3a3f6304dfe5` |
| 58 / R081 | `fnv1a64:a893c6de40fb2d8b` | `fnv1a64:1eaf19aba062e5ca` |
| 59 / R091 | `fnv1a64:56a4c873fbfd000e` | `fnv1a64:869c8fec97c8ff07` |
| 60 / R101 | `fnv1a64:c570863c36e37e9e` | `fnv1a64:d72a15dc3278c917` |
| 61 / R111 | `fnv1a64:4bd1df2a552db1f1` | `fnv1a64:8c7e5f9e09a93040` |
| 62 / R12 | `fnv1a64:f5cf92fc681d9a31` | `fnv1a64:e138739d94db1280` |
| 63 / R13 | `fnv1a64:984d126ed4749245` | `fnv1a64:ef8ff88dc2dc93c4` |
| 64 / R14 | `fnv1a64:07016bff8200188b` | `fnv1a64:7d1cbecce167d0ca` |
| 65 / R15 | `fnv1a64:80bfe4a3b9d430ff` | `fnv1a64:166dda68b23f617e` |
| 66 / O010 | `fnv1a64:9e57539145a2b439` | `fnv1a64:81604fb677cfb888` |
| 67 / O0200 | `fnv1a64:b6ead986587b51ce` | `fnv1a64:62c1c64f880e82c7` |
| 68 / O011 | `fnv1a64:e4e535a56396bc5e` | `fnv1a64:264573ccd8c271d7` |
| 69 / O0210 | `fnv1a64:bb900f8d02e50511` | `fnv1a64:be5c1cf18d131c60` |

Exact certificate cells (only fingerprint tokens changed; full old/new values in the named JSON files):

| Row / ID | Feature | Column | Evidence |
| --- | --- | ---: | --- |
| 2 / C01 | hash=true | 5 | `certificate-cell-changes-true.json` |
| 3 / C02 | hash=true | 5 | `certificate-cell-changes-true.json` |
| 2 / C01 | hash=false | 6 | `certificate-cell-changes-false.json` |
| 3 / C02 | hash=false | 6 | `certificate-cell-changes-false.json` |

Every obligation evidence position cell (other fields unchanged):

| Row / ID | Before evidence | After evidence |
| --- | --- | --- |
| 57 / N:Count | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:91; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 66 / N:DeleteCases | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:91; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 103 / N:FreeQ | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:91; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 147 / N:MatchQ | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:91; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 151 / N:MemberQ | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:69; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:91; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57` |
| 177 / N:NormalizeRules | `crates/rum-runtime/src/runtime/definitions/definitions.rs:146; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:200; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:248` | `crates/rum-runtime/src/runtime/definitions/definitions.rs:147; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:200; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:248` |
| 227 / N:Rule | `crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:111; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13` | `crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:128; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13` |
| 228 / N:RuleDelayed | `crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:111; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13` | `crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:128; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13` |

Final calibrated attribution (residual native rows are in the retained candidate’s complete panel above):

```tsv
label	base_total	candidate-08_total	base_rule_self	candidate-08_rule_self	candidate-08_continuation_self
g4-rule-blank-constraint	16795.5	17946.3	1227.0	1783.0	0
g4-rule-pattern-child	22396.8	21457.7	2164.0	2709.0	0
g4-ruledelayed-pattern-child	22021.7	20789.7	2084.0	2629.0	0
g4-rule-wrapper	26107.2	23386.7	3550.0	4031.0	0
g4-rule-skew-depth-8	59905.0	61126.3	11116.0	11979.0	0
g4-rule-skew-depth-32	217325.5	207317.2	80500.0	67315.0	0
matchq-integer-success	4943.4	4562.7	401.0	289.0	0
matchq-integer-fail	5095.4	5156.5	400.0	288.0	0
cases-pattern-child	39793.9	40274.4	1795.0	2232.0	0
g4-matchq-pattern-child	21782.4	22942.8	1706.0	2143.0	0
g4-matchq-pattern-head	21186.8	21755.1	1774.0	2170.0	0
g4-cases-pattern-child	39745.6	40169.7	1795.0	2232.0	0
g4-matchq-prepared-call	15224.1	15897.1	2742.0	3116.0	0
g4-matchq-skew-depth-8	52396.3	54561.2	7095.0	8074.0	0
```

All self-reaching rows, host payload rows and closed runtime frame lists are unchanged. New unknowns remain unknown. This is attribution of the present flat implementation’s cost, not a proof that further reductions are impossible.

