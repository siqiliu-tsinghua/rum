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

---

## Codex design report — 2026-09-05

### Status and evidence labels

**Design-only; no source change.** This report is the only deliverable. I did
not run a test gate, a recursion probe, or any workload at or beyond a measured
ceiling. The five release brackets and their approximately 50-level uncertainty
are accepted as the brief's **measured** facts
(`docs/dev/codex/brief-471-basic-key-family-design.md:14-35`). Everything below
is marked **measured/observed** when it comes from those supplied brackets or
the verbatim local disassembly, and **reasoned** when it is a conclusion from
source or packet arithmetic. The disassembly is supporting evidence rather
than a reproducible build certificate: the only provenance established here is
the source/binary timestamps and current `HEAD`, reproduced verbatim below.

The future briefs must test rum's own contract, never an external engine
(`AGENTS.md:36-50`; `docs/testing-strategy.md:3-14`), and must use bounded
process-group execution for long commands (`AGENTS.md:86-93`).

### 1. Real call graph and reachability

There are **seven explicit operation-transform recursive edges**, all reachable
on successful, valid calls. None is a diagnostic-only recursion like the `KeyComplement`
case. The public evaluators do create diagnostics after a worker rejects a
leaf, but invalid input is not required to reach any edge
(`crates/rum-stdlib/src/associations/basic.rs:332-384,419-473,699-779,781-810,818-915`).

| operation | route and exact recursive edge | axis | diagnostic-only? / evaluator boundary |
|---|---|---|---|
| `KeyTake` | `eval_key_take` calls `keyed_take_drop_result_for_expr(..., true)` (`crates/rum-stdlib/src/associations/basic.rs:419-445`); the worker calls itself once per nested-list child (`crates/rum-stdlib/src/associations/basic.rs:818-827,890-914`). | **Collection shape:** nested `List`s in the value argument. A terminal association, rule, or rule-list is selected without descending through its values (`crates/rum-stdlib/src/associations/basic.rs:829-888`). | No. A valid nested collection reaches the edge; the worker signature has no `Session` (`crates/rum-stdlib/src/associations/basic.rs:818-823`). |
| `KeyDrop` | `eval_key_drop` calls the same worker with `false` (`crates/rum-stdlib/src/associations/basic.rs:447-473`); the exact self-edge is again `keyed_take_drop_result_for_expr -> keyed_take_drop_result_for_expr` (`crates/rum-stdlib/src/associations/basic.rs:897-900`). | **Collection shape**, identical descent to `KeyTake`; only terminal filtering and error attribution depend on `keep_selected` (`crates/rum-stdlib/src/associations/basic.rs:837-885,901-909`). | No, and no `Session` enters the worker (`crates/rum-stdlib/src/associations/basic.rs:818-823`). |
| `Keys` | `eval_keys -> keys_result_for_expr` (`crates/rum-stdlib/src/associations/basic.rs:332-356`), then the closure calls `keys_result_for_expr` for each nested-list child (`crates/rum-stdlib/src/associations/basic.rs:752-779`). | **Collection shape:** nested `List`s in the value argument. Rules and keyed collections are terminal projections (`crates/rum-stdlib/src/associations/basic.rs:752-766`). | No. Invalid leaves are an error exit at lines 768-770, not the only way to reach the valid list edge; the worker has no `Session` (`crates/rum-stdlib/src/associations/basic.rs:752-778`). |
| `Values` | `eval_values -> values_result_for_expr` (`crates/rum-stdlib/src/associations/basic.rs:359-383`), then a direct self-call in the explicit child loop (`crates/rum-stdlib/src/associations/basic.rs:781-810`). | **Collection shape:** nested `List`s in the value argument. Rules and keyed collections are terminal projections (`crates/rum-stdlib/src/associations/basic.rs:781-794`). | No. Invalid leaves are an error exit at lines 797-805; the worker has no `Session` (`crates/rum-stdlib/src/associations/basic.rs:781-806`). |
| `Lookup` — query count | `prepare_lookup -> lookup_query_count`, whose fold calls `lookup_query_count(key)` (`crates/rum-stdlib/src/associations/basic.rs:665-670,699-707`). | **Argument shape:** nested `List`s in the caller's key specification. | No. It is run before keyed-data validation and therefore can also precede an invalid-data diagnostic, but ordinary valid lookups run it too (`crates/rum-stdlib/src/associations/basic.rs:665-674`). It has no `Session` (`crates/rum-stdlib/src/associations/basic.rs:699-706`). |
| `Lookup` — outcome planning | `prepare_lookup -> lookup_entries_outcome`, whose child loop calls `lookup_entries_outcome(entries, key)` (`crates/rum-stdlib/src/associations/basic.rs:665-689,709-721`). | **Argument shape:** it mirrors the nested key specification into `LookupOutcome::List`. A list of data inputs is only iterated one level by `prepare_lookup`; it is not recursively descended as the measured axis (`crates/rum-stdlib/src/associations/basic.rs:674-689`). | No. Valid hit and missing leaves both reach the planner (`crates/rum-stdlib/src/associations/basic.rs:724-740`), and it has no `Session` (`crates/rum-stdlib/src/associations/basic.rs:709-740`). |
| `Lookup` — normal owned render | `finish_prepared_lookup -> LookupOutcome::into_expr`; `item.into_expr(...)` is the recursive edge (`crates/rum-stdlib/src/associations/basic.rs:248-278,584-599`). | **Argument-derived result shape:** the recursive outcome mirrors the nested key spec, with at most the one outer data-list layer added by `prepare_lookup` (`crates/rum-stdlib/src/associations/basic.rs:674-688,709-721`). | No. Hits, misses, and lists are normal result arms; no `Session` is taken (`crates/rum-stdlib/src/associations/basic.rs:584-599`). |
| `Lookup` — suspended borrowed render | `finish_suspended_prepared_lookup -> LookupOutcome::to_expr`; `item.to_expr(...)` is the recursive edge (`crates/rum-stdlib/src/associations/basic.rs:281-311,601-615`). | The same **argument-derived result shape** as the owned renderer. | No. This is the ordinary post-resume result path and the method has no `Session` (`crates/rum-stdlib/src/associations/basic.rs:281-311,601-615`). |

The seven-edge count is deliberately the explicit operation transform. The
self-recursive `LookupOutcome` representation also induces generated field
tracing and ordinary recursive destruction; those are not another language
walk, but they are stack-safety surfaces that the `Lookup` conversion must
close (`crates/rum-stdlib/src/associations/basic.rs:571-582`;
`crates/rum-derive/src/lib.rs:134-182`;
`crates/rum-runtime/src/runtime/suspension/trace.rs:70-75`).

`Lookup` is the only public route here that crosses `Session::evaluate`, but it
does so outside all four pure recursive edges: data and key are evaluated
before preparation, while a needed default and then a needed wrapper are
evaluated after preparation (`crates/rum-stdlib/src/associations/basic.rs:139-199,201-278`).
The default is evaluated only when `has_missing`, and the wrapper only when
`has_hit` (`crates/rum-stdlib/src/associations/basic.rs:208-245,248-278`). Thus a
bounded front may discard partial pure work and redo the whole phase in a cold
heap tail without replaying user code. Suspension still constrains the stored
outcome representation, addressed in the `Lookup` brief below
(`crates/rum-stdlib/src/associations/basic.rs:221-234,254-267`).

`KeyTake`/`KeyDrop` parse a key spec before inspecting each recursive value
node, but that parser itself is shallow: it maps only the immediate `List`
items and unwraps one-argument `Key` forms (`crates/rum-stdlib/src/associations/data.rs:97-116`).
It is therefore not another depth edge and can be parsed once before the
bounded/iterative value walk, provided the existing error result is preserved.

### 2. `Keys` versus `Values`, and the packet accounting

#### Resolution and corrected citation

**Settled from source:** `Keys` and `Values` do **not** share a walk. The
brief-454 row joins two adjacent functions into one citation and assigns both
the same packet (`docs/dev/codex/brief-454-association-depth-class.md:326-332`).
The corrected inventory citation is:

- `Keys`: `keys_result_for_expr`, including its iterator-closure self-call, at
  `crates/rum-stdlib/src/associations/basic.rs:752-779` (the recursive call is
  specifically lines 772-778).
- `Values`: `values_result_for_expr`, including its direct loop self-call, at
  `crates/rum-stdlib/src/associations/basic.rs:781-810` (the recursive call is
  specifically lines 801-806).

The inventory test already declares separate free-function edges for these two
names (`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:369-374,389-395`),
so the mistaken part is brief-454's combined explanatory row, not the current
declaration identity.

#### Why the apparent 336-byte equality produces nearly 2x ceilings

**Observed in the current local release artifact; reasoned arithmetic:** the
body packet for each function happens to be 336 bytes, but only `Values` makes
a direct body-to-body recursive call. `Values` has six 8-byte pushes, a
`0x118` reservation, and the child's return address: `48 + 280 + 8 = 336`.
Its earlier `try_process` call belongs to terminal keyed-entry parsing and has
returned before the direct self-call seen at `basic.rs:803`
(`crates/rum-stdlib/src/associations/basic.rs:786-806`).

For `Keys`, the source expression is
`.map(|item| keys_result_for_expr(...)).collect::<Result<Vec<_>, _>>()`
(`crates/rum-stdlib/src/associations/basic.rs:772-778`). The disassembly shows
the recursive call under two still-resident iterator frames:

- `keys_result_for_expr`: four pushes + `0x128` + return =
  `32 + 296 + 8 = 336` bytes;
- `core::iter::adapters::try_process`: five pushes + `0x60` + return =
  `40 + 96 + 8 = 144` bytes;
- the specialized `Vec::from_iter`: six pushes + `0xa8` + return =
  `48 + 168 + 8 = 224` bytes.

The current artifact's effective recursive chain is therefore nominally
`336 + 144 + 224 = 704` bytes per `Keys` list layer, versus 336 bytes for
`Values`. The supplied passing ceilings have ratio `5,896 / 3,056 = 1.93`,
while the artifact packet ratio is `704 / 336 = 2.10`; the direction and scale
resolve the contradiction, while fixed stack use, the supplied approximately
50-level instability, compiler layout, and unproven artifact identity prohibit
treating either ratio as an exact prediction
(`docs/dev/codex/brief-471-basic-key-family-design.md:14-35`).

#### Why `KeyTake`/`KeyDrop` are 720 bytes

**Observed packet, reasoned source attribution:** their one shared function has
six pushes, a `0x298` reservation, and a return address:
`48 + 664 + 8 = 720` bytes. The monolithic frame contains immediate key-spec
parsing, both indexed and scan selection implementations, association-entry
storage, and the recursive list-result/error path
(`crates/rum-stdlib/src/associations/basic.rs:818-914`). Because both public
heads enter that exact symbol and differ only in the boolean argument
(`crates/rum-stdlib/src/associations/basic.rs:419-473`), converting this worker
once genuinely repairs both recursive edges; each public head still needs its
own certificate.

The nominal products cluster around the configured stack rather than proving a
ceiling: `2,856 * 720 = 2,056,320` bytes for `KeyTake`,
`5,896 * 336 = 1,981,056` for `Values`, and
`3,056 * 704 = 2,151,424` for `Keys`. This is a reasoned cross-check against
the supplied brackets, not a new measurement
(`docs/dev/codex/brief-471-basic-key-family-design.md:14-35`).

#### The omitted `Lookup` renderer

The current brief-454 row names only query count and planning
(`docs/dev/codex/brief-454-association-depth-class.md:332`), and the closed
inventory likewise declares only those two edges
(`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:376-387`).
That is incomplete: `LookupOutcome::into_expr` and `LookupOutcome::to_expr`
also recurse (`crates/rum-stdlib/src/associations/basic.rs:577-615`). The scanner
documents that it recognizes direct and `self`/`Self` method calls, but its
method-call visitor records only a receiver literally spelled `self`
(`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:1-9,123-141`).
The actual receivers are named `item`, so both same-type method edges escape
the scan (`crates/rum-stdlib/src/associations/basic.rs:591-596,607-612`).

**Observed packet, reasoned likely binder:** the local artifact gives 80 bytes
for `lookup_query_count`, 304 bytes for `lookup_entries_outcome`, 432 bytes for
the normal owned render (`240` in `into_expr` plus `192` in its in-place collect
helper), and 272 bytes for the borrowed suspended render (`144 + 128`). The
ordinary measured probe necessarily runs count, planning, and then the owned
render sequentially; their packets are maxima, not additive across phases
(`crates/rum-stdlib/src/associations/basic.rs:665-740,248-278,584-615`). On this
artifact, `4,760 * 432 = 2,056,320` bytes, so the omitted owned renderer is the
best explanation for the supplied ordinary-path ceiling. This phase was not
isolated by measurement, so it remains a reasoned attribution rather than a
settled phase measurement.

### 3. How many walks, and what must remain exact

There is no one conversion that closes all five operations.

- At **explicit transform-edge granularity** there are seven recursions: one shared
  `KeyTake`/`KeyDrop` edge, one `Keys` edge, one `Values` edge, and four
  `Lookup` edges (count, plan, owned render, borrowed render), cited in the call
  graph above.
- At **semantic-pass granularity** there are six walks: shared take/drop,
  `Keys`, `Values`, lookup count, lookup planning, and lookup rendering; the
  renderer currently has owned and borrowed implementations
  (`crates/rum-stdlib/src/associations/basic.rs:584-615`).
- At **repair/dispatch granularity** there are four implementation briefs:
  shared `KeyTake`+`KeyDrop`, `Keys`, all of `Lookup`, and `Values`. Only the
  first brief repairs two public operations with one converted worker
  (`crates/rum-stdlib/src/associations/basic.rs:419-473,818-915`).

The conversion must preserve these contracts exactly:

- Sibling order and every nested `List` wrapper for `Keys` and `Values`; the
  existing examples pin association/rule-list projection, wrappers, and nested
  output shape (`crates/rum-stdlib/src/tests/associations/associations.rs:419-473`).
- The deliberately different invalid payload: `Keys` propagates the deepest
  bad leaf, while `Values` reports the enclosing root collection
  (`crates/rum-stdlib/src/associations/basic.rs:768-778,797-806`;
  `crates/rum-stdlib/src/tests/associations/associations_basic_diagnostics.rs:100-137`).
- For take/drop, an empty list remains `List[]`; a terminal rule-list renders
  as an `Association`; outer nested collection wrappers remain `List`s
  (`crates/rum-stdlib/src/associations/basic.rs:829-887,890-914`;
  `crates/rum-stdlib/src/tests/associations/associations_key_diagnostics.rs:7-37`;
  `crates/rum-stdlib/src/tests/strings/incidental_runtime/association_key_ops.rs:91-105`).
- `KeyTake` follows selection order, collapses repeated selection as currently
  rendered, and silently skips misses; `KeyDrop` retains input-entry order and
  ignores repeated/missing selectors. Both scan and indexed branches must agree
  (`crates/rum-stdlib/src/associations/basic.rs:834-885`;
  `crates/rum-stdlib/src/tests/strings/incidental_runtime/association_key_ops.rs:394-463`).
- `KeyTake` reports the enclosing/root collection on a nested invalid leaf,
  whereas `KeyDrop` retains the deepest bad leaf
  (`crates/rum-stdlib/src/associations/basic.rs:897-909`;
  `crates/rum-stdlib/src/tests/associations/associations_key_diagnostics.rs:53-91`).
- `Rule` versus `RuleDelayed` heads must survive take/drop. That bit is carried
  by `AssociationEntryForm` and used by the preserving renderer
  (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:12-20,137-139,308-324`),
  with behavior pinned at
  `crates/rum-stdlib/src/tests/strings/incidental_runtime/association_key_ops.rs:49-58`.
- `Lookup` must preserve key-spec nesting/order, `Key[x]` unwrapping, exact hit
  and `Missing["KeyAbsent", key]` placement, list-of-data ordering, and the
  first matching entry selected by its indexed/scan paths
  (`crates/rum-stdlib/src/associations/basic.rs:624-647,665-750`;
  `crates/rum-stdlib/src/tests/associations/associations.rs:233-247,273-287,329-417`).
- `Lookup` must also preserve conditional, once-only default/wrapper evaluation
  and resume without replaying its prepared result
  (`crates/rum-stdlib/src/associations/basic.rs:201-311`;
  `crates/rum-stdlib/src/tests/associations/associations.rs:217-231,257-271,305-327`;
  `crates/rum-stdlib/src/tests/incidental_session_requests_input_resume.rs:307-337`).

### 4. Ordered repair plan

All five supplied probes fail open above their ceiling, so the ordering first
places the below-contract sites by largest shortfall, then the passing sites by
thinnest margin: **(1) shared `KeyTake`/`KeyDrop` (1.43x short), (2) `Keys`
(1.34x short), (3) `Lookup` (1.16x clear), (4) `Values` (1.44x clear)**
(`docs/dev/codex/brief-471-basic-key-family-design.md:14-35`). This combines the
short-of-contract queue with the residual queue described in
`TODO.md:2302-2352`; it does not pull the out-of-scope `KeyComplement` or other
families into these briefs.

#### Common implementation shape and gates

Each brief uses a depth-16 recursive front and, on exhaustion, discards the
bounded partial attempt and redoes the original pure phase with an explicit
heap work stack. That is the established `BoundedTraversal::{Complete,
Exhausted}` shape used by key reshape and `MapIndexed`
(`crates/rum-stdlib/src/associations/key_reshape/traversal.rs:5-21,104-167,232-315`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal/map_indexed.rs:3-40,174-205`).
The iterative DFS must push siblings in reverse and finish them in original
order, as the precedents do
(`crates/rum-stdlib/src/associations/key_reshape/traversal.rs:301-381`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal/map_indexed.rs:326-383`).
Every cold tail is `#[cold]` and `#[inline(never)]`, following those same
implementations
(`crates/rum-stdlib/src/associations/key_reshape/traversal.rs:313-315,447-452`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal/map_indexed.rs:198-204`).

Every brief must include an in-module direct-helper falsifier: a private
`*_with_front_depth`, front `0` asserted `Exhausted`, front `16` asserted
`Complete`, and `tail == front == exact_expected` on a branched fixture with a
different witness in every branch. This is the existing shape, including the
exact three-way comparison
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal/map_indexed.rs:417-493`;
`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths_tests.rs:3-45,49-105`).
There is no cargo feature and no production forcing switch.

The release certificate is run **only after conversion** on a named 2 MiB
thread at raw physical height 4,097. Its fixture and height are constructed and
checked iteratively, it carries one unique deepest sentinel, and the operation
is checked against the exact rebuilt value—not merely `Depth`, `Length`, or
process survival. The recent certificate provides the concrete pattern
(`crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:6-21,58-122,257-307`).
Each certificate gets a three-level branched companion pinning order, wrappers,
terminal shape, and rule heads. No pre-conversion 4,097 run is permitted for
`KeyTake`, `KeyDrop`, or `Keys`, because their supplied ceilings are already
below it (`docs/dev/codex/brief-471-basic-key-family-design.md:14-25`).

After each step, update its depth registry rows from `UnguardedNative` to
`BoundedHeap`, clear `owed_by`, and record `MeetsContract` with unmeasured
failure direction, matching converted rows. The current rows are at
`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:211-219,310-318,346-363,454-462`.
Update the closed recursion declarations at the same time
(`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:362-395`).

All command gates must be bounded. Run focused in-module and semantic tests,
then `cargo fmt --check`, `cargo test -p rum-stdlib`, and the bounded workspace
suite (`AGENTS.md:75-93`). Because these are evaluator/output construction hot
paths, take a clean base-versus-candidate `scripts/core-language-microbench.sh`
A/B and compare target labels plus same-tier neighbors; intent is not evidence
(`AGENTS.md:46-49,117-127`). Add allocation-count comparisons for shallow,
depth-8, threshold-crossing, and wide inputs, plus untouched family controls;
any unexplained default-path allocation or timing increase is a stop-and-report
condition.

`basic.rs` is already 28,418 bytes (verbatim `wc` output below), placing it in
the 20k-35k split-plan range where an on-touch split assessment is required
(`docs/code-organization-guidelines.md:64-81`). The recommended boundary is a
normal `basic/traversal/` module: a small `mod.rs`, then responsibility files
for `key_take_drop`, `projections`, and `lookup`, leaving public evaluator and
suspension orchestration in `basic.rs`. Make the extraction mechanically before
behavioral edits and keep visibility minimal; the repository requires coherent
responsibility boundaries rather than arbitrary cuts
(`docs/code-organization-guidelines.md:124-174`).

#### Draft implementation brief 1 — shared `KeyTake` / `KeyDrop`

**Goal.** Convert the one collection-shape recursion serving both heads to a
depth-16 front plus a cold iterative tail, preserving byte-for-byte expression
semantics and diagnostics. Both public routes share the worker today
(`crates/rum-stdlib/src/associations/basic.rs:419-473,818-915`).

**Implementation.** Parse the immediate key spec once, then pass the selected
keys into a small nonrecursive terminal selector and a collection traversal;
the current parser neither evaluates nor recursively descends
(`crates/rum-stdlib/src/associations/data.rs:97-116`). Introduce
`keyed_take_drop_result_for_expr_with_front_depth(..., remaining_depth) ->
BoundedTraversal<Result<Expr, KeyTakeDropInvalid>>`. Its recursive front stops
before the seventeenth collection edge. On exhaustion, redo the original root
using `Visit` and `FinishList` tasks, reverse-schedule children, and keep a
completed-result stack. Classify empty `List` before keyed terminals, classify
rule/rule-list/association terminals before collection descent, and reuse the
existing terminal selection code so scan/index thresholds and head-preserving
rendering remain unchanged
(`crates/rum-stdlib/src/associations/basic.rs:829-888`). Track whether an
invalid leaf is at the root: direct invalid input stays `invrl`; a descendant
becomes `invru`; `KeyTake` reports the outer root and `KeyDrop` reports the
first deepest invalid leaf in left-to-right traversal
(`crates/rum-stdlib/src/associations/basic.rs:890-914`). Use safe Rust only
(`AGENTS.md:109-115`).

**Forced-front falsifier.** In the traversal module, build a three-level
branched value containing both association and rule-list terminals, distinct
left/right keys and values, a `RuleDelayed` witness, and a selection order that
differs from input order. Call the private helper for both `keep_selected`
values at depth 0 and 16. Assert the respective variants and
`tail == front == exact_expected` for both public semantics. As a deliberate
review perturbation, reverse the tail's child scheduling; the exact branch
order assertion must fail. Add a separate branched invalid fixture pinning the
root-versus-leaf diagnostic distinction already tested end-to-end at
`crates/rum-stdlib/src/tests/associations/associations_key_diagnostics.rs:53-91`.

**Certificates and gates.** Add one ignored release-only 2 MiB/4,097 test per
public head. Each input is an iteratively built single-child `List` chain ending
in an association with a unique deepest surviving `RuleDelayed` sentinel;
`KeyTake` selects it, while `KeyDrop` removes a side key and retains it. Assert
the unique input height iteratively and assert the entire nested-list result
and terminal `Association` exactly. The shared conversion is not complete
unless both heads pass. The shallow companion must also cover `List[]`, a
terminal rule-list becoming `Association`, nested lists remaining lists,
selection/input order, duplicates/misses, both sides of the width-16 indexed
threshold, and preserved delayed heads; existing contract coverage is at
`crates/rum-stdlib/src/tests/associations/associations_key_diagnostics.rs:7-91`
and `crates/rum-stdlib/src/tests/strings/incidental_runtime/association_key_ops.rs:34-105,394-463`.
Allocation/performance panels must include shallow and depth-8 take/drop,
15/16/17-entry threshold cases, a wide collection, and `Keys`/`Values` controls.

**Stop conditions.** Stop and report rather than weakening behavior if either
public certificate fails, the forced tail is not observed, the scheduling
perturbation does not redden the falsifier, diagnostic provenance changes, rule
heads/order change, or the hot-path panel regresses. Do not probe above any old
ceiling; the only deep run is the post-conversion contract certificate.

#### Draft implementation brief 2 — `Keys`

**Goal.** Convert only `keys_result_for_expr`; do not treat `Values` as fixed.
The corrected edge is `crates/rum-stdlib/src/associations/basic.rs:752-779`.

**Implementation.** Add
`keys_result_for_expr_with_front_depth(expr, wrapper, remaining_depth)` with a
depth-16 front and whole-root iterative redo. The tail uses `Visit` /
`FinishList` tasks and a completed stack, preserving left-to-right early error
propagation. Keep the current terminal precedence—single rule, keyed
association/rule-list, then nested list—and keep `wrap_keyed_result` at each
projected key (`crates/rum-stdlib/src/associations/basic.rs:692-697,752-778`).
The iterative error must carry the deepest invalid expression, not replace it
with an ancestor (`crates/rum-stdlib/src/associations/basic.rs:768-778`).

**Forced-front falsifier.** Use a three-level branched fixture with an
association on one branch, a rule-list on another, distinct keys in each
sibling, and a nontrivial wrapper. Directly call the private helper at 0 and 16,
assert `Exhausted` and `Complete`, then
`tail == front == exact_expected`. Reversing the tail's child scheduling must
swap distinct key witnesses and fail the test. Add an invalid sibling
companion that proves the deepest bad leaf remains the diagnostic payload,
matching `crates/rum-stdlib/src/tests/associations/associations_basic_diagnostics.rs:100-109,123-129`.

**Certificate and gates.** After conversion, run one ignored release-only
2 MiB certificate on an iteratively built raw-height-4,097 nested `List` chain
whose terminal association contains one unique deepest key sentinel. Check
height/sentinel uniqueness iteratively and compare the entire rebuilt nested
`List` chain ending in `List[sentinel]` exactly. The three-level companion pins
association/rule-list handling, key order, nesting, wrapper placement, and the
deepest-invalid rule already covered at
`crates/rum-stdlib/src/tests/associations/associations.rs:419-441` and
`crates/rum-stdlib/src/tests/associations/associations_basic_diagnostics.rs:100-109,123-129`.
The allocation/timing panel covers scalar terminal, shallow branch, depth 8,
wide siblings, wrapper/no-wrapper, and `Values` plus shared take/drop controls.

**Stop conditions.** Stop if the forced-front variants, exact certificate,
error payload, output ordering/nesting, or hot-path A/B fails. A reduced or
length-only assertion is not an acceptable substitute.

#### Draft implementation brief 3 — all `Lookup` phases

**Goal.** Convert query counting, outcome planning, and both render edges as
one repair; changing only the two edges named by the current inventory leaves
the normal and resumed render recursion in place
(`crates/rum-stdlib/src/associations/basic.rs:584-615,699-740`).

**Implementation.** Give each semantic phase its own depth-16 budget and
private forced-depth entry point:

1. `lookup_query_count_with_front_depth` retains zero for an empty key list and
   `saturating_add` for every leaf, then redoes the root with an iterative DFS
   on exhaustion (`crates/rum-stdlib/src/associations/basic.rs:699-707`).
2. `lookup_entries_outcome_with_front_depth` keeps a bounded recursive fast
   representation for a shallow spec, but on any exhaustion abandons that
   attempt and replans the original spec into a **nonrecursive flat postorder
   arena/token stream** (`Hit`, `Missing`, `FinishList { child_count }`) plus
   the same combined `has_hit`/`has_missing` flags. It must preserve left-to-right
   spec order and the current `LookupPlan::combine` logic
   (`crates/rum-stdlib/src/associations/basic.rs:650-663,709-740`).
3. Add bounded owned and borrowed renderer entry points. A shallow outcome may
   retain the allocation-efficient owned versus borrowed fast paths; a forced
   exhaustion or flat outcome renders iteratively from postorder nodes. If the
   two render methods are unified, both normal and resumed callers still need
   direct coverage because ownership/cloning differs today
   (`crates/rum-stdlib/src/associations/basic.rs:248-311,584-615`).

The flat cold representation is required, not optional cleanup. The current
recursive `LookupOutcome::List(Vec<Self>)` derives `FrameRootTrace`
(`crates/rum-stdlib/src/associations/basic.rs:571-582`). That derive recursively
traces fields, `Vec<T>` recursively dispatches to every child, and a frozen
prepared lookup delegates through `Arc`
(`crates/rum-derive/src/lib.rs:13-16,85-108,134-182`;
`crates/rum-runtime/src/runtime/suspension/trace.rs:18-21,70-75,106-109`;
`crates/rum-stdlib/src/associations/suspended_state.rs:25-41`). A merely
iterative planner that still constructs a 4,097-deep self-recursive outcome
would leave recursive render, trace, clone, and ordinary drop liabilities. The
only recursive tree retained in the proposed hybrid must be privately
constructible and invariant-bounded to the 16-level fast front; the cold form's
node type must not contain itself. No `unsafe` workaround is permitted
(`AGENTS.md:109-115`).

Keep the public evaluation sequence unchanged. Data and key are evaluated once
before pure preparation; default and wrapper stay conditional and ordered;
prepared state remains frozen across either suspension
(`crates/rum-stdlib/src/associations/basic.rs:139-311`). Preserve the lookup
index threshold and first-position map, list-of-data ordering, `Key` unwrapping,
hit/missing payloads, and flags
(`crates/rum-stdlib/src/associations/basic.rs:624-750`).

**Forced-front falsifiers.** All helpers are private and tested in-module; no
feature or production switch.

- Count: a branched nested key spec with a distinct symbol at every leaf must
  give depth 0 `Exhausted`, depth 16 `Complete`, and
  `tail == front == exact_leaf_count`. Temporarily omit one scheduled child;
  this test must fail.
- Planner: use a branched mix of unique hits, misses, and `Key[...]` leaves,
  with unequal branch widths and enough association entries/query leaves to
  exercise the indexed choice. At 0 and 16 assert the variants, flags, and the
  exact rendered witness; `tail == front == exact_expected`. Reverse cold-tail
  scheduling as the deliberate perturbation; distinct branch witnesses must
  make the test fail.
- Renderer: construct a shallow branched outcome with distinct hit/missing
  witnesses, default, and wrapper. Directly force both owned and borrowed
  renderers at 0 and 16 (or both entry points if unified), assert variants, and
  assert `tail == front == exact_expected`. Reversing finish order must fail.

**Certificates and gates.** The primary ignored release certificate builds a
raw-height-4,097 nested key-spec `List` iteratively, with one unique deepest key
that hits a distinct value, invokes public `Lookup` on a named 2 MiB thread,
checks the input height/sentinel iteratively, and compares the entire nested
result chain exactly. The shallow companion combines unequal branches, hits,
misses, `Key`, default, wrapper, duplicate data keys, and list-of-data input,
pinning behavior already covered at
`crates/rum-stdlib/src/tests/associations/associations.rs:217-417`.

Add a second deep 2 MiB certificate that forces a needed default or wrapper to
suspend **after** planning the deep spec, resumes it, and checks the same exact
result plus no replay. This is what exercises flat-outcome freeze, trace, and
borrowed rendering; the current shallow contract test demonstrates the request
sequence to reproduce (`crates/rum-stdlib/src/tests/incidental_session_requests_input_resume.rs:307-337`).
The cost panel covers scalar lookup, shallow nested specs, depth 8, empty spec,
15/16/17 and wide indexed associations, mixed hit/miss, normal versus
suspend/resume, and `Keys`/`Values` controls.

**Inventory gate.** Eliminate the two untracked `item.method(...)` self-edges
by expressing bounded recursion as scanner-visible free helpers, or extend the
scanner with a focused test that proves same-type non-`self` method recursion is
reported. The expected declaration set must name count, plan, and every
remaining bounded renderer edge; the present two-row expectation is incomplete
(`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:1-9,123-141,376-387`).

**Stop conditions.** Stop if any pure redo crosses `Session::evaluate`, either
deep normal or suspended certificate fails, a recursive deep cold outcome
remains, the scanner still misses an edge, conditional evaluation/replay
changes, the forced perturbations stay green, or the performance/allocation
panel regresses. Do not answer a trace/drop failure with `unsafe`.

#### Draft implementation brief 4 — `Values`

**Goal.** Convert the separate `values_result_for_expr` collection walk after
the thinner-margin work is closed. Its exact range is
`crates/rum-stdlib/src/associations/basic.rs:781-810`.

**Implementation.** Add
`values_result_for_expr_with_front_depth(expr, wrapper, remaining_depth)` with
the same depth-16/whole-root-redo form and an iterative `Visit`/`FinishList`
tail. Retain terminal precedence, left-to-right values, wrapper placement, and
the explicit parent-error behavior: when any child is invalid, every enclosing
call replaces the child error with its own `expr`, so the public diagnostic
names the outer input (`crates/rum-stdlib/src/associations/basic.rs:781-810`).
Do not factor it through `Keys`; their error contracts and current fast-path
code shapes differ
(`crates/rum-stdlib/src/tests/associations/associations_basic_diagnostics.rs:100-137`).

**Forced-front falsifier.** A three-level branched fixture contains an
association and a rule-list, distinct values in every branch, a delayed value,
and a wrapper. Direct calls at 0 and 16 assert `Exhausted` and `Complete`, then
`tail == front == exact_expected`; reversing child scheduling must reorder the
distinct witnesses and fail. A companion invalid branch must prove the outer
root remains the payload.

**Certificate and gates.** After conversion, use an iteratively built
raw-height-4,097 single-child `List` chain ending in an association whose value
is one unique deepest sentinel. On a named 2 MiB thread, iteratively prove input
height/uniqueness and compare the complete nested-list output ending in the
exact projected/wrapped sentinel. The shallow companion pins order, nesting,
association/rule-list terminals, delayed-value behavior, wrapper placement,
and root-invalid diagnostics already covered at
`crates/rum-stdlib/src/tests/associations/associations.rs:443-473` and
`crates/rum-stdlib/src/tests/associations/associations_basic_diagnostics.rs:110-137`.
The cost panel covers terminal, wrapper/no-wrapper, depth 8, wide siblings, and
`Keys`/`Lookup` controls.

**Stop conditions.** Stop on any exact-output, root-diagnostic, forced-tail,
perturbation, certificate, or cost-gate failure; do not weaken the contract to
make a test green (`AGENTS.md:46-50`).

### 5. Risks and open questions

1. **Lookup's binding phase is reasoned, not phase-measured.** The 432-byte
   owned renderer best fits the supplied 4,760 ceiling, but a same-build,
   strictly sub-ceiling VmStk slope isolated by phase could show planner,
   renderer, or drop interaction differently. That is the director measurement
   most likely to revise this report; it would change optimization emphasis,
   not the requirement to convert all four edges
   (`crates/rum-stdlib/src/associations/basic.rs:584-615,699-740`).
2. **Packet sizes are compiler-artifact facts.** The source-level separation of
   `Keys` and `Values` is settled, but 704/336/720 and the `Lookup` packets may
   change with inlining or toolchain/codegen. The workspace pins Rust 1.85.x,
   but hot-path claims still require a fresh A/B rather than arithmetic
   (`AGENTS.md:75-78,117-127`).
3. **Lookup's recursive intermediate is a second stack surface.** Planning and
   rendering alone are insufficient if a deep `LookupOutcome` can still be
   recursively traced or dropped during suspension; this has not been bracketed
   and must be closed structurally with the flat cold form and deep resume
   certificate (`crates/rum-stdlib/src/associations/basic.rs:221-234,571-615`).
4. **Diagnostic attribution is deliberately asymmetric.** A generic traversal
   abstraction can easily normalize away the `Keys`/`Values` and
   `KeyTake`/`KeyDrop` differences; exact diagnostic tests are gates, not cleanup
   opportunities (`crates/rum-stdlib/src/tests/associations/associations_basic_diagnostics.rs:100-137`;
   `crates/rum-stdlib/src/tests/associations/associations_key_diagnostics.rs:53-91`).
5. **Order has several meanings.** DFS sibling order, `KeyTake` selector order,
   `KeyDrop` input order, duplicate-key canonicalization, and Lookup key-spec/data
   order are independent invariants; every iterative scheduler needs distinct
   witnesses, not a homogeneous chain
   (`crates/rum-stdlib/src/associations/basic.rs:624-647,674-689,834-885`).
6. **The inventory has a real blind spot.** Until same-type method calls through
   non-`self` receivers are either eliminated or detected, a green closed-world
   recursion test is not proof that all recursion is declared
   (`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:1-9,123-141`).
7. **Certificate construction can accidentally test another axis.** Physical
   height and the unique deepest sentinel must be established iteratively, and
   exact traversal of the rebuilt result must also be iterative, following the
   existing certificate rather than relying on logical wrapper counts
   (`crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:58-122`).

### 6. Verbatim local evidence

No command below executes rum input or changes repository state.

Artifact/source identity check (this does **not** prove the binary's full build
flags or a clean source identity):

```text
$ stat -c '%y %n' crates/rum-stdlib/src/associations/basic.rs target/release/rum-cli && git rev-parse HEAD
2026-09-03 20:36:25.259178759 +0800 crates/rum-stdlib/src/associations/basic.rs
2026-09-05 20:20:08.609103131 +0800 target/release/rum-cli
a663b0bd9f7632e78a39769ca7b6309f95377482
```

Direct worker frames and calls:

```text
$ objdump -d -C --start-address=0x1ecab0 --stop-address=0x1ee7f5 target/release/rum-cli | rg '^[0-9a-f]+ <rum_stdlib::associations::basic::(lookup_query_count|lookup_entries_outcome|keys_result_for_expr|values_result_for_expr|keyed_take_drop_result_for_expr)>:|^  (1ecab[0-9a-f]|1ecb7[0-9a-f]|1ecfc[0-9a-f]|1ed2e[0-9a-f]|1ed7d[0-9a-f]):|call.*<(rum_stdlib::associations::basic::(lookup_query_count|lookup_entries_outcome|values_result_for_expr|keyed_take_drop_result_for_expr)|core::iter::adapters::try_process)>'
00000000001ecab0 <rum_stdlib::associations::basic::lookup_query_count>:
  1ecab0:	41 57                	push   %r15
  1ecab2:	41 56                	push   %r14
  1ecab4:	41 54                	push   %r12
  1ecab6:	53                   	push   %rbx
  1ecab7:	48 83 ec 28          	sub    $0x28,%rsp
  1ecabb:	48 89 fb             	mov    %rdi,%rbx
  1ecabe:	48 8d 35 8f c3 65 00	lea    0x65c38f(%rip),%rsi        # 848e54 <anon.03df2acce02377c53853700d7d7f1ac1.26.llvm.2898413782587572939>
  1ecb13:	e8 98 ff ff ff       	call   1ecab0 <rum_stdlib::associations::basic::lookup_query_count>
00000000001ecb70 <rum_stdlib::associations::basic::lookup_entries_outcome>:
  1ecb70:	55                   	push   %rbp
  1ecb71:	41 57                	push   %r15
  1ecb73:	41 56                	push   %r14
  1ecb75:	41 55                	push   %r13
  1ecb77:	41 54                	push   %r12
  1ecb79:	53                   	push   %rbx
  1ecb7a:	48 81 ec f8 00 00 00	sub    $0xf8,%rsp
  1ecdf5:	e8 76 fd ff ff       	call   1ecb70 <rum_stdlib::associations::basic::lookup_entries_outcome>
00000000001ecfc0 <rum_stdlib::associations::basic::keys_result_for_expr>:
  1ecfc0:	41 57                	push   %r15
  1ecfc2:	41 56                	push   %r14
  1ecfc4:	41 54                	push   %r12
  1ecfc6:	53                   	push   %rbx
  1ecfc7:	48 81 ec 28 01 00 00	sub    $0x128,%rsp
  1ecfce:	49 89 d7             	mov    %rdx,%r15
  1ed056:	e8 85 56 19 00       	call   3826e0 <core::iter::adapters::try_process>
  1ed0c9:	e8 b2 70 19 00       	call   384180 <core::iter::adapters::try_process>
00000000001ed2e0 <rum_stdlib::associations::basic::values_result_for_expr>:
  1ed2e0:	55                   	push   %rbp
  1ed2e1:	41 57                	push   %r15
  1ed2e3:	41 56                	push   %r14
  1ed2e5:	41 55                	push   %r13
  1ed2e7:	41 54                	push   %r12
  1ed2e9:	53                   	push   %rbx
  1ed2ea:	48 81 ec 18 01 00 00	sub    $0x118,%rsp
  1ed37e:	e8 5d 53 19 00       	call   3826e0 <core::iter::adapters::try_process>
  1ed569:	e8 72 fd ff ff       	call   1ed2e0 <rum_stdlib::associations::basic::values_result_for_expr>
00000000001ed7d0 <rum_stdlib::associations::basic::keyed_take_drop_result_for_expr>:
  1ed7d0:	55                   	push   %rbp
  1ed7d1:	41 57                	push   %r15
  1ed7d3:	41 56                	push   %r14
  1ed7d5:	41 55                	push   %r13
  1ed7d7:	41 54                	push   %r12
  1ed7d9:	53                   	push   %rbx
  1ed7da:	48 81 ec 98 02 00 00	sub    $0x298,%rsp
  1edd6f:	e8 5c fa ff ff       	call   1ed7d0 <rum_stdlib::associations::basic::keyed_take_drop_result_for_expr>
```

The iterator frames resident across `Keys` recursion:

```text
$ objdump -d -C --start-address=0x384180 --stop-address=0x384254 target/release/rum-cli | rg '^000000|^  38418|call.*2f2ff0' && objdump -d -C --start-address=0x2f2ff0 --stop-address=0x2f3180 target/release/rum-cli | rg '^000000|^  2f2ff|call.*keys_result_for_expr'
0000000000384180 <core::iter::adapters::try_process>:
  384180:	41 57                	push   %r15
  384182:	41 56                	push   %r14
  384184:	41 55                	push   %r13
  384186:	41 54                	push   %r12
  384188:	53                   	push   %rbx
  384189:	48 83 ec 60          	sub    $0x60,%rsp
  38418d:	48 89 fb             	mov    %rdi,%rbx
  3841c5:	e8 26 ee f6 ff       	call   2f2ff0 <<alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter>
00000000002f2ff0 <<alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter>:
  2f2ff0:	55                   	push   %rbp
  2f2ff1:	41 57                	push   %r15
  2f2ff3:	41 56                	push   %r14
  2f2ff5:	41 55                	push   %r13
  2f2ff7:	41 54                	push   %r12
  2f2ff9:	53                   	push   %rbx
  2f2ffa:	48 81 ec a8 00 00 00	sub    $0xa8,%rsp
  2f3072:	e8 49 9f ef ff       	call   1ecfc0 <rum_stdlib::associations::basic::keys_result_for_expr>
  2f314e:	e8 6d 9e ef ff       	call   1ecfc0 <rum_stdlib::associations::basic::keys_result_for_expr>
```

The omitted owned and borrowed `Lookup` renderer frames:

```text
$ objdump -d -C --start-address=0x1ec430 --stop-address=0x1ec710 target/release/rum-cli | rg '^00000000001ec(430|5e0)|^  1ec43|^  1ec5e[0-9a-f]:|call.*(3d2db0|305e50)' && objdump -d -C --start-address=0x3d2db0 --stop-address=0x3d2e60 target/release/rum-cli | rg '^000000|^  3d2db|call.*LookupOutcome::into_expr' && objdump -d -C --start-address=0x305e50 --stop-address=0x305f30 target/release/rum-cli | rg '^000000|^  305e5|call.*LookupOutcome::to_expr'
00000000001ec430 <rum_stdlib::associations::basic::LookupOutcome::into_expr>:
  1ec430:	41 57                	push   %r15
  1ec432:	41 56                	push   %r14
  1ec434:	41 55                	push   %r13
  1ec436:	41 54                	push   %r12
  1ec438:	53                   	push   %rbx
  1ec439:	48 81 ec c0 00 00 00	sub    $0xc0,%rsp
  1ec551:	e8 5a 68 1e 00       	call   3d2db0 <alloc::vec::in_place_collect::from_iter_in_place>
00000000001ec5e0 <rum_stdlib::associations::basic::LookupOutcome::to_expr>:
  1ec5e0:	41 57                	push   %r15
  1ec5e2:	41 56                	push   %r14
  1ec5e4:	53                   	push   %rbx
  1ec5e5:	48 83 ec 70          	sub    $0x70,%rsp
  1ec5e9:	49 89 f6             	mov    %rsi,%r14
  1ec5ec:	48 89 fb             	mov    %rdi,%rbx
  1ec5ef:	48 89 54 24 08       	mov    %rdx,0x8(%rsp)
  1ec69f:	e8 ac 97 11 00       	call   305e50 <<alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter>
00000000003d2db0 <alloc::vec::in_place_collect::from_iter_in_place>:
  3d2db0:	55                   	push   %rbp
  3d2db1:	41 57                	push   %r15
  3d2db3:	41 56                	push   %r14
  3d2db5:	41 55                	push   %r13
  3d2db7:	41 54                	push   %r12
  3d2db9:	53                   	push   %rbx
  3d2dba:	48 81 ec 88 00 00 00	sub    $0x88,%rsp
  3d2e46:	e8 e5 95 e1 ff       	call   1ec430 <rum_stdlib::associations::basic::LookupOutcome::into_expr>
0000000000305e50 <<alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter>:
  305e50:	55                   	push   %rbp
  305e51:	41 57                	push   %r15
  305e53:	41 56                	push   %r14
  305e55:	41 55                	push   %r13
  305e57:	41 54                	push   %r12
  305e59:	53                   	push   %rbx
  305e5a:	48 83 ec 48          	sub    $0x48,%rsp
  305e5e:	48 89 d3             	mov    %rdx,%rbx
  305f0f:	e8 cc 66 ee ff       	call   1ec5e0 <rum_stdlib::associations::basic::LookupOutcome::to_expr>
```

File-size evidence used for the split assessment:

```text
$ wc -c crates/rum-stdlib/src/associations/basic.rs
28418 crates/rum-stdlib/src/associations/basic.rs
```

Completion scope/whitespace check (`git diff --check` produced no additional
output):

```text
$ git status --short && git diff --check
 M docs/dev/codex/brief-471-basic-key-family-design.md
```
