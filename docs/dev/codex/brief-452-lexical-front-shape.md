# brief-452 — reshape the lexical substitution front

## Goal

brief-451's conversion is functionally complete and correct, and it stays. What
has to change is its **shape**: threading `remaining_call_depth` as a new
argument through every arm of the lexical recursion group costs **+1.115%
instructions on the ordinary `With[{x = 1}, x + 1]` path**. Move the depth so
that the shallow path pays nothing, without weakening the depth bound or
changing any behaviour.

**The working tree already contains brief-451's change. Revise it in place.**
Do not revert it, do not start over, and do not lower `TARGET_PHYSICAL_HEIGHT`.

## What I measured at the gate — treat as given, do not re-run

Base is `3890308a` (brief-451's brief commit, before its code), candidate is the
current working tree. Plain `--release` builds, `perf stat -e instructions:u`,
minimum of the tight clean cluster, migrated runs discarded.

| probe | base | candidate | movement |
|---|---:|---:|---:|
| `With[{wx = 1}, wx + 1]` x20 000 | 269 460 746 | 272 465 683 | **+1.115%** |
| `With @@ {{wv -> 7}, Nest[List, wv, 8]}` x2 000 | 110 274 429 | 111 099 699 | **+0.748%** |
| `Module[{mx = 1}, mx + 1]` x20 000 | 466 272 191 | 467 013 104 | +0.159% |
| `Replace[7, x_ :> Nest[List, rv, 8]]` x2 000 | 172 944 661 | 172 640 563 | −0.176% |

Within-cluster spread was 0.004%-0.006%, so +1.115% is roughly 180x the noise.
Allocation counts are identical, so this is pure instruction cost, and the
`With` probes never reach the iterative tail — the body is nine levels deep and
the front is sixteen. **The rule-binding engine is fine and is not part of this
brief.**

The `With` / `Module` asymmetry (+1.115% against +0.159%) is a clue worth
following: whatever the added argument costs, it costs most on the `With` arm.

## Why this is a revision and not an acceptance

brief-451's Measurement section said a movement of "a fraction of a percent" was
the result and told you not to open a stop-loss on one. **That wording was mine
and it was too permissive** — 1.115% is literally a fraction of a percent and is
still ten times what this line has accepted anywhere else (the five conversions
before it measured −1.65%, −0.94%, −0.28%, +0.073% and +0.235%). The numeric bar
below replaces that sentence.

This line also has direct precedent that a better shape exists:

- **brief-441 was rejected for exactly this shape** — "a depth parameter and an
  `Option` return on the most-consulted function in the system is not free".
- **brief-442 fixed it by changing data layout, not control flow**: the hot
  function's signature stayed, the cold branches were outlined, and the cost
  went to zero.
- **brief-449 avoided it entirely** by deriving the cutoff from a counter the
  function already carried.

## The shape to try first

Every arm currently takes `(expr, bindings, shadowed, remaining_call_depth)`,
i.e. one more argument than before. Bundle the three walk-invariant-ish values
into a single context passed by reference:

```rust
struct LexicalWalk<'a> {
    bindings: LexicalBindings<'a>,   // or whatever the current type is
    shadowed: &'a BTreeSet<Symbol>,
    remaining_call_depth: usize,
}
```

so the arms take `(expr, &LexicalWalk)` — **fewer arguments than before
brief-451**, with the depth riding along free.

**The shadow set is the catch and it is not optional.** Several arms build a
*different* shadowed set for one child than for its siblings — `rhs_shadowed`
(`lexical_substitution.rs:109-111`), `condition_shadowed` (`:139-141`),
`body_shadowed` (`:167-169`, `:196-198`, `:256-260`). A context struct must be
cheap to re-derive per child (a copy of two references and a `usize`), never
shared mutably across siblings. If that re-derivation costs as much as the
argument did, say so with numbers and try the next shape instead.

Other shapes are welcome if they meet the bar — a thread-local depth, an
outlined cold entry, anything that leaves the shallow path as it was. What is
**not** acceptable is removing the bound or raising the front's depth to hide
the cost.

## Acceptance bar (this replaces brief-451's wording)

Measured as at the gate — plain `--release` builds, `perf stat -e
instructions:u`, minimum of the tight clean cluster, migrated runs discarded,
base and candidate executable paths the **same length**:

- `With[{wx = 1}, wx + 1]` x20 000: within **±0.15%** of base.
- `Module[{mx = 1}, mx + 1]` x20 000 and `With @@ {{wv -> 7}, Nest[List, wv,
  8]}` x2 000: no worse than **+0.15%**.
- `Replace[7, x_ :> Nest[List, rv, 8]]` x2 000: no worse than its current
  −0.176%.
- Allocation counts unchanged against base on all four probes.

±0.15% is about 25x the measured cluster spread, so it is a real bar and not a
noise chase. **If two isolated attempts cannot reach it, stop and report the
numbers for each attempt** — a well-measured "no shape does better" is a valid
outcome and I will take it.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs`
- `crates/rum-runtime/src/runtime/substitution/lexical_substitution/iterative.rs`
- new files under
  `crates/rum-runtime/src/runtime/substitution/lexical_substitution/`

Out of scope: `binding_substitution.rs` and its new module (its numbers are
fine), every consumer, the certificate file, the microbench script, and every
test — **the existing tests and certificates must pass unchanged**. If a test
needs editing, that is a signal you changed behaviour; stop and report.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

`cargo test --workspace` may fail nine `rum-jupyter` native ZeroMQ tests if your
sandbox cannot bind loopback; say so and also run
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace`, pasting both.

Then the release certificates, which must still pass:

```
cargo test -p rum-stdlib --release -- --ignored
```

That panel takes about 465 s; size the timeout from that.

Then the four instruction probes and the four allocation probes above, base
against candidate, with the calibration run for each instrument (one extra loop
iteration must move it) and the discarded migrated runs named.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave everything in
  the working tree, including brief-451's change, for the gate.
- Do NOT modify files outside Scope; touch nothing outside this repository; no
  network.
- No new `unsafe`; no default-path cost; no profiling state in a work item or
  frame.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything long.
- Paste real verbatim output.

## Report format

1. What changed — the new shape, per file and line range, and how the
   per-subtree shadow set travels in it.
2. Each attempt you made, with its four instruction numbers, in order.
3. Verification output verbatim, including the release panel.
4. Whether the bar was met, per probe.
5. Stop-loss / open questions.
