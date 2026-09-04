# brief-448 — iterative containment and differentiation in `D`

## Goal

`D` is the second of the four remaining fail-open sites in the deep-rebuild
class: over a deep spine it ends the process instead of raising an error.
Remove both depth-proportional native recursions from `crates/rum-stdlib/src/calculus.rs`
— the containment prepass and the differentiation walk — so that a 4,097-level
nested-`Sin` derivative cannot terminate the process, and every documented `D`
result is byte-identical to what it is today.

This is step 4 of brief-439's plan, re-derived against measurements taken after
steps 1 and 2 landed. Its hard acceptance is **process safety**; what the
4,097-level case does *after* the recursion is bounded is a measurement you
report, not an assumption this brief makes. See "The two outcomes".

## What I measured — treat as given, do not re-run

Release `rum-cli` at `59410b75`, `ulimit -s 2048`, 1 MiB evaluator budget.

| case | result |
|---|---|
| `D` over a nested-`Sin` spine | **process kill**, last ok 1,738, first fail 1,746 |
| `D[Nest[Sin, q, 3], q]` (ground truth) | `Derivative[1][Sin][q] * Derivative[1][Sin][Sin[q]] * Derivative[1][Sin][Sin[Sin[q]]]`, `Length` 3 |

`calculus.rs` contains **zero** occurrences of `poll_native_stack_limit` and
`evaluation_depth`, which is why this is a process kill rather than an in-band
error.

**The probe shape matters, and two obvious ones measure nothing.**

- `D[Nest[Sin, q, N], q]` at a large `N` fails on the **fixed-point pass limit
  (256)** before it ever reaches the stack. Build the spine iteratively instead:
  `e = q; Do[e = Sin[e], {i, N}]`, with `Depth[e] == N + 1` and
  `Length[D[e, q]] == N` as the liveness assertions.
- **`D` does not descend into lists.** `D[{{{q}}}, q]` returns a chain-rule
  expression over `List`, so a list-spine probe measures the construction, not
  `D`. This matters for the certificate — see below.

Do not re-measure any of this, and do not write a probe that drives the
recursion past its ceiling.

## The recursion edges to remove

Both in `crates/rum-stdlib/src/calculus.rs`:

1. **The differentiation walk.** `differentiate_expr` (`:88`) dispatches to
   `differentiate_plus` (`:114`), `differentiate_times` (`:131`),
   `differentiate_power` (`:159`) and `differentiate_abstract_call` (`:198`),
   and all four call back into `differentiate_expr` — at `:122`, `:139`, `:174`
   and `:207` respectively. One mutual recursion group, one native level per
   spine level.
2. **The containment prepass.** `expr_contains_exact_subexpr` (`:253`) recurses
   over head and args (`:260-263`) and is called from `differentiate_expr:93`
   for **every** visited node, so it is both an independent depth-proportional
   descent and the first one to run.

`differentiate_expr` takes `&Session`, not `&mut Session`. **There is no
suspension path in `D`**: no resume frames, no `FrameRootTrace` work, nothing to
add to the frame inventory. If you find yourself needing any of that, stop —
you have gone outside the change.

## Semantics that must not move

Read these off the code before designing the task machine; an iterative
postorder driver changes *when* things happen, and these are the things that
must happen in the same order anyway.

- `expr == variable` returns exact `1` before anything else (`:89-91`).
- The containment prepass returns exact `0` for a subtree that does not contain
  the variable, **before** the match arm and before any combiner runs
  (`:92-95`).
- `differentiate_plus` drops exact-zero derivatives, then calls
  `combine_additive_args` exactly once (`:117-128`).
- `differentiate_times` builds, for each non-zero factor derivative, a full
  factor list with that derivative substituted at its own index, calls
  `combine_multiplicative_args` once per term, then `combine_additive_args`
  once (`:131-157`). Both the per-term call order and the argument order inside
  a term are observable.
- `differentiate_power` with an exact rational exponent: zero exponent → `0`;
  zero base derivative → `0`; unit exponent → the base derivative; otherwise
  `Times[exponent, base^(exponent-1), base']` **in that argument order**
  (`:159-193`). A non-rational exponent falls through to
  `differentiate_abstract_call` with head `Power` and the two args (`:194-196`).
- **`differentiate_abstract_call` returns early.** The moment
  `partial_derivative_call` yields `None` it returns an unevaluated
  `D[original, variable]` and **discards the terms accumulated so far**, after
  those terms' combiner calls have already happened (`:198-225`). A driver that
  defers combination to a single flush at the end does not reproduce this. This
  is the trap in the change.
- Every combiner returns `Result` and `?` aborts the whole differentiation.

## The two outcomes — report them separately

- **Safety (the hard acceptance).** At 4,097 the process must not terminate.
  This is what the change is for and it is not negotiable.
- **Contract (a measurement, not an assumption).** Whether the 4,097 case then
  *completes* or stops with an in-band error is for you to measure and report.
  brief-439's draft assumed an in-band resource error, but that projection
  predates two repairs: after brief-447 the comparable converted site reaches
  5,843 levels and fails closed, and its binding limit is now the shared
  stability walk rather than its own traversal. So do not write the certificate
  from either expectation — run it, see what happens, and assert **exactly what
  you measured**, naming the limiter with evidence when it is an error.

Never weaken the deep test to "any failure": a thread panic and an
`EvalError`/`TerminatedEvaluation` are the two outcomes this whole line exists
to distinguish.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/calculus.rs`
- new modules under `crates/rum-stdlib/src/calculus/`, declared from
  `calculus.rs`, if the file-size rule requires a split (it is 335 lines today,
  so it probably does not). **Do not touch `crates/rum-stdlib/src/lib.rs`** — if
  a split needs a `lib.rs` edit, you have chosen the wrong split; stop and
  report.
- `crates/rum-stdlib/src/tests/calculus.rs`
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh` — only for the one new label below

Out of scope: arithmetic canonicalization and the session combiners, evaluator
limits, the catalog contract, and every other stdlib family.

## Explicitly out of scope

The containment prepass runs once per visited node and scans that node's whole
subtree, so `D` is quadratic on a deep spine. **Do not memoize it, cache it, or
hoist it.** Keep the same call schedule and make each individual call
non-recursive. Changing what is computed and changing how deep it nests are two
changes, and a combined diff cannot be gated. If the quadratic cost makes the
deep certificate slow, raise its timeout with a bound derived from a measured
shorter run — do not "fix" it here.

## Deliverable

1. Iterative exact-subexpression containment.
2. A postorder differentiation task machine preserving branch order, zero
   elision, the partial-derivative early exit, and additive/multiplicative
   combination order exactly as listed above. Task results must not deep-clone
   shared expressions; `Expr` is `Arc`-shared.
3. Shallow exact-result regressions in `tests/calculus.rs` covering each arm:
   `Plus`, `Times`, rational-exponent `Power`, non-rational-exponent `Power`,
   the abstract-call chain rule, the early-exit fallback, and a
   variable-free subtree that must elide to `0`.
4. A release-only `#[ignore]` deep certificate on a 2 MiB declared thread,
   built with **its own nested-`Sin` spine** — the panel's existing
   `evaluate_certificate*` helpers build a *list* spine
   (`iterative_traversal_depth.rs:24-33`), which `D` does not descend, so
   reusing them would certify nothing. Assert the measured outcome exactly, and
   assert the session still evaluates `1 + 1` to `2` afterwards.
5. One new microbench label `d-nested-sin-small` — a fixture built with
   `Nest[Sin, <var>, 8]` and the liveness assertion
   `If[SameQ[Length[rumMicroSink], 8], Null, Exit[2]]`, following the existing
   sink-assertion convention (`scripts/core-language-microbench.sh:546,552,558`).

## The falsifier — required section

Before changing any source, add the shallow chain-rule regression, run it
against unmodified HEAD, and paste the pass. Then break it deliberately once —
assert a two-factor product instead of three — and paste the failure. Restore
it before continuing.

Do **not** run the deep certificate against unmodified HEAD: at 4,097 the
pre-change code ends the process rather than failing a test. That is why the
falsifier is the shallow test.

## rum / Rust pitfalls (read twice)

1. The early exit in `differentiate_abstract_call` is control flow, not an
   optimization. Reproduce it, including the combiner calls that already
   happened before it fires.
2. `expr == needle` at `:254` compares an `Expr` against a shallow variable and
   short-circuits on shape; do not replace it with anything that walks.
3. No new `unsafe`; `rum-core` and `rum-runtime` are zero-`unsafe` and this
   crate should stay that way too.
4. No default-path cost: a shallow `D[x^2, x]` must not pay for the deep
   machinery — allocate the work stack only when the shallow front is exceeded,
   the way `Level`, `Map` and now `Replace` do.
5. Probe budget: derive every timeout from a measured shorter run of the same
   shape; state the cap you used.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

`cargo test --workspace` may fail nine `rum-jupyter` native ZeroMQ tests if your
sandbox cannot bind loopback. That is an environment limit, not a result: say so
explicitly and also run
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace`, pasting both.

Then, after the shallow suite is green:

```
cargo test -p rum-stdlib --release -- --ignored
```

Then the A/B, base = unmodified HEAD build, candidate = yours, profiling env
OFF:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <new-dir>
```

Paste the `d-nested-sin-small` rows and **three untouched same-tier neighbors
you pick by measured magnitude** — name them and say why they are same-tier.
Best-of-2 per arm, and report each label's same-binary spread beside its
cross-binary movement; a movement inside its own spread is not a result.

## Stop-loss

Stop and report, with what you have, if:

- an algebraic-order mismatch recurs a second time;
- the driver would need a change to the session combiners or the evaluator;
- two isolated revisions do not remove an A/B regression;
- a split would need `lib.rs`;
- the deep certificate can only be made to pass by lowering the height.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the tree for the
  gate.
- Do NOT modify files outside Scope. Touch nothing outside this repository; no
  network.
- Testing Policy is a hard line (`AGENTS.md`): rum's own documented behavior
  only; no oracle harnesses, no third-party engine outputs.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
  can run long or spawn children.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file and line range, naming both removed recursive edges
   and how each was removed.
2. Verification — every command with verbatim output, including the falsifier
   pass, its deliberate failure, and both workspace runs.
3. Safety and contract as two separate outcomes, with the limiter named if the
   deep case stops early.
4. A/B rows: target, the three neighbors you chose, and the same-binary spread
   for each.
5. Stop-loss / open questions.
