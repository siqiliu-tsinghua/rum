# brief-453 — put the lexical depth back in the walk context

## Goal

A short, mechanical revert. brief-452 produced two shapes; the first is better
on both counts. Put the depth back into the `LexicalWalk` context struct and
delete the thread-local counter. Change nothing else.

This brief is deliberately small and its verification is deliberately short —
two dispatches of brief-452 were cut off by an upstream fault during their long
measurement phases, so the workspace run and the release panel stay on my side
this time.

## Why — both attempts are measured

Base `3890308a`, plain `--release`, `perf stat -e instructions:u`, minimum of
the tight clean cluster, migrated runs discarded, director-measured:

| probe | attempt 1 (`LexicalWalk` depth field) | attempt 2 (thread-local) |
|---|---:|---:|
| `With[{wx = 1}, wx + 1]` x20 000 | **+0.713%** | +1.085% |
| `Module[{mx = 1}, mx + 1]` x20 000 | not measured | +0.351% |
| `Replace[7, x_ :> Nest[List, rv, 8]]` x2 000 | −0.176% (brief-451 shape) | +0.068% |

Attempt 2 is slower. It also has a fragility attempt 1 does not:
`enter_recursive_lexical_call` / `leave_recursive_lexical_call`
(`lexical_substitution.rs:86,98`) are **five manual pairs with no `Drop`
guard** (`:459/:489`, `:504/:510,:542`, `:564/:590`, `:594/:603`). Every path
balances today, but the invariant is "every path from an enter reaches a leave",
and breaking it is silent: the counter drifts up and the recursion budget
shrinks for the rest of that thread. A panic between an enter and its leave
leaks it across requests on a reused thread.

## What to do

1. Restore `remaining_call_depth: usize` as a field of `LexicalWalk`
   (`lexical_substitution.rs:59-83`), seeded by
   `RECURSIVE_LEXICAL_SUBSTITUTION_CALL_DEPTH` in `new`, with a `descend()` that
   returns a copy with the value decremented and `with_shadowed` carrying it
   through unchanged.
2. At each of the five sites that now call `enter_recursive_lexical_call`,
   branch on `walk.remaining_call_depth == 0` to the iterative path exactly as
   the thread-local version branches on `!enter_...`, and otherwise recurse with
   `walk.descend()`.
3. Delete `LEXICAL_SUBSTITUTION_CALL_DEPTH`, `enter_recursive_lexical_call` and
   `leave_recursive_lexical_call`.
4. Change nothing else — not `binding_substitution.rs`, not the iterative
   modules, not tests, not the microbench, not the certificates.

The `#[derive(Clone, Copy)]` on `LexicalWalk` must stay; the struct is two
references plus a `usize` and is passed by reference.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs`

Nothing else. If you believe another file must change, STOP and report instead.

## Verification (keep it short — this is all you run)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
```

Then exactly two instruction probes, base against candidate, plain `--release`
builds with **equal-length executable paths**, `perf stat -e instructions:u -x,`,
minimum of the tight clean cluster, migrated runs (both hybrid PMU counters
non-zero) discarded and named:

- `Do[With[{wx = 1}, wx + 1], {i, 20000}]` — expect about +0.71% against base,
  and it must not be worse than attempt 2's +1.085%.
- `Do[Replace[7, RuleDelayed @@ {Pattern[rv, Blank[]], Nest[List, rv, 8]}], {i, 2000}]`
  — expect about −0.18%.

Each probe file must end with a liveness assertion (`If[SameQ[...], Null,
Exit[2]]`) so a probe that stops doing the work fails loudly.

Do **not** run the workspace suite or the release panel; I run those.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the tree.
- Do NOT modify files outside Scope. Nothing outside this repository; no network.
- No new `unsafe`; no default-path cost; no profiling state in a frame.
- Paste real verbatim output.

## Report format

1. The diff shape — what moved back into `LexicalWalk`, and the five branch
   sites.
2. `cargo fmt --check`, `cargo test -p rum-runtime`, `cargo test -p rum-stdlib`
   output verbatim.
3. The two instruction numbers with their clusters and discarded runs.
4. Anything you could not do.
