# Report 483 — bounded destruction on the print sink

The revised 2026-09-21 brief is implemented. The worktree is uncommitted.
The inherited partial adapter/re-export work was preserved and completed; the
prior stop report is archived at
`target/brief-483/continued/prior-stop-report.md`.

## 1. What changed

Line ranges refer to the final working tree. Imports are included where needed.

| File | Lines | Why |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/print_sink.rs` | 1–40 | Object-safe `PrintSink: BoundedDestruction + Send` and private-field `ClosurePrintSink<F>`; the sole constructor explicitly asserts bounded destruction of captures. |
| `crates/rum-runtime/src/runtime/mod.rs` | 49, 72–75 | Register and export the new sink module. |
| `crates/rum-runtime/src/runtime/kernel_api.rs` | 56–58 | Expose the trait and adapter through the existing kernel API. |
| `crates/rum-runtime/src/runtime/model/session.rs` | 95, 131–146 | Store `Option<Box<dyn PrintSink>>`, accept `Box<dyn PrintSink>`, and document the closure declaration. Clearing is unchanged. |
| `crates/rum-runtime/src/runtime/session/session_helpers.rs` | 305 | Replace the unique installed-sink invocation with `sink.print_line(&text)`. |
| `crates/rum-host/src/lib.rs` | 69 | Re-export `PrintSink`, `ClosurePrintSink`, and `BoundedDestruction` to host-only consumers. |
| `crates/rum-host/src/session.rs` | 8, 246–266 | Accept `impl PrintSink + 'static`, retain host-side boxing, and document the declaration. |
| `crates/rum-cli/src/main.rs` | 16–19, 332–343 | Wrap stdout streaming through `rum_host::ClosurePrintSink`; no runtime dependency added. |
| `crates/rum-stdlib/src/tests/lazy.rs` | 2, 109–114 | Declare the cancellation closure's captured state. |
| `crates/rum-stdlib/src/tests/incidental_print_sink.rs` | 4, 18–22 | Declare the print collector closure's captured state. |
| `crates/rum-stdlib/src/tests/valueq_effect_suppression.rs` | 10, 123–127 | Declare the suppressed/restored collector closure's captured state. |
| `crates/rum-host/tests/lifecycle_contract.rs` | 1–5, 176–181 | Migrate the cancellation signal sink through the host surface. |
| `crates/rum-host/tests/suspension_abandonment_contract.rs` | 1–4, 25–29 | Migrate the counting sink through the host surface. |
| `crates/rum-host/src/tests/session_lifecycle.rs` | 48–50 | Migrate reset/buffering coverage. |
| `crates/rum-host/src/tests/editor_surface.rs` | 47–49 | Migrate exclusive streaming/clear coverage. |
| `crates/rum-host/src/tests/eval_contract.rs` | 222–227 | Migrate the host cancellation signal sink. |
| `crates/rum-host/tests/stress.rs` | 1–6, 62–67 | Migrate the rewrite cancellation signal sink. |
| `crates/rum-runtime/src/runtime/tests/session_requests/api.rs` | 37–47, 823–830 | Migrate both kernel cancellation signal sinks. |
| `scripts/check-host-payload-destruction.py` | 20–114, 154–170, 207–210 | Extend the existing case table to sixteen with exact per-case diagnostics, host-only consumers, and explicit legacy-baseline eligibility. |
| `crates/rum-host/tests/host_payload_destruction_contract.rs` | 5, 11–20 | Name the expanded gate and derive its 1320-second timeout from the recorded final cold run. |
| `crates/rum-host/tests/fixtures/host_payload_destruction_contract/legacy_raw_closure_sink.rs` | 1–12 | Raw legacy host closure, with its old `&str` parameter type explicit. |
| `crates/rum-host/tests/fixtures/host_payload_destruction_contract/legacy_boxed_fnmut_sink.rs` | 1–11 | Raw legacy boxed `FnMut` kernel input. |
| `crates/rum-host/tests/fixtures/host_payload_destruction_contract/named_sink_without_marker.rs` | 1–7 | Reject a named implementor lacking the destruction marker. |
| `crates/rum-host/tests/fixtures/host_payload_destruction_contract/adapter_private_field_bypass.rs` | 1–7 | Reject direct struct-literal construction outside the crate. |
| `crates/rum-host/tests/fixtures/host_payload_destruction_contract/declared_named_sink.rs` | 1–14 | Accept an explicitly declared named sink using only `rum-host`. |
| `crates/rum-host/tests/fixtures/host_payload_destruction_contract/declared_closure_adapter.rs` | 1–11 | Accept the named constructor using only `rum-host`. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | 23–24, 26 | Apply both authorized payload spellings and pin the authorized constructor admission row. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 547 | Source-line drift only. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 1033–1035 | Source-line drift only. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 953 | Macro start/end source-line drift only. |
| `docs/guide/embedding.md` | 339–350 | Describe the declaration API while retaining exclusive routing, thread, panic/poison, clear, and reset behavior. |
| `docs/dev/codex/report-483-bounded-destruction-print-sink.md` | Entire file | Record the change, exact verification, and falsifiers. |

Confirmed twelve pre-existing installations (eleven files) and four clear
calls, excluding the host-to-kernel forwarding calls and certificate fixtures.
All twelve installations are migrated. Captured state comprises flat
string collectors, atomic counters/flags, cancellation handles, unit-message
channels, or a static string; no recursive host ownership chain is introduced.

The existing provider negatives retain their exact E0277 message, primary
consumer source/implementation line, and required-bound note checks; the four
existing positive cases still require successful compilation. The boxed
sink case additionally pins the exact expected/found type note and primary
label; the privacy case pins E0451 and its private-field label. No arbitrary
compile failure is accepted. Each consumer has its own `[workspace]`, target
directory, and dependency boundary beneath repository-local `target/`; consumers
are only checked offline, never executed.

Organization assessment: `rum-host/src/session.rs` (30.1k characters) remains a
host lifecycle/request facade; a future split should separate configuration
and lifecycle from request dispatch. `session_helpers.rs` (24.6k) should
ultimately separate print/message routing from request-error mapping, but this
brief allows only its one invocation edit. `rum-cli/src/main.rs` (14.5k) can
later separate invocation parsing from stream I/O. The larger tests remain in
their existing behavior families: lazy descriptor/realizer tests (31.3k),
request API tests (29.5k), and abandonment contracts (25.9k); future splits can
follow those families. The runtime directory has 59 direct children; the new
40-line sink module is the explicitly required host-payload boundary alongside
the existing provider modules. Reorganizing those files is outside this brief.

## 2. Verification

All build/test commands were offline and process-group bounded. `TMPDIR` for
workspace checks was repository-local; the certificate runner likewise sets
its own repository-local `TMPDIR`. No external network access was used; the workspace loopback probes were denied by the sandbox as recorded below.

Formatting command (exit 0, no output):

```sh
TMPDIR="$PWD/target/brief-483/continued/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 120 -- cargo fmt --check > target/brief-483/continued/fmt-check.log 2>&1
```

Initial workspace command (exit 101):

```sh
TMPDIR="$PWD/target/brief-483/continued/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 2400 -- cargo test --offline --locked --workspace > target/brief-483/continued/workspace.log 2>&1
```

The initial run stopped at nine native ZeroMQ loopback capability probes because the sandbox rejects socket binding. Verbatim failure excerpts:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
error: test failed, to rerun pass `-p rum-jupyter --lib`
```

The existing harness explicitly permits skipping that PermissionDenied condition with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`. The workspace was rerun with that switch; no adapter code or test assertions changed. Those nine native socket test bodies were not exercised, and their early-return skips are counted as passed by Rust, not as ignored. The reported ignored tests are separate existing ignored cases.

```sh
TMPDIR="$PWD/target/brief-483/continued/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --offline --locked --workspace > target/brief-483/continued/workspace-sandbox.log 2>&1
```

Sandbox-supported workspace run: exit 0.

Workspace total, summed from the 31 reported test/doc-test results: **2431 passed, 0 failed, 57 ignored**, 0 measured, 0 filtered out. Verbatim result lines:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.95s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 972 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.89s
test result: ok. 925 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 8.36s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.69s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Standalone certificate command, after the workspace suite (exit 0):

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 1320 -- python3 scripts/check-host-payload-destruction.py > target/brief-483/continued/certificate-final.log 2>&1
```

All sixteen case results, verbatim:

```text
PASS legacy_impl: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
PASS legacy_new: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
PASS legacy_from_arc: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
PASS declared_new: compiled (0.18s)
PASS declared_from_arc: compiled (0.18s)
PASS legacy_file_io_impl: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
PASS legacy_file_io_new: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (0.19s)
PASS legacy_file_io_from_arc: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
PASS declared_file_io_new: compiled (0.19s)
PASS declared_file_io_from_arc: compiled (0.18s)
PASS legacy_raw_closure_sink: rejected only for the trait bound `{closure@src/lib.rs:6:28: 6:45}: PrintSink` is not satisfied (1 E0277) (0.18s)
PASS legacy_boxed_fnmut_sink: rejected only for mismatched types (1 E0308) (0.23s)
PASS named_sink_without_marker: rejected only for the trait bound `NamedSink: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
PASS adapter_private_field_bypass: rejected only for field `closure` of struct `ClosurePrintSink` is private (1 E0451) (0.18s)
PASS declared_named_sink: compiled (0.19s)
PASS declared_closure_adapter: compiled (0.18s)
PASS: 16/16 cases (2.99s)
```

The raw log also contains every exact `CASE` command and compiler diagnostic:
`target/brief-483/continued/certificate-final.log`. Its full contents are
included below under the certificate transcript.

Timeout derivation: the fresh, independent sixteen-consumer run named
`cold-sixteen-final` took **129.24s**, with **10.13s** as the largest single
check. It ran without a concurrent CPU-heavy workload. Ten times those
measurements rounded upward to whole minutes gives **1320s** for the gate
and **120s** per check. The bound still covers the whole process group.
The recorded run used the same sixteen final fixture sources and independent
consumer/target layout as the gate.

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 1320 -- python3 scripts/check-host-payload-destruction.py --work-dir target/brief-483/continued/cold-sixteen-final > target/brief-483/continued/cold-sixteen-final.log 2>&1
```

```text
PASS legacy_impl: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (7.05s)
PASS legacy_new: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (7.11s)
PASS legacy_from_arc: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (7.07s)
PASS declared_new: compiled (6.97s)
PASS declared_from_arc: compiled (7.27s)
PASS legacy_file_io_impl: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (7.42s)
PASS legacy_file_io_new: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (7.06s)
PASS legacy_file_io_from_arc: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (7.16s)
PASS declared_file_io_new: compiled (7.72s)
PASS declared_file_io_from_arc: compiled (7.07s)
PASS legacy_raw_closure_sink: rejected only for the trait bound `{closure@src/lib.rs:6:28: 6:45}: PrintSink` is not satisfied (1 E0277) (10.13s)
PASS legacy_boxed_fnmut_sink: rejected only for mismatched types (1 E0308) (7.23s)
PASS named_sink_without_marker: rejected only for the trait bound `NamedSink: BoundedDestruction` is not satisfied (1 E0277) (9.87s)
PASS adapter_private_field_bypass: rejected only for field `closure` of struct `ClosurePrintSink` is private (1 E0451) (10.12s)
PASS declared_named_sink: compiled (10.09s)
PASS declared_closure_adapter: compiled (9.87s)
PASS: 16/16 cases (129.24s)
```

`git diff --check` also completed with exit 0 and no output.

## 3. Falsifier

**Step 1 — unchanged final legacy sources compile under the old interface.**
The storage type, kernel setter, host setter, and invocation were temporarily
restored by hand to their original `FnMut` forms. The unused new trait and
re-exports do not participate in either fixture. Both final legacy fixtures
contain no `PrintSink`, `ClosurePrintSink`, or `BoundedDestruction` reference.
Their source bytes were kept identical for baseline, candidate, and applicable
perturbation. These independent consumers were never executed.

The final raw closure explicitly writes its existing `&str` parameter type.
This is legacy closure syntax, not an adapter or destruction declaration.
The initial collector fixture hit E0282 before trait checking; a cancellation
closure without the annotation reached E0277 but hit a higher-ranked lifetime
inference error under the blanket perturbation. Those failures were rejected
by the runner, not admitted as enforcement evidence. The explicit legacy type
makes the required compile-success perturbation meaningful. The final baseline
was rerun after that fixture refinement.

Command and complete output (exit 0):

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 300 -- python3 scripts/check-host-payload-destruction.py --legacy-baseline --case legacy_raw_closure_sink --case legacy_boxed_fnmut_sink --work-dir target/brief-483/continued/legacy-baseline > target/brief-483/continued/legacy-baseline-certified.log 2>&1
```

```text
CASE legacy_raw_closure_sink: cargo check --offline --manifest-path /home/remi/dev/rum/target/brief-483/continued/legacy-baseline/legacy_raw_closure_sink/Cargo.toml --target-dir /home/remi/dev/rum/target/brief-483/continued/legacy-baseline/legacy_raw_closure_sink/target --message-format=json --color=never
    Checking rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Checking rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Checking rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
    Checking host-payload-legacy-raw-closure-sink v0.0.0 (/home/remi/dev/rum/target/brief-483/continued/legacy-baseline/legacy_raw_closure_sink)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.47s
PASS legacy_raw_closure_sink: compiled (2.60s)
CASE legacy_boxed_fnmut_sink: cargo check --offline --manifest-path /home/remi/dev/rum/target/brief-483/continued/legacy-baseline/legacy_boxed_fnmut_sink/Cargo.toml --target-dir /home/remi/dev/rum/target/brief-483/continued/legacy-baseline/legacy_boxed_fnmut_sink/target --message-format=json --color=never
    Checking rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Checking host-payload-legacy-boxed-fnmut-sink v0.0.0 (/home/remi/dev/rum/target/brief-483/continued/legacy-baseline/legacy_boxed_fnmut_sink)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.62s
PASS legacy_boxed_fnmut_sink: compiled (1.75s)
PASS: 2/2 cases (4.35s)
```

**Step 2 — blanket sink enforcement perturbation.** The final interface was
restored. The following temporary implementations were added only to
`print_sink.rs`. The marker blanket is necessary to make the requested sink
blanket well-formed while leaving its `BoundedDestruction` supertrait intact;
`host_payload.rs` was never edited.

```rust
impl<F: FnMut(&str) + Send> BoundedDestruction for F {}

impl<F: FnMut(&str) + Send> PrintSink for F {
    fn print_line(&mut self, line: &str) {
        self(line);
    }
}
```

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 180 -- python3 scripts/check-host-payload-destruction.py --case legacy_raw_closure_sink --work-dir target/brief-483/continued/diagnostic-cases > target/brief-483/continued/falsifier-blanket-certified.log 2>&1
```

Verbatim output: consumer compiles, runner exits 1 (required red).

```text
CASE legacy_raw_closure_sink: cargo check --offline --manifest-path /home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/legacy_raw_closure_sink/Cargo.toml --target-dir /home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/legacy_raw_closure_sink/target --message-format=json --color=never
    Checking rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Checking rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Checking rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
    Checking host-payload-legacy-raw-closure-sink v0.0.0 (/home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/legacy_raw_closure_sink)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.27s
FAIL legacy_raw_closure_sink: compiled despite expected rejection; boundary is not enforced (2.40s)
FAIL: 0/1 cases (2.40s)
```

Both temporary blanket implementations were removed by hand. The full recovery run was `cold-sixteen-final`, whose command and sixteen case lines are above:

```text
PASS: 16/16 cases (129.24s)
```

**Step 3 — marker supertrait perturbation.** Only
`pub trait PrintSink: BoundedDestruction + Send` was changed temporarily to
`pub trait PrintSink: Send`; the fixture was unchanged.

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 180 -- python3 scripts/check-host-payload-destruction.py --case named_sink_without_marker --work-dir target/brief-483/continued/diagnostic-cases > target/brief-483/continued/falsifier-marker.log 2>&1
```

Verbatim output: consumer compiles, runner exits 1 (required red).

```text
CASE named_sink_without_marker: cargo check --offline --manifest-path /home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/named_sink_without_marker/Cargo.toml --target-dir /home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/named_sink_without_marker/target --message-format=json --color=never
    Checking rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Checking rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Checking rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
    Checking host-payload-named-sink-without-marker v0.0.0 (/home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/named_sink_without_marker)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.36s
FAIL named_sink_without_marker: compiled despite expected rejection; boundary is not enforced (2.50s)
FAIL: 0/1 cases (2.50s)
```

The supertrait was restored by hand, then all sixteen cases were rerun (exit 0):

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 1320 -- python3 scripts/check-host-payload-destruction.py --work-dir target/brief-483/continued/cold-sixteen-final > target/brief-483/continued/restored-marker.log 2>&1
```

```text
PASS legacy_impl: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (1.46s)
PASS legacy_new: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (1.50s)
PASS legacy_from_arc: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (1.35s)
PASS declared_new: compiled (1.45s)
PASS declared_from_arc: compiled (1.44s)
PASS legacy_file_io_impl: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (1.59s)
PASS legacy_file_io_new: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (1.36s)
PASS legacy_file_io_from_arc: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (1.56s)
PASS declared_file_io_new: compiled (1.45s)
PASS declared_file_io_from_arc: compiled (1.49s)
PASS legacy_raw_closure_sink: rejected only for the trait bound `{closure@src/lib.rs:6:28: 6:45}: PrintSink` is not satisfied (1 E0277) (2.20s)
PASS legacy_boxed_fnmut_sink: rejected only for mismatched types (1 E0308) (1.45s)
PASS named_sink_without_marker: rejected only for the trait bound `NamedSink: BoundedDestruction` is not satisfied (1 E0277) (2.15s)
PASS adapter_private_field_bypass: rejected only for field `closure` of struct `ClosurePrintSink` is private (1 E0451) (2.31s)
PASS declared_named_sink: compiled (2.20s)
PASS declared_closure_adapter: compiled (2.21s)
PASS: 16/16 cases (27.21s)
```

**Additional privacy perturbation for the new-API-only case.** Only
`closure: F` became `pub closure: F` temporarily. The struct-literal fixture
then compiled, and the runner exited 1:

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 180 -- python3 scripts/check-host-payload-destruction.py --case adapter_private_field_bypass --work-dir target/brief-483/continued/diagnostic-cases > target/brief-483/continued/falsifier-private-field.log 2>&1
```

```text
CASE adapter_private_field_bypass: cargo check --offline --manifest-path /home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/adapter_private_field_bypass/Cargo.toml --target-dir /home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/adapter_private_field_bypass/target --message-format=json --color=never
    Checking rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Checking rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Checking rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
    Checking host-payload-adapter-private-field-bypass v0.0.0 (/home/remi/dev/rum/target/brief-483/continued/diagnostic-cases/adapter_private_field_bypass)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.60s
FAIL adapter_private_field_bypass: compiled despite expected rejection; boundary is not enforced (2.67s)
FAIL: 0/1 cases (2.67s)
```

The field was made private again by hand and all sixteen cases rerun (exit 0):

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 1320 -- python3 scripts/check-host-payload-destruction.py --work-dir target/brief-483/continued/cold-sixteen-final > target/brief-483/continued/restored-private-field.log 2>&1
```

```text
PASS legacy_impl: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (1.59s)
PASS legacy_new: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (1.40s)
PASS legacy_from_arc: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (1.35s)
PASS declared_new: compiled (1.53s)
PASS declared_from_arc: compiled (1.41s)
PASS legacy_file_io_impl: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (1.25s)
PASS legacy_file_io_new: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (1.34s)
PASS legacy_file_io_from_arc: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (1.60s)
PASS declared_file_io_new: compiled (1.45s)
PASS declared_file_io_from_arc: compiled (1.45s)
PASS legacy_raw_closure_sink: rejected only for the trait bound `{closure@src/lib.rs:6:28: 6:45}: PrintSink` is not satisfied (1 E0277) (2.00s)
PASS legacy_boxed_fnmut_sink: rejected only for mismatched types (1 E0308) (1.30s)
PASS named_sink_without_marker: rejected only for the trait bound `NamedSink: BoundedDestruction` is not satisfied (1 E0277) (2.25s)
PASS adapter_private_field_bypass: rejected only for field `closure` of struct `ClosurePrintSink` is private (1 E0451) (2.25s)
PASS declared_named_sink: compiled (2.01s)
PASS declared_closure_adapter: compiled (2.25s)
PASS: 16/16 cases (26.44s)
```

`named_sink_without_marker` and `adapter_private_field_bypass` have **no
pre-change baseline** because their source names newly introduced APIs. Their
post-change perturbations above certify the intended enforcement. The two
positive fixtures likewise use new APIs. No fixture was removed for any
falsifier; no `git checkout` or index-based restoration was used.

A separate read of the saved Cargo JSON confirmed successful builds with zero
compiler errors for all three perturbations, and byte-identical fixtures:

```text
legacy_raw_closure_sink: identical baseline/candidate source; no new API names
legacy_boxed_fnmut_sink: identical baseline/candidate source; no new API names
legacy_raw_closure_sink: perturbation build-finished=[true]; compiler errors=0; fixture unchanged
named_sink_without_marker: perturbation build-finished=[true]; compiler errors=0; fixture unchanged
adapter_private_field_bypass: perturbation build-finished=[true]; compiler errors=0; fixture unchanged
```

## 4. Inventory diff

The final revised inventory ruling governs: **two existing payload spelling
changes**, plus the explicitly authorized constructor public-input row. The
report-format phrase “the one content change” is stale relative to that ruling.

The storage path remains `Option[Own]/Box[Own]`; the public setter input remains
`Box[Own]`. Both payloads become `dyn PrintSink`. The new constructor row was
copied at discovery's actual line/column, **27:48**, not assigned manually.
No new storage edge, other input, removed row, changed ownership path,
reclassification, or `rum-host` scanner-root row appeared.

The five other changed rows are positions only: one derive, three closures,
and the module macro's start/end lines. `self_reaching.tsv` and
`unknown_storage.tsv` are unchanged. There are 49 host-payload rows (six storage,
43 public inputs), 7068 unknown-construct rows, and 57 self-reaching rows.
The workspace's `production_inventory_is_exact` test passed.

Discovery command (exit 0):

```sh
TMPDIR="$PWD/target/brief-483/continued/tmp" CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 900 -- cargo test --offline --locked -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture > target/brief-483/continued/discovery.log 2>&1
```

```text
UnknownHostPayload count: 49
UnknownConstruct count: 7068
UnknownHostPayload breakdown: 6 storage edges; 43 public input sites
Self-reaching type count: 57
Production files: 577; test-only files: 278; nominal definitions: 1504; aliases: 35; owning edges: 3440; resolved Drop owners: 12
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 976 filtered out; finished in 5.10s
```

The pre-write audit validated every old row against discovery before applying the authorized changes. Verbatim audit output:

```text
host_payloads.tsv: 48 existing rows; 2 payload spelling changes; 0 position-only changes
self_reaching.tsv: 57 existing rows; 0 payload spelling changes; 0 position-only changes
unknown_attributes.tsv: 1328 existing rows; 0 payload spelling changes; 1 position-only changes
unknown_captures.tsv: 2774 existing rows; 0 payload spelling changes; 3 position-only changes
unknown_macros.tsv: 2896 existing rows; 0 payload spelling changes; 1 position-only changes
unknown_storage.tsv: 70 existing rows; 0 payload spelling changes; 0 position-only changes
AUTHORIZED NEW INPUT:
UnknownHostPayload	crates/rum-runtime/src/runtime/print_sink.rs	27	48	public input rum_runtime::runtime::print_sink::impl(ClosurePrintSink < F >)@21::assert_bounded_destruction(closure): opaque owned generic parameter F : FnMut (& str) + Send
EXISTING CHANGED ROWS (old -> discovery):
- UnknownHostPayload	crates/rum-runtime/src/runtime/model/session.rs	95	40	rum_runtime::runtime::model::session::Session.print_sink/std::option::Option[Own]/arg0/std::boxed::Box[Own]/arg0: opaque owned payload dyn FnMut (& str) + Send
+ UnknownHostPayload	crates/rum-runtime/src/runtime/model/session.rs	95	40	rum_runtime::runtime::model::session::Session.print_sink/std::option::Option[Own]/arg0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PrintSink
- UnknownHostPayload	crates/rum-runtime/src/runtime/model/session.rs	137	44	public input rum_runtime::runtime::model::session::impl(Session)@130::set_print_sink(sink)/std::boxed::Box[Own]/arg0: opaque owned payload dyn FnMut (& str) + Send
+ UnknownHostPayload	crates/rum-runtime/src/runtime/model/session.rs	141	44	public input rum_runtime::runtime::model::session::impl(Session)@130::set_print_sink(sink)/std::boxed::Box[Own]/arg0: opaque owned payload dyn PrintSink
- UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	172	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
+ UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	176	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
- UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	154	27	closure captures: |scope| (capture types/modes unresolved)
+ UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	158	27	closure captures: |scope| (capture types/modes unresolved)
- UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	155	27	closure captures: |buckets| (capture types/modes unresolved)
+ UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	159	27	closure captures: |buckets| (capture types/modes unresolved)
- UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	156	22	closure captures: |bucket| (capture types/modes unresolved)
+ UnknownConstruct	crates/rum-runtime/src/runtime/model/session.rs	160	22	closure captures: |bucket| (capture types/modes unresolved)
- UnknownConstruct	crates/rum-runtime/src/runtime/mod.rs	87	16	unexpanded macro matches! (ends at 90:14)
+ UnknownConstruct	crates/rum-runtime/src/runtime/mod.rs	88	16	unexpanded macro matches! (ends at 91:14)
PASS: two payload spelling changes; one authorized constructor input; all remaining changes are source positions; no removals, new storage, changed ownership paths or reclassifications
```

## 5. A/B premise

Source searches before and after the migration confirm a unique installed-sink
invocation: `crates/rum-runtime/src/runtime/session/session_helpers.rs:305`.
The only edit in that file is `sink(&text)` → `sink.print_line(&text)`.
The no-sink branch at line 307 remains byte-for-byte `self.prints.push(text)`.
Other accesses initialize, assign, clear, temporarily take/restore, or measure
the sink; none invoke it.

Both the old `Box<dyn FnMut(&str) + Send>` and new `Box<dyn PrintSink>` dispatch
through one trait-object call. `ClosurePrintSink<F>::print_line` directly calls
its concrete generic closure, adding no allocation or second boxed dispatch.
No default-path counter, branch, lock, allocation, or control-flow change was
introduced. **No microbench A/B was run**, as the brief requires under these
verified premises.

## 6. Stop-loss / open questions

No implementation stop-loss or open question remains. Verification limitation: the nine native ZeroMQ loopback test bodies could not run in this sandbox; the supported permission-denied skip was used and is reported above. The earlier constructor-row
stop is resolved by the brief's explicit ruling. The marker, provider files,
suspension implementation, scanner code, core expression representation,
`Cargo.lock`, and other prohibited source/doc files were not modified.

No new unsafe, external network access, crash/overflow/process-termination reproducer,
commit, branch, push, git-config edit, or out-of-scope source change was made.
All temporary consumers, compiler diagnostics, audit scripts, and logs are
beneath repository-local `target/`. The source tree is left uncommitted.

Final read-only scope and restoration audit, verbatim:

```text
PASS scope: 32 changed/new files, all authorized
PASS final enforcement: supertrait and private field restored; no blanket implementation
PASS call-site audit: 12 migrated installations; 4 clear calls
PASS unique invocation: crates/rum-runtime/src/runtime/session/session_helpers.rs:305
```

## Earlier iteration commands and outcomes

These are retained to distinguish failed fixture-design probes from the final
certificate. The first raw collector source failed with E0282; the next
unannotated cancellation closure failed with a higher-ranked lifetime error
under the blanket perturbation. Neither was counted as a successful falsifier.
No existing provider case regressed. Final evidence is the certified baseline,
three successful-compilation perturbations, and restored 16/16 runs above.

```sh
TMPDIR="$PWD/target/brief-483/continued/tmp" CARGO_NET_OFFLINE=true python3 scripts/run-bounded.py --timeout 120 -- cargo fmt > target/brief-483/continued/fmt-apply.log 2>&1
```

Exit 0; no output.

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 300 -- python3 scripts/check-host-payload-destruction.py --legacy-baseline --case legacy_raw_closure_sink --case legacy_boxed_fnmut_sink --work-dir target/brief-483/continued/legacy-baseline > target/brief-483/continued/legacy-baseline.log 2>&1
```

```text
PASS legacy_raw_closure_sink: compiled (9.81s)
PASS legacy_boxed_fnmut_sink: compiled (6.81s)
PASS: 2/2 cases (16.62s)
```

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 900 -- python3 scripts/check-host-payload-destruction.py --case legacy_raw_closure_sink --case legacy_boxed_fnmut_sink --case named_sink_without_marker --case adapter_private_field_bypass --case declared_named_sink --case declared_closure_adapter --work-dir target/brief-483/continued/diagnostic-cases > target/brief-483/continued/diagnostic-cases.log 2>&1
```

```text
FAIL legacy_raw_closure_sink: failure was not solely the expected diagnostic (10.60s)
PASS legacy_boxed_fnmut_sink: rejected only for mismatched types (1 E0308) (6.51s)
PASS named_sink_without_marker: rejected only for the trait bound `NamedSink: BoundedDestruction` is not satisfied (1 E0277) (8.98s)
PASS adapter_private_field_bypass: rejected only for field `closure` of struct `ClosurePrintSink` is private (1 E0451) (9.67s)
PASS declared_named_sink: compiled (9.91s)
PASS declared_closure_adapter: compiled (9.67s)
FAIL: 5/6 cases (55.37s)
```

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 180 -- python3 scripts/check-host-payload-destruction.py --case legacy_raw_closure_sink --work-dir target/brief-483/continued/diagnostic-cases > target/brief-483/continued/raw-diagnostic.log 2>&1
```

```text
FAIL legacy_raw_closure_sink: failure was not solely the expected diagnostic (0.18s)
FAIL: 0/1 cases (0.18s)
```

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 300 -- python3 scripts/check-host-payload-destruction.py --legacy-baseline --case legacy_raw_closure_sink --case legacy_boxed_fnmut_sink --work-dir target/brief-483/continued/legacy-baseline > target/brief-483/continued/legacy-baseline-final.log 2>&1
```

```text
PASS legacy_raw_closure_sink: compiled (2.10s)
PASS legacy_boxed_fnmut_sink: compiled (1.21s)
PASS: 2/2 cases (3.31s)
```

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 1200 -- python3 scripts/check-host-payload-destruction.py --work-dir target/brief-483/continued/cold-sixteen > target/brief-483/continued/cold-sixteen.log 2>&1
```

```text
PASS legacy_impl: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (7.17s)
PASS legacy_new: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (7.06s)
PASS legacy_from_arc: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (6.76s)
PASS declared_new: compiled (6.82s)
PASS declared_from_arc: compiled (7.08s)
PASS legacy_file_io_impl: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (7.02s)
PASS legacy_file_io_new: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (7.11s)
PASS legacy_file_io_from_arc: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (7.83s)
PASS declared_file_io_new: compiled (7.35s)
PASS declared_file_io_from_arc: compiled (7.37s)
PASS legacy_raw_closure_sink: rejected only for the trait bound `{closure@src/lib.rs:6:28: 6:39}: PrintSink` is not satisfied (1 E0277) (9.92s)
PASS legacy_boxed_fnmut_sink: rejected only for mismatched types (1 E0308) (7.58s)
PASS named_sink_without_marker: rejected only for the trait bound `NamedSink: BoundedDestruction` is not satisfied (1 E0277) (10.28s)
PASS adapter_private_field_bypass: rejected only for field `closure` of struct `ClosurePrintSink` is private (1 E0451) (10.17s)
PASS declared_named_sink: compiled (10.12s)
PASS declared_closure_adapter: compiled (9.98s)
PASS: 16/16 cases (129.66s)
```

```sh
CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never python3 scripts/run-bounded.py --timeout 180 -- python3 scripts/check-host-payload-destruction.py --case legacy_raw_closure_sink --work-dir target/brief-483/continued/diagnostic-cases > target/brief-483/continued/falsifier-blanket.log 2>&1
```

```text
FAIL legacy_raw_closure_sink: failure was not solely the expected diagnostic (2.00s)
FAIL: 0/1 cases (2.00s)
```

The inventory audit initially rejected an overly strict uniqueness assumption for repeated, position-normalized closure witnesses before writing anything. The audit was corrected to compare their ordered multiplicities; the successful complete audit is reproduced above. Scanner code and inventory semantics were unchanged.

## Certificate transcript (standalone final run, verbatim)

```text
CASE legacy_impl: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_impl/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_impl/target --message-format=json --color=never
    Checking host-payload-legacy-impl v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_impl)
error: could not compile `host-payload-legacy-impl` (lib) due to 1 previous error
error[E0277]: the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied
   --> src/lib.rs:5:24
    |
5   | impl EntropySource for LegacyEntropy {
    |                        ^^^^^^^^^^^^^ the trait `BoundedDestruction` is not implemented for `LegacyEntropy`
    |
    = help: the trait `BoundedDestruction` is implemented for `ClosurePrintSink<F>`
note: required by a bound in `EntropySource`
   --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/random.rs:141:26
    |
141 | pub trait EntropySource: super::host_payload::BoundedDestruction + Send + Sync {
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `EntropySource`


PASS legacy_impl: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
CASE legacy_new: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_new/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_new/target --message-format=json --color=never
    Checking host-payload-legacy-new v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_new)
error: could not compile `host-payload-legacy-new` (lib) due to 1 previous error
error[E0277]: the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied
   --> src/lib.rs:5:24
    |
5   | impl EntropySource for LegacyEntropy {
    |                        ^^^^^^^^^^^^^ the trait `BoundedDestruction` is not implemented for `LegacyEntropy`
    |
    = help: the trait `BoundedDestruction` is implemented for `ClosurePrintSink<F>`
note: required by a bound in `EntropySource`
   --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/random.rs:141:26
    |
141 | pub trait EntropySource: super::host_payload::BoundedDestruction + Send + Sync {
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `EntropySource`


PASS legacy_new: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
CASE legacy_from_arc: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_from_arc/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_from_arc/target --message-format=json --color=never
    Checking host-payload-legacy-from-arc v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_from_arc)
error: could not compile `host-payload-legacy-from-arc` (lib) due to 1 previous error
error[E0277]: the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied
   --> src/lib.rs:6:24
    |
6   | impl EntropySource for LegacyEntropy {
    |                        ^^^^^^^^^^^^^ the trait `BoundedDestruction` is not implemented for `LegacyEntropy`
    |
    = help: the trait `BoundedDestruction` is implemented for `ClosurePrintSink<F>`
note: required by a bound in `EntropySource`
   --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/random.rs:141:26
    |
141 | pub trait EntropySource: super::host_payload::BoundedDestruction + Send + Sync {
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `EntropySource`


PASS legacy_from_arc: rejected only for the trait bound `LegacyEntropy: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
CASE declared_new: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/declared_new/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/declared_new/target --message-format=json --color=never
    Checking host-payload-declared-new v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/declared_new)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
PASS declared_new: compiled (0.18s)
CASE declared_from_arc: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/declared_from_arc/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/declared_from_arc/target --message-format=json --color=never
    Checking host-payload-declared-from-arc v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/declared_from_arc)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
PASS declared_from_arc: compiled (0.18s)
CASE legacy_file_io_impl: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_impl/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_impl/target --message-format=json --color=never
    Checking host-payload-legacy-file-io-impl v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_impl)
error: could not compile `host-payload-legacy-file-io-impl` (lib) due to 1 previous error
error[E0277]: the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied
  --> src/lib.rs:6:17
   |
6  | impl FileIo for LegacyFileIo {
   |                 ^^^^^^^^^^^^ the trait `BoundedDestruction` is not implemented for `LegacyFileIo`
   |
   = help: the trait `BoundedDestruction` is implemented for `ClosurePrintSink<F>`
note: required by a bound in `FileIo`
  --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/file_io.rs:71:19
   |
71 | pub trait FileIo: super::host_payload::BoundedDestruction + Send + Sync {
   |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `FileIo`


PASS legacy_file_io_impl: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
CASE legacy_file_io_new: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_new/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_new/target --message-format=json --color=never
    Checking host-payload-legacy-file-io-new v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_new)
error: could not compile `host-payload-legacy-file-io-new` (lib) due to 1 previous error
error[E0277]: the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied
  --> src/lib.rs:6:17
   |
6  | impl FileIo for LegacyFileIo {
   |                 ^^^^^^^^^^^^ the trait `BoundedDestruction` is not implemented for `LegacyFileIo`
   |
   = help: the trait `BoundedDestruction` is implemented for `ClosurePrintSink<F>`
note: required by a bound in `FileIo`
  --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/file_io.rs:71:19
   |
71 | pub trait FileIo: super::host_payload::BoundedDestruction + Send + Sync {
   |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `FileIo`


PASS legacy_file_io_new: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (0.19s)
CASE legacy_file_io_from_arc: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_from_arc/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_from_arc/target --message-format=json --color=never
    Checking host-payload-legacy-file-io-from-arc v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_file_io_from_arc)
error: could not compile `host-payload-legacy-file-io-from-arc` (lib) due to 1 previous error
error[E0277]: the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied
  --> src/lib.rs:7:17
   |
7  | impl FileIo for LegacyFileIo {
   |                 ^^^^^^^^^^^^ the trait `BoundedDestruction` is not implemented for `LegacyFileIo`
   |
   = help: the trait `BoundedDestruction` is implemented for `ClosurePrintSink<F>`
note: required by a bound in `FileIo`
  --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/file_io.rs:71:19
   |
71 | pub trait FileIo: super::host_payload::BoundedDestruction + Send + Sync {
   |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `FileIo`


PASS legacy_file_io_from_arc: rejected only for the trait bound `LegacyFileIo: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
CASE declared_file_io_new: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/declared_file_io_new/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/declared_file_io_new/target --message-format=json --color=never
    Checking host-payload-declared-file-io-new v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/declared_file_io_new)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
PASS declared_file_io_new: compiled (0.19s)
CASE declared_file_io_from_arc: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/declared_file_io_from_arc/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/declared_file_io_from_arc/target --message-format=json --color=never
    Checking host-payload-declared-file-io-from-arc v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/declared_file_io_from_arc)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
PASS declared_file_io_from_arc: compiled (0.18s)
CASE legacy_raw_closure_sink: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_raw_closure_sink/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_raw_closure_sink/target --message-format=json --color=never
    Checking host-payload-legacy-raw-closure-sink v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_raw_closure_sink)
error: could not compile `host-payload-legacy-raw-closure-sink` (lib) due to 1 previous error
error[E0277]: the trait bound `{closure@src/lib.rs:6:28: 6:45}: PrintSink` is not satisfied
   --> src/lib.rs:6:28
    |
6   |       session.set_print_sink(move |line: &str| {
    |  _____________--------------_^
    | |             |
    | |             required by a bound introduced by this call
7   | |         assert_eq!(line, "evaluation-started");
8   | |         evaluation_started_tx
9   | |             .send(())
10  | |             .expect("canceller should receive evaluation-start signal");
11  | |     });
    | |_____^ the trait `PrintSink` is not implemented for closure `{closure@src/lib.rs:6:28: 6:45}`
    |
    = help: the trait `PrintSink` is implemented for `ClosurePrintSink<F>`
note: required by a bound in `HostSession::set_print_sink`
   --> /home/remi/dev/rum/crates/rum-host/src/session.rs:260:49
    |
260 |     pub fn set_print_sink(&mut self, sink: impl PrintSink + 'static) {
    |                                                 ^^^^^^^^^ required by this bound in `HostSession::set_print_sink`


PASS legacy_raw_closure_sink: rejected only for the trait bound `{closure@src/lib.rs:6:28: 6:45}: PrintSink` is not satisfied (1 E0277) (0.18s)
CASE legacy_boxed_fnmut_sink: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_boxed_fnmut_sink/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_boxed_fnmut_sink/target --message-format=json --color=never
    Checking host-payload-legacy-boxed-fnmut-sink v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/legacy_boxed_fnmut_sink)
error: could not compile `host-payload-legacy-boxed-fnmut-sink` (lib) due to 1 previous error
error[E0308]: mismatched types
   --> src/lib.rs:10:28
    |
10  |     session.set_print_sink(sink);
    |             -------------- ^^^^ expected trait `PrintSink`, found trait `for<'a> FnMut(&'a str) + Send`
    |             |
    |             arguments to this method are incorrect
    |
    = note: expected struct `Box<(dyn PrintSink + 'static)>`
               found struct `Box<dyn for<'a> FnMut(&'a str) + Send>`
note: method defined here
   --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/model/session.rs:141:12
    |
141 |     pub fn set_print_sink(&mut self, sink: Box<dyn PrintSink>) {
    |            ^^^^^^^^^^^^^^


PASS legacy_boxed_fnmut_sink: rejected only for mismatched types (1 E0308) (0.23s)
CASE named_sink_without_marker: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/named_sink_without_marker/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/named_sink_without_marker/target --message-format=json --color=never
    Checking host-payload-named-sink-without-marker v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/named_sink_without_marker)
error: could not compile `host-payload-named-sink-without-marker` (lib) due to 1 previous error
error[E0277]: the trait bound `NamedSink: BoundedDestruction` is not satisfied
 --> src/lib.rs:5:20
  |
5 | impl PrintSink for NamedSink {
  |                    ^^^^^^^^^ the trait `BoundedDestruction` is not implemented for `NamedSink`
  |
  = help: the following other types implement trait `BoundedDestruction`:
            ClosurePrintSink<F>
            StdEntropySource
            StdFileIo
note: required by a bound in `PrintSink`
 --> /home/remi/dev/rum/crates/rum-runtime/src/runtime/print_sink.rs:7:22
  |
7 | pub trait PrintSink: BoundedDestruction + Send {
  |                      ^^^^^^^^^^^^^^^^^^ required by this bound in `PrintSink`


PASS named_sink_without_marker: rejected only for the trait bound `NamedSink: BoundedDestruction` is not satisfied (1 E0277) (0.18s)
CASE adapter_private_field_bypass: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/adapter_private_field_bypass/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/adapter_private_field_bypass/target --message-format=json --color=never
    Checking host-payload-adapter-private-field-bypass v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/adapter_private_field_bypass)
error: could not compile `host-payload-adapter-private-field-bypass` (lib) due to 1 previous error
error[E0451]: field `closure` of struct `ClosurePrintSink` is private
 --> src/lib.rs:5:9
  |
5 |         closure: |_: &str| {},
  |         ^^^^^^^^^^^^^^^^^^^^^ private field


PASS adapter_private_field_bypass: rejected only for field `closure` of struct `ClosurePrintSink` is private (1 E0451) (0.18s)
CASE declared_named_sink: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/declared_named_sink/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/declared_named_sink/target --message-format=json --color=never
    Checking host-payload-declared-named-sink v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/declared_named_sink)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
PASS declared_named_sink: compiled (0.19s)
CASE declared_closure_adapter: cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/declared_closure_adapter/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/declared_closure_adapter/target --message-format=json --color=never
    Checking host-payload-declared-closure-adapter v0.0.0 (/home/remi/dev/rum/target/host-payload-destruction-contract/declared_closure_adapter)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
PASS declared_closure_adapter: compiled (0.18s)
PASS: 16/16 cases (2.99s)
```
