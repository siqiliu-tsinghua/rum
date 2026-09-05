# brief-470 — convert the exact-path walks behind held association mutation

## Goal

Make the exact-path collect, read and replace walks bounded by heap work rather
than by native stack depth, so the `AssociateTo`/`KeyDropFrom` `Part` target can
no longer terminate the process, and flip its registry row to `BoundedHeap`.

**This is the last conversion in this line.** After it, every association
surface row is `NoProportionalDescent`, `BoundedHeap` or `GuardedNative` except
the four sites that are short of the contract and tracked separately.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3.

| site | ok <= | fail >= | direction above |
|---|---|---|---|
| `AssociateTo` / `KeyDropFrom` on a `Part` target | 8,048 | 8,080 | **terminates the process** |

It clears the ruled 4,097 by 1.96x and fails open above it. **Width is clean** —
director-measured 3/3 at 1,000 through 64,000 entries.

**The axis is path length, not value depth.** `exact_expr_at_path` recurses once
per path component, and a key selector expands to more than one raw component
(`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:112-126`),
so the measured physical height corresponds to a path, not to a nesting count.
Build fixtures accordingly and say in your report what the relationship is —
getting this wrong produces a fixture that exercises the value walk instead.

A ceiling is not deterministic; the same height flips run to run inside a band
tens of levels wide.

## The three walks

All in `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs`:

- `exact_expr_at_path` (`:35`), self-recursive at `:41`;
- `collect_exact_paths_for_position_spec` (`:88`), self-recursive at `:103`, and
  reached from `:8` and `:26`;
- `replace_exact_path` (`:173`), self-recursive at `:179` — the head — and
  `:194` — a child.

Enumerate any edge I have missed; brief-460's recursion inventory is your
completeness check, and this file is inside its scanned roots.

Also check `crates/rum-stdlib/src/associations/mutation.rs:560-692`, the held
`Part` mutation path that plans, reads and rebuilds through these helpers, for
recursion of its own. If it has none, say so — that is a useful negative.

## The shape — the simple one

Bounded front plus cold iterative tail, as `Replace`, `DeleteCases`,
`MergeNested`, `DeleteMissing`, the key-reshape walks and `MapIndexed` all use.
**The cold tail may redo the traversal.**

Safe here for the reason that has held for every conversion since brief-468, and
checkable from the signatures: `exact_expr_at_path(&Expr, &[isize])`,
`collect_exact_paths_for_position_spec(&Expr, …)` and
`replace_exact_path(Expr, &[isize], Expr)` **take no `Session`**. None can reach
the evaluator, so a redo has no observable effect beyond time. **Do not build
`DeleteMissing`'s carried-continuation handoff.**

## Shared machinery — the thing most likely to go wrong here

These helpers are **not private to association mutation**. They are consumed
throughout `crates/rum-stdlib/src/functional_ops/targeted_map.rs` — `:323`,
`:344`, `:360`, `:378`, `:394`, `:416`, `:446`, `:664` — by `MapAt`, `ReplaceAt`,
`Insert`, `Delete` and `Extract`. brief-450 measured six such consumers as
already clearing raw 4,097, so **they are working code and your conversion must
leave them exactly as they behave today**.

Concretely: those operations, plus `Part` and `ReplacePart`, are required
controls in both the behavioural gate and the cost gate. A regression in any of
them is a stop, not a trade.

## Deliverable — certificates and the falsifier

**Certificates**, release-only, on declared 2 MiB threads, three-part rule:

- `AssociateTo` on a deep `Part` target at raw physical height 4,097 — which it
  clears today, so a completing certificate is achievable — with a **unique
  deepest target**, asserting the **exact** stored value read back.
- The same for `KeyDropFrom`, since brief-458's inventory treats them as one
  site and they are not interchangeable: `KeyDropFrom` requires `Key[...]`
  selectors on an association `Part` target where `AssociateTo` also accepts a
  bare string. Pin both spellings in the shallow companion.
- A three-level **shallow companion** pinning selector spelling, multi-selection
  order, and the stored-versus-returned value contract.

**Forced-front falsifier**, in-module, no cargo feature and no production
switch — the shape briefs 463, 468 and 469 settled on. A private
`*_with_front_depth` per walk, called from `mod tests`, on a **branched** fixture
with a distinct witness per branch:

- front `0` returns `Exhausted`;
- front `16` returns `Complete`;
- **tail == front == exact expected**.

One per walk. `replace_exact_path` has two edges — the head and a child — so its
fixture must reach both, or say which it cannot and why.

## Registry and inventory

Flip the `AssociateTo`/`KeyDropFrom` `Part` target row to
`Mechanism::BoundedHeap`, `failure_direction: Some(FailureDirection::Unmeasured)`,
`owed_by: None`. Update inventory rows for every edge you convert.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs`
    and a new sibling module under
    `crates/rum-runtime/src/runtime/kernel_data/structure_data/` if it needs
    splitting (6 215 bytes today, so probably not);
  - `crates/rum-stdlib/src/associations/mutation.rs`, **only** if it has its own
    depth recursion to convert — say so first;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - a new focused certificate file plus its wiring in
    `crates/rum-stdlib/src/tests.rs`.
- Files you MUST NOT touch: `functional_ops/targeted_map.rs` and every other
  consumer, the guard substrate, `kernel_api.rs`, the already-converted sites,
  `metadata.rs`, `Expr`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other
  `docs/**`, `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended change is that these three walks stop growing
  the native stack. Any change to selector semantics, multi-selection order,
  head selection, the key-to-rule-RHS mapping, diagnostics, the
  stored-versus-returned value contract, or any consumer's behaviour is a
  **stop**.

## rum / Rust pitfalls (read twice)

1. **Path length is the axis.** A fixture built around nesting depth measures a
   different walk. Calibrate the fixture at a small size and show it reaches the
   intended function before scaling it.
2. **The consumers are working code.** `MapAt`, `ReplaceAt`, `Insert`, `Delete`,
   `Extract`, `Part` and `ReplacePart` all go through these helpers and all
   clear the contract today.
3. **A per-element allocation is a regression even when nothing else changes.**
   The association surface has been caught twice adding one, both times found
   only by counters.
4. `replace_exact_path` takes an owned `Expr` and returns one; a tail that
   clones where the recursion moved is an allocation regression even if the
   value matches.
5. Assert exact expressions, never `Depth` or `Length`.
6. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the release certificates and the forced-front falsifiers.

Then the cost gate, **counters first**: allocation counts
(`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base against candidate, both
arms built at the same filesystem path, for an `AssociateTo` `Part`-target
probe, a **`ReplacePart` probe**, a **`MapAt` probe**, and one untouched control
— sensitivity calibrated and the same-binary spread stated. Then
`perf stat -e instructions:u`, minimum of the tight clean cluster, migrated runs
discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`, and this
  conversion is in `rum-runtime`.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling** before
  your conversion is in place; afterwards the 4,097 certificates are the
  intended runs.
- **Stop-loss.** Stop and report if: a certificate does not complete with its
  exact value; a falsifier does not exercise its tail; allocation counts move on
  the `ReplacePart` or `MapAt` control; a consumer's behaviour changes; or the
  same step fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. Each walk: its front constant and its tail's work representation; and whether
   `mutation.rs` had recursion of its own.
3. The relationship between path length and physical height in your fixtures.
4. How the shared consumers are preserved.
5. Verification: every command with verbatim output, and the ZeroMQ statement.
6. The certificates and the falsifiers, including which edges each reaches.
7. Registry and inventory row changes.
8. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
9. Stop-loss and open questions.
