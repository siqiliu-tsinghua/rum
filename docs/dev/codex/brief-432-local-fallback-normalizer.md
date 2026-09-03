# brief-432 — the same bound, without the per-node wrapper

brief-431 built a correct bounded normalizer and it was rejected on cost. Its
engine, its tests and its certificates are good and **are already in your
working tree** — that is your starting point, not a scope violation. This brief
changes one thing: how exhaustion travels.

## What happened, so you do not repeat it

brief-431 made the recursive worker return
`Result<Option<Expr>, MatchNormalizerDepthExhausted>` and propagate exhaustion
to the root with `?`, then restart the whole normalization iteratively. The
depth budget is consumed only on `Expr::Call`, but **the return type is paid on
every node**, atoms included.

My four-pass gate (base, candidate, base, candidate, one window) found the cost
tracks nodes visited per normalization, not call nodes:

| Label | same-binary spread | cross, both replicates |
|---|---:|---|
| `cases-wide-no-match` (1 call node, ~1,600 atoms) | +2.68% | +15.57%, +6.45% |
| `count-wide-no-match` | +0.66% | +13.40%, +8.37% |
| `position-wide-no-match` | +3.42% | +12.49%, +6.59% |
| `matchq-integer-success` (one node) | −1.55% | +4.14%, +2.56% |

A wide list has exactly one call node, so the decrement cannot be what costs
6-15% there. What is paid 1,601 times is `Ok(None)` through a `?`.

(The `matcher-single-*` rows moved +9.7% to +11.2% between two runs of the
*identical* binary, so ignore that family entirely; it cannot resolve an effect
of this size. Same for `replace-wide-no-match`.)

## Goal

Keep brief-431's bound and its iterative engine. Change the recursion plumbing
so that:

1. **The recursive worker returns `Option<Expr>` again**, exactly the signature
   it had before brief-431. No `Result`, no `?`, no error type on that path.
2. **Exhaustion is handled locally, where it is detected.** When the bounded
   recursive worker reaches a call node with no budget left, it normalizes *that
   subtree* by calling the existing cold iterative engine and returns the
   result, then its caller carries on normally. Nothing propagates to the root
   and nothing is re-walked.
3. **Atoms take the byte-identical path they take today**: `Expr::Atom(_) =>
   None`, no budget test, no wrapper. A call node pays one compare and branch.

That is the whole change. The bound is unchanged, the iterative engine is
unchanged, the shared finishers are unchanged, the ordering guarantees are
unchanged, and the tests and certificates are unchanged.

brief-428's plan preferred root restart because it matched the `Level`/`Map`
precedent and was easier to prove. The measurement settles it the other way: a
local fallback also removes the double traversal that precedent costs, and the
equivalence tests you already have are what proves it.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution/match_normalization.rs`
  - `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs` —
    **only** to adjust how the tests reach the bounded entry if the signature
    change requires it. Do not weaken, delete, or narrow an assertion; the
    three-producer equivalence and the budget-boundary cases stay.
  - `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs` — expected to
    need no change at all; touch it only if it fails to compile.
  - Appending your report to
    `docs/dev/codex/brief-432-local-fallback-normalizer.md`.
- MUST NOT touch: `crates/rum-core/src/expr.rs`, `resolution.rs`,
  `call_preparation.rs`, any caller, and every other file.
- Do not "also" fix anything. No constant tuning, no constructor work, no
  removal of `Cases`' duplicate normalization, no compile-once pattern plan.

## Verification (MANDATORY — you run it; paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo test -p rum-stdlib --release iterative_traversal_depth -- --ignored
```

The certificate at physical height 20,001 must still pass, and the eight
contract tests must still pass unchanged. If your sandbox cannot bind loopback,
use the documented ZeroMQ opt-out and say so; I re-run that part here.

## The A/B protocol — four passes, not two

This is the part brief-431 got wrong, and it was my error in that brief, not the
worker's. A single base/candidate pair cannot distinguish a regression from this
panel's noise.

```
cargo build --release -p rum-cli          # BASE: before any edit
cp target/release/rum-cli /tmp/rum-cli-base432
# ... make the change ...
cargo build --release -p rum-cli
cp target/release/rum-cli /tmp/rum-cli-cand432
# then, in one window, four passes in this order:
bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base432 --output-dir <base1>
bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-cand432 --output-dir <cand1>
bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base432 --output-dir <base2>
bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-cand432 --output-dir <cand2>
```

Report one table with four columns per label: **the same-binary delta
(base1→base2) first**, then base1→cand1 and base2→cand2, each adjusted by the
control median of its own pair. Controls: `literal-integer`, `literal-string`,
`symbol-ownvalue-read`, `loop-empty`, `sameq-integer`, `head-list`,
`length-list`, `part-list`, `if-true`, `module-local`, `block-local`,
`with-local`, `table-small`, `do-small`.

Affected panel: the fourteen matcher/rewrite/traversal labels at
`scripts/core-language-microbench.sh:538-598`.

**A label whose cross-binary movement lies inside its own same-binary spread is
not evidence of anything — say so rather than reporting it as a change.**

## The prediction this tests

If the diagnosis is right, `cases`/`count`/`position`/`deletecases-wide-no-match`
return to parity, because a wide list of atoms has one call node and atoms now
pay nothing. State plainly whether that happened. **If those labels are still
6%+ slow in both replicates, the diagnosis is wrong** — stop and report that,
because then the cost is in the depth logic itself and this design cannot be
tuned into shape.

## Stop-loss

- A family positive in **both** replicates and outside its own same-binary
  spread by more than 3% → stop and report. Do not tune.
- Any existing contract assertion or certificate fails → stop and report; do not
  adjust the test.
- The same step fails twice for the same reason → stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do not run `git
  stash`, `git checkout`, or `git restore` — the working tree's current contents
  are deliberate and are your starting point. Leave your changes in the tree.
- Do NOT modify files outside Scope.
- No network. Touch nothing outside this repository.
- **No new `unsafe`.**
- No allocation on the shallow path; the iterative engine's vectors are
  allocated only when it runs.
- The normalizer takes `&self` and cannot suspend. Keep it that way.
- Profiling env OFF for every timing number. Bound long commands with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run.
- Paste real verbatim output; if a run did not reach green, say so.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges. The diff against the tree you
   started from should be small; say how small.
2. Verification — every command, verbatim, with totals; the certificate's pass.
3. The A/B table — same-binary column first.
4. The prediction — did the wide traversal labels return to parity?
5. Stop-loss / open questions.

## Worker report (brief-432)

### 1. What changed

- crates/rum-runtime/src/runtime/resolution/match_normalization.rs:54-103,
  215-222: removed MatchNormalizerDepthExhausted and
  BoundedMatchNormalization from the recursive path. The bounded worker and
  argument worker return Option directly again. Atoms take the literal
  Expr::Atom(_) => None arm; a call with zero remaining budget invokes the
  existing cold iterative engine for that subtree; all parents then continue
  through the ordinary Option path. Calls above the boundary retain one budget
  comparison/decrement, and the shallow path constructs no work vectors.
- crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:558-608:
  adjusted only the test accessor's signature-driven budget-boundary test.
  The zero-budget atom, zero/one-budget call, and changed/unchanged head and
  argument depth-17 cases remain. The 16-call local-fallback and 17-call
  recursive outcomes are now both checked against the frozen reference on
  Option discriminant, logical value, and whole-tree input-Arc origins.
- crates/rum-stdlib/src/tests/iterative_traversal_depth.rs: unchanged.
- docs/dev/codex/brief-432-local-fallback-normalizer.md: appended this report.

Against the tree at the start of this brief, the implementation/test diff is
small: 39 insertions and 42 deletions across two files (runtime 14+/26-,
contract test 25+/16-), net three lines removed. No caller, finisher,
iterative-engine body, constant, constructor, stdlib certificate, or other file
was changed.

### 2. Verification

The focused contract gate was:

~~~text
scripts/run-bounded.py --timeout 300 -- cargo test -p rum-runtime match_normalization_contract
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.88s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 8 tests
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_bounded_front_handles_exhaustion_locally ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 948 filtered out; finished in 0.01s
~~~

The first cargo fmt --check after editing did not reach green: it reported four
mechanical rustfmt wrapping differences, at match_normalization.rs:80 and
match_normalization_contract.rs:559,586,598. I applied exactly those shapes.
The mandatory final command then exited 0 with verbatim empty output:

~~~text
cargo fmt --check
~~~

The bare mandatory workspace command was bounded as follows:

~~~text
scripts/run-bounded.py --timeout 600 -- cargo test --workspace
~~~

It did not reach green (exit 101) solely because this sandbox denies loopback
binding. Every earlier suite was green. The verbatim failure tail was:

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

Each failure had the same environmental cause; one verbatim instance was:

~~~text
thread 'zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
~~~

Per the brief, I used that documented opt-out:

~~~text
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 120 -- cargo test --workspace
~~~

The full unfiltered run exited 0. A cached identical green run selected every
suite header and unmodified result line for this compact verbatim transcript
and total:

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
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
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
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-3ac6f360947900e7)
running 956 tests
test result: ok. 954 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.78s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-aba1a16a35694de9)
running 858 tests
test result: ok. 843 passed; 0 failed; 15 ignored; 0 measured; 0 filtered out; finished in 4.12s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-31857783d4a127c3)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-55990d0475006dc5)
running 190 tests
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
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
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.81s
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

The nine native ZeroMQ subjects are unobserved, not genuine passes; the
repository's explicit skip path returns from them because loopback is
unavailable. The three dead-code warnings shown by the focused gate are
pre-existing.

The mandatory ignored release-depth command was:

~~~text
scripts/run-bounded.py --timeout 300 -- cargo test -p rum-stdlib --release iterative_traversal_depth -- --ignored
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `release` profile [optimized] target(s) in 31.45s
     Running unittests src/lib.rs (target/release/deps/rum_stdlib-eeb79aceb990740c)

running 12 tests
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib ... ok
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ok
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 846 filtered out; finished in 1.33s

     Running unittests src/bin/generate-builtin-reference.rs (target/release/deps/generate_builtin_reference-e09809f9b99114dc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
~~~

Thus the unchanged certificate at physical height 20,001 passed, as did its
separate construction-liveness check and all ten existing height-4,097
certificates.

### 3. Four-pass A/B

The pre-edit base and post-edit candidate were built and copied before any
timing pass. Verbatim build outputs:

~~~text
scripts/run-bounded.py --timeout 1200 -- cargo build --release -p rum-cli
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 20.77s

cp target/release/rum-cli /tmp/rum-cli-base432

scripts/run-bounded.py --timeout 120 -- cargo build --release -p rum-cli
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 20.53s

cp target/release/rum-cli /tmp/rum-cli-cand432
~~~

The binaries were distinct:

~~~text
caeb794b52cd4e9bfa5baf4fc8271b5d9901b4787643b52f98a5992084874719  /tmp/rum-cli-base432
e05caf46329c53b4b60eca29c0f245b0dff8ad16bd30c2a07a05c923d375bc32  /tmp/rum-cli-cand432
~~~

The four passes ran consecutively in the required order. The 60-second outer
bounds were sized from the same-shape sub-second run; the benchmark also used
its own bounded runner. Its disable_profile_env array explicitly set every
profiling variable to zero for every timing number.

~~~text
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base432 --output-dir /tmp/brief432-micro-base1
core microbench output: /tmp/brief432-micro-base1
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-cand432 --output-dir /tmp/brief432-micro-cand1
core microbench output: /tmp/brief432-micro-cand1
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base432 --output-dir /tmp/brief432-micro-base2
core microbench output: /tmp/brief432-micro-base2
scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-cand432 --output-dir /tmp/brief432-micro-cand2
core microbench output: /tmp/brief432-micro-cand2
~~~

All runs.tsv rows were green:

~~~text
rum	ok	0	0.849258	/tmp/brief432-micro-base1/raw/rum.stdout	/tmp/brief432-micro-base1/raw/rum.stderr
rum	ok	0	0.849261	/tmp/brief432-micro-cand1/raw/rum.stdout	/tmp/brief432-micro-cand1/raw/rum.stderr
rum	ok	0	0.849240	/tmp/brief432-micro-base2/raw/rum.stdout	/tmp/brief432-micro-base2/raw/rum.stderr
rum	ok	0	0.850286	/tmp/brief432-micro-cand2/raw/rum.stdout	/tmp/brief432-micro-cand2/raw/rum.stderr
~~~

The named-control medians were +0.013406% for base1 to base2,
-0.358908% for base1 to candidate1, and -1.474814% for base2 to
candidate2. The same-binary column is the raw delta; each cross-binary column
is its pair's raw percentage delta minus that pair's control median, matching
the required ordering:

| Label | base1 to base2, same binary | base1 to candidate1, adjusted | base2 to candidate2, adjusted |
|---|---:|---:|---:|
| matchq-integer-success | +1.57% | +0.43% | +1.16% |
| matchq-integer-fail | +1.36% | -0.45% | +0.54% |
| matcher-single-tiny | +0.05% | -1.73% | -0.87% |
| matcher-single-wide-small | -8.03% | +2.40% | +1.81% |
| matcher-single-wide-large | -8.24% | +3.07% | +1.62% |
| replaceall-simple | -2.81% | -1.24% | +2.27% |
| rewrite-guarded-hit | -1.77% | -2.26% | +1.26% |
| rewrite-guarded-decline-fallback | -0.89% | -0.71% | +1.99% |
| cases-wide-no-match | -0.08% | -4.27% | -5.23% |
| count-wide-no-match | -1.27% | -3.04% | -2.58% |
| position-wide-no-match | +0.02% | -1.20% | -4.53% |
| deletecases-wide-no-match | +1.70% | -3.35% | -3.37% |
| deletecases-wide-association-no-match | -0.42% | +1.68% | +1.25% |
| replace-wide-no-match | +0.45% | +1.80% | +0.73% |

Per the brief, the matcher-single family and replace-wide-no-match are ignored:
their known same-binary instability cannot resolve movements of this size.
Both matchq rows, replaceall-simple, and the two wide matcher rows have
cross-binary movement inside their own same-binary spread and are not evidence
of a change. The guarded rewrite rows split direction or cross their spread in
only one replicate, so they likewise show no replicated effect.

### 4. Prediction

Yes: the four predicted wide-list traversal labels returned to at least parity.
Cases was -4.27% and -5.23%, Count -3.04% and -2.58%, Position -1.20% and
-4.53%, and DeleteCases -3.35% and -3.37% in the two control-adjusted
replicates. None remained slow, let alone 6%+ slow in both replicates. Their
consistent non-positive direction, while a one-node MatchQ stays inside its
same-binary spread, supports the diagnosis that the removed per-node
Result/Ok/try plumbing caused brief-431's wide-atom cost.

### 5. Stop-loss / open questions

- No performance stop-loss fired. No affected family is positive in both
  replicates by more than 3% outside its same-binary spread.
  deletecases-wide-association-no-match is a small +1.68%/+1.25% against a
  0.42% same-binary movement, below the stop-loss.
- All eight contract tests and all 12 release depth certificates passed. The
  only non-green command was the explicitly anticipated native-ZeroMQ
  loopback denial; those nine subjects remain for the director to run on a
  host that can bind loopback.
- No tuning, extra optimization, caller change, allocation on the shallow
  path, suspension path, network access, or unsafe code was introduced. There
  are no implementation open questions.

## Director's gate (2026-09-03)

**The diagnosis was right and the repair is not finished.** Same protocol as
brief-431's gate — four passes in one window, base and candidate binaries
rebuilt here — plus a third binary to separate layout from cost.

**The layout control.** `/tmp/rum-cli-baseP` is HEAD with
`normalize_definition_identity_for_matching` moved to the end of its `impl`
block: identical semantics, different code layout. Measured against HEAD in its
own window, the whole affected panel moves at most **4.45%**, and the
`matcher-single-*` family moves 0.10% to 1.52%. That is this host's layout
floor for this panel, and it is much smaller than brief-431's gate assumed.
(The first attempt at this control used `#[no_mangle]`, which does not compile
under this toolchain without an `unsafe` attribute the project forbids; the
`cp` that followed silently copied the previous binary and the hash check
caught it.)

**What the wrapper removal bought.** The per-atom `Result` cost is gone. The
wide traversal family, which read +6.2% to +15.6% in both replicates before,
now reads inside or near its own noise: `cases` −6.28%/+6.92%, `count`
+1.25%/+4.28%, `position` +1.97%/+3.27%, `deletecases` −2.08%/+2.97%. The
prediction the brief made held.

**What is left, and it is real.** Three labels are positive in both replicates,
well outside the 1.52% layout floor for that family:

| Label | b1→c1 adj | b2→c2 adj | layout-only |
|---|---:|---:|---:|
| `matcher-single-tiny` | +11.80% | +4.12% | +0.38% / +0.10% |
| `matcher-single-wide-small` | +7.74% | +6.20% | +0.34% / +0.79% |
| `matcher-single-wide-large` | +7.96% | +7.96% | +1.52% / −0.49% |

Same-binary spread on those labels was ±0.65% in this window, so this is not run
noise either.

**The mechanism, and why it is consistent with the traversal rows being clean.**
`matcher-single-wide-small` spends ~48 ns per argument; `cases-wide-no-match`
spends ~99 ns per node because it also runs the traversal machinery. A fixed
per-atom cost of a few nanoseconds is therefore ~8% of one and ~4% of the
other — which is what both read. The remaining cost is per *atom*, and the only
thing the repair still does per atom is enter
`normalize_match_expr_changed_recursive_bounded`, which now carries a third
parameter it does not use on that arm.

**Verdict: not committed. brief-433 hoists the atom test to the call sites** so
the recursive function is entered only for call nodes, and atoms are decided by
an inline discriminant check in the loop that already iterates them. If that
does not bring these three labels inside the layout floor, the per-node bound is
not affordable in this design and the decision comes back to the user.

Correctness is not in question at any point here: workspace 2327/0/19 with the
nine native ZeroMQ tests actually running on this host, all twelve 2 MiB
certificates green including `MatchQ` at physical height 20,001, and the eight
contract tests including three-producer equivalence.
