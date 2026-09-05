# brief-465 — semantic and suspension locks for `DeleteMissing`'s target mode

## Goal

Pin every observable behaviour of `DeleteMissing`'s finite-target mode with
tests, **against the untouched implementation**, so that the conversion that
follows has a net under it. This is step A of brief-464's plan.

**Test-only. Do not modify any production source.** If a test cannot be made to
pass without changing production code, that is a finding: report it and stop —
do not change the code and do not weaken the test.

## Why this step exists on its own

The conversion that follows cannot use the shape every previous conversion in
this line used. `Replace`, `DeleteCases`, `MergeNested` and `DeleteMissing`'s own
recursive mode all let a cold iterative tail **redo** the traversal once the
bounded front exhausts, which is sound only for a pure walker. Here a
target-level delayed evaluation runs user code, so a redo could run it twice.
The replacement shape has to carry a live continuation and completed prefixes
across the handoff — materially harder, and easy to get subtly wrong in ways
that only show up as a callback running twice or a prefix being revisited.

**That is what this step is protecting against**, and it is why the callback
count and resume tests below matter more than the value assertions.

## The contract to lock — eight items

Taken from brief-464 §4, each already cited there against
`crates/rum-stdlib/src/associations/missing.rs`:

1. Lists and associations are visited in source order; an immediate keyed child
   is completed depth-first before the next sibling.
2. At target level in a list, an item is dropped when it is a rule-like form
   with a syntactically `Missing` value, **or** when the structural scan finds
   `Missing` within `scan` — and the rule-like test **short-circuits** the scan.
3. At every visited association level, an entry with a syntactically `Missing`
   RHS is dropped **before** target-level delayed evaluation, so it runs no user
   code.
4. At target level in an association, each reached `RuleDelayed` RHS is
   evaluated **exactly once** and each immediate RHS **zero** times. The original
   entry, `Rule` versus `RuleDelayed` included, is retained only if the result is
   neither root `Missing` nor structurally missing within `scan`.
5. Above target level, an immediate list/association RHS is traversed now; a
   delayed RHS is not evaluated and receives **exactly one** delayed cleanup
   wrapper for `level - 1`.
6. Non-keyed leaves are retained unchanged; rebuilt lists and associations keep
   input order, association keys, and rule heads.
7. A suspension resumes **after** the pending evaluation: its returned `Expr`
   feeds the pending predicate or child finish, **completed prefixes are not
   revisited**, the pending delayed RHS is **not re-evaluated**, and the next
   callback is the next entry in depth-first source order.
8. Non-input errors propagate unchanged, and cancellation or abandonment remains
   a pure frame discard.

## Deliverable 1 — the value and order tests

A shallow mixed list/association tree, three levels, **multiple siblings at each
level**, containing immediate and delayed rules, a syntactic `Missing`, a nested
`Missing` reachable only by the scan, and a **distinct per-level survivor** so a
dropped or transposed branch is visible. Assert the **exact** resulting
expression — its order, its keys, and its rule heads. Not `Depth`, not `Length`,
not a key set.

Cover both `scan` settings that the measured brackets used (`0` and `All`) and
both parent shapes: a list parent around an association child, and an
association parent around a list child.

Existing coverage to build on rather than duplicate:
`crates/rum-stdlib/src/tests/associations_missing_surface.rs:78-223`.

## Deliverable 2 — the callback-count and suspension tests

This is the important half. The current request suite has **one** eager delayed
predicate suspension and **one** nested association suspension
(`crates/rum-stdlib/src/tests/session_requests/transforms_resume_associations.rs:508-558`),
which is not enough to catch a replay.

Add tests that:

- **Count callbacks exactly.** Each delayed RHS appends a distinct token to a
  side-effect log. Assert the log's exact contents and length, so a second
  evaluation of any RHS fails — not a count of distinct tokens, the full ordered
  log.
- **Suspend at the first, a middle, and the last sibling**, in separate tests,
  with completed siblings before and after the suspension point.
- **Suspend more than once in one traversal**, in different branches, and assert
  the prompt order and the exact final value.
- **Assert no prefix replay across resume**: a delayed RHS that already ran
  before the suspension must not appear a second time in the log after it.
- Cover both parent shapes, so the ordinary parent suspension sites and the
  association parent sites are both reached.

Follow the request-test idiom already in the file
(`crates/rum-stdlib/src/tests/session_requests/transforms_resume_associations.rs:11-47`),
and compare queued-input against explicit-resume outcomes where that idiom does.

## Deliverable 3 — the falsifiers, and a correction to how they are done

brief-464's draft proposed perturbing the **expected values** in the tests — make
the expected log contain a replay, transpose two expected siblings — and
checking they fail. **That is not a falsifier.** A test whose expectation you
corrupt always fails; it demonstrates that the assertion is wired up, and says
nothing about whether the test can detect a change in the *implementation*.

Do it the other way. For each contract item, make a **minimal perturbation to
the production source**, capture the test red verbatim, then restore. Suggested
perturbations, one per item — choose a smaller edit if you find one:

| item | perturbation |
|---|---|
| 1 | process one level's siblings in reverse order |
| 2 | remove the rule-like short circuit so the scan also runs |
| 3 | move the syntactic-`Missing` drop to after the delayed evaluation |
| 4 | evaluate a reached `RuleDelayed` RHS twice |
| 5 | build the delayed wrapper for `level` instead of `level - 1` |
| 6 | rebuild a `RuleDelayed` entry with a `Rule` head |
| 7 | on resume, restart the current level from index 0 |

**Every perturbation must be reverted before you finish; the tree you hand back
must contain test files only.** Report each perturbation precisely enough that
the director can reproduce one or two by hand.

For item 8, say whether a minimal perturbation exists. If none does — because
the property is structural rather than behavioural — **say so and explain why**;
that is a legitimate answer and more useful than a contrived edit. A contract
item that nothing can violate is either already enforced by construction or is
not really a contract, and either way I want to know which.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/tests/associations_missing_surface.rs`;
  - `crates/rum-stdlib/src/tests/session_requests/transforms_resume_associations.rs`;
  - a new focused test file plus its wiring in `crates/rum-stdlib/src/tests.rs`
    or the `session_requests` module root.
- Files you MUST NOT touch, except transiently for a falsifier that you then
  revert: **all production source**, and in particular
  `crates/rum-stdlib/src/associations/missing.rs` and everything under
  `crates/rum-stdlib/src/associations/missing/`. Also off limits entirely: the
  registry and recursion inventory, `metadata.rs`, `Expr`, `SECURITY.md`,
  `TODO.md`, `PROGRESS.md`, other `docs/**`, `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** this brief adds tests. `git diff --stat` at the end must show
  test files and wiring only. Any production change that survives to the end is
  a **stop**.

## rum / Rust pitfalls (read twice)

1. **Characterize, do not bless.** If a behaviour looks wrong, write the test
   that documents what happens, mark it clearly in your report as a suspected
   defect, and stop rather than deciding it. Weakening a documented contract to
   make a test pass is prohibited (`AGENTS.md`).
2. **Do not run any probe that drives a recursion to or past its ceiling**, on
   either the depth or the width axis. Every fixture here is shallow and narrow.
   The measured brackets are: depth 4,944 physical, width 2,921 siblings. Stay
   far below both.
3. `DeleteMissing[expr, level]` is the **recursive** mode. Target mode is the
   **three-argument** form. A two-argument probe tests the wrong walker.
4. The memory preflight is unreachable unless a limit is armed
   (`crates/rum-stdlib/src/associations/missing.rs:89-91`).
5. Assert exact expressions, not summaries. Twelve certificates in this line were
   green while exercising nothing because they asserted `Depth` and `Length`.
6. Wall-clock timing is not evidence on this host, and nothing here needs it.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; if so, say
plainly that the nine native tests did not run in your gate.

Then the falsifier table: for each contract item, the perturbation, the verbatim
red, and the verbatim green after restoring.

Then `git diff --stat`, to show test files only.

No performance gate: no production code changes. Say that explicitly rather than
running the panel.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT leave any production edit in the tree.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behaviour only, no oracle harnesses, no comparison against another engine.
- **Stop-loss.** Stop and report if: a test cannot pass without a production
  change; a perturbation does not turn its test red (that means the behaviour is
  unprotected — say which); or the same step fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The eight contract items, and which test pins each.
3. The falsifier table: perturbation, red, green — plus your answer for item 8.
4. Any behaviour you found that looks like a defect, characterized but not
   blessed.
5. Verification: every command with verbatim output, the ZeroMQ statement, and
   `git diff --stat`.
6. Stop-loss and open questions.
