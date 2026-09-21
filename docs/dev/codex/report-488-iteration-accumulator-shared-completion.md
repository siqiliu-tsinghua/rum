# brief-488 — iteration accumulator shared completion

## 1. What changed

| File | Final lines | Change and reason |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs` | 64; 95–98 | Derive `Clone` for the node and clone a shared node instead of panicking; uniquely owned nodes still move out through `Arc::try_unwrap`. |
| `crates/rum-runtime/src/runtime/tests/iterative_loop_drivers.rs` | 154–196 | Exact-result certificates for `Table`, `Sum`, and `Product` completing after two input suspensions. |
| `crates/rum-runtime/src/runtime/tests/iterative_loop_drivers.rs` | 198–233 | Preserve exact values for all three consumers with queued input and with a frozen prefix plus a uniquely owned tail. |
| `crates/rum-host/src/tests/eval_contract.rs` | 91–113 | Require successful resumed `Table`, an unpoisoned session, and a successful subsequent evaluation without reset. |
| `docs/dev/codex/report-488-iteration-accumulator-shared-completion.md` | Entire file | Record the requested before/after certificates, validation, and source evidence. |

The production repair changes two lines. On `Arc::try_unwrap` success the node and its payload are moved exactly as before. On failure the node's `Expr` and previous `Arc` link are cloned, preserving the rooted continuation's state. Suffix reversal and prefix conversion are unchanged. This applies the same move-when-unique/clone-when-shared rule already used by `SharedArgs::into_vec` at `crates/rum-core/src/expr.rs:647–659`.

The implementation file is 5,167 bytes; the runtime and host test files are 8,752 and 12,129 bytes respectively, all below their preferred size thresholds. No module split is needed. This required report is the only addition to the already broad codex brief/report directory; reorganizing that directory is outside the brief's scope.

## 2. Certificates: pre-repair failures and post-repair successes

The worktree was clean at entry. The pre-repair runs below contained the new tests while the production implementation still had the original uniqueness panic. No production repair was applied until both kernel and host failures were recorded. The same tests were rerun after the two-line repair.

These tests cover the generated builtin reference entries for `InputString`, `Input`, `Table`, `Sum`, and `Product`, and the `HostSession::eval` / `resume_with_input` rustdoc contract. They assert rum's own values and resumable-input behavior.

| Certificate | Program / inputs | Required result |
| --- | --- | --- |
| Kernel `Table` | `Table[InputString[], 2]`, then `0`, `1` | `List["0", "1"]` |
| Kernel `Sum` | `Sum[Input[], {i, 2}]`, then `2`, `3` | `5` |
| Kernel `Product` | `Product[Input[], {i, 2}]`, then `2`, `3` | `6` |
| Host | The same `Table` and inputs | `Success` with `List["0", "1"]`, no poison, then `1 + 1` succeeds with `2` |
| Unsuspended controls | All three input-using programs with `2`, `3` queued before evaluation | `List["2", "3"]`, `5`, `6` |
| Unique-tail controls | Body `If[SameQ[i, 2], Input[], i]`, iterator `{i, 3}`, one resume with `5` | `List[1, 5, 3]`, `9`, `15` |

The unique-tail cases collect `1` before suspending, then append `5` and `3` without suspending again. They exercise frozen-prefix preservation, unique-node extraction, and list ordering. The unsuspended controls exercise actual input-using bodies without a continuation, rather than only a constant-table shortcut. Both controls passed before and after the repair.

All commands ran in `/home/remi/dev/rum`. `CARGO_NET_OFFLINE=true` was set for Rust commands, cargo test commands also used `--offline --locked`, and `TMPDIR` kept generated temporary files inside `target/brief-488/tmp`. Raw captured output is in `target/brief-488/logs/`. Output blocks below are verbatim excerpts; compilation preambles are omitted from the certificate blocks.

### Kernel certificates

The command was identical before and after:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-488/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-runtime --offline --locked iteration_accumulator -- --test-threads=1
```

Pre-repair: exit 101; `target/brief-488/logs/pre-runtime-certificate.log`:

```text
running 5 tests
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_unsuspended_values ... ok
test runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions ... FAILED
test runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions ... FAILED
test runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions ... FAILED

failures:

---- runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions stdout ----

thread 'runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions' panicked at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:97:27:
completed iteration accumulator must be uniquely owned
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions stdout ----

thread 'runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions' panicked at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:97:27:
completed iteration accumulator must be uniquely owned

---- runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions stdout ----

thread 'runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions' panicked at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:97:27:
completed iteration accumulator must be uniquely owned


failures:
    runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions
    runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions
    runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions

test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured; 980 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

Post-repair: exit 0; `target/brief-488/logs/post-runtime-certificate.log`:

```text
running 5 tests
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_unsuspended_values ... ok
test runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 980 filtered out; finished in 0.02s
```

### Host certificate

The command was identical before and after:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-488/tmp" python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-host --offline --locked host_iteration_accumulator -- --test-threads=1
```

Pre-repair: exit 101; `target/brief-488/logs/pre-host-certificate.log`:

```text
running 1 test
test tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning ... FAILED

failures:

---- tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning stdout ----

thread 'tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning' panicked at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:97:27:
completed iteration accumulator must be uniquely owned
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning' panicked at crates/rum-host/src/tests/common.rs:58:18:
expected success, got Failure(HostFailure { code: Panic, exit_status: None, message: "host session caught panic: completed iteration accumulator must be uniquely owned", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })


failures:
    tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p rum-host --lib`
```

Post-repair: exit 0; `target/brief-488/logs/post-host-certificate.log` (the selected unit-test group):

```text
running 1 test
test tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.01s
```

The remaining eight host integration-test groups selected zero tests and each exited successfully. The full workspace run below runs those groups without the certificate filter.

### Initial fixture correction

The first pre-repair runtime run, using the same command, exited 101 before the numeric-input certificates could reach completion: I initially asserted an empty default prompt for `Input[]`; its prompt is `?`. I corrected only those test expectations and reran on the still-unrepaired implementation to obtain the genuine panic failures above. `InputString[]` retains its empty prompt. This was a test setup correction, not a change or weakening of the value/resumption contract.

Initial pre-repair output, verbatim (`target/brief-488/logs/pre-runtime.log`):

```text
running 5 tests
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values ... FAILED
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_unsuspended_values ... ok
test runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions ... FAILED
test runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions ... FAILED
test runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions ... FAILED

failures:

---- runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values stdout ----

thread 'runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values' panicked at crates/rum-runtime/src/runtime/tests/iterative_loop_drivers.rs:13:13:
assertion `left == right` failed
  left: "?"
 right: ""
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions stdout ----

thread 'runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions' panicked at crates/rum-runtime/src/runtime/tests/iterative_loop_drivers.rs:13:13:
assertion `left == right` failed
  left: "?"
 right: ""

---- runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions stdout ----

thread 'runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions' panicked at crates/rum-runtime/src/runtime/tests/iterative_loop_drivers.rs:13:13:
assertion `left == right` failed
  left: "?"
 right: ""

---- runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions stdout ----

thread 'runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions' panicked at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:97:27:
completed iteration accumulator must be uniquely owned


failures:
    runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values
    runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions
    runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions
    runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 980 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

## 3. Mandatory verification

Only the two edited test files needed a scoped formatting invocation; it exited 0 with no output:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-488/tmp" python3 scripts/run-bounded.py --timeout 180 -- rustfmt --edition 2024 --config skip_children=true crates/rum-runtime/src/runtime/tests/iterative_loop_drivers.rs crates/rum-host/src/tests/eval_contract.rs
```

The mandatory workspace formatting check exited 0:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-488/tmp" python3 scripts/run-bounded.py --timeout 180 -- cargo fmt --check
```

`cargo fmt --check` emitted no stdout or stderr; `target/brief-488/logs/fmt-check.log` is empty (0 bytes). There is no success message to paste.

Full workspace command:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-488/tmp" python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

**The mandatory workspace gate did not reach green.** The first run exited 101 at nine native ZeroMQ tests because this sandbox denies loopback binding. It stopped before the runtime suite. Contrary to the guide's shorthand about automatic skipping, `zeromq_native_test_support.rs:335–350` requires the existing explicit skip environment variable for `PermissionDenied`.

Initial workspace failure output, verbatim (`target/brief-488/logs/workspace.log`):

```text
failures:

---- zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets stdout ----

thread 'zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket stdout ----

thread 'zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip stdout ----

thread 'zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages stdout ----

thread 'zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata stdout ----

thread 'zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown stdout ----

thread 'zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply stdout ----

thread 'zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages stdout ----

thread 'zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end stdout ----

thread 'zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip


failures:
    zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket
    zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets
    zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip
    zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages
    zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata
    zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown
    zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply
    zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end
    zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages

test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

error: test failed, to rerun pass `-p rum-jupyter --lib`
```

The same bounded workspace command was then rerun with the existing sandbox skip option:

```bash
CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/brief-488/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

**That run also exited 101**, at `tests::ownership_graph::production_inventory_is_exact`. The runtime unit suite passed: 982 passed, 0 failed, 3 ignored. The host unit and integration suites also passed. The ownership inventory still describes the old derive attribute and the deleted panic macro; updating those inventory files is outside Scope, so implementation work stopped as required.

Ownership-inventory failure, verbatim (`target/brief-488/logs/workspace-sandbox.log`):

```text
failures:

---- tests::ownership_graph::production_inventory_is_exact stdout ----
EXACT INVENTORY self-reaching types: 57 rows
EXACT INVENTORY UnknownHostPayload: 50 rows

thread 'tests::ownership_graph::production_inventory_is_exact' panicked at crates/rum-stdlib/src/tests/ownership_graph/mod.rs:70:37:
UnknownConstruct inventory mismatch:
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs	64	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs	64	1	unresolved attribute/expansion derive (FrameRootTrace)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs	97	27	unexpanded macro panic! (ends at 97:91)


failures:
    tests::ownership_graph::production_inventory_is_exact

test result: FAILED. 924 passed; 1 failed; 52 ignored; 0 measured; 0 filtered out; finished in 7.17s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

The following is the verbatim output of the log-summary command, saved in `target/brief-488/logs/workspace-summary.log`. It copies Cargo's test-binary headers and result rows unchanged, then sums each run's completed groups. These are **partial workspace totals**, not a completed green gate:

```text
workspace.log:
     Running unittests src/main.rs (target/debug/deps/rum_cli-071a386fb6152ac7)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/cli.rs (target/debug/deps/cli-527e9f2d0cca7f28)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 36.66s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
AGGREGATE OF COMPLETED GROUPS: 16 groups; 284 passed; 9 failed; 2 ignored; 0 measured; 0 filtered out

workspace-sandbox.log:
     Running unittests src/main.rs (target/debug/deps/rum_cli-071a386fb6152ac7)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/cli.rs (target/debug/deps/cli-527e9f2d0cca7f28)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.77s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-78a8fab085fc98d8)
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 982 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.45s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: FAILED. 924 passed; 1 failed; 52 ignored; 0 measured; 0 filtered out; finished in 7.17s
AGGREGATE OF COMPLETED GROUPS: 20 groups; 2244 passed; 1 failed; 57 ignored; 0 measured; 0 filtered out
```

The sandbox run's 2,244 reported passes include nine native ZeroMQ cases that returned early through the explicit skip path; their native socket test bodies were not exercised. Excluding those nine leaves 2,235 reported passes. The 57 ignored tests are separate. The fail-fast runs did not reach the syntax or terminal test groups or any doc-tests.

The three runtime test-helper dead-code warnings (`register_test_if_builtin`, `register_test_return_builtin`, `register_test_vm_template_builtins`) appeared in both pre-repair and post-repair builds. `git diff --check` passed.


## 4. A/B premise verified from source

No microbench A/B was run. Source confirms that the changed match is unreachable on an unsuspended completion:

- `iteration/driver.rs:263–290`: ordinary `Table` collects into `Vec<Expr>` and returns that vector directly in `Expr::call` at lines 276 and 283. It does not construct a persistent accumulator or call its `into_vec`.
- `iteration/driver.rs:292–304`: `PersistentExprAccumulator::freeze(collected)` is reached only on `EvalError::InputRequired`. `iteration/mod.rs:71–77` puts those prior values in `SharedArgs` and sets `tail: None`.
- `iteration/driver.rs:310–322, 351–355`: resumed values are appended to the linked tail. Its conversion is used only in the suspended driver, at lines 333 and 344.
- `iteration/table_fast_path.rs:10–58, 61–98, 101–132`: the constant, identity, integer-plus, and list-template fast paths construct their result directly and never use `PersistentExprAccumulator`.
- `iteration_aggregate.rs:125–155`: ordinary `Sum`/`Product` likewise accumulate a `Vec<Expr>` and pass it directly to `finish_iteration_aggregate`; freezing is confined to `InputRequired` at lines 157–167.

More precisely than the brief's shorthand, an entirely unsuspended `Table` does not freeze its values at all: it uses the vector directly. Freezing happens at the first suspension and initially leaves the tail empty. In both cases the required premise holds: no unsuspended completion runs the modified linked-tail loop. The ordinary path is unchanged, and no diagnostic counter, allocation, or branch was added there.

## 5. Sum/Product and host documentation

`Sum` and `Product` needed the same repair to the shared accumulator, with no edit to `iteration_aggregate.rs`. Both use `SuspendedIterationAggregate.collected: PersistentExprAccumulator` at line 21 and complete through `state.collected.into_vec()` at lines 199 and 213. The suspended aggregate frame still clones its state at line 321. The two numeric certificates failed at the same `iteration/mod.rs:97` panic before the repair and now return exactly `5` and `6`.

`crates/rum-host/src/session.rs` is unchanged. Its `eval` and `resume_with_input` rustdoc still correctly describes the resume contract. The repair makes the valid program fulfill that contract; the host panic-isolation behavior needs no change.

## 6. Stop-loss / open questions

**Stopped at the scope boundary.** Option 1 works and all six new tests pass, with genuine pre-repair failures for all four suspension certificates. There is no unresolved semantic question about the repair, and no alternative repair option was substituted. The workspace gate remains red.

Clearing the observed ownership-inventory failure requires edits outside the permitted files:

- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv:247`: replace the old node derive row with the new `(Clone , FrameRootTrace)` row.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv:435`: remove the stale row for the deleted uniqueness `panic!`.

Those files and the scanner remain untouched. The brief's hard rule is: “Do NOT modify files outside Scope. If you believe you must, STOP and report.” This is the reason the requested all-green workspace handoff is unfinished. An authorized follow-up can update the two inventory rows and rerun the mandatory workspace gate; the nine native ZeroMQ cases still need a host that permits loopback for actual native coverage.


No commits, branches, pushes, git configuration changes, or stash operations were performed. No source files outside Scope were edited; the only documentation edit is this report. Generated build files, logs, and test temporaries stay under the repository's `target` directory. No new crash probe, unsafe code, `AccumulatorChain` adoption, suspension-driver change, or frame-protocol change was introduced. The existing suspend/resume continuation shape is preserved.

