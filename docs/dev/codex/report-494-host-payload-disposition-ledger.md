# brief-494 — host payload disposition ledger

## 1. Live discovery count

Derived by the existing `discovery_report_without_inventory_input` test, which
calls `graph::discover(source::production())`. The existing test binary was run
before implementing the ledger; no pinned inventory was used as its input.
The director’s **50 / 6 / 44** count matches. Discovery stdout excerpts:

```text
UnknownHostPayload count: 50
UnknownHostPayload breakdown: 6 storage edges; 44 public input sites
```

The ledger records **3 Declared, 3 Unbounded, and 44 PublicInput** entries.

## 2. Pinned ledger schema and real entries

`crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv`
has nine tab-separated columns: `kind`, `file`, `line`, `column`, `construct`,
`disposition`, `witness`, `owed_by`, `external_implementation_coverage`.
The first five identify an exact live discovery row, including its complete
ownership/signature trail. Every entry has a checked witness. Unbounded entries
must name `TODO.md#standing-engineering-debt` and explicitly say `unresolved`
for external implementation coverage; those columns are `-` for other entries.
Unknown dispositions, malformed rows, duplicates, and witness drift fail.

Three verbatim entries:

```tsv
UnknownHostPayload	crates/rum-runtime/src/runtime/file_io.rs	95	12	rum_runtime::runtime::file_io::FileIoProvider.inner/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn FileIo	Declared	rum_runtime::runtime::file_io::FileIo: rum_runtime::runtime::host_payload::BoundedDestruction (compiler-checked supertrait)	-	-
UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	17	13	rum_runtime::runtime::suspension::core::SuspendedEvaluation.frames/std::collections::VecDeque[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame	Unbounded	rum_runtime::runtime::suspension::core::SuspensionFrame => rum_runtime::runtime::attributes::EvalUpdateAttributesSuspensionFrame (impl at crates/rum-runtime/src/runtime/attributes.rs:234:1); rum_runtime::runtime::attributes::EvalUpdateAttributesSuspensionFrame.args -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
UnknownHostPayload	crates/rum-core/src/expr.rs	109	25	public input rum_core::expr::impl(Symbol)@108::new(context): opaque owned payload impl AsRef < str >	PublicInput	public by-value input; owning storage sites are separate rows	-	-
```

## 3. Mechanical checks and their limits

1. Completeness compares live discovered `UnknownHostPayload` rows against the
   ledger, naming every missing row. No count, wildcard, or pinned discovery
   inventory substitutes for discovery.
2. No staleness compares every ledger identity against live discovery and names
   entries whose rows have disappeared. Duplicate entries also fail.
3. Narrow attribution re-walks the exact field’s normalized storage shape,
   resolves its opaque trait (including the actual non-generic alias scope),
   resolves a concrete implementation of that same trait, and follows structural
   owning edges to the canonical `rum_core::expr::{Expr, SharedExpr, SharedArgs}`
   identities. The ledger pins the implementation’s location and every field
   edge in the resulting witness. An enclosing type’s sibling fields, an
   unrelated trait implementation, or an unrelated type named `Expr` cannot
   supply that witness.

`Declared` additionally compiles generic proofs that every `FileIo`,
`EntropySource`, and `PrintSink` implementor satisfies the existing destruction
bound. Source resolution ties the compiled public exports to the exact trait
in the storage row. No marker implementation or supertrait was added.
The declaration’s truth remains the implementor’s responsibility.

The two suspension frame rows reach `SharedArgs` through
`EvalUpdateAttributesSuspensionFrame.args`. The finalizer row reaches it through
`FinishReapBucketsFinalizer.rendered -> ReapRenderAccumulator.prefix`.
The cited standing-debt heading and its expression-drop statement are also
checked for presence. This is a citation check, not a fresh measurement of the
debt; no overflow, crash, or fatal-process probe was run.

**Limits:** the Unbounded proof is existential: the exact opaque slot can own
an expression-bearing implementation. It does not prove that every implementation
contains an expression, that all other fields are covered, or that arbitrary
external code has bounded destruction. Each of the three entries keeps
`external_implementation_coverage: unresolved`. Closing that would require
coverage of downstream implementations and their owned/captured state, not a
claim about an internal witness. Conditional/expanded/foreign representations
remain limited by the discovery model and the separate UnknownConstruct axis.

Generic implementation obligations are not treated as verified witnesses;
they need a concrete instantiation and obligation proof. Generic aliases at an
opaque site fail closed because resolving a substituted trait requires lexical
provenance that discovery’s rendered shape does not retain. Multiple trait
bounds at an opaque storage site also require extending that proof. None of the
six production storage rows needs these unsupported cases.

`PublicInput` mechanically means an explicit public by-value signature site,
not an owning storage row. It is **not** an interprocedural non-retention proof:
constructors/setters can forward the supplied value into separately recorded
storage, and generic storage/captures can require additional analysis. Proving
that a particular value is never retained would require body/escape analysis,
including generic and external callees. The ledger deliberately does not claim
that this has been checked.

## 4. Required falsifiers

All mutations were confined to test code and the new ledger. The source fixtures
alter the in-memory `Source` returned by production discovery, so no production
file was ever edited. Scratchpad byte copies restored the test files/ledger;
no git checkout, stash, or index restoration was used. Every run below uses:

```sh
CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/.brief-494-scratch python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-stdlib --lib --offline --locked tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed -- --exact --nocapture
```

### 1. New payload without an entry

The gate used `graph::discover(tests::with_unrelated_payload(source::production()))` with the unchanged pinned ledger. The fixture appends `unrelated_payload: fixture::OtherChain<Box<dyn fixture::OtherPayload>>` to the actual `SuspendedEvaluation` source definition. `OtherChain<T>` owns `Option<Box<Self>>` and `T`; the separate `OtherPayload` implementor `OtherLink` also owns `Option<Box<Self>>`. The gate was then restored from its scratchpad copy, removing the extra field from its discovery input.


Red, exit 101; complete stdout/stderr verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.21s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test

thread 'tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs:145:57:
check 1: missing ledger entry: UnknownHostPayload	test-side-suspension-copy.rs	1	1	rum_runtime::runtime::suspension::core::SuspendedEvaluation.unrelated_payload/fixture::OtherChain<arg0> via payload/std::boxed::Box[Own]/arg0: opaque owned payload dyn fixture :: OtherPayload
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... FAILED

failures:

failures:
    tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 982 filtered out; finished in 5.29s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```


Green, exit 0; complete stdout/stderr verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.32s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
HOST PAYLOAD LEDGER: 50 rows; 6 storage edges (3 Declared, 3 Unbounded); 44 PublicInput
checks 1/2/3: complete, no stale entries, field-specific expression attribution verified
external_implementation_coverage: unresolved for every Unbounded entry
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 982 filtered out; finished in 5.04s

```

### 2. Entry whose row has disappeared

The gate cleared `EntropyProvider`’s field list in its in-memory source snapshot while leaving its existing ledger entry present. Restoring the gate’s scratchpad copy restored the field to discovery.


Red, exit 101; complete stdout/stderr verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.72s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test

thread 'tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs:147:57:
check 2: stale ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/random.rs	147	12	rum_runtime::runtime::random::EntropyProvider.inner/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn EntropySource
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... FAILED

failures:

failures:
    tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 982 filtered out; finished in 4.94s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```


Green, exit 0; complete stdout/stderr verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 12.75s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
HOST PAYLOAD LEDGER: 50 rows; 6 storage edges (3 Declared, 3 Unbounded); 44 PublicInput
checks 1/2/3: complete, no stale entries, field-specific expression attribution verified
external_implementation_coverage: unresolved for every Unbounded entry
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 982 filtered out; finished in 5.08s

```

### 3. Unrelated recursive ownership falsely charged to expression debt

The same additional suspension field and both recursive chain types were reintroduced into the gate’s source snapshot. Its newly discovered row was appended to the pinned ledger with `Unbounded`, the expression debt citation, `external_implementation_coverage: unresolved`, and a copied real suspension-frame expression witness. Check 3 rejected the unrelated trait’s ownership; checks 1 and 2 had matching row sets. The source mutation and false ledger entry were both removed by restoring the scratchpad copies before the green run.


Red, exit 101; complete stdout/stderr verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.73s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test

thread 'tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs:145:57:
check 3: opaque ownership via fixture::OtherPayload has no verified path to Expr/SharedExpr/SharedArgs; cannot attribute Expr debt
row: UnknownHostPayload	test-side-suspension-copy.rs	1	1	rum_runtime::runtime::suspension::core::SuspendedEvaluation.unrelated_payload/fixture::OtherChain<arg0> via payload/std::boxed::Box[Own]/arg0: opaque owned payload dyn fixture :: OtherPayload
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... FAILED

failures:

failures:
    tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 982 filtered out; finished in 5.00s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```


Green, exit 0; complete stdout/stderr verbatim:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.06s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
HOST PAYLOAD LEDGER: 50 rows; 6 storage edges (3 Declared, 3 Unbounded); 44 PublicInput
checks 1/2/3: complete, no stale entries, field-specific expression attribution verified
external_implementation_coverage: unresolved for every Unbounded entry
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 982 filtered out; finished in 5.02s

```


## 5. Required verification

The required formatting command exited **0**, with empty stdout/stderr:

```sh
CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/crates/rum-stdlib/src/tests/ownership_graph/.brief-494-scratch python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check
```

The first required workspace run exited 101. I had put `TMPDIR` under the test
source tree to keep all temporary files in the repository. The existing
`suspension_protocol_repository_invariants_hold` gate scans every file under
`crates`, while other runtime tests create and remove temporary files there;
that caused a read-after-removal race. No test or production code was changed
to fix it. Its complete failure section follows verbatim:

```text
failures:

---- runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold stdout ----

thread 'runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold' panicked at crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:122:47:
source gate should read Rust source: Os { code: 2, kind: NotFound, message: "No such file or directory" }


failures:
    runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold

test result: FAILED. 989 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.51s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

Moving `TMPDIR` to the repository’s ignored `target` directory fixed the setup.
The previously failing gate was then rerun alone and passed:

```sh
CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/target python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --lib --offline --locked runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold -- --exact --nocapture
```

```text
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 992 filtered out; finished in 1.43s
```

The required workspace command then exited **0**:

```sh
CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/target RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

Summing the 32 test-target summaries gives **2,464 passed, 0 failed, 57 ignored**
(2,454 unit/integration tests and 10 doctests passed). The 57 ignored tests are
pre-existing: 2 host, 3 runtime, and 52 stdlib. Native ZeroMQ’s guarded early
returns are separate from Rust’s ignored count. The sandbox’s loopback probe
reported, verbatim:

```text
Loopback bind unavailable: [Errno 1] Operation not permitted
```

The skip used was `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, implemented by
`crates/rum-jupyter/src/zeromq_native_test_support.rs::loopback_bind_available`.
No elevation or network access was requested.

Ownership-gate result lines, verbatim:

```text
test tests::ownership_graph::host_payload_ledger::ownership::attribution_resolves_alias_scopes_and_excludes_nonowning_paths ... ok
test tests::ownership_graph::production_inventory_is_exact ... ok
test tests::ownership_graph::host_payload_ledger::tests::metadata_duplicates_and_witnesses_are_enforced ... ok
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... ok
test tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected ... ok
```

Every workspace target and its result line, verbatim (intervening individual
test lines omitted):

```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-071a386fb6152ac7)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/cli.rs (target/debug/deps/cli-527e9f2d0cca7f28)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.95s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
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
test result: ok. 990 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.99s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 931 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 10.95s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.47s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```



## 6. Scope and inventory disclosure

The final source diff contains only the new test module
`host_payload_ledger.rs`, its `ownership.rs` and `tests.rs` children, and the
`ownership_graph/mod.rs` wiring/documentation. The remaining additions are the
new pinned ledger and this report. No production file changed. No evaluation,
construction, destruction, dispatch, or matcher path changed, so no A/B was run.
This was checked against the actual diff and the complete untracked-file list.

All six pre-existing discovery TSV files are byte-for-byte unchanged; the
workspace `production_inventory_is_exact` test is green. No row was added,
removed, reclassified, or moved, including position drift. Inventory disclosure:
(a) expected content changes: **none**; (b) other content changes: **none**.
The new 50-entry ledger is separate disposition data derived from live output,
not a replacement discovery inventory.

The three new Rust files are 6,668, 10,289, and 4,776 bytes respectively. All are
below the 15k test-file target; the ownership module includes its cohesive
adversarial source fixture. Its non-test helper implementation is below 10k.
Scratchpad copies and logs were removed after their required output was pasted
here. Build/test temporary files stay in the ignored repository `target` tree.

No commit, branch, push, stash, checkout, git configuration change, production
`cfg(test)` divergence, marker implementation, new unsafe code, or fatal-process
probe was made. All Cargo tests used `--offline --locked` and bounded execution.


## 7. Stop-loss and remaining work

No production-change or inventory-drift stop condition was encountered.
Step D remains cancelled: no `BoundedDestruction` was added to any type or trait.
External suspension implementations remain unresolved, as required.

Step E2 still needs the independent UnknownConstruct axis: distinguish an
unresolved expansion from an unresolved construct that may introduce an owning
edge. It must retain discovery coverage while giving macros, attributes,
conditional alternatives, closure/async captures, and foreign storage their
appropriate ownership evidence and falsifiers. An exact inventory match is not
a disposition or a proof that these constructs are safe. This change assigns
no dispositions to that axis and does not close it.
