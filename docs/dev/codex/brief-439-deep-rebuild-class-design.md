# brief-439 — the deep-rebuild class, and the certificate panel that misses it

**Shape: design-only. `Scope: NONE — no source change.`**

## Goal

Briefs 422-433 converted ten traversals to iterative walks and shipped twelve
release-only 2 MiB certificates. Every one of those certificates either does
not descend at all, or descends and returns a *shallow* result. **None
exercises a traversal that rebuilds a deep result** — and that is exactly the
case that still ends the process.

`ReplaceAll` is the clearest example. Its certificate replaces a symbol that
does not occur in the certificate value, so no rebuild ever happens and the
test passes. Change the rule so that it matches, and the same expression at the
same height kills the process.

I want a plan that (a) explains the two distinct failure modes below, (b)
inventories the full class rather than the sites I happened to probe, (c)
repairs the certificate panel so a rebuild is actually covered, and (d) drafts
the implementation briefs. No source change; see Hard rules.

## What I measured — treat as given, do not re-run

Release `rum-cli`, `ulimit -s 2048`, current `main` (`cb5521df`, evaluator byte
budget 1 MiB), value built by the certificate harness's own shape
`x = 1; Do[x = {x}, {i, 4096}]` unless stated.

**The two failure modes are distinguishable and the distinction matters.**
`exit 134` with `overflowed its stack` means the recursion runs entirely inside
a builtin: it advances no evaluation depth, so `poll_native_stack_limit` never
polls during the descent and the process dies. `exit 1` with
`General::reslim` means the recursion goes through the evaluator, which polls,
so it fails recoverably.

| case | result |
|---|---|
| `Length[Cases[x, z, Infinity]]` | ok |
| `Scan[Null &, x, {0, Infinity}]` | ok |
| `Count`, `Position`, `MemberQ`, `FreeQ`, `Level`, `MatchQ`, `ToString`, construction | ok |
| `Depth[ReplaceAll[x, z -> q]]` — **the certificate** | ok, and **no rebuild happens: `z` does not occur in `x`** |
| `Depth[ReplaceAll[x, 1 -> 2]]` | **PROCESS KILL** |
| `Depth[ReplaceAll[x, List -> List]]` | **PROCESS KILL** |
| `y = ReplaceAll[x, 1 -> 2]` | **PROCESS KILL** |
| `Depth[Map[Identity, x, {-4097, 0}]]` — **the certificate** | ok, and **it does not descend**, see below |
| `Depth[Map[Identity, x, {0, Infinity}]]` | in-band `reslim` |
| `Depth[Map[f, x, {0, Infinity}]]`, `{0, 4097}`, `{1, Infinity}`, `Infinity`, `{-4097, -1}` | in-band `reslim` |
| `Depth[Apply[f, x, {-4097, 0}]]` — **the certificate** | ok, does not descend |
| `Depth[Apply[List, x, {0, Infinity}]]` | **PROCESS KILL** |
| `Depth[Apply[g, x, {-4097, -1}]]` | **PROCESS KILL** |
| `Replace[x, 1 -> 2, {0, Infinity}]` | **PROCESS KILL**; last ok 1,308, first fail ~1,312 |
| `MapIndexed[#1 &, x, {0, Infinity}]` | in-band `reslim`; last ok 109, first fail 112 |
| `D[Nest[Sin, x, N], x]` | **PROCESS KILL**; ok at 1,600, killed at 2,000 |
| `MapAt[f, x, {1}]`, `ReplacePart[x, {1} -> 9]` | ok (shallow rebuild only) |

**Why the `Map`/`Apply` certificates do not descend.** They use the level spec
`{-4097, 0}`, and in rum that matches level 0 only:

```
y = {{{1}}};
Level[y, {-1}]          -> {1}
Level[y, {-4, 0}]       -> {{{{1}}}}          one element, the whole expression
Level[y, {0, Infinity}] -> {1, {1}, {{1}}, {{{1}}}}
Map[f, y, {-4, 0}]      -> f[{{{1}}}]          applied once, at the top
Map[f, y, {0, Infinity}]-> f[{f[{f[{f[1]}]}]}] applied at every level
```

So `Depth[Map[Identity, x, {-4097, 0}]]` wraps the value once and asks `Depth`,
which is already iterative. It certifies nothing about `Map`'s traversal.

**Both probes above are calibrated.** `MapIndexed[g[#1] &, {{{1}}}, {0,
Infinity}]` returns `g[{g[{g[{g[1]}]}]}]`, so it reaches level 0 and the
innermost atom; `D[Sin[Sin[Sin[x]]], x]` returns the three-factor chain-rule
product, so it really differentiates. Two earlier probes of mine were invalid
and are recorded so you do not repeat them: a literal nested source pins at 125
levels at *every* stack size because it hits the parser's 128 nesting guard,
and `Nest`-built values above 2 MiB pin at 252 because of the fixed-point pass
limit (256), neither of which is a stack ceiling.

**Where the machinery is and is not.** `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-60`
and `crates/rum-runtime/src/runtime/tree_walkers/level.rs:76` carry the
depth-16 recursive front plus explicit-stack fallback. A search for
`_recursive_bounded`, `_iterative`, or a work stack across
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/` returns
nothing, so the replace family has no such fallback at all.
`map_indexed_parts_recursive` (`.../apply_map/traversal.rs:309`) and
`collect_scan_parts_recursive` (`.../traversal.rs:415`) sit in the converted
file but were left unbounded; `Scan` nonetheless clears 4,097 and `MapIndexed`
does not, which is itself worth explaining.

## Context pointers

- `AGENTS.md`, then `docs/dev/codex/brief-425-iterative-traversals.md` (the
  conversion this brief re-examines) and
  `docs/dev/codex/brief-437-native-stack-budget-design.md` with its gate note
  (the guard, the frame cost, and the intra-builtin poll gap).
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:1-140` — the twelve
  certificates and the harness.
- `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs` and
  `.../traversal/recursive.rs` — the converted pattern, plus the two functions
  left recursive.
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/` —
  `all.rs`, `once.rs`, `list.rs`, `repeated.rs`.
- `crates/rum-stdlib/src/calculus.rs` — the `D` walker.
- `crates/rum-runtime/src/runtime/eval_core/mod.rs:34-79` — the guard and its
  stride, which explains the two failure modes.

## Deliverables

1. **The mechanism for each failure, with `path:line`, and which of the two
   modes it is in and why.** For a `PROCESS KILL` case, name the recursive
   function and show that nothing on that path increments evaluation depth. For
   an in-band case, name where the evaluator re-enters. In particular explain
   why `ReplaceAll` survives a walk that finds no match but dies when it
   rebuilds, and why `Scan` clears 4,097 while `MapIndexed` fails at 110.

2. **The full class inventory, as a falsifiable prediction.** Every builtin
   that can rebuild a deep result, whether it is bounded today, and which
   failure mode it would show. I want a prediction I can test by probe, not a
   list of the examples I already gave you. Include the ones I measured as
   passing (`MapAt`, `ReplacePart`, `Scan`) and say why they pass — a
   prediction that only covers failures is not testable.

3. **Why the depth-16 fallback does not save `Map` at `{0, Infinity}`.**
   `apply_heads_with_config` and `map_parts_with_config` fall back to
   `transform_parts_iterative` when the bounded front returns `None`. Either
   that fallback is unreachable for this level spec, or the recursion is
   somewhere else entirely. Settle which, from source.

4. **The certificate panel repair.** What a certificate for a rebuilding
   traversal must assert so it cannot silently stop covering the rebuild —
   a value that forces every level to be rebuilt, an assertion on the rebuilt
   result rather than on a summary, and a level spec that provably descends.
   Include a general rule that would have caught all three defective
   certificates, and say which of the twelve need replacing versus keeping.
   State plainly whether `{-4097, 0}` matching level 0 only is rum's intended
   documented semantics or a separate defect; do not repair it here either way.

5. **The repair plan**, ordered, with each option's cost and what enforces it.
   Distinguish repairs that stop a process kill (safety) from repairs that
   raise a recoverable ceiling (contract). Say explicitly which sites can
   reach physical height 4,097 on 2 MiB after each repair and which cannot —
   brief-437 established that the evaluator's own recursion cannot, so a
   traversal whose per-level cost is an evaluator re-entry has a ceiling no
   traversal repair can lift.

6. **Drafted implementation briefs** in `docs/dev/codex/TEMPLATE.md` shape, one
   per step, each with Scope, gate, stop-loss, and — where a hot path is
   touched — an A/B panel. Drafts; I rewrite before dispatch.

## Scope

- Files you MAY modify: **NONE.** Append your report to this brief only.
- Write no Rust, add no test, run no benchmark.

## Hard rules

- **Do not reproduce a stack overflow.** Do not write, generate, or run any
  program whose purpose is to exhaust the stack, and do not run the deep cases
  in the table above. Every number you need is in this brief. Reading source,
  reading the compiled binary, and running *small* expressions that cannot
  approach the limit are the tools here.
- Do NOT `git commit`, branch, push, or touch git config.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy). Level-spec semantics are settled against rum's
  own documentation and behaviour, never against another implementation.
- No new `unsafe` in `rum-core` or `rum-runtime`.
- If a deliverable cannot be established from source, say so and name the
  non-crashing measurement that would settle it. Do not guess and do not pad.

## Report format

Append under `## Appended design report`, in deliverable order, every claim
carrying `path:line`. Mark projections as projections.
