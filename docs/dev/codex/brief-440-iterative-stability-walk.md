# brief-440 — make the post-result structural checks iterative

**Shape: implementation.** One variable: the two shared structural walks that
run over a freshly built result. No traversal is converted here.

## Goal

A traversal that rebuilds a deep result hands the evaluator a tree with fresh
`Arc` identities at every level. Every stability cache key misses, and
`call_stability_uncached_with_bloom` walks the whole tree natively without
crossing `Session::evaluate` — so `poll_native_stack_limit` never polls and the
process dies. That single walk is why `ReplaceAll` on a match, `Apply` at a
descending level spec, and `Replace` at all levels all end the process at
similar heights, while the same operations with a rule that never matches
survive.

When you are done, both shared post-result walks are bounded, a deep rebuilt
result cannot end the process through them, and the three certificates that
never covered a rebuild do cover one.

This does not make every rebuilding traversal reach physical height 4,097. Some
sites have their own unbounded walker (`Replace`, `MapIndexed`, `D`), and a
result that is *active* at every level re-enters the evaluator per level, whose
ceiling brief-437 priced out of local repair. Those are later briefs.

## Context pointers

Read `AGENTS.md`, then `docs/dev/codex/brief-439-deep-rebuild-class-design.md`
(report and my gate note) and `docs/dev/codex/brief-437-native-stack-budget-design.md`
(the guard, the frame cost, and why the byte poll cannot see this descent).

- `crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33,82-95` — the
  mutual recursion. `evaluation_depth` appears nowhere in this file.
- `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:204-349` — the
  cached entry point, the pointer key, the dirty-overlap test, and stable-only
  insertion.
- `crates/rum-runtime/src/runtime/eval_core/mod.rs:34-53` — the strided poll
  that this descent bypasses.
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346` —
  `expr_memory_proxy_bytes`, the second recursive walk.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-161`
  — where it runs, before stability, whenever a memory constraint is armed.
- The four existing instances of the pattern you will follow:
  `crates/rum-runtime/src/runtime/tree_walkers/level.rs:76-190`,
  `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-60`,
  `crates/rum-runtime/src/runtime/resolution/match_normalization.rs:5-80`.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:1-140` — the
  harness and the twelve certificates.
- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` — existing cache
  assertion helpers.

## Deliverable

### 1. Bound the stability walk

Convert the `cached_expr_stability_with_bloom` / `call_stability_uncached_with_bloom`
mutual recursion to a bounded recursive front plus an explicit owned work stack,
following the four precedents above. Use the same recursive-front depth of 16
those use, and give the constant a name in this module rather than reusing
theirs.

**The invariants are the whole difficulty. Every one of these must survive, and
your report must say how you verified each:**

- **Pointer-key cache lookup and stable-only insertion.** The iterative walk
  must consult and populate exactly the entries the recursive one did, keyed the
  same way (`evaluation_cache/stability.rs:219-243,293-329`).
- **The early-unstable short-circuit.** The recursive version returns on the
  first unstable argument and does *not* look at the remaining ones. An
  ordinary postorder machine that visits all children first would union more
  blooms and populate more cache entries. The observable result must not change.
- **Bloom union order.** `bloom.union_assign(arg_bloom)` runs in argument order
  after the head (`eval_core/stability.rs:20,25-30`). Preserve it.
- **The data-constructor table and the head-effect test run before the argument
  walk** (`eval_core/stability.rs:15-23`), so a head with an effect still
  returns without touching arguments.
- **Cache insertion order.** The entry cache has a fixed cap and clears on
  overflow (`evaluation_cache/stability.rs:6-16,301-304,361-367`), so a
  different insertion order can change which entries survive. State whether
  your order matches and, if it cannot, what you tested to show the difference
  is not observable.

### 2. Bound the memory-proxy walk

`expr_memory_proxy_bytes` (`debug_tools/resources/memory.rs:346`) recurses over
call arguments. It runs *before* stability whenever `current_memory_limit` is
armed, and the sandboxed host profile arms one — so leaving it recursive would
keep the defect alive in exactly the configuration that most needs it closed.
Convert it with the same pattern. It is a pure function with no cache, so this
half is mechanically simpler; keep it a separate, separately testable change
inside this brief.

It cannot affect the default-path A/B, because it does not run without an armed
constraint. Say so in the report rather than attributing any movement to it.

### 3. Repair the three certificates

`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs` currently has three
certificates that cannot fail for the reason they exist. Keep the other nine
unchanged. Each replacement must satisfy all three conditions:

1. **A unique deepest target whose transformation is semantically different**,
   so `changed` must propagate through every ancestor and no implementation can
   pass by returning the input pointer.
2. **Assert the exact rebuilt value, not a summary.** `Depth`, `Length` and
   `StringLength` are all correct when the transform never ran. Render with
   `ToString[..., InputForm]`, take the resulting `Atom::String` directly, and
   build the expected string in Rust with `"{".repeat(n)`, the exact centre, and
   `"}".repeat(n)`, so the Rust-side assertion never asks `Display` to walk the
   deep tree itself.
3. **A descending selector whose semantics are pinned separately** on a
   three-level fixture, in a shallow companion test.

The replacements:

| certificate | new body | expected |
|---|---|---|
| `Map` | `ToString[Map[deepMapProbe, x, {-1}], InputForm]` | 4,096 `{`, `deepMapProbe[1]`, 4,096 `}` |
| `Apply` | `ToString[Apply[deepApplyProbe, x, {-2}], InputForm]` | 4,095 `{`, `deepApplyProbe[1]`, 4,095 `}` |
| `ReplaceAll` | `ToString[ReplaceAll[x, 1 -> 2], InputForm]` | 4,096 `{`, `2`, 4,096 `}` |

Shallow companions, on `{{{1}}}`, asserting exact values:
`Map[deepMapProbe, {{{1}}}, {-1}]` is `{{{deepMapProbe[1]}}}`;
`Apply[deepApplyProbe, {{{1}}}, {-2}]` is `{{deepApplyProbe[1]}}`;
`ReplaceAll[{{{1}}}, 1 -> 2]` is `{{{2}}}`.

`{-4097, 0}` selecting level 0 only is rum's intended, test-pinned semantics
(`crates/rum-stdlib/src/tests/traversal.rs:142-149`), not a defect. Do not
change level-spec behaviour.

### 4. The falsifier — a watched failure with no crash

Standing rule: a check nobody has seen fail is not known to work. The pre-fix
behaviour here is a process abort, so a `#[should_panic]` proof is not
available — it would take the test binary with it, and you are forbidden from
producing one.

**The shallow companions are the falsifier.** Point each companion at the
*old* certificate's selector or rule — `{-4097, 0}` reduced to the fixture's
equivalent, and `z -> q` — run the three companions, and paste the failures
verbatim. Then point them at the new selectors and paste them green. That
demonstrates the companions can detect a selector that does not descend and a
rule that never matches, which is exactly what went undetected, and it needs no
deep value at all.

Do **not** run the three deep certificates before the stability change is in
place.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
cargo test -p rum-stdlib --release --lib tests::iterative_traversal_depth -- --ignored
```

The workspace run must not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`. If your sandbox
cannot bind loopback, run it without the flag, report the nine failures as
environmental, and say plainly that those tests did not execute — do not
present a flagged run as a clean gate.

**A/B is mandatory and this is the hottest path in the system**: stability is
consulted on every fixed-point pass. Follow the protocol, in this order:

1. **Four passes in one window**, base/candidate/base/candidate. Report the
   same-binary spread FIRST. A label whose cross-binary movement lies inside its
   own same-binary spread is not evidence.
2. If a label still looks real, build a **layout-control binary** — HEAD with
   one function moved to the end of its `impl` block, identical semantics.
   `#[no_mangle]` does not compile here without an `unsafe` attribute this
   project forbids. **Check the binary hashes**: a failed build followed by a
   `cp` silently makes the control a copy of the candidate.
3. If control adjustment moves the whole panel one way, use **within-run
   quotients** against same-tier untouched neighbours.

Panel: the evaluator and rewrite labels plus `do-count-null`,
`do-iterator-null`, `symbol-ownvalue-read`, `set-ownvalue`,
`list-literal-small`, `downvalue-exact-miss-repeat`,
`downvalue-pattern-miss-repeat`, and the existing `replace-wide-no-match` row.

**Stop-loss.** Stop and report, without further tuning, if: any stability
verdict changes for any existing test; the bloom or cache-entry set differs
observably; a companion cannot be made to fail against the old selector; any
existing release certificate regresses; or a panel label regresses beyond its
own same-binary spread after the layout control and quotient checks. If the
same failure occurs twice, stop.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/eval_core/stability.rs`,
  `crates/rum-runtime/src/runtime/eval_core/mod.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`,
  `crates/rum-runtime/src/runtime/debug_tools/resources/mod.rs`,
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`, and
  `scripts/core-language-microbench.sh` if you add a label.
- **General accept-vs-stop criterion for anything I failed to list**: a module
  root that must change only because a type or function it re-exports changed is
  in scope. Any other non-test source file outside the list is a stop — write it
  in the report instead of making the change.
- Do not touch `Expr`/`Symbol` representation, serde shapes, level-spec
  semantics, generated docs, or `Cargo.lock`.

## rum / Rust pitfalls

- An explicit work stack that owns `Expr` clones where the recursion borrowed
  will allocate on the hot path. The precedents keep the borrow in the bounded
  front and only pay for owned state in the cold fallback; do the same.
- The bounded front returning `Option` is what makes the fallback reachable.
  Make sure an atom cannot enter the recursive worker at all — the brief-433
  gate showed a decomposed-call signature was needed to reach parity, because
  atoms entering a function with an unused depth parameter cost about 8% on
  `matcher-single`.
- `poll_native_stack_limit` stays `#[cold] #[inline(never)]`; do not add a poll
  inside these walks. The fix is to stop consuming native frames, not to add
  instrumentation to a path that has none today.
- Counters before timings. If `shared_expr_alloc` or the stability counters do
  not move the way you expect, do not quote a time.

## Hard rules

- **Do not write, generate, or run any program whose purpose is to exhaust the
  stack**, and do not run the three deep certificates before the stability
  change is in place.
- Do NOT `git commit`, branch, push, or touch git config. Leave the tree
  uncommitted.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`.
- No default-path cost for diagnostics or off-by-default features.

## Report format

Per-file changes; how each of the five stability invariants was preserved and
verified; the companions' red run and green run verbatim; verbatim
`cargo fmt --check`, the three `cargo test` runs, and the release certificates;
the full A/B panel with the same-binary column first and the layout-control and
quotient results where used; and any stop-loss or open question.
