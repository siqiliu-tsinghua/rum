# brief-492 — stdlib chains and family coverage gate

All ten remaining members now use `AccumulatorChain`, and the discovery-based
participation gate passes with zero uncovered paths. The mandatory workspace
suite passed: **2,460 passed, 0 failed, 57 ignored**. Formatting passed.

**Measured ordinary-path cost: +3.563417% instructions** on the `DeleteMissing`
heap-traversal workload; all allocation counters are identical. This cost is
reported as the brief directs, without tuning the settled wrapper.

The entry tree was clean at `3d19f1f85b54883a9d348c76ffbb094ac13efe16`.
The change remains uncommitted. Scratch copies, generated `.rum` input,
regeneration tooling and raw logs are in `target/brief-492`; build/test temporary
files used its `tmp` directory. No suspension frame was declared
`BoundedDestruction`; step D remains separate.

## 1. Derived list

Selected the entry inventory rows whose ownership path contains
`Option[Own]/arg0/std::sync::Arc` and does not contain `AccumulatorChain`.
There are exactly ten, all in `rum-stdlib`, matching the brief's cross-check
with no missing or unexpected member. Entry source witnesses:

| Member | Entry construction module / declaration |
| --- | --- |
| `PersistentAssociationGroupNode` | `crates/rum-stdlib/src/associations/suspended_state.rs:111` |
| `PersistentAssociationSequenceNode` | `crates/rum-stdlib/src/associations/suspended_state.rs:57` |
| `PersistentFunctionalSequenceNode` | `crates/rum-stdlib/src/functional_ops/suspended_state.rs:36` |
| `PersistentLogicSequenceNode` | `crates/rum-stdlib/src/logic/suspended_state.rs:13` |
| `PersistentPredicateMatchNode` | `crates/rum-stdlib/src/predicates/suspended_state.rs:29` |
| `PersistentStringSequenceNode` | `crates/rum-stdlib/src/strings/suspended_state.rs:45` |
| `PersistentStringTextNode` | `crates/rum-stdlib/src/strings/suspended_state.rs:101` |
| `PersistentStructureCountNode` | `crates/rum-stdlib/src/structure_ops/suspended_state.rs:150` |
| `PersistentStructureSequenceNode` | `crates/rum-stdlib/src/structure_ops/suspended_state.rs:56` |
| `PersistentStructureSetNode` | `crates/rum-stdlib/src/structure_ops/suspended_state.rs:234` |

## 2. Changes per member

Paths in this table are relative to `crates/rum-stdlib/src/`; ranges refer to
the final candidate. Each member derives `AccumulatorChainNode`, marks its
`previous` field with `#[accumulator_link]`, and wraps both that link and its
holder's `tail`. Imports use `rum_runtime::kernel_api`, following `IndexChunk`.
Each of the six modules adds those imports at its top (lines 3–5, or 4–6 for
`structure_ops`).

| Member | Module and changed regions | Why |
| --- | --- | --- |
| PersistentAssociationGroupNode | `associations/suspended_state.rs:107–138` | Iteratively release the grouping contribution log. |
| PersistentAssociationSequenceNode | `associations/suspended_state.rs:51–84` | Iteratively release association output chains, including ordinary heap traversal. |
| PersistentFunctionalSequenceNode | `functional_ops/suspended_state.rs:30–63` | Iteratively release resumed output/history chains. |
| PersistentLogicSequenceNode | `logic/suspended_state.rs:7–40` | Iteratively release resumed logical residuals. |
| PersistentPredicateMatchNode | `predicates/suspended_state.rs:24–55` | Iteratively release recorded match updates. |
| PersistentStringSequenceNode | `strings/suspended_state.rs:39–73` | Iteratively release resumed string result sequences. |
| PersistentStringTextNode | `strings/suspended_state.rs:96–128` | Iteratively release replacement text fragments using the same wrapper as every member. |
| PersistentStructureCountNode | `structure_ops/suspended_state.rs:147–159,168–175,199–219` | Iteratively release both count-update variants. |
| PersistentStructureSequenceNode | `structure_ops/suspended_state.rs:50–98` | Iteratively release chunks while retaining unique-tail mutation and capacity 32. |
| PersistentStructureSetNode | `structure_ops/suspended_state.rs:141–145,239–275` | Iteratively release resumed set insertion logs. |

`PersistentStructureSequence::push` detaches the optional Arc through
`take().into_inner()` so its existing `Arc::get_mut` check remains available,
then rewraps it on either the reused-chunk or new-chunk path. It adds no branch,
clone, or allocation. Across all six modules, the explicit `Arc::new`,
`Vec::with_capacity`, `.clone()` and `if` counts are unchanged. This source
preservation does not imply zero instruction cost; see section 4.

The new test file is `tests/ownership_graph/accumulator_coverage.rs`, wired
through `tests/ownership_graph/mod.rs`. No immediate caller needed an edit.
Runtime traversal inventories passed unchanged. All touched implementation
modules remain below 10k characters (largest: 8,927); no module split is needed.

## 3. Coverage gate and falsifier

The gate calls `graph::discover(source::production())`. It checks discovered
cycle witnesses plus every direct self-edge, so a canonical witness through a
different field cannot conceal a raw self-link. It selects paths containing
`Option[Own]/arg0/std::sync::Arc` and rejects paths without the resolved runtime
`AccumulatorChain` wrapper. There is no family name list, allowlist, exemption
table, snapshot input, or fixed family count in the gate.

Failures print the type, declaration file and line, and complete ownership path.
Its comment separates the three claims: the derive proves the link's type,
the existing helper tests prove the iterative drain, and this gate proves
participation. A separate small source fixture checks fresh names, an extra raw
self-edge behind a different canonical cycle, and a misleading type name
containing `AccumulatorChain`; `Option<Box<Self>>` is outside this family.

The gate **would have caught all 29 members in the tree before the conversion
series**: each had this raw Option/Arc self-reaching ownership path. That is a
structural conclusion, not a claim that this turn rebuilt that historical tree.
The actual entry tree had nineteen conversions already, and running the new
gate on it found all ten remaining raw paths (expected exit 101). Fresh final
discovery has 29 family members, all wrapped, and the gate currently finds zero.

For the mandatory falsifier, I saved the converted logic module in
`target/brief-492/logic-converted.rs`, manually changed only that member and its
holder back to raw `Option<Arc<...>>`, and removed its derive/field marker so
the build could reach the discovery assertion. The pinned inventory was not
changed for this test. After the red run, I restored the conversion by hand with
a patch; `cmp` against the scratch copy exited 0. No checkout/stash was used.

Verbatim red run, exit 101:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.20s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
uncovered accumulator ownership paths: 1

thread 'tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates' panicked at crates/rum-stdlib/src/tests/ownership_graph/accumulator_coverage.rs:39:5:
self-reaching Option<Arc<_>> chains must carry AccumulatorChain:
rum_stdlib::logic::suspended_state::PersistentLogicSequenceNode at crates/rum-stdlib/src/logic/suspended_state.rs:13: ownership path: rum_stdlib::logic::suspended_state::PersistentLogicSequenceNode.previous/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::logic::suspended_state::PersistentLogicSequenceNode
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... FAILED

failures:

failures:
    tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 978 filtered out; finished in 4.92s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

Verbatim manually restored green run, exit 0:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.38s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
uncovered accumulator ownership paths: 0
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 978 filtered out; finished in 5.79s

     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-44228b396944f9ba)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## 4. Construction paths and performance

Paths below are relative to `crates/rum-stdlib/src/`. The construction-site
column gives the final `Arc::new(Node)` line, not just a holder declaration.
All freeze call sites were reviewed, including indirect helpers and calls
inside already-suspended loops.

| Member | Construction site | Ordinary versus post-suspension evidence / A/B decision |
| --- | --- | --- |
| PersistentAssociationGroupNode | `associations/suspended_state.rs:132` | **Post-suspension only.** `associations/grouping/list.rs:58–68,79–89` freezes only in InputRequired/suspend arms; the suspended driver appends at `:164`. Association grouping, key grouping and CountsBy have the same guarded ingress (`grouping/association.rs:69,92`, `key_reshape.rs:119`, `resumable/aggregate.rs:217`). No A/B for this member. |
| PersistentAssociationSequenceNode | `associations/suspended_state.rs:78` | **Ordinary and resumed.** `associations/missing/target.rs:202–207,323–328` freezes cursors when the bounded recursive front ends, without InputRequired. `:380–382` calls the heap driver directly, which appends at `:421` and on completion. Cursor holders freeze at `:605,615`. A/B required. Other ingress includes actual suspension, e.g. `associations/resumable/map.rs:65–74`, then appends at `:111,130`. |
| PersistentFunctionalSequenceNode | `functional_ops/suspended_state.rs:57` | **Post-suspension only.** Fold freezes in the InputRequired/suspend arm at `functional_ops/sequence/fold/plain.rs:46–55`; normal work keeps its vector; resumed appends are at `:111,126`. Nest/history ingress is in suspend arms at `functional_ops/iteration.rs:277,366–367,490,549`. The apparent unguarded tensor freezes at `functional_ops/arrays/tensor.rs:550,588` are inside already-suspended drivers (entry at `:422–424,445`; frame calls in `arrays/tensor/resume_frames.rs:96,134`). No A/B. |
| PersistentLogicSequenceNode | `logic/suspended_state.rs:34` | **Post-suspension only.** All four freeze sites are InputRequired/suspend arms at `logic/truth.rs:184,276,368,461`; resumed appends occur through `:231,533–545`, and directly at `:415,508`. No A/B. |
| PersistentPredicateMatchNode | `predicates/suspended_state.rs:49` | **Post-suspension only.** `predicates/collections.rs:530–546` creates suspended state only on InputRequired; `:599,611` record resumed matches. Ordinary work mutates the matrix directly at `:554`. No A/B. |
| PersistentStringSequenceNode | `strings/suspended_state.rs:66` | **Post-suspension only.** InputRequired freezes rendered prefixes at `strings/patterns/search/rules.rs:489–491`; resumed append is at `:529`. Cases/guarded variants likewise freeze in their InputRequired arms (`rules/cases.rs:210,464`, `guarded/cases.rs:121,191,430`, `guarded/replace.rs:570`). No A/B. |
| PersistentStringTextNode | `strings/suspended_state.rs:123` | **Post-suspension only.** `strings/patterns/search/rules.rs:218–219` dispatches InputRequired to the cold helper that freezes at `:238`; resumed append sites include `:270,281,285`. Guarded ingress freezes at `strings/patterns/search/guarded/replace.rs:174,263`, reached exclusively from InputRequired arms at `:129,243`; resumed character append is `:288`. No A/B and no reproduction of the long-chain construction. |
| PersistentStructureCountNode | `structure_ops/suspended_state.rs:202,213` | **Post-suspension only.** `structure_ops/collection_stats.rs:155–164` freezes after InputRequired; suspended tally appends at `:194` and increments at `:219`. Ordinary work mutates the vector at `:142,172`. No A/B. |
| PersistentStructureSequenceNode | `structure_ops/suspended_state.rs:87` | **Post-suspension only.** `structure_ops/dedup.rs:173–183` freezes in InputRequired; resumed appends at `:227,246`. Other freezes are guarded in collection stats/list lookup/sorting. The indirect `ordering_sets/suspended.rs:24` constructor is called only in suspend arms at `ordering_sets.rs:576,613,643`. No A/B. |
| PersistentStructureSetNode | `structure_ops/suspended_state.rs:268` | **Post-suspension only.** Its sole freeze at `structure_ops/dedup.rs:181` is in InputRequired/suspend; resumed inserts at `:224,245`. The ordinary path uses `AdaptiveExprSet::insert` at `:190`. No A/B. |

The association sequence's pre-existing comment about the ordinary vector path
does not cover the `DeleteMissing` heap handoff. Classification above follows
the callers rather than that comment or the helper names. Nine members require
no A/B; the one ordinary-path member was measured.

Both arms were built at `target/release/rum-cli`, Rust 1.85.0, release profile.
Default-feature builds were used for instructions; profiling environment was
OFF (no inherited `RUM_*` variables). Three runs per arm used CPU 2 and stdin
closed. No build or test ran concurrently with a measurement. Every counted
`cpu_core/instructions/u` row has `100.00` multiplex coverage; the uncounted
`cpu_atom` rows were not used. This brief's instruction-count recipe supplies
the performance gate; no timing A/B was substituted.

The workload performs 200 ordinary `DeleteMissing` calls at target level 20,
with sixteen retained leaf integers and one Missing value. Depth 20 crosses the
implementation's 16-level recursive-front handoff; it does not cross a resource
ceiling or configure lower limits. The returned value is checked against the
expected nested list. No input, long-chain, crash, overflow or fatal-process
construction is involved. This measures the complete ordinary workload, not
isolated per-node instructions or the other nine members' suspension paths.

`target/brief-492/delete-missing-target.rum`, verbatim:

```rum
dmInput = {{{{{{{{{{{{{{{{{{{{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, Missing[]}}}}}}}}}}}}}}}}}}}};
Do[dmResult = DeleteMissing[dmInput, 20, 0], {200}];
Print[dmResult === {{{{{{{{{{{{{{{{{{{{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16}}}}}}}}}}}}}}}}}}}}];
```

Every instruction and allocation run returned this stdout:

```text
True
Null
```

| Arm | Run 1 | Run 2 | Run 3 | Mean instructions | Within-arm range | Range / mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| base | 21,769,734 | 21,769,577 | 21,771,487 | 21,770,266.000 | 1,910 | 0.008773% |
| candidate | 22,545,468 | 22,543,642 | 22,548,984 | 22,546,031.333 | 5,342 | 0.023694% |

The candidate costs **775,765.333 more instructions, +3.563417%**. This is well
outside the observed 0.008773% base and 0.023694% candidate spreads, so it is a
measurable cost, not noise or a zero-default-cost result. Per the brief's
explicit instruction, it is reported and retained without wrapper tuning.

Verbatim perf output, base runs 1–3 followed by candidate runs 1–3:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
21769734,,cpu_core/instructions/u,6963818,100.00,,
<not counted>,,cpu_atom/instructions/u,0,0.00,,
21769577,,cpu_core/instructions/u,6225523,100.00,,
<not counted>,,cpu_atom/instructions/u,0,0.00,,
21771487,,cpu_core/instructions/u,5692428,100.00,,
<not counted>,,cpu_atom/instructions/u,0,0.00,,
22545468,,cpu_core/instructions/u,12555244,100.00,,
<not counted>,,cpu_atom/instructions/u,0,0.00,,
22543642,,cpu_core/instructions/u,10537538,100.00,,
<not counted>,,cpu_atom/instructions/u,0,0.00,,
22548984,,cpu_core/instructions/u,9961582,100.00,,
```

Allocation checks used separate release builds with `--features alloc-count`
at the same path, setting `RUM_PROFILE_ALLOC=1` only for the allocation runs.
Base and candidate stderr are byte-identical, verbatim:

```text
rum-profile alloc count=40490 bytes_total=6264789 peak_live_bytes=2215834
```

## 5. Verification

Final `cargo fmt --check` exited 0 with empty output. Final workspace tests
exited 0: **2,460 passed, 0 failed, 57 ignored**, summed across all 32 unit,
integration and doc-test summaries. These include the family gate, exact
ownership inventory, existing iterative-drain and derive tests, suspension
behavior, and unchanged runtime frame-root traversal inventories.

Loopback is unavailable in this sandbox: the capability check was denied
already at socket creation with `PermissionError: [Errno 1] Operation not
permitted`. The workspace command used the repository's existing
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` option, implemented at
`crates/rum-jupyter/src/zeromq_native_test_support.rs:335–353`. Native socket
tests therefore return early; they are included in harness pass counts rather
than the 57 ignored count. No skip implementation or test contract changed.

The deliberate entry-tree and falsifier gate runs exited 101 as recorded.
An initial formatting check exited 1 for line wrapping in the new test;
that line was corrected manually before the final green check. The first
regeneration run enforced ten changes of each kind, but its copied display
string incorrectly said sixteen link/path changes; the display was fixed and
regeneration rerun from the same discovery output against saved HEAD rows.
Both raw regeneration logs are retained. No class-(c) change occurred.

Build, test, benchmark and regeneration commands, verbatim and in execution
order (including expected failing checks and the regeneration summary fix):

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --offline --locked > target/brief-492/logs/build-base.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/release/rum-cli --file target/brief-492/delete-missing-target.rum < /dev/null > "target/brief-492/logs/instructions-base-${run}.stdout" 2> "target/brief-492/logs/instructions-base-${run}.csv" || exit; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --features alloc-count --offline --locked > target/brief-492/logs/build-alloc-base.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" RUM_PROFILE_ALLOC=1 python3 scripts/run-bounded.py --timeout 300 -- taskset -c 2 target/release/rum-cli --file target/brief-492/delete-missing-target.rum < /dev/null > target/brief-492/logs/alloc-base.stdout 2> target/brief-492/logs/alloc-base.stderr
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::accumulator_coverage -- --nocapture > target/brief-492/logs/gate-before-conversion.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check > target/brief-492/logs/fmt-initial.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::accumulator_coverage -- --nocapture > target/brief-492/logs/gate-converted.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates -- --exact --nocapture > target/brief-492/logs/falsifier-red.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates -- --exact --nocapture > target/brief-492/logs/falsifier-restored-green.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture > target/brief-492/logs/discovery.log 2>&1
python3 scripts/run-bounded.py --timeout 180 -- python3 target/brief-492/regenerate-inventory.py > target/brief-492/logs/inventory-regeneration.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --offline --locked > target/brief-492/logs/build-candidate.log 2>&1
python3 scripts/run-bounded.py --timeout 180 -- python3 - <<'PY'
from pathlib import Path
from collections import Counter
import json, subprocess
root=Path('crates/rum-stdlib/src/tests/ownership_graph/inventory')
base={p.name:subprocess.check_output(['git','show',f'HEAD:{p.as_posix()}'],text=True).splitlines() for p in sorted(root.glob('*.tsv'))}
Path('target/brief-492/base-inventory.json').write_text(json.dumps(base,indent=2)+'\n')
p=Path('target/brief-492/regenerate-inventory.py')
s=p.read_text().replace("old = {path.name: path.read_text().splitlines() for path in sorted(root.glob('*.tsv'))}", "old = json.loads(Path('target/brief-492/base-inventory.json').read_text())")
s=s.replace('Allowed changes: 10 derives, 16 link attributes, 16 ownership paths', 'Allowed changes: 10 derives, 10 link attributes, 10 ownership paths')
p.write_text(s)
log=Path('target/brief-492/logs/inventory-regeneration.log')
Path('target/brief-492/logs/inventory-regeneration-initial.log').write_text(log.read_text())
changes=json.loads(Path('target/brief-492/inventory-changes.json').read_text())
print('Audited content changes:', dict(Counter(c['kind'] for c in changes)))
PY
python3 scripts/run-bounded.py --timeout 180 -- python3 target/brief-492/regenerate-inventory.py > target/brief-492/logs/inventory-regeneration.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/release/rum-cli --file target/brief-492/delete-missing-target.rum < /dev/null > "target/brief-492/logs/instructions-candidate-${run}.stdout" 2> "target/brief-492/logs/instructions-candidate-${run}.csv" || exit; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --features alloc-count --offline --locked > target/brief-492/logs/build-alloc-candidate.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" RUM_PROFILE_ALLOC=1 python3 scripts/run-bounded.py --timeout 300 -- taskset -c 2 target/release/rum-cli --file target/brief-492/delete-missing-target.rum < /dev/null > target/brief-492/logs/alloc-candidate.stdout 2> target/brief-492/logs/alloc-candidate.stderr
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked > target/brief-492/logs/workspace.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-492/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check > target/brief-492/logs/fmt-final.log 2>&1
cmp crates/rum-stdlib/src/logic/suspended_state.rs target/brief-492/logic-converted.rs
git diff --check
```

`cmp` and final `git diff --check` both exited 0 with no output. The `cmp` was
run immediately after the manual falsifier restoration. All cargo build/test
commands used `--offline --locked`; `CARGO_NET_OFFLINE=true` also covered fmt.

Verbatim workspace result lines:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.84s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 990 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.22s
test result: ok. 927 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 8.89s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.87s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 6. Inventory disclosure

All six partitions were regenerated from the fresh discovery report. Before
writing, the regeneration audit compared position-normalized semantic multisets
and required identity after undoing only the ten allowed derive additions,
ten link attributes and ten wrapper-path insertions. The self-reaching type
set is unchanged: all 57 rows remain, including all ten converted members.
No new UnknownHostPayload, removed semantic row or reclassification occurred.
Class (c): **none**.

`host_payloads.tsv` and `unknown_macros.tsv` are byte-unchanged.
`unknown_captures.tsv` and `unknown_storage.tsv` have position drift only.
`unknown_attributes.tsv` has twenty class-(a) changes: ten existing
`derive (FrameRootTrace)` rows become
`derive (FrameRootTrace , AccumulatorChainNode)`, and ten
`accumulator_link` rows are added. `self_reaching.tsv` has ten class-(b) changes:
the same self-ownership path now passes through the runtime wrapper.

Verbatim regeneration summary:

```text
host_payloads.tsv: 50 -> 50 rows; changed=False
self_reaching.tsv: 57 -> 57 rows; changed=True
unknown_attributes.tsv: 1347 -> 1357 rows; changed=True
unknown_captures.tsv: 2774 -> 2774 rows; changed=True
unknown_macros.tsv: 2895 -> 2895 rows; changed=False
unknown_storage.tsv: 70 -> 70 rows; changed=True
Allowed changes: 10 derives, 10 link attributes, 10 ownership paths; class (c): none.
No added UnknownHostPayload, removed row, or reclassification. All ten members remain self-reaching.
```

Every non-position content change follows, with its final TSV witness line.

Class (a) — Attribute/derive rows:

`unknown_attributes.tsv:960` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/associations/suspended_state.rs	58	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:961` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/associations/suspended_state.rs	60	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:963` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/associations/suspended_state.rs	113	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:964` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/associations/suspended_state.rs	115	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1077` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/functional_ops/suspended_state.rs	37	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1078` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/functional_ops/suspended_state.rs	39	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1103` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/logic/suspended_state.rs	14	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1104` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/logic/suspended_state.rs	16	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1148` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/predicates/suspended_state.rs	30	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1149` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/predicates/suspended_state.rs	32	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1255` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/strings/suspended_state.rs	46	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1256` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/strings/suspended_state.rs	48	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1258` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/strings/suspended_state.rs	103	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1259` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/strings/suspended_state.rs	105	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1331` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/structure_ops/suspended_state.rs	57	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1332` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/structure_ops/suspended_state.rs	59	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1335` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/structure_ops/suspended_state.rs	154	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1336` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/structure_ops/suspended_state.rs	156	5	unresolved attribute/expansion accumulator_link
```

`unknown_attributes.tsv:1338` — derive:

```text
UnknownConstruct	crates/rum-stdlib/src/structure_ops/suspended_state.rs	239	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
```

`unknown_attributes.tsv:1339` — link attribute:

```text
UnknownConstruct	crates/rum-stdlib/src/structure_ops/suspended_state.rs	241	5	unresolved attribute/expansion accumulator_link
```

Class (b) — Ownership-path changes:

`self_reaching.tsv:37` — ownership path:

```text
rum_stdlib::associations::suspended_state::PersistentAssociationGroupNode	crates/rum-stdlib/src/associations/suspended_state.rs	114	rum_stdlib::associations::suspended_state::PersistentAssociationGroupNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::associations::suspended_state::PersistentAssociationGroupNode
```

`self_reaching.tsv:38` — ownership path:

```text
rum_stdlib::associations::suspended_state::PersistentAssociationSequenceNode	crates/rum-stdlib/src/associations/suspended_state.rs	59	rum_stdlib::associations::suspended_state::PersistentAssociationSequenceNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::associations::suspended_state::PersistentAssociationSequenceNode
```

`self_reaching.tsv:40` — ownership path:

```text
rum_stdlib::functional_ops::suspended_state::PersistentFunctionalSequenceNode	crates/rum-stdlib/src/functional_ops/suspended_state.rs	38	rum_stdlib::functional_ops::suspended_state::PersistentFunctionalSequenceNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::functional_ops::suspended_state::PersistentFunctionalSequenceNode
```

`self_reaching.tsv:42` — ownership path:

```text
rum_stdlib::logic::suspended_state::PersistentLogicSequenceNode	crates/rum-stdlib/src/logic/suspended_state.rs	15	rum_stdlib::logic::suspended_state::PersistentLogicSequenceNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::logic::suspended_state::PersistentLogicSequenceNode
```

`self_reaching.tsv:43` — ownership path:

```text
rum_stdlib::predicates::suspended_state::PersistentPredicateMatchNode	crates/rum-stdlib/src/predicates/suspended_state.rs	31	rum_stdlib::predicates::suspended_state::PersistentPredicateMatchNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::predicates::suspended_state::PersistentPredicateMatchNode
```

`self_reaching.tsv:48` — ownership path:

```text
rum_stdlib::strings::suspended_state::PersistentStringSequenceNode	crates/rum-stdlib/src/strings/suspended_state.rs	47	rum_stdlib::strings::suspended_state::PersistentStringSequenceNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::strings::suspended_state::PersistentStringSequenceNode
```

`self_reaching.tsv:49` — ownership path:

```text
rum_stdlib::strings::suspended_state::PersistentStringTextNode	crates/rum-stdlib/src/strings/suspended_state.rs	104	rum_stdlib::strings::suspended_state::PersistentStringTextNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::strings::suspended_state::PersistentStringTextNode
```

`self_reaching.tsv:53` — ownership path:

```text
rum_stdlib::structure_ops::suspended_state::PersistentStructureCountNode	crates/rum-stdlib/src/structure_ops/suspended_state.rs	155	rum_stdlib::structure_ops::suspended_state::PersistentStructureCountNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::structure_ops::suspended_state::PersistentStructureCountNode
```

`self_reaching.tsv:54` — ownership path:

```text
rum_stdlib::structure_ops::suspended_state::PersistentStructureSequenceNode	crates/rum-stdlib/src/structure_ops/suspended_state.rs	58	rum_stdlib::structure_ops::suspended_state::PersistentStructureSequenceNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::structure_ops::suspended_state::PersistentStructureSequenceNode
```

`self_reaching.tsv:55` — ownership path:

```text
rum_stdlib::structure_ops::suspended_state::PersistentStructureSetNode	crates/rum-stdlib/src/structure_ops/suspended_state.rs	240	rum_stdlib::structure_ops::suspended_state::PersistentStructureSetNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_stdlib::structure_ops::suspended_state::PersistentStructureSetNode
```

## 7. Stop-loss / open questions

No stop-loss condition remains. The ordinary association sequence cost is
reported explicitly under the brief's measured-cost instruction; no attempt
was made to hide or optimize it away. The remaining design step is D, outside
this brief. Participation does not certify frame or payload destruction.

Production changes are limited to the six modules owning the ten members.
The other changes are the new ownership-graph test/wiring, four regenerated
TSVs and this report. The nineteen earlier conversions, wrapper, derive crate,
host payload code, suspension traits, rum-core, lockfile and other documents
were left unchanged. No git mutation, network access, new unsafe, production
test-configuration divergence, or suspension/resume rewrite was introduced.
