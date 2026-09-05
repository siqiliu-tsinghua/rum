# brief-468 — convert the association key-reshape walks

## Goal

Make `AssociationKeyFlatten` and `AssociationKeyDeflatten` bounded by heap work
rather than by native stack depth, so neither can terminate the process, and
flip both registry rows to `BoundedHeap`.

Two operations, three walks, one brief: they live in one file, share one shape,
and are siblings of each other.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3. Both **terminate the process** above their ceiling.

| operation | ok <= | fail >= | vs the ruled 4,097 |
|---|---|---|---|
| `AssociationKeyFlatten` | 8,292 | 8,334 | clears 2.02x |
| `AssociationKeyDeflatten` | 7,344 | 7,376 | clears 1.79x |

**The width axis is clean.** Director-measured 3/3 at 1,000 / 4,000 / 16,000 /
64,000 entries: both operations pass at every width, as do association
`MapIndexed` and the `AssociateTo` Part target. `DeleteMissing`'s target family
was the only site with a width defect, and that is now closed. **This brief is
depth-only** — but keep the entry loops loops; do not introduce a per-entry
recursion where none exists.

A ceiling is not deterministic; the same height flips run to run inside a band
tens of levels wide. No test may depend on one.

## The shape — and why it is the simple one this time

**Use the bounded front plus cold iterative tail that `Replace`, `DeleteCases`,
`MergeNested` and `DeleteMissing`'s recursive mode all use.** The cold tail may
redo the traversal after the front exhausts.

That is safe here, and the reason is checkable from the signatures: **none of
the three walkers takes a `Session`** —
`insert_path(&mut NestedPathNodes, &[Expr], Expr, bool)`
(`crates/rum-stdlib/src/associations/key_reshape.rs:359`),
`render_nested_path_entries(NestedPathNodes)` (`:392`), and
`flatten_association_entries(Vec<Expr>, Vec<AssociationEntryForm>)` (`:417`).
A function that cannot reach the evaluator cannot run user code, so a redo has
no observable effect beyond time.

**Do not build `DeleteMissing`'s carried-continuation handoff here.** That shape
exists because its target mode evaluates delayed right-hand sides mid-traversal;
these walks cannot. Carrying live cursors would be complexity with nothing to
buy, and this line's standing boundary is that scope is set by the problem.

## The three walks

- **Flatten**: `flatten_association_entries` (`:417`), self-recursive at `:429`,
  entered from `eval_association_key_flatten` (`:126`) at `:141`.
- **Deflatten, insertion phase**: `insert_path` (`:359`), self-recursive at
  `:383` and `:387`, entered from `association_key_deflatten_entries` (`:340`)
  at `:345`.
- **Deflatten, render phase**: `render_nested_path_entries` (`:392`) and
  `render_nested_path_entry` (`:402`) are **mutually** recursive — the entry
  renderer calls the entries renderer on a `Branch`. Convert them as a pair; a
  bounded front on only one half of a mutual cycle bounds nothing.

The recursion inventory from brief-460 is your completeness check: when you are
done, none of these may remain an unconverted recursive edge.

## What must not change

`AssociationKeyFlatten` and `AssociationKeyDeflatten` are documented inverses on
the cases where both apply. Preserve: path order, the overwrite and conflict
behaviour when two flattened paths collide, `Rule` versus `RuleDelayed` heads,
the `delayed` flag `insert_path` carries, and the exact key-list shape Flatten
produces. Existing coverage is in
`crates/rum-stdlib/src/tests/associations/associations_key_reshape.rs`.

## Deliverable — certificates and the falsifier

**Certificates**, release-only, on declared 2 MiB threads, following the
three-part rule:

- Each operation at raw physical height 4,097, with a **unique deepest target**
  whose transformation must propagate, asserting the **exact** result — not
  `Depth`, not `Length`, not a key count.
- A **round-trip** certificate at 4,097: deflatten a deep flat-key association
  and flatten it back, asserting the exact original. That exercises all three
  walks in one value and is the assertion a per-walk test cannot make.
- A three-level **shallow companion** for each, pinning path order and the
  collision behaviour, which is also the falsifier if a deep case ever aborts
  the test binary.

**Forced-front falsifier**, in the in-module shape brief-463 settled on — a
private `*_with_front_depth` entry called directly from a `mod tests` block, no
cargo feature and no production switch. On one shallow **branched** fixture with
a distinct witness per level and more than one entry per level:

- front `0` returns `Exhausted`;
- front `16` returns `Complete`;
- **tail == front == exact expected**.

Do this for **each** of the three walks. A branched fixture is required: a
single-chain fixture cannot catch a tail that drops or reorders a branch, which
is how brief-462's panel was green while branch handling was untested.

## Registry and inventory

Flip both `AssociationKeyFlatten` and `AssociationKeyDeflatten` to
`Mechanism::BoundedHeap`, `failure_direction: Some(FailureDirection::Unmeasured)`
and `owed_by: None`, matching the `MergeNested` row
(`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:389-396`).
Update the inventory rows for every edge you convert.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/key_reshape.rs` and new sibling modules
    under `crates/rum-stdlib/src/associations/key_reshape/` if the file needs
    splitting by responsibility (it is 13 760 bytes today, so probably not);
  - `crates/rum-stdlib/src/associations/mod.rs` if exports change;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - `crates/rum-stdlib/src/tests/associations/associations_key_reshape.rs`
    and/or a new focused certificate file plus its wiring in
    `crates/rum-stdlib/src/tests.rs`.
- Files you MUST NOT touch: the guard substrate, `kernel_api.rs`,
  `associations/missing*` (just converted), the other two conversion sites
  (`MapIndexed`, the `AssociateTo`/`KeyDropFrom` Part target), `metadata.rs`,
  `Expr`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other `docs/**`,
  `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended change is that these three walks stop growing
  the native stack. Any change to path order, collision behaviour, rule heads,
  the delayed flag, the key-list shape, or any other builtin is a **stop**.

## rum / Rust pitfalls (read twice)

1. **A per-element allocation on the entry loops is a regression.** This
   file family was caught adding one allocation per association level two
   conversions ago, found only by counters — an owned
   `into_iter().filter_map(…).collect()` can be satisfied in place, and
   rewriting it as `Vec::with_capacity` plus `push` gives that up. Counters are
   the acceptance.
2. Convert the mutual render pair together; half a cycle is unbounded.
3. Assert exact expressions, never `Depth` or `Length`. Twelve certificates in
   this line were green while exercising nothing for that reason.
4. Logical wrappers are not physical height; the association fixture is two
   physical levels per wrapper, so raw 4,097 is `N=2048`.
5. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the release certificates including the round trip, and the three
forced-front falsifiers.

Then the cost gate, **counters first**: allocation counts
(`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base against candidate, both
arms built at the same filesystem path, for a shallow flatten probe, a shallow
deflatten probe, a **wide** probe at 4,000 entries, and one untouched control —
with sensitivity calibrated and the same-binary spread stated. Then
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
  exact value; a forced-front falsifier does not exercise its tail; allocation
  counts move on the wide probe or the control; or the same step fails twice for
  the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. Each of the three walks: its front constant, its tail's work representation,
   and for the render pair how you bounded both halves.
3. Verification: every command with verbatim output, and the ZeroMQ statement.
4. The certificates including the round trip, and the three falsifiers.
5. Registry and inventory row changes.
6. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
7. Stop-loss and open questions.
