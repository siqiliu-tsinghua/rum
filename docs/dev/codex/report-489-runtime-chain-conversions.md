# brief-489 — runtime chain conversions

**STOP: the instruction-count gate fails on all three ordinary-evaluation workloads.**
All sixteen runtime conversions are present and the mandatory workspace test and
format check pass, but this candidate does **not** satisfy the no-default-path-cost
hard rule. The measured instruction increases are +0.545%, +1.036%, and +0.741%,
well outside the within-arm spreads. Allocation counters are identical.
No performance workaround or out-of-scope helper change was attempted.

The entry tree was clean at `da5bfd864007e85e1349bb521d54ca003649b886`.
Everything remains uncommitted. Generated `.rum` workloads, build products,
regeneration tooling, and raw logs are under `target/brief-489`; `TMPDIR` was
set inside that directory for all builds/tests. No network, git mutation,
unsafe addition, crash/overflow/fatal-process probe, or production/test body
split was introduced. No node acquired `Drop`, and no suspension frame acquired
`BoundedDestruction`.

## 1. Derived member list

I selected the rows of `self_reaching.tsv` whose ownership path contains
`Option[Own]/arg0/std::sync::Arc`, then excluded `WithBindingNode`, `IndexChunk`,
and `PersistentExprAccumulatorNode`, and selected the `rum_runtime::` rows.
This is an ownership-path selection, not a name-pattern selection. The initial
inventory contains 29 family rows: three already converted, sixteen remaining
runtime members, and ten remaining stdlib members.

The derived runtime list, in inventory order, is exactly the director's list:

1. `PreparedArgumentNode`
2. `ReapRenderNode`
3. `TestReportResultNode`
4. `ResolvedArgumentNode`
5. `DefinitionLhsParentNode`
6. `PreparedValueTableRuleNode`
7. `UnevaluatedRestoreNode`
8. `AssociationSelectorNode`
9. `PersistentRewriteExprNode`
10. `PersistentRewriteQueueNode`
11. `GuardedTargetChunk`
12. `GuardedTargetUpdateNode`
13. `PersistentReplacePartTargetNode`
14. `PersistentMatchResultsNode`
15. `FixedPointHistoryNode`
16. `PersistentPositionNode`

Verbatim derivation summary:

```text
Arc-link family rows: 29; remaining runtime: 16; remaining stdlib: 10
Cross-check missing: []; unexpected: []
```

## 2. Changes per member

Paths in the next two tables are relative to `crates/rum-runtime/src/runtime/`;
line ranges refer to the final candidate. Each node derives `AccumulatorChainNode`,
marks its owning link with `#[accumulator_link]`, and uses `AccumulatorChain`
for both the node link and the holder link. Constructors wrap their existing
`Arc::new` without adding allocations or clones; consuming loops extract links
with `into_inner`. Existing unique/shared completion policies remain intact.
That source-level preservation did not prevent the measured instruction cost.

| Member | Module and final line ranges | Why |
| --- | --- | --- |
| PreparedArgumentNode | `call_preparation.rs:2–4,42–114` | Drain prepared-argument suffixes while preserving both unique and shared vector completion. |
| ReapRenderNode | `debug_tools/reap_state.rs:2–64` | Drain abandoned rendered-bucket/group suffixes. |
| TestReportResultNode | `debug_tools/test_report/report.rs:7–9,33–128` | Drain retained report results without changing report continuation logic. |
| ResolvedArgumentNode | `definitions/definition_catalog/arguments.rs:2–57` | Drain suspended argument-resolution suffixes and retain ordered completion. |
| DefinitionLhsParentNode | `definitions/definition_lhs_preparation/resume_frames.rs:2,45–70` | Drain the parent stack; `pop` still clones the same link and context. |
| PreparedValueTableRuleNode | `definitions/value_table_assignment.rs:6–8,34–86` | Drain prepared transactional rules; preserve move-when-unique/clone-when-shared completion. |
| UnevaluatedRestoreNode | `eval_call_prepare.rs:5–7,21–87` | Drain saved wrapper entries without changing restored positions or suspension. |
| AssociationSelectorNode | `kernel_data/association_data.rs:5–7,584–644` | Drain evaluated selector suffixes and retain selector ordering. |
| PersistentRewriteExprNode | `kernel_data/rewrite_data/suspended_state.rs:2–4,103–171` | Drain output chunks while retaining chunk and element order. |
| PersistentRewriteQueueNode | `kernel_data/rewrite_data/suspended_state.rs:2–4,44–101` | Wrap the `next` link and queue head; preserve move-only `pop_front`. |
| GuardedTargetChunk | `kernel_data/structure_data/replace/suspended_state.rs:3–5,88–95,115–146,306–317` | Drain appended target chunks while preserving shared snapshot lookup. |
| GuardedTargetUpdateNode | `kernel_data/structure_data/replace/suspended_state.rs:103–144,254–260` | Drain both frozen-binding and later update links. |
| PersistentReplacePartTargetNode | `kernel_data/structure_data/replace/suspended_state.rs:3–5,21–86` | Drain target chunks and preserve unique payload moves during completion. |
| PersistentMatchResultsNode | `pattern_matching/suspended_state.rs:2–4,74–120` | Enforce the wrapper despite today's empty tail and absent node-growth site. |
| FixedPointHistoryNode | `resolution.rs:5–7,10–63` | Drain suspended cycle-history suffixes without changing lookup. |
| PersistentPositionNode | `tree_walkers/position.rs:5–7,10–85` | Drain position chunks and preserve unique/shared ordered completion. |

The four behavioural tests are at `definitions/value_table_assignment.rs:358–397`,
`eval_call_prepare.rs:89–120`, `kernel_data/rewrite_data/suspended_state.rs:181–202`,
and `kernel_data/structure_data/replace/suspended_state.rs:351–390` (see section 5).
The only other source-data changes are the four regenerated inventory TSVs in
section 6. No immediate call-site changes were necessary. The root-tracing
inventories remain unchanged and their workspace gates pass.

The first compile failed because the qualified derive in the LHS module still
requires the `AccumulatorChainNode` trait import in scope. Adding that import
fixed it. This was **not** a move-out failure. All existing moves from unwrapped
nodes compile unchanged apart from extraction of the wrapped link; no node
`Drop` or workaround was added.

Organization assessment: the brief prohibits restructuring. The touched larger
modules keep their existing responsibilities. Future boundaries are the prepared
argument accumulator versus call planning (`call_preparation.rs`, 14.3k chars),
report state versus report rendering (`test_report/report.rs`, 22.3k), transaction
state versus rule parsing (`value_table_assignment.rs`, 13.6k including the new
test), restore state versus argument evaluation (`eval_call_prepare.rs`, 21.7k
including its new test), selector assignment versus association representation
(`association_data.rs`, 32.8k), guarded target overlays versus unguarded chunks
(`replace/suspended_state.rs`, 13.5k including the new test), fixed-point driving
versus symbol resolution (`resolution.rs`, 16.5k), and accumulation versus traversal
(`position.rs`, 10.9k). Other touched implementation modules remain under 10k.
No TODO or out-of-scope module split was made.

## 3. Construction paths and the required A/B

**Three members build nodes on ordinary evaluation paths.** Twelve grow only
at/after suspension. The sixteenth, `PersistentMatchResultsNode`, has no growth
site at all: its holder is created at suspension with an empty link.

| Member | Construction site | Ordinary versus suspension evidence; A/B decision |
| --- | --- | --- |
| PreparedArgumentNode | `call_preparation.rs:69` | **Ordinary and resumed.** `definitions/definition_lhs_preparation.rs:552–565` freezes/appends whenever an LHS argument changes, reached from the ordinary driver at `:371–375`; structural merging also appends at `:713–721`. The helper's name is not a suspension guard. A/B required. |
| ReapRenderNode | `debug_tools/reap_state.rs:34` | **Post-suspension only.** Ordinary buckets/groups use vectors; freezes occur under InputRequired at `debug_tools/reap.rs:155–159,239–243`, appends at `:171,198,255,271,279` are in suspended drivers. No A/B required. |
| TestReportResultNode | `debug_tools/test_report/report.rs:103` | **Post-suspension only.** Ordinary cases use `rendered_results: Vec`; suspended state originates in InputRequired arms at `:270–273,309–316`, and its append is `:447`. No A/B required. |
| ResolvedArgumentNode | `definitions/definition_catalog/arguments.rs:34` | **Post-suspension only.** Ordinary argument loops push vectors; InputRequired freezes at `:91–98,121–128`, resumed appends at `:140,150`. No A/B required. |
| DefinitionLhsParentNode | `definitions/definition_lhs_preparation/resume_frames.rs:59` | **Ordinary and resumed.** `definitions/definition_lhs_preparation.rs:181–194` enters the driver directly; `:362` pushes a call-argument context and `:664` pushes a structural context without requiring suspension. A/B required. |
| PreparedValueTableRuleNode | `definitions/value_table_assignment.rs:60` | **Ordinary and resumed.** Normal replacement starts preparation at `:164`; `definitions/definition_lhs_preparation.rs:410` invokes the continuation on ordinary completion, which reaches the append at `definitions/value_table_assignment.rs:204`. A/B required despite the `resume_prepared_value_table_entry` name. |
| UnevaluatedRestoreNode | `eval_call_prepare.rs:63` | **Post-suspension only.** InputRequired freezes state at `:257–270`; appended wrapper entries at `:464` occur only in `resume_suspended_call_argument_preparation` (`:401`). LHS preparation has a separate vector of restores. No A/B required. |
| AssociationSelectorNode | `kernel_data/association_data.rs:620` | **Post-suspension only.** Ordinary selector evaluation uses a vector; InputRequired freezes at `:798–807`, suspended appends are `:819,834`. No A/B required. |
| PersistentRewriteExprNode | `kernel_data/rewrite_data/suspended_state.rs:138` | **Post-suspension only.** For example, `rewrite_data/cases/collect.rs:241–250` freezes after InputRequired, and `:282,312` append from suspended traversal. The alternate frozen-spec entry is invoked by suspension frames (`cases/collect/resume_frames.rs:123,134,170`). Replace/DeleteCases/List likewise retain vector fast paths and freeze in their InputRequired arms (e.g. `rewrite_data/replace/once.rs:553–568,644–659`; `replace/list/matching.rs:291–301`). No A/B required. |
| PersistentRewriteQueueNode | `kernel_data/rewrite_data/suspended_state.rs:70` | **At suspension only.** The two production `freeze` calls are inside InputRequired arms in `kernel_data/rewrite_data/replace/list/matching.rs:218–224,291–301`, consuming remaining binding maps. The owning link is `next`. No A/B required. |
| GuardedTargetChunk | `kernel_data/structure_data/replace/suspended_state.rs:312` | **Post-suspension only.** Holder freezes are under InputRequired in `replace/guarded.rs:100,196` or the suspension helper `:278–298`; only suspended planning calls append targets (`:334,369`). No A/B required. |
| GuardedTargetUpdateNode | `kernel_data/structure_data/replace/suspended_state.rs:129,255` | **At/after suspension only.** Initial updates arise in the same guarded-target freeze; subsequent changes occur in the suspended target list, not the ordinary vector. Same entry witnesses as GuardedTargetChunk. No A/B required. |
| PersistentReplacePartTargetNode | `kernel_data/structure_data/replace/suspended_state.rs:52` | **Post-suspension only.** `replace/replace_part.rs:211–215,312–315` freezes in InputRequired arms; resumed target accumulation appends at `:403,419,438,455`. No A/B required. |
| PersistentMatchResultsNode | `pattern_matching/suspended_state.rs:88–94` (holder constructor) | **No node construction today.** `freeze` sets an empty tail; there is no `Arc::new(PersistentMatchResultsNode)` or growth method. Holder construction is suspension-only, e.g. `pattern_matching/sequence/orderless/optional.rs:328–331`, and `orderless/collect.rs:210–215,248–252`. Convert the link to enforce future safety anyway; no A/B required. |
| FixedPointHistoryNode | `resolution.rs:57` | **Post-suspension only.** Ordinary fixed-point history is a vector; InputRequired freezes it at `:353–360`, and `resume_fixed_point_from` appends at `:412`. No A/B required. |
| PersistentPositionNode | `tree_walkers/position.rs:41` | **Post-suspension only.** Ordinary positions use a vector; InputRequired freezes at `:221–231`, resumed appends at `:252,288`. No A/B required. |

Paths shortened to `rewrite_data/...` or `replace/...` in explanations refer to
the corresponding `kernel_data/rewrite_data/` or `kernel_data/structure_data/replace/`
subtree, respectively.

Both measurement arms were built and executed at the same `target/release/rum-cli`
path, with the workspace Rust 1.85.0 release profile. Profiling environment was
empty for instruction runs; no build/test ran concurrently with those runs.
Each arm has three runs per workload on CPU 2. Every `cpu_core/instructions/u`
row has multiplex coverage `100.00`. The unused `cpu_atom` row is not counted.
The brief's calibrated instruction recipe supersedes the generic timing-panel
rule here; no wall-clock timing or microbench timing was used to override it.

The workloads actually reach the changed ordinary paths:

- `prepared-arguments`: each SetDelayed prepares eight arithmetic expressions
  in the LHS, making the argument list change and creating eight argument nodes.
- `lhs-parents`: nested `g[h[x_Integer]]` and `y_List` traverse pattern and
  structural child contexts, exercising parent push/pop.
- `value-table-rules`: each assignment prepares and installs eight serialized
  rules, building and consuming the rule accumulator.

Each runs 200 ordinary iterations with small expressions and no input. These
are end-to-end workload deltas, not isolated per-node instruction measurements.
The workloads do not measure the other thirteen members' suspended/no-growth paths.

`target/brief-489/prepared-arguments.rum`:

```rum
Do[f[1+1, 2+2, 3+3, 4+4, 5+5, 6+6, 7+7, 8+8] := 1, {200}]; Print[Length[DownValues[f]]];
```

`target/brief-489/lhs-parents.rum`:

```rum
Do[f[g[h[x_Integer]], y_List] := x, {200}]; Print[Length[DownValues[f]]];
```

`target/brief-489/value-table-rules.rum`:

```rum
Do[DownValues[f] = {HoldPattern[f[1]] :> 1, HoldPattern[f[2]] :> 2, HoldPattern[f[3]] :> 3, HoldPattern[f[4]] :> 4, HoldPattern[f[5]] :> 5, HoldPattern[f[6]] :> 6, HoldPattern[f[7]] :> 7, HoldPattern[f[8]] :> 8}, {200}]; Print[Length[DownValues[f]]];
```

Verbatim normal stdout is `1` then `Null` for the first two workloads, and
`8` then `Null` for the third, identical for every base/candidate run.

| Workload | Arm | Run 1 | Run 2 | Run 3 | Mean | Spread | Spread / mean |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| prepared-arguments | base | 17,774,929 | 17,771,256 | 17,772,907 | 17,773,030.667 | 3,673 | 0.020666% |
| prepared-arguments | candidate | 17,869,662 | 17,868,968 | 17,870,818 | 17,869,816.000 | 1,850 | 0.010353% |
| lhs-parents | base | 13,705,002 | 13,703,460 | 13,702,788 | 13,703,750.000 | 2,214 | 0.016156% |
| lhs-parents | candidate | 13,846,812 | 13,844,809 | 13,845,554 | 13,845,725.000 | 2,003 | 0.014467% |
| value-table-rules | base | 18,685,575 | 18,686,905 | 18,687,259 | 18,686,579.667 | 1,684 | 0.009012% |
| value-table-rules | candidate | 18,824,655 | 18,824,232 | 18,826,284 | 18,825,057.000 | 2,052 | 0.010900% |
| Workload | Candidate minus base mean | Delta | Gate |
| --- | ---: | ---: | --- |
| prepared-arguments | +96,785.333 | +0.544563% | FAIL: exceeds both arm spreads |
| lhs-parents | +141,975.000 | +1.036030% | FAIL: exceeds both arm spreads |
| value-table-rules | +138,477.333 | +0.741052% | FAIL: exceeds both arm spreads |

Verbatim counted rows, three base then three candidate for each workload:

`prepared-arguments`:

```text
17774929,,cpu_core/instructions/u,7155829,100.00,,
17771256,,cpu_core/instructions/u,6980297,100.00,,
17772907,,cpu_core/instructions/u,7400518,100.00,,
17869662,,cpu_core/instructions/u,9295188,100.00,,
17868968,,cpu_core/instructions/u,7388532,100.00,,
17870818,,cpu_core/instructions/u,7417952,100.00,,
```

`lhs-parents`:

```text
13705002,,cpu_core/instructions/u,6896293,100.00,,
13703460,,cpu_core/instructions/u,5478410,100.00,,
13702788,,cpu_core/instructions/u,3896924,100.00,,
13846812,,cpu_core/instructions/u,6343563,100.00,,
13844809,,cpu_core/instructions/u,5716289,100.00,,
13845554,,cpu_core/instructions/u,4392390,100.00,,
```

`value-table-rules`:

```text
18685575,,cpu_core/instructions/u,3794920,100.00,,
18686905,,cpu_core/instructions/u,3189486,100.00,,
18687259,,cpu_core/instructions/u,3374332,100.00,,
18824655,,cpu_core/instructions/u,3775585,100.00,,
18824232,,cpu_core/instructions/u,3271311,100.00,,
18826284,,cpu_core/instructions/u,2955773,100.00,,
```

Allocation builds used the same release path with `--features alloc-count`;
only their runs set `RUM_PROFILE_ALLOC=1`. Each following line is verbatim and
byte-identical between base and candidate for the named workload:

`prepared-arguments`:

```text
rum-profile alloc count=11620 bytes_total=1471622 peak_live_bytes=310063
```

`lhs-parents`:

```text
rum-profile alloc count=14522 bytes_total=1727301 peak_live_bytes=311149
```

`value-table-rules`:

```text
rum-profile alloc count=10260 bytes_total=2761739 peak_live_bytes=335417
```

## 4. Verification

**Workspace: 2,458 passed, 0 failed, 57 ignored across 32 groups; exit 0.**
The runtime group has 990 passed and 3 ignored. The stdlib group has 925 passed
and 52 ignored, including the exact ownership inventory gate. Formatting,
the four new behavioural tests, and all five unchanged helper tests passed.

The sandbox **cannot bind loopback**. I used `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`
for the workspace and the focused native run. The nine native ZeroMQ tests
explicitly returned early, as shown below; libtest counts those returns as
passes. Excluding them leaves 2,449 exercised reported passes. They are separate
from the 57 ignored tests. The director must rerun native coverage without that
skip in an environment with loopback access.

The initial fmt check exited 1 for the expanded queue constructor's wrapping;
the first runtime compile exited 101 for the missing trait import. Both were
corrected within the member modules. No move-out compiler error occurred.
The following are the exact validation/measurement commands, including those
failed attempts, run from `/home/remi/dev/rum`. Raw redirected output remains
under `target/brief-489/logs`.

```bash
python3 scripts/run-bounded.py --timeout 30 -- taskset -c 2 perf stat -e instructions:u -x, -- /usr/bin/true
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --offline --locked > target/brief-489/logs/build-base.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for label in prepared-arguments lhs-parents value-table-rules; do for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/release/rum-cli --file "target/brief-489/${label}.rum" > "target/brief-489/logs/instructions-base-${label}-${run}.stdout" 2> "target/brief-489/logs/instructions-base-${label}-${run}.csv" || exit; done; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --features alloc-count --offline --locked > target/brief-489/logs/build-alloc-base.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" RUM_PROFILE_ALLOC=1 python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for label in prepared-arguments lhs-parents value-table-rules; do taskset -c 2 target/release/rum-cli --file "target/brief-489/${label}.rum" > "target/brief-489/logs/alloc-base-${label}.stdout" 2> "target/brief-489/logs/alloc-base-${label}.stderr" || exit; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check > target/brief-489/logs/fmt-initial.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime --offline --locked --no-run > target/brief-489/logs/compile-runtime.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime --offline --locked --no-run > target/brief-489/logs/compile-runtime-restored.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 180 -- rustfmt --edition 2024 --config skip_children=true crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs crates/rum-runtime/src/runtime/eval_call_prepare.rs crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime --offline --locked abandoning_ -- --nocapture > target/brief-489/logs/abandonment-tests.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --offline --locked > target/brief-489/logs/build-candidate.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for label in prepared-arguments lhs-parents value-table-rules; do for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/release/rum-cli --file "target/brief-489/${label}.rum" > "target/brief-489/logs/instructions-candidate-${label}-${run}.stdout" 2> "target/brief-489/logs/instructions-candidate-${label}-${run}.csv" || exit; done; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --features alloc-count --offline --locked > target/brief-489/logs/build-alloc-candidate.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" RUM_PROFILE_ALLOC=1 python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for label in prepared-arguments lhs-parents value-table-rules; do taskset -c 2 target/release/rum-cli --file "target/brief-489/${label}.rum" > "target/brief-489/logs/alloc-candidate-${label}.stdout" 2> "target/brief-489/logs/alloc-candidate-${label}.stderr" || exit; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture > target/brief-489/logs/discovery.log 2>&1
python3 scripts/run-bounded.py --timeout 180 -- python3 target/brief-489/regenerate-inventory.py > target/brief-489/logs/inventory-regeneration.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check > target/brief-489/logs/fmt-check.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::accumulator_chain::tests -- --nocapture --test-threads=1 > target/brief-489/logs/helper-tests.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked > target/brief-489/logs/workspace.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-489/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-jupyter --offline --locked zeromq::tests::native -- --nocapture --test-threads=1 > target/brief-489/logs/native-coverage.log 2>&1
git diff --check
```

Verbatim first compile error (the remaining trait-bound errors were consequences):

```text
error[E0405]: cannot find trait `AccumulatorChainNode` in this scope
  --> crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs:50:38
   |
50 | #[derive(rum_derive::FrameRootTrace, rum_derive::AccumulatorChainNode)]
   |                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope
   |
   = note: this error originates in the derive macro `rum_derive::AccumulatorChainNode` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider importing this trait
   |
1  + use crate::runtime::accumulator_chain::AccumulatorChainNode;
   |
```

Verbatim successful compile completion:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.78s
  Executable unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
```

Verbatim new-test output:

```text
running 4 tests
test runtime::kernel_data::rewrite_data::suspended_state::tests::abandoning_shared_rewrite_queue_preserves_remaining_binding_order ... ok
test runtime::kernel_data::structure_data::replace::suspended_state::tests::abandoning_guarded_target_branch_preserves_surviving_chunks_and_updates ... ok
test runtime::eval_call_prepare::abandonment_tests::abandoning_prepared_arguments_and_restores_skips_remaining_arguments ... ok
test runtime::definitions::value_table_assignment::tests::abandoning_prepared_value_table_keeps_old_definitions_and_skips_pending_lhs ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 989 filtered out; finished in 0.00s
```

Verbatim unchanged helper output:

```text
running 5 tests
test runtime::accumulator_chain::tests::exclusive_depth_16 ... exclusive length=16: maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::exclusive_depth_64 ... exclusive length=64: maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::live_weak_does_not_prevent_drain ... live Weak length=16: maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::shared_tail_stops_then_drains ... shared tail first prefix: 8 nodes/payloads, maximum payload drop depth=1
shared tail second prefix + tail: 24 nodes/payloads, maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::wrapper_is_transparent_and_extraction_preserves_ownership ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 988 filtered out; finished in 0.00s
```

Verbatim workspace group summaries:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 59.33s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 990 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.45s
test result: ok. 925 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 8.57s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.83s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The workspace total above is the arithmetic sum of these real libtest rows.

Verbatim native skip and result output:

```text
running 9 tests
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s
```

The three existing unused test-builtin warnings appeared during builds. The
successful final fmt command emitted no output. Exit-0 tool returns are not
replaced here with invented cargo output lines.

## 5. Behavioural tests, per member

The new tests assert cancellation/transaction or surviving-state behaviour,
not destruction depth, type names, or a hand-maintained family certificate.
The derive supplies the compile-time link proof; the unchanged shared helper
supplies the destruction-depth proof. No family coverage gate was added.
Cancellation tests use the documented no-continuation-execution/reusable-session
contract (`docs/guide/embedding.md:300–315`) and preserve definition storage
semantics. The sharing tests exercise the existing internal snapshot/owned
completion contracts documented on these state helpers.

| Member | Added test and reason, or reason for no new test |
| --- | --- |
| PreparedArgumentNode | **Yes**, `abandoning_prepared_arguments_and_restores_skips_remaining_arguments`: resume the first argument, build retained arguments, suspend again, cancel, and prove the later increment never executes. |
| ReapRenderNode | No member-specific test: existing `resume_eval_request_with_input_can_finish_reap_handler_rendering` covers multi-bucket resumed output; this link change adds no distinct cancellation side effect. Shared release is already certified. |
| TestReportResultNode | No member-specific test: existing `resume_eval_request_with_input_can_finish_test_report_input_and_sametest` covers resumed input and comparison; result-node payload release is pure, with no new per-node abandonment contract. |
| ResolvedArgumentNode | No member-specific test: existing resumed argument-resolution consumers cover ordering; its abandoned suffix has the same pure Expr payload contract as the prepared-argument case. |
| DefinitionLhsParentNode | **Yes**, `abandoning_prepared_value_table_keeps_old_definitions_and_skips_pending_lhs`: suspend in a nested typed pattern with live parent contexts and abandon without continuing the pending LHS or installing definitions. |
| PreparedValueTableRuleNode | **Yes**, the same value-table test: two earlier entries are already prepared; cancellation preserves the entire old table, installs neither earlier entry, and leaves the input-side hit count at one. This is a distinct transactional abandonment boundary. |
| UnevaluatedRestoreNode | **Yes**, the prepared-argument cancellation test includes an Unevaluated entry between the two prompts, so its restore node exists before abandonment. |
| AssociationSelectorNode | No member-specific test: ordered selectors and assignment remain covered by existing association tests; node destruction cannot perform an assignment, and this patch leaves its driver/finalizer untouched. |
| PersistentRewriteExprNode | No member-specific test: existing resumed Cases, Replace, ReplaceList, and DeleteCases tests cover output ordering and retained prefixes; pure chunk release needs no duplicate helper certificate. |
| PersistentRewriteQueueNode | **Yes**, `abandoning_shared_rewrite_queue_preserves_remaining_binding_order`: consume one binding map, abandon one owner of the remaining queue, then move all survivor maps out in exact order. This specifically exercises the `next` queue shape and owned-map completion. |
| GuardedTargetChunk | **Yes**, `abandoning_guarded_target_branch_preserves_surviving_chunks_and_updates`: discard a branch with its own appended target while the survivor retains its original length and paths. |
| GuardedTargetUpdateNode | **Yes**, the same guarded-target test: frozen bindings and the survivor's Accepted update remain intact after an abandoned branch writes Inactive. This pins independent overlay snapshots. |
| PersistentReplacePartTargetNode | No member-specific test: existing ReplacePart multi-stage resume tests cover collection and rule positions; discarded target chunks have no additional observable destructor. |
| PersistentMatchResultsNode | No test manufactures an unreachable grown chain: no production node-growth site exists today. Its conversion is enforced by the derive; existing matcher suspension tests cover its frozen prefix. |
| FixedPointHistoryNode | No member-specific test: existing `resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend` covers continued resolution; abandoning a history does not run evaluations, and cancellation is pinned at the call/LHS boundaries above. |
| PersistentPositionNode | No member-specific test: existing `position_collection_survives_multiple_matcher_suspensions` and the continuation-roundtrip test cover retained ordering; adding another pure Expr-chunk release certificate would duplicate the helper. |

Four tests were added, covering seven distinct members (PreparedArgument,
DefinitionLhsParent, PreparedValueTableRule, UnevaluatedRestore, RewriteQueue,
GuardedTargetChunk, and GuardedTargetUpdate). No stress/crash probe or
production-only/test-only implementation was added.

## 6. Inventory diff, classified

All six TSV partitions were regenerated from fresh discovery. Before writing
any partition, the regeneration script compared semantic multisets, ignoring
source-position drift, allowing only the sixteen specific derive additions,
sixteen link attributes, and sixteen ownership-wrapper path changes. It checked
that no row disappears, no category changes, and no UnknownHostPayload arrives.
This is a one-run regeneration audit, not a shipped partial family coverage gate.

Verbatim regeneration output:

```text
host_payloads.tsv: 50 -> 50 rows; changed=False
self_reaching.tsv: 57 -> 57 rows; changed=True
unknown_attributes.tsv: 1331 -> 1347 rows; changed=True
unknown_captures.tsv: 2774 -> 2774 rows; changed=True
unknown_macros.tsv: 2895 -> 2895 rows; changed=True
unknown_storage.tsv: 70 -> 70 rows; changed=False
Allowed changes: 16 derives, 16 link attributes, 16 ownership paths; class (c): none.
No added UnknownHostPayload, removed row, or reclassification. All sixteen members remain self-reaching.
```

Every content change is enumerated below. For changed derives, the previous
row had the same derive list without `AccumulatorChainNode` (qualified in the
LHS module). Each `accumulator_link` row is new. For ownership paths, the
previous row went directly from `.previous` (or `.next`) to `Option`; the new
path inserts the already-known generic wrapper. Inventory line numbers are
shown before the verbatim witness rows.

Class (a) — Attribute/derive rows:

`unknown_attributes.tsv:34` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	49	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:35` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	51	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:51` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap_state.rs	14	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:52` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap_state.rs	16	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:73` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs	39	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:74` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs	41	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:116` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs	14	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:117` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs	16	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:146` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs	50	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: AccumulatorChainNode)
```

`unknown_attributes.tsv:147` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs	52	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:179` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs	40	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:180` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs	42	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:184` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/eval_call_prepare.rs	28	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:185` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/eval_call_prepare.rs	30	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:218` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/association_data.rs	591	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:219` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/association_data.rs	593	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:429` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs	58	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:430` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs	61	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:432` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs	110	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:433` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs	112	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:470` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	28	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:471` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	30	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:472` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	89	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:473` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	91	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:475` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	104	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:476` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	106	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:683` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/pattern_matching/suspended_state.rs	80	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:684` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/pattern_matching/suspended_state.rs	82	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:739` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/resolution.rs	17	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:740` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/resolution.rs	19	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:855` — derive:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/position.rs	17	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:856` — link attribute:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/tree_walkers/position.rs	19	5	unresolved attribute/expansion accumulator_link
```

Class (b) — Ownership-path rows:

`self_reaching.tsv:4` — ownership path:

```text
rum_runtime::runtime::call_preparation::PreparedArgumentNode	crates/rum-runtime/src/runtime/call_preparation.rs	50	rum_runtime::runtime::call_preparation::PreparedArgumentNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::call_preparation::PreparedArgumentNode
```

`self_reaching.tsv:5` — ownership path:

```text
rum_runtime::runtime::debug_tools::reap_state::ReapRenderNode	crates/rum-runtime/src/runtime/debug_tools/reap_state.rs	15	rum_runtime::runtime::debug_tools::reap_state::ReapRenderNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderNode
```

`self_reaching.tsv:6` — ownership path:

```text
rum_runtime::runtime::debug_tools::test_report::report::TestReportResultNode	crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs	40	rum_runtime::runtime::debug_tools::test_report::report::TestReportResultNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::debug_tools::test_report::report::TestReportResultNode
```

`self_reaching.tsv:7` — ownership path:

```text
rum_runtime::runtime::definitions::definition_catalog::arguments::ResolvedArgumentNode	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs	15	rum_runtime::runtime::definitions::definition_catalog::arguments::ResolvedArgumentNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::definitions::definition_catalog::arguments::ResolvedArgumentNode
```

`self_reaching.tsv:8` — ownership path:

```text
rum_runtime::runtime::definitions::definition_lhs_preparation::resume_frames::DefinitionLhsParentNode	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs	51	rum_runtime::runtime::definitions::definition_lhs_preparation::resume_frames::DefinitionLhsParentNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::definitions::definition_lhs_preparation::resume_frames::DefinitionLhsParentNode
```

`self_reaching.tsv:9` — ownership path:

```text
rum_runtime::runtime::definitions::value_table_assignment::PreparedValueTableRuleNode	crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs	41	rum_runtime::runtime::definitions::value_table_assignment::PreparedValueTableRuleNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::definitions::value_table_assignment::PreparedValueTableRuleNode
```

`self_reaching.tsv:10` — ownership path:

```text
rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode	crates/rum-runtime/src/runtime/eval_call_prepare.rs	29	rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode
```

`self_reaching.tsv:11` — ownership path:

```text
rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode	crates/rum-runtime/src/runtime/kernel_data/association_data.rs	592	rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode
```

`self_reaching.tsv:13` — ownership path:

```text
rum_runtime::runtime::kernel_data::rewrite_data::suspended_state::PersistentRewriteExprNode	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs	111	rum_runtime::runtime::kernel_data::rewrite_data::suspended_state::PersistentRewriteExprNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::rewrite_data::suspended_state::PersistentRewriteExprNode
```

`self_reaching.tsv:14` — ownership path:

```text
rum_runtime::runtime::kernel_data::rewrite_data::suspended_state::PersistentRewriteQueueNode	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs	59	rum_runtime::runtime::kernel_data::rewrite_data::suspended_state::PersistentRewriteQueueNode.next/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::rewrite_data::suspended_state::PersistentRewriteQueueNode
```

`self_reaching.tsv:16` — ownership path:

```text
rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::GuardedTargetChunk	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	90	rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::GuardedTargetChunk.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::GuardedTargetChunk
```

`self_reaching.tsv:17` — ownership path:

```text
rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::GuardedTargetUpdateNode	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	105	rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::GuardedTargetUpdateNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::GuardedTargetUpdateNode
```

`self_reaching.tsv:18` — ownership path:

```text
rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::PersistentReplacePartTargetNode	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs	29	rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::PersistentReplacePartTargetNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::structure_data::replace::suspended_state::PersistentReplacePartTargetNode
```

`self_reaching.tsv:25` — ownership path:

```text
rum_runtime::runtime::pattern_matching::suspended_state::PersistentMatchResultsNode	crates/rum-runtime/src/runtime/pattern_matching/suspended_state.rs	81	rum_runtime::runtime::pattern_matching::suspended_state::PersistentMatchResultsNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::pattern_matching::suspended_state::PersistentMatchResultsNode
```

`self_reaching.tsv:26` — ownership path:

```text
rum_runtime::runtime::resolution::FixedPointHistoryNode	crates/rum-runtime/src/runtime/resolution.rs	18	rum_runtime::runtime::resolution::FixedPointHistoryNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::resolution::FixedPointHistoryNode
```

`self_reaching.tsv:33` — ownership path:

```text
rum_runtime::runtime::tree_walkers::position::PersistentPositionNode	crates/rum-runtime/src/runtime/tree_walkers/position.rs	18	rum_runtime::runtime::tree_walkers::position::PersistentPositionNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::tree_walkers::position::PersistentPositionNode
```

Class (c): **none**. `host_payloads.tsv` and `unknown_storage.tsv` are
byte-identical to the baseline. Besides the enumerated derive/link additions,
`unknown_attributes.tsv` has position drift; `unknown_captures.tsv` and
`unknown_macros.tsv` have position drift only (including embedded end positions).
The generic wrapper's pinned constructor row remains unchanged. The
self-reaching inventory still contains all 57 rows, including all sixteen
converted members and all ten remaining stdlib members.

## 7. Stop-loss and open questions

**The no-default-path-cost gate is not green.** All three measured ordinary
construction paths regress in retired instructions by much more than their
within-arm spreads. Identical allocation count/bytes/peak proves only that
allocation behaviour is unchanged; it does not waive the instruction cost.
The candidate is left for review, not declared complete or ready to accept.
After this gate failure, no further production implementation changes or
alternate performance experiment were made; only the prescribed validation,
inventory disclosure, and report were completed.

The source-level conversion retains the original allocation and clone sites,
but installing a wrapper also changes drop glue on replaced/extracted links.
That is a possible explanation, not an isolated causal measurement. Resolving
it needs a director decision about the allowed next scope; I did not alter the
finished generic helper/derive, restructure the ordinary drivers, add cfg(test)
shortcuts, remove the wrapper from an owning link, or weaken the performance
criterion. No node-field move stopped compiling. The sole compile obstacle
was the missing trait import and it was fixed on the next compile.

The ten stdlib members are unchanged: `PersistentAssociationGroupNode`,
`PersistentAssociationSequenceNode`, `PersistentFunctionalSequenceNode`,
`PersistentLogicSequenceNode`, `PersistentPredicateMatchNode`,
`PersistentStringSequenceNode`, `PersistentStringTextNode`,
`PersistentStructureCountNode`, `PersistentStructureSequenceNode`, and
`PersistentStructureSetNode`. Their conversions and the complete family gate
remain brief-490 work. Their construction paths were not newly audited here,
and no performance conclusion about them is claimed. Step D remains blocked;
no suspension frame was declared `BoundedDestruction`.
