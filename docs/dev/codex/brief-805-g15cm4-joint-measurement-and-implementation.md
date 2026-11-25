# Brief 805 — G15-CM4 D.2: measure the joint J1/J2 package, land it, and verify (implementation)

Implementation brief. Scope: the seven live source paths named below and `.temp/brief-805/` only.
No render, no install, no re-freeze, no epoch, no tag, no network, no new dependency, no new
`unsafe` in `rum-core`/`rum-runtime`, no git command. All scratch under `.temp/brief-805/`; the live
source is edited only in Stage D.

## Why this brief

Brief 804 sealed DONE and ruling 837 accepted it. The D.1 plan fixed the repair package on the
closure-corrected ledger: seven outline attributes (**1034 13,973,401; 1038 297,745; 1276 199,557;
1083 39,600; 1184 190,434; 1185 76,786; 1110 73,624** — 14,851,147 gross, never summed) plus the
coldout refactor's differential (**3,735,504**; armed 22,584,060 − real 18,848,556). Three prices
are withdrawn as re-billing (1383 +376,710, 1086 +45,220, 1199 +18,347), three are refused on their
publication choice (1618/1263/1014-always), and the fastpath (41,712 inside 51,790) is a separate
default-path decision. The joint J1/J2 build is the only admissible price; the nominal 18,586,651 is
a preregistered comparison target and is never booked. This brief lands exactly the measured
package in the live tree; the render, install, re-freeze and epoch are later, separately authorized
briefs.

## Goal

1. Measure **J1** (the seven attributes) and **J2** (J1 + the coldout refactor) on the **armed** and
   **real** roles at one path, with a same-path **J0 control per role**, and publish the definitive
   differential `(armed J0 − armed J2) − (real J0 − real J2)` on the deduplicated
   closure/evaluator union.
2. Land exactly the measured J2 in the seven live source paths, and verify the semantic,
   publication, standing and workspace gates on the formatted live sources.
3. Seal the receipt that briefs 806 and 807 consume: source identity, the measured package, the
   per-row reconciliation and the standing-producer verdict lines.

## Context and pins

- Four sealed role roots, read-only: baseline `ec602109…`/5,842; real `022d60af…`/5,851; disabled
  `c7156d50…`/5,851; armed `318933db…`/5,851 (`.temp/brief-804/evidence/entry-index.json`). Start
  every variant from a copy of its authenticated role root at one absolute staging path and one
  target path.
- Sealed evidence to reuse, never rewrite: `.temp/brief-803/report.json` (`67cae6b5…`) and its
  `evidence/` (`calibration-803.json` `359945f5…`, `calibration-803.tsv`, `evaluator.tsv`
  `4e80b930…`, `outline.tsv` `c6e9e996…`, `group-1186.tsv` `322cdea8…`, `battery-verification.json`,
  `classification-ledger.tsv` `64794aee…`, `class-form.tsv` `ae7b9c7a…`); `.temp/brief-804/report.json`
  (`4fcbbae9…`) with `evidence/price-ledger.tsv`, `row-mapping.tsv`, `family-mapping.tsv`,
  `residual-arithmetic.json`, `implementation-sites.json` (`1bc72299…`), `cell-accounting.json`
  (`0561873d…`); the J0 binding captures `.temp/brief-800/callgrind/whole-new…` (`431a8f23…`) and
  `whole-base…` (`34bea875…`); the seven outline diffs and the coldout diff (pins below); the LP01
  inventory `.temp/brief-797/amendment6/images/{real,armed}/crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-required.tsv`
  (1,115 rows, `d60dcf9a…`, eight producer paths, zero overlap with the seven package paths); the R4
  map `.temp/brief-797/amendment6/handoff/r4-paths.tsv` (`8dbc7792…`, 190 paths; the package's three
  in-map files are `definition_updates.rs`, `value_table_assignment.rs`, `association_data.rs`; the
  four outside are `definition_analysis_helpers.rs`, `guarded_plan.rs`, `profiling/config.rs`,
  `profiling/timing.rs`).
- The seven live declarations were read at the gate and each live file is byte-equal to its armed
  image (the three overlay files' real images carry the declared one-constant reverts). Pre-edit
  live sha256: `definition_analysis_helpers.rs` `6fa6a832…`, `guarded_plan.rs` `8c2de9f2…`,
  `definition_updates.rs` `033d0d97…`, `association_data.rs` `ae7783b8…`,
  `value_table_assignment.rs` `1bf24494…`, `profiling/timing.rs` `aa94f1d5…`,
  `profiling/config.rs` `4fff5fdf…`. A pre-edit mismatch is a STOP.
- The build recipe is the campaign's pinned one: offline, locked, rustc 1.85.0, codegen-units=1,
  one codegen unit, profile off, the same launcher/cwd/stdin, CPU 14, the canonical 120-label
  workload (normalized `c193d87f337d36a1f4d194e216adb8b33f7f58e0147582d9e2ace9434d498bed`, 14
  zero `RUM_PROFILE_*` flags) and the eight N=50/100 short inputs. The 803 preparation recipes and
  instruments are the reference (`.temp/brief-803/tools/battery.py`, `metrics.py`, `cal801.py`).

## The exact source shape (Stage D applies this; the diffs are the measured forms)

One `#[inline(always)]` on its own line immediately before each declaration, nothing else:

| Cell | Live declaration | Retained diff |
| --- | --- | --- |
| 1034 | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:254` `collect_upvalue_candidate_symbols` | `.temp/brief-802/evidence/outline-1034-always.diff` |
| 1038 | `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:485` `normalize_rewrite_rules` | `.temp/brief-802/evidence/outline-1038-always.diff` |
| 1276 | `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded_plan.rs:4` `build_guarded_replace_plan_from_targets` | `.temp/brief-802/evidence/outline-1276-always.diff` |
| 1083 | `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:382` `current_symbol_held_update_value` | `.temp/brief-802/evidence/outline-1083-always.diff` |
| 1184 | `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:846` `normalize_association_set_lvalue_target` | `.temp/brief-802/evidence/outline-1184-always.diff` |
| 1185 | `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:878` `resume_association_set_lvalue_selector_evaluation` | `.temp/brief-803/evidence/outline-1185-always.diff` |
| 1110 | `crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs:109` `normalize_value_table_target_expr` | `.temp/brief-803/evidence/outline-1110-always.diff` |

The coldout refactor is exactly `.temp/brief-803/evidence/coldout.diff` (byte-identical to
`.temp/brief-802/evidence/coldout.diff`, `dec072a6…`): in `profiling/timing.rs` the two adopted
functions call `run_configured_timing(|| …)`, and the new `#[inline] fn run_configured_timing` +
`#[cold] #[inline(never)] fn run_configured_timing_cold` preserve the relaxed enabled-byte load,
the immediate `T::default()` for value 2, the cold non-inlined initializer/active check, and the
test/configuration lease acquisition and drop before the operation; in `profiling/config.rs:569`
`initialize_lexical_profile_enabled` becomes `pub(super)`. Phase, result, error, event ordering and
call counts are unchanged. No other source edit is permitted. The refused, withdrawn, null and
unpriced variants (1199, 1383, 1086, 1618, 1263, 1014), foldaddr, hoist and the fastpath are not
implemented.

## Stage A — entry pins and J0 controls (500 s)

Hash the scene, environment, preregistration and stage clock before inspecting new output; copy the
four role bindings; check the fourteen `RUM_PROFILE_*` flags are 0 and the ordinary settings match.
For each role, build **J0** (the unchanged role tree) at the staging path and take one whole
canonical Callgrind capture. Match the J0 structural currencies against the sealed 803 binding
exactly: the staged tree membership, the measured symbol ranges (the 136-byte / 24-symbol mimalloc
stamp excepted), the flag PC/flag/destination table, the four short slopes (armed 22/22/58/26, real
3/3/5/5), row 1056's non-gate per-visit, the publication census/gates, and the canonical 120
labels. Whole-panel totals are data, not gates. A missing or differing pin, or an unbound capture,
is a STOP. Emit `.temp/brief-805/evidence/entry-index.json`.

## Stage B — the four J builds (4,200 s)

Visit **armed J1, armed J2, real J1, real J2** at the same path, starting each from the
authenticated role copy and applying the package by source identity. Save the source membership,
diffs, ELF, section bytes and symbol ranges before replacing the staged variant. Run exactly one
whole canonical Callgrind capture per build, plus the eight N=50/100 short inputs. Save every raw
stream, resolution, argv, CPU, marker, raw hash and the caller/PC tables before parsing. J0
captures come from Stage A; reusing a sealed J0 binding instead of a fresh control capture is a
STOP. Keep whole-panel totals informational. Emit `.temp/brief-805/evidence/builds.tsv`.

## Stage C — the joint decision and batteries (2,600 s)

**J1 on armed.** Rebind every listed helper and caller by object, symbol and context. Count raw
self, subtract exclusively owned publication PCs, and measure the union once. For every outside
callee compare its incoming call count **from the measured set** across the two roles of the
comparison, ignoring call-site address relocation, and add the whole non-publication self delta of
each callee whose count changed, once; preserve a zero-self callee whose edge changed. The seven
row-local closures must equal their single-cell closures exactly (row-local band 0). Caller 1186 is
shared by 1184 and 1185 and is counted **once**: its joint-growth reference is **+469,690**
(+325,676 + 144,014) and the 1184/1185/1186 group-local reference is **−267,220**. Row 1199 is a
neighbour inside the caller's subtree, not a member of any clean closure: track it separately, with
the binding departure bound **2,824**, and allow the expanded group/union figure that includes it to
vary by at most 2,824, exactly once. **Exact J1 union falsifier: −14,851,147.** A total within a band
cannot rescue a failed individual closure.

**J1 on real.** The seven declarations must produce **zero** saving on the matching attributes,
their parents and their changed-edge closure (all seven standalone bodies have zero real visits; the
real build has no separate helper symbols). A nonzero real-side saving is a STOP.

**J2 increment and the definitive price.** On rows 1115/1116 and their caller contexts 1117/1118
plus any executed cold outline, compare J1 to J2 independently in both roles. Expected: armed
**22,584,060** (tolerance 20,120), real **18,848,556** (31,670), differential **3,735,504** (51,790).
Check that J2 retains the J1 outline currency and that J1 introduced no separate evaluator drift
outside its role band. On each role form the **single union** of the seven closures and the
evaluator union at J0 and J2, with changed outside rows deduplicated, and publish the raw
before/after counts and the price `(armed J0 − armed J2) − (real J0 − real J2)` on that currency.
The nominal **18,586,651** is a preregistered comparison target only. Any new cross-cell edge is
explicitly attributed; a per-cell or joint failure stops with the cells, caller, callee, counts and
displacement named. Isolate a shared-1186 failure by the 1184-only and 1185-only footprints; a
novel overlap is proposed leave-one-out under a separately authorized split. No subtraction of a
noisy whole-panel total can rescue a failed closure. `additive_claimed=false`.

**PC currencies and batteries.** Deliver new PC/flag/destination rows and whole, N50 and N100
multiplicities, including non-executed ACTIVE PCs, normalized stdout/stderr, and the absence of
calls to the cold helper with profiling disabled. Compare the armed and real flag-class J2
increments to **−1,680,268** and **+6** within their own 616 and 135 binding bands; the evaluator
union decides the price. Slopes: armed J1 **22/22/58/26**, armed J2 **20/20/52/24**, real **3/3/5/5**
on both; check per-source visits as well as totals. Compare eager/deferred profiling event phases,
snapshot counts/lifecycle, all functional outputs and the disabled lease path. Rebind all **38**
reached publication gates to the same source-owned ordinary choices and their order: **37**
independent four-instruction gates plus **one** proven three-instruction reuse, no slow-path self on
the ordinary arm, no missing, duplicated, dropped or migrated choice; recensus unreached copies;
forced slow path, typed 4+3, 350/700 typed entries, both owner tests
(`publication_balances_each_independent_owner`, `publication_preserves_control_scope`),
`route_a_positive_and_all_falsifiers`, the exact ordered **26-line, 1,741-byte** driven ledger
SHA-256 `a2c48c90fab83d49ffceb07e8c74992268fea353cf5e03574f8cbe63435d3895`, P0256, the 12 routes,
the two mid-request callback edges (two hits each), resume to 42, owner visibility
before/during/after, and the queued request's order, result and closed scopes. An exact PC cost
never excuses semantic drift. Archive the verdict lines. Emit
`.temp/brief-805/evidence/joint-decision.tsv` and `battery.json`.

## Stage D — implementation and acceptance (8,400 s)

Land exactly the measured J2 package in the seven live paths above (the seven attribute insertions
and the coldout diff), then:

1. `cargo fmt --check` clean; the formatted sources must compile and their semantic declarations
   must match the accepted staged J2 sources; a release-byte comparison of the staged J2 build vs
   the live build at one path is the identity receipt.
2. Replay every source-covered standing producer on the final formatted sources **before** the
   first acceptance selector (the source-neutrality deferral of 803 ends here): F13 local fingerprints
   and contract records with falsifiers, class discovery and source authentication, the ownership
   discovery/positions and the 152-member chain, LP01 (1,115 rows, statement identity, columns
   17/19, binding column 3, the loss record), typed-slot receipts, the rejection census, the
   promotion reviews/certificates in both configurations, overlays/additions/mutants through their
   producers. Quote the actual verdict lines; a red caused solely by a stale fingerprint,
   certificate or position is an ordering artifact — replay the producer, rerun the affected suite,
   and preserve both receipts. A changed identity, host payload, exclusion or semantics stops.
3. Bounded `cargo fmt --check`, focused runtime/stdlib/suspension tests, the full
   `cargo test --workspace` at `RUST_TEST_THREADS=8 --no-fail-fast` inside `memory-scope`, and the
   role-required no-default/flag-on gates. Compare per binary and per name against the pinned
   per-binary gate; preserve only the pinned gate's named exclusions and the 71 ignores; a new
   failure or an unexpected pass returns to the director. Rerun any sanctioned host-only loopback
   lane with the same binary under director control and keep both receipts.
4. Microbench: base vs candidate `scripts/core-language-microbench.sh`, per label and same-tier
   neighbour, serially, in-kernel `AbsoluteTiming`; record the block and stop on a new regression.
5. Report every affected position, fingerprint, certificate and generated cell; the derived
   inventories are adopted atomically and re-read.

## Stage F — seal (300 s)

Rehash the parents, the live delta and the role roots; verify the package and capture manifests;
seal the before/after source plus generated inventory receipts. Run
`python3 -I scripts/report.py check 805`; the report and its detached summary must agree; the exit
audit must find zero unexplained live changes and zero sealed differences. Print the summary lines
the report carries verbatim.

## STOP rules

- A missing or differing entry pin, or a J0 control that fails any structural currency, is a STOP.
- A cell closure outside its bar (row-local 0; the J1 union −14,851,147; row 1199's 2,824 once), a
  nonzero real-side J1 saving, a J2 increment outside 20,120 / 31,670 / 51,790, or a new cross-cell
  edge is a STOP naming the exact cells, caller, callee, counts and deltas.
- A source edit beyond the seven live paths, a publication choice moved/dropped/reordered, or a
  semantic difference in the coldout refactor is a STOP.
- Row 1056 stays `cannot_settle`; the fastpath, the withdrawn, refused, null and unpriced variants
  are not implemented.
- A missing producer, a changed identity/owner/exclusion, or an unaccounted file stops the
  acceptance.
- `additive_claimed=false` is binding: a sum of prices in place of the joint measurement is a STOP.

## Pitfalls (the campaign's lessons)

- The closure is the price: inlining moves work between rows and the rows that appear or disappear
  are exact (the changed-edge rows' same-code spreads are 0 for the iterator/map/tree/proof/key
  rows; the allocator/deallocator/slice-drop/is_system_name rows carry spreads 1,026/1,026/3,174/644
  and their closure bands 614/3,160/2,852 — the withdrawn costs exceed them).
- `#[inline]` can remove a symbol and un-inline its callees (1199); only the seven measured
  `#[inline(always)]` insertions and the retained coldout diff are the package.
- Whole-panel instruction counts are not deterministic (14 armed draws spread 17,333,653; 3 real
  draws 3,842,503); the per-row currencies decide.
- The mimalloc compilation stamp makes whole-`.text` identity impossible; compare measured ranges.
- The flag class is PC-based: a relocated build needs its own table.
- The 1186 group is counted once; row 1199's draw is bounded at 2,824; the group net carries it.
- The derived-inventory ordering rule: replay the standing producers before the first acceptance
  suite; a stale derived cell red is an ordering artifact, not a regression.
- Every run is serial and alone; retain every raw before parsing; print only numbers you can point
  to in a retained file.

## Charge and hard rules

- Stages: A 500 / B 4,200 / C 2,600 / D 8,400 / F 300 = **16,000 s**, funded by brief 804's
  remaining 2,199.226 s plus an explicit **13,800.774 s** new allocation (「预算不够就追加吧」).
  Every stage inside its box; a box that would overrun stops at the deadline with the exact missing
  measurement.
- Q25's thirteen named STOPs stand (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387,
  P2409, P2411, P2416, P2418); 744/745/746 held; CM5 unauthorized; no tag; no network; no new
  dependency; no new `unsafe` in `rum-core`/`rum-runtime`. `.temp` only, never `/tmp`. Instruction
  counts only; no render, install, re-freeze or epoch. No acceptance suite in the render sense; the
  workspace gate above is the acceptance.
- After the seal: `python3 -I scripts/report.py check 805` must be OK, the detached summary equal to
  the report's, and the report must carry the verbatim printed lines and every evidence path.
