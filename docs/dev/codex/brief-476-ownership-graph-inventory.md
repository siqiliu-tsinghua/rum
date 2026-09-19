# brief-476 — the ownership-graph inventory (design-only)

## Goal

rum's depth tooling models **calls** and does not model **ownership**. Design the
missing half: a check that every type whose field graph can reach itself carries
a declared, checkable disposition for what happens when it is dropped — and that
a type nobody has classified cannot ship.

**Scope: NONE — no source change.** The deliverable is a reviewable plan plus a
drafted implementation brief, every claim carrying `path:line`.

## Why this exists, stated as evidence rather than principle

This line built two enforcement layers in brief-460: a `syn` inventory of
direct and mutual **call** cycles, and a per-surface registry of descent
mechanisms. Neither can see an ownership edge. The consequence is on the record:

- **brief-468**: `NestedPathNodes` carries `NestedPathNode::Branch(NestedPathNodes)`,
  so its derived `Drop` recurses. One of two independent implementations of the
  same brief dropped a partially built tree recursively and **terminated the
  process at depth 60,000**; the other added an iterative drop. Only the
  accidental cross-comparison of the two caught it — no test, no inventory row,
  no registry field.
- **brief-474**: `LookupOutcome` carries `List(Vec<Self>)`, same class. It was
  found because the director remembered brief-468, not because anything reported
  it.

Two hazards of the same shape, both found by accident, days apart. A third would
also be found by accident.

**A director scan on 2026-09-18 found 45 self-referential type definitions
across the workspace and not one manual `Drop`** — and that scan is itself an
undercount, because it matched only types naming themselves literally: it
**missed both known hazards**, `LookupOutcome` (which says `Vec<Self>`) and the
`NestedPathNodes`/`NestedPathNode` mutual pair. That miss is the specification
for this check: **whatever you build must find those two without being told
about them.**

## The four questions

### 1. What is the ownership graph, precisely

Define the edge relation and defend it. An owning edge goes from a type to what
its fields can cause to be dropped: through `Vec<T>`, `Option<T>`, `Box<T>`,
`Arc<T>`/`Rc<T>` (last holder), tuples, arrays, `HashMap`/`BTreeMap` key and
value, and enum variant payloads. A **borrowed** field (`&T`) and a `Weak<T>` are
**not** owning edges and must not be treated as such. Generic parameters are the
hard case: `Node<T>` owning `T` closes a cycle only for some instantiations.

Say what you do with each of: generics, trait objects, function-pointer and
closure captures, and types from outside the workspace. **A construct you cannot
decide must be reported as unknown and must fail the check** — not skipped.
A scan that reports clean has to be shown capable of finding something, and this
line lost a round to a regex silently truncated by a comma.

### 2. The disposition vocabulary

Every type on a cycle needs a row. My proposed vocabulary, to be criticised
rather than accepted:

- `IterativeDrop` — a manual `Drop` that drains without recursing. Structural:
  fires wherever the value dies.
- `CallSiteDiscipline(sites)` — an explicit iterative dropper that **someone has
  to remember to call**, so the row must enumerate the call sites and the check
  must verify that set. This is what `NestedPathNodes` has today
  (`drop_nested_path_nodes_iterative`), and it is strictly weaker than the
  previous entry; say whether it should be allowed at all.
- `BoundedByConstruction(reason)` — the depth is capped elsewhere, with the cap
  named and checkable.
- `Unbounded(owed_by)` — honest, visible, and owed. The registry already refuses
  an `UnguardedNative` row whose `owed_by` is `None`; mirror that.

State per disposition **what evidence discharges it**. brief-460's registry was
criticised in a neighbouring project's review for recording a hand-filled safety
conclusion rather than a checkable obligation; that criticism does not fully
land there — its rows carry two orthogonal measured fields — but it lands
squarely on a field that would just say "safe".

### 3. Where the check lives, and what it costs

`basic.rs`-style `syn` scanning of type definitions is far more tractable than a
call graph, because **types are closed**: a struct's fields are all visible in
its definition. Say whether that holds across the workspace and where it breaks.

The check must cover `rum-core`, `rum-runtime` and `rum-stdlib` production
source. Note that `rum-runtime` has no `[dev-dependencies]`, which is why
brief-460's registry had to live in `rum-stdlib`; say where this one goes and
why.

### 4. Two dispositions I expect to be contentious — settle them with evidence

- **`Expr`.** It is recursive through `SharedExpr`/`SharedArgs` and has no manual
  `Drop`. `TODO.md`'s standing engineering debt records that deep `Expr` trees
  built through the Rust API can overflow on drop and that **three iterative
  `Drop` designs were rejected after regressing the hot path 20-50%**. The right
  outcome is almost certainly an explicit exemption row pointing at that
  decision — which would move a debt that currently lives in a prose paragraph
  into the same ledger as everything else. Confirm the citation and propose the
  row.
- **The `Persistent*Node` family — brief-477 has since settled this, and the
  answer is the bad one.** Eleven files define
  `previous: Option<Arc<Persistent…Node<T>>>` — a singly linked list with no
  compaction and no length cap at the append site. All 152 `.appended(` sites sit
  behind a `resume_suspended_*` path, but **a source-language construction
  exists**: on the string-replacement resume paths the tail grows **per element**,
  one suspension and then a node per character, so it needs one host round trip
  and not N. Two constructions reached tail length 1 000, each verified by
  instrumentation planted in the `appended` path.

  **Nothing caps it.** With the generated-element limit and the iteration-work
  limit *both set to 8*, the sequence still reached 1 000: those suffix loops
  enforce neither, and the scalar replacement count defaults to `usize::MAX`
  (`crates/rum-stdlib/src/strings/patterns.rs:746`).

  So this family's row is **reachable and unbounded**, and it is the strongest
  argument that this inventory needs to exist. What is still unmeasured is the
  *drop ceiling* — how long the chain has to be before its destruction stops
  being survivable — and that bracket is the director's, not yours. Your row must
  record the reachability and the absence of a cap; it must **not** guess the
  ceiling.

## Also deliver

5. **The falsifier, and it is already known.** The check ships with a proof it
   fires. The strongest available proof is that it **independently rediscovers
   `LookupOutcome` and the `NestedPathNodes`/`NestedPathNode` pair** — the two
   real hazards this project found by accident, and the two that the director's
   naive scan missed. If your design cannot find both, it does not work yet.
   Say so rather than weakening the requirement.

6. **A drafted implementation brief** in `TEMPLATE.md` shape, with its gate
   tests and its falsifier.

## Context pointers

- `AGENTS.md` first, including the Testing Policy hard lines.
- `crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs`
  and `registry.rs` — the call-side half, and the enforcement patterns to mirror.
- `crates/rum-stdlib/src/associations/key_reshape/traversal.rs` — the
  `NestedPathNodes` hazard and its call-site-discipline repair.
- `crates/rum-stdlib/src/associations/basic/lookup.rs` — the `LookupOutcome`
  hazard and its `impl Drop` repair; contrast the two strengths.
- `crates/rum-core/src/expr.rs` — `Expr`, and `TODO.md`'s standing debt on it.
- `crates/rum-derive/src/lib.rs` — `syn` is already a workspace dependency and
  the crate already turns validation failures into compile errors.

## Scope (files)

- Files you MAY modify: **NONE — design-only.** Append your report to this file.
- Do not run any probe that drives a recursion to or past its ceiling.

## rum / Rust pitfalls (read twice)

1. **Mark every claim measured or reasoned.** Reasoned packet arithmetic has
   been reliable here for operation-local walks and unreliable once a descent
   crosses the evaluator; ownership edges are a third thing and have no track
   record either way.
2. **`Arc`/`Rc` own conditionally** — only the last holder drops the payload.
   Say how the disposition vocabulary handles "recurses only sometimes".
3. A count derived by a script is a claim, not a fact; cross-check the type count
   by hand on a few files and say which.
4. Wall-clock timing is not evidence on this host, and nothing here needs it.

## Verification

Design-only: no test gate. Every claim carries `path:line`; paste verbatim any
command output you rely on.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any file except appending your report to this one.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Do not fabricate a citation; if you cite evidence it must be at the line you
  name.
- Paste real verbatim output. If something was not established, say so.

## Report format

1. Design-only, no source change — state it.
2. The edge relation, and your decision for generics, trait objects, closures
   and foreign types.
3. The disposition vocabulary, with what discharges each, and your verdict on
   whether `CallSiteDiscipline` should exist.
4. Where the check lives and what it costs.
5. The `Expr` and `Persistent*Node` rows.
6. The falsifier, and confirmation that your design finds both known hazards.
7. The drafted implementation brief.
8. Risks and open questions.
