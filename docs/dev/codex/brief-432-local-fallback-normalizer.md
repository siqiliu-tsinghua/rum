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
