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
