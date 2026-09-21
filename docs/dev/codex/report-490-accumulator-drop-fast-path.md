# brief-490 — accumulator drop fast path

**STOP: acceptance is not met after two attempts.** The final candidate is
+2.242822%, +1.940998%, and +3.571836% above `HEAD` on the three instruction
workloads, outside both arms' observed spreads. Both attempts are worse than
brief-489's failing candidate. No third shape was attempted. The final patch
remains uncommitted for the director; it is not ready to accept.

The unchanged mechanism certificate, formatting, exact ownership inventory,
and full workspace suite pass. The literal drain-only falsifier reports depths
8 and 32 with the suggested fast path; a separately disclosed fully recursive
mutation reports 16 and 64. Neither result is relabelled or hidden.

All scratch files, source exports, build products, logs, and saved measurement
binaries are under `target/brief-490`. Builds/tests used an in-repository
`TMPDIR`, offline Cargo, and the bounded runner. No network fetch, git mutation
command, unsafe addition, production/test body divergence, crash, overflow,
past-ceiling, or fatal-process probe was introduced.

## 1. What changed

Only `crates/rum-runtime/src/runtime/accumulator_chain.rs:74–89` changed in
production: `Drop` takes its link, returns for an empty or shared link, acquires
with `Arc::into_inner`, detaches and immediately releases one node, and calls
the unchanged cold drain only if a tail remains. Attempt 1 used `#[inline]`;
attempt 2 changes only that annotation to `#[inline(always)]`.

The final drain at lines 99–109, its immediate-release premise at lines 91–98,
and the test module at lines 111–280 retain their entry contents. Tests were
never edited. Shared ownership still stops acquisition immediately, live Weak
handles do not prevent acquisition, and no strong-count inspection, worklist,
payload-order preservation, or retained batch was introduced. Auxiliary space
remains O(1), with one acquired node released before acquiring the next.

The source now branches around the cold drain for empty/singleton cases, but
**the stronger compiled-code requirement is not achieved**: generated
`drop_in_place` calls survive, including a call on a known-empty wrapper.
Section 6 includes the actual instructions. The file is 8,908 characters,
including unchanged tests, below the preferred 10k threshold.

## 2. Instruction counts and comparison with brief-489

Base is `HEAD` at `2aaa627367b84d1e8c760959402131dd561b11ef`, without any
brief-489 conversions. I exported it with read-only `git archive` to
`target/brief-490/build-src`; the original dirty working tree was never reverted,
stashed, or cleaned. After measuring base, I copied the entry conversions plus
the helper change into that isolated source export. The list is retained in
`target/brief-490/candidate-overlay.json`.

All three builds used that same source path and the same
`target/brief-490/build-target/release/rum-cli` output/execution path, Rust
1.85.0, and the ordinary release profile. No profiling environment variables
were enabled. No build or test ran concurrently with the measurements. There
are exactly three runs per workload for base and each attempt; no arm was
rerun to reduce its spread. All 27 `cpu_core/instructions/u` rows have `100.00`
multiplex coverage; `cpu_atom` rows are excluded.

The inputs are the original, byte-unchanged `target/brief-489/*.rum` files:
200 ordinary iterations preparing eight evaluated LHS arguments; nested LHS
parent traversal; and preparing/installing eight value-table rules. The first
two print `1` followed by `Null`; the third prints `8` followed by `Null`.
Every base/candidate stdout was checked against those exact bytes.

Spread means maximum minus minimum; percentage spread is spread / mean.
Delta compares each candidate mean with the freshly measured `HEAD` mean.

| Workload | Arm | Run 1 | Run 2 | Run 3 | Mean | Spread | Spread / mean |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| prepared-arguments | base | 17,772,564 | 17,774,523 | 17,773,325 | 17,773,470.667 | 1,959 | 0.011022% |
| prepared-arguments | attempt-1 | 18,180,877 | 18,180,280 | 18,178,708 | 18,179,955.000 | 2,169 | 0.011931% |
| prepared-arguments | attempt-2 | 18,174,473 | 18,169,838 | 18,171,983 | 18,172,098.000 | 4,635 | 0.025506% |
| lhs-parents | base | 13,701,454 | 13,706,135 | 13,699,110 | 13,702,233.000 | 7,025 | 0.051269% |
| lhs-parents | attempt-1 | 13,986,895 | 13,974,656 | 13,975,126 | 13,978,892.333 | 12,239 | 0.087553% |
| lhs-parents | attempt-2 | 13,980,457 | 13,961,333 | 13,962,789 | 13,968,193.000 | 19,124 | 0.136911% |
| value-table-rules | base | 18,684,616 | 18,686,417 | 18,684,031 | 18,685,021.333 | 2,386 | 0.012770% |
| value-table-rules | attempt-1 | 19,354,339 | 19,352,994 | 19,352,999 | 19,353,444.000 | 1,345 | 0.006950% |
| value-table-rules | attempt-2 | 19,353,621 | 19,351,947 | 19,351,691 | 19,352,419.667 | 1,930 | 0.009973% |

| Workload | brief-489 delta | Attempt 1 delta (instructions) | Attempt 1 delta | Attempt 2 delta (instructions) | Attempt 2 delta | Gate |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| prepared-arguments | +0.544563% | +406,484.333 | +2.287028% | +398,627.333 | +2.242822% | Both FAIL; exceed both arm spreads |
| lhs-parents | +1.036030% | +276,659.333 | +2.019082% | +265,960.000 | +1.940998% | Both FAIL; exceed both arm spreads |
| value-table-rules | +0.741052% | +668,422.667 | +3.577318% | +667,398.333 | +3.571836% | Both FAIL; exceed both arm spreads |

Verbatim counted rows below: base, then attempt 1, then attempt 2, three rows each.

`prepared-arguments`:

```text
17772564,,cpu_core/instructions/u,9285208,100.00,,
17774523,,cpu_core/instructions/u,6859311,100.00,,
17773325,,cpu_core/instructions/u,6239891,100.00,,
18180877,,cpu_core/instructions/u,8934997,100.00,,
18180280,,cpu_core/instructions/u,6955601,100.00,,
18178708,,cpu_core/instructions/u,6917738,100.00,,
18174473,,cpu_core/instructions/u,8915233,100.00,,
18169838,,cpu_core/instructions/u,7021982,100.00,,
18171983,,cpu_core/instructions/u,6948153,100.00,,
```

`lhs-parents`:

```text
13701454,,cpu_core/instructions/u,5691781,100.00,,
13706135,,cpu_core/instructions/u,5499499,100.00,,
13699110,,cpu_core/instructions/u,4490216,100.00,,
13986895,,cpu_core/instructions/u,5851001,100.00,,
13974656,,cpu_core/instructions/u,4832086,100.00,,
13975126,,cpu_core/instructions/u,3474609,100.00,,
13980457,,cpu_core/instructions/u,5842646,100.00,,
13961333,,cpu_core/instructions/u,4709996,100.00,,
13962789,,cpu_core/instructions/u,3563115,100.00,,
```

`value-table-rules`:

```text
18684616,,cpu_core/instructions/u,3605264,100.00,,
18686417,,cpu_core/instructions/u,2959638,100.00,,
18684031,,cpu_core/instructions/u,2506661,100.00,,
19354339,,cpu_core/instructions/u,3109021,100.00,,
19352994,,cpu_core/instructions/u,2656857,100.00,,
19352999,,cpu_core/instructions/u,2554325,100.00,,
19353621,,cpu_core/instructions/u,3482517,100.00,,
19351947,,cpu_core/instructions/u,2839999,100.00,,
19351691,,cpu_core/instructions/u,2544682,100.00,,
```

Build command, run for base and each attempt with only the log filename changed:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" CARGO_TARGET_DIR="$PWD/target/brief-490/build-target" python3 scripts/run-bounded.py --timeout 900 -- cargo build --manifest-path target/brief-490/build-src/Cargo.toml -p rum-cli --release --offline --locked > target/brief-490/logs/build-base.log 2>&1
```

Exact measurement commands:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for label in prepared-arguments lhs-parents value-table-rules; do for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/brief-490/build-target/release/rum-cli --file "target/brief-489/${label}.rum" > "target/brief-490/logs/instructions-base-${label}-${run}.stdout" 2> "target/brief-490/logs/instructions-base-${label}-${run}.csv" || exit; done; done'
```

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for label in prepared-arguments lhs-parents value-table-rules; do for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/brief-490/build-target/release/rum-cli --file "target/brief-489/${label}.rum" > "target/brief-490/logs/instructions-attempt-1-${label}-${run}.stdout" 2> "target/brief-490/logs/instructions-attempt-1-${label}-${run}.csv" || exit; done; done'
```

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 300 -- bash -c 'for label in prepared-arguments lhs-parents value-table-rules; do for run in 1 2 3; do taskset -c 2 perf stat -e instructions:u -x, -- target/brief-490/build-target/release/rum-cli --file "target/brief-489/${label}.rum" > "target/brief-490/logs/instructions-attempt-2-${label}-${run}.stdout" 2> "target/brief-490/logs/instructions-attempt-2-${label}-${run}.csv" || exit; done; done'
```

All builds and measurement commands exited 0. Raw logs are under
[logs](../../../target/brief-490/logs); binaries and their SHA-256 digests were
saved per arm. No new allocation-counter claim is made; brief-490's instruction
gate already fails decisively.

The additional `AGENTS.md` timing A/B used the repository's unchanged
`core-language-microbench.sh`, CPU 2, the same release binary path, profiling off,
and in-kernel monotonic `AbsoluteTiming`. It ran once for base and once for the
final candidate, sequentially with no concurrent build/test. All 120 source
labels and iteration counts match. It does not override the instruction gate.
Positive timing observations are disclosed below, including large movements
in untouched controls; no timing rerun or causal claim was made. Every label is
retained in [microbench-comparison.tsv](../../../target/brief-490/microbench-comparison.tsv).

```text
Compared 120 timing labels; generated sources and iteration counts identical.
Untouched controls:
literal-integer	7.9524e-05	8.3609e-05	+5.136814%
literal-string	8.0185e-05	9.1786e-05	+14.467793%
symbol-ownvalue-read	0.000211776	0.000207038	-2.237270%
downvalue-exact-hit	0.000780987	0.000787593	+0.845853%
Ten largest positive deltas:
association-part-key-read-width32000	0.00732148	0.00874546	+19.449346%
association-part-key-read-width4000	0.000966718	0.00114129	+18.058213%
table-count-null	8.1717e-05	9.5886e-05	+17.339109%
literal-string	8.0185e-05	9.1786e-05	+14.467793%
stringtrim-whitespace-32000	0.00313396	0.00356669	+13.807770%
stringtrim-literal-list-32000	0.00316311	0.00359773	+13.740275%
stringtrim-single-literal-32000	0.00314942	0.00358038	+13.683789%
module-chain-escape-large	9.4165e-05	0.000106801	+13.418999%
table-iterator-null	1.8944e-05	2.1347e-05	+12.684755%
appendto-part-outer1000-large	0.00355974	0.00400999	+12.648396%
```

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 240 -- taskset -c 2 bash scripts/core-language-microbench.sh --rum-cli "$PWD/target/brief-490/build-target/release/rum-cli" --output-dir target/brief-490/microbench-base > target/brief-490/logs/microbench-base.log 2>&1
```

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 240 -- taskset -c 2 bash scripts/core-language-microbench.sh --rum-cli "$PWD/target/brief-490/build-target/release/rum-cli" --output-dir target/brief-490/microbench-candidate > target/brief-490/logs/microbench-candidate.log 2>&1
```

## 3. Unchanged certificate: green, falsifier red, hand-restored green

The original five tests remain byte-identical. Exclusive lengths 16 and 64,
the shared-tail case, and live Weak all still reach the cold drain after their
first detached node. They pass because every payload destructor runs after its
node's link has been detached; the fast path does not bypass the drain for
these chains or weaken their assertions.

Initial certificate, exit 0 (`certificate-green.log`):

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::accumulator_chain::tests -- --nocapture --test-threads=1 > target/brief-490/logs/certificate-green.log 2>&1
```

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

The literal requested drain-body falsifier was:

```rust
fn drain<N: AccumulatorChainNode>(link: Option<Arc<N>>) {
    drop(link);
}
```

The fast path stayed installed. This discriminates, but **does not give the
brief's predicted depths 16 and 64**. Inline release detaches one node between
each ordinary recursive release, so the observed depths are 8 and 32. These
are actual assertion failures, not a fatal-process or overflow probe.
Verbatim output, expected exit 101 (`certificate-drain-falsifier-red.log`):

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::accumulator_chain::tests::exclusive_depth -- --nocapture --test-threads=1 > target/brief-490/logs/certificate-drain-falsifier-red.log 2>&1
```

```text
running 2 tests
test runtime::accumulator_chain::tests::exclusive_depth_16 ... exclusive length=16: maximum payload drop depth=8

thread 'runtime::accumulator_chain::tests::exclusive_depth_16' panicked at crates/rum-runtime/src/runtime/accumulator_chain.rs:205:9:
assertion `left == right` failed
  left: 8
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test runtime::accumulator_chain::tests::exclusive_depth_64 ... exclusive length=64: maximum payload drop depth=32

thread 'runtime::accumulator_chain::tests::exclusive_depth_64' panicked at crates/rum-runtime/src/runtime/accumulator_chain.rs:205:9:
assertion `left == right` failed
  left: 32
 right: 1
FAILED

failures:

failures:
    runtime::accumulator_chain::tests::exclusive_depth_16
    runtime::accumulator_chain::tests::exclusive_depth_64

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 991 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

To also demonstrate the requested full-chain readings without editing tests,
I performed one additional, explicitly broader falsifier: while `drain` was
`drop(link)`, temporarily replace the `Drop` body by its original
`drain(self.0.take())`. This removes the new inline detachment too. This is a
mechanism falsifier, not a third performance candidate. Verbatim output,
expected exit 101 (`certificate-full-falsifier-red.log`):

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::accumulator_chain::tests::exclusive_depth -- --nocapture --test-threads=1 > target/brief-490/logs/certificate-full-falsifier-red.log 2>&1
```

```text
running 2 tests
test runtime::accumulator_chain::tests::exclusive_depth_16 ... exclusive length=16: maximum payload drop depth=16

thread 'runtime::accumulator_chain::tests::exclusive_depth_16' panicked at crates/rum-runtime/src/runtime/accumulator_chain.rs:195:9:
assertion `left == right` failed
  left: 16
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test runtime::accumulator_chain::tests::exclusive_depth_64 ... exclusive length=64: maximum payload drop depth=64

thread 'runtime::accumulator_chain::tests::exclusive_depth_64' panicked at crates/rum-runtime/src/runtime/accumulator_chain.rs:195:9:
assertion `left == right` failed
  left: 64
 right: 1
FAILED

failures:

failures:
    runtime::accumulator_chain::tests::exclusive_depth_16
    runtime::accumulator_chain::tests::exclusive_depth_64

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 991 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

Both the fast-path body and the original iterative drain were then restored
by an explicit hand-written patch, never `git checkout`. Verbatim restored
certificate, exit 0 (`certificate-restored-green.log`):

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

After attempt 2 changed only the inlining annotation, the same focused
certificate was run again and passed. Verbatim final output, exit 0
(`certificate-final-green.log`):

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

All green certificate runs use the command above with the corresponding log
filename; both red runs use the exclusive-depth filter shown. Full logs retain
the compiler output, including the existing three unused test-helper warnings.
No `#[cfg(test)]` divergence was introduced in a production body.

## 4. Formatting and workspace verification

`cargo fmt --check` exited 0 both before measurement and on the final shape.
Both format logs are empty; there is no fabricated success stdout.

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check > target/brief-490/logs/fmt-final.log 2>&1
```

**Workspace: 2,458 passed, 0 failed, 57 ignored across 32 groups; exit 0.**
Runtime has 990 passed and 3 ignored; stdlib has 925 passed and 52 ignored.
The final exact ownership-inventory test is included among those passes.

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked > target/brief-490/logs/workspace.log 2>&1
```

Verbatim workspace group/result lines:

```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-071a386fb6152ac7)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/cli.rs (target/debug/deps/cli-527e9f2d0cca7f28)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.48s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-78a8fab085fc98d8)
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 990 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.51s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test tests::ownership_graph::production_inventory_is_exact ... ok
test result: ok. 925 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 8.03s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.94s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The sandbox **cannot bind loopback**. I used the existing
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` path for the workspace and focused native run.
Nine native ZeroMQ tests return early under that explicit skip; libtest counts
those returns as passes. Excluding those nine leaves 2,449 exercised reported
passes, separate from the 57 ignored tests. Native coverage still requires a
run without the skip in a loopback-capable environment.

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-jupyter --offline --locked zeromq::tests::native -- --nocapture --test-threads=1 > target/brief-490/logs/native-skips.log 2>&1
```

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

     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-d780f9d4cfdbc563)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 5. No inventory row moved

All six inventory files remain byte-identical to their entry state. None was
regenerated or edited in the working tree. Brief-489's 13 changed runtime
files (16 converted members), its four already-modified inventory files, and
its untracked report also remain byte-identical. `Cargo.lock`, the derive,
scanner, core crate, suspension traits, and all other pre-existing files retain
their entry hashes. The entry manifest records 1,450 files; the helper is the
only one whose bytes changed. The only new non-build file is this report.

The exact discovery/inventory check passed before measurement, and passed
again as part of the final workspace suite. Verbatim focused output:

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.08s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
EXACT INVENTORY self-reaching types: 57 rows
EXACT INVENTORY UnknownHostPayload: 50 rows
EXACT INVENTORY UnknownConstruct: 7086 rows
test tests::ownership_graph::production_inventory_is_exact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 976 filtered out; finished in 4.75s

     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-44228b396944f9ba)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-490/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::production_inventory_is_exact -- --exact --nocapture > target/brief-490/logs/inventory-exact.log 2>&1
```

## 6. Stop-loss and open questions

Two production attempts were measured, and implementation changes stopped.
Attempt 2 remains in the working tree. Neither satisfies the no-call fast-path
requirement or the instruction gate; no weaker performance interpretation is
used to accept it. Destruction safety and the payload premise remain intact.

The new finding is in generated drop glue. With attempt 1, the wrapper's
`drop_in_place` routines reserve stack and save registers before testing for
an empty link. Attempt 2's `#[inline(always)]` changes code generation but
still leaves callable `drop_in_place<AccumulatorChain<...>>` routines for all
three measured node types. The annotation on the trait's `Drop` method does
not eliminate calls to this compiler-generated glue.

A concrete final-binary example follows. At `40270d` the stack slot is set to
zero, `402715` passes its address, and `402718` calls another wrapper drop.
Later `402744` also calls the current wrapper's drop routine while releasing
the detached node. These are verbatim instruction lines; byte-only continuation
lines are omitted. The complete output remains in the raw assembly log:

```asm
  4026ff:	4c 8b 74 24 70       	mov    0x70(%rsp),%r14
  402704:	48 c7 44 24 70 00 00 	movq   $0x0,0x70(%rsp)
  40270d:	48 c7 04 24 00 00 00 	movq   $0x0,(%rsp)
  402715:	48 89 e7             	mov    %rsp,%rdi
  402718:	e8 33 37 21 00       	call   615e50 <core::ptr::drop_in_place<rum_runtime::runtime::accumulator_chain::AccumulatorChain<rum_runtime::runtime::call_preparation::PreparedArgumentNode>>>
  40271d:	4c 89 74 24 30       	mov    %r14,0x30(%rsp)
  402722:	48 8b 44 24 70       	mov    0x70(%rsp),%rax
  402727:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  40272c:	0f 10 44 24 50       	movups 0x50(%rsp),%xmm0
  402731:	0f 10 4c 24 60       	movups 0x60(%rsp),%xmm1
  402736:	0f 29 4c 24 10       	movaps %xmm1,0x10(%rsp)
  40273b:	0f 29 04 24          	movaps %xmm0,(%rsp)
  40273f:	48 8d 7c 24 20       	lea    0x20(%rsp),%rdi
  402744:	e8 27 ff ff ff       	call   402670 <core::ptr::drop_in_place<rum_runtime::runtime::accumulator_chain::AccumulatorChain<rum_runtime::runtime::call_preparation::PreparedArgumentNode>>>
```

The final wrapper entry also reserves `0x78` bytes and saves two registers
before its empty test; corresponding rule and parent wrappers reserve `0x160`
and `0x1e8` bytes and save registers before testing. The node-release body
contains payload moves/copies as well as these nested drop calls. Full
`nm -C` and `objdump -d -C` output for base and both attempts is retained in
`target/brief-490/logs`.

This identifies concrete work that survives the source-level fast path. It is
not a dynamic per-instruction attribution of every measured delta. The existing
cold-call cause from the brief was accepted; no separate attempt was spent
re-establishing it. The two allowed attempts show that simply moving acquisition
and node release into `Drop`, even with `#[inline(always)]`, does not deliver the
requested compiled fast path on Rust 1.85.0. A different design or additional
scope is a director decision; no such changes were made.

There is also a precise gate wording issue: leaving the new first-node release
installed while changing only `drain` to recursive `drop(link)` yields half-chain
depths, not full-chain depths. The falsifier still discriminates, and the
additional full recursive mutation produces exactly 16 and 64, but the literal
drain-only numeric requirement is not claimed as satisfied. The tests were
preserved throughout. No unsafe, shared-link shortcut, alternative ownership
acquisition, or O(N) retained destruction stack was used to recover instructions.
