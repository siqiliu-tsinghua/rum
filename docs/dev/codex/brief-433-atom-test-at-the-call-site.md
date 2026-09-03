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
