# brief-471 — the `associations/basic.rs` key family (design-only)

## Goal

Five operations in one file recurse on nested input and **all five terminate the
process** above their ceiling; three of them do it below the ruled physical
height 4,097. Establish their real call graph, explain a two-fold cost
difference that the current inventory says should not exist, and produce an
ordered repair plan with drafted implementation briefs.

**Scope: NONE — no source change.** The deliverable is a reviewable diagnosis
plus drafted briefs, every claim carrying `path:line`.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3. Every probe carried a liveness assertion on the exact value.
**All five terminate the process above their ceiling.**

| operation | probe shape | ok <= | fail >= | vs 4,097 |
|---|---|---|---|---|
| `KeyTake` | nested list collection | 2,856 | 2,864 | **1.43x short** |
| `KeyDrop` | nested list collection | 2,856 | 2,872 | **1.43x short** |
| `Keys` | nested list collection | 3,056 | 3,080 | **1.34x short** |
| `Lookup` | nested **key specification** | 4,760 | 4,784 | clears by 1.16x |
| `Values` | nested list collection | 5,896 | 5,904 | clears by 1.44x |

`Lookup`'s 1.16x is the thinnest margin anywhere in this class — thinner than
the `DeleteMissing` target mode this line treated as urgent — and a margin
thinner than the drift is not a margin: the lexical bracket moved about 8% over
fourteen unrelated commits.

A ceiling is not deterministic; the same height flips run to run inside a band
tens of levels wide. Do not treat any number above as exact to better than about
50 levels, and do not build a test that depends on one.

## The contradiction this brief exists to resolve

brief-454's inventory records:

- `Keys` / `Values` over nested lists as one recursion at
  `crates/rum-stdlib/src/associations/basic.rs:752-810`, **336 B per list
  layer**;
- `KeyTake` / `KeyDrop` over nested lists as one recursion at
  `crates/rum-stdlib/src/associations/basic.rs:818-915`, **720 B per list
  layer**.

The second pairing is confirmed: `KeyTake` and `KeyDrop` bracket **identically**
at 2,856. The first is not: `Keys` gives out at 3,056 and `Values` at 5,896 on
the *same fixture shape* — nearly 2x apart, which one shared walk at one packet
size cannot produce.

**Establish which is true.** Either they do not share the walk, or they share it
and diverge somewhere the inventory does not name — a different wrapper, an
extra rebuild, a per-layer allocation on one side only. Say which, with
`path:line`, and correct the inventory's citation in your report (not in the
file — that is not in scope here).

This matters beyond bookkeeping: a repair planned on "one shared walk, fix it
once" would leave one of them unfixed.

## The four questions

### 1. The real call graph

For each of the five, the exact recursive edges and what each recursion is
over. Name, per operation, whether the axis is:

- a **collection shape** — nested `List`s in the *value* argument; or
- an **argument shape** — a nested specification the caller passes, which is
  what `Lookup`'s nested key spec is.

That distinction is a reachability statement, and this line has just found it
matters: `KeyComplement`'s recursion turned out to live on an **error-message**
path, reached by a deep *bad argument* rather than a deep value. Check whether
any of these five has the same property.

### 2. Why 336 B and 720 B, and why `Keys` differs from `Values`

Account for the per-layer costs from source. The `/proc` VmStk high-water method
at several sub-ceiling heights is available if you want a slope; it produces one
with no failure at all.

### 3. Does one conversion close all five, or how many are there really?

Given the answer to question 1, say how many distinct walks exist and which
operations each serves. If two operations share a walk, converting it once must
be shown to fix both — that is exactly what the `Keys`/`Values` discrepancy
casts doubt on.

State what must be preserved: result order and nesting shape, the
association-versus-rule-list distinction visible in the results
(`KeyDrop` on a nested list returns lists wrapping an `Association`), key
order, and rule heads.

### 4. The plan and drafted implementation briefs

Ordered by this line's ranking — fail-open before fail-closed, then by shortfall
— with gate tests per step, the certificate shape (a unique deepest target, an
assertion on the exact rebuilt value, a shallow companion), and the falsifier
each step needs.

**On falsifiers, follow what this line settled on** rather than inventing a
shape: an in-module test that calls a private `*_with_front_depth` directly,
asserting front `0` returns `Exhausted`, front `16` returns `Complete`, and
**tail == front == exact expected** on a **branched** fixture with a distinct
witness per branch. No cargo feature, no production switch. A single-chain
fixture cannot catch a tail that drops or reorders a branch.

**On the shape**, note before drafting: every conversion in this line since
brief-468 has used the simple bounded-front-plus-redo form, and the test for
whether that is safe is mechanical — does the walker take a `Session`? If none
of these five does, say so and draft the simple shape. Only
`DeleteMissing`'s target mode needed a carried continuation, because it
evaluates user code mid-walk.

## Context pointers

- `AGENTS.md` first, including the Testing Policy hard lines.
- `crates/rum-stdlib/src/associations/basic.rs` — the whole file; `:549-560`,
  `:699-721`, `:752-810`, `:818-915` are the ranges brief-454 named.
- `docs/dev/codex/brief-454-association-depth-class.md`, section 4's second
  table — the inventory whose citation is in question.
- The conversions to pattern after:
  `crates/rum-stdlib/src/associations/key_reshape/traversal.rs`,
  `crates/rum-stdlib/src/functional_ops/apply_map/traversal/map_indexed.rs`,
  and `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs`,
  each with its in-module forced-front test.
- `TODO.md`, the 2026-09-05 entries for briefs 462-470 and the residual-shape
  measurements.

## Scope

- Files you MAY modify: **NONE — design-only.** Append your report to this file.
- **Do not run any probe that drives a recursion to or past its ceiling**, and
  do not write a reproduction whose purpose is to terminate a process. Every
  bracket you need is above. Sub-ceiling `/proc` VmStk sampling is permitted.
- If a claim cannot be settled without a past-ceiling measurement, say so and
  stop there. That is a correct outcome.

## rum / Rust pitfalls (read twice)

1. **Mark every claim measured or reasoned.** The claim this brief exists to
   check was a reasoned one that read as settled.
2. **Reasoned packet arithmetic is reliable for an operation-local native walk
   and unreliable once the descent can reach `Session::evaluate`.** State, per
   walk, which side of that line it is on — it also decides the repair shape.
3. `Keys`/`Values` and `KeyTake`/`KeyDrop` produce *different result shapes* on
   the same input; do not assume symmetry from the names.
4. Logical wrappers are not physical height. These fixtures are nested lists,
   one physical level per wrapper.
5. Wall-clock timing is not evidence on this host.

## Verification

Design-only: no test gate. The evidence is the deliverable, and every claim
carries `path:line`. Paste verbatim any command output you rely on.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any file except appending your report to this one.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Do not fabricate a citation; if you cite evidence it must be at the line you
  name.
- Paste real verbatim output. If something was not established, say so.

## Report format (your final message)

1. Design-only, no source change — state it.
2. The call graph: per operation, its recursive edges and its axis
   (collection shape or argument shape), and whether any is a diagnostic path.
3. The `Keys`-versus-`Values` resolution, with the corrected citation.
4. How many distinct walks there are and which operations each serves.
5. The ordered plan and the drafted implementation briefs, with falsifiers in
   the in-module forced-front shape.
6. Risks and open questions, including anything you expect a director
   measurement to contradict.
