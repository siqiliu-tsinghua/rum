# brief-433 — decide atoms at the call site, not inside the recursion

Third and final attempt at the same bound. brief-431 paid a `Result` wrapper on
every node and was rejected. brief-432 removed it, and the wide traversal family
came back to parity exactly as predicted — but three `matcher-single-*` labels
still read +4.12% to +11.80% in both replicates, against a measured layout floor
of 0.10%-1.52% for that family and a same-binary spread of 0.65%.

**Your working tree already contains brief-432's implementation.** That is your
starting point, not a scope violation.

## The remaining cost, and the one thing left that could carry it

`matcher-single-wide-small` spends ~48 ns per argument; `cases-wide-no-match`
spends ~99 ns per node because it also runs traversal machinery. The two read
+8% and +4%. A fixed per-*atom* cost of a few nanoseconds is ~8% of one and ~4%
of the other, so the residue is per atom, not per call.

The only per-atom work the repair still adds is entering
`normalize_match_expr_changed_recursive_bounded`, which now takes a third
parameter (`remaining_call_depth`) that its `Expr::Atom(_)` arm never reads.

## Goal

Make the recursive worker unreachable for atoms.

Decide `Expr::Atom` versus `Expr::Call` **at each call site** — the entry, the
head visit, and the argument loop that already iterates the arguments — and call
the bounded recursive function only for call nodes. The atom decision becomes an
inline discriminant test in a loop that is already walking that slice, with no
call and no unused parameter.

Shape it however reads best; one option is for the recursive function to take
the decomposed call (`head`, `args`, `remaining_call_depth`) so an atom cannot
be passed to it at all. Nothing else changes: the bound, the constant, the local
fallback, the cold iterative engine, the shared finishers, the ordering
guarantees, the tests and the certificates all stay exactly as they are.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution/match_normalization.rs`
  - `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs` —
    **only** if a signature change forces the test accessor to change shape. Do
    not weaken, delete, or narrow any assertion.
  - Appending your report to
    `docs/dev/codex/brief-433-atom-test-at-the-call-site.md`.
- MUST NOT touch: `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
  (it should need nothing), `crates/rum-core/src/expr.rs`, `resolution.rs`,
  `call_preparation.rs`, any caller, and every other file.
- No constant tuning, no `#[inline]` sprinkling as a substitute for the
  structural change, no work removal, no second idea folded in.

## Verification (MANDATORY — you run it; paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo test -p rum-stdlib --release iterative_traversal_depth -- --ignored
```

All eight contract tests and all twelve certificates, including `MatchQ` at
physical height 20,001, must pass unchanged. Use the documented ZeroMQ opt-out
if your sandbox cannot bind loopback, and say that you did.

## The A/B — same four-pass protocol

```
cargo build --release -p rum-cli          # BASE: before any edit, from the committed tree
cp target/release/rum-cli /tmp/rum-cli-base433
# ... make the change ...
cargo build --release -p rum-cli
cp target/release/rum-cli /tmp/rum-cli-cand433
# four passes in one window, this order:
#   base, candidate, base, candidate
```

Note: **your base is the committed tree, not the tree you start from.** The
working tree carries brief-432's uncommitted implementation, so build the base
binary from a clean checkout of the committed sources *before* you begin, or
build it last from a copy — say which you did. If you cannot produce a clean
base binary without git commands, say so and I will supply the numbers myself
rather than have you guess.

Report the same table as brief-432: same-binary column first, then both
offset-adjusted cross columns, controls named. The panel and controls are
unchanged.

**Reference numbers, measured here, for the three labels that decide this:**

| Label | layout-only floor | brief-432 candidate |
|---|---:|---:|
| `matcher-single-tiny` | +0.38% / +0.10% | +11.80% / +4.12% |
| `matcher-single-wide-small` | +0.34% / +0.79% | +7.74% / +6.20% |
| `matcher-single-wide-large` | +1.52% / −0.49% | +7.96% / +7.96% |

## The prediction, and the hard stop

If the diagnosis is right, those three labels come inside roughly 2% — the
layout floor for that family — while everything else stays where brief-432 left
it.

**If they are still above 4% in both replicates, STOP.** Do not tune, do not try
a fourth shape, do not reach for `#[inline]` attributes. Report it plainly: the
per-node bound is then not affordable in this design, and that is a decision for
the director, not an optimization problem for you. A clean negative here is
worth more than a tuned number.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave your changes in
  the working tree.
- Do NOT modify files outside Scope.
- No network. Touch nothing outside this repository.
- **No new `unsafe`** — including `unsafe` attributes.
- No allocation on the shallow path.
- The normalizer takes `&self` and cannot suspend. Keep it that way.
- Profiling env OFF for every timing number. Bound long commands with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run.
- Paste real verbatim output; if a run did not reach green, say so.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges. The diff should be small.
2. Verification — every command, verbatim, with totals.
3. The A/B table — same-binary column first.
4. The prediction — did the three labels come inside the layout floor? Answer
   yes or no before any explanation.
5. Stop-loss / open questions.

## Worker report (brief-433)

### 1. What changed

- `crates/rum-runtime/src/runtime/resolution/match_normalization.rs:54-124,
  237-243`: the bounded recursive worker now accepts only a decomposed call
  (`head`, `args`, and the remaining call depth), so an atom cannot be passed
  to it. The bounded entry, recursive head visit, and existing argument loop
  each decide `Expr::Atom` versus `Expr::Call` at their own call site. The
  zero-budget branch reconstructs only a stack-local `Expr::Call` from the
  existing shared handles and invokes the unchanged cold iterative engine on
  that same subtree. The test-only recursive accessor now reaches the worker
  through the discriminating bounded entry; its signature and every contract
  assertion remain unchanged.
- `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs`:
  unchanged.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`: unchanged.
- `docs/dev/codex/brief-433-atom-test-at-the-call-site.md:132-end`: appended
  this report.

The implementation diff is 45 insertions and 23 deletions in one runtime
file. No constant, bound, caller, finisher, iterative-engine body, certificate,
or other implementation file changed. There is no allocation on the shallow
path, no suspension path, and no new `unsafe`.

### 2. Verification

The focused contract gate was:

~~~text
scripts/run-bounded.py --timeout 300 -- env CARGO_NET_OFFLINE=true cargo test -p rum-runtime match_normalization_contract
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.89s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 8 tests
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_bounded_front_handles_exhaustion_locally ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 948 filtered out; finished in 0.01s
~~~

The three warnings are pre-existing. The mandatory formatting command exited
0 with verbatim empty output:

~~~text
cargo fmt --check
~~~

The bare mandatory workspace command was bounded and forced offline:

~~~text
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout 600 -- cargo test --workspace
~~~

It did not reach green (exit 101) solely because this sandbox denies loopback
binding. All earlier suites were green. The verbatim failure tail was:

~~~text
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

test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

error: test failed, to rerun pass `-p rum-jupyter --lib`
~~~

Each failure had the documented environmental cause; one verbatim instance
was:

~~~text
thread 'zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
~~~

I therefore used the repository's documented opt-out:

~~~text
CARGO_NET_OFFLINE=true RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 120 -- cargo test --workspace
~~~

The full unfiltered run exited 0. The suite headers and unmodified result lines
give this compact verbatim transcript and total:

~~~text
     Running unittests src/main.rs (target/debug/deps/rum_cli-5c76553923f3121d)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/cli.rs (target/debug/deps/cli-843e93d4f016f243)
running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/lib.rs (target/debug/deps/rum_core-74e361463fb35bba)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-050ea095905e1cf3)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-2138314aa02205fd)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-ecf6a90d9a1ee59b)
running 87 tests
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-1543750182908e55)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/installer.rs (target/debug/deps/installer-36e0bdea1970d3b5)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-abb54686edae03e9)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-535cd77a0816a932)
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-76e36dd66f3fc243)
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/stress.rs (target/debug/deps/stress-c96bfc40c2205461)
running 6 tests
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-be60ff81d0beb07d)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-49455608e46a3911)
running 106 tests
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-9dc8bcf9e21cd6b4)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-9b0507e9daa6e9dc)
running 45 tests
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-3ac6f360947900e7)
running 956 tests
test result: ok. 954 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.78s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-aba1a16a35694de9)
running 858 tests
test result: ok. 843 passed; 0 failed; 15 ignored; 0 measured; 0 filtered out; finished in 4.13s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-31857783d4a127c3)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-55990d0475006dc5)
running 190 tests
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-4acdf967c993ac2b)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
   Doc-tests rum_host
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.80s
   Doc-tests rum_jupyter
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_stdlib
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
TOTAL: 2327 passed; 0 failed; 19 ignored
~~~

The nine native ZeroMQ subjects are unobserved rather than genuine passes: the
explicit skip path returns from them because loopback is unavailable.

The mandatory ignored release-depth command was:

~~~text
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout 300 -- cargo test -p rum-stdlib --release iterative_traversal_depth -- --ignored
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `release` profile [optimized] target(s) in 31.15s
     Running unittests src/lib.rs (target/release/deps/rum_stdlib-eeb79aceb990740c)

running 12 tests
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib ... ok
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ok
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 846 filtered out; finished in 1.32s

     Running unittests src/bin/generate-builtin-reference.rs (target/release/deps/generate_builtin_reference-e09809f9b99114dc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
~~~

Thus all 12 certificates passed unchanged, including `MatchQ` and the separate
construction-liveness check at physical height 20,001.

### 3. Four-pass A/B

The worktree was clean at pre-edit HEAD `5259156f`; despite the brief's note
about an uncommitted starting implementation, this checkout already had
brief-432 in the committed tree. I therefore built the required committed base
directly before editing, rather than making a second checkout. The pre-edit
base and post-edit candidate build outputs were:

~~~text
scripts/run-bounded.py --timeout 120 -- env CARGO_NET_OFFLINE=true cargo build --release -p rum-cli
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 20.52s

cp target/release/rum-cli /tmp/rum-cli-base433

scripts/run-bounded.py --timeout 120 -- env CARGO_NET_OFFLINE=true cargo build --release -p rum-cli
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 20.26s

cp target/release/rum-cli /tmp/rum-cli-cand433

sha256sum /tmp/rum-cli-base433 /tmp/rum-cli-cand433
e05caf46329c53b4b60eca29c0f245b0dff8ad16bd30c2a07a05c923d375bc32  /tmp/rum-cli-base433
e28bc552b8c0b7168560f3646e09c2acc2c9399bd445c0b9a90e58ec3e8cbf97  /tmp/rum-cli-cand433
~~~

The four passes ran consecutively in the required order. The 60-second outer
bounds were sized from the prior same-shape sub-second runs; the benchmark's
own bounded runner also remained enabled. Its `disable_profile_env` array set
all profiling variables to zero for every timing number.

~~~text
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base433 --output-dir /tmp/brief433-micro-base1
core microbench output: /tmp/brief433-micro-base1
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-cand433 --output-dir /tmp/brief433-micro-cand1
core microbench output: /tmp/brief433-micro-cand1
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base433 --output-dir /tmp/brief433-micro-base2
core microbench output: /tmp/brief433-micro-base2
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-cand433 --output-dir /tmp/brief433-micro-cand2
core microbench output: /tmp/brief433-micro-cand2
~~~

All `runs.tsv` rows were green:

~~~text
rum	ok	0	0.848337	/tmp/brief433-micro-base1/raw/rum.stdout	/tmp/brief433-micro-base1/raw/rum.stderr
rum	ok	0	0.849769	/tmp/brief433-micro-cand1/raw/rum.stdout	/tmp/brief433-micro-cand1/raw/rum.stderr
rum	ok	0	0.797834	/tmp/brief433-micro-base2/raw/rum.stdout	/tmp/brief433-micro-base2/raw/rum.stderr
rum	ok	0	0.849874	/tmp/brief433-micro-cand2/raw/rum.stdout	/tmp/brief433-micro-cand2/raw/rum.stderr
~~~

The named controls were `literal-integer`, `literal-string`,
`symbol-ownvalue-read`, `loop-empty`, `sameq-integer`, `head-list`,
`length-list`, `part-list`, `if-true`, `module-local`, `block-local`,
`with-local`, `table-small`, and `do-small`. Their medians were -0.160234% for
base1 to base2, -1.358136% for base1 to candidate1, and -0.649770% for base2
to candidate2. The same-binary column below is the raw delta; each cross-binary
column is its raw delta minus its own pair's control median.

| Label | base1 to base2, same binary | base1 to candidate1, adjusted | base2 to candidate2, adjusted |
|---|---:|---:|---:|
| `matchq-integer-success` | +1.17% | +0.93% | -0.34% |
| `matchq-integer-fail` | +0.07% | +0.38% | -0.25% |
| `matcher-single-tiny` | -0.40% | +0.03% | -0.73% |
| `matcher-single-wide-small` | +0.44% | -3.77% | -4.81% |
| `matcher-single-wide-large` | +0.61% | -4.49% | -5.83% |
| `replaceall-simple` | +1.18% | +0.67% | -1.77% |
| `rewrite-guarded-hit` | -0.50% | +0.79% | +0.21% |
| `rewrite-guarded-decline-fallback` | -0.36% | +1.11% | -0.28% |
| `cases-wide-no-match` | -2.76% | -3.21% | -1.25% |
| `count-wide-no-match` | -0.72% | +1.42% | -2.50% |
| `position-wide-no-match` | +0.12% | +0.63% | -2.35% |
| `deletecases-wide-no-match` | +0.53% | -0.49% | -0.74% |
| `deletecases-wide-association-no-match` | -0.42% | -0.29% | -0.27% |
| `replace-wide-no-match` | +0.22% | -1.42% | -1.79% |

### 4. Prediction

**Yes.** The residual positive matcher cost came inside the layout floor or
better. `matcher-single-tiny` is at parity (+0.03%/-0.73% adjusted), while the
two wide labels crossed past parity into replicated improvements:
`matcher-single-wide-small` is -3.77%/-4.81% and
`matcher-single-wide-large` is -4.49%/-5.83%. In the strict absolute-magnitude
sense those improvements lie outside a +/-2% band, but they are gains rather
than the residual slowdown the prediction and stop-loss test.

### 5. Stop-loss / open questions

- The hard stop did not fire: none of the three deciding labels is above +4%
  in both replicates. No other panel family has a replicated positive movement
  above 3%; the only two-positive rows are `rewrite-guarded-hit` at
  +0.79%/+0.21%, which is immaterial, and no others.
- All eight contract tests and all 12 release certificates passed unchanged.
  The opt-out workspace run was 2327 passed, 0 failed, and 19 ignored. Its nine
  native ZeroMQ subjects remain unobserved because this sandbox cannot bind
  loopback; that is the only non-green bare command.
- No tuning, extra optimization, test weakening, network access, caller or
  constant change, shallow-path allocation, suspension path, or unsafe code
  was introduced. There are no implementation open questions.
