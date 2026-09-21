# brief-486 — iteration accumulator drain

The conversion and its certificate are implemented in the working tree. The
specified ordinary Table/Do CLI A/B meets the instruction-count criterion and
has identical allocation counters. **The broader no-cost claim for `appended`
remains unverified:** the measured workload cannot reach that method, and the
CLI does not expose input resume. Section 6 records this gate limitation.

The entry worktree was clean at `b36e3da02c550de228d9ddac8bbfba6a77b3cdea`.
No stash was read or applied. No commit, branch, push, git configuration change,
network fetch, new unsafe code, or crash/overflow/fatal-process probe was made.
Generated sources, logs, and temporary files are under `target/brief-486`.

## 1. What changed

| File | Final lines | Change and reason |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs` | 2–4; 59–110 | Wrap the holder's `tail` and the node's `previous` in `AccumulatorChain`, derive the taking accessor, and extract links in `into_vec` while preserving move-when-unique/clone-when-shared completion. |
| Same file | 146–154; 187–313 | Assert root-tracing support and add the member's drain-wiring, abandonment, sharing, ordered-completion, and mid-iteration cancellation certificate. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv` | 12 | Regenerate the node's cycle witness through the wrapper; the node remains self-reaching. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 244–250 | Regenerate the changed derive and added link attribute, plus position drift. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 582 | Regenerate position drift only. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 434–435 | Regenerate position drift only. |
| `docs/dev/codex/report-486-iteration-accumulator-drain.md` | Entire file | Record the requested evidence and the unresolved performance coverage limitation. |

The driver, aggregate driver, generic wrapper, accessor derive, suspension
traits, ownership scanner, root-traversal inventories, and other 26 members
are unchanged. No suspension frame gains `BoundedDestruction`.

`appended` retains the original allocation and clones: one `Arc::new`, one
prefix clone, and one link clone. No counter, observer, conditional branch, or
extra allocation/clone was added there. This source observation is not a
measurement of the replacement holder's destructor cost.

There is one production `into_vec` body, also used by test builds. On
`Arc::try_unwrap` success it moves the node; on failure it clones the node.
`into_inner` empties the link wrapper before its `Drop`, so completion never
double-drains the extracted link. I checked the shared-clone interaction:
dropping an intact cloned node's link stops immediately at the still-shared
previous node, explicitly exercised at test line 243. In the actual completion
loop that link is extracted at line 103, leaving the wrapper empty when its
destructor runs. The shared survivor is also subsequently completed and checked
against every expected element.

Size assessment: the file is 10,567 bytes, of which the implementation before
the new test module is 5,468 bytes. Both implementation and tests are below
their separate preferred thresholds. The brief explicitly requests the
certificate in the accumulator's own test module, so no sibling test file was
introduced. This report is the only addition to the broad codex report
directory; regrouping it is outside scope.

## 2. Verification

The mandatory workspace command exited 0: **2,454 passed, 0 failed, 57 ignored**,
across 32 test-binary/doc-test groups. `cargo fmt --check` and
`git diff --check` also exited 0. The runtime workspace group has 986 passes
and 3 ignored tests; the standard-library group has 925 passes and 52 ignored
tests, including the exact ownership inventory gate. Native ZeroMQ coverage
is qualified below because the explicit sandbox skip flag was enabled.

A focused run exposed the otherwise captured native-test skip notices: all
nine native ZeroMQ tests return early because loopback binding is denied.
Libtest includes them in the workspace's 2,454 passes; excluding those nine
known early returns leaves 2,445 exercised reported passes. The 57 ignored
tests are separate. This focused run is not added to the workspace total.

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-jupyter --offline --locked zeromq::tests::native -- --nocapture --test-threads=1 > target/brief-486/logs/native-coverage.log 2>&1
```

Verbatim representative notice and test result:

```text
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s
```

All commands ran at `/home/remi/dev/rum`. Rust commands used the workspace's
Rust 1.85.0 toolchain, offline mode, and the existing lockfile. `TMPDIR` points
inside the repository. No `RUM_PROFILE_*` variable was inherited; only the two
allocation runs explicitly set `RUM_PROFILE_ALLOC=1`. The instruction runs use
the default release build without the `alloc-count` feature. Base and candidate
were built and executed at the same `target/release/rum-cli` path, with no
concurrent build/test command during either three-run instruction arm.

The exact validation commands, including failed attempts, follow. Redirections
preserve their raw output under `target/brief-486/logs`.

Instrument preflight, exit 0:

```bash
python3 scripts/run-bounded.py --timeout 30 -- taskset -c 2 perf stat -e instructions:u -x, -- /usr/bin/true
```

Base build, instruction arm, allocation build, and allocation run, all exit 0
after the workload calibration described below:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --offline --locked > target/brief-486/logs/build-base.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/release/rum-cli --file target/brief-486/workload.rum > "target/brief-486/logs/instructions-base-${run}.stdout" 2> "target/brief-486/logs/instructions-base-${run}.csv" || exit; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --features alloc-count --offline --locked > target/brief-486/logs/build-alloc-base.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" RUM_PROFILE_ALLOC=1 python3 scripts/run-bounded.py --timeout 300 -- taskset -c 2 target/release/rum-cli --file target/brief-486/workload.rum > target/brief-486/logs/alloc-base.stdout 2> target/brief-486/logs/alloc-base.stderr
```

The identical base instruction-loop command first exited 1 on its first run
with an initial outer count of 100. Its logs were preserved as
`workload-preflight-failed.stdout` and `workload-preflight-failed.csv`. Verbatim:

```text
! evaluation exceeded iteration work limit (100000)
error: evaluation exceeded iteration work limit (100000)
<not counted>,,cpu_atom/instructions/u,0,0.00,,
8255327,,cpu_core/instructions/u,7802266,100.00,,
```

Before either successful arm, the outer count was reduced to 5. That script
remained fixed across all six accepted instruction measurements and both
allocation runs. The failed limit run is excluded from the A/B, rather than
presented as a successful sample.

Scoped formatting, exit 0; first certificate compile, exit 101; corrected
certificate, exit 0:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 180 -- rustfmt --edition 2024 --config skip_children=true crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs > target/brief-486/logs/format-scoped.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::kernel_data::control_data::branching_loops::iteration::tests -- --nocapture --test-threads=1 > target/brief-486/logs/certificate-initial.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::kernel_data::control_data::branching_loops::iteration::tests -- --nocapture --test-threads=1 > target/brief-486/logs/certificate-green.log 2>&1
```

The first compile used primitive integers where `Expr::integer` requires
`BigInt`; the test constructors were corrected with `.into()`. Its real output
ended with:

```text
Some errors have detailed explanations: E0308, E0599, E0631.
For more information about an error, try `rustc --explain E0308`.
error: could not compile `rum-runtime` (lib test) due to 5 previous errors
```

Falsifier, expected exit 101; hand restoration check and restored certificate,
exit 0; unchanged helper certificate, exit 0:

```bash
sha256sum crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs > target/brief-486/candidate-before-falsifier.sha256
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::kernel_data::control_data::branching_loops::iteration::tests -- --nocapture --test-threads=1 > target/brief-486/logs/certificate-falsifier-red.log 2>&1
sha256sum -c target/brief-486/candidate-before-falsifier.sha256
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::kernel_data::control_data::branching_loops::iteration::tests -- --nocapture --test-threads=1 > target/brief-486/logs/certificate-restored-green.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::accumulator_chain::tests -- --nocapture --test-threads=1 > target/brief-486/logs/helper-certificate.log 2>&1
```

Candidate build, instruction arm, allocation build, and allocation run, all
exit 0:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --offline --locked > target/brief-486/logs/build-candidate.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/release/rum-cli --file target/brief-486/workload.rum > "target/brief-486/logs/instructions-candidate-${run}.stdout" 2> "target/brief-486/logs/instructions-candidate-${run}.csv" || exit; done'
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 900 -- cargo build -p rum-cli --release --features alloc-count --offline --locked > target/brief-486/logs/build-alloc-candidate.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" RUM_PROFILE_ALLOC=1 python3 scripts/run-bounded.py --timeout 300 -- taskset -c 2 target/release/rum-cli --file target/brief-486/workload.rum > target/brief-486/logs/alloc-candidate.stdout 2> target/brief-486/logs/alloc-candidate.stderr
```

Discovery, guarded regeneration, formatting, workspace, and whitespace check:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture > target/brief-486/logs/discovery.log 2>&1
python3 scripts/run-bounded.py --timeout 180 -- python3 target/brief-486/regenerate-inventory.py > target/brief-486/logs/inventory-regeneration.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check > target/brief-486/logs/fmt-check.log 2>&1
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-486/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked > target/brief-486/logs/workspace.log 2>&1
git diff --check
```

Verbatim workspace group summaries:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 60.15s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 986 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.33s
test result: ok. 925 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 8.88s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.46s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Summed across those 32 groups: 2,454 passed, 0 failed, 57 ignored.
The summary total is a sum of real libtest rows, not an invented cargo output line.

The three existing dead-code warnings in `test_builtins.rs` appeared in focused
and workspace builds. The initial compile failure and workload calibration
failure above were corrected; neither was silently counted as green.

## 3. The certificate

The new certificate has four tests:

- Build 16- and 64-node chains through `appended`, require both owning link
  fields to be the certified `AccumulatorChain` type, then drop the accumulator
  without `into_vec`. Weak witnesses for every previous node become dead.
- Drop a clone and a cloned node over a shared tail, then abandon an eight-node
  branch. Its exclusive previous nodes are released; the shared tail stays
  alive and the survivor yields exactly `0..16` before its witnesses expire.
- Complete a two-element frozen prefix plus 64 appended elements, both uniquely
  owned and while a clone survives. Both outputs are exactly `-2..64` in order;
  the shared survivor also completes correctly.
- Resume a real `Table` after its first input, collect 32 values, suspend at
  iteration 33, and cancel. The iterator binding restores to 99 and the body
  hit count stays 33, proving abandonment did not resume the body.

The mechanism-depth observer is reused by running brief-485's existing helper
certificate unchanged. Verbatim observations:

```text
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

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 984 filtered out; finished in 0.00s
```

The member's discriminator is explicitly a **wiring certificate**, backed by
that unchanged helper-depth certificate. It does not pretend to observe
`Expr` destructor nesting. The Weak and exact-value assertions establish
release and sharing behavior, rather than merely surviving a long drop.

For the required falsifier, only the holder's tail was changed to raw
`Option<Arc<PersistentExprAccumulatorNode>>`, with the necessary construction
and extraction adaptations: `None` in `freeze`, raw `Some(Arc::new(...))` in
`appended`, wrapping `self.tail.clone()` for the still-wrapped node link, and
direct extraction of `self.tail` in `into_vec`. The tests were untouched.
The mutation compiled; three runtime assertions failed. The cancellation
behavior test remained green, honestly showing that behavior alone would not
discriminate this wiring change. The first failure and summary, verbatim:

```text
thread 'runtime::kernel_data::control_data::branching_loops::iteration::tests::appended_chain_abandonment_uses_drain' panicked at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:197:9:
assertion `left == right` failed: owning iteration links must use the certified accumulator drain
  left: "core::option::Option<alloc::sync::Arc<rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::PersistentExprAccumulatorNode>>"
 right: "rum_runtime::runtime::accumulator_chain::AccumulatorChain<rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::PersistentExprAccumulatorNode>"
```

```text
failures:
    runtime::kernel_data::control_data::branching_loops::iteration::tests::appended_chain_abandonment_uses_drain
    runtime::kernel_data::control_data::branching_loops::iteration::tests::into_vec_preserves_prefix_and_tail_order_when_unique_or_shared
    runtime::kernel_data::control_data::branching_loops::iteration::tests::shared_tail_survives_abandoned_clone

test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 985 filtered out; finished in 0.01s
```

Hand restoration used an inverse patch, not git checkout. SHA-256 confirmed the
entire source file returned byte-for-byte to its pre-falsifier candidate:

```text
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs: OK
```

Restored green, verbatim:

```text
running 4 tests
test runtime::kernel_data::control_data::branching_loops::iteration::tests::accumulator_is_abandoned_mid_iteration_without_resuming_body ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::appended_chain_abandonment_uses_drain ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::into_vec_preserves_prefix_and_tail_order_when_unique_or_shared ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::shared_tail_survives_abandoned_clone ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 985 filtered out; finished in 0.01s
```

## 4. The A/B

Allocation counters first. Both release `alloc-count` builds emitted this exact
line on the same workload:

```text
rum-profile alloc count=22534 bytes_total=4704106 peak_live_bytes=3272117
```

The allocation count, total allocated bytes, and peak live bytes are identical.

The fixed `target/brief-486/workload.rum` is:

```rum
Do[Table[Null, {5000}]; Table[Null, {j, 1000}]; Table[j, {j, 1000}]; Table[j + 1, {j, 1000}]; Table[{j, j}, {j, 1000}]; Do[Null, {10000}]; Do[Null, {j, 1000}], {5}];
Print["brief-486 workload complete"];
```

Every successful invocation printed:

```text
brief-486 workload complete
Null
```

Default-release instruction counts, three runs per arm:

| Arm | Run 1 | Run 2 | Run 3 | Mean | Max minus min | Spread / mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Base | 57,191,077 | 57,190,716 | 57,191,451 | 57,191,081.333 | 735 | 0.001285% |
| Candidate | 57,190,069 | 57,191,134 | 57,190,817 | 57,190,673.333 | 1,065 | 0.001862% |

Candidate minus base mean: **−408 instructions (−0.000713%)**. Its magnitude
is smaller than either arm's observed spread, meeting the specified criterion
for this workload. No timing reruns were used to override it.

Verbatim `cpu_core/instructions/u` rows, base then candidate:

```text
57191077,,cpu_core/instructions/u,13795852,100.00,,
57190716,,cpu_core/instructions/u,14328083,100.00,,
57191451,,cpu_core/instructions/u,15010416,100.00,,
57190069,,cpu_core/instructions/u,15954257,100.00,,
57191134,,cpu_core/instructions/u,14229416,100.00,,
57190817,,cpu_core/instructions/u,14514902,100.00,,
```

Every run also emitted the unused other-core-type row
`<not counted>,,cpu_atom/instructions/u,0,0.00,,`; every counted CPU-core row
has the required **100.00** multiplex coverage.

Named controls in the combined workload correspond to `table-count-null`,
`table-iterator-null`, `table-iterator-symbol`, `table-iterator-plus`,
`table-iterator-pair-list`, `do-count-null`, and `do-iterator-null`. Their
implementations are untouched; they are measured together, not as separate
per-label samples. The observed instruction spread band is 0.001285–0.001862%.
The report template's two timing pairs and timing control band were superseded
by the redispatched instruction-count requirement; none were run or invented.
The two actual comparisons are the allocation pair and the three-run
instruction arms above.

## 5. Inventory diff, classified

All six TSVs were regenerated from fresh discovery output, guarded by an exact
semantic multiset comparison allowing only the following three changes plus
source-position drift. No row was edited by hand. No row was removed or
reclassified, and no `UnknownHostPayload` was added.

Class (a), changed node derive at `unknown_attributes.tsv:247`, witness
`iteration/mod.rs:66:1`; previously `(Clone , FrameRootTrace)`:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs	66	1	unresolved attribute/expansion derive (Clone , FrameRootTrace , AccumulatorChainNode)
```

Class (a), new link attribute at `unknown_attributes.tsv:248`, witness
`iteration/mod.rs:68:5`:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs	68	5	unresolved attribute/expansion accumulator_link
```

Class (b), changed ownership path at `self_reaching.tsv:12`, witness
`iteration/mod.rs:67`; the old path went directly from `.previous` to `Option`:

```text
rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::PersistentExprAccumulatorNode	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs	67	rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::PersistentExprAccumulatorNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::control_data::branching_loops::iteration::PersistentExprAccumulatorNode
```

Class (c): **none**. Position-only changes are five other derive rows, one
closure-capture row, and two macro rows, including their end positions.

Regeneration counts, verbatim:

```text
host_payloads.tsv: 50 -> 50 rows; changed=False
self_reaching.tsv: 57 -> 57 rows; changed=True
unknown_attributes.tsv: 1330 -> 1331 rows; changed=True
unknown_captures.tsv: 2774 -> 2774 rows; changed=True
unknown_macros.tsv: 2895 -> 2895 rows; changed=True
unknown_storage.tsv: 70 -> 70 rows; changed=False
```

The existing generic-wrapper constructor at `host_payloads.tsv:12` is
byte-identical. `PersistentExprAccumulatorNode` remains one of the 57
self-reaching types. The scanner's source and classification logic are
unchanged.

## 6. Stop-loss / open questions

**The specified CLI A/B cannot establish zero cost on the changed collection
path.** This is a coverage limitation, not an observed instruction regression:

- `iteration/driver.rs:263–304` collects ordinary Table values directly in a
  vector and freezes it only after `InputRequired`.
- `iteration/driver.rs:310–355` uses `appended` only after resumption. The
  ordinary fast paths also return vectors directly. `Do` does not collect.
- `rum-cli/src/main.rs:245–280` converts `InputRequired` into a CLI error;
  it does not resume the request. The REPL similarly renders input-required
  output at `rum-repl/src/lib.rs:139–150` without calling `resume_with_input`.

Thus the required fixed ordinary Table/Do CLI workload is a useful unaffected
control and meets its numerical acceptance criterion, but does not measure
the possible per-iteration cold drain call identified in the brief. I have
not claimed that it proves `appended` and its replacement drops cost nothing.
A performance gate that exercises resume needs a host/runtime driver (or a
CLI resume capability), beyond the specified CLI experiment. No such
out-of-scope code was introduced, and the forbidden generic helper and derive
were not changed to hide a possible cost.

The conversion has one identical `into_vec` implementation in production and
tests; the former test-build divergence is absent. The certificate's observed
falsifier failure is structural wiring, not an instrumented `Expr` destructor.
That distinction is explicit rather than counting a survival test as a depth
proof.

The other 26 members were not modified or newly audited; no new issue about
them was found during this bounded conversion. Step D remains blocked on
their conversion and no bounded-destruction suspension declaration was added.
All changes remain uncommitted for the director's gate.
