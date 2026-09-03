# brief-429 — lock the candidate normalizer's copy-on-write contract in tests

Step 1 of three. This step is **test-only**: no production behavior changes.
Step 2 (brief-430) moves the normalizer into its own module with no logic
change; step 3 (brief-431) replaces its recursion with a bounded front path and
an explicit-stack fallback. This brief exists so that step 3 is provably
equivalence-preserving rather than argued to be.

## Goal

`Session::normalize_match_expr` and its two private workers
(`crates/rum-runtime/src/runtime/resolution.rs:62-153`) are a copy-on-write
normalizer whose exact `Option` protocol and `Arc` reuse are load-bearing.
Freeze that contract in a test module that compares production against a
frozen, test-only transcription of the current algorithm over a deterministic
corpus, comparing three things per case: the `Option` discriminant, the logical
expression, and which input `Arc`s the output reuses.

When you are done, a future rewrite that changes the discriminant, the value, or
the sharing on any corpus shape fails a named test — and you have *shown* it
fails, twice, by breaking it deliberately.

## Why the sharing half matters as much as the value half

Pointer identity here is an observable internal contract, not an optimization
detail: the evaluation-stability cache validates entries with exact
`SharedExpr`/`SharedArgs` pointer equality
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:332-349`), and an
uncached stability analysis recursively visits heads and args
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33`). A rewrite that
returns a logically equal but freshly allocated call would pass every value
assertion and silently cost cache hits and add a recursion. The `Arc`-origin
assertions are the ones most likely to catch step 3's real failure mode, which
is why this brief calibrates them separately.

## Context pointers

- Read `AGENTS.md` first (Testing Policy hard lines, verification discipline).
- **`docs/dev/codex/brief-428-candidate-normalizer-depth-design.md` §2 is the
  specification** this brief turns into tests. It was written from the same
  source and reviewed; where your reading disagrees with it, follow the source
  and say so in your report.
- Key files:
  - `crates/rum-runtime/src/runtime/resolution.rs:62-153` — the subject.
  - `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111` —
    `flatten_flat_call_args`, one level only; full flattening of a nested `Flat`
    chain is an emergent property of the bottom-up order.
  - `crates/rum-runtime/src/runtime/call_preparation.rs:340-372` — attribute
    normalization: one-level flatten, canonical sort, `Nothing` filtering.
  - `crates/rum-core/src/expr.rs:596-602,673-679` — `SharedExpr::ptr_eq` and
    `SharedArgs::ptr_eq`, the predicates the sharing assertions use.
  - `crates/rum-core/src/expr.rs:813-828` — the three call constructors and
    which wrapper each allocates.
- Test placement convention: runtime tests are files under
  `crates/rum-runtime/src/runtime/tests/`, registered in
  `crates/rum-runtime/src/runtime/tests/mod.rs` (71 modules today).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs` (new)
  - `crates/rum-runtime/src/runtime/tests/mod.rs` (one module registration)
  - `crates/rum-runtime/src/runtime/resolution.rs` — **`#[cfg(test)]` accessor
    only.** The two private workers are not visible outside their module; expose
    them through an accessor gated on `cfg(test)` so no production visibility
    widens. Every non-`cfg(test)` line in this file must be byte-identical when
    you finish; that is the reviewer's proof for this step.
  - Appending your report to
    `docs/dev/codex/brief-429-normalizer-contract-lock.md`.
- Files/areas you MUST NOT touch: everything else, and in particular
  `crates/rum-core/src/expr.rs`, the normalizer's own logic, and any file under
  `docs/` other than appending to this brief.
- No production code path changes in this step. If you find a bug in the
  normalizer while writing the tests, **do not fix it** — write it in the
  report as a finding. A test that encodes current behavior is the deliverable
  even if current behavior turns out to be wrong.

## Deliverable

1. **A frozen test-only reference** transcribed from
   `crates/rum-runtime/src/runtime/resolution.rs:62-153`, used by no production
   path. It is a snapshot of today's algorithm, not a second implementation to
   maintain: comment it as such.
2. **A deterministic corpus.** At minimum: atoms; empty, unary and wide calls;
   a changed argument in first, middle and last position; the unchanged,
   head-only-changed, args-only-changed and both-changed roots; heads that are
   themselves calls (normalized `Function` heads); `Flat`, `Orderless` and both
   together; a nested `Flat` chain deep enough that one-level flattening alone
   gives the wrong answer; argument spines and head spines at depth 17 and
   depth 65; and held definition identities.
3. **Three comparisons per case**: the `Option` discriminant, the logical
   expression, and the `Arc` origin relative to the input, using
   `SharedExpr::ptr_eq`/`SharedArgs::ptr_eq`. Logical `Expr::eq` alone is not
   sufficient — it has pointer fast paths and deliberately cannot see whether a
   wrapper was rebuilt (`crates/rum-core/src/expr.rs:755-775`).
4. **Direct assertions on all four rebuild arms**
   (`resolution.rs:107-117`) and on the definition-identity wrapper's hold-none
   early return and its own four arms (`resolution.rs:67-96`).
5. **The equality collapse at `resolution.rs:142-153` executed, not merely
   described.** Factor the reference's argument finisher so the test can call it
   directly and inject a logically-equal `Some` child, forcing the branch where
   an already-materialized changed prefix collapses back to `None`.
6. **An answer to the reachability question.** brief-428 could not construct a
   natural tree that reaches that collapse (§7.3 of that file). Say whether you
   can. If you cannot either, say so plainly — "not established" is a real
   answer and I will record it as an open observation. Do not change the branch
   either way.
7. **Two recorded red calibrations** (see Verification).

## rum / Rust pitfalls (read twice)

1. `None` and `Some(expr.clone())` can carry the same wrapper-level value.
   Compare the discriminant directly; never infer it from the value.
2. The args vector is lazy until the first changed child, and later unchanged
   children are shallow clones (`resolution.rs:125-135`). Pin that boundary
   through pointer origin, not through allocation timing or counters.
3. The head is normalized before the arguments and the attribute lookup uses the
   **normalized** head (`resolution.rs:103-106,137`). A corpus case with a head
   that normalizes into a different attribute set is what proves this ordering.
4. `flatten_flat_call_args` flattens one level only. The nested-`Flat` case must
   be built so that a parent-before-child order produces a visibly different
   answer, otherwise it proves nothing.
5. Your frozen reference recurses. Keep the corpus at depth 65 or below so the
   reference itself does not overflow; deep-height certificates belong to
   brief-431, not here.
6. Testing Policy is a hard line: this is rum-vs-rum, verifying rum's own
   current internal contract.

## Verification (MANDATORY — you run it; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime match_normalization_contract
cargo test -p rum-runtime
```

`-p rum-runtime` is the correct gate here and the workspace run is mine: this
change is `cfg(test)`-only inside one crate, so it is provably crate-local. Say
so in your report rather than running a wider gate.

**Do NOT run `scripts/core-language-microbench.sh` for this brief.** No release
code changes, so an A/B would measure run noise and nothing else. State that
reasoning in the report in place of the numbers. Performing a gate that cannot
observe the change is worse than skipping it, because its green is mistaken for
evidence.

**Two calibrations, both recorded verbatim, both reverted before the final
green run.** An assertion nobody has watched fail is not known to work.

1. *Discriminant/value calibration.* Temporarily make the `cfg(test)` accessor
   report an unchanged result as `Some(expr.clone())` instead of `None`. The
   discriminant test must go red. Paste the failure.
2. *Sharing calibration.* Temporarily make the frozen reference's args-only arm
   build its result with `Expr::call(head.clone(), args)` instead of
   `Expr::call_shared(head.clone(), SharedArgs::new(args))` — logically
   identical, different `Arc` origin for the head. The sharing test must go red.
   Paste the failure.

If either calibration passes, the corresponding assertions are not testing what
they claim; fix the test, not the calibration, and record both attempts.

## Stop-loss

If production and your frozen reference disagree on any corpus case, **STOP and
report the disagreement with the case**. Do not adjust the reference until they
agree — a disagreement means brief-428's specification is wrong somewhere, which
is a finding worth more than a green test, and it must reach me before step 3 is
written.

If the same test fails twice for the same reason, stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; I run the authoritative gate and commit.
- Do NOT modify files outside Scope. If you believe you must, STOP and say so in
  the report instead.
- No network. Touch nothing outside this repository.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No production behavior change of any kind in this step.
- Bound long-running commands with `scripts/run-bounded.py --timeout SECONDS`,
  sized from a measured run of the same shape, not guessed.
- Paste real verbatim output. If a run did not reach green, say so explicitly.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command with verbatim output, plus both red calibrations
   and the green run that followed them.
3. The corpus — what shapes it covers, and which assertion covers each of the
   four rebuild arms and the collapse branch.
4. The reachability answer for `resolution.rs:142-153`.
5. Findings — anything the tests revealed about current behavior, including
   anything you believe is a bug (recorded, not fixed).
6. Stop-loss / open questions.

## Worker report (brief-429)

### 1. What changed

- `crates/rum-runtime/src/runtime/resolution.rs:120-126` adds the only
  production-file change: a `#[cfg(test)]` accessor for the private changed
  outcome. The diff contains no non-`cfg(test)` change, so release behavior and
  production visibility are unchanged.
- `crates/rum-runtime/src/runtime/tests/mod.rs:155` registers the focused test
  module.
- `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:1-671`
  adds the frozen recursive snapshot, deterministic corpora, structural value
  comparison without `Expr::eq` pointer shortcuts, whole-tree input-Arc origin
  signatures, and the targeted contract assertions.
- `docs/dev/codex/brief-429-normalizer-contract-lock.md:198-1540` appends this
  durable report and the required calibration/gate evidence.

The pre-existing uncommitted worker report in
`docs/dev/codex/brief-428-candidate-normalizer-depth-design.md` was preserved
byte-for-byte and was not touched by this brief.

### 2. Verification

Timeouts were not guessed. The 120-second formatting bound and 1,200-second
runtime-test bound came from the completed same-shape repository run recorded
in `docs/dev/codex/brief-415-session-memory-walk-completeness.md:345-360`.
The first focused run here then measured 10.71 seconds including compilation;
the final focused and full runtime runs completed in 4.33 and 10.09 seconds,
respectively.

#### Red calibration 1 — Option discriminant/value

Temporary mutation (reverted immediately after this run):

```rust
self.normalize_match_expr_changed(expr)
    .or_else(|| Some(expr.clone()))
```

Command:

```text
scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime match_normalization_contract
```

Verbatim output (exit 101):

```text
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.66s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 7 tests
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... FAILED
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... FAILED

failures:

---- runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins stdout ----

thread 'runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins' panicked at crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:512:9:
assertion `left == right` failed: unchanged: Option discriminant
  left: true
 right: false
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference stdout ----

thread 'runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference' panicked at crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:445:9:
assertion `left == right` failed: atom: Option discriminant differs from frozen reference
  left: true
 right: false


failures:
    runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference
    runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins

test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 948 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

The intended discriminant assertions went red even though the injected
`Some(expr.clone())` carried the same wrapper-level value.

#### Red calibration 2 — sharing

Temporary mutation in the frozen reference's args-only arm (reverted
immediately after this run):

```rust
(None, Some(args)) => Some(Expr::call(head.clone(), args)),
```

Command:

```text
scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime match_normalization_contract
```

Verbatim output (exit 101):

```text
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.70s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 7 tests
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... FAILED
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... FAILED

failures:

---- runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms stdout ----

thread 'runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms' panicked at crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:591:9:
assertion `left == right` failed: held-head-only: definition input Arc reuse
  left: [CallArcOrigin { head_input_paths: [], args_input_paths: ["root"] }, CallArcOrigin { head_input_paths: ["root.h"], args_input_paths: [] }, CallArcOrigin { head_input_paths: ["root.a0"], args_input_paths: ["root.a0"] }]
 right: [CallArcOrigin { head_input_paths: [], args_input_paths: ["root"] }, CallArcOrigin { head_input_paths: [], args_input_paths: [] }, CallArcOrigin { head_input_paths: ["root.a0"], args_input_paths: ["root.a0"] }]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference stdout ----

thread 'runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference' panicked at crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:464:9:
assertion `left == right` failed: changed-argument-first: input Arc reuse differs
  left: [CallArcOrigin { head_input_paths: ["root"], args_input_paths: [] }, CallArcOrigin { head_input_paths: ["root.a0"], args_input_paths: [] }, CallArcOrigin { head_input_paths: ["root.a1"], args_input_paths: ["root.a1"] }, CallArcOrigin { head_input_paths: ["root.a2"], args_input_paths: ["root.a2"] }]
 right: [CallArcOrigin { head_input_paths: [], args_input_paths: [] }, CallArcOrigin { head_input_paths: [], args_input_paths: [] }, CallArcOrigin { head_input_paths: ["root.a1"], args_input_paths: ["root.a1"] }, CallArcOrigin { head_input_paths: ["root.a2"], args_input_paths: ["root.a2"] }]


failures:
    runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference
    runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms

test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 948 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

The logical values still matched, but the Arc-origin signatures caught the
fresh head wrapper exactly as intended.

#### Final green formatting gate

Command:

```text
scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

Verbatim output: empty. Exit status was 0.

#### Final green focused gate (after both reversions)

Command:

```text
scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime match_normalization_contract
```

Verbatim output (exit 0):

```text
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.33s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 7 tests
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 948 filtered out; finished in 0.00s
```

#### Final green runtime gate

Command:

```text
scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime
```

Exit status was 0. A subsequent identical bounded green rerun was made solely
to retain an untruncated transcript; that rerun's complete verbatim output is
in Appendix A below. The first final run's result lines were:

```text
test result: ok. 953 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.86s

   Doc-tests rum_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The workspace gate was intentionally not run: every change is crate-local and
the only production-file addition is `#[cfg(test)]`, exactly as the brief's
verification boundary specifies. The microbenchmark was intentionally not run
because no release code changed; it could only measure noise and cannot observe
this patch.

### 3. The corpus and branch coverage

The 21-case normalizer corpus covers atoms; empty, unary, and twelve-argument
calls; first/middle/last changed arguments; unchanged, head-only, args-only,
and both-changed roots; Flat, Orderless, already-canonical Orderless, and
Flat+Orderless; a four-level nested Flat chain; a normalized Function head
whose attribute set changes from empty to Orderless; and changed argument/head
spines at both call depths 17 and 65.

Every case compares, in order: the exact `Option` discriminant; a recursive
logical value comparison that does not use `Expr::eq`; and a whole-output-tree
signature of every input `SharedExpr`/`SharedArgs` Arc reused, obtained only
through `SharedExpr::ptr_eq` and `SharedArgs::ptr_eq`.

The direct rebuild-arm test maps the four cases as follows:

- `unchanged`: `(None, None)`, with both original root Arcs retained by the
  wrapper clone.
- `head-only`: `(Some(head), None)`, with a new head Arc and the original args
  Arc.
- `args-only`: `(None, Some(args))`, with the original head Arc and a new args
  Arc.
- `both`: `(Some(head), Some(args))`, with both root Arcs new.

The first/middle/last changed-child test separately pins the lazy-prefix
boundary: the changed child's args Arc is rebuilt while every unchanged child
before and after it retains both nested Arcs. The held-definition corpus pins
the hold-none no-walk return, the held atom return, and the wrapper's own
unchanged/head-only/args-only/both arms with the same explicit root-origin
expectations.

The factored frozen argument finisher is invoked directly with
`Some(vec![args[0].clone(), args[1].clone()])` under an already-canonical
Orderless head. That executes the attribute-normalization equality comparison
with an already-materialized changed prefix and asserts that it collapses to
`None`. The nested-Flat test also proves a single parent-first one-level
flatten is unequal to the expected result before asserting the production
bottom-up result is fully flat. The Function-head test asserts the parent args
are sorted only after its head normalizes into a Function carrying Orderless.

### 4. Reachability answer

Natural reachability of the changed-prefix equality collapse is **not
established**. I could not construct a natural expression whose child worker
returns a logically equal `Some` that attribute normalization then restores to
the parent's original args. In the covered current semantics, a natural
`Some` changes logical structure, while already-canonical attribute args
return `None`. The synthetic finisher injection locks the explicit branch
without claiming it is naturally reachable.

### 5. Findings

Production and the frozen transcription agreed on every corpus case, so the
brief-428 specification did not disagree with source and the stop-loss did not
fire. The tests confirmed the two subtle current behaviors called out by the
design: nested Flat normalization is bottom-up, and attribute lookup for a
parent uses the fully normalized Function head. No current normalizer bug was
found.

The three `test_builtins.rs` dead-code warnings predate and are unrelated to
this change.

### 6. Stop-loss / open questions

No stop-loss condition occurred. The only open observation is the natural
reachability question above; it remains "not established." No files outside
the brief's scope were modified, no production behavior changed, no network or
external oracle was used, and no `unsafe` was added.

### Appendix A — full verbatim runtime-gate output


```text
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 955 tests
test runtime::capabilities::tests::capability_policy_builder_can_allow_and_deny_individual_capabilities ... ok
test runtime::capabilities::tests::capability_policy_can_explicitly_allow_diagnostics ... ok
test runtime::capabilities::tests::capability_policy_allowlist_is_default_deny ... ok
test runtime::capabilities::tests::capability_policy_defaults_to_unrestricted ... ok
test runtime::capabilities::tests::capability_policy_narrowing_preserves_existing_restrictions ... ok
test runtime::call_preparation::tests::planner_keeps_held_arguments_verbatim_under_holdallcomplete ... ok
test runtime::call_preparation::tests::planner_allows_raw_sequence_splicing_for_held_arguments ... ok
test runtime::call_preparation::tests::planner_respects_unevaluated_wrapper_for_nonheld_arguments ... ok
test runtime::call_preparation::tests::planner_respects_evaluate_wrapper_for_held_arguments ... ok
test runtime::definitions::application::replace_all::plan::frame_root_trace_tests::rewrite_rule_plan_visits_the_exact_rule_roots ... ok
test runtime::definitions::unique_symbols::tests::keeps_small_symbol_sets_on_the_linear_path ... ok
test runtime::definitions::unique_symbols::tests::preserves_first_seen_order_across_the_adaptive_index_threshold ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_clock_builtins ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_diagnostic_builtins ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_io_builtins ... ok
test runtime::definitions::definition_lhs_preparation::syntax_policy::tests::every_registered_matcher_head_has_an_explicit_lhs_policy ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::replace_plan_resolves_duplicates_and_prefixes_during_insertion ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_sequence_splice_preserves_spliced_item_shared_nodes ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::root_target_discards_existing_and_later_descendants ... ok
test runtime::evaluation_cache::tests::shared_args_make_mut_profiles_only_cow_and_preserves_alias_values ... ok
test runtime::substitution::lexical_substitution::tests::explicit_function_parameters_shadow_outer_bindings ... ok
test runtime::session::session_requests::tests::eval_error_mapper_preserves_non_request_surface_limit_codes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_preserves_unchanged_sibling_call_shared_nodes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_reuses_bound_call_shared_nodes ... ok
test runtime::substitution::lexical_substitution::tests::rule_rhs_substitution_respects_pattern_locals ... ok
test runtime::pattern_syntax::tests::matcher_pattern_registry_round_trips_system_heads_only ... ok
test runtime::suspension::traversal::tests::option_cases_level_spec_continuation_round_trips_some_none_and_bare_spec ... ok
test runtime::suspension::primitive::tests::option_vec_expr_continuation_round_trips_some_none_and_fallback_payloads ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_alpha_rename_does_not_rewrite_inserted_bound_values ... ok
test runtime::substitution::lexical_substitution::tests::with_body_treats_binding_names_as_shadowed ... ok
test runtime::suspension::rewrite_rules::tests::option_rewrite_rule_info_continuation_round_trips_some_none_and_fallbacks ... ok
test runtime::suspension::primitive::tests::binding_vec_continuation_round_trips_multiple_symbol_maps ... ok
test runtime::builtin_catalog::tests::moved_stdlib_system_attributes_remain_kernel_known_without_registration ... ok
test runtime::builtin_catalog::tests::core_catalog_drives_both_dispatch_and_default_attributes ... ok
test runtime::definitions::definition_lhs_preparation::tests::already_canonical_lhs_reuses_its_shared_arguments ... ok
test runtime::evaluation_cache::tests::dirty_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::evaluation_cache::tests::append_to_preserves_an_aliased_list_value ... ok
test runtime::tests::arithmetic::power_memory_projection_refuses_before_large_allocation ... ok
test runtime::evaluation_cache::tests::pointer_mismatched_entry_is_left_untouched ... ok
test runtime::evaluation_cache::tests::association_lvalue_direct_update_rolls_back_append_and_replace ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_new_down_value ... ok
test runtime::evaluation_cache::tests::association_stability_index_preserves_distinct_and_duplicate_verdicts ... ok
test runtime::evaluation_cache::tests::repeated_writes_to_one_symbol_keep_invalidation_sound ... ok
test runtime::tests::arithmetic::exact_integer_mod_quotient_gcd_lcm_and_powermod_follow_floor_style_semantics ... ok
test runtime::evaluation_cache::tests::path_eviction_removes_only_the_named_exact_call_nodes ... ok
test runtime::tests::arithmetic::factorial_respects_integer_work_limit ... ok
test runtime::evaluation_cache::tests::generation_stale_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::tests::association_lookup_cache::association_lookup_index_weak_identity_survives_cow_updates ... ok
test runtime::tests::arithmetic::power_integer_work_limit_allows_normal_and_trivial_results ... ok
test runtime::tests::association_part_cursor::duplicate_association_keys_keep_first_position_and_last_value ... ok
test runtime::evaluation_cache::tests::association_lvalue_update_preserves_an_aliased_value ... ok
test runtime::evaluation_cache::tests::association_lvalue_recovery_falls_back_when_data_became_unstable ... ok
test runtime::evaluation_cache::tests::removing_valid_call_stability_entry_preserves_evaluation_outcome ... ok
test runtime::tests::arithmetic::exact_division_and_negative_integer_powers_produce_rationals ... ok
test runtime::tests::association_part_cursor::below_threshold_part_reads_stay_on_the_linear_scan_path ... ok
test runtime::evaluation_cache::tests::valid_call_stability_proof_preflight_is_non_consuming_and_reuses_validity ... ok
test runtime::evaluation_cache::tests::proof_reinsertion_uses_current_generation_dirty_snapshot_and_bloom ... ok
test runtime::tests::association_part_cursor::part_reads_reuse_the_lookup_cache_and_observe_cow_writes ... ok
test runtime::tests::arithmetic::exact_integer_arithmetic_evaluates_core_heads ... ok
test runtime::evaluation_cache::tests::repeated_unrelated_writes_do_not_bump_call_stability_generation ... ok
test runtime::tests::arithmetic::power_realistic_memory_ceiling_admits_one_million_bit_result ... ok
test runtime::tests::arithmetic::plus_and_times_fold_numeric_parts_without_forcing_symbolic_algebra ... ok
test runtime::tests::comparison_lowering::comparison_lowering_preserves_resolver_call_count_and_order ... ok
test runtime::evaluation_cache::tests::association_lvalue_direct_update_polls_interrupt_before_mutation ... ok
test runtime::evaluation_cache::tests::stability_cache_survives_unrelated_symbol_writes ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_attribute_change ... ok
test runtime::tests::arithmetic::power_reads_the_live_integer_work_limit ... ok
test runtime::tests::arithmetic::power_integer_work_limit_guards_large_denominators ... ok
test runtime::evaluation_cache::tests::append_to_args_become_exclusive_after_definition_take_and_cache_entry_removal ... ok
test runtime::tests::arithmetic::power_respects_integer_work_limit_for_large_positive_exponents ... ok
test runtime::tests::arithmetic::powermod_negative_exponents_use_modular_inverse ... ok
test runtime::tests::arithmetic::round_handles_exact_numbers_decimal_literals_and_half_even_ties ... ok
test runtime::tests::arithmetic::rational_atoms_expose_a_rational_head_for_patterns ... ok
test runtime::tests::association_lookup_cache::repeated_wide_association_reads_build_one_linear_index ... ok
test runtime::evaluation_cache::tests::association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof ... ok
test runtime::tests::arithmetic::scalar_math_builtins_thread_over_lists_and_leave_skipped_forms_symbolic ... ok
test runtime::evaluation_cache::tests::association_lvalue_nested_multi_and_bare_part_targets_use_fallback ... ok
test runtime::tests::arithmetic::scalar_math_builtins_report_declared_attributes ... ok
test runtime::tests::arithmetic::scalar_math_builtins_exact_cases ... ok
test runtime::tests::association_write_index::rewriting_an_appended_key_replaces_it_through_the_index ... ok
test runtime::tests::arithmetic::total_evaluates_documented_level_forms_and_leaves_invalid_specs_inert ... ok
test runtime::tests::association_write_index::small_association_writes_bypass_the_lookup_cache ... ok
test runtime::tests::context_and_loader::begin_and_end_balance_context_and_context_path ... ok
test runtime::tests::context_and_loader::bare_symbols_search_context_path_before_current_context ... ok
test runtime::tests::context_and_loader::begin_package_and_end_package_restore_outer_context_and_export_package_context ... ok
test runtime::tests::context_and_loader::begin_package_expands_relative_context_specs_against_the_current_context ... ok
test runtime::tests::context_and_loader::begin_package_missing_dependency_fails_closed_and_restores_state ... ok
test runtime::tests::context_and_loader::begin_package_second_dependency_failure_restores_state_and_package_list ... ok
test runtime::tests::context_and_loader::begin_package_loads_present_dependency_and_enters_package ... ok
test runtime::tests::context_and_loader::context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols ... ok
test runtime::tests::context_and_loader::context_variables_and_builtin_inspection_reflect_session_contexts ... ok
test runtime::tests::context_and_loader::begin_package_usage_messages_make_public_symbols_visible_from_private_context ... ok
test runtime::tests::context_and_loader::begin_package_dependencies_stay_on_private_context_path_when_already_loaded ... ok
test runtime::tests::context_and_loader::context_path_and_packages_assignments_update_session_fields ... ok
test runtime::tests::context_and_loader::eval_script_preserves_messages_emitted_before_a_later_error ... ok
test runtime::tests::context_and_loader::contexts_lists_known_session_contexts ... ok
test runtime::tests::context_and_loader::split_script_into_forms_skips_initial_shebang_trivia ... ok
test runtime::tests::context_and_loader::split_script_into_forms_stops_hard_syntax_errors_from_poisoning_later_forms ... ok
test runtime::tests::context_and_loader::split_script_into_forms_treats_multiline_comments_as_incomplete_trivia ... ok
test runtime::tests::context_and_loader::eval_script_pins_the_second_form_syntax_error_message ... ok
test runtime::tests::context_and_loader::findfile_resolves_package_names_and_file_wrappers ... ok
test runtime::tests::context_and_loader::explicitly_protected_session_configuration_variables_reject_assignments ... ok
test runtime::tests::association_write_index::appended_keys_after_a_wide_read_stay_readable ... ok
test runtime::tests::context_and_loader::get_cycle_termination_does_not_abort_remaining_file_forms ... ok
test runtime::tests::context_and_loader::get_loads_a_script_file_and_returns_the_last_value ... ok
test runtime::tests::association_write_index::repeated_replacement_writes_stay_indexed ... ok
test runtime::tests::context_and_loader::get_propagates_exit_without_collapsing_it_to_failed_load ... ok
test runtime::tests::context_and_loader::get_rolls_back_loader_side_effects_and_collapses_failed_file_messages ... ok
test runtime::tests::context_and_loader::loader_builtins_accept_resolved_load_target_specs ... ok
test runtime::tests::context_and_loader::lex_fallback_preserves_normalized_crlf_values_and_file_error_spans ... ok
test runtime::tests::context_and_loader::needs_accepts_resolved_context_and_file_specs ... ok
test runtime::tests::context_and_loader::missing_findfile_emits_runtime_message ... ok
test runtime::tests::context_and_loader::get_treats_resource_termination_as_failed_load_and_rolls_back ... ok
test runtime::tests::context_and_loader::missing_needs_returns_failed_and_emits_messages ... ok
test runtime::tests::context_and_loader::needs_loads_package_once_and_tracks_loaded_packages ... ok
test runtime::tests::context_and_loader::path_assignment_remains_writable_without_explicit_protect ... ok
test runtime::tests::context_and_loader::nested_get_prefers_the_current_loader_directory_over_package_search_paths ... ok
test runtime::tests::context_and_loader::needs_supports_explicit_file_targets ... ok
test runtime::tests::debug_tools::on_and_off_gate_matching_messages_at_session_scope ... ok
test runtime::tests::arithmetic::wide_plus_and_times_group_late_symbolic_duplicates ... ok
test runtime::tests::debug_tools::short_uses_the_current_canonical_text_truncation_subset ... ok
test runtime::tests::debug_tools::trace_detail_returns_the_full_evaluation_pass_association_without_leaking_trace_state ... ok
test runtime::tests::context_and_loader::get_and_needs_syntax_messages_report_the_loaded_file_location ... ok
test runtime::tests::debug_tools::trace_detail_form_filter_matches_the_underlying_expression_not_the_association ... ok
test runtime::tests::debug_tools::echo_family_prints_and_returns_values_in_the_current_first_pass_subset ... ok
test runtime::evaluation_cache::tests::association_lvalue_observable_or_constrained_environments_use_fallback ... ok
test runtime::tests::debug_tools::rum_profile_hooks_are_inert_without_profile_env ... ok
test runtime::tests::debug_tools::trace_detail_preserves_definition_symbol_kind_and_value ... ok
test runtime::tests::debug_tools::trace_detail_identifies_the_downvalue_that_won_among_overlapping_definitions ... ok
test runtime::tests::debug_tools::trace_detail_has_the_same_list_nesting_shape_as_trace ... ok
test runtime::tests::debug_tools::trace_detail_supports_levelspec_and_heads_options ... ok
test runtime::tests::debug_tools::trace_print_uses_the_same_nested_structure_for_deeper_trace_branches ... ok
test runtime::evaluation_cache::tests::association_lvalue_absent_stale_and_dirty_proofs_recover_by_recomputation ... ok
test runtime::tests::debug_tools::trace_uses_recorded_depths_to_keep_nested_branches_stable ... ok
test runtime::tests::definition_lhs_preparation::adding_hold_attributes_after_an_existing_definition_keeps_old_and_new_identities_live ... ok
test runtime::tests::debug_tools::trace_returns_a_first_pass_structured_expression_list_without_leaking_session_trace_state ... ok
test runtime::tests::debug_tools::trace_print_emits_trace_lines_and_returns_the_underlying_value ... ok
test runtime::tests::definition_lhs_preparation::active_definition_lhs_positions_complete_own_value_evaluation_for_both_set_forms ... ok
test runtime::tests::definition_lhs_preparation::active_nested_calls_dispatch_held_nested_calls_do_not_and_callable_heads_prepare_first ... ok
test runtime::tests::definition_lhs_preparation::clear_then_reinstall_has_no_stale_prepared_identity_or_matcher ... ok
test runtime::tests::definition_lhs_preparation::active_lhs_and_pattern_test_pattern_operands_still_prepare_once ... ok
test runtime::tests::definitions_and_rules::definition_returns_empty_hold_for_symbols_without_definitions ... ok
test runtime::tests::definitions_and_rules::defaultvalues_and_definition_include_explicit_syntax_information ... ok
test runtime::tests::definitions_and_rules::definition_inspection_builtins_accept_symbol_name_strings ... ok
test runtime::tests::definitions_and_rules::defaultvalues_lists_optional_defaults_and_clearall_removes_them ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_pattern_regions_prepare_constraints_but_defer_binders_and_predicates ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_preparation_obeys_evaluate_unevaluated_and_sequence_controls ... ok
test runtime::tests::definitions_and_rules::immediate_self_alias_preserves_downvalues ... ok
test runtime::tests::definition_lhs_preparation::held_definition_lhs_positions_keep_raw_symbols_and_source_lookup_reaches_them ... ok
test runtime::tests::definitions_and_rules::delayed_self_alias_preserves_downvalues ... ok
test runtime::tests::definition_lhs_preparation::reassigned_then_cleared_lhs_own_value_changes_only_future_definition_identities ... ok
test runtime::tests::definition_lhs_preparation::one_metadata_snapshot_controls_every_argument_even_after_an_earlier_mutation ... ok
test runtime::tests::definitions_and_rules::definition_returns_held_first_pass_definition_statements ... ok
test runtime::tests::definition_lhs_preparation::held_ground_flat_and_orderless_identities_are_reachable_from_both_forms ... ok
test runtime::tests::definitions_and_rules::information_uses_catalog_and_symbol_name_usage_fallbacks_and_stored_message_tags ... ok
test runtime::tests::definitions_and_rules::information_returns_first_pass_symbol_summary ... ok
test runtime::tests::definitions_and_rules::ownvalues_lists_immediate_and_delayed_symbol_definitions ... ok
test runtime::tests::definitions_and_rules::set_assigns_simple_symbol_and_lookup_works ... ok
test runtime::tests::definitions_and_rules::self_referential_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules_cases::cases_does_not_treat_heads_as_separate_candidates ... ok
test runtime::tests::comparison_lowering::comparison_parser_rejects_a_long_uniform_chain_gracefully ... ok
test runtime::tests::definitions_and_rules::self_referential_delayed_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules_cases::cases_can_match_the_root_expression ... ok
test runtime::tests::definitions_and_rules::setdelayed_stores_rhs_without_immediate_evaluation ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_collects_nested_matches ... ok
test runtime::tests::definition_lhs_preparation::matcher_wrappers_keep_literal_pattern_regions_raw_and_prepare_nested_constraints ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_applies_rule_transformations ... ok
test runtime::tests::definitions_and_rules::valueq_uses_rollback_trial_evaluation_without_side_effects ... ok
test runtime::tests::definitions_and_rules::dynamic_iterator_lhs_values_are_normalized_with_the_existing_cycle_guard ... ok
test runtime::tests::definitions_and_rules_part2::lowers_named_pattern_assignment ... ok
test runtime::tests::definitions_and_rules_part2::lowers_slots_and_pure_functions ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_preserves_immediate_and_delayed_rule_behavior ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_stay_direct_past_the_cache_size_cap ... ok
test runtime::tests::definitions_and_rules_cases::cases_rule_rhs_splices_named_blanksequence_matches ... ok
test runtime::tests::definition_store_sealing::prepared_store_and_removal_call_site_inventory_is_closed ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_standard_integer_level_specs ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_deletes_top_level_matches_by_default ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_can_substitute_multiple_named_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_traverses_depth_first_with_leaves_before_roots_and_respects_match_limits ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_matches_after_argument_evaluation ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_exact_range_negative_and_infinity_level_specs ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanknullsequence_into_empty_lists ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_supports_standard_levelspecs_and_limits ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_false_for_non_matching_pattern ... ok
test runtime::tests::definitions_and_rules::real_iterator_definitions_precede_dynamic_values_and_clear_reveals_them_again ... ok
test runtime::tests::definitions_and_rules_part2::matchq_operator_form_is_inert_and_applies_to_data ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_can_delete_heads_and_association_values ... ok
test runtime::tests::definitions_and_rules_part2::inspect_definition_catalog_reports_kinds_and_delayedness ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_true_for_matching_pattern ... ok
test runtime::tests::definitions_and_rules_part2::compiled_matcher_matches_interpreted_matcher_on_supported_subset ... ok
test runtime::tests::definitions_and_rules_part2::interpreted_matcher_owned_can_resume_sequence_conditions_across_input ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_explicit_head_traversal_control ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanksequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanknullsequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_optional_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_except_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_conditions_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::named_slots_fill_from_rules_and_association_arguments ... ok
test runtime::tests::definitions_and_rules_part2::setdelayed_stores_and_applies_simple_downvalue ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part2::unmatched_downvalue_leaves_call_in_place ... ok
test runtime::tests::definitions_and_rules_part2::set_supports_immediate_pattern_based_downvalues_and_subvalues ... ok
test runtime::tests::definitions_and_rules::iterator_lhs_normalization_covers_nested_and_shadowed_iterators ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_plain_blank_arguments_under_fixed_heads ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rule_uses_current_rhs_value_at_application_time ... ok
test runtime::tests::definitions_and_rules_part3::immediate_rule_captures_current_rhs_value ... ok
test runtime::tests::definitions_and_rules_part3::replace_rewrites_association_values_without_touching_keys ... ok
test runtime::tests::definitions_and_rules_part3::replace_defaults_to_the_root_level ... ok
test runtime::tests::definitions_and_rules_part3::replace_heads_option_controls_head_traversal ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_preserve_ordinary_symbol_semantics_past_the_cache_size_cap ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_rule_list_in_order ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_single_rule ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_except_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_shape_prefilter_stays_conservative_for_head_attributes ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_traverses_nested_expressions ... ok
test runtime::tests::definitions_and_rules_part3::replace_supports_levelspecs_and_rule_lists ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_part3::flat_and_orderless_flat_heads_regroup_for_match_replace_and_replacelist ... ok
test runtime::tests::definitions_and_rules_part3::replace_and_replaceat_exact_rule_index_preserve_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_preserves_same_name_local_binding_behavior ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_delayed_rule_limit_returns_terminated_value ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_collects_multiple_backtracking_matches_for_sequence_patterns ... ok
test runtime::tests::context_and_loader::parse_script_forms_matches_the_line_driven_splitter_over_the_corpus ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_limit_returns_terminated_value_for_non_terminating_rewrite_cycle ... ok
test runtime::tests::definitions_and_rules_part4::function_accepts_explicit_attributes_for_application ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_supports_operator_form_rule_order_and_max_results ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_reaches_a_fixed_point ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_options_report_new_defaults ... ok
test runtime::tests::definitions_and_rules_patterns::alternatives_downvalue_matches_each_branch ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_max_iterations_option_controls_effective_limit ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_binds_multiple_named_args ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_handles_named_typed_blank_hit ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_alternatives_patterns ... ok
test runtime::tests::association_part_cursor::wide_association_single_key_and_position_reads_return_exact_values ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_keeps_typed_blank_miss_unevaluated ... ok
test runtime::tests::definitions_and_rules_patterns::conditioned_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_family_operator_forms_are_inert_and_apply_to_data ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_preserves_exact_vs_pattern_precedence ... ok
test runtime::tests::definitions_and_rules_patterns::except_downvalue_prefilter_stays_conservative ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_exact_rule_index_preserves_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::except_optional_and_repeated_downvalues_match_expected_shapes ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_keeps_inserted_compound_values_outside_new_locals ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_rejects_malformed_repetition_bounds_with_head_specific_messages ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::longest_and_shortest_control_sequence_pattern_greediness ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_sequence_matching_still_accepts_small_normal_cases ... ok
test runtime::tests::definitions_and_rules_patterns::outer_conditioned_downvalue_is_stored_and_applied ... ok
test runtime::tests::definitions_and_rules_patterns::patterntest_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_patterns::rule_lhs_evaluates_values_without_evaluating_pattern_names ... ok
test runtime::tests::definitions_and_rules_patterns::orderlesspatternsequence_uses_orderless_group_matching_inside_orderless_heads ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_heads_backtrack_across_exact_and_patternsequence_groups ... ok
test runtime::tests::definitions_and_rules_patterns::verbatim_matches_literal_pattern_language_forms ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_reports_normalized_rules_and_delayed_flags ... ok
test runtime::tests::definitions_and_rules_tags::downvalues_lists_stored_pattern_definitions_in_order ... ok
test runtime::tests::definitions_and_rules_patterns::wide_downvalue_store_index_preserves_specificity_and_alpha_replacement ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_and_inspect_rewrite_rules_rebuild_rhs_guards ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_rules_lists_single_rules_rule_lists_and_preserves_delayedness ... ok
test runtime::tests::definitions_and_rules_patterns::patternsequence_flattens_argument_patterns_in_matching_rules_and_definitions ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_index_preserves_pattern_priority_fallback_and_invalidation ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_scan_falls_back_to_live_order_after_matcher_mutation ... ok
test runtime::tests::definitions_and_rules_tags::literal_downvalue_is_prioritized_over_existing_typed_downvalue ... ok
test runtime::tests::definitions_and_rules_tags::explicit_tagunset_removes_tagged_subvalue_definitions ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_upvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_downvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::clearall_and_remove_expand_name_patterns ... ok
test runtime::tests::definitions_and_rules_tags::tagged_subvalues_attach_to_the_head_symbol_not_to_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::subvalues_list_and_apply_nested_head_definitions ... ok
test runtime::tests::definitions_and_rules_tags::tagset_immediate_captures_rhs_before_later_changes ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_subvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::tagset_preserves_tag_ownership_for_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::string_targets_flow_through_clear_tag_and_unset_normalization ... ok
test runtime::tests::definitions_and_rules_tags::tag_definition_messages_distinguish_missing_tags_from_invalid_positions ... ok
test runtime::tests::definitions_and_rules_tags::clear_name_patterns_expand_before_mutation_and_preserve_exact_targets ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_can_suspend_and_resume_with_host_input ... ok
test runtime::tests::definitions_and_rules_tags::typed_upvalues_are_prioritized_over_earlier_generic_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::upset_emits_nosym_when_no_valid_argument_tag_symbol_exists ... ok
test runtime::tests::definitions_and_rules_tags::unset_can_remove_symbol_downvalue_and_upvalue_definitions ... ok
test runtime::tests::definitions_and_rules_tags::upset_stores_immediate_upvalues_on_inferred_argument_heads ... ok
test runtime::tests::definitions_and_rules_tags::upsetdelayed_can_store_on_multiple_inferred_tag_symbols ... ok
test runtime::tests::definitions_and_rules_tags::typed_subvalues_are_prioritized_over_earlier_generic_subvalues ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_candidate_filter_preserves_matching_order_for_valid_tags ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_dispatch_ignores_tags_that_appear_only_below_direct_arguments ... ok
test runtime::tests::definitions_and_rules_tags::upvalues_lists_and_applies_tagged_definitions ... ok
test runtime::tests::definitions_rhs_guards::delayed_downvalue_rhs_guard_accepts_or_declines_without_messages ... ok
test runtime::tests::definitions_rhs_guards::guard_errors_propagate_like_pattern_position_condition_errors ... ok
test runtime::tests::definitions_rhs_guards::downvalues_guard_reconstruction_round_trips_behavior ... ok
test runtime::tests::definitions_rhs_guards::rhs_guard_side_effects_run_before_the_body_is_selected ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_does_not_capture_inserted_values_in_lexical_scopes ... ok
test runtime::tests::definitions_rhs_guards::explicit_true_guard_is_distinct_from_an_unguarded_downvalue ... ok
test runtime::tests::definitions_rhs_guards::immediate_downvalue_rhs_evaluates_completely_before_guard_splitting ... ok
test runtime::tests::definitions_rhs_guards::every_non_true_guard_result_declines_to_the_next_candidate ... ok
test runtime::tests::definitions_rhs_guards::rhs_guards_and_bodies_share_the_definition_option_scope ... ok
test runtime::tests::definitions_rhs_guards::guarded_and_unguarded_downvalues_coexist_in_installation_order ... ok
test runtime::tests::definitions_and_rules::non_iterator_lhs_binding_controls_keep_their_exact_results ... ok
test runtime::tests::definitions_rhs_guards::unset_lhs_removes_every_rhs_guard_variant ... ok
test runtime::tests::definitions_rhs_guards::wide_exact_dispatch_preserves_guard_variant_order ... ok
test runtime::tests::definitions_rhs_guards::same_alpha_canonical_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards::nested_rhs_guards_flatten_in_order_for_both_spellings ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::listable_guarded_downvalue_recompiles_and_enforces_its_guard ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::interleaved_guarded_downvalue_definitions_survive_later_bumps ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::matcher_ownership_guard_is_enforced_after_attribute_revision_invalidation ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::eval_file_rollback_preserves_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_default_values::declining_default_guard_leaves_optional_matching_without_a_default ... ok
test runtime::tests::definitions_and_rules_tags::exact_downvalue_dispatch_preserves_ground_and_pattern_semantics ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::attributes_on_the_guarded_symbol_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_default_values::every_non_true_defaultvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_default_values::nested_defaultvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_own_values::a_later_guarded_ownvalue_replaces_the_single_existing_slot ... ok
test runtime::tests::definitions_rhs_guards_default_values::definition_inspection_rebuilds_all_remaining_definition_guard_forms ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::repeated_attribute_bumps_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_default_values::immediate_and_delayed_defaultvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_own_values::non_true_ownvalue_guards_decline_and_valueq_reports_false ... ok
test runtime::tests::definitions_rhs_guards_default_values::same_defaultvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_own_values::immediate_and_delayed_ownvalue_rhs_guards_accept_or_decline ... ok
test runtime::tests::definitions_rhs_guards_own_values::declining_ownvalue_guards_fall_through_inside_block_and_module ... ok
test runtime::tests::definitions_rhs_guards_default_values::defaultvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_guards_run_left_to_right_and_decline_before_the_body ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::unrelated_set_and_clear_attributes_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_rhs_guard_resumes_without_replaying_the_guard ... ok
test runtime::tests::definitions_rhs_guards_default_values::unset_clears_every_defaultvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::guarded_exact_rule_preserves_wide_plan_order_and_fallback_index ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::valueq_rollback_preserves_every_guarded_lane_and_post_trigger_value ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::every_non_true_rewrite_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::declining_guarded_rules_handoff_in_rule_list_order ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_repeated_stops_when_the_only_guard_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::immediate_and_delayed_rewrite_rules_split_bound_guards ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::nested_rewrite_guards_run_left_to_right_and_second_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rule_shaped_data_keeps_its_existing_condition_forms ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::suspending_rewrite_guard_resumes_same_rule_and_position_without_replay ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_list_guard_decline_keeps_later_binding_enumeration ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_sub_values::same_alpha_canonical_subvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_guard_side_effect_runs_once_per_candidate_tested ... ok
test runtime::tests::definitions_rhs_guards_sub_values::immediate_and_delayed_subvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_rhs_guards_accept_or_decline_for_every_generic_consumer ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_rhs_guards_sub_values::unset_clears_every_subvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_sub_values::every_non_true_subvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_sub_values::nested_subvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_up_values::nested_upvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_up_values::same_alpha_canonical_upvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_up_values::every_non_true_upvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_up_values::immediate_and_delayed_upvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_and_rules_patterns::wide_pattern_downvalue_store_index_replaces_renamed_condition_pattern ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_rhs_guards_up_values::tagunset_clears_every_upvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_tagged_preparation::upset_infers_tags_only_after_preparation_and_keeps_nosym ... ok
test runtime::tests::definitions_tagged_preparation::upset_multiple_owner_protection_is_preserved_after_canonicalization ... ok
test runtime::tests::definitions_unset_preparation::clear_removes_all_identities_and_reinstall_has_no_stale_unset_target ... ok
test runtime::tests::definitions_unset_preparation::unset_after_own_value_rebinding_and_clear_preserves_the_stored_snapshot ... ok
test runtime::tests::definitions_unset_preparation::unset_after_attribute_invalidation_addresses_only_the_new_identity ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_works_after_target_substitution ... ok
test runtime::tests::definitions_unset_preparation::unset_and_postfix_unset_prepare_active_definition_identities ... ok
test runtime::tests::definitions_tagged_preparation::all_tagged_install_entry_points_store_the_prepared_identity_by_owner ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_fails_closed_on_head_mismatch ... ok
test runtime::tests::definitions_value_table_assignment::later_mismatched_rule_cannot_partially_replace_a_value_table ... ok
test runtime::tests::frame_root_trace::association_entry_form_visits_key_and_value_roots_in_field_order ... ok
test runtime::tests::frame_root_trace::block_symbol_state_map_visits_the_bare_symbol_key ... ok
test runtime::tests::definitions_value_table_assignment::downvalues_identity_round_trip_restores_definition_behavior ... ok
test runtime::tests::definitions_unset_preparation::unset_prepares_held_and_pattern_lhs_by_construction ... ok
test runtime::tests::frame_root_trace::enum_shapes_and_generic_arrays_trace_without_special_cases ... ok
test runtime::tests::frame_root_trace::container_implementations_visit_keys_values_and_elements ... ok
test runtime::tests::frame_root_trace::file_io_error_is_the_zero_root_negative_calibration_target ... ok
test runtime::tests::frame_root_trace::expr_is_one_whole_root_and_bare_symbol_is_an_independent_root ... ok
test runtime::tests::frame_root_trace::matcher_binding_table_visits_the_exact_symbol_and_expression_roots ... ok
test runtime::tests::frame_root_trace::nested_derived_fields_are_all_traversed ... ok
test runtime::tests::frame_root_trace::recursive_matcher_arg_visits_every_nested_root_exactly_once ... ok
test runtime::tests::definitions_value_table_assignment::malformed_value_table_rhs_leaves_the_existing_table_unchanged ... ok
test runtime::tests::frame_root_trace::shared_expression_types_borrow_their_expression_roots ... ok
test runtime::tests::definitions_unset_preparation::unset_snapshot_identity_removes_without_rebinding_but_not_after_rebinding ... ok
test runtime::tests::definitions_value_table_assignment::downvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::definitions_value_table_assignment::protected_value_table_target_rejects_replacement ... ok
test runtime::tests::definitions_value_table_assignment::ownvalues_identity_round_trip_restores_the_owned_value ... ok
test runtime::tests::definitions_tagged_preparation::tagged_preparation_preserves_patterns_all_holds_and_callable_heads ... ok
test runtime::tests::definitions_value_table_assignment::successful_value_table_assignment_replaces_instead_of_merging ... ok
test runtime::tests::definitions_value_table_assignment::protection_is_rechecked_after_value_table_lhs_preparation ... ok
test runtime::tests::definitions_tagged_preparation::tagged_snapshot_identity_tracks_attribute_rebind_clear_and_owner_invalidation ... ok
test runtime::tests::definitions_value_table_assignment::prepared_value_table_identity_is_a_snapshot_across_rebind_clear_and_reinstall ... ok
test runtime::tests::definitions_value_table_assignment::value_table_assignment_accepts_immediate_delayed_and_unwrapped_rules ... ok
test runtime::tests::definitions_value_table_assignment::subvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::file_names::file_names_guard_precedes_the_provider_check ... ok
test runtime::tests::file_names::file_names_depth_controls_nested_provider_enumeration ... ok
test runtime::tests::definitions_and_rules_tags::wide_subvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::holding_and_control_io::pause_rejects_unrepresentable_integer_and_rational_durations ... ok
test runtime::tests::definitions_value_table_assignment::upvalues_and_subvalues_identity_round_trips_restore_behavior ... ok
test runtime::tests::file_names::file_names_malformed_specs_remain_inert ... ok
test runtime::tests::file_names::file_names_matches_immediate_entries_and_sorts_and_deduplicates_paths ... ok
test runtime::tests::definitions_value_table_assignment::upvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::holding_and_control_part2::binary_function_syntax_applies_explicit_parameter ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_ownvalues_and_recovers ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_downvalues ... ok
test runtime::tests::holding_and_control_part2::dispatch_plan_uses_builtin_metadata_and_head_traits ... ok
test runtime::tests::holding_and_control_part2::evaluation_pass_limit_bounds_fixed_point_revisits ... ok
test runtime::tests::holding_and_control_io::input_print_and_pause_cover_current_first_pass_io_subset ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_ignore_extra_arguments ... ok
test runtime::tests::holding_and_control_part2::fixed_point_evaluation_revisits_expressions_after_side_effects_change_head_meaning ... ok
test runtime::tests::holding_and_control_part2::function_applies_slots_positionally ... ok
test runtime::tests::holding_and_control_part2::native_stack_limit_stops_unbounded_definition_recursion_and_session_stays_usable ... ok
test runtime::tests::holding_and_control_part2::head_evaluation_can_activate_special_forms_and_held_builtins ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_respect_shadowing_and_pattern_locals ... ok
test runtime::tests::holding_and_control_part2::recursion_limit_stops_expanding_recursive_downvalues ... ok
test runtime::tests::holding_and_control_part2::pure_function_syntax_uses_slots_and_evaluated_arguments ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_attribute_round_trips ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_applies_at_every_application_level ... ok
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_flattens_sequence_like_hold_all ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_strips_and_restores_unevaluated ... ok
test runtime::tests::holding_and_control_part2::tracing_records_definition_rule_and_evaluation_events ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_holds_outer_arguments_and_evaluate_pierces ... ok
test runtime::tests::definitions_and_rules_tags::wide_upvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_cross_family_shape_and_resolver_order ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_message_name_double_resolution_order ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_keeps_direct_canonical_calls_opaque ... ok
test runtime::tests::iterative_loop_drivers::unbounded_while_reaches_the_existing_eval_deadline ... ok
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::iterative_loop_drivers::for_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::left_chain_lowering::flat_chain_evaluated_results_remain_unchanged ... ok
test runtime::tests::iterative_loop_drivers::module_resumes_mid_initializer_without_replaying_bindings ... ok
test runtime::tests::iterative_loop_drivers::until_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_rejects_a_20_000_link_hand_built_ast_gracefully ... ok
test runtime::tests::lowering_and_context::long_statement_chain_lowers_to_flat_nary_compound_expression ... ok
test runtime::tests::lowering_and_context::lowering_rejects_invalid_integer_literals_before_runtime ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_valid_iterators_remain_message_free ... ok
test runtime::tests::lowering_and_context::lowers_big_integer_into_core_expr ... ok
test runtime::tests::lowering_and_context::lowers_multiline_string_literals_like_ordinary_strings ... ok
test runtime::tests::lowering_and_context::lowers_message_name_syntax_into_core_expr ... ok
test runtime::tests::lowering_and_context::lowers_simple_decimal_literals_to_exact_rationals ... ok
test runtime::tests::lowering_and_context::lowers_tagset_and_unset_syntax_into_core_expr ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_variadic_non_flat_chain_shapes ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_messages_and_context_heads ... ok
test runtime::tests::iterative_loop_drivers::while_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_tagged_definition_sugar ... ok
test runtime::tests::definitions_rhs_guards_sub_values::wide_subvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::lowering_and_context::over_cap_statement_chain_line_does_not_swallow_later_lines ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_flat_arithmetic_family_transitions ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_flat_chain_shapes ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_three_link_left_chains ... ok
test runtime::tests::lowering_and_context::evaluates_identity_call ... ok
test runtime::tests::lowering_and_context::attributes_reports_builtin_and_user_defined_attribute_sets ... ok
test runtime::tests::lowering_and_context::compound_expression_sugar_flows_through_lowering_and_evaluation ... ok
test runtime::tests::memory_growth_contract::core_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::memory_growth_contract::known_unprojected_preflight_builtin_allowlist_is_sorted_and_unique ... ok
test runtime::tests::definitions_rhs_guards_up_values::wide_upvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::left_chain_lowering::variadic_non_flat_chain_matching_shapes_are_explicit ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::lowering_and_context::lowering_preserves_tagset_shape_for_runtime_semantic_validation ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_rules_patterns_slots_and_scoping ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume - should panic ... ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch - should panic ... ok
test runtime::tests::hygiene::kernel_crates_do_not_use_std_fs_outside_tests ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::memory_growth_contract::preflight_contract_accepts_recorded_projection ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume - should panic ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_malformed_iterators_emit_argform_messages ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::remaining_attribute_revision_sources_preserve_every_guarded_lane ... ok
test runtime::tests::memory_growth_contract::preflight_contract_remembers_projection_across_input_resume ... ok
test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch - should panic ... ok
test runtime::tests::messages::invalid_message_name_form_emits_runtime_message ... ok
test runtime::tests::messages::message_family_accepts_symbol_name_strings_and_messagename_targets ... ok
test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok
test runtime::tests::messages::builtin_message_registry_populates_listings_and_information ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok
test runtime::tests::messages::delayed_message_templates_are_evaluated_when_emitted ... ok
test runtime::tests::messages::inspect_message_catalog_merges_builtin_templates_and_user_overrides ... ok
test runtime::tests::messages::inspect_reports_message_definition_presence ... ok
test runtime::tests::messages::direct_set_and_setdelayed_calls_share_message_assignment_normalization ... ok
test runtime::tests::messages::inert_message_carriers_list_seeded_tags_and_reject_assignment ... ok
test runtime::tests::messages::invalid_builtin_argument_shapes_emit_runtime_messages ... ok
test runtime::tests::messages::inspect_message_catalog_exposes_templates_and_delayed_flags ... ok
test runtime::tests::definitions_and_rules::iterator_bindings_normalize_set_lhs_for_every_eager_iterator_and_assignment ... ok
test runtime::tests::messages::message_name_and_message_emit_runtime_messages ... ok
test runtime::tests::messages::message_definitions_builtin_lists_immediate_and_delayed_templates ... ok
test runtime::tests::messages::message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages::message_templates_leave_unknown_placeholders_unchanged ... ok
test runtime::tests::messages::message_templates_can_be_defined_and_formatted ... ok
test runtime::tests::messages::push_message_allows_test_local_builtins_without_declared_messages ... ok
test runtime::tests::messages::message_templates_support_message_name_and_all_args_placeholders ... ok
test runtime::tests::messages::message_templates_leave_unsupported_format_specs_unchanged ... ok
test runtime::tests::messages::push_message_rejects_unregistered_builtin_message_tags - should panic ... ok
test runtime::tests::messages::push_message_allows_user_defined_builtin_message_names ... ok
test runtime::tests::messages::message_templates_support_richer_format_specifiers ... ok
test runtime::tests::messages::push_message_allows_open_user_symbol_families ... ok
test runtime::tests::messages::quiet_keeps_suppressed_messages_out_of_message_list ... ok
test runtime::tests::messages::request_message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages_part2::check_can_catch_non_loader_evaluation_errors_via_general_messages ... ok
test runtime::tests::messages::upto_stays_inert_and_selector_diagnostic_stays_exact ... ok
test runtime::tests::messages::message_tags_builtin_lists_global_symbol_and_exact_message_tags ... ok
test runtime::tests::messages_part2::check_and_quiet_accept_message_name_selector_forms ... ok
test runtime::tests::messages_part2::check_can_match_specific_general_error_selectors ... ok
test runtime::tests::messages_part2::check_can_match_by_message_symbol_only ... ok
test runtime::tests::messages::usage_message_assignment_holds_symbol_with_existing_own_value ... ok
test runtime::tests::messages_part2::check_can_target_specific_message_selectors ... ok
test runtime::tests::messages_part2::check_leaves_value_unchanged_when_no_messages_are_emitted ... ok
test runtime::tests::messages_part2::check_returns_fallback_when_messages_are_emitted ... ok
test runtime::tests::messages_part2::cycle_termination_emits_pass_limit_message_at_top_level ... ok
test runtime::tests::messages_part2::quiet_accepts_selector_lists ... ok
test runtime::tests::messages_part2::invalid_builtin_argument_counts_emit_runtime_messages ... ok
test runtime::tests::messages_part2::invalid_message_selector_specs_emit_runtime_messages ... ok
test runtime::tests::comparison_lowering::comparison_lowering_bounds_direct_chain_nodes ... ok
test runtime::tests::messages_part2::explicit_parameter_function_arity_mismatch_emits_runtime_message ... ok
test runtime::tests::messages_part2::quiet_can_selectively_suppress_message_streams ... ok
test runtime::tests::messages_part2::quiet_suppresses_cycle_termination_messages_without_masking_value ... ok
test runtime::tests::messages_part2::quiet_prevents_check_from_observing_suppressed_messages ... ok
test runtime::tests::messages_part2::quiet_suppresses_runtime_messages_from_nested_evaluation ... ok
test runtime::tests::comparison_lowering::comparison_lowering_keeps_parenthesized_subtrees_independent ... ok
test runtime::tests::messages_part3::explicit_non_system_heads_do_not_trigger_builtin_dispatch ... ok
test runtime::tests::messages_part2::unbalanced_context_frames_emit_runtime_messages ... ok
test runtime::tests::messages_part3::beginpackage_accepts_resolved_dependency_lists ... ok
test runtime::tests::messages_part3::check_can_catch_iteration_limit_messages_from_replace_repeated ... ok
test runtime::tests::messages_part3::beginpackage_can_load_dependencies_via_needs ... ok
test runtime::tests::messages_part3::hold_family_respects_builtin_hold_attributes ... ok
test runtime::tests::messages_part3::path_builtin_reflects_package_search_roots ... ok
test runtime::tests::mutation_operator_forms::append_to_modified_list_attributes_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_association_key_part_stays_on_generic_path ... ok
test runtime::tests::comparison_lowering::comparison_lowering_accepts_a_uniform_chain_at_the_boundary ... ok
test runtime::tests::mutation_operator_forms::append_to_capture_defers_its_value_until_application ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_copies_list_shared_at_two_positions ... ok
test runtime::tests::mutation_operator_forms::append_to_deep_exact_part_preserves_middle_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_inner_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_outer_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_rolls_back_multilevel_spine ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_tracing_preserves_existing_trace ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_monitor_preserves_spine_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_non_list_leaf_still_runs_definitions ... ok
test runtime::tests::mutation_operator_forms::append_to_list_downvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_protected_owner_still_refuses ... ok
test runtime::tests::mutation_operator_forms::append_to_item_that_requires_target_evaluation_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_list_upvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_protected_list_target_preserves_value_and_message ... ok
test runtime::tests::mutation_operator_forms::append_to_non_list_head_still_runs_matching_user_code ... ok
test runtime::tests::mutation_operator_forms::append_to_plain_list_preserves_alias_and_returned_value ... ok
test runtime::tests::mutation_operator_forms::append_to_nothing_keeps_list_normalization_semantics ... ok
test runtime::tests::mutation_operator_forms::append_to_rolls_back_if_fast_update_fails_after_definition_take ... ok
test runtime::tests::mutation_operator_forms::append_to_and_add_to_direct_calls_match_their_operator_forms ... ok
test runtime::tests::mutation_operator_forms::append_to_with_monitor_preserves_old_and_new_display_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_resumes_item_input_without_losing_or_replaying_the_update ... ok
test runtime::tests::mutation_operator_forms::append_to_with_tracing_preserves_the_existing_trace ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_keeps_nothing_and_upvalue_behavior ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_heads_advertise_subvalues_hold_all ... ok
test runtime::tests::mutation_operator_forms::mutation_captures_are_quiet_but_invalid_applications_report_argx ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_operator_forms_apply_to_raw_held_targets ... ok
test runtime::tests::nothing::nothing_removal_happens_after_sequence_splicing_for_lists_only ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_spelling_correction_values ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_resolve_context_alias_values ... ok
test runtime::tests::namespace_names::symbol_name_and_name_q_core_anchors ... ok
test runtime::tests::options_and_protection::codec_and_schema_options_are_discoverable ... ok
test runtime::tests::nothing::nothing_resolves_as_system_symbol ... ok
test runtime::tests::options_and_protection::block_localizes_options_alongside_other_symbol_state ... ok
test runtime::tests::options_and_protection::options_assignment_and_queries_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::delayed_option_rules_are_preserved_and_evaluated_on_lookup ... ok
test runtime::tests::options_and_protection::options_assignment_unset_and_clearall_interact_with_symbol_state ... ok
test runtime::tests::namespace_names::names_without_explicit_context_only_sees_visible_contexts ... ok
test runtime::tests::options_and_protection::argumentsoptions_and_checkarguments_cover_current_first_pass_subset ... ok
test runtime::tests::namespace_names::names_accepts_current_options_subset ... ok
test runtime::tests::options_and_protection::optionspattern_and_one_arg_optionvalue_cover_current_first_pass_subset ... ok
test runtime::tests::mutation_operator_forms::mutation_operator_form_preserves_direct_protection_failure ... ok
test runtime::tests::options_and_protection::optionspattern_can_use_an_explicit_target_symbol ... ok
test runtime::tests::options_and_protection::optionvalue_emits_current_optnf_and_nodef_messages ... ok
test runtime::tests::options_and_protection::optionvalue_accepts_lists_of_option_names ... ok
test runtime::tests::options_and_protection::protect_blocks_direct_options_assignment_and_unset_until_unprotected ... ok
test runtime::tests::options_and_protection::protected_targets_reject_attribute_updates_and_preserve_attributes ... ok
test runtime::tests::options_and_protection::setoptions_accepts_target_lists_and_returns_per_target_results ... ok
test runtime::tests::options_and_protection::setoptions_and_optionvalue_match_known_system_options_by_name ... ok
test runtime::tests::options_and_protection::setoptions_blocks_protected_system_head_and_unprotect_allows_update ... ok
test runtime::tests::package_tools::declared_package_stub_autoloads_when_symbol_is_used ... ok
test runtime::tests::options_and_protection::protected_attribute_removal_and_unprotect_remain_escape_hatches ... ok
test runtime::tests::options_and_protection::setoptions_updates_known_options_but_rejects_unknown_ones ... ok
test runtime::tests::options_and_protection::test_create_options_are_discoverable_and_match_consumed_defaults ... ok
test runtime::tests::package_tools::declare_package_declares_names_and_updates_context_path ... ok
test runtime::tests::package_tools::dump_save_context_strings_skip_module_gensyms_but_keep_dollar_globals ... ok
test runtime::tests::package_tools::dump_save_returns_qualified_symbol_names_without_reevaluating_values ... ok
test runtime::tests::options_and_protection::protect_and_unprotect_expand_name_patterns_without_literal_wildcards ... ok
test runtime::tests::package_tools::once_caches_per_session_and_per_location_key ... ok
test runtime::tests::package_tools::dump_save_rejects_unsupported_compression_levels_and_preserves_default_bytes ... ok
test runtime::tests::namespace_names::names_supports_current_first_pass_string_patterns ... ok
test runtime::tests::package_tools::once_accepts_option_rules_in_second_position_and_respects_zero_persistence_subset ... ok
test runtime::tests::package_tools::once_rejects_unimplemented_expiration_policies_without_evaluating_body ... ok
test runtime::tests::package_tools::save_rejects_unknown_options_and_once_reports_unknown_option_names ... ok
test runtime::tests::mutation_operator_forms::append_to_complex_part_selectors_keep_generic_behavior ... ok
test runtime::tests::messages::messages_builtin_lists_defined_message_names ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_shared_child_at_a_second_position ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_root_alias ... ok
test runtime::tests::package_tools::save_and_dumpsave_honor_current_context_filter_and_attribute_options_subset ... ok
test runtime::tests::package_tools::dump_save_overwrites_file_and_returns_saved_symbols ... ok
test runtime::tests::package_tools::save_rebuilds_downvalue_rhs_guards_for_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_keeps_spine_identity_and_leaves_proofs_evicted ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_three_level_middle_alias ... ok
test runtime::tests::package_tools::save_writes_textual_state_that_get_can_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_an_intermediate_list_alias ... ok
test runtime::tests::package_tools::definition_snapshot_rebuilds_all_rhs_guards_for_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_the_selected_child_alias ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_polls_interrupt_before_cache_eviction_or_mutation ... ok
test runtime::tests::package_tools::save_and_dumpsave_accept_names_pattern_objects ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_rolls_back_slot_and_definition_after_replacement ... ok
test runtime::tests::part_assignment::set_positional_part_refuses_a_target_cleared_by_a_selector ... ok
test runtime::tests::pattern_matching_orderless_optional::installed_system_down_values_surface_as_a_whole_after_user_mutation ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_with_sequence_neighbors ... ok
test runtime::tests::part_assignment::set_positional_part_rederives_after_rhs_and_orders_selectors_left_to_right ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_optional_omission_collects_every_match_for_builtin_heads ... ok
test runtime::tests::part_assignment::set_positional_part_supports_flat_nested_and_value_semantics ... ok
test runtime::tests::part_assignment::set_positional_part_rechecks_protection_after_rhs_and_selector_callbacks ... ok
test runtime::tests::part_assignment::set_positional_part_route_preserves_other_lvalues_and_definition_lhs_forms ... ok
test runtime::tests::definition_store_sealing::definition_store_and_targeted_removal_are_sealed_behind_prepared_tokens ... ok
test runtime::tests::pattern_matching_orderless_optional::one_identity_promotes_singletons_when_remaining_arguments_are_omissible ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_omission_uses_explicit_defaults ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_present_matches_and_binds_in_documented_order ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_dot_uses_head_default ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_other_args_matches_present_and_omitted ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_matches_collects_and_omits ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_sequence_neighbors_uses_documented_backtracking_order ... ok
test runtime::tests::reap::reap_buckets_stay_in_first_appearance_order ... ok
test runtime::tests::registered_builtin_resolution::registered_builtin_absent_from_static_catalog_resolves_as_system_symbol ... ok
test runtime::tests::reap::nested_reap_scopes_keep_their_own_buckets ... ok
test runtime::tests::pattern_matching_orderless_optional::snapshot_after_unset_resurrects_an_installed_system_down_value ... ok
test runtime::tests::reap::reap_patterns_select_which_sows_are_captured ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_nonmatch_edges_stay_false ... ok
test runtime::tests::reap::reap_index_preserves_wide_order_and_appends_to_existing_buckets ... ok
test runtime::tests::part_assignment::set_positional_part_selector_resume_is_single_pass_and_rederives_the_owner ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_operator_form_applies_to_multiple_inputs ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_uses_original_and_compound_head_attributes ... ok
test runtime::tests::rule_rhs_materialization::held_delayed_rhs_firing_is_consumer_independent ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_rule_shape_messages_match_replace_all ... ok
test runtime::tests::scoping_gensym_gc::all_live_modules_run_one_sweep_before_the_next_watermark ... ok
test runtime::tests::scoping_gensym_gc::clear_all_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_watermark_arithmetic_saturates ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_wrapping_a_sequence_binding_keeps_its_literal_head ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_masks_only_held_argument_positions ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_splicing_is_deferred_until_replacement_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::replaceat_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::rule_rhs_materialization::replaceat_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::replacement_produced_sequence_splices_into_held_parent_rebuild ... ok
test runtime::tests::scoping_gensym_gc::immediate_module_reclaim_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::rule_rhs_materialization::replacepart_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::top_level_sequence_replacement_has_no_parent_to_splice_into ... ok
test runtime::tests::rule_rhs_materialization::replacepart_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::rule_rhs_materialization::other_rewrite_traversals_share_replacement_sequence_splicing ... ok
test runtime::tests::scoping_gensym_gc::loader_snapshot_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::rule_rhs_materialization::sequence_pattern_bindings_keep_structural_splicing ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_threshold_parses_env_in_fresh_processes ... ok
test runtime::tests::scoping_gensym_gc::attribute_state_on_dead_candidate_is_cleared_during_reclaim ... ok
test runtime::tests::rule_rhs_materialization::unchanged_literal_sequence_does_not_splice_during_rebuild ... ok
test runtime::tests::rule_rhs_materialization::replacement_sequence_splice_respects_parent_attributes ... ok
test runtime::tests::scoping_gensym_gc::partial_sweep_rearms_from_the_post_sweep_candidate_count ... ok
test runtime::tests::scoping_gensym_gc::remove_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::delayed_own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::empty_candidate_set_sweep_is_a_noop ... ok
test runtime::tests::scoping_gensym_gc::downvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::down_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::live_candidate_definition_marks_transitive_candidate_dependencies ... ok
test runtime::tests::scoping_gensym_gc::global_root_keeps_escaped_candidate_until_root_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::module_throw_payload_root_retains_its_gensym_definition ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::executing_frame_and_queued_suffix_remain_rooted_during_resume_sweep ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::block_frame_bare_symbol_is_reclaimed_after_resume_and_next_sweep ... ok
test runtime::tests::part_assignment::set_positional_part_observable_constrained_and_dynamic_scopes_use_generic_path ... ok
test runtime::tests::scoping_gensym_gc::message_root_escape_keeps_candidate_until_message_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::module_error_path_definition_escape_stays_deferred ... ok
test runtime::tests::scoping_gensym_gc::module_result_root_retains_only_its_transitive_local_chain ... ok
test runtime::tests::scoping_gensym_gc::outcome_extra_roots_are_empty_for_non_value_errors_and_candidate_is_reclaimed ... ok
test runtime::tests::scoping_gensym_gc::nested_module_activations_keep_their_candidates_until_each_activation_is_removed ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::matcher_guard_binding_survives_a_sweep_while_suspended ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_b_resume_sweep_keeps_remaining_frame_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_a_second_request_keeps_frame_only_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::option_root_escape_keeps_candidate_until_options_are_cleared ... ok
test runtime::tests::scoping_gensym_gc::unreachable_candidates_are_reclaimed_at_the_next_watermark_boundary ... ok
test runtime::tests::scoping_gensym_gc::suspended_computation_keeps_candidate_live_until_resume_finishes ... ok
test runtime::tests::scoping_gensym_gc::returned_expr_extra_root_keeps_candidate_definition_live ... ok
test runtime::tests::scoping_gensym_gc::subvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::large_frame_state_trips_memory_limit_and_session_recovers ... ok
test runtime::tests::scoping_gensym_gc::sub_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_unique::unique_generates_fresh_symbols_for_current_prefix_subset ... ok
test runtime::tests::scoping_gensym_gc::nested_module_error_reclamation_mirrors_success_conservatism ... ok
test runtime::tests::scoping_gensym_gc::unique_generated_symbols_are_not_module_gc_candidates ... ok
test runtime::tests::scoping_unique::unique_reports_current_invalid_boundaries ... ok
test runtime::tests::scoping_unique::unique_skips_existing_generated_names ... ok
test runtime::tests::scoping_gensym_gc::syntax_information_root_escape_keeps_candidate_until_syntax_info_is_cleared ... ok
test runtime::tests::scoping_unique::unique_registers_generated_symbols_as_known_names ... ok
test runtime::tests::scoping_gensym_gc::transitive_definition_kind_chain_keeps_all_reachable_candidates_live ... ok
test runtime::tests::scoping_gensym_gc::up_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::sealed_packages::declaration_validation ... ok
test runtime::tests::scoping_gensym_gc::upvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::sealed_packages::context_validation_returns_the_call_inert ... ok
test runtime::tests::sealed_packages::internal_snapshot_round_trips_registry ... ok
test runtime::tests::sealed_packages::decoy_assignment_is_distinct ... ok
test runtime::tests::sealed_packages::package_install_resumes_at_the_next_form ... ok
test runtime::tests::sealed_packages::public_reaches_sealed_private ... ok
test runtime::tests::sealed_packages::private_state_inside_package ... ok
test runtime::tests::sealed_packages::package_scope_is_fqn_reachable ... ok
test runtime::tests::part_assignment::set_positional_part_invalid_targets_report_without_protected_part_fallback ... ok
test runtime::tests::scoping_gensym_gc::module_unreferenced_error_paths_reclaim_gensyms_immediately ... ok
test runtime::tests::sealed_packages::registration_failure_cleans_sealed_state ... ok
test runtime::tests::sealed_packages::remove_sealed_writes_no_generation ... ok
test runtime::tests::sealed_packages::re_registration_replaces_in_place ... ok
test runtime::tests::sealed_snapshot::marker_refused_in_ordinary_evaluation ... ok
test runtime::tests::sealed_snapshot::sealed_free_snapshot_bytes_unchanged ... ok
test runtime::tests::sealed_packages::spelled_private_name_is_ordinary ... ok
test runtime::tests::sealed_snapshot::save_refuses_sealed_references ... ok
test runtime::tests::sealed_snapshot::information_sealed_unit_row ... ok
test runtime::tests::sealed_visibility::definition_hides_sealed_body ... ok
test runtime::tests::sealed_visibility::completion_hides_sealed ... ok
test runtime::tests::sealed_visibility::contexts_hides_private_context ... ok
test runtime::tests::sealed_visibility::completion_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::contexts_reveals_with_diagnostic ... ok
test runtime::tests::sealed_snapshot::snapshot_refuses_orphans ... ok
test runtime::tests::session_config::session_with_config_normalizes_contexts_paths_and_limits ... ok
test runtime::tests::sealed_visibility::definition_reveals_with_diagnostic ... ok
test runtime::tests::session_memory::byte_array_proxy_tracks_encoded_string_capacity_and_preserves_invalid_fallback ... ok
test runtime::tests::session_memory::session_memory_field_decision_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::session_memory::session_memory_walk_counts_declared_exception_storage ... ok
test runtime::tests::session_memory::stability_blooms_are_fixed_size_inline_values ... ok
test runtime::tests::scoping_gensym_gc::outcome_extra_roots_keep_success_return_throw_and_cleanup_candidates_live ... ok
test runtime::tests::sealed_visibility::decoy_visibility_is_callers_own ... ok
test runtime::tests::sealed_visibility::information_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::nameq_stays_false ... ok
test runtime::tests::sealed_visibility::information_hides_sealed ... ok
test runtime::tests::session_config::default_session_configuration_matches_default_session_state ... ok
test runtime::tests::session_config::random_u64_sequences_are_pinned_for_byte_and_integer_seeds ... ok
test runtime::tests::session_config::apply_config_updates_host_settings_without_clearing_existing_definitions ... ok
test runtime::tests::sealed_visibility::nested_registration_keeps_registry_consistent ... ok
test runtime::tests::session_config::trace_message_and_print_accessors_expose_and_take_buffers ... ok
test runtime::tests::session_config::session_setters_update_limits_capabilities_asserts_and_context_lookup ... ok
test runtime::tests::sealed_visibility::save_excludes_sealed ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_restores_seal ... ok
test runtime::tests::sealed_visibility::names_reveals_with_diagnostic ... ok
test runtime::tests::sealed_snapshot::snapshot_persists_public_mutation ... ok
test runtime::tests::session_requests::api::complete_request_returns_transport_friendly_matches ... ok
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... ok
test runtime::tests::sealed_visibility::names_hides_sealed ... ok
test runtime::tests::session_requests::api::accepted_cancel_abandons_input_before_conversion_or_continuation ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_relinks_leaked_values ... ok
test runtime::tests::sealed_visibility::wildcard_ops_skip_sealed ... ok
test runtime::tests::session_requests::api::completed_request_handle_is_inert_for_later_requests ... ok
test runtime::tests::session_requests::api::dropped_accepted_ticket_and_generation_rotation_retire_handles ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_success_data ... ok
test runtime::tests::session_requests::api::eval_request_nested_memory_constraint_catches_before_higher_request_ceiling ... ok
test runtime::tests::session_requests::api::eval_expr_request_aggregates_messages_and_classifies_budget_termination ... ok
test runtime::tests::session_requests::api::eval_request_memory_ceiling_returns_classified_termination_and_session_recovers ... ok
test runtime::tests::session_requests::api::eval_request_take_pending_input_request_moves_and_clears_the_request ... ok
test runtime::tests::session_requests::api::eval_request_surfaces_structured_input_requests ... ok
test runtime::tests::session_requests::api::is_complete_request_distinguishes_complete_incomplete_and_invalid_input ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_cycle_termination_data ... ok
test runtime::tests::session_requests::api::inspect_request_aggregates_symbol_definitions_and_messages ... ok
test runtime::tests::session_requests::api::parse_request_returns_structured_syntax_diagnostics ... ok
test runtime::tests::session_requests::api::stale_cancel_cannot_overwrite_current_request_cancel ... ok
test runtime::tests::session_requests::api::ordinary_request_and_lazy_observer_leave_interrupt_poll_unarmed ... ok
test runtime::tests::session_requests::api::accepted_script_and_file_stop_before_later_forms_after_cancellation ... ok
test runtime::tests::session_requests::api::cancellation_observer_held_across_session_rebuild_is_inert ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_reap_handler_rendering ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_trace_capture ... ok
test runtime::tests::session_requests::api::eval_request_print_flood_trips_memory_ceiling_with_classified_termination ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_repeated_timing_samples ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_replace_all_without_restarting_target ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_finish_delayed_message_templates ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend ... ok
test runtime::tests::session_requests::input_resume::queued_input_responses_feed_inputstring_without_prompting_again ... ok
test runtime::tests::session_requests::input_resume::nested_input_suspension_precedes_the_old_suffix_without_reordering_effects ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_suspend_multiple_times_without_restarting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_dialog_subsets ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_with_bindings_without_reprompting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_keeps_monitor_display_and_finishes_the_body ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_reports_input_parse_errors_as_syntax_diagnostics ... ok
test runtime::tests::hygiene::kernel_crates_do_not_write_stdout_outside_tests_and_bins ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_formats_delayed_message_templates_after_resume ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_block_scope_until_completion ... ok
test runtime::tests::session_requests::api::file_request_maps_completed_syntax_io_and_missing_provider_errors ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_skips_uninitialized_module_bindings ... ok
test runtime::tests::session_requests::matcher_resume::downvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::defaultvalue_rhs_guard_resumes_during_matching_without_replaying_the_candidate ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fifth_arg ... ok
test runtime::tests::file_names::file_names_infinity_stops_at_the_fixed_entry_cap_without_recursing_the_stack ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fourth_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_level_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_third_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_cases ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_downvalue_matching ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::cases_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::guarded_replacement_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::matcher_binding_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_count_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::matcher_resume::nested_rule_application_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_freeq_with_suspending_heads_option ... ok
test runtime::tests::session_requests::matcher_resume::ordered_single_arg_cursor_resumes_without_replaying_the_matched_prefix ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_nested_matcher_inside_sequence_conditioned_matchq ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_a_boundary_hand_built_alternating_ast ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_sequence_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_multi_sequence_backtracking_conditions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_replace_all ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_orderless_patternsequence_group_matching ... ok
test runtime::tests::session_requests::matcher_resume::subvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::upvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_pattern_symbol ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_match_limit ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_target ... ok
test runtime::tests::sealed_visibility::message_catalog_reveals_with_diagnostic ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_normalize_rules_after_suspending_rule_source ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_target ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_levelspec ... ok
test runtime::tests::session_requests::transforms_resume::normalized_rule_expr_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::sealed_visibility::message_catalog_hides_sealed ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceall_after_suspending_rule_source ... ok
test runtime::tests::session_requests::matcher_resume::traversal_count_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_position_spec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_after_suspending_pattern_position_spec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_multiple_backtracking_matches ... ok
test runtime::tests::session_requests::matcher_resume::sequence_binding_vec_continuation_roundtrip_preserves_multiple_bindings ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_with_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::compiled_rule_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume_patterns::eval_request_reports_prints_on_failure_and_across_input_resume ... ok
test runtime::tests::session_requests::transforms_resume::replace_levelspec_option_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_except_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_setattributes_with_suspending_attribute_spec ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_sequence_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume::replacepart_resumes_mid_walk_with_the_same_rule_and_partial_result ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_thread_with_suspending_arg_spec ... ok
test runtime::tests::structure_data::replace::replacepart_rule_lists_preserve_sequential_conflicts_and_missing_positions ... ok
test runtime::tests::structure_data::replace::replaceat_supports_exact_positions_operator_form_and_rule_lists ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_delayed_rhs ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_declining_later_guard_hands_position_to_earlier_rule ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_repeated_patterntest_matchq ... ok
test runtime::tests::structure_data::replace::replacepart_handles_duplicate_and_overlapping_targets_more_stably ... ok
test runtime::tests::structure_data::replace::replacepart_span_preserves_lenient_missing_positions_and_raw_delayed_rhs ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_acceptance_preserves_raw_delayed_bodies ... ok
test runtime::tests::structure_data::replace::replacepart_spans_reuse_part_selection_and_fail_closed_on_malformed_shapes ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_unguarded_rules_keep_explicit_later_rule_wins_behavior ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_evaluates_each_rule_position_guard_at_most_once ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_suspension_resumes_same_rule_position_and_plan ... ok
test runtime::tests::structure_data::selectors_part2::invalid_span_messages_render_infix_specs ... ok
test runtime::tests::structure_data::replace::replacepart_supports_pattern_position_specs_and_rhs_bindings ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_emit_single_take_for_explicit_invalid_spans ... ok
test runtime::tests::structure_data::selectors_part2::invalid_upto_span_diagnostics_are_not_duplicated_after_normalization ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_rhs_guards_accept_and_decline_exact_position_forms ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_contested_guards_run_last_to_first_and_stop_after_acceptance ... ok
test runtime::tests::structure_data::selectors_part2::direct_empty_span_selects_full_range ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_take_messages_render_span_ranges ... ok
test runtime::tests::structured_exceptions::exception_type_registry_round_trips_through_loader_snapshot ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_report_branch_selector_failures ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_pattern_guards_receive_bindings_and_option_scope ... ok
test runtime::tests::structured_exceptions::suspending_exception_handler_guard_resumes_its_scan_cursor_without_replay ... ok
test runtime::tests::structure_data::selectors_part2::direct_single_endpoint_span_selects_prefix_or_suffix_ranges ... ok
test runtime::tests::structured_exceptions::declining_exception_handler_guard_continues_ancestor_and_handler_scans ... ok
test runtime::tests::structure_data::replace::replacepart_supports_immediate_delayed_and_head_replacements ... ok
test runtime::tests::suspension_abandonment::abandonment_generations_are_monotonic_and_pure_frames_run_no_continuation ... ok
test runtime::tests::session_requests::api::script_requests_report_file_coordinates_for_every_form_shape ... ok
test runtime::tests::suspension_abandonment::abandonment_finishes_module_activation_without_initializers_or_body ... ok
test runtime::tests::suspension_abandonment::abandonment_evaluates_only_withcleanup_cleanup_and_abandons_nested_input_lifo ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_random_prng_state ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_an_existing_time_deadline_exactly ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_abort_protect_and_check_abort_checkpoints ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_reap_without_rendering_or_running_body_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_monitor_without_a_final_refresh_or_callback ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_option_value_scope_without_running_definition_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_both_loop_iterator_scope_forms_and_iteration_budget ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_initializers_and_bodies_inner_to_outer ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_test_report_stream_capture_without_a_report ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_check_and_quiet_captures_without_check_fallback ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_constraint_nesting_without_fallbacks ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_preserves_its_captured_generation ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_trace_and_evaluation_data_captures_without_rendering ... ok
test runtime::tests::suspension_abandonment::abandonment_rolls_back_partial_package_install_without_reporting_failure ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_abandons_a_trial_generation_before_restoring_state ... ok
test runtime::tests::suspension_abandonment::queued_input_abandons_the_older_generation_before_consuming_the_value ... ok
test runtime::tests::syntax_information_debug_catalog::syntax_information_covers_current_cl09_debugging_surface_subset ... ok
test runtime::tests::thread_and_dispatch::evaluated_head_function_attributes_control_argument_evaluation ... ok
test runtime::tests::syntax_information_lifecycle::unsetting_system_syntax_information_reverts_to_default_rules ... ok
test runtime::tests::syntax_information_lifecycle::protect_blocks_direct_syntax_information_assignment_and_unset ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_defaults_assignment_and_unset_cover_current_first_pass_subset ... ok
test runtime::tests::thread_and_dispatch::downvalue_miss_stability_shortcut_invalidates_on_later_definition ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_participates_in_block_and_clearall ... ok
test runtime::tests::thread_and_dispatch::block_localizes_definitions_attributes_and_messages ... ok
test runtime::tests::thread_and_dispatch::held_downvalue_matching_normalizes_nested_attribute_heads ... ok
test runtime::tests::thread_and_dispatch::listable_interacts_with_builtin_and_function_attributes ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_standard_sequence_spec_subset ... ok
test runtime::tests::thread_and_dispatch::argument_preparation_cow_preserves_dispatch_semantics ... ok
test runtime::tests::thread_and_dispatch::listable_mismatched_lengths_emit_thread_message_and_leave_call_unevaluated ... ok
test runtime::tests::thread_and_dispatch::flat_attribute_flattens_calls_and_matches_flat_patterns ... ok
test runtime::tests::thread_and_dispatch::listable_threads_user_defined_calls_and_broadcasts_scalars ... ok
test runtime::tests::thread_and_dispatch::module_skips_existing_generated_symbol_names ... ok
test runtime::tests::thread_and_dispatch::one_identity_allows_singleton_head_patterns_to_match_atoms ... ok
test runtime::tests::session_requests::transforms_resume::replace_and_replaceat_indexed_rule_plans_survive_mid_traversal_suspension ... ok
test runtime::tests::thread_and_dispatch::stable_call_shortcut_preserves_symbol_and_upvalue_evaluation ... ok
test runtime::tests::thread_and_dispatch::orderless_attribute_canonicalizes_calls_and_rules ... ok
test runtime::tests::thread_and_dispatch::thread_evaluates_its_first_argument_before_threading ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_default_and_custom_thread_heads ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_accepts_resolved_head_and_spec_values ... ok
test runtime::tests::thread_and_dispatch_part2::thread_builtin_mismatch_emits_message_and_stays_unevaluated ... ok
test runtime::tests::session_requests::api::file_request_maps_resource_limit_exit_and_cancel_errors ... ok
test runtime::tests::traversal_part2::cases_and_position_report_invalid_match_limit_messages ... ok
test runtime::tests::traversal_part2::count_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_wide_dense_argument_selection ... ok
test runtime::tests::traversal_part2::level_reports_invalid_levelspec_messages ... ok
test runtime::tests::traversal_part2::freeq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::memberq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::position_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_populates_only_evaluation_caches ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_skips_snapshot_and_suppression_even_after_attribute_revision ... ok
test runtime::tests::wide_child_traversals::cases_resumes_mid_walk_between_children ... ok
test runtime::tests::traversal_part2::traversal_family_reports_invalid_levelspec_messages ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::valueq_stable_shortcut::value_q_stability_shortcut_matches_the_trial_path_over_one_shared_corpus ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_to_next_candidate_when_condition_fails ... ok
test runtime::tests::left_chain_lowering::left_chain_parser_rejects_20_000_links_gracefully_for_all_operators ... ok
test runtime::tests::scoping_gensym_gc::recursive_module_sweep_reclaims_dead_retained_gensyms ... ok
test runtime::tests::wide_child_traversals::position_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::delete_cases_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::replace_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::replace_all_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::count_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::cases_walks_thirty_thousand_children ... ok
test runtime::tests::structure_data::replace::replacepart_handles_twenty_thousand_structural_rules_in_one_walk ... ok
test runtime::tests::wide_definition_dispatch::replace_list_walks_thirty_thousand_rules ... ok
test runtime::tests::wide_definition_dispatch::reap_walks_thirty_thousand_buckets ... ok
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok
test runtime::tests::thread_and_dispatch::module_handles_many_uninitialized_bindings_without_exhausting_the_native_stack ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_the_boundary_for_all_operators ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_walks_thirty_thousand_definitions ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_walks_thirty_thousand_definitions ... ok
test runtime::tests::iterative_loop_drivers::until_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::module_drives_thirty_thousand_initialized_bindings_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::while_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::for_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok

test result: ok. 953 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.88s

   Doc-tests rum_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
