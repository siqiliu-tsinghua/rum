# brief-431 — bound the candidate normalizer's native recursion

Step 3 of three, and the one that changes behavior. Step 1 (brief-429,
`66faf671`) locked the copy-on-write contract in tests; step 2 (brief-430,
`8a94cef0`) moved the cluster into its own module. Everything you need to
preserve is already expressed as a failing-if-broken test.

## Goal

Replace the unbounded recursion in
`crates/rum-runtime/src/runtime/resolution/match_normalization.rs` with a
depth-bounded recursive front path plus a cold explicit-stack fallback, so that
normalizing a deep candidate no longer grows the native stack in proportion to
the candidate's height — while producing bit-for-bit the same results, including
which input `Arc`s the output reuses.

The measured ceiling this removes, on a 2 MiB stack (`ulimit -s 2048`, release
CLI), values built with `x = 1; Do[x = {x}, {i, N}]`, height N+1, each run's
height confirmed by `Depth[x]` before the operation:

| Probe | Before |
|---|---|
| `MatchQ[x, _?IntegerQ]` — one match, no traversal | OK at 4,601; stack overflow at 4,651 |
| `Count[x, _, Infinity]` / `Cases[x, _, Infinity]` / `Position[x, 1]` | OK at 4,501; overflow at 4,701 |
| `MatchQ[x, _]` (blank fast path, never enters the normalizer) | OK at 20,001 |
| `Depth[x]` | clears 20,001 |

A single general match dies where an all-level traversal over the same value
does, which is why this one function is the whole target and why the acceptance
certificate is a single `MatchQ`.

## Why this is not just a loop rewrite

**Pointer identity is an observable contract here.** The evaluation-stability
cache validates entries by exact `SharedExpr`/`SharedArgs` pointer equality
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:332-349`), and its
uncached analysis is itself a recursive walk
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33`). An
implementation that returns a logically equal but freshly allocated call would
pass every value assertion, lose cache hits, and reintroduce a recursion behind
the one you just removed. `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs`
already fails on exactly that mistake — I verified it fires by making the args-only
arm allocate a new head.

Three ordering facts the current code depends on:

- The head is normalized **before** the arguments, and the attribute lookup uses
  the normalized head.
- `flatten_flat_call_args` flattens **one level only**
  (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`). A nested
  `Flat` chain flattens fully only because children finish before parents. Change
  the order and it silently under-flattens.
- The argument vector is lazy until the first changed child; unchanged children
  after it are shallow clones.

## Design — what to build

1. **`MATCH_NORMALIZER_RECURSIVE_CALL_DEPTH: usize = 16`**, with a comment
   naming the two precedents it follows:
   `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-7`
   (`RECURSIVE_TRANSFORM_CALL_DEPTH`, `Map`/`Apply`) and
   `crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-80` (`Level`).
   Sixteen is precedent, not a measured optimum; say so in the comment rather
   than implying it was derived.
2. **A distinct depth-exhaustion outcome.** "Completed unchanged" is `None` and
   must stay unmistakable for "retry iteratively": use a type the compiler
   distinguishes, not a sentinel or a boolean out-parameter. The budget
   decrements once per `Expr::Call` visited, whether reached through a head or
   an argument; atoms complete on a zero budget.
3. **The recursive front path**, keeping today's lazy prefix: no args vector is
   allocated until the first child returns `Some`. On exhaustion, propagate to
   the root and return no partial result.
4. **A cold explicit-stack fallback**, `#[inline(never)]`, whose work vectors
   are allocated only when it runs. `Visit(Call)` schedules its finish item,
   then its arguments in reverse, then its head, so visitation stays head-first
   and arguments left-to-right; `FinishCall` consumes one head result and
   `args.len()` argument results.
5. **One shared finisher for the argument-attribute step and one for the four
   rebuild arms, called by both engines.** This is the anti-drift mechanism and
   it is not optional: the two high-risk branches — the four-arm rebuild and the
   equality collapse at the end of the argument finisher — must exist once in
   the source, or the engines will diverge under a later edit.
6. **Restart from the requested root** when the front path is exhausted, as
   `Level`, `Map` and `Apply` do. On a unary spine that discards sixteen visited
   calls; on a broad tree whose first deep branch is late it can approach one
   extra visit. Accept that for this step; if the depth-65 gate below fails,
   stop rather than redesigning the fallback under time pressure.
7. Keep `normalize_definition_identity_for_matching` as its own wrapper with its
   own hold gate and outer rebuild arms, routing each head/argument child through
   the shared core. Do not replace it with a call to `normalize_match_expr`;
   that would apply a different outer policy.

**Not in this brief**: any change to `Expr`, `SharedExpr`, `SharedArgs` or their
constructors (brief-426 measured 30.04% for construction-side work and was
rejected); any new configuration knob (ruled out — the design extends the
existing `recursion_limit` + `native_stack_limit` pair rather than adding a
third); `Cases`' duplicate normalization and per-node pattern recompilation, both
real and both deliberately deferred so this A/B stays attributable.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution/match_normalization.rs` — the repair.
  - `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs` —
    extend the corpus to drive both engines (below). **Do not weaken or delete an
    existing assertion.** If one now fails, that is the finding; stop and report.
  - `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs` — the new
    certificates. Add a height-parameterized sibling helper and have the existing
    `evaluate_certificate` delegate to it; do not change that function's
    signature, because ten existing certificates call it.
  - Appending your report to
    `docs/dev/codex/brief-431-bounded-candidate-normalizer.md`.
- MUST NOT touch: `crates/rum-core/src/expr.rs`, `resolution.rs`,
  `call_preparation.rs`, any caller of the normalizer, and every other file.

## Tests you must add

**Equivalence, in the contract module.** Today's corpus compares production
against the frozen reference. Extend it so that, for every corpus case, *three*
producers are compared against the frozen reference on all three axes (`Option`
discriminant, logical value, whole-tree `Arc`-origin signature):

- the ordinary entry,
- the bounded front path forced at budgets on both sides of the boundary,
- the iterative fallback invoked directly.

Add an argument spine and a head spine at depth 17 (crossing the boundary) and
at 65, changed and unchanged variants of each. Depth 65 stays within what the
frozen recursive reference can itself handle.

**Deployment certificates, in the stdlib 2 MiB harness.** Two separate tests,
both `#[ignore]`d in the existing release-only style:

- *The certificate.* At physical height 20,001, evaluate exactly
  `MatchQ[x, _?IntegerQ]` and assert `False`. Nothing else in the timed body —
  `_?IntegerQ` is required because a bare `_` returns through the blank fast
  path before reaching the normalizer
  (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:106-131`).
- *A liveness test.* Same construction, asserting `Depth[x]` is `20001`, so a
  future regression in the construction cannot make the certificate vacuous.
  Keep it a separate test: I measured `Depth` clearing 20,001 on 2 MiB, so it is
  safe here, but the certificate must fail for exactly one reason.

## The falsifier (MANDATORY — this is the acceptance ordering)

**Add the certificate first and watch it fail, before you change any production
code.** Build the release binary from the current tree, run the certificate, and
record the verbatim stack-overflow failure. Only then write the repair, and
afterwards the identical test must pass.

**If the certificate passes before the repair, STOP and report.** It is then not
calibrated and cannot certify anything. A check nobody has watched fail is not
known to work; that rule cost this project two wrong declarations and it is not
negotiable here.

## Verification (MANDATORY — you run it; paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo test -p rum-stdlib --release iterative_traversal_depth -- --ignored
```

If your sandbox cannot bind loopback, the nine native ZeroMQ tests fail loudly
by design; use the documented opt-out and say in the report that you did, so I
know that part of your green is untested on your host. I re-run the workspace
gate here where those tests actually execute.

**The A/B, required** (this is the matcher's hot entry):

```
cargo build --release -p rum-cli            # BASE, before any edit
cp target/release/rum-cli /tmp/rum-cli-base
# ... make the change ...
cargo build --release -p rum-cli            # CANDIDATE
bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

- **Affected panel**: `matchq-integer-success`, `matchq-integer-fail`,
  `matcher-single-tiny`, `matcher-single-wide-small`, `matcher-single-wide-large`,
  `replaceall-simple`, `rewrite-guarded-hit`, `rewrite-guarded-decline-fallback`,
  `cases-wide-no-match`, `count-wide-no-match`, `position-wide-no-match`,
  `deletecases-wide-no-match`, `deletecases-wide-association-no-match`,
  `replace-wide-no-match` (`scripts/core-language-microbench.sh:538-598`).
- **Named controls**, from which you compute the whole-run offset:
  `literal-integer`, `literal-string`, `symbol-ownvalue-read`, `loop-empty`,
  `sameq-integer`, `head-list`, `length-list`, `part-list`, `if-true`,
  `module-local`, `block-local`, `with-local`, `table-small`, `do-small`.
  **Report the control median first, then every affected row adjusted by it.** A
  raw direction split is not evidence: a run of the identical script has read
  −4.07% with 7 of 38 labels up, and family A's gate carried a +1.81% whole-run
  offset that reads as a regression until the controls are consulted.

## Stop-loss

- Any existing contract assertion fails → stop and report; do not adjust the test.
- The certificate passes before the repair → stop and report.
- Offset-adjusted coherent slowdown above 3% across a family, or a single label
  above 5% reproduced in a second A/B and outside the base/base spread → stop and
  report. Do not tune the constant, move work into constructors, or fold in the
  deferred work-removal items to buy the number back.
- The same step fails twice for the same reason → stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; I run the authoritative gate and commit.
- Do NOT modify files outside Scope. If you believe you must, STOP and say so.
- No network. Touch nothing outside this repository.
- **No new `unsafe`** — `rum-core` and `rum-runtime` are zero-`unsafe`.
- No default-path cost beyond the one budget decrement the design requires; no
  allocation on the shallow path.
- Suspension: the normalizer takes `&self` and cannot evaluate or return
  `InputRequired`. Your work stacks are local `Vec`s and depend on that. Do not
  introduce anything that could suspend inside them.
- Bound long commands with `scripts/run-bounded.py --timeout SECONDS`, sized from
  a measured run of the same shape. Profiling env OFF for every timing number.
- Paste real verbatim output; if a run did not reach green, say so.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges and a one-line why.
2. The falsifier — the certificate's verbatim pre-repair failure, then its
   verbatim post-repair pass.
3. Verification — every command with verbatim output and test totals.
4. The A/B — control median first, then the adjusted affected rows.
5. What the fallback costs on the depth-65 case, measured.
6. Stop-loss / open questions.

## Worker report (brief-431)

### 1. What changed

- `crates/rum-runtime/src/runtime/resolution/match_normalization.rs:1-274`:
  added the precedent-based depth-16 recursive front path, a distinct
  `MatchNormalizerDepthExhausted` result, and a cold `#[inline(never)]`
  explicit-stack fallback. The recursive and iterative engines both use the
  single argument-attribute finisher at lines 184-207 and the single four-arm
  rebuild finisher at lines 262-273. The definition-identity wrapper retains
  its own hold gate and outer policy while routing each child through the
  complete bounded/fallback core. No work vector is constructed on the shallow
  path.
- `crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:77-111,238-253,472-599`:
  added changed and unchanged argument/head spines at depths 17 and 65; compared
  the ordinary entry, bounded entries at budgets 0/1/15/16/17, and the directly
  invoked iterative fallback against the frozen reference on `Option`, logical
  value, and whole-tree Arc-origin signature; and directly checked zero-budget
  atoms plus the 16/17 call-depth boundary.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:15-47`:
  added the height-parameterized helper, kept `evaluate_certificate`'s signature
  by delegating to it, and added separate ignored 2 MiB `MatchQ` and construction
  liveness certificates at physical height 20,001.
- `docs/dev/codex/brief-431-bounded-candidate-normalizer.md`: appended this
  report.

Only those four in-scope files are modified. No caller, constructor,
`Expr`/`SharedExpr`/`SharedArgs` implementation, configuration, or `unsafe` code
was touched.

### 2. Falsifier

The untouched release CLI was built and copied to `/tmp/rum-cli-base` before
the production edit. An initial command with `--exact` selected zero tests and
was not treated as a falsifier. The corrected pre-repair command was:

```text
$ scripts/run-bounded.py --timeout 120 -- cargo test -p rum-stdlib --release matchq_completes_at_physical_height_20001_on_two_mib -- --ignored
    Finished `release` profile [optimized] target(s) in 0.02s
     Running unittests src/lib.rs (target/release/deps/rum_stdlib-eeb79aceb990740c)

running 1 test

thread 'matchq-depth-20001' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --lib`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/target/release/deps/rum_stdlib-eeb79aceb990740c matchq_completes_at_physical_height_20001_on_two_mib --ignored` (signal: 6, SIGABRT: process abort signal)
```

The identical post-repair command passed:

```text
$ scripts/run-bounded.py --timeout 300 -- cargo test -p rum-stdlib --release matchq_completes_at_physical_height_20001_on_two_mib -- --ignored
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `release` profile [optimized] target(s) in 31.39s
     Running unittests src/lib.rs (target/release/deps/rum_stdlib-eeb79aceb990740c)

running 1 test
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 857 filtered out; finished in 0.02s

     Running unittests src/bin/generate-builtin-reference.rs (target/release/deps/generate_builtin_reference-e09809f9b99114dc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 3. Verification

The focused contract command passed before the A/B:

```text
$ scripts/run-bounded.py --timeout 300 -- cargo test --offline -p rum-runtime match_normalization_contract
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.30s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 8 tests
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_bounded_front_distinguishes_completion_from_exhaustion ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 948 filtered out; finished in 0.01s
```

`cargo fmt --check`, `cargo test --workspace`, and
`cargo test -p rum-stdlib --release iterative_traversal_depth -- --ignored`
were not run: the required A/B below triggered the stop-loss first. `cargo fmt
--all` and `git diff --check` had completed successfully during iteration, but
they are not reported as substitutes for the unrun mandatory commands.

### 4. Required A/B

The base and candidate release builds both completed in 20.42 seconds. Their
SHA-256 hashes differed (`d7c72076...f519` base,
`caeb794b...4719` candidate). The stock commands completed successfully:

```text
$ scripts/run-bounded.py --timeout 240 -- bash scripts/core-language-microbench.sh --rum-cli /tmp/rum-cli-base --output-dir /tmp/brief431-micro-base
core microbench output: /tmp/brief431-micro-base
$ scripts/run-bounded.py --timeout 60 -- bash scripts/core-language-microbench.sh --output-dir /tmp/brief431-micro-candidate
core microbench output: /tmp/brief431-micro-candidate
```

Both `runs.tsv` rows were green: base elapsed 0.849413 seconds and candidate
elapsed 0.799196 seconds. The script explicitly set all profiling variables to
zero. The named-control median candidate/base delta was **-1.775737%**. The
adjusted value below is `raw delta - control median`, as required.

| Affected row | Base seconds | Candidate seconds | Raw delta | Offset-adjusted |
|---|---:|---:|---:|---:|
| `matchq-integer-success` | 0.000316397 | 0.000319219 | +0.891917% | +2.667655% |
| `matchq-integer-fail` | 0.000321353 | 0.000319130 | -0.691763% | +1.083974% |
| `matcher-single-tiny` | 0.000677484 | 0.000708952 | +4.644833% | +6.420570% |
| `matcher-single-wide-small` | 0.000038579 | 0.000041116 | +6.576117% | +8.351854% |
| `matcher-single-wide-large` | 0.000075318 | 0.000081170 | +7.769723% | +9.545460% |
| `replaceall-simple` | 0.000364513 | 0.000360886 | -0.995026% | +0.780711% |
| `rewrite-guarded-hit` | 0.001806880 | 0.001816300 | +0.521341% | +2.297078% |
| `rewrite-guarded-decline-fallback` | 0.002540410 | 0.002537040 | -0.132656% | +1.643081% |
| `cases-wide-no-match` | 0.000157926 | 0.000173993 | +10.173752% | +11.949489% |
| `count-wide-no-match` | 0.000146572 | 0.000160791 | +9.701034% | +11.476771% |
| `position-wide-no-match` | 0.000178180 | 0.000194232 | +9.008867% | +10.784605% |
| `deletecases-wide-no-match` | 0.000174190 | 0.000187995 | +7.925254% | +9.700991% |
| `deletecases-wide-association-no-match` | 0.000416611 | 0.000412659 | -0.948607% | +0.827130% |
| `replace-wide-no-match` | 0.000104645 | 0.000088599 | -15.333747% | -13.558010% |

The three `matcher-single-*` rows are a coherent +6.42% to +9.55% adjusted
slowdown family. The four list-traversal rows `Cases`/`Count`/`Position`/
`DeleteCases` are a second coherent +9.70% to +11.95% adjusted slowdown family.
Either exceeds the brief's coherent-family 3% stop-loss, so no second A/B was
run and no tuning was attempted.

### 5. Depth-65 fallback cost

No depth-65 cost timing was run. The A/B stop-loss required an immediate stop,
so reporting a number here would require violating the acceptance ordering.
The depth-65 changed/unchanged argument/head cases did pass the frozen-reference
equivalence test, but that is correctness evidence rather than a cost
measurement.

### 6. Stop-loss / open questions

- **Stop-loss fired.** The implementation remains in the uncommitted working
  tree for review, but it is not an accepted repair. No constant tuning,
  constructor work, deferred `Cases` work removal, or other compensating edit
  was attempted.
- The broad workspace gate, full ignored deployment-depth module, and the
  separate height-20,001 construction liveness test remain unrun because the
  performance gate stopped the step.
- No existing contract assertion failed; all eight focused contract tests were
  green.
- The independent recursive `Orderless` comparator and logical-equality paths
  remain outside scope, as stated in the brief.

## Director's gate (2026-09-03)

**The stop-loss stands, but only half of it survived verification, and the
worker's A/B could not have told the difference.** I rebuilt both binaries and
confirmed the SHA-256 hashes match the ones reported above, then ran four
microbench passes in one window — base, candidate, base, candidate — to measure
the same-binary noise floor the single A/B never established.

Same-binary spread on the affected labels: **−7.78% to +11.19%.**

| Label | base1→base2 (SAME BINARY) | b1→c1 adj | b2→c2 adj |
|---|---:|---:|---:|
| `matchq-integer-success` | −1.55% | +4.14% | +2.56% |
| `matchq-integer-fail` | −1.81% | +3.36% | +2.51% |
| `matcher-single-tiny` | −6.86% | +1.08% | +5.68% |
| `matcher-single-wide-small` | **+11.19%** | +13.31% | +0.11% |
| `matcher-single-wide-large` | **+9.73%** | +12.44% | +0.21% |
| `replaceall-simple` | −4.49% | −0.45% | +8.59% |
| `rewrite-guarded-hit` | −7.73% | −3.27% | +9.19% |
| `rewrite-guarded-decline-fallback` | −7.78% | −3.77% | +10.29% |
| `cases-wide-no-match` | +2.68% | +15.57% | +6.45% |
| `count-wide-no-match` | +0.66% | +13.40% | +8.37% |
| `position-wide-no-match` | +3.42% | +12.49% | +6.59% |
| `deletecases-wide-no-match` | +2.13% | +11.30% | +6.22% |
| `deletecases-wide-association-no-match` | +10.73% | +5.44% | −0.34% |
| `replace-wide-no-match` | +1.98% | +3.17% | +1.32% |

Control median moved −3.42% between two runs of the identical binary, so the
whole-run offset is itself of the same size as several of the reported effects.

**What is real.** Four traversal labels — `cases`, `count`, `position`,
`deletecases-wide-no-match` — are positive in *both* replicates (+6.2% to
+15.6%) while their same-binary spread is +0.66% to +3.42%. `matchq-integer-*`
is smaller but likewise positive in both (+2.5% to +4.1%) against a same-binary
spread under 2%. That is a regression.

**What is not.** The `matcher-single-*` family, cited in the report as one of
the two coherent families, has a same-binary spread of +9.7% to +11.2% — larger
than the effect attributed to it, and its two replicates read +13.31% and
+0.11%. It carries no information. `replace-wide-no-match`'s reported −13.6%
"improvement" reads +3.17% and +1.32% here; it was noise in the other
direction. The worker followed the brief exactly; the brief's protocol was
wrong, and that is my error, not the worker's.

**The mechanism, from which labels moved.** The regression tracks the number of
*nodes visited per normalization*, not the number of call nodes: a wide list of
1,600 atoms moved most, a single-node `MatchQ` least. The depth budget is
consumed only on `Expr::Call`, and `cases-wide-no-match` has exactly one call
node per candidate — so the decrement cannot be what costs 6-15% there. What is
paid 1,601 times is the new return type: the recursive worker went from
returning `Option<Expr>` to `Result<Option<Expr>, MatchNormalizerDepthExhausted>`,
and every atom now returns `Ok(None)` through a `?`.

**Verdict: rejected as built, and the design is not refuted — its plumbing is.**
The implementation is correct: the pre-repair certificate failed, the post-repair
certificate passed, and all eight contract tests including the three-producer
equivalence stayed green. It is preserved in `stash@{0}`. brief-432 keeps the
engine and removes the wrapper: the recursive path's return type goes back to
`Option<Expr>` and exhaustion is handled locally at the node where it is
detected, rather than propagated to the root and restarted.
