# brief-469 — convert the association `MapIndexed` traversal

## Goal

Make `MapIndexed`'s traversal bounded by heap work rather than by native stack
depth, so it can no longer terminate the process, and flip the
`MapIndexed (association axis)` registry row to `BoundedHeap`.

## Two things that look like the answer here and are not

**1. `Scan` sits beside this walker in the same file and was *guarded*, not
converted. Do not copy that.** brief-461 guarded `Scan`'s three edges because
`Scan` fits the 1 MiB operation boundary at contract height — measured, 3/3.
`MapIndexed` does **not**: under the same halved-stack test it is a stable kill
3/3. A guard refuses at the budget and cannot make work fit inside it, so
guarding `MapIndexed` would turn a contract-meeting site into a
contract-failing one. **This is a conversion.**

**2. Converting this walker does not close the generic-shape shortfall.**
`map_indexed_parts_recursive` serves both the association shape and the standard
call shape, so your conversion bounds both walkers — but the generic
`MapIndexed[ff, _, {0, Infinity}]` shortfall (2,944 physical, **fail-closed**,
tracked as brief-439 draft 3) is **downstream**, on the post-result stability
path, not in this walker. brief-457 established that only an evaluator-side task
stack lifts it. So: convert the walker, and **flip only the
`MapIndexed (association axis)` row**. Do not touch the generic item's tracking
and do not claim it is fixed.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3.

| axis | ok <= | fail >= | direction above |
|---|---|---|---|
| depth, association shape | 7,344 | 7,376 | **terminates the process** |

It clears the ruled 4,097 by 1.79x and fails open above that.

**Width is clean**: director-measured 3/3 at 1,000 / 4,000 / 16,000 / 64,000
entries. This brief is depth-only — keep the entry loops loops; do not introduce
a per-entry recursion where none exists.

A ceiling is not deterministic; the same height flips run to run inside a band
tens of levels wide. No test may depend on one.

## The shape — the simple one, and why it is safe

Use the bounded front plus cold iterative tail that `Replace`, `DeleteCases`,
`MergeNested`, `DeleteMissing` and the key-reshape walks all use. **The cold
tail may redo the traversal after the front exhausts.**

That is safe here for the same reason it was safe in brief-468, and it is
checkable from the signature: `map_indexed_parts_recursive(&Expr, &Expr,
TraversalConfig, usize, &[isize]) -> (Expr, usize)`
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:310`) **takes no
`Session`**. It *constructs* `operator[value, position]` expressions; it does not
evaluate them — the evaluator does that afterwards on the returned result. A
function that cannot reach the evaluator cannot run user code, so a redo has no
observable effect beyond time. **Do not build `DeleteMissing`'s
carried-continuation handoff.**

## The walk

- Entry `map_indexed_parts_with_config` (`:302`), which calls the worker at
  `:307`.
- `map_indexed_parts_recursive` (`:310`), recursive at `:337` — association
  values.
- `map_indexed_parts_on_standard_call` (`:370`), recursive at `:383` — the head
  — and `:393` — arguments.

Three edges, the same shape `Scan` has in this file. The recursion inventory
from brief-460 is your completeness check.

## What must not change

`MapIndexed` returns a pair `(Expr, usize)` — the rebuilt value and a depth —
and both are used by callers. Preserve: the position vectors handed to the
operator, visit and construction order, the level specification, head handling
under `include_heads`, and the returned depth.

**`Scan`'s guards in this same file are not yours to touch.** brief-461 installed
`NativeStackDepth` on `collect_scan_parts_recursive`'s three edges; leave them
exactly as they are. If your conversion requires changing a shared helper that
`Scan` also uses, say so in the report and explain how `Scan`'s behaviour and
its guard are preserved.

## Deliverable — certificates and the falsifier

**Certificates**, release-only, on declared 2 MiB threads, three-part rule:

- The association shape at raw physical height 4,097, with a **unique deepest
  target** whose mapped result must propagate, asserting the **exact** rebuilt
  value — not `Depth`, not `Length`.
- A second certificate at the same height with `include_heads` engaged, so the
  head edge at `:383` is exercised by a certificate and not only by the
  falsifier.
- A three-level **shallow companion** pinning the position vectors and the level
  specification, which is also the falsifier if a deep case aborts the binary.

**Forced-front falsifier**, in-module, no cargo feature and no production
switch — the shape briefs 463 and 468 settled on. A private `*_with_front_depth`
entry, called from `mod tests`, on one shallow **branched** fixture with more
than one entry per level and a distinct witness per branch:

- front `0` returns `Exhausted`;
- front `16` returns `Complete`;
- **tail == front == exact expected**.

The fixture must exercise all three edges — association values, the head, and
arguments — or say which edge it cannot reach and why.

## Registry and inventory

Flip `MapIndexed (association axis)` to `Mechanism::BoundedHeap`,
`failure_direction: Some(FailureDirection::Unmeasured)`, `owed_by: None`,
matching the `MergeNested` row
(`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:389-396`).
Update inventory rows for every edge you convert.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs` and new sibling
    modules under `crates/rum-stdlib/src/functional_ops/apply_map/` if the file
    needs splitting by responsibility (17 516 bytes today, so probably not);
  - `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs` if the entry
    signature changes;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - a new focused certificate file plus its wiring in
    `crates/rum-stdlib/src/tests.rs`.
- Files you MUST NOT touch: `Scan`'s guard edges, the guard substrate,
  `kernel_api.rs`, `associations/missing*`, `associations/key_reshape*`, the
  remaining conversion site (`AssociateTo`/`KeyDropFrom` Part target),
  `metadata.rs`, `Expr`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other
  `docs/**`, `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended change is that this walk stops growing the
  native stack. Any change to position vectors, visit or construction order, the
  returned depth, `include_heads` behaviour, `Scan`'s or `Map`'s behaviour, or
  any other builtin is a **stop**.

## rum / Rust pitfalls (read twice)

1. **`Map` and `Scan` share this file and this machinery.** Both must be
   allocation-neutral controls in the cost gate; `Map` in particular is hot.
2. **A per-element allocation is a regression even when nothing else changes.**
   The association surface has been caught twice adding one, both times found
   only by counters — an owned `into_iter().filter_map(…).collect()` can be
   satisfied in place and `Vec::with_capacity` plus `push` gives that up.
3. The walker returns a **depth** alongside the value; a tail that recomputes it
   differently is a behaviour change even if the value matches.
4. Assert exact expressions, never `Depth` or `Length`.
5. Logical wrappers are not physical height; the association fixture is two
   physical levels per wrapper, so raw 4,097 is `N=2048`.
6. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the release certificates and the forced-front falsifier.

Then the cost gate, **counters first**: allocation counts
(`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base against candidate, both
arms built at the same filesystem path, for a shallow `MapIndexed` probe, a
**`Map` probe**, a **`Scan` probe**, and one untouched control — sensitivity
calibrated and the same-binary spread stated. Then
`perf stat -e instructions:u`, minimum of the tight clean cluster, migrated runs
discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling** before
  your conversion is in place; afterwards the 4,097 certificates are the
  intended runs.
- **Stop-loss.** Stop and report if: a certificate does not complete with its
  exact value; the falsifier does not exercise its tail; allocation counts move
  on the `Map` or `Scan` control; or the same step fails twice for the same
  reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The front constant, the tail's work representation, and how the returned
   depth is computed in the tail.
3. How `Scan`'s guards and `Map`'s behaviour are preserved.
4. Verification: every command with verbatim output, and the ZeroMQ statement.
5. The certificates and the falsifier, including which edges the falsifier
   fixture reaches.
6. Registry and inventory row changes.
7. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
8. Stop-loss and open questions.
