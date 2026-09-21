# brief-491 — node-level Drop experiment

One prototype was implemented and measured. It safely preserves `Expr` move-out
semantics and measures **+0.475267% instructions** against
`HEAD`, versus the brief's **+0.567%** wrapper result. Recommend node-level
`Drop` as the family design direction, with the member-by-member limitations
below. **The working tree is not green:** the required workspace command fails
the protected ownership inventory. The timing panel also has unresolved
regressions. This report does not claim acceptance.

## 1. Prototype and move-out resolution

The only changed production file is
`crates/rum-runtime/src/runtime/call_preparation.rs`:

- Lines 1–3 and 40–51 remove the wrapper import, accessor derive, and link
  attribute for this member. Both holder `tail` and node `previous` are plain
  `Option<Arc<PreparedArgumentNode>>`; `value` remains an `Expr`.
- Lines 53–74 put `Drop` on the node. It detaches its predecessor and invokes
  the local, generic `drain_prepared_argument_nodes` loop. The accessor exists
  so synthetic payloads can observe this exact production loop. No observer,
  counter, or test-specific branch exists in production.
- Lines 76–95 restore the holder's plain optional link construction. There is
  still one `Arc` allocation per appended node, with the existing prefix/link
  clones; neither field grows and no payload is wrapped in `Option`.
- Lines 97–143 preserve both existing completion policies. At lines 104–108
  and 124–128, an exclusively acquired node yields its original payload via
  `std::mem::replace(&mut current.value, Expr::small_integer(0))` and its
  predecessor via `current.previous.take()`. The shared branch still clones
  the original value and link. The uniqueness panic is unchanged.
- Lines 496–691 add the prototype's six tests. No immediate call-site edit
  was necessary; the other fifteen conversions retain their entry bytes.

`Expr::small_integer(0)` constructs `Atom::Integer(IntValue::Small(0))`
(`crates/rum-core/src/expr.rs:342–344,785–787`). It needs no allocation, symbol
interning, reference-count operation, or payload clone. The replacement is
observable only inside a uniquely owned, consumed node; the original value
moves into the result vector unchanged. Source-level costs are an inert
replacement value, clearing the predecessor, and running the emptied node's
`Drop` and field drop glue. This experiment measures the resulting binary; it
does not claim LLVM eliminates those obligations or isolate their instruction
cost from the other fifteen wrappers.

The destructor acquires each predecessor with `Arc::into_inner`, stops on a
shared node, and leaves the surviving holder responsible. Live weak handles
do not prevent acquisition. Each acquired node's predecessor is detached
before it is released, and it is released before acquiring another node.
There is no worklist, retained batch, or chain-proportional recursion: an
acquired node's own `Drop` only sees an empty predecessor. Auxiliary space is
O(1). The outer node's own payload is released when its destructor returns;
no recursive payload-order promise is made, consistent with the existing
pure-memory-reclamation premise for `Expr`.

The file is 21,901 characters, comprising a 12,555-character production prefix
and 9,346 characters of tests/trace assertion. It remains cohesive for this
scoped experiment. A future organization change can separate the prepared
argument accumulator and its tests from argument planning and metadata;
this brief forbids adding those modules or changing `TODO.md`.

## 2. Measurement

Base is `8c4d7d364fa5182e19a769afe546b73c4bbba9dc` without brief-489's conversions. A read-only `git archive`
exported it to `target/brief-491/build-src`; the dirty working tree was never
reverted, stashed, or cleaned. After base measurement, the 17 changed entry
files (13 source files and four existing inventory files), including the
prototype, were overlaid into that export. The overlay manifest is
`target/brief-491/candidate-overlay.json`.

Both arms used the same source path, same ordinary release profile, same
Rust 1.85.0 toolchain, and the same build/output/execution path:
`target/brief-491/build-target/release/rum-cli`. Builds used:

```bash
TMPDIR="$PWD/target/brief-491/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never CARGO_TARGET_DIR="$PWD/target/brief-491/build-target" python3 scripts/run-bounded.py --timeout 1200 -- cargo build --manifest-path target/brief-491/build-src/Cargo.toml --release -p rum-cli --offline --locked
```
There were three executions per arm, under a 180-second outer bounded runner,
with the following inner command and stdin closed:

```bash
taskset -c 2 perf stat -e instructions:u -x, -- /home/remi/dev/rum/target/brief-491/build-target/release/rum-cli target/brief-489/prepared-arguments.rum < /dev/null
```
No profiling environment variable was enabled, and no build/test or other
observed CPU-heavy workload ran concurrently with either measurement arm.
The original workload was unchanged (SHA-256
`1078f430b848b725347da336a5a99d9c86bda873b9299c5e7c2ef0bf28e930c6`).
Every run exited 0 and printed exactly `1\nNull\n`.
All six `cpu_core/instructions/u` rows had `100.00` multiplex coverage;
`cpu_atom` rows are not instruction measurements.

Spread is max minus min, percentage spread is spread / mean, and delta is
candidate mean / base mean minus one.

| Arm | Run 1 | Run 2 | Run 3 | Mean | Spread | Spread / mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| base | 17,775,958 | 17,770,656 | 17,775,791 | 17,774,135.000 | 5,302 | 0.029830% |
| candidate | 17,858,216 | 17,857,220 | 17,860,393 | 17,858,609.667 | 3,173 | 0.017767% |

The candidate adds 84,474.667
instructions on average: **+0.475267%**. That is
**0.091733 percentage points below** the
brief's +0.567% wrapper result. The improvement exceeds the observed arm
spreads, but it does not establish zero overhead. The wrapper number is the
director's supplied premise, not a newly measured third arm. The remaining
fifteen wrappers prevent attributing all residual overhead to this node.

Verbatim perf output, base runs 1–3 followed by candidate runs 1–3:

base, run 1:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
17775958,,cpu_core/instructions/u,9299375,100.00,,
```
base, run 2:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
17770656,,cpu_core/instructions/u,6381467,100.00,,
```
base, run 3:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
17775791,,cpu_core/instructions/u,6495779,100.00,,
```
candidate, run 1:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
17858216,,cpu_core/instructions/u,8641456,100.00,,
```
candidate, run 2:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
17857220,,cpu_core/instructions/u,6560876,100.00,,
```
candidate, run 3:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
17860393,,cpu_core/instructions/u,6539159,100.00,,
```
The additional required timing A/B used the repository suite once per arm,
with default scale/repetition settings, the same binary path, CPU 2, a
240-second inner timeout, profiling off, and no competing build/test. Both
suite commands exited 0. Both generated `.rum` sources and every per-label
iteration count are identical. Decisions use the in-kernel `AbsoluteTiming`
rows, not the script's wall-clock duration.

```bash
TMPDIR="$PWD/target/brief-491/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 300 -- taskset -c 2 bash scripts/core-language-microbench.sh --output-dir target/brief-491/microbench-base --timeout 240 --rum-cli "$PWD/target/brief-491/build-target/release/rum-cli"
TMPDIR="$PWD/target/brief-491/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 300 -- taskset -c 2 bash scripts/core-language-microbench.sh --output-dir target/brief-491/microbench-candidate --timeout 240 --rum-cli "$PWD/target/brief-491/build-target/release/rum-cli"
```
Untouched controls are `literal-integer`, `literal-string`,
`symbol-ownvalue-read`, and `downvalue-exact-hit`. Their changes span
−4.557233% to +5.757506%. Definition/call neighbors generally stay inside
that observed control range, but `appendto-list-incremental-small`
(+42.039116%), its large neighbor (+12.943667%), the two association key-read
labels (+14.099815%, +11.499648%), and `replacepart-wide-rule-list`
(+7.164964%) exceed the positive control movement. These are unresolved
observed regressions, not a clean timing gate and not evidence that the new
node alone causes them. No tuning or repeat A/B was attempted.

All 120 rows are retained in `target/brief-491/microbench-comparison.tsv`;
both original `timings.tsv` files and raw output are beside it. Verbatim
comparison output, including the controls before the neighboring labels:

```text
Compared 120 timing labels; generated sources and iteration counts identical.
Untouched controls:
literal-integer	7.8732e-05	8.3265e-05	+5.757506%
literal-string	9.0211e-05	8.9767e-05	-0.492179%
symbol-ownvalue-read	0.00021456	0.000204782	-4.557233%
downvalue-exact-hit	0.000794639	0.000795335	+0.087587%
Definition/call/hold/with labels:
set-ownvalue	0.000254976	0.000258909	+1.542498%
with-local	0.000481972	0.000484387	+0.501066%
with-deep-body-small	0.0018101	0.00176105	-2.709795%
downvalue-exact-miss	0.000553753	0.000564999	+2.030869%
downvalue-pattern-hit	0.000963798	0.000961879	-0.199108%
downvalue-pattern-miss	0.000287342	0.000291151	+1.325598%
downvalue-exact-miss-repeat	0.00125546	0.00125943	+0.316219%
downvalue-pattern-miss-repeat	0.00130922	0.00129	-1.468050%
downvalue-guarded-hit	0.00189335	0.00188697	-0.336969%
downvalue-guarded-decline-fallback	0.0014346	0.00142335	-0.784191%
upvalue-guarded-hit	0.00234953	0.00233088	-0.793776%
subvalue-unguarded-hit	0.00140861	0.00139461	-0.993888%
subvalue-unguarded-miss	0.000779009	0.000778283	-0.093195%
subvalue-guarded-hit	0.00245207	0.00243866	-0.546885%
association-set-key-incremental-small	0.00152046	0.00151866	-0.118385%
association-set-key-incremental-large	0.00567916	0.00560169	-1.364110%
set-part-fixed100-len100	0.000136139	0.000130248	-4.327195%
set-part-fixed100-len800	0.000491858	0.000477529	-2.913239%
set-part-fill-small	0.000386056	0.000368837	-4.460234%
set-part-fill-large	0.00116918	0.00113967	-2.523991%
upvalue-candidates-tiny	0.00208153	0.00191071	-8.206464%
upvalue-candidates-wide-small	0.000151772	0.00014878	-1.971378%
upvalue-candidates-wide-large	0.000304226	0.000294349	-3.246600%
sort-callback-small	0.000175781	0.000164762	-6.268596%
sort-callback-large	0.000400775	0.000376849	-5.969933%
sortby-callback-small	0.000203352	0.000197389	-2.932354%
sortby-callback-large	0.000456508	0.000442526	-3.062816%
reversesort-callback-small	0.000157398	0.000155972	-0.905984%
reversesort-callback-large	0.000353623	0.00034876	-1.375193%
keysort-callback-small	8.6574e-05	8.1321e-05	-6.067642%
keysort-callback-large	0.000190256	0.0001801	-5.338071%
keysortby-callback-small	4.49e-05	4.4969e-05	+0.153675%
keysortby-callback-large	9.6143e-05	9.7722e-05	+1.642345%
Ten largest positive deltas:
appendto-list-incremental-small	0.000415582	0.000590289	+42.039116%
association-part-key-read-width4000	0.000985786	0.00112478	+14.099815%
appendto-list-incremental-large	0.00151634	0.00171261	+12.943667%
association-part-key-read-width32000	0.00777476	0.00866883	+11.499648%
replacepart-wide-rule-list	0.000277224	0.000297087	+7.164964%
literal-integer	7.8732e-05	8.3265e-05	+5.757506%
module-retained-gensym-gc-batch	6.8098e-05	7.0244e-05	+3.151341%
part-list	0.00107187	0.0011022	+2.829634%
length-list	0.000905341	0.000930759	+2.807561%
replaceall-simple	0.000345474	0.000355102	+2.786896%
```
## 3. Depth certificate and falsifier

The observer is entirely in `call_preparation.rs`'s `node_drop_tests` module.
It tests the exact generic loop called by the production node destructor,
using synthetic nodes whose payloads observe active destruction nesting.
Moving any undetached predecessor into the test payload makes ordinary
recursive release nest those observers. The production node is not changed
under `cfg(test)`. This is a certificate of the shared drain mechanism, not
a claim that the synthetic payload is an `Expr` or that all family members
have been converted.

The exclusive tests assert depth exactly 1 at lengths 16 and 64, exactly-once
node/payload destruction, and release of each previous acquisition before
the next. Shared-tail and live-weak cases exercise the same loop. Separate
production-node weak-handle checks verify exclusive-prefix release and later
release by the surviving holder at both lengths. Consumption checks cover
unique `into_vec`, unique/shared `into_vec_allow_shared`, result order, and
preservation of the surviving holder's payloads.

The first compile succeeded and all six tests passed. To falsify the
mechanism, I temporarily replaced only the production helper body with:

```rust
let _ = take_previous;
drop(link);
```
Its now-unused `mut` parameter binding was removed during this mutation.
There was no alternate `cfg(test)` production body. Only the two short depth
tests were run, producing normal assertion failures rather than a crash or
stack-overflow probe:

```bash
TMPDIR="$PWD/target/brief-491/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::call_preparation::node_drop_tests::exclusive_depth_ -- --nocapture --test-threads=1
```
```text
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 2 tests
test runtime::call_preparation::node_drop_tests::exclusive_depth_16 ... node drain exclusive length=16: maximum payload drop depth=16

thread 'runtime::call_preparation::node_drop_tests::exclusive_depth_16' panicked at crates/rum-runtime/src/runtime/call_preparation.rs:598:9:
assertion `left == right` failed
  left: 16
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test runtime::call_preparation::node_drop_tests::exclusive_depth_64 ... node drain exclusive length=64: maximum payload drop depth=64

thread 'runtime::call_preparation::node_drop_tests::exclusive_depth_64' panicked at crates/rum-runtime/src/runtime/call_preparation.rs:598:9:
assertion `left == right` failed
  left: 64
 right: 1
FAILED

failures:

failures:
    runtime::call_preparation::node_drop_tests::exclusive_depth_16
    runtime::call_preparation::node_drop_tests::exclusive_depth_64

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 997 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```
Exit code: 101. The loop and `mut` binding were then restored by an explicit
patch, and the file was checked byte-for-byte against the pre-falsifier
snapshot and the already-built candidate source. Restored command:

```bash
TMPDIR="$PWD/target/brief-491/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked runtime::call_preparation::node_drop_tests -- --nocapture --test-threads=1
```
```text
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 6 tests
test runtime::call_preparation::node_drop_tests::consumption_preserves_payloads_and_order ... ok
test runtime::call_preparation::node_drop_tests::exclusive_depth_16 ... node drain exclusive length=16: maximum payload drop depth=1
ok
test runtime::call_preparation::node_drop_tests::exclusive_depth_64 ... node drain exclusive length=64: maximum payload drop depth=1
ok
test runtime::call_preparation::node_drop_tests::live_weak_does_not_prevent_drain ... node drain live Weak length=16: maximum payload drop depth=1
ok
test runtime::call_preparation::node_drop_tests::production_node_releases_exclusive_prefix_then_surviving_tail ... ok
test runtime::call_preparation::node_drop_tests::shared_tail_stops_then_drains ... node drain shared first prefix: 8 nodes/payloads, maximum payload drop depth=1
node drain shared second prefix + tail: 24 nodes/payloads, maximum payload drop depth=1
ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 993 filtered out; finished in 0.00s
```
Exit code: 0. Full compiler output, including the existing test-builtin
unused-function warnings, is retained in the certificate logs under
`target/brief-491/logs`.

## 4. Formatting, workspace total, and protected inventory

The required formatting command ran with the repository-local `TMPDIR`,
`CARGO_NET_OFFLINE=true`, and `CARGO_TERM_COLOR=never`:

```bash
python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check
```
Exit code: **0**. It emitted no stdout or stderr (`logs/fmt.log` is empty).

The required workspace command was run exactly with the same environment
and `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`:

```bash
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```
Exit code: **101**. Before Cargo's fail-fast stop, totals were **2,258 passed,
1 failed, 57 ignored**. The only failure was the changed ownership-inventory
row. Verbatim failure excerpt:

```text
---- tests::ownership_graph::production_inventory_is_exact stdout ----

thread 'tests::ownership_graph::production_inventory_is_exact' panicked at crates/rum-stdlib/src/tests/ownership_graph/mod.rs:70:37:
self-reaching types inventory mismatch:
missing inventory row: rum_runtime::runtime::call_preparation::PreparedArgumentNode	crates/rum-runtime/src/runtime/call_preparation.rs	48	rum_runtime::runtime::call_preparation::PreparedArgumentNode.previous/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::call_preparation::PreparedArgumentNode
stale inventory row: rum_runtime::runtime::call_preparation::PreparedArgumentNode	crates/rum-runtime/src/runtime/call_preparation.rs	50	rum_runtime::runtime::call_preparation::PreparedArgumentNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::call_preparation::PreparedArgumentNode


failures:
    tests::ownership_graph::production_inventory_is_exact

test result: FAILED. 924 passed; 1 failed; 52 ignored; 0 measured; 0 filtered out; finished in 7.46s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```
To finish the remaining workspace targets without suppressing or changing
the failing test, I then ran:

```bash
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked --no-fail-fast
```
Exit code: **101**. Complete workspace totals, including doctests, are
**2,463 passed, 1 failed, 57 ignored; 0 measured, 0 filtered**. `rum-runtime`
passes 996 tests with 3 existing ignored tests. All targets other than the
`rum-stdlib` inventory check pass. The workspace-green requirement is **not
met**, and no inventory skip or weakened assertion was used.

Verbatim per-target result lines and failure status from the complete run:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.76s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 996 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.10s
test result: FAILED. 924 passed; 1 failed; 52 ignored; 0 measured; 0 filtered out; finished in 7.57s
error: test failed, to rerun pass `-p rum-stdlib --lib`
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.72s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
    `-p rum-stdlib --lib`
```
Loopback availability, verbatim:

```text
Loopback socket creation/binding denied: [Errno 1] Operation not permitted; using RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1.
```
The first availability check also failed at socket creation, before entering
its bind-error handler. The subsequent check above catches that same denial.
The existing native ZeroMQ capability branch permits nine socket test bodies
to return early under `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`. Those early returns
are included in Rust's passed count; they are not the 57 ignored tests. The
socket round trips were not exercised. No adapter or skip logic was changed.

Because inventories are explicitly outside scope, discovery was run
read-only to enumerate all changes, rather than updating them:

```bash
TMPDIR="$PWD/target/brief-491/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --offline --locked tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```
The discovery test passes. Compared with the untouched entry inventories,
there are 14 missing and 14 stale rows, all in `call_preparation.rs` (one
self-reaching row and 13 unknown-construct rows on each side). Besides line
moves, the node loses its wrapper edge/derive/attribute and gains the drain
accessor closure. Raw discovery is in `logs/inventory-discovery.log`;
verbatim row comparison output follows:

```text
Missing rows: 14; stale rows: 14.
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	102	33	closure captures: |_| (capture types/modes unresolved)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	102	37	unexpanded macro panic! (ends at 102:91)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	226	33	closure captures: |item| (capture types/modes unresolved)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	26	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	285	46	closure captures: |items| (capture types/modes unresolved)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	304	18	closure captures: |arg| (capture types/modes unresolved)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	312	18	closure captures: |pair| (capture types/modes unresolved)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	33	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , FrameRootTrace)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	398	36	closure captures: |arg| (capture types/modes unresolved)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	40	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	47	1	unresolved attribute/expansion derive (FrameRootTrace)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	5	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , FrameRootTrace)
missing: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	55	61	closure captures: |node| (capture types/modes unresolved)
missing: rum_runtime::runtime::call_preparation::PreparedArgumentNode	crates/rum-runtime/src/runtime/call_preparation.rs	48	rum_runtime::runtime::call_preparation::PreparedArgumentNode.previous/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::call_preparation::PreparedArgumentNode
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	199	33	closure captures: |item| (capture types/modes unresolved)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	258	46	closure captures: |items| (capture types/modes unresolved)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	277	18	closure captures: |arg| (capture types/modes unresolved)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	28	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	285	18	closure captures: |pair| (capture types/modes unresolved)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	35	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , FrameRootTrace)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	371	36	closure captures: |arg| (capture types/modes unresolved)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	42	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	49	1	unresolved attribute/expansion derive (FrameRootTrace , AccumulatorChainNode)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	51	5	unresolved attribute/expansion accumulator_link
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	7	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , FrameRootTrace)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	82	33	closure captures: |_| (capture types/modes unresolved)
stale: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	82	37	unexpanded macro panic! (ends at 82:91)
stale: rum_runtime::runtime::call_preparation::PreparedArgumentNode	crates/rum-runtime/src/runtime/call_preparation.rs	50	rum_runtime::runtime::call_preparation::PreparedArgumentNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::call_preparation::PreparedArgumentNode
```
No production edit was made to satisfy the inventory check. The explicit
instruction to leave inventories unchanged was honored; a green workspace
cannot be claimed with that exact inventory and this field-type change.

## 5. The uniqueness panic

**A suspension temporarily shares the accumulator, but this panicking
conversion is deferred until the retained originating frame has been
released. Source reasoning does not identify the brief-487 defect here.**

`PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame::resume` and
`ResumeSuspendedCallArgumentPreparationSuspensionFrame::resume` clone their
state (`eval_call_prepare/resume_frames.rs:19,49`). This can share an existing
persistent tail during resumed evaluation. On successful preparation, both
call `finish_prepared_call_action`, which returns `FrameAction::Finalize`
containing `FinishPreparedCallArgumentsFinalizer` (lines 64–82); it does not
call `into_vec` during `resume`.

The driver in `suspension/core.rs:614–631` receives the action, pops the
completed frame, explicitly drops it, and only then calls `finalizer.finish()`.
The dispatch `Arc<Self>` has also been consumed by the completed `resume`
call. The finalizer invokes `finish_suspended_prepared_call`, whose
`state.evaluated_args.into_vec()` at `eval_call_prepare.rs:122–123` is the
production caller of the panicking method. On another input suspension,
the driver similarly drops the old frame before propagating the nested
suspension. Thus the retained-frame owner cited in the question is gone before
this conversion, including after repeated suspensions. This ordering is also
the documented `PureFrameFinalizer` contract at `suspension/core.rs:84–87`.

Definition-LHS preparation has its own consumption sites and already uses
`into_vec_allow_shared` at `definitions/definition_lhs_preparation.rs:604,683`.
This finding does not establish uniqueness for arbitrary accumulator clones
or suggest replacing that shared-safe API.

The existing tests cover both preserved prefix arguments and two input
suspensions without restarting. Verbatim results from the required workspace
run:

```text
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_suspend_multiple_times_without_restarting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments ... ok
```
No panic probe or repair was added for this adjacent question.

## 6. Recommendation and handoff

**Recommend node-level `Drop` as the family design direction.** For this
member, safe payload extraction exists, preserves the value semantics, and
needs no allocation or clone on the unique path. The instruction result is
measurably lower than the supplied wrapper result by 0.091733 percentage
points. The measured gain and safe move-out support pursuing this shape;
implementation effort is not a soundness argument for keeping the wrapper.

This is a design recommendation, with concrete limits: fifteen wrappers
remain in the measured candidate, so the remaining +0.475267% does not show
that a full node-level family would have the same residual cost. Conversely,
this one member does not prove an inert replacement exists for every other
payload or that a full conversion would be free. Each member's move-out
semantics and a family A/B still need their own gate. The workspace inventory
failure and the timing-panel regressions also prevent accepting the present
prototype as a green change. No second prototype or tuning attempt was made.

The prototype is left uncommitted for review. At handoff, entry-content
hashes show that all 1,451 other tracked/untracked entry files are unchanged,
including the other fifteen members, `accumulator_chain.rs`, all inventories,
and the prior reports. The restored source and measured candidate source
are byte-identical. Only `call_preparation.rs` and this report differ from
entry outside ignored `target/brief-491` artifacts.

No git mutation command, network fetch, unsafe addition, `ManuallyDrop`,
production/test implementation divergence, fatal-process probe, or new
out-of-repository artifact was introduced. All experiment sources, exported
build trees, temporary directories, binaries, logs, and comparisons are
inside `target/brief-491`; Cargo builds/tests were offline and locked, and
long-running/child-spawning commands used the bounded runner.
