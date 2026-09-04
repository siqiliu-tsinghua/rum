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

## Appended design report

### 1. Failure mechanisms

#### The boundary and the two modes

For this report, a **deep rebuilder** is a builtin or shared builtin engine
that follows a path whose length is proportional to an already-built `Expr`
(or to a selector/rank path) and can allocate a fresh `Expr::Call` at every
ancestor before returning. `Expr::Call` stores its head and argument vector in
separate `Arc`-backed objects, and `Expr::call` allocates both; that makes
"reused subtree" versus "fresh ancestry" a mechanically observable
distinction (`crates/rum-core/src/expr.rs:573-608`,
`crates/rum-core/src/expr.rs:637-708`,
`crates/rum-core/src/expr.rs:749-829`). Pure generators that make a tall value
without following existing ancestry, read-only walkers, encoders that return a
string, and one-level shapers are outside this inventory; they are listed as
negative controls in section 2 so that the boundary is falsifiable
(`TODO.md:1070-1077`).

The native-stack guard is entered only through
`evaluate_once_unstable_after_input_check_with_cycle_tracking`: it compares and
occasionally polls at the current `evaluation_depth`, then increments that
depth immediately before `evaluate_inner_unstable` and restores it afterward
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:14-64`). Therefore:

- **Unpolled native recursion / process-kill mode** means a helper recursively
  calls itself while one evaluator entry remains resident. No per-node call
  crosses `Session::evaluate`, so neither `evaluation_depth` nor the strided
  native-stack poll advances (`crates/rum-runtime/src/runtime/eval_core/mod.rs:34-53`).
- **Evaluator-reentry / in-band mode** means each structural level eventually
  reaches a call-shaped argument and `evaluate_atom_fast_path` calls
  `Session::evaluate` again. The guard then advances and returns a recoverable
  stack/resource error before the OS stack is exhausted
  (`crates/rum-runtime/src/runtime/eval_core/mod.rs:136-155`,
  `crates/rum-runtime/src/runtime/eval_call_prepare.rs:190-220`). The measured
  resident evaluator packet is 4,736 bytes, so this route cannot provide the
  ruled physical height 4,097 on a 2 MiB stack
  (`docs/dev/codex/brief-437-native-stack-budget-design.md:244-272`,
  `docs/dev/codex/brief-437-native-stack-budget-design.md:419-436`).

#### Case-by-case diagnosis

| Case | Mechanism and classification |
|---|---|
| `ReplaceAll`, no match | The target is evaluated first, so its existing call identities acquire stability-cache entries. The explicit rewrite work stack propagates `changed = false` and reconstructs with `Expr::call_shared(original_head, original_args)`, preserving those identities; the explicit final `self.evaluate(replaced)` can therefore hit the root stability entry without descending. This is why the no-match certificate passes, and it is not evidence for a rebuild (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:16-27`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:59-69`, `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:176-255`, `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:267-314`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:41-44`). |
| `ReplaceAll`, leaf or head match | A successful child sets `changed = true`; every finishing ancestor calls `Expr::call`, hence gets new head/args pointer keys. The final `self.evaluate` enters the fixed-point stability check, each new cache key misses, and `call_stability_uncached_with_bloom` recursively checks the head and arguments without changing evaluator depth. **Mode: unpolled native recursion, process kill.** `List -> List` still reports a match and therefore rebuilds even though the printed value is equal (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:199-245`, `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:363-415`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:59-69`, `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-329`, `crates/rum-runtime/src/runtime/eval_core/stability.rs:5-34`, `crates/rum-runtime/src/runtime/model/cache.rs:480-510`). |
| `Apply[List, ..., {0, Infinity}]` and stable `Apply[g, ...]` | The transform itself has a depth-16 front and an explicit-stack fallback. At selected nodes it creates a new call with the replacement head, and `changed` propagates upward, so a stable result presents fresh cache identities at every level. The subsequent outer fixed-point stability check is the same unpolled recursion as the matched `ReplaceAll` case. **Mode: unpolled native recursion, process kill**, not failure of `transform_parts_iterative` (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61`, `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:94-203`, `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:134-147`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:48-50`). |
| `Map[Identity, ..., {0, Infinity}]` | The iterative transformer succeeds and inserts an `Identity[...]` call at every selected level. A registered builtin head is classified as having an evaluation effect, so the fixed-point driver cannot declare the result stable; ordinary argument preparation then follows the nested call-shaped argument and re-enters `Session::evaluate` per physical level. **Mode: evaluator reentry, recoverable `General::reslim`.** The traversal fallback is not the recursion (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:94-203`, `crates/rum-runtime/src/runtime/eval_core/stability.rs:213-247`, `crates/rum-runtime/src/runtime/eval_core/stability.rs:269-273`, `crates/rum-runtime/src/runtime/resolution.rs:305-343`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:45-47`). |
| `Map[f, ...]` in the supplied run | **Established classification:** the supplied outcome is evaluator reentry because it is an in-band `reslim`, and the only post-transform proportional route is fixed-point/argument evaluation (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:46-47`, `crates/rum-runtime/src/runtime/resolution.rs:318-343`, `crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`). **Not established from source:** whether that session's `f` had a downvalue, upvalue, attributes, or other effect; an undefined unqualified head with no definitions is classified stable by the source (`crates/rum-runtime/src/runtime/eval_core/stability.rs:249-293`, `crates/rum-runtime/src/runtime/eval_core/stability.rs:321-345`). A non-crashing inspection of `OwnValues[f]`, `DownValues[f]`, `UpValues[f]`, and `Attributes[f]` in the exact measured session would settle that state dependency; this report does not guess it. |
| `Replace[..., {0, Infinity}]` | Before attempting a rule, `replace_owned` computes subtree depth using `replace_expr_depth` or `replace_expr_depth_with_heads`; both recurse once per physical level. If no replacement occurs at a node, child processing calls `replace_owned` recursively, and the association path does the same. None of those calls enters `Session::evaluate`; evaluation happens only after the traversal returns. **Mode: unpolled native recursion, process kill.** Source fixes the ordering (depth prepass first) but not which frame family consumes the final byte at the measured boundary; a shallow, at-most-16-level stack-marker slope per helper, or release-prologue accounting, would distinguish them without approaching overflow (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:192-222`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-319`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:439-513`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:518-595`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:794-840`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:51-51`). |
| `MapIndexed[#1 &, ..., {0, Infinity}]` | `map_indexed_parts_recursive` is unbounded and reconstructs every standard call even when that call is not selected. It nevertheless finishes at the observed height near 110; the returned tree contains nested anonymous-`Function` heads, which the stability classifier marks active. Function application substitutes the body and explicitly calls `self.evaluate(applied)`, producing the measured per-level evaluator reentry. **Observed mode at 110: recoverable `reslim`. Projection for a stable inert operator at sufficient height: unpolled process kill in `map_indexed_parts_recursive`.** The older inert-path bracket below 4,097 supports that projection (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-408`, `crates/rum-runtime/src/runtime/eval_core/stability.rs:235-246`, `crates/rum-runtime/src/runtime/scoping/callable.rs:5-29`, `crates/rum-runtime/src/runtime/scoping/callable.rs:59-75`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:52-52`, `TODO.md:1085-1105`). |
| `D[Nest[Sin, ...], x]` | `differentiate_expr` first calls the recursive `expr_contains_exact_subexpr`, then recursively differentiates `Plus`, `Times`, `Power`, and generic-call branches while retaining derivative construction state. Neither descent re-enters the evaluator; the surrounding `eval_d` evaluator call occurs outside them. **Mode: unpolled native recursion, process kill.** Source proves that the containment pass runs first but cannot prove whether it alone exhausts the measured stack before derivative recursion becomes resident; the same shallow stack-marker/prologue measurement just described would settle the first failing frame without a deep probe (`crates/rum-stdlib/src/calculus.rs:16-47`, `crates/rum-stdlib/src/calculus.rs:88-225`, `crates/rum-stdlib/src/calculus.rs:253-267`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:53-53`). |
| `MapAt[f, x, {1}]` | The exact-path engine selects one outer child, evaluates one callback, and replaces along a path of length one. The deep child is shared rather than rebuilt; at most the callback wrapper and root are new, so cached deep identities remain usable. **Observed pass; shallow rebuild only. Projection:** a 4,096-component path with a stable callback enters the unbounded exact lookup/rebuild helpers and is in native process-kill mode at a sufficiently large path (`crates/rum-stdlib/src/functional_ops/targeted_map.rs:183-215`, `crates/rum-stdlib/src/functional_ops/targeted_map.rs:295-394`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:173-201`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:54-54`). |
| `ReplacePart[x, {1} -> 9]` | The replacement discards the only deep child and allocates only a new root, so there is no deep rebuilt result for the post-evaluation stability check. **Observed pass; shallow rebuild only. Projection:** an exact deepest-leaf path, or a pattern-position plan that changes that leaf, enters recursive path collection/plan application and can rebuild every ancestor in unpolled native mode (`crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:225-307`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:431-518`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan.rs:269-299`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:54-54`). |
| `Scan[Null &, ..., {0, Infinity}]` | `collect_scan_parts_recursive` is genuinely unbounded, but it clones selected nodes into one flat vector and returns no rebuilt tree. `evaluate_scan_parts` then evaluates callbacks sequentially, so each callback's evaluator depth unwinds before the next starts. **Observed pass at 4,097, not a structural bound:** its native collector frame is simply lean enough for this target; the measured ceiling is 11,400--12,100 on 2 MiB (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:411-487`, `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:446-462`, `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:38-40`, `TODO.md:1094-1105`). |

The common `ReplaceAll`/`Apply` failure is therefore not "rebuilding uses too
much heap stack"; the explicit rebuilders finish. The fatal walk is the
recursive cache miss caused by the fresh `SharedExpr`/`SharedArgs` pointer
identities during the builtin's post-result evaluation
(`crates/rum-runtime/src/runtime/model/cache.rs:480-510`,
`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:224-329`).

### 2. Full class inventory and falsifiable predictions

The following inventory is complete under the source-defined boundary above:
it covers direct ancestry rebuilders, exact-path/selector rebuilders, tensor or
schema rebuilders whose recursion is rank/depth proportional, and the three
shared substitution engines. “Bounded” means a source-enforced constant-depth
front plus nonrecursive fallback or an entirely iterative driver; “unbounded”
means native call depth grows with the input/path even if the measured 4,097
case happens to pass (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-106`,
`TODO.md:1070-1077`). Every statement in the last column is explicitly a
**projection** unless it cites an observed row.

| Surface(s) | Current structural owner and bound | Falsifiable current prediction |
|---|---|---|
| `Map`, `MapAll`, `Apply` | The transform has a 16-call recursive front and `transform_parts_iterative`; it is bounded. Changed descendants rebuild ancestors (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61`, `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:94-268`). | **Projection:** a stable deepest-only result kills in recursive stability today; an active call at every level reaches evaluator `reslim`. Both are distinguished by using an undefined head versus `Identity` (`crates/rum-runtime/src/runtime/eval_core/stability.rs:213-247`, `docs/dev/codex/brief-439-deep-rebuild-class-design.md:45-50`). |
| `MapIndexed` | Unbounded direct recursion over heads, association values, and arguments; standard calls are always rebuilt (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-408`). | **Observed active projection witness:** anonymous function gives `reslim` near 110. **Projection:** stable undefined operator plus a deepest-only level spec reaches an unpolled native kill before 4,097, consistent with the older 1,600/1,775 bracket (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:52-52`, `TODO.md:1085-1105`). |
| `Scan` | Unbounded direct collector, but it returns a flat `Vec<Expr>` and ultimately `Null`, not a deep rebuilt expression (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:411-487`, `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131`). | **Observed pass/control:** 4,097 clears because callbacks are sequential; sufficiently deeper input can still kill in the native collector (measured 11,400/12,100), so “passes target” is not “bounded” (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:39-39`, `TODO.md:1096-1096`). |
| `MapAt`, `SubsetMap` | Exact target discovery/lookup/replacement is recursive in path length; each replacement may rebuild the path. No constant cap (`crates/rum-stdlib/src/functional_ops/targeted_map.rs:183-215`, `crates/rum-stdlib/src/functional_ops/targeted_map.rs:295-456`, `crates/rum-stdlib/src/functional_ops/targeted_map.rs:634-693`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:173-201`). | **Observed shallow control:** `{1}` passes. **Projection:** deepest exact path plus a stable result is an unpolled lookup/rebuild kill at sufficient path length; an active callback result can instead become evaluator `reslim` after rebuilding (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:54-54`). |
| `MapThread` | Tensor/rank traversal and result construction recurse by rank (`crates/rum-stdlib/src/functional_ops/targeted_map.rs:217-293`, `crates/rum-stdlib/src/functional_ops/targeted_map.rs:458-538`). | **Projection:** stable callback output is native process-kill mode at sufficient rank; active output nested under the rebuilt container becomes evaluator-reentry mode if that ceiling is met first (`crates/rum-runtime/src/runtime/eval_core/mod.rs:136-155`). |
| `ReplaceAll`, `ReplaceAllUnheld` | The rule traversal is iterative and suspension-safe, but both explicitly evaluate the result; recursive stability remains unbounded (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:136-314`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:5-70`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:72-138`). | **Observed:** no-match pointer reuse passes; stable leaf/head rebuild kills in unpolled stability; active rebuilt output would be evaluator `reslim` once stability reports it unstable (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:41-44`). |
| `ReplaceRepeated` | Each iteration uses the same iterative `ReplaceAll` engine, explicitly evaluates the result, then recursively loops and uses structural `PartialEq` for convergence (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:136-181`, `crates/rum-core/src/expr.rs:755-775`). | **Projection:** first stable deep rebuild has the same stability process kill today. After that is fixed, height 4,097 can pass for a quickly convergent rule because the measured equality ceiling is above 7,900; arbitrary depth remains unbounded in equality (`TODO.md:1091-1093`). |
| `Replace` | Unbounded recursive depth prepasses and mutually recursive node/child traversal; post-result evaluation is outside them (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:192-222`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-319`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:439-595`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:794-840`). | **Observed:** stable leaf replacement kills around 1,312 in unpolled native mode. **Projection after native conversion:** stable output reaches 4,097 once central stability is iterative; active-per-level output stops recoverably in evaluator reentry (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:51-51`). |
| `DeleteCases` | Direct recursive call and association walkers propagate child deletion and rebuild calls; no fallback (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:137-164`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:55-96`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:139-270`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:329-477`). | **Projection:** a deepest unique deletion is unpolled native process-kill mode at sufficient height; a no-match result can reuse the original and is the existing wide control. Source does not establish whether its native ceiling is below 4,097; a <=16-level stack-marker slope or release prologue accounting settles that without an overflow probe (`scripts/core-language-microbench.sh:584-593`). |
| `ReplaceAt`, `ReplacePart` | `ReplaceAt` uses recursive exact lookup/replacement. `ReplacePart` adds recursive exact/spec path collection, pattern-candidate collection, and recursive plan application (`crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:125-208`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:225-307`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:431-518`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan.rs:269-299`). | **Observed shallow control:** `{1} -> 9` passes by discarding the deep child. **Projection:** unique deepest replacement rebuilds every ancestor and is unpolled native mode; rule RHS that remains active may become evaluator `reslim` after native repair (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:54-54`). |
| `Part`, `Extract` | Exact paths recurse; general selectors recursively apply remaining selectors and construct selector-shaped results (`crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:99-243`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/apply.rs:112-223`, `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42`). | **Projection:** scalar exact-path selection of a leaf is native lookup-kill mode at sufficient selector length; nested selector shapes can additionally build a deep result and hit recursive stability. Source does not establish the 4,097 threshold; use shallow stack slope/static frames. |
| `Set` on `Part`; `AddTo`, `SubtractFrom`, `TimesBy`, `DivideBy`, `ApplyTo`, `AppendTo`, `PrependTo`, `Increment`, `Decrement`, `PreIncrement`, `PreDecrement` on `Part`; `AssociateTo`, `KeyDropFrom` on nested targets | Generic copy-on-write stores repeatedly call `replace_exact_path`; only the narrowly proven direct positional `Set` route mutates through a prevalidated path without rebuilding (`crates/rum-runtime/src/runtime/definitions/part_assignment.rs:123-215`, `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:523-635`, `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:15-48`, `crates/rum-stdlib/src/associations/mutation.rs:652-692`). | **Projection:** direct-route `Set` is the passing control; forced generic COW or bulk nested updates are unpolled exact-path rebuild mode. Active updated values may later trigger evaluator reentry when read, but the store itself makes no evaluator call from rebuild to commit (`crates/rum-runtime/src/runtime/definitions/part_assignment.rs:130-154`). |
| `Take`, `Drop`, `TakeDrop`; `Insert`, `Delete` | Multi-dimensional take/drop recursively consumes one spec and rebuilds one level; edit-plan application recursively consumes path depth (`crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:303-386`, `crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs:31-107`, `crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs:111-204`). | **Projection:** a 4,096-component spec/path with a stable output is unpolled native process-kill mode unless shallow stack accounting proves its frame below the 2 MiB allowance; a one-component spec is the passing control. |
| `Reverse` | `reverse_at_levels` recursively visits every call and always constructs a fresh call, selected or not (`crates/rum-stdlib/src/structure_ops/reordering.rs:4-26`, `crates/rum-stdlib/src/structure_ops/reordering.rs:157-170`). | **Projection:** stable nested lists eventually process-kill in the native traversal or, if it finishes first, in recursive stability. Source cannot rank the two or establish the 4,097 threshold; shallow stack slope/static prologues settle it. |
| `Flatten` | Same-head flattening recursively collects descendants; level-group mode recursively computes depth/records and recursively rebuilds groups (`crates/rum-stdlib/src/structure_ops/flatten.rs:224-272`, `crates/rum-stdlib/src/structure_ops/flatten.rs:274-341`, `crates/rum-stdlib/src/structure_ops/flatten.rs:344-397`). | **Observed target control:** the native same-head path clears 25,400, so 4,097 passes even though it is unbounded (`TODO.md:1097-1099`). **Projection:** an identity/permutation level-group spec that retains a deep output can expose recursive stability; no conversion is justified without a non-crashing slope showing the target at risk. |
| `Transpose` | Shape discovery and output construction recurse by tensor rank (`crates/rum-stdlib/src/structure_ops/reordering.rs:28-75`, `crates/rum-stdlib/src/structure_ops/reordering.rs:236-258`, `crates/rum-stdlib/src/structure_ops/reordering.rs:280-318`). | **Projection:** sufficiently high rank is unpolled native mode; a stable identity permutation then also presents a fresh deep tree to stability. The 4,097 threshold is not source-provable; use shallow rank slope/static frames. |
| `Join[..., n]` | Head collection, validation, and `join_level_unchecked` each recurse to requested level; the last rebuilds every output level (`crates/rum-stdlib/src/structure_ops/join.rs:21-50`, `crates/rum-stdlib/src/structure_ops/join.rs:207-293`). | **Projection:** sufficiently high `n` is unpolled native process-kill mode before or after rebuild. `Join` without a level is a one-level negative control (`crates/rum-stdlib/src/structure_ops/join.rs:53-110`). |
| `PadLeft`, `PadRight` with a dimension vector | `pad_expr_to_shape` and the generated-node count recurse with dimension count while rebuilding a shaped result (`crates/rum-stdlib/src/structure_ops/list_plumbing.rs:55-70`, `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:102-135`, `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:285-359`). | **Projection:** high-rank stable padding is unpolled native mode; scalar padding is the one-level control. This class includes following an existing shape, while pure `ConstantArray` construction is excluded below (`crates/rum-stdlib/src/structure_ops/list_plumbing.rs:218-229`). |
| `Total` with a positive level or `Infinity` | `build_total_levels`, `build_total_all_levels`, and `build_total_at_level` recurse and replace nested `List` heads with `Plus`; `eval_total` then explicitly evaluates the result (`crates/rum-runtime/src/runtime/arithmetic/parts.rs:10-23`, `crates/rum-runtime/src/runtime/arithmetic/parts.rs:54-137`). | **Projection:** current high-rank input first risks unpolled builder recursion. After an iterative builder, a nested `Plus` result is evaluator-active and therefore still stops with recoverable `reslim`; local traversal repair cannot make the `Infinity` spine reach 4,097 (`crates/rum-runtime/src/runtime/eval_core/stability.rs:334-345`, `docs/dev/codex/brief-437-native-stack-budget-design.md:517-532`). |
| `Operate`, `Through` | `Through` recursively transforms a nested head expression and `Operate` recursively descends/rebuilds the requested head level; both explicitly evaluate the result (`crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:4-43`, `crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:45-119`). | **Projection:** a stable undefined operator is unpolled native traversal/rebuild mode at sufficient head depth; an active operator becomes evaluator `reslim` after a native repair. |
| `D` | Recursive containment plus recursive derivative construction, both unbounded (`crates/rum-stdlib/src/calculus.rs:88-225`, `crates/rum-stdlib/src/calculus.rs:253-267`). | **Observed:** process kill below 2,000 on nested `Sin`. **Projection after iterative conversion:** generated `Plus`/`Times`/derivative calls make the same shape evaluator-active, so 4,097 still requires an evaluator task stack (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:53-53`, `docs/dev/codex/brief-437-native-stack-budget-design.md:419-436`). |
| `Inner`, `Dot`, `ArrayFilter`, `ArrayReduce` | Shared tensor parsing, shape computation, scalar flattening/index enumeration, and output rebuilding contain rank-proportional recursion; the four public consumers enter that machinery (`crates/rum-stdlib/src/functional_ops/arrays/mod.rs:14-136`, `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:593-639`, `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:642-689`, `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:709-733`, `crates/rum-stdlib/src/functional_ops/arrays/filter.rs:71-132`). | **Projection:** stable callback/scalar output is unpolled rank-recursion mode at sufficient rank; `Inner`/`Dot` arithmetic output and any active scalar nested under rebuilt lists can transition to evaluator `reslim` after native conversion. The 4,097 threshold needs shallow rank slope/static frames. |
| `DeleteMissing` | Both delayed target-level paths and the whole-data mode have depth-proportional recursion; the latter always rebuilds lists/associations (`crates/rum-stdlib/src/associations/missing.rs:105-187`, `crates/rum-stdlib/src/associations/missing.rs:331-454`, `crates/rum-stdlib/src/associations/missing.rs:936-1003`). | **Projection:** a stable deep keyed/list value is unpolled native mode at sufficient depth; predicate-active targeted mode may instead return evaluator `reslim`. The target threshold needs a non-crashing shallow slope because source supplies no bound. |
| `MergeNested` | The resumable driver calls `evaluate_merge_nested_group`, which recursively invokes the driver for nested association groups and rebuilds associations on return (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:55-85`, `crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-218`). | **Projection:** stable deep associations are unpolled native mode at sufficient depth; the existing construction bracket masks the exact ceiling, so first measure a <=16-level slope rather than repeating a deep construction (`TODO.md:1094-1115`). |
| `AssociationKeyDeflatten` | It consumes list-valued key paths, recursively inserts each remaining path component into a native node tree, then recursively renders that tree as nested associations. Neither phase has a source bound (`crates/rum-stdlib/src/associations/key_reshape.rs:105-124`, `crates/rum-stdlib/src/associations/key_reshape.rs:340-415`). | **Projection:** one 4,096-component key path can enter unpolled native process-kill mode during insertion or rendering, followed by recursive stability over the fresh association result. Source cannot rank those phases or establish the target threshold; use shallow path slopes/static frames. |
| `ConformSchema`, `SchemaErrors` | Both call the same recursive `conform_value`; it descends and rebuilds nested associations/lists. `SchemaErrors` discards the conformed value but still pays the recursive work (`crates/rum-stdlib/src/schema.rs:55-166`, `crates/rum-stdlib/src/schema.rs:177-300`). | **Projection:** deep schema/value pairs are unpolled native mode at sufficient depth; a leaf matcher/callback can add evaluator reentry, but it does not bound the retained native ancestor frames. The 4,097 threshold needs shallow slope/static frames. |
| Slot `Function`; explicit `Function`, `With`, `Module`; `DefinePackage`, `RestoreSealedReferences`; string-rule `StringCases`, `StringReplace` | Slot substitution recursively rebuilds every call. Lexical substitution is changed-propagating but recursively rebuilds every ancestor above a replaced symbol; these surfaces invoke it and then generally evaluate the product (`crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:3-34`, `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:287-312`, `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:366-427`, `crates/rum-runtime/src/runtime/scoping/callable.rs:5-75`, `crates/rum-runtime/src/runtime/scoping/lexical/with.rs:105-148`, `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:119-260`, `crates/rum-runtime/src/runtime/scoping/packages.rs:235-329`, `crates/rum-runtime/src/runtime/scoping/packages.rs:332-408`, `crates/rum-stdlib/src/strings/patterns/search/rules.rs:564-575`). | **Projection:** a deepest bound symbol in an otherwise stable held body kills in unpolled substitution/rebuild today; empty bindings or absent symbols reuse the original and are controls. After iteration, stable products reach 4,097, while active bodies stop in evaluator reentry (`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:371-383`). |
| Rule binding substitution used by downvalues, upvalues, subvalues, RHS guards, `Replace`, `ReplaceAll`, `ReplaceAllUnheld`, `ReplaceRepeated`, `ReplaceList`, rule-form `Cases`, `ReplaceAt`, `ReplacePart`, and `CatchExceptions` | `substitute_bindings` and `substitute_rule_bindings_changed` recursively follow RHS templates and rebuild changed ancestry; empty bindings have an explicit clone fast path. The named consumers are direct call sites (`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:47-108`, `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:125-160`, `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:280-328`, `crates/rum-runtime/src/runtime/definitions/application/down_values.rs:152-358`, `crates/rum-runtime/src/runtime/definitions/application/up_values.rs:83-279`, `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:103-322`, `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:202-298`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:433-454`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:245-280`, `crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:442-458`). | **Projection:** a nonempty match binding occurring only at the deepest RHS leaf is unpolled substitution/rebuild mode; an empty-binding rule is the pointer-reuse control. Subsequent active output is evaluator-reentry mode after the substitution is made iterative. |

Negative controls and exclusions are part of the falsifier:

- `MapApply`, `Comap`, and `ComapApply` use the common transformer but are fixed
  to level 1 rather than accepting an unbounded level spec, so they cannot make
  fresh ancestry proportional to input height; their outer result can still
  contain a shared deep subtree (`crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:66-87`,
  `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:302-443`,
  `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:13-55`,
  `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:150-163`).
- `Cases` without a rule, `Level`, `Count`, `Position`, `MemberQ`, `FreeQ`,
  `MatchQ`, and `ToString` are collectors, predicates, or renderers rather than
  deep-result rebuilders; their supplied certificates are therefore legitimate
  negative controls even though some underlying walks are independently
  unbounded (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:31-56`,
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:85-136`,
  `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:20-66`).
- `Nest`, `NestList`, `Fold` families, `Array`, `Outer`, `ConstantArray`, random
  arrays, and iteration constructs can generate tall values, but they do not
  follow and rebuild an existing ancestry path, so they are outside this class.
  A tall active result can still enter the separate strict-residual evaluator
  class already ruled unreachable at 4,097 (`crates/rum-stdlib/src/functional_ops/arrays/generators.rs:84-136`,
  `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:37-52`,
  `TODO.md:1215-1236`).
- `AssociationKeyFlatten` has an unbounded native source walk, but it emits a
  flat association whose keys are flat `List` paths and reuses leaf values; it
  is an adjacent safety audit rather than a deep-result rebuilder. Its inverse
  `AssociationKeyDeflatten` is included because it builds a nested result
  proportional to a supplied path (`crates/rum-stdlib/src/associations/key_reshape.rs:126-142`,
  `crates/rum-stdlib/src/associations/key_reshape.rs:417-440`).
- Polynomial transforms recursively parse algebraic structure, but a
  non-polynomial nested call becomes a shared indeterminate and canonical
  polynomial output is flattened; they are adjacent native-recursion risks,
  not ancestry-preserving deep rebuilders under this definition
  (`crates/rum-stdlib/src/polynomials/core.rs:60-110`,
  `crates/rum-stdlib/src/polynomials/core.rs:405-416`). Codecs return strings or
  generate expressions from text, and string-pattern searches return strings,
  booleans, or shallow collections; only their rule-RHS lexical substitution
  call is included above (`crates/rum-stdlib/src/json/encode.rs:76-76`,
  `crates/rum-stdlib/src/strings/patterns/search/rules.rs:564-575`).

This inventory does not imply that every unbounded helper fails below 4,097.
The repository itself distinguishes “unbounded” from “known below the target”
and requires measurement before scheduling speculative conversions
(`TODO.md:1070-1077`). The safe non-crashing discriminator is a stack-marker
slope over shallow depths no greater than 16 plus release-binary prologue/call
edge accounting, the method already reconciled against the evaluator packet
(`docs/dev/codex/brief-437-native-stack-budget-design.md:244-272`).

### 3. The depth-16 `Map`/`Apply` fallback

The fallback is **reachable and is reached** for a unary spine deeper than 16,
regardless of `{0, Infinity}`. Both bounded functions decrement
`remaining_call_depth` with `checked_sub(1)?` whenever they encounter another
call; the seventeenth call returns `None`. The wrappers immediately feed that
`None` to `unwrap_or_else(transform_parts_iterative)`, and the iterative driver
then visits/schedules the entire tree (`crates/rum-stdlib/src/functional_ops/apply_map/traversal/recursive.rs:3-67`,
`crates/rum-stdlib/src/functional_ops/apply_map/traversal/recursive.rs:113-180`,
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61`,
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:94-149`).

The level spec does not short-circuit this decision: matching is tested while
finishing nodes, after their child depths are known, and changed descendants
are rebuilt on the iterative completion stack
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:150-203`). Thus
`Map[Identity, ..., {0, Infinity}]` is transformed iteratively; its recursion is
the subsequent evaluator descent through the active nested result. Conversely,
`Apply[List, ...]` produces a stable-but-fresh result and dies in the subsequent
recursive stability miss (`crates/rum-runtime/src/runtime/resolution.rs:318-343`,
`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-34`).

### 4. Certificate-panel repair

#### Certificate rule

A certificate for a **rebuilding traversal** must satisfy all three conditions:

1. Pick one unique deepest target whose transformation is semantically
   different, so its `changed` bit must propagate through every ancestor and
   no implementation can pass by returning the input pointer
   (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:171-203`,
   `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:215-245`).
2. Assert the exact rebuilt value, not `Depth`, `Length`, `StringLength`, or
   another summary that could be correct when the transform never ran. Extract
   the returned `Atom::String` directly so the Rust-side assertion never formats
   the deep `Expr` recursively (`crates/rum-core/src/expr.rs:749-790`,
   `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:25-66`).
3. Use a descending level spec whose shallow semantics are independently
   asserted on a three-level fixture. A deep certificate must not itself be the
   only proof that the selector descends (`crates/rum-stdlib/src/tests/traversal.rs:142-165`).

That rule would reject all three bad certificates: `Map` and `Apply` use a
cross-zero range that selects only the root, while `ReplaceAll` chooses a symbol
absent from the value; all three assert only `Depth`
(`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:58-83`).

#### Exact replacements

Keep the other **nine** tests unchanged: `MatchQ`, construction, `ToString`,
`Cases`, `Level`, `MemberQ`, `Count`, `Position`, and `FreeQ`. They certify a
predicate, constructor, renderer, or collector rather than a rebuilding
traversal, and the file contains exactly those nine plus the three defective
rebuilders (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:31-56`,
`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:85-136`). Replace
exactly `Map`, `Apply`, and `ReplaceAll` as follows:

- `Map`: evaluate `ToString[Map[deepMapProbe, x, {-1}], InputForm]`; expect
  4,096 opening braces, `deepMapProbe[1]`, and 4,096 closing braces. The unique
  atom is changed and every `List` ancestor must be rebuilt. A shallow companion
  pins `Map[deepMapProbe, {{{1}}}, {-1}]` to
  `{{{deepMapProbe[1]}}}` (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:108-203`,
  `crates/rum-stdlib/src/strings/text/builtins.rs:624-645`).
- `Apply`: evaluate
  `ToString[Apply[deepApplyProbe, x, {-2}], InputForm]`; expect 4,095 opening
  braces, `deepApplyProbe[1]`, and 4,095 closing braces. `{-2}` selects the
  unique innermost `List[1]`, changes its head, and forces all 4,095 outer lists
  to rebuild. A shallow companion pins
  `Apply[deepApplyProbe, {{{1}}}, {-2}]` to
  `{{deepApplyProbe[1]}}` (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:150-203`,
  `crates/rum-runtime/src/runtime/model/traversal.rs:34-58`).
- `ReplaceAll`: evaluate `ToString[ReplaceAll[x, 1 -> 2], InputForm]`; expect
  4,096 opening braces, `2`, and 4,096 closing braces. The unique leaf match
  sets `changed` at every completion frame
  (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:199-245`).

The test helper should return an `Expr`, and a new string-certificate helper
should require `Expr::Atom(Atom::String(value))`; expected strings should be
built in Rust with `"{".repeat(n)`, the exact center, and `"}".repeat(n)`.
This keeps the final assertion exact without asking Rust `Display` to walk the
rebuilt tree (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:15-29`,
`crates/rum-core/src/expr.rs:749-790`). The three tests remain release-only on
the existing 2 MiB thread and target input physical height 4,097
(`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-12`).

#### `{-4097, 0}` verdict

`{-4097, 0}` matching level 0 only is **rum's intended, test-pinned standard
level-spec semantics, not a separate `Map`/`Apply` defect**. For a range
spanning negative and nonnegative levels, `matches` deliberately requires both
the positive-level and negative-depth conditions; with maximum 0, only level 0
can pass. Dedicated tests pin `{-3, 0}` and `{-Infinity, 0}` to the root-only
result (`crates/rum-runtime/src/runtime/model/traversal.rs:34-58`,
`crates/rum-stdlib/src/tests/traversal.rs:142-165`). The generated reference
only says “standard level specification,” so its prose is underspecified, but
it does not contradict the implementation or tests; clarification belongs in a
separate documentation brief and this repair must not change semantics
(`docs/reference/builtins.md:2886-2896`,
`docs/reference/builtins.md:2969-3018`,
`docs/testing-strategy.md:60-72`).

### 5. Ordered repair plan

#### Repair choices and enforcement

There are three distinct choices, and they must not be reported as equivalent:

- A local stack poll or explicit maximum-height check is **low-cost safety
  only**: it converts an OS abort to an in-band error, but it deliberately
  rejects a 4,097 traversal. The current evaluator poll cannot enforce this
  inside a pure helper because evaluation depth does not advance there
  (`crates/rum-runtime/src/runtime/eval_core/mod.rs:34-53`).
- A bounded recursive front plus explicit heap task/result stacks is
  **moderate-cost safety plus stable-result contract**. The 16-level
  `Map`/`Apply` implementation is the local model; suspension-capable engines
  must put owned work/results in their existing continuation state rather than
  retaining Rust frames (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-106`,
  `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:136-255`).
- A whole-evaluator explicit task stack/trampoline is **high-cost contract for
  active results**. Frame polishing cannot reduce the measured 4,736-byte
  evaluator packet below the <512-byte/level allowance for height 4,097; this
  is a separate architecture project and is not bundled into local traversal
  briefs (`docs/dev/codex/brief-437-native-stack-budget-design.md:419-436`,
  `docs/dev/codex/brief-437-native-stack-budget-design.md:517-532`).

In this plan, **safety** means “the repaired site itself cannot abort the
process through depth-proportional native recursion”; **contract** means “the
operation successfully accepts an input of physical height 4,097 on the
existing 2 MiB certificate thread.” A local repair can satisfy safety while an
active result still returns recoverable `reslim`, so every gate needs a stable
probe and an active-result probe (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:29-34`).

#### Steps

1. **Make post-result structural checks iterative; replace the three
   certificates.** Convert the cache-aware stability miss to a postorder task
   machine while preserving pointer-key lookup, early unstable decisions,
   bloom union order, dirty-overlap policy, and stable-only insertion. Also
   make the `Expr` portion of the active memory proxy iterative, because an
   armed memory constraint runs that recursive walk *before* stability and
   after each evaluator step; default sessions have no memory limit, but
   sandbox safety must not depend on that default
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:200-329`,
   `crates/rum-runtime/src/runtime/eval_core/stability.rs:5-96`,
   `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-161`,
   `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`,
   `crates/rum-runtime/src/runtime/model/config.rs:25-36`). **Cost: high hot-path
   risk**, because every fixed-point pass consults stability; enforce with
   helper equivalence tests, the repaired release certificates, memory-limited
   deep stable coverage, and rum-vs-rum A/B on stability/evaluator/rewrite
   labels (`crates/rum-runtime/src/runtime/resolution.rs:318-326`,
   `scripts/core-language-microbench.sh:463-478`,
   `scripts/core-language-microbench.sh:502-558`).

2. **Convert `Replace` completely, not just the visible child loop.** Replace
   both depth helpers and the `replace_owned`/child mutual recursion with an
   owned postorder work machine integrated with existing suspension frames;
   retain rule order, head inclusion, sequence splicing, association handling,
   and exactly-once callback behavior
   (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-319`,
   `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:434-595`,
   `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:794-840`).
   **Cost: high semantic/suspension risk and hot candidate-path risk.** Enforce
   with focused runtime tests, suspend/resume parity, the stable 4,097 leaf
   certificate, an active-result recoverable-error certificate, and the
   existing `replace-wide-no-match`/guarded rewrite A/B rows
   (`scripts/core-language-microbench.sh:559-613`).

3. **Convert `MapIndexed`; convert `Scan` in the same isolated file change.**
   Add a bounded shallow front and one explicit path-aware task driver for
   `MapIndexed`; its finish record owns the path/index information and preserves
   association key semantics and depth-first order. Give `Scan` an iterative
   collector using the same visit order, even though it already clears the
   target, so this touched module does not retain an unbounded native sibling
   (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-487`).
   **Cost: medium; hot traversal construction risk.** Enforce with exact shallow
   order/path tests, the stable deepest-only 4,097 case, the existing active
   anonymous-function `reslim` expectation, the existing `Scan` certificate,
   and new exact A/B labels plus general list/evaluator controls
   (`crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:59-74`,
   `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131`,
   `scripts/core-language-microbench.sh:473-478`).

4. **Convert both calculus descents.** First replace
   `expr_contains_exact_subexpr` with an explicit read-only stack; then express
   derivative construction as a postorder task/result machine that preserves
   short-circuit zero handling and the exact `Plus`/`Times` combination order
   (`crates/rum-stdlib/src/calculus.rs:88-225`,
   `crates/rum-stdlib/src/calculus.rs:253-267`). **Cost: high algebraic semantic
   risk, but not an evaluator hot-path change.** Enforce with the full calculus
   suite, shallow exact derivatives, and a 4,097 nested-`Sin` safety test that
   must return an in-band resource error rather than requiring successful
   evaluation. Add and A/B a small exact `D` label because the implementation
   allocates `Expr` nodes even though dispatch itself is not being changed
   (`docs/dev/codex/brief-437-native-stack-budget-design.md:526-532`).

5. **Consolidate exact paths, selectors, and edit plans behind iterative
   primitives.** Make exact lookup a loop; make exact replacement a validated
   descent plus reverse rebuild; make multi-selector shape traversal,
   `ReplacePart` candidate collection/plan application, and take/drop/edit
   plans explicit task machines. Route `MapAt`, `SubsetMap`, `ReplaceAt`,
   `ReplacePart`, `Part`, `Extract`, `Set`/held updates, `AssociateTo`,
   `KeyDropFrom`, `Take`, `Drop`, `TakeDrop`, `Insert`, and `Delete` through
   those primitives without changing messages or mutation atomicity
   (`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-201`,
   `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/apply.rs:112-223`,
   `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:225-518`,
   `crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs:31-204`).
   **Cost: very high ripple and hot candidate/`Expr` construction risk.** Split
   it into reviewable helper commits/patches but keep one semantic gate; enforce
   with unit equivalence for plans, mutation rollback/suspension tests, stable
   4,097 exact-path cases, and A/B on `part-list`, association part lookup, all
   `replacepart-*` rows, plus new `MapAt`/edit labels
   (`scripts/core-language-microbench.sh:450-478`,
   `scripts/core-language-microbench.sh:599-613`).

6. **Convert residual rewrite and all substitution engines.** Give
   `DeleteCases` the explicit visit/finish pattern. Convert slot, lexical, and
   rule-binding substitution to changed-propagating explicit stacks while
   preserving scoping, alpha-renaming, sequence splicing, pointer reuse, and
   empty-binding fast paths; all listed callers then inherit the repair
   (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:55-477`,
   `crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:3-34`,
   `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:287-427`,
   `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:47-160`).
   **Cost: very high semantic and hot definition-dispatch risk.** Enforce with
   pointer-sharing unit tests, lexical-shadow/alpha tests, every rule consumer's
   suspend/resume suite, stable 4,097 RHS/body certificates, and A/B on
   `deletecases-*`, `module-local`, `with-local`, and all down/up/subvalue hit
   and miss neighbors (`scripts/core-language-microbench.sh:480-535`,
   `scripts/core-language-microbench.sh:584-593`).

7. **Conditionally convert remaining structural/head/tensor rebuilders.** First
   run the non-crashing <=16-level slope/static-frame gate for `Reverse`,
   `Transpose`, levelled `Join`, shaped pads, `Total`, `Operate`, `Through`,
   `MapThread`, and the shared tensor helpers. Convert only a site whose
   projected 4,097 consumption exceeds the 2 MiB allowance; `Flatten` already
   has measured headroom and remains a monitored control unless its
   level-group path measures differently (`TODO.md:1070-1105`). Use bounded
   fronts plus explicit builders and preserve exact ordering/shape semantics
   (`crates/rum-stdlib/src/structure_ops/reordering.rs:157-318`,
   `crates/rum-stdlib/src/structure_ops/join.rs:207-293`,
   `crates/rum-runtime/src/runtime/arithmetic/parts.rs:54-137`,
   `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:593-733`). **Cost:
   high but conditional; several independent algorithms.** Enforce per-site
   unit tests, stable 4,097 rank/depth certificates only for converted sites,
   active-result `reslim` tests for `Total`/arithmetic cases, and newly added
   exact A/B labels with list/arithmetic controls
   (`scripts/core-language-microbench.sh:463-478`).

8. **Conditionally convert association/schema rebuilders.** Apply the same
   <=16-level slope/static gate separately to `DeleteMissing`, `MergeNested`,
   `AssociationKeyDeflatten`, and shared `ConformSchema`/`SchemaErrors`; do not infer a failure from source
   recursion alone. Sites that fail the budget gate receive owned
   visit/finish state integrated with their existing suspension drivers
   (`crates/rum-stdlib/src/associations/missing.rs:331-1003`,
   `crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-218`,
   `crates/rum-stdlib/src/associations/key_reshape.rs:340-415`,
   `crates/rum-stdlib/src/schema.rs:177-300`). **Cost: high callback,
   association-order, and suspension risk.** Enforce with association/schema
   exact-output and resume tests, stable 4,097 certificates for converted
   sites, active callback `reslim` tests, and new exact A/B labels plus existing
   association lookup controls (`scripts/core-language-microbench.sh:450-454`).

#### Height-4,097 matrix after each step

“Reaches” below means a stable-result probe accepts an **input** of physical
height 4,097; a `Map` at every level can make an output taller than its input,
so no claim is made for that separate output height. “Cannot” means the native
site is safe after the local step but the current evaluator must return
`reslim` for a call-active chain (`docs/dev/codex/brief-437-native-stack-budget-design.md:517-532`).

| Point in plan | Reaches input height 4,097 | Still cannot / not yet established |
|---|---|---|
| Baseline | The supplied construction, `ToString`, `MatchQ`, plain `Cases`, `Level`, `MemberQ`, `Count`, `Position`, `FreeQ`, and `Scan` cases; shallow-path `MapAt`/`ReplacePart` also pass but do not certify deep rebuild (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:36-54`). | Matched `ReplaceAll`, stable all-level `Apply`, and `Replace`/`D` process-kill; active `Map`/`MapIndexed` stop in-band. Other inventory rows are unestablished until the non-crashing slope gate (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:41-53`, `TODO.md:1070-1077`). |
| After step 1 | The new deepest-only stable `Map`, stable `Apply`, and matching `ReplaceAll` certificates; the same repair covers `ReplaceAllUnheld` and a quickly convergent stable `ReplaceRepeated` at this target (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:59-138`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:136-181`, `TODO.md:1091-1093`). | `Map[Identity, ..., {0, Infinity}]` and any deep rebuilt tree with an active leaf still return evaluator `reslim`; `Replace`, `MapIndexed`, and `D` retain native recursion (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`). |
| After step 2 | Stable deepest-leaf `Replace` now reaches 4,097 because both its traversal and post-result stability are iterative (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:192-222`). | A `Replace` result active along the spine remains evaluator-limited; `MapIndexed` and `D` remain as above (`docs/dev/codex/brief-437-native-stack-budget-design.md:526-532`). |
| After step 3 | `MapIndexed` with a stable undefined operator and a deepest-only selection reaches 4,097; `Scan` retains its already-passing exact order with a structural bound (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-487`). | The supplied `#1 &` at every level still returns `reslim`; traversal iteration cannot lift evaluator recursion (`crates/rum-runtime/src/runtime/scoping/callable.rs:59-75`). |
| After step 4 | `D` no longer process-kills on the 4,097 nested-`Sin` safety probe (`crates/rum-stdlib/src/calculus.rs:88-267`). | That derivative is built from active `Plus`/`Times` calls, so successful completion at 4,097 is still impossible without the evaluator trampoline; expected result is recoverable `reslim` (`docs/dev/codex/brief-437-native-stack-budget-design.md:419-436`). |
| After step 5 | Stable deepest paths/specs reach 4,097 for `MapAt`, `SubsetMap`, `ReplaceAt`, `ReplacePart`, `Part`, `Extract`, generic part assignment/updates, nested association mutation, `Take`/`Drop`/`TakeDrop`, `Insert`, and `Delete` (`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-201`, `crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs:31-204`). | Any returned deep tree with an active leaf still stops in-band; mutation success does not imply later evaluation of an active stored value can reach 4,097 (`crates/rum-runtime/src/runtime/eval_core/mod.rs:136-155`). |
| After step 6 | Stable unique deletion and stable deepest substitutions reach 4,097 for `DeleteCases`, slot/explicit `Function`, `With`, `Module`, package rewriting, string-rule rewriting, rule-form `Cases`, all `Replace*` consumers, exception handlers, and down/up/subvalue application (`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:47-160`, `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:366-427`). | Active substituted bodies/RHSs remain evaluator-limited (`crates/rum-runtime/src/runtime/scoping/callable.rs:59-75`). |
| After conditional step 7 | Each site that fails the slope gate and is converted reaches 4,097 for a stable-output variant: `Reverse`, `Transpose`, levelled `Join`, shaped pads, `Operate`, `Through`, stable `MapThread`, and stable tensor rebuilding. `Flatten` already clears the target (`TODO.md:1097-1099`). | `Total[..., Infinity]`, `D`, `Inner`/`Dot` arithmetic chains, and active operator variants remain evaluator-limited. A site that passes the slope gate is “measured target-safe,” not structurally bounded (`crates/rum-runtime/src/runtime/arithmetic/parts.rs:95-104`, `docs/dev/codex/brief-437-native-stack-budget-design.md:526-532`). |
| After conditional step 8 | Converted stable variants of `DeleteMissing`, `MergeNested`, `AssociationKeyDeflatten`, `ConformSchema`, and `SchemaErrors` reach 4,097; unconverted variants have a recorded non-crashing budget proof for the target (`crates/rum-stdlib/src/associations/missing.rs:936-1003`, `crates/rum-stdlib/src/associations/key_reshape.rs:340-415`, `crates/rum-stdlib/src/schema.rs:177-300`). | Active nested results remain evaluator-limited; source-only unboundedness is not promoted to a below-target claim (`TODO.md:1070-1077`). |

This is a target-height plan, not a claim of arbitrary-depth safety. `Expr`'s
`PartialEq`, `Hash`, `Display`, serde, recursive drops, and other resource-proxy
helpers remain a residual cross-cutting audit; several have measured headroom
at 4,097, but none is thereby proven bounded (`TODO.md:1070-1100`,
`crates/rum-core/src/expr.rs:755-775`). In particular, locally recursive memory
preflight helpers in stdlib families must be converted alongside a selected
site if the gate exercises `MemoryConstrained`; central step 1 fixes only the
runtime-wide expression meter invoked by the fixed-point driver
(`crates/rum-stdlib/src/arithmetic/stats.rs:46-80`,
`crates/rum-stdlib/src/structure_ops/surface.rs:261-283`,
`crates/rum-stdlib/src/associations/missing.rs:212-228`).

### 6. Draft implementation briefs

These are raw drafts in `TEMPLATE.md` shape; each has Goal, Context pointers,
Scope, Deliverable, pitfalls, mandatory verification, Hard rules, and Report
format. They are not dispatch-ready, and the conditional drafts must be
rewritten after their named budget gate selects concrete sites
(`docs/dev/codex/TEMPLATE.md:17-37`, `TODO.md:1070-1077`).

#### Draft 1 — iterative post-result stability and real rebuild certificates

**Goal.** Make cache-miss stability classification and the runtime-wide active
memory expression meter bounded in native stack, then replace the three false
depth certificates with exact stable-result rebuild certificates at physical
height 4,097 (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:200-329`,
`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`,
`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:58-83`).

**Context pointers.** Read `AGENTS.md`, brief 437's evaluator-stack ruling,
this report sections 1/4/5, and the cache key/entry definitions before editing.
Acceptance is the named semantic tests, release certificates, and A/B rows—not
an assertion that the new stack is iterative
(`docs/dev/codex/brief-437-native-stack-budget-design.md:419-436`,
`crates/rum-runtime/src/runtime/model/cache.rs:480-510`,
`docs/dev/codex/TEMPLATE.md:47-53`).

**Scope.** Files allowed:

- `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`
- `crates/rum-runtime/src/runtime/eval_core/stability.rs` and one coherent new
  sibling module if needed
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`
- focused tests under `crates/rum-runtime/src/runtime/tests/`
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `crates/rum-stdlib/src/tests/traversal.rs`
- `scripts/core-language-microbench.sh`

Do not change `Expr` representation, level-spec semantics, public limits, or
other docs (`crates/rum-core/src/expr.rs:749-829`,
`docs/dev/codex/TEMPLATE.md:55-69`).

**Deliverable.** Add a postorder work/result machine that produces the same
stability/bloom result and cache side effects as the recursive implementation;
preserve stable-only insertion, generation/pointer/dirty miss accounting,
short-circuit order, and data-constructor policies. Make only the `Expr` walk
inside the runtime memory proxy iterative. Replace exactly the three
certificates and add the shallow `{-1}`/`{-2}` semantic checks described in
section 4. Add small exact labels
`map-deepest-rebuild-small`, `apply-deepest-rebuild-small`, and
`replaceall-leaf-rebuild-small` to the benchmark script
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:224-329`,
`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-96`,
`scripts/core-language-microbench.sh:538-558`).

**rum / Rust pitfalls (read twice).** Preserve pointer-key caching and do not
clone whole trees; task entries should retain `Arc`-shared nodes. Preserve the
observable cache counters and bloom-union/early-exit order. Do not put a
per-node allocation on cache hits. Do not format the rebuilt `Expr` in Rust;
extract the returned string atom. The active memory count must remain
saturating and must preserve the special `ByteArray` accounting
(`crates/rum-core/src/expr.rs:573-708`,
`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405`,
`docs/dev/codex/TEMPLATE.md:78-100`).

**Verification / gate.** Run, through `scripts/run-bounded.py` with timeouts
derived from a prior run of the same command: `cargo fmt --check`,
`cargo test -p rum-runtime`, `cargo test -p rum-stdlib`, then
`cargo test --workspace`. After the shallow/unit gates are green, run the
release-only `iterative_traversal_depth` ignored tests on their declared 2 MiB
threads, including a memory-constrained stable rebuilt-result case
(`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-29`,
`docs/dev/codex/TEMPLATE.md:102-123`,
`docs/dev/codex/TEMPLATE.md:147-155`). Run best-of-2 base/new
`core-language-microbench.sh` with the three new exact labels plus
`literal-integer`, `list-literal-small`, `sameq-list`,
`downvalue-exact-hit`, `downvalue-exact-miss`, and `replaceall-simple`; reject
any target or same-tier-neighbor regression
(`scripts/core-language-microbench.sh:463-478`,
`scripts/core-language-microbench.sh:502-558`).

**Stop-loss.** Stop and report if exact cache-side-effect equivalence cannot be
stated, if a second attempt repeats the same correctness/suspension failure, or
if two measured revisions fail to move an A/B regression. Do not “fix” the
gate by weakening the exact output, reducing 4,097, changing level semantics,
or disabling caching (`docs/dev/codex/TEMPLATE.md:96-100`,
`docs/testing-strategy.md:60-72`).

**Hard rules.** No git mutation, network, third-party oracle, new `unsafe`, or
files outside Scope; no unconditional diagnostic cost; use bounded commands
and preserve all unrelated user changes (`AGENTS.md:1-1`,
`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** List files changed and invariants preserved; paste verbatim
test/ignored-test output and the before/after `rum_seconds` rows; report the
three exact rebuilt strings conceptually by brace count/center, the memory-limit
case, and any remaining active-result `reslim` separately
(`docs/dev/codex/TEMPLATE.md:107-123`).

#### Draft 2 — fully iterative `Replace`

**Goal.** Remove all depth-proportional Rust recursion from `Replace`—both
depth calculation and traversal—while preserving exact rules, levels, heads,
associations, splicing, messages, and suspension behavior
(`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:192-222`,
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-319`,
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:434-595`).

**Context pointers.** Read `AGENTS.md`, this report section 1, the iterative
`ReplaceAll` state machine, and every `Replace` resume frame. Use
`ReplaceAllTraversalWork` only as a structural pattern; `Replace` must retain
its different top-down match/level/depth semantics
(`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:136-314`,
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:1-1`).

**Scope.** Files allowed:

- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs`
- focused runtime rewrite/suspension tests
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh`

Do not touch matcher semantics, `ReplaceAll`, standard levelspec parsing, or
public resource limits (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:251-268`).

**Deliverable.** Introduce owned visit/finish tasks and completed outcomes that
carry level/depth/change state across suspension. Remove or bound all calls from
`replace_owned` back into itself and replace both recursive depth helpers. Add
a stable unique-leaf 4,097 `Replace` certificate and an active-spine case that
asserts a recoverable resource error, never process termination. Add a small
exact `replace-leaf-rebuild-small` A/B label
(`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:278-318`,
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:489-590`,
`scripts/core-language-microbench.sh:595-613`).

**rum / Rust pitfalls (read twice).** A rule is attempted before children when
its level matches; a successful replacement does not traverse that replacement.
Preserve `Heads`, association-value treatment, `Sequence` splicing and computed
depth of spliced results. Every suspended callback must resume exactly once
with the same rule-plan cursor and partial result; do not turn a suspension
frame into an eager evaluator call
(`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-319`,
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:453-512`,
`docs/dev/codex/TEMPLATE.md:90-95`).

**Verification / gate.** Bounded `cargo fmt --check`,
`cargo test -p rum-runtime`, `cargo test -p rum-stdlib`, and
`cargo test --workspace`; then the new ignored release certificate after all
shallow tests pass. A/B best-of-2 the new target label with
`replace-wide-no-match`, `rewrite-guarded-hit`,
`rewrite-guarded-decline-fallback`, and all three `replacepart-*` neighbors
(`scripts/core-language-microbench.sh:559-613`,
`docs/dev/codex/TEMPLATE.md:102-123`).

**Stop-loss.** Stop if exact top-down semantics require a global evaluator
change, if suspension parity fails twice for the same reason, or if two
isolated revisions do not remove the A/B regression. Do not cap depth or change
the level spec to make the certificate pass (`docs/dev/codex/TEMPLATE.md:96-100`).

**Hard rules.** No git/network/oracle/new `unsafe`; modify Scope only, preserve
safe Rust and user changes, and use bounded commands
(`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** State which recursive edges were removed; paste all gate
output and target/neighbor A/B rows; report the stable 4,097 result and the
active-spine in-band result as separate safety/contract outcomes
(`docs/dev/codex/TEMPLATE.md:107-123`).

#### Draft 3 — iterative `MapIndexed` and `Scan`

**Goal.** Replace the two unbounded recursive walkers left in the converted
apply/map module with bounded-front explicit-stack traversals while preserving
path values, association behavior, visit order, and callback order
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-487`).

**Context pointers.** Read `AGENTS.md`, this report sections 1/3, the existing
`TransformWork` driver, normalization, and apply/map tests. `MapIndexed` and
`Scan` have different outputs and should share scheduling concepts, not a
forced common result type (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:64-299`,
`crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:182-300`).

**Scope.** Files allowed:

- `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs`
- `crates/rum-stdlib/src/functional_ops/apply_map/traversal/recursive.rs`
- `crates/rum-stdlib/src/tests/functional_ops/apply_map.rs`
- relevant session-request tests under `crates/rum-stdlib/src/tests/session_requests/`
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh`

Do not change level-spec semantics, callback evaluation, or `Map`/`Apply`
output (`crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:59-131`).

**Deliverable.** Add a depth-16 front and fallback for `MapIndexed`, with owned
paths only where needed and finish records that preserve one-based argument
indices/zero head indices. Replace `Scan` collection recursion with an
explicit-stack depth-first collector producing the identical flat part order.
Add a stable undefined-operator deepest-only 4,097 `MapIndexed` certificate,
retain the 4,097 `Scan` certificate, and pin the anonymous-function active case
to in-band resource failure. Add small exact
`mapindexed-deepest-rebuild-small` and `scan-all-levels-small` labels
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:319-408`,
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:423-487`).

**rum / Rust pitfalls (read twice).** `MapIndexed` currently rebuilds every
standard call, includes association values but not keys, and emits exact path
expressions at selection time. `Scan` appends a selected parent after its
children and evaluates the resulting vector sequentially. Avoid cloning the
whole path at every unselected node; preserve `Arc` sharing and callback
suspension order (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:329-408`,
`crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:446-462`).

**Verification / gate.** Bounded `cargo fmt --check`,
`cargo test -p rum-stdlib`, and `cargo test --workspace`; then the two ignored
release certificates after shallow order/path tests. A/B best-of-2 both new
labels with `list-literal-small`, `part-list`, `sameq-list`, and the step-1
`map-deepest-rebuild-small` label (`scripts/core-language-microbench.sh:473-478`,
`docs/dev/codex/TEMPLATE.md:102-123`).

**Stop-loss.** Stop if exact traversal order changes, path allocation regresses
twice without improvement, or the active case process-kills instead of failing
in band. Do not reinterpret the active `reslim` as a traversal failure
(`docs/dev/codex/brief-437-native-stack-budget-design.md:526-532`,
`docs/dev/codex/TEMPLATE.md:96-100`).

**Hard rules.** Scope only; no git/network/oracle/unsafe; bounded commands; no
unconditional profiling cost (`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** Paste semantic/release gates and target/neighbor A/B rows;
state maximum native recursive front, stable 4,097 outcome, active outcome,
and exact `Scan` ordering evidence (`docs/dev/codex/TEMPLATE.md:107-123`).

#### Draft 4 — iterative calculus containment and differentiation

**Goal.** Remove depth-proportional native recursion from both the containment
prepass and derivative construction, without changing any documented `D`
result or claiming the active nested derivative can evaluate to completion at
height 4,097 (`crates/rum-stdlib/src/calculus.rs:88-225`,
`crates/rum-stdlib/src/calculus.rs:253-267`).

**Context pointers.** Read `AGENTS.md`, this report section 1, all calculus
tests, and brief 437's evaluator ceiling. The acceptance distinction is
process safety versus successful evaluation
(`crates/rum-stdlib/src/tests/calculus.rs:1-1`,
`docs/dev/codex/brief-437-native-stack-budget-design.md:517-532`).

**Scope.** Files allowed:

- `crates/rum-stdlib/src/calculus.rs` and one coherent sibling module if the
  file-size rule requires it
- `crates/rum-stdlib/src/tests/calculus.rs`
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh`

Do not change arithmetic canonicalization, evaluator limits, or the catalog
contract (`docs/code-organization-guidelines.md:64-89`).

**Deliverable.** Implement iterative exact-subexpression containment and a
postorder derivative task machine preserving branch order, zero-elision,
partial-derivative fallback, and additive/multiplicative combination order.
Add shallow exact-result regressions and a release-only 4,097 nested-`Sin`
**safety** certificate whose accepted outcome is the current in-band resource
error. Add exact `d-nested-sin-small` microbench coverage
(`crates/rum-stdlib/src/calculus.rs:116-225`,
`docs/dev/codex/brief-439-deep-rebuild-class-design.md:71-78`).

**rum / Rust pitfalls (read twice).** Do not memoize or reorder calls to
session arithmetic combiners unless tests prove identical messages/evaluation
order. Task results must not deep-clone shared expressions. The release safety
test must distinguish `EvalError`/`TerminatedEvaluation` from thread panic;
never relax it to “any failure” (`crates/rum-stdlib/src/calculus.rs:208-225`,
`docs/testing-strategy.md:41-55`).

**Verification / gate.** Bounded `cargo fmt --check`,
`cargo test -p rum-stdlib`, and `cargo test --workspace`; then the release-only
safety test after shallow exact derivatives pass. A/B best-of-2
`d-nested-sin-small` with `literal-integer`, `list-literal-small`, and any
same-tier arithmetic rows; add the label identically to base and candidate
scripts (`scripts/core-language-microbench.sh:463-478`,
`docs/dev/codex/TEMPLATE.md:102-123`).

**Stop-loss.** Stop on a second repeated algebraic-order mismatch, if the new
driver needs evaluator changes, or if two A/B revisions do not improve a
regression. Do not demand successful 4,097 evaluation from the active output
(`docs/dev/codex/TEMPLATE.md:96-100`,
`docs/dev/codex/brief-437-native-stack-budget-design.md:526-532`).

**Hard rules.** Scope only; no git/network/oracle/new `unsafe`; bounded runs;
no deep pre-fix reproduction (`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** Paste exact derivative, workspace, release safety, and A/B
outputs; name both removed recursive edges and report the final in-band tag
without calling it contract success (`docs/dev/codex/TEMPLATE.md:107-123`).

#### Draft 5 — iterative exact paths, selectors, and structural edit plans

**Goal.** Provide shared, bounded-native-stack primitives for exact lookup,
reverse path rebuilding, selector-shaped selection, `ReplacePart` planning, and
sequence edit plans, then route every named path consumer through them without
changing values, messages, rollback, sharing, or suspension
(`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-201`,
`crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/apply.rs:112-223`,
`crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:225-518`).

**Context pointers.** Read `AGENTS.md`, this report sections 2/5, selector
diagnostics, replace plans, definition update rollback, targeted-map request
tests, and sequence edit-plan tests before touching code
(`crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/diagnostics.rs:1-175`,
`crates/rum-runtime/src/runtime/definitions/part_assignment.rs:123-215`,
`crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs:272-360`).

**Scope.** Files allowed:

- `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs` and
  `selectors/{apply,diagnostics,extract,parse,spans}.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs` and
  `replace/{plan,guarded_plan,replace_at,replace_part,suspended_state}.rs` plus
  their existing resume-frame siblings
- `crates/rum-runtime/src/runtime/definitions/part_assignment.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_updates.rs` and its
  resume frames
- `crates/rum-stdlib/src/functional_ops/targeted_map.rs`
- `crates/rum-stdlib/src/associations/mutation.rs`
- `crates/rum-stdlib/src/structure_ops/sequence/{take_drop,insert,delete,edit_plan}.rs`
- directly relevant runtime/stdlib tests, the depth-certificate file, and
  `scripts/core-language-microbench.sh`

No changes to `Expr`, parser syntax, level-spec semantics, unrelated
association transforms, or public mutation contracts
(`docs/dev/codex/TEMPLATE.md:55-69`).

**Deliverable.** Stage the patch internally as (a) exact lookup/reverse rebuild,
(b) selector/replace-plan walkers, and (c) sequence edit consumers, running
focused tests after each stage. Preserve exact-path normalization and all
diagnostic paths. Add stable 4,097 certificates for one representative of each
primitive, including deepest `MapAt`, deepest `ReplacePart`, scalar `Part`, a
generic-COW part update, multi-spec `Take`, and deep `Insert`/`Delete`. Add
small exact A/B labels for path lookup, rebuild, targeted mapping, and edit
plans (`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:44-110`,
`crates/rum-stdlib/src/functional_ops/targeted_map.rs:295-456`,
`crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs:31-204`).

**rum / Rust pitfalls (read twice).** Rebuild in reverse path order and preserve
heads at component zero, negative-index normalization, association keys versus
values, multi-selection output shape, duplicate/overlapping path precedence,
and atomic mutation rollback. Do not replace the proven direct positional
assignment route with COW rebuilding. Candidate paths and `Expr` construction
are hot; prefer borrowed descent plus `Arc` clones
(`crates/rum-runtime/src/runtime/definitions/part_assignment.rs:157-215`,
`docs/dev/codex/TEMPLATE.md:90-98`).

**Verification / gate.** After each internal stage run bounded focused crate
tests; at the end run `cargo fmt --check`, `cargo test -p rum-runtime`,
`cargo test -p rum-stdlib`, and `cargo test --workspace`, then only the new
release certificates. Best-of-2 A/B all new target labels with
`association-part-positional-read-width32000`, `part-list`,
`replacepart-wide-rule-list`, `replacepart-guarded-hit`, and
`replacepart-guarded-decline-fallback`
(`scripts/core-language-microbench.sh:450-478`,
`scripts/core-language-microbench.sh:599-613`).

**Stop-loss.** Stop if a stage requires changing diagnostics/precedence, if
rollback or suspension parity fails twice, if Scope proves too broad for one
reviewable patch, or if two A/B revisions fail. In the “too broad” case report
the last green stage and propose an exact split; do not partially route
consumers to inconsistent primitives (`docs/code-organization-guidelines.md:64-89`,
`docs/dev/codex/TEMPLATE.md:96-100`).

**Hard rules.** Scope only; no git/network/oracle/unsafe; no destructive
mutation probes; bounded commands and no unconditional diagnostics
(`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** Give a consumer-to-primitive table, removed recursive edges,
verbatim focused/workspace/release output, A/B rows, and explicit 4,097 results
for each representative (`docs/dev/codex/TEMPLATE.md:107-123`).

#### Draft 6 — iterative `DeleteCases` and substitution engines

**Goal.** Remove depth-proportional native recursion from `DeleteCases`, slot
substitution, lexical substitution, and rule-binding substitution while
preserving every consumer's evaluation, scoping, sharing, splicing,
alpha-renaming, and suspension semantics
(`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:55-477`,
`crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:3-34`,
`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:366-427`,
`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:47-160`).

**Context pointers.** Read `AGENTS.md`, this report's substitution inventory,
all in-file sharing tests, definition application/guard callers, scoping tests,
string-rule tests, and exception-handler tests
(`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:607-732`,
`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:533-650`,
`crates/rum-runtime/src/runtime/definitions/application/down_values.rs:152-358`).

**Scope.** Files allowed:

- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs` and
  `cases/delete/traversal.rs`
- `crates/rum-runtime/src/runtime/substitution/{slot_substitution,lexical_substitution,binding_substitution,lexical_presence}.rs`
- existing caller files only if a signature adaptation is mechanically needed:
  definition `down_values.rs`, `up_values.rs`, `sub_values.rs`, `rhs_guards.rs`;
  scoping `callable.rs`, lexical `with.rs`/`module.rs`, `packages.rs`; rewrite
  `cases/collect.rs`, `replace/list/matching.rs`; control `exceptions.rs`;
  stdlib string `patterns/search/rules.rs`
- directly relevant tests, the depth-certificate file, and
  `scripts/core-language-microbench.sh`

Do not change match results, definition precedence, lexical shadow rules,
package identity, or callback scheduling
(`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:386-427`).

**Deliverable.** Add explicit visit/finish stacks with changed propagation for
each engine; preserve the empty-binding clone fast path and unchanged sibling
pointer reuse. Make `DeleteCases` use a matching explicit state. Add stable
4,097 certificates for unique deepest deletion, slot body substitution,
lexical body substitution, and rule-RHS binding substitution; add active-body
in-band controls. Add exact small labels for function/lexical/rule substitution
while retaining existing `deletecases-*` rows
(`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:88-108`,
`scripts/core-language-microbench.sh:584-593`).

**rum / Rust pitfalls (read twice).** Preserve special handling for `Pattern`,
assignment-like constructs, `Condition`, nested `With`/`Module`/`Block`, and
explicit `Function` parameter shadowing. Preserve alpha-renames and only splice
a `Sequence` inserted by a directly bound pattern symbol. Suspension-capable
callers must still evaluate exactly once; do not duplicate side effects
(`crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:396-425`,
`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:280-326`,
`docs/dev/codex/TEMPLATE.md:90-95`).

**Verification / gate.** Bounded `cargo fmt --check`, runtime and stdlib tests,
then workspace and release certificates. Best-of-2 A/B new target labels plus
`deletecases-wide-no-match`, `deletecases-wide-association-no-match`,
`module-local`, `with-local`, every down/up/subvalue hit/miss row, and
`replaceall-simple` (`scripts/core-language-microbench.sh:480-558`,
`scripts/core-language-microbench.sh:584-593`).

**Stop-loss.** Stop if pointer-sharing assertions regress, if a shadowing or
suspension failure repeats twice, if signatures ripple outside Scope, or if two
A/B revisions fail. Never weaken alpha-renaming/splicing contracts or remove
the empty-binding fast path (`docs/dev/codex/TEMPLATE.md:96-100`).

**Hard rules.** Scope only; no git/network/oracle/new `unsafe`; bounded runs;
no default-path profiling cost (`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** Provide the complete consumer list, pointer-sharing results,
verbatim tests/release certificates/A-B rows, and separate stable-success from
active-`reslim` results (`docs/dev/codex/TEMPLATE.md:107-123`).

#### Draft 7 — budget-gated structural, head, and tensor rebuilders

**Goal.** Using only non-crashing shallow stack measurements and static release
frame accounting, select the remaining structural/head/tensor rebuilders that
cannot meet physical height 4,097 on 2 MiB, then convert exactly those selected
sites to bounded-front explicit-stack algorithms. `Flatten` remains unchanged
unless its level-group path independently fails the gate
(`TODO.md:1070-1105`,
`docs/dev/codex/brief-437-native-stack-budget-design.md:244-272`).

**Context pointers.** Read `AGENTS.md`, this report sections 2/5, brief 437's
measurement method, all candidate implementations, and their focused tests.
This draft must be rewritten with the selected functions before dispatch; if
the gate selects none, it ends as a diagnostic report with no source edit
(`docs/dev/codex/TEMPLATE.md:17-37`).

**Scope.** Candidate files, to be narrowed before dispatch:

- `crates/rum-stdlib/src/structure_ops/{reordering,join,flatten,list_plumbing}.rs`
- `crates/rum-runtime/src/runtime/arithmetic/parts.rs`
- `crates/rum-stdlib/src/functional_ops/callable/head_ops.rs`
- `crates/rum-stdlib/src/functional_ops/targeted_map.rs`
- `crates/rum-stdlib/src/functional_ops/arrays/{mod,tensor,filter,spec}.rs`
- their directly relevant tests, `iterative_traversal_depth.rs`, and
  `scripts/core-language-microbench.sh`

No evaluator, `Expr`, unrelated arithmetic, generator, or public semantic
change (`docs/dev/codex/TEMPLATE.md:55-69`).

**Deliverable.** First produce for each candidate a <=16-level stack-marker
slope, release prologue/call-edge accounting, and projected 4,097 consumption;
run no near-limit expression. Freeze the selected list. For each selected site,
add a bounded front and explicit task/result builder preserving order, shape,
heads, callbacks and messages. Add stable 4,097 certificates for converted
sites and active-result in-band cases for `Total`, arithmetic tensor results,
and active operator variants. Add one exact small A/B label per selected
builtin (`crates/rum-stdlib/src/structure_ops/reordering.rs:157-318`,
`crates/rum-runtime/src/runtime/arithmetic/parts.rs:54-137`,
`crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:593-733`).

**rum / Rust pitfalls (read twice).** “Unbounded” alone is not selection
evidence. Preserve `Flatten`'s same-head versus level-group semantics,
`Transpose` permutation order, `Join`'s three validation phases, pad generated
element/memory preflight, `Total`'s explicit final evaluation, `Operate` head
component zero, and tensor callback suspension. Convert a selected family’s
local recursive memory estimator too when its memory-constrained gate reaches
that helper (`crates/rum-stdlib/src/structure_ops/list_plumbing.rs:173-187`,
`docs/dev/codex/TEMPLATE.md:90-100`).

**Verification / gate.** Record the shallow/static selection table first. Then
bounded focused tests, `cargo fmt --check`, relevant crate tests, workspace,
and post-conversion release certificates. A/B best-of-2 each selected exact
label with `literal-integer`, `list-literal-small`, `sameq-list`, and its
same-tier family neighbors; `Total`/active cases pass safety by recoverable
error, not successful 4,097 evaluation
(`scripts/core-language-microbench.sh:463-478`,
`docs/dev/codex/brief-437-native-stack-budget-design.md:526-532`).

**Stop-loss.** Stop before editing if source-to-symbol mapping or shallow slope
cannot identify a selected frame. After editing, stop on a repeated shape/order
failure or two failed A/B revisions. Do not run a deep pre-fix probe, convert a
target-safe site “for completeness,” or broaden into the evaluator trampoline
(`TODO.md:1070-1077`, `docs/dev/codex/TEMPLATE.md:96-100`).

**Hard rules.** Narrow Scope before dispatch; no git/network/oracle/unsafe;
bounded shallow measurements only; no files outside the narrowed list
(`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** Paste the selection table, narrowed Scope, removed recursive
edges, verbatim gates/certificates/A-B rows, and a per-site safety-versus-contract
result; explicitly list target-safe sites left unchanged
(`docs/dev/codex/TEMPLATE.md:102-123`).

#### Draft 8 — budget-gated association and schema rebuilders

**Goal.** Non-destructively determine which of `DeleteMissing`, `MergeNested`,
`AssociationKeyDeflatten`, and shared `ConformSchema`/`SchemaErrors` cannot meet physical height 4,097 on
2 MiB, then make exactly the selected walkers bounded in native stack while
preserving association order, schema paths/errors, callback count, and
suspension (`crates/rum-stdlib/src/associations/missing.rs:331-1003`,
`crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-218`,
`crates/rum-stdlib/src/associations/key_reshape.rs:340-415`,
`crates/rum-stdlib/src/schema.rs:177-300`).

**Context pointers.** Read `AGENTS.md`, this report sections 2/5, brief 437's
non-crashing method, association/schema tests, and session-request resume tests.
Rewrite this draft with the measured selected sites before dispatch; the
existing `Merge` ceiling is explicitly masked by construction
(`TODO.md:1112-1115`, `docs/dev/codex/TEMPLATE.md:17-37`).

**Scope.** Candidate files, to be narrowed before dispatch:

- `crates/rum-stdlib/src/associations/missing.rs`
- `crates/rum-stdlib/src/associations/resumable/aggregate.rs` and its existing
  resume-frame module
- `crates/rum-stdlib/src/associations/key_reshape.rs`
- `crates/rum-stdlib/src/schema.rs`
- directly relevant association/schema/session-request tests,
  `iterative_traversal_depth.rs`, and `scripts/core-language-microbench.sh`

Do not change association representation, schema contract, matcher semantics,
or evaluator limits (`docs/dev/codex/TEMPLATE.md:55-69`).

**Deliverable.** Produce <=16-level slopes and static frame projections without
near-limit probes, freeze the selected list, and convert selected recursion to
owned visit/finish state. Preserve `DeleteMissing` target/recursive modes,
delayed-wrapper accounting, exact association entry order, `MergeNested`
group/callback order, key-path insertion/render order for
`AssociationKeyDeflatten`, `ConformSchema` defaults/path errors, and
`SchemaErrors` caps. Convert `DeleteMissing`'s local expression-memory estimator if selected.
Add stable 4,097 certificates and active callback in-band controls; add one
small exact A/B label per selected surface
(`crates/rum-stdlib/src/associations/missing.rs:105-228`,
`crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-218`,
`crates/rum-stdlib/src/associations/key_reshape.rs:340-415`,
`crates/rum-stdlib/src/schema.rs:233-300`).

**rum / Rust pitfalls (read twice).** Association keys are not ordinary values;
do not traverse or reorder them accidentally. A suspended predicate/combiner
must resume exactly once. `SchemaErrors` may discard the conformed value but
must retain identical error paths and cap behavior. Construction must be
bracketed separately from the consumer in shallow measurement
(`TODO.md:1112-1115`, `docs/dev/codex/TEMPLATE.md:90-100`).

**Verification / gate.** Paste the shallow/static selection evidence, then run
bounded focused tests, `cargo fmt --check`, `cargo test -p rum-stdlib`, and
`cargo test --workspace`; run release certificates only after shallow parity is
green. Best-of-2 A/B each selected exact label plus association positional-read
controls and same-tier callback/schema neighbors
(`scripts/core-language-microbench.sh:450-454`,
`docs/dev/codex/TEMPLATE.md:102-123`).

**Stop-loss.** Stop before editing if construction cannot be separated from
consumer frames; after editing stop on a repeated callback/order/suspension
failure or two failed A/B revisions. Do not infer selection from recursion or
run a deep pre-fix overflow probe (`TODO.md:1070-1077`,
`docs/dev/codex/TEMPLATE.md:96-100`).

**Hard rules.** Narrow Scope before dispatch; no git/network/oracle/new
`unsafe`; bounded shallow measurements only; no out-of-scope docs or generated
artifacts (`docs/dev/codex/TEMPLATE.md:125-159`).

**Report format.** Paste the selection table, narrowed scope, semantic and
suspension outputs, release certificate results, and exact A/B rows. For each
surface state “stable 4,097 success,” “active recoverable ceiling,” or
“measured target-safe and unchanged” (`docs/dev/codex/TEMPLATE.md:102-123`).

---

## Director gate — 2026-09-04

**Scope**: clean, this file only, 1047 insertions, no source change.

**The dispatch exited 1 on the upstream content filter, after the report was
written.** The error appears twice below the last appended line, and the report
is complete: all six deliverables, eight drafts, the last closing on its own
`Report format` section. This is a post-write cutoff, not a truncation — worth
recording, because the standing rule is to suspect truncation when a report
cites evidence that is not in the file, and here nothing is missing.

**Hard rule respected.** The log contains `Do[x = {x}, {i, 100000}]`,
`{i, 80000}` and `{i, 20000}`, but every one is a quotation from
`brief-422`'s report, read as context; no `exec` line runs any of them. No deep
probe was executed.

### The finding that reorganizes the whole line

**The process-kill mode is one shared defect, not eight.** I verified it in
source: `call_stability_uncached_with_bloom` walks the head, then each argument,
through `cached_expr_stability_with_bloom`, which re-enters it for a call —
mutual recursion over the entire tree
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33,82-95`). The string
`evaluation_depth` does not appear anywhere in that file, so
`poll_native_stack_limit` never polls for the whole descent
(`crates/rum-runtime/src/runtime/eval_core/mod.rs:34-53`).

Any traversal that rebuilds a deep result hands the evaluator a tree with fresh
`Arc` identities at every level. Every cache key misses, and the stability walk
descends the whole thing natively. That is why `ReplaceAll` on a match, `Apply`
at a descending spec, and `Replace` all die the same way and at similar heights,
and why `ReplaceAll` with a rule that never matches survives — the traversal
returns `Expr::call_shared` with the original pointers, so the root's existing
proof still hits.

So brief-425's conversions were not wrong. The traversals were converted; the
shared post-result check downstream of them was not, and the certificate panel
never reached it.

### Claims verified

**`{-4097, 0}` is rum's intended semantics, not a defect** — its verdict holds,
and my own first reading of the evidence was too quick. `traversal.rs:144` pins
`Level[f[g[1], 2], {-3, 0}]` to `List[f[g[1], 2]]`, root-only. Live, on a
*different* fixture, `Level[{{{1}}}, {-3, 0}]` returns `{}` while
`{-4, 0}` returns the root. Those are not in conflict: the negative bound is
relative to that value's own depth, so `-3` names level 0 in a depth-3 value and
level 1 in a depth-4 one, and a maximum of 0 then admits nothing. The rule is
consistent and test-pinned; the certificates were written against correct
semantics with a selector that happens to reach one level.

**The certificate rule is adopted as written** — unique deepest target that
must propagate `changed`, an assertion on the exact rebuilt value rather than a
summary, and a descending selector whose semantics are pinned separately on a
shallow fixture. It rejects all three defective certificates.

**The three-way repair taxonomy is adopted**: a local poll is safety only and
deliberately refuses 4,097; a bounded front plus explicit stacks is safety plus
the stable-result contract; only a whole-evaluator task stack lifts the
active-result ceiling, which brief-437 already priced out of local repair.

### Verdict and sequencing

Accepted. Step 1 is dispatched first and alone, because it is the shared root
cause: making the stability walk iterative removes the process-kill mode for
every deep rebuilder in the inventory at once, and no per-site conversion is
worth gating until it lands. It is also the single hottest path in the system —
stability is consulted on every fixed-point pass — so it carries the mandatory
A/B and gets its own brief rather than being bundled.

I keep the memory-proxy `Expr` walk in that brief, against the instinct to
narrow. It is the same recursion in the same position, it runs *before*
stability whenever a memory constraint is armed, and the sandbox profile is
precisely the configuration that arms one. Excluding it would leave the defect
live exactly where it matters most, which is the shape brief-388 was gated on.

Steps 2-8 stay drafts. They are per-site native conversions and several are
large; each will be re-derived against the post-step-1 measurements rather than
dispatched from a projection.

One thing to carry into every later step: the report is careful to mark which
of its ceilings are observed and which are projected, and several projections
rest on my own B4 brackets, which were measured under the old 6 MiB budget.
Re-probe before briefing any of them.
