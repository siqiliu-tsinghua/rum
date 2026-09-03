# brief-430 — move the candidate normalizer into its own module, no logic change

Step 2 of three. Step 1 (brief-429, `66faf671`) locked the copy-on-write
contract in tests. Step 3 (brief-431) replaces the recursion. **This step moves
code and changes nothing else**, so that step 3's diff shows only the algorithm.

The repository's own precedent is the reason for the separation: a file split
folded into a semantic change makes both unreviewable, which is why the
assignment-target line deferred its split until the repair closed
(`TODO.md`, family A's "evaluate a split when family B closes — not
mid-repair" note).

## Goal

Move the candidate-normalization cluster out of
`crates/rum-runtime/src/runtime/resolution.rs` into a new
`crates/rum-runtime/src/runtime/resolution/match_normalization.rs`, preserving
behavior exactly. `resolution.rs` is 20,037 bytes and the repository requires a
split assessment when a file at that size is touched
(`docs/code-organization-guidelines.md:64-79,124-140`); normalization is a real
responsibility boundary, and step 3 will add an explicit-stack engine to it.

When you are done, every moved function body is **byte-identical** to what it
was, `cargo test --workspace` is green, and the only semantic difference in the
tree is which file the code lives in.

## What moves

From `crates/rum-runtime/src/runtime/resolution.rs`:

- `Session::normalize_match_expr` (`:62-65`)
- `Session::normalize_definition_identity_for_matching` (`:67-96`)
- `Session::normalize_match_expr_changed` (`:99-118`)
- `Session::normalize_match_expr_changed_for_test` (`:120-127`, `#[cfg(test)]`)
- `Session::normalize_match_arg_list_changed` (`:128-153`)

Nothing else moves. `resolution.rs` keeps everything above and below that
cluster, including `mod resume_frames;`.

## The one thing that will break if you are not careful

`pub(super)` **changes meaning when the code moves down a level.** Inside
`resolution.rs` it means visible throughout `crate::runtime`; inside
`resolution::match_normalization` the same token would mean visible only inside
`resolution`, and the twelve callers across the runtime would stop compiling.

`normalize_match_expr` and `normalize_definition_identity_for_matching` must
therefore become `pub(in crate::runtime)`, which is exactly their current reach
— not `pub`, not `pub(crate)`, not `pub(super)`. The two private workers stay
private to the new module (their only callers move with them). The
`#[cfg(test)]` accessor is already `pub(in crate::runtime)` and keeps that.

Widening visibility beyond `pub(in crate::runtime)` is a stop condition, not a
convenience: brief-395 had to repair a published surface that had been widened
without anyone deciding to publish it.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution.rs` — remove the moved cluster,
    add the `mod match_normalization;` declaration.
  - `crates/rum-runtime/src/runtime/resolution/match_normalization.rs` (new).
  - Appending your report to
    `docs/dev/codex/brief-430-normalizer-module-move.md`.
- Files/areas you MUST NOT touch: everything else. In particular **no caller
  changes anywhere**. These are inherent methods on `Session`, so no call site
  needs an import change; if you find yourself editing a caller, you have
  changed visibility wrongly — stop and report instead.
- No logic change, no renaming, no reformatting of the moved code, no doc
  comment edits, no reordering of the functions relative to each other.
- The test module from step 1
  (`crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs`) must
  keep passing untouched. It is not in scope; if it needs an edit to compile,
  that means the move changed reach — stop and report.

## Deliverable

- The new module, with whatever `use` items it needs to compile on its own.
  Prefer explicit imports over a blanket glob if a glob would pull in more than
  the moved code uses.
- `resolution.rs` with the cluster removed and the module declared.
- **A mechanical byte-identity proof** in your report: extract each moved
  function's body from `git show HEAD:crates/rum-runtime/src/runtime/resolution.rs`
  and from the new file, and diff them. Paste the command and its empty output.
  Prose asserting the bodies are unchanged is not the deliverable; the diff is.
  Signature lines legitimately differ for the two visibility changes named
  above — show those two lines explicitly and separately, and let the diff cover
  the bodies.

## Verification (MANDATORY — you run it; paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

The workspace gate is right for this step even though the change is inside one
crate: a visibility mistake is exactly the failure mode here, and it surfaces
as a compile error in dependent crates, not as a test failure in `rum-runtime`.

**Do NOT run `scripts/core-language-microbench.sh`.** A pure move that the
compiler inlines identically cannot be measured against run noise, and a green
A/B here would be mistaken for evidence about step 3. State that reasoning in
the report in place of numbers. If you believe the move could change codegen,
say why in the report — do not measure it.

## Stop-loss

- If any caller needs an edit, stop and report: the reach changed.
- If `match_normalization_contract.rs` needs an edit, stop and report.
- If the byte-identity diff is non-empty for any body, either restore the
  original text or stop and report what forced the difference.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; I run the authoritative gate and commit.
- Do NOT modify files outside Scope.
- No network. Touch nothing outside this repository.
- No new `unsafe`.
- Bound long commands with `scripts/run-bounded.py --timeout SECONDS`, sized
  from a measured run of the same shape.
- Paste real verbatim output; if a run did not reach green, say so.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges.
2. The byte-identity proof — commands and their output.
3. The two visibility lines, before and after.
4. Verification — `cargo fmt --check` and `cargo test --workspace`, verbatim,
   with the test totals.
5. Why no microbench was run.
6. Stop-loss / open questions.

## Worker report (brief-430)

### 1. What changed

- `crates/rum-runtime/src/runtime/resolution.rs:1` declares the new private
  `match_normalization` child module. The five-function cluster formerly at
  `HEAD:62-162` was removed; the unchanged `Session` implementation resumes at
  current line 62. `mod resume_frames;` remains in place at current line 2.
- `crates/rum-runtime/src/runtime/resolution/match_normalization.rs:1-105` is
  the new module. Line 1 imports exactly the six runtime names used by the
  moved cluster, and lines 3-105 contain the five methods in their original
  order. Only the two visibility qualifiers required by the brief differ from
  `HEAD`.
- `docs/dev/codex/brief-430-normalizer-module-move.md:136-431` appends
  this worker report.

No caller and no test file changed. In particular,
`crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs` remains
untouched. The split reduces `resolution.rs` from 20,037 bytes to 16,394 bytes;
the extracted responsibility is 3,999 bytes.

### 2. Byte-identity proof

The following command finds each named function, excludes its signature and
outer closing brace, extracts its body by balanced brace depth, and diffs the
`HEAD` body against the new-module body:

```sh
set -euo pipefail
extract_body() {
    local function_name="$1"
    awk -v function_name="$function_name" '
        index($0, "fn " function_name "(") {
            found = 1
        }
        found && !started {
            line = $0
            opens = gsub(/\{/, "", line)
            line = $0
            closes = gsub(/\}/, "", line)
            if (opens > 0) {
                depth = opens - closes
                started = 1
            }
            next
        }
        started {
            line = $0
            opens = gsub(/\{/, "", line)
            line = $0
            closes = gsub(/\}/, "", line)
            depth += opens - closes
            if (depth == 0) {
                exit
            }
            print
        }
    '
}
for function_name in \
    normalize_match_expr \
    normalize_definition_identity_for_matching \
    normalize_match_expr_changed \
    normalize_match_expr_changed_for_test \
    normalize_match_arg_list_changed
do
    diff -u \
        --label "HEAD:${function_name} body" \
        --label "new:${function_name} body" \
        <(git show HEAD:crates/rum-runtime/src/runtime/resolution.rs | extract_body "$function_name") \
        <(extract_body "$function_name" < crates/rum-runtime/src/runtime/resolution/match_normalization.rs)
done
```

Verbatim output (empty; exit 0):

```text
```

Thus all five function bodies are byte-identical to `HEAD`.

### 3. Visibility lines before and after

Command:

```sh
git show HEAD:crates/rum-runtime/src/runtime/resolution.rs | rg -n 'pub\(super\) fn (normalize_match_expr\(|normalize_definition_identity_for_matching\()'
rg -n 'pub\(in crate::runtime\) fn (normalize_match_expr\(|normalize_definition_identity_for_matching\()' crates/rum-runtime/src/runtime/resolution/match_normalization.rs
```

Verbatim output:

```text
62:    pub(super) fn normalize_match_expr(&self, expr: &Expr) -> Expr {
67:    pub(super) fn normalize_definition_identity_for_matching(
4:    pub(in crate::runtime) fn normalize_match_expr(&self, expr: &Expr) -> Expr {
9:    pub(in crate::runtime) fn normalize_definition_identity_for_matching(
```

The before/after pairs are therefore:

```text
pub(super) fn normalize_match_expr
pub(in crate::runtime) fn normalize_match_expr

pub(super) fn normalize_definition_identity_for_matching
pub(in crate::runtime) fn normalize_definition_identity_for_matching
```

The workers remain private, and the test accessor retains its existing
`pub(in crate::runtime)` visibility.

### 4. Verification

The 120-second final bounds were sized from the measured cached workspace run
of the same shape, which completed in 14.77 seconds. The initial 1,200-second
workspace bound and 120-second formatting bound came from completed same-shape
repository gates recorded in the preceding worker reports; after measuring the
current shape, the final exact rerun used the tighter 120-second bound.

Formatting command:

```text
scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

Verbatim output: empty. Exit status was 0.

The first workspace command was the bare mandatory command under the bounded
runner:

```text
scripts/run-bounded.py --timeout 1200 -- cargo test --workspace
```

It did **not** reach green (exit 101). Compilation succeeded, all suites before
`rum-jupyter` were green, and the sandbox then denied loopback binding. The
failure tail was:

```text
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
```

Each failure reported the same repository-provided environmental instruction;
one verbatim instance was:

```text
thread 'zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
```

I then used exactly that documented opt-out and reran the full workspace gate:

```text
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 120 -- cargo test --workspace
```

The final unfiltered run exited 0 after 14.87 seconds. An identical cached
green run selected every suite header and every unmodified `test result:` line
to retain a compact verbatim transcript, then summed those result lines. Its
output was:

```text
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
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
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
running 955 tests
test result: ok. 953 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.75s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-aba1a16a35694de9)
running 856 tests
test result: ok. 843 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out; finished in 4.23s
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
TOTAL: 2326 passed; 0 failed; 17 ignored
```

The runtime line includes the seven untouched
`match_normalization_contract` tests. The three warnings emitted at the start
of the workspace run are the pre-existing dead-code warnings for
`register_test_if_builtin`, `register_test_return_builtin`, and
`register_test_vm_template_builtins`; there were no new warnings from the
moved module.

### 5. Why no microbench was run

Per the brief, `scripts/core-language-microbench.sh` was not run. This change
only relocates byte-identical function bodies behind an ordinary Rust module
boundary; the two visibility spellings preserve their prior runtime-wide
reach, and the compiler can inline the same bodies. A timing A/B would compare
run noise and would not provide evidence for step 3's later algorithm change.

### 6. Stop-loss / open questions

No stop-loss fired: no caller needed an edit, the contract test module needed
no edit, the body diff is empty, neither moved worker was made visible, and no
visibility was widened beyond `pub(in crate::runtime)`. No `unsafe`, dependency,
logic, naming, documentation-contract, or generated-file change was made.

The only open environmental caveat is the bare workspace run's loopback-bind
denial described above. The complete workspace is green when run with the test
harness's explicit `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` switch; the authoritative
gate on a host that permits loopback can exercise those nine native transports
without that switch.
