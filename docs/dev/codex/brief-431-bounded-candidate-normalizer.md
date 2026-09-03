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
