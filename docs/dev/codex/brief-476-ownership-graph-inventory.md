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

---

## Report — 2026-09-19

### 1. Design-only, no source change

**Measured (work performed):** this is a source-reading and design report, appended
only to this brief. No implementation, build, test, runtime probe, timing, network
access, or git mutation was performed. The original brief is retained; the
append-only check is recorded below. This follows
docs/dev/codex/brief-476-ownership-graph-inventory.md:150 and :167.

**Evidence notation:** **M** means a local source observation, not a runtime
measurement. **M-record** means a measurement reported in an existing repository
document, not reproduced here. **R** means reasoning or a proposed requirement.
All proposed interfaces, filenames, rows, and tests below are **R**, not artifacts
already implemented. This distinction implements
docs/dev/codex/brief-476-ownership-graph-inventory.md:155.

**R — verdict:** use a test-only, cross-module ownership inventory and an exact
per-type disposition ledger. It can independently identify both required
hazards. A clean, exhaustive production check is **not yet established**:
open host-supplied trait objects already prevent closing the graph from these
source files alone. Keep those as fatal unknowns; do not disguise them as
Unbounded debt. The discovery algorithm and the release acceptance predicate
must be separate so that known cycles remain visible even when unknowns exist
(crates/rum-stdlib/src/associations/basic.rs:300;
crates/rum-stdlib/src/associations/key_reshape/traversal.rs:25;
crates/rum-runtime/src/runtime/model/session.rs:95;
docs/dev/codex/brief-476-ownership-graph-inventory.md:51).

### 2. Edge relation and difficult constructs

**R — definition:** identify each nominal struct/enum by crate, resolved module,
and item identity; include private and function-local items, with lexical owner
identity for local names. A field edge A -> B means that destroying an A can
destroy a contained B through that field's storage. Expand storage wrappers,
aliases, tuples and variant payloads, but retain the full field/wrapper witness.
Use a *may-own* graph: a possible populated variant, nonempty container, or final
strong reference is enough. A cycle is a singleton self-edge or a strongly
connected component (SCC) with more than one nominal type. Every member needs its
own row, even if several rows share one certificate. Owners that merely reach an
SCC are reported as affected owners, not incorrectly called SCC members.
This is the field-graph contract of
docs/dev/codex/brief-476-ownership-graph-inventory.md:44 and :59; the actual mutual
example is crates/rum-stdlib/src/associations/key_reshape/traversal.rs:24.

**R — do not erase an edge because Drop exists.** Discover the structural graph
before reading dispositions; then verify how the disposition interrupts its
cycles. Otherwise removing the declaration can also remove the evidence that
a declaration was needed. This matters for the existing separate definition and
destructor in crates/rum-stdlib/src/associations/basic.rs:297 and
crates/rum-stdlib/src/associations/basic/lookup.rs:24.

**R — storage rules (all are proposed scanner rules, grounded in the required
relation at docs/dev/codex/brief-476-ownership-graph-inventory.md:44):**

| Construct | Ownership treatment |
|---|---|
| Direct field, tuple member, enum payload | Visit every component; record variant and field index/name. Commas are AST punctuation, never delimiters in a type regex. |
| Vec<T>, Option<T>, Box<T>, Result<T,E>, owned slices/arrays | Follow every owned payload. A literal zero-length array has no element-drop edge; otherwise retain a conditional element edge. An unhandled const expression is Unknown, not silently zero. |
| Arc<T>, Rc<T> | Follow T with the condition LastStrongHolder. This changes when the hazard fires, not whether it needs a row. A runtime strong-reference cycle that leaks is distinct from an acyclic value chain whose type graph is cyclic. |
| HashMap<K,V,S>, BTreeMap<K,V>, sets, queues | Follow keys and values and any other actually stored owned parameter, including a map's hasher. Use an explicit wrapper signature, not “the first type argument.” |
| &T, &mut T, Weak<T> | No owning edge to T; do not descend through the borrowed/weak target to invent one. The real weak wrapper is crates/rum-core/src/expr.rs:687. |
| Cell/RefCell/Mutex/RwLock/OnceLock and Pin | Model the owned storage; Pin does not suppress destruction. Unsupported wrapper/version/parameter positions are Unknown. |
| PhantomData<T>, raw pointers, ManuallyDrop<T>, MaybeUninit<T>, unions | No automatic pointee/payload-drop edge may be inferred merely from T's spelling. Model known non-dropping storage explicitly; a custom ownership/destructor protocol or an unhandled union is Unknown until audited. This is not permission for new unsafe code (AGENTS.md:109). |
| Primitive scalars, str/string storage, bare function pointers | Known leaves with respect to workspace payload ownership. Function arguments and return types are not stored captures; the existing function-pointer alias is crates/rum-runtime/src/runtime/metadata.rs:332. |

**R — generics:** preserve formal parameter ports and a symbolic ownership
summary, for example Own(0) for a wrapper storing T, and no Own(0) for a wrapper
storing only &T. Substitute through aliases and local wrapper uses before SCC
analysis. Node<u8> containing only T is not a witnessed recursive type.
Root { item: Wrapper<Box<Root>> } closes a cycle through Wrapper's owned port;
Wrapper<&Root> does not. Do not combine unrelated instantiations into an alleged
exact witness. For open owned parameters, report a *conditional generic
obligation* and require a parameterized row conservatively covering the possible
instantiations, including downstream ones; this is a known Own(T) effect, not
a claim that every instantiation recurses. A parameter-independent link such as
PersistentStringSequenceNode<T>::previous is already a concrete nominal
self-cycle irrespective of T (crates/rum-stdlib/src/strings/suspended_state.rs:45;
docs/dev/codex/brief-476-ownership-graph-inventory.md:49).

**R — termination and unknowns:** use finite nominal/parameter summaries with
substitution witnesses, not unlimited monomorphization. Retain “conditional”
rather than upgrading a type-constructor recurrence to an exact monomorphic
cycle. Associated-type projections, unresolvable bounds, opaque impl Trait,
or substitutions that the implemented summary algebra cannot represent produce
a located Unknown and fail. Missing generic support cannot be covered by an
Unbounded row. Fixtures must distinguish generic ownership, borrowed parameters,
mutual substitution, and changing arguments such as Node<Vec<T>>
(docs/dev/codex/brief-476-ownership-graph-inventory.md:49).

**R — trait objects and closures:** an owned dyn Trait requires its concrete
payload ownership graph. A sealed, fully resolved set of implementations can
supply that graph, with each implementation discovered and checked. A list of
currently seen implementations of a public open trait is not a closed set.
A closure owns its actual captures, not the types in its Fn signature; borrowed
captures do not become owning edges. Source-level syn does not establish capture
types/modes in general. Unresolved captures and open owned trait objects must
therefore fail as Unknown. Actual production examples are
Session::print_sink and its public setter
(crates/rum-runtime/src/runtime/model/session.rs:95 and :137),
FileIoProvider::inner (crates/rum-runtime/src/runtime/file_io.rs:95 and :105),
and the suspension frame/finalizer storage
(crates/rum-runtime/src/runtime/suspension/core.rs:17 and :98).
FrameRootTrace bounds do not establish a drop-depth bound
(crates/rum-runtime/src/runtime/suspension/core.rs:76).

**R — foreign types:** scan reachable workspace dependency definitions as support
nodes, including rum-syntax when required; do not stop at the three mandatory
roots. For standard/external crates require an explicit, version-bound ownership
summary: owned parameter positions, terminal payloads, hidden owner edges and
drop protocol. Validate its identity against the toolchain/lock/declaration
evidence and reject stale or missing summaries. BigInt is not a leaf merely
because it is imported; its opaque representation needs such evidence
(crates/rum-core/src/expr.rs:9 and :331). No foreign representation audit was
performed here. External summaries establish graph effects; they are not a
blanket “safe Drop” waiver (docs/dev/codex/brief-476-ownership-graph-inventory.md:51).

### 3. Dispositions and discharge obligations

**R — row schema:** each discovered cyclic nominal type and conditional generic
obligation gets a sorted, unique row with: stable type ID; observed source span;
cycle/field witnesses and generic conditions; disposition; coverage scope;
evidence selectors; reachability provenance; debt owner when applicable; and
drop-ceiling evidence as Measured(reference) or NotMeasured. The scanner owns the
observed edges. The ledger cannot redefine or suppress them. Unknown is a fatal
analysis result, never a disposition. Reject missing rows, extra/stale rows,
duplicate IDs, missing evidence, and unsupported evidence checks. The existing
registry already checks missing/extra rows and debt attribution
(crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:569 and :515).

**R — evidence rules:** references must resolve to actual AST items/fields and
test items. Bind reviewed obligations to exact normalized token snapshots of
the relevant type, destructor/helper, and producer bodies; source changes force
review. A snapshot proves freshness, not safety. A supported structural verifier
must check the claimed detach/drain or cap operation, and targeted certificates
must test its behavior. No rule accepts an arbitrary impl Drop, function name,
reason string, passing shallow example, or “safe” boolean as the whole proof.
This addresses docs/dev/codex/brief-476-ownership-graph-inventory.md:74.

| Proposed disposition (R) | What discharges the declared obligation |
|---|---|
| IterativeDrop { impl, drained_edges, helpers, certificate } | Resolve the actual Drop impl even across files; verify that every edge claimed to break the SCC is detached before child destruction and that the work loop leaves those fields empty. Check that removing the certified edges breaks every covered cycle. Bind helpers and variants to the evidence snapshot. Require a relevant runtime certificate and an in-memory mutation falsifier that removes the drain. Cover ordinary destruction and unwinding, or expose any residual debt rather than claiming unconditional coverage. The concrete candidate is crates/rum-stdlib/src/associations/basic/lookup.rs:24 and :39. |
| ViaIterativeDrop { handler_type, cycle_cut, certificate } | Additional vocabulary for an SCC member with automatic drop whose cycles are interrupted by another member's validated structural handler. Verify the shared whole-SCC certificate, no cycle bypassing the handler, and a bounded native prefix between handlers; forbid delegation to debt rows or circular certificate references. Do not pretend this member has its own Drop impl. The need for per-member treatment is illustrated by the mutual fields at crates/rum-stdlib/src/associations/key_reshape/traversal.rs:25 and :31. |
| CallSiteDiscipline { helper, sites, producers, consumers, certificate, owed_by } | **Allow only as explicit debt**, not as a structural safety classification. Resolve the exact set and multiplicity of calls/references, bind enclosing bodies, and inventory success, error, early-return, replacement, partial construction, and unwind exits. New sites or helper escapes force review. Keep unresolved implicit exits as stated debt. Removing a listed call must invalidate the row. Existing examples: crates/rum-stdlib/src/associations/key_reshape/traversal.rs:84 and :88. |
| BoundedByConstruction { cap, producers, boundary_checks, certificate } | A named finite cap must bound ownership-chain height, with evidence at every constructor/mutator/deserializer and public construction boundary. Verify the supported guard shape and producer-set equality; certificate exercises acceptance at the cap and rejection before constructing cap+1, not a drop-ceiling crash. Bind the evidence to the cap value. A resource default, traversal-front depth, or per-node chunk size does not discharge this. Counterexamples: crates/rum-stdlib/src/strings/patterns.rs:746; crates/rum-stdlib/src/structure_ops/suspended_state.rs:73 and :82. |
| Unbounded { owed_by, decision, reachability, ceiling } | Discharges **classification only**. Require a nonempty owning debt/decision reference that exists and names this obligation, an explicit hazard mechanism, and honest measured/reasoned reachability and ceiling fields. “Exempt” without ownership is invalid. This mirrors crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:515 and the explicit debt at TODO.md:1197. |

**R — conditional ownership:** all dispositions must cover the LastStrongHolder
case. “Usually shared” is not a bound. An iterative Arc/Rc drain must handle both
uniquely owned and shared nodes and the eventual last release; evidence for only
the first release is insufficient. A type's certified self-cycle does not confer
safety on other payloads: LookupOutcome's Expr leaves retain Expr's separate
debt (crates/rum-stdlib/src/associations/basic.rs:298;
crates/rum-core/src/expr.rs:574;
docs/dev/codex/brief-476-ownership-graph-inventory.md:159).

**R — NestedPath row:** create a row for each of NestedPathNodes and
NestedPathNode, sharing one CallSiteDiscipline debt record. Its current explicit
helper-call set is the two branches in association_key_deflatten_entries:
invalid path at :84 and failed insertion at :88; success transfers to the
renderer at :93. Bind the whole enclosing function and consuming helpers, not
just a count of two. The cleanup drains entries at :224 and :227. The existing
certificate names the partial-tree error case, but no general unwind proof is
established here. Thus the row records conditional cleanup plus an obligation
to replace/audit it structurally, not universal bounded destruction
(crates/rum-stdlib/src/associations/key_reshape/traversal.rs:77 and :222;
crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:310 and :325).
Neither that depth certificate nor any ceiling probe was run.

### 4. Location, coverage, cost, and implementation sequence

**R — home:** put a sibling ownership_graph test module under rum-stdlib's
tests. syn with full/visit already exists there; rum-runtime's complete manifest
has only normal dependencies and its two feature declarations
(crates/rum-stdlib/Cargo.toml:25;
crates/rum-runtime/Cargo.toml:1).
The existing test root is cfg(test), and the association inventory is already
wired into it (crates/rum-stdlib/src/lib.rs:152;
crates/rum-stdlib/src/tests.rs:7).
rum-derive demonstrates AST validation and compile-error reporting, but a derive
on an annotated type cannot discover an unannotated type or a workspace-wide
mutual cycle. Keep whole-source enumeration independent of opt-in derives
(crates/rum-derive/src/lib.rs:45;
docs/dev/codex/brief-476-ownership-graph-inventory.md:6).

**R — source closure:** enumerate all production modules rooted at the lib.rs
of rum-core, rum-runtime and rum-stdlib, following inline modules, mod files,
path attributes, imports, re-exports, aliases and lexical scopes. Reconcile
against all .rs files under their src directories; every file is either covered,
provably test-only, or a located unresolved coverage error. Do not use git's
tracked-file list. Evaluate cfg with test=false and cover every supported
production feature/target alternative, retaining conditions rather than unioning
incompatible variants into a claimed concrete witness. Unresolved cfg or
cfg_attr is fatal. Local types inside functions count. Unhandled generated
items, include!, type macros and attribute/procedural macro effects are fatal
unless expansion or a pinned, checked expansion contract accounts for them.
The existing same-file call scanner is not an adequate resolver to copy
(crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:3
and :20; crates/rum-stdlib/src/lib.rs:153;
crates/rum-stdlib/Cargo.toml:12).

**R — “types are closed” has a boundary:** ordinary explicit field lists are
closed; their ownership interpretation is not necessarily closed. Aliases,
owned generic ports, external representations, macros and open dyn payloads
cross that boundary. Actual aliases include a three-parameter HashMap and an
aliased dyn SuspensionFrame; actual macros include resume_or_suspend
(crates/rum-runtime/src/runtime/model/cache.rs:13;
crates/rum-runtime/src/runtime/suspension/core.rs:9 and :658).
A compiler-assisted resolver could improve name/capture resolution but would
not reveal arbitrary future host callback captures
(crates/rum-runtime/src/runtime/model/session.rs:137).

**R — steps, files/functions and per-step gates** (new paths/functions below are
proposed, following docs/dev/codex/TEMPLATE.md:19):

1. **source.rs / collect_production_sources; resolve.rs / resolve_type.**
   Build canonical identities, cfg-aware coverage and aliases/imports, with a
   read-only in-memory source provider for fixtures. Gate: cross-file,
   re-export/glob, duplicate-name, local-type, path-attribute, production-only
   cfg and missing-file fixtures. Preserve production modules and APIs
   (crates/rum-stdlib/src/associations/basic.rs:11).
2. **effects.rs / ownership_effect; graph.rs / discover_cycles.**
   Build located edges and Unknown records; use parameter summaries and an
   iterative SCC algorithm. Gate: every storage rule above, both real hazards,
   generic/alias/mutual fixtures, and opaque-construct rejection. Return facts
   even when there are errors; never return a false empty success
   (crates/rum-stdlib/src/associations/basic.rs:300;
   docs/dev/codex/brief-476-ownership-graph-inventory.md:51).
3. **registry.rs / validate_rows; evidence.rs / validate_obligation.**
   Exact row equality, per-disposition structural/evidence checks, and explicit
   Expr/persistent/call-site debt. Gate: delete/add/duplicate rows, empty debt
   references, stale evidence, changed cap, removed drain/call and a lying
   IterativeDrop declaration. No production Drop rewrite is part of this step
   (crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:515 and :595).
4. **mod.rs / production_ownership_is_classified; fixtures.rs.**
   Wire through tests.rs, require all roots and zero Unknowns, and run the
   independent falsifiers below. Emit full diagnosed failures if closure remains
   impossible; never silently baseline unknowns. Gate: focused module suite,
   existing association enforcement suite, workspace tests and formatting
   (crates/rum-stdlib/src/tests.rs:7; AGENTS.md:86).

**R — ripple and cost:** normal production code, layout, Drop and dispatch stay
unchanged because the checker is under the existing cfg(test) root. Test builds
pay source parsing, resolution, summaries and registry validation. Once the
finite resolved graph is built, iterative SCC detection takes O(V+E) work and
storage; that is not a bound on the resolver/generic analysis. Cache parsed
files, use deterministic ordering, and report coverage counts plus Unknowns.
No timing or whole-workspace type count was measured here. Add only a dev
proc-macro2 dependency with span-locations if needed for accurate path:line
diagnostics; do not infer line numbers by ambiguous substring matches. This
library already exists in the workspace dependency graph
(crates/rum-derive/Cargo.toml:14). No hot-path performance claim or benchmark is
needed for the proposed test-only patch; any later production fix must obey
AGENTS.md:117.

**R — shipping contract:** a passing mandatory test/release gate establishes
classification, not absence of accepted debt. Ordinary cargo build alone would
not enforce this test-only design. The release workflow must run the gate;
this report does not establish that such a mandatory release hook already
exists. A compile-time guarantee would need separately scoped build integration
and cannot be promised by an optional derive
(crates/rum-stdlib/src/lib.rs:152;
docs/dev/codex/brief-476-ownership-graph-inventory.md:6).

### 5. Expr and Persistent*Node rows

**M-record — Expr decision confirmed:** TODO.md:1197 records native-stack
overflow on direct Rust-constructed deep Expr destruction and three rejected
iterative Drop designs with 20–50% hot-path regressions. The host contract repeats
that limitation and tells hosts to depth-limit direct construction
(docs/host-embedding-and-hardening.md:125). These are existing recorded findings;
no performance experiment or historical artifact reproduction was performed here.

**R — the row cannot be only Expr:** Expr::Call owns SharedExpr and SharedArgs;
those own Arc<Expr> and Arc<Vec<Expr>> respectively. All three are cyclic nominal
members (crates/rum-core/src/expr.rs:574, :638 and :750).
Proposed explicit rows:

| Type ID (R) | Disposition and evidence |
|---|---|
| rum-core::expr::Expr | Unbounded; owed_by = standing engineering debt at TODO.md:1197; decision = three rejected Drop designs at TODO.md:1199; reachability = direct Rust construction (M-record); drop ceiling = NotMeasured here. |
| rum-core::expr::SharedExpr | Unbounded; same debt/decision; edge condition = LastStrongHolder; witness = tuple field at crates/rum-core/src/expr.rs:574 back to Expr. |
| rum-core::expr::SharedArgs | Unbounded; same debt/decision; edge condition = LastStrongHolder, nonempty Vec; witness = tuple field at crates/rum-core/src/expr.rs:638 back to Expr. |

**R:** these are narrow, reviewable exemptions from remediation, not from
discovery or classification. Keep the structural witnesses visible. Source-depth
guards do not prove BoundedByConstruction for all direct Rust construction, and
the rejection decision does not justify ignoring unrelated new recursive types
(TODO.md:1197; docs/host-embedding-and-hardening.md:129).

**M-record — persistent family settled:** the recorded result is
**source-reachable and unbounded**, with two string-resume constructions reaching
tail length 1,000 after one suspension; limits of 8 for both generated elements
and iteration work did not cap those loops (TODO.md:2504 and :2511).
The scalar replacement limit defaults to usize::MAX
(crates/rum-stdlib/src/strings/patterns.rs:746).
The current source shows the append-per-character paths at
crates/rum-stdlib/src/strings/patterns/search/rules.rs:348 and
crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:288.
The recorded drop ceiling is still unmeasured (TODO.md:2525).
I did not repeat the reachability probes or estimate a ceiling.

**R — materialized family ledger:** create one explicit Unbounded row for
*every* cyclic node declaration in the evidence-C shortlist below; a wildcard
Persistent* entry cannot satisfy exact coverage. For the previous-linked family,
use the following shared decision record, with each row's own field witness:

~~~text
disposition: Unbounded
owed_by: "Persistent node destruction debt — TODO.md:2502"
decision: TODO.md:2511
mechanism: previous -> Option -> Arc(last strong holder) -> same nominal node
ownership_depth_cap: absent; NOT a traversal-depth or resource-limit exemption
family_reachability: SourceLanguage, M-record(TODO.md:2504)
witnessed_member: PersistentStringTextNode,
    crates/rum-stdlib/src/strings/suspended_state.rs:101
other_members_individual_source_witnesses: not established by this report
drop_ceiling: NotMeasured, director-owned bracket (TODO.md:2525)
~~~

**R:** this preserves the affirmative family reachability result without
pretending that each member was independently exercised: TODO.md:2528 explicitly
says eight of eleven families remain unexamined. The string node append operation
adds a predecessor without a chain-depth check
(crates/rum-stdlib/src/strings/suspended_state.rs:116).
Other rows need their own constructor evidence during implementation; none may
be called bounded merely for using Arc or having a len field
(docs/dev/codex/brief-476-ownership-graph-inventory.md:116 and :159).

**M — small count, explicitly not the complete inventory:** the read-only
lexical shortlist in evidence C printed **16 Persistent*Node declarations in
11 files: 15 previous fields and one next field**. I hand-checked the actual
declaration blocks in four files: strings has two
(crates/rum-stdlib/src/strings/suspended_state.rs:45 and :101);
associations has two
(crates/rum-stdlib/src/associations/suspended_state.rs:57 and :111);
structure_ops has three
(crates/rum-stdlib/src/structure_ops/suspended_state.rs:56, :150 and :234);
rewrite_data has two
(crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:57
and :105).
This cross-check is deliberately a check of the shortlist, not a claim that
regex proves absence of other recursive types. The earlier “45” is not used as
a current completeness target
(docs/dev/codex/brief-476-ownership-graph-inventory.md:32 and :161).

**R — two useful distinctions:** PersistentRewriteQueueNode uses **next**,
and freeze links one node per input element, so it needs a separate field
witness and an Unbounded debt row even though an appended/previous search would
miss it; its individual source-language reachability is not established here
(crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:57
and :63). PersistentStructureSequence chunks values within a node but still
links a new predecessor after the chunk fills; that is not an ownership-depth
cap (crates/rum-stdlib/src/structure_ops/suspended_state.rs:71 and :82).
Do not transfer the measured string reachability or any other type's drop
ceiling to these members.

### 6. Falsifier and the two required discoveries

**R — confirmation by tracing the proposed rules, not an executed checker:**
both required cases follow without special-casing their names. Discovery takes
only source and wrapper semantics, never registry type names. The following
witnesses are derived directly from the field ASTs:

~~~text
LookupOutcome
  -- List.0 / Vec element / resolve Self --> LookupOutcome

NestedPathNodes
  -- entries / Vec element / tuple.1 --> NestedPathNode
NestedPathNode
  -- Branch.0 --> NestedPathNodes
~~~

**R:** the first AST is in
crates/rum-stdlib/src/associations/basic.rs:297, including Vec<Self> at :300.
Its Drop is separately located in
crates/rum-stdlib/src/associations/basic/lookup.rs:24.
The second witness uses both definitions at
crates/rum-stdlib/src/associations/key_reshape/traversal.rs:24 and :30.
Therefore generic structural traversal plus SCCs finds both hazards; literal
self-name searches and per-file destructor association are insufficient.
Actual executable rediscovery remains an acceptance test for the implementation,
as required by docs/dev/codex/brief-476-ownership-graph-inventory.md:125.

**R — required independent falsifiers** (test-only/in-memory mutations, grounded
in the existing real-row misdeclaration pattern at
crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:595):

1. Analyze the real production source provider with **no registry input**;
   assert the two witnesses and all three type IDs above occur in discovery.
   Unknowns elsewhere must be retained but must not suppress these facts.
   Then validate with each corresponding row removed in turn and require a
   diagnostic naming the type, defining file/line and field cycle. Deleting
   only one member of the mutual SCC must fail
   (crates/rum-stdlib/src/associations/basic.rs:300;
   crates/rum-stdlib/src/associations/key_reshape/traversal.rs:25 and :31).
2. Change only the in-memory LookupOutcome Drop/helper to leave its List
   payload intact; retaining IterativeDrop must fail its obligation check.
   Remove either in-memory NestedPath cleanup call while retaining the declared
   sites; fail with the missing site. This requires no deep values or process
   crash (crates/rum-stdlib/src/associations/basic/lookup.rs:33 and :42;
   crates/rum-stdlib/src/associations/key_reshape/traversal.rs:84 and :88).
3. Inject a freshly named recursive type, a mutual pair split across modules,
   and map/tuple fields with recursion after a comma into an in-memory source
   tree. Empty the provider or remove a mandatory root: coverage must fail,
   not return a clean zero. These are liveness and anti-truncation requirements,
   not approved-type-name seeds
   (docs/dev/codex/brief-476-ownership-graph-inventory.md:32 and :54).
4. Inject only borrowed/Weak recurrence and bare fn parameter/return mentions:
   require no payload ownership cycle. Inject an unmodeled owned dyn field,
   unresolved alias/projection or generated type: require Unknown and a failing
   acceptance result. Assert Unknown cannot be neutralized by adding an
   Unbounded row (docs/dev/codex/brief-476-ownership-graph-inventory.md:47 and :51;
   crates/rum-core/src/expr.rs:687;
   crates/rum-runtime/src/runtime/metadata.rs:332).

**R — certificate limits:** the real Lookup test declares height 4,097 and a
2 MiB stack; the NestedPath certificate documents a prior crashing recursive
drop and is ignored by default
(crates/rum-stdlib/src/associations/basic/lookup.rs:848;
crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:319 and :324).
Their presence is source evidence, not a test run in this report. Do not run a
broken destructor on either deep fixture to prove this new inventory fires;
mutate source inputs/rows in memory and assert analysis errors instead
(docs/dev/codex/brief-476-ownership-graph-inventory.md:151).

### 7. Draft implementation brief — ownership-graph enforcement

**R — draft status:** the following is review material in TEMPLATE.md shape,
not authorization to implement during brief-476. Before dispatch, the director
must resolve the open-payload closure blocker below and supply bounded-command
timeouts from prior gate evidence (docs/dev/codex/TEMPLATE.md:151). Do not dispatch it with an expectation that
a source-only scan of open host captures will return clean
(docs/dev/codex/TEMPLATE.md:32;
crates/rum-runtime/src/runtime/model/session.rs:137).

#### Goal

**R:** add a test-only ownership inventory covering every production definition
in rum-core, rum-runtime and rum-stdlib, plus necessary support definitions.
Discover structural and conditional generic ownership obligations independently
of the ledger. Require exactly one checked row per obligation/type, no unknown
analysis constructs, and independent rediscovery of LookupOutcome and the
NestedPathNodes/NestedPathNode SCC. Passing means classified with explicit debt,
not that all destruction is bounded
(docs/dev/codex/brief-476-ownership-graph-inventory.md:6, :51 and :125).

#### Context pointers

**R — read these exact anchors before implementation:**

- AGENTS.md:36 and :90 — own-contract testing and bounded commands.
- crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:196
  and :832 — syn parsing and a synthetic falsifier; its scope at :20 is narrower
  than the new checker and must not be copied.
- crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:515 and :595 —
  required debt ownership and a real-row falsifier.
- crates/rum-stdlib/src/associations/basic.rs:297 and
  crates/rum-stdlib/src/associations/basic/lookup.rs:24 — cross-file self-cycle
  and structural Drop.
- crates/rum-stdlib/src/associations/key_reshape/traversal.rs:24 and :77 —
  mutual cycle and two explicit cleanup sites.
- TODO.md:1197 and :2504 — Expr debt and the established persistent reachability.
- crates/rum-runtime/src/runtime/model/session.rs:95 and
  crates/rum-runtime/src/runtime/file_io.rs:95 — open ownership inputs that
  cannot be treated as leaves.
- crates/rum-derive/src/lib.rs:45 — validation precedent, not the discovery home.

#### Scope

**R — proposed allowed files only, derived from the test wiring at
crates/rum-stdlib/src/lib.rs:152 and crates/rum-stdlib/src/tests.rs:7:**

- crates/rum-stdlib/src/tests.rs — add the ownership_graph test module.
- New crates/rum-stdlib/src/tests/ownership_graph/mod.rs.
- New crates/rum-stdlib/src/tests/ownership_graph/source.rs.
- New crates/rum-stdlib/src/tests/ownership_graph/resolve.rs.
- New crates/rum-stdlib/src/tests/ownership_graph/effects.rs.
- New crates/rum-stdlib/src/tests/ownership_graph/graph.rs.
- New crates/rum-stdlib/src/tests/ownership_graph/registry.rs.
- New crates/rum-stdlib/src/tests/ownership_graph/evidence.rs.
- New crates/rum-stdlib/src/tests/ownership_graph/fixtures.rs.
- crates/rum-stdlib/Cargo.toml — only the dev proc-macro2 span-locations
  dependency for source diagnostics, if needed.
- Cargo.lock — only the direct dev-dependency bookkeeping above; no package
  upgrades. proc-macro2 is already used (crates/rum-derive/Cargo.toml:14).
- The future dispatched brief — append its report only.

**R:** no production source, public API, Expr layout/serde, Drop, evaluator,
dispatch, builtin catalog, runtime dependencies, existing depth inventories,
AGENTS.md, TODO.md or other documentation changes. No unknown allowlist, wildcard
type exemption, runtime instrumentation or dependency/network fetches.
An API change to close opaque ownership is outside this scope and requires
a rewritten brief before implementation. These boundaries follow
docs/dev/codex/TEMPLATE.md:55 and the no-default-cost requirement at AGENTS.md:117.

#### Deliverable

**R — implement the four steps in report section 4, with these reviewable
outputs (docs/dev/codex/TEMPLATE.md:19):**

- Separate discovery facts from validate_rows/acceptance. Discovery is not
  seeded by row IDs or known-hazard names; diagnostics retain every discovered
  cycle and Unknown, with exact source spans.
- Per-root coverage, field witnesses, SCC membership, owned generic-port
  obligations, resolved Drop implementations and explicit unknown diagnostics.
  Prove borrowed/Weak fields are excluded
  (docs/dev/codex/brief-476-ownership-graph-inventory.md:44).
- Materialized rows for all discovered obligations; the Expr SCC has three
  separate debt rows, persistent links are not marked bounded, and NestedPath
  cleanup is conditional debt with the verified two-site set. Require evidence
  for each row, not just the initial examples
  (crates/rum-core/src/expr.rs:752;
  crates/rum-stdlib/src/associations/key_reshape/traversal.rs:84;
  TODO.md:2515).
- Independent real-source rediscovery and each falsifier in report section 6.
  Mutate only in-memory source/rows. Include deliberate broken-drain and
  stale-call-set cases
  (crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:595).
- A source-closure report identifying any unresolved macros, cfg, foreign
  representations, generic projections and open trait/closure payloads.
  If any remain, the production acceptance test must fail and the handoff must
  say the implementation is blocked; do not mark the task green by weakening
  unknown handling
  (docs/dev/codex/brief-476-ownership-graph-inventory.md:51).

#### rum / Rust pitfalls (read twice)

**R — implementation constraints and their anchors:**

1. Resolve Self within its owning nominal definition and substitute generic
   ports without splitting textual types at commas. Do not flatten module
   names or confuse duplicate local names
   (crates/rum-stdlib/src/associations/basic.rs:300;
   crates/rum-runtime/src/runtime/model/cache.rs:13).
2. LastStrongHolder is a real owning edge; Weak is not. A traversal cap is not
   an ownership cap, and chunking does not cap predecessor count
   (crates/rum-core/src/expr.rs:638 and :687;
   crates/rum-stdlib/src/structure_ops/suspended_state.rs:82).
3. Keep cfg(test) out of the production graph while following every production
   alternative. Do not let parsed test fixtures become production rows
   (crates/rum-stdlib/src/lib.rs:152).
4. A Drop impl, trace derive, evidence snapshot or certificate name alone does
   not prove iterative destruction. Require the supported structural evidence,
   residual-payload classification, and the corresponding negative mutation
   (crates/rum-stdlib/src/associations/basic/lookup.rs:33 and :42).
5. Do not claim a graph is closed around Box<dyn FnMut> or an open provider
   trait. No handwritten “external leaf” entry can erase an unresolved payload
   (crates/rum-runtime/src/runtime/model/session.rs:95;
   crates/rum-runtime/src/runtime/file_io.rs:99).
6. Do not run drop-ceiling probes, disabled broken runtime destructors, external
   engine comparisons, or ad hoc timing. Only AST/ledger mutants are falsifiers
   in this brief (AGENTS.md:41;
   docs/dev/codex/brief-476-ownership-graph-inventory.md:151).
7. If the same implementation step fails twice for the same reason, stop and
   report the defect rather than weakening the gate
   (docs/dev/codex/TEMPLATE.md:99).

#### Verification (MANDATORY for the future implementation)

**R — required gates:** give the following tests stable names under
tests::ownership_graph; these are proposed names, not tests run in brief-476.
They operationalize the required falsifier at
docs/dev/codex/brief-476-ownership-graph-inventory.md:125:

| Proposed test | Required result |
|---|---|
| production_ownership_is_classified | Every mandatory root covered; zero Unknowns; exact row coverage; all evidence checks pass. |
| real_sources_rediscover_self_and_mutual_cycles | Production AST provider finds LookupOutcome and both NestedPath types without registry input. |
| removing_each_real_hazard_row_is_rejected | Each removed row, including either mutual member, names its missing obligation and source witness. |
| ownership_storage_and_generic_effects | Positive/negative wrapper, map-key/value/hasher, tuple, array, Self, generic and alias fixtures satisfy section 2. |
| source_coverage_and_resolution_fail_closed | Empty/missing roots, ambiguous imports, unsupported cfg/macro/foreign/projection constructs fail with locations; valid cross-file/local/path cases resolve. |
| owned_opaque_payloads_are_fatal_unknowns | Open dyn/capture ownership fails even when an Unbounded row is supplied. |
| disposition_evidence_mutants_are_rejected | Missing/no-op drain, removed/extra cleanup call, stale evidence, incorrect bound, duplicate/stale row and empty debt owner fail. |

**R — commands:** before dispatch, fill OWNERSHIP_GATE_TIMEOUT_SECONDS from
the director's prior runs for these command shapes; no numerical timeout was
measured by this design report. Run all potentially long commands through the
repository wrapper (AGENTS.md:90). For the proposed implementation:

~~~bash
python3 scripts/run-bounded.py --timeout "$OWNERSHIP_GATE_TIMEOUT_SECONDS" -- cargo fmt --check
python3 scripts/run-bounded.py --timeout "$OWNERSHIP_GATE_TIMEOUT_SECONDS" -- cargo test --offline -p rum-stdlib ownership_graph -- --nocapture
python3 scripts/run-bounded.py --timeout "$OWNERSHIP_GATE_TIMEOUT_SECONDS" -- cargo test --offline -p rum-stdlib --no-default-features ownership_graph -- --nocapture
python3 scripts/run-bounded.py --timeout "$OWNERSHIP_GATE_TIMEOUT_SECONDS" -- cargo test --offline -p rum-stdlib association_depth_contract
python3 scripts/run-bounded.py --timeout "$OWNERSHIP_GATE_TIMEOUT_SECONDS" -- cargo test --offline --workspace
~~~

**R:** the no-default-features run checks that enforcement is not accidentally
gated by the optional hash family (crates/rum-stdlib/Cargo.toml:12).
Require nonzero selected test counts; paste the complete focused results and
workspace summaries, including Unknown diagnostics if failing. Do not turn
production_ownership_is_classified into an expected-failure or ignored test
to make the suite green. If cached dependencies are unavailable, report that
rather than accessing the network. The draft has no runtime or performance
change; a separately authorized hot-path fix would require the A/B gate in
AGENTS.md:117. No ignored depth/ceiling certificates are part of these commands
(crates/rum-stdlib/src/associations/basic/lookup.rs:848;
crates/rum-stdlib/src/tests/association_key_reshape_depth.rs:324).

#### Hard rules

**R — carry these into the dispatched brief unchanged** (current hard rules:
docs/dev/codex/brief-476-ownership-graph-inventory.md:170;
template: docs/dev/codex/TEMPLATE.md:125):

- Do not git commit, branch, push, or touch git config. Leave the tree for the
  director's authoritative gate.
- Modify only the listed files. Preserve unrelated work. No production fixes
  bundled into the inventory.
- Touch nothing outside the repository; no network. Use bounded commands;
  no deep runtime/ceiling probes.
- Testing Policy is a hard line: rum's own contracts only, no black-box
  comparisons, oracle harnesses or third-party engine outputs (AGENTS.md:38).
- No unsafe, no new default-path instrumentation or runtime dependency cost
  (AGENTS.md:109 and :117).
- Unknown means failure. No allowlist, expected-failure production test,
  hidden root exclusion or undocumented foreign-type leaf
  (docs/dev/codex/brief-476-ownership-graph-inventory.md:51).
- Paste actual verification output. Mark measurements, recorded measurements
  and reasoning separately; report blockers without inventing clean results
  (docs/dev/codex/brief-476-ownership-graph-inventory.md:155 and :178).

#### Report format

**R — future handoff, following docs/dev/codex/TEMPLATE.md:161:**

1. Per-file change and purpose, source coverage, row count and Unknown count.
2. Verbatim verification commands/results and real-source cycle witnesses.
3. Falsifier outcomes, disposition/evidence checks, and debt rows retained.
4. Source-closure blockers, unsupported constructs, scope/stop-loss decisions,
   and anything still unmeasured.

### 8. Risks and open questions

**R — blocking closure decision:** a source-only check cannot certify arbitrary
host captures supplied through set_print_sink or FileIoProvider::from_arc.
This is an existing API boundary, not a hypothetical future syntax edge
(crates/rum-runtime/src/runtime/model/session.rs:137;
crates/rum-runtime/src/runtime/file_io.rs:105).
The strict rule therefore produces real Unknowns today. The director needs
a separately reviewed ownership contract/representation for open payloads,
or a separately authorized policy change; neither is silently adopted here.
Compiler name resolution alone does not close an open host API. Until the
boundary is decided and checkable, “every unclassified type cannot ship” cannot
honestly be declared achieved for a green full-production inventory under this
draft's test-only scope
(docs/dev/codex/brief-476-ownership-graph-inventory.md:6 and :51).

**R — unknown is not debt:** known structural recursion with unbounded depth is
classifiable debt; unknown ownership effects are analysis failures. Unknown
drop ceilings and unmeasured per-member reachability may remain explicit
evidence fields without pretending the edge relation is unknown. This keeps
the persistent-family debt visible while enforcing the unknown-construct rule
(TODO.md:2515 and :2525;
docs/dev/codex/brief-476-ownership-graph-inventory.md:51).

**R — proof limits:** general destructor behavior, macro expansion, const/cfg
reasoning and generic substitution are not solved by syn parsing alone. Each
supported evidence rule needs a defined check and a negative fixture; unsupported
proof shapes fail. Call-site equality and snapshots detect drift but do not
prove every implicit destruction path safe. This is why that disposition remains
debt and why Expr leaf hazards survive a LookupOutcome cycle certificate
(crates/rum-stdlib/src/associations/key_reshape/traversal.rs:84;
crates/rum-stdlib/src/associations/basic.rs:298;
docs/dev/codex/brief-476-ownership-graph-inventory.md:74).

**R — gate and measurement limits:** this design has no executable scanner yet,
no exhaustive workspace SCC count, no benchmark, and no new drop-ceiling
measurement. Test-only enforcement also depends on the gate actually being
mandatory before release. The two known hazards are established rule-level
witnesses; the drafted falsifier must still demonstrate executable rediscovery.
The persistent drop ceiling remains the director's measurement
(docs/dev/codex/brief-476-ownership-graph-inventory.md:116, :125 and :167).

### Verbatim evidence for this report

**M:** the following are actual outputs from read-only commands. Source excerpts
were printed by Python using Path.read_text().splitlines(), with the original
one-based source line numbers retained; disjoint ranges are visible in the
numbers. They are source observations, not execution of the proposed checker.
The original brief's lines 1–191 supply the requirement citations without
duplicating that text here
(docs/dev/codex/brief-476-ownership-graph-inventory.md:167).

#### Evidence A — Fields, destruction mechanisms and recorded decisions

<details>
<summary>Read-only command</summary>

~~~bash
python3 -B - <<'PY'
from pathlib import Path
sections = [["crates/rum-core/src/expr.rs",[[574,574],[638,638],[687,687],[750,753]]],["crates/rum-stdlib/src/associations/basic.rs",[[11,11],[296,301]]],["crates/rum-stdlib/src/associations/basic/lookup.rs",[[24,45],[847,863]]],["crates/rum-stdlib/src/associations/key_reshape/traversal.rs",[[23,33],[77,94],[206,230]]],["TODO.md",[[1195,1201],[2504,2516],[2525,2529]]],["docs/host-embedding-and-hardening.md",[[125,129]]],["crates/rum-stdlib/src/strings/suspended_state.rs",[[60,69],[116,125]]],["crates/rum-stdlib/src/strings/patterns.rs",[[743,747]]],["crates/rum-stdlib/src/strings/patterns/search/rules.rs",[[258,265],[313,318],[345,352]]],["crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs",[[268,275],[284,291]]],["crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs",[[62,70]]],["crates/rum-stdlib/src/structure_ops/suspended_state.rs",[[71,87]]]]
for name, ranges in sections:
    lines = Path(name).read_text().splitlines()
    print(name)
    for first, last in ranges:
        for number in range(first, last + 1):
            print(f'{number}: {lines[number - 1]}'.rstrip())
PY
~~~

</details>

~~~text
crates/rum-core/src/expr.rs
574: pub struct SharedExpr(Arc<Expr>);
638: pub struct SharedArgs(Arc<Vec<Expr>>);
687: pub struct WeakSharedArgs(Weak<Vec<Expr>>);
750: pub enum Expr {
751:     Atom(Atom),
752:     Call { head: SharedExpr, args: SharedArgs },
753: }
crates/rum-stdlib/src/associations/basic.rs
11: mod lookup;
296: #[derive(Clone, Debug, FrameRootTrace)]
297: enum LookupOutcome {
298:     Hit(Expr),
299:     Missing(Expr),
300:     List(Vec<Self>),
301: }
crates/rum-stdlib/src/associations/basic/lookup.rs
24: impl Drop for LookupOutcome {
25:     fn drop(&mut self) {
26:         let Self::List(items) = self else {
27:             return;
28:         };
29:         if items.is_empty() {
30:             return;
31:         }
32:
33:         drop_lookup_outcomes_iterative(std::mem::take(items));
34:     }
35: }
36:
37: #[cold]
38: #[inline(never)]
39: fn drop_lookup_outcomes_iterative(mut work: Vec<LookupOutcome>) {
40:     while let Some(mut outcome) = work.pop() {
41:         if let LookupOutcome::List(items) = &mut outcome {
42:             work.append(items);
43:         }
44:     }
45: }
847:     #[test]
848:     #[ignore = "release-only deployment-stack drop certificate; run explicitly with --release"]
849:     fn lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib() {
850:         let joined = std::thread::Builder::new()
851:             .name("lookup-outcome-drop-depth-4097".to_string())
852:             .stack_size(2 * 1024 * 1024)
853:             .spawn(|| {
854:                 let mut outcome = LookupOutcome::Missing(Expr::string("deep-drop-witness"));
855:                 for _ in 1..4_097 {
856:                     outcome = LookupOutcome::List(vec![outcome]);
857:                 }
858:
859:                 drop(outcome);
860:             })
861:             .expect("2 MiB LookupOutcome drop certificate thread should spawn")
862:             .join();
863:         assert!(joined.is_ok(), "LookupOutcome drop certificate panicked");
crates/rum-stdlib/src/associations/key_reshape/traversal.rs
23: #[derive(Clone, Debug, PartialEq, Eq)]
24: enum NestedPathNode {
25:     Branch(NestedPathNodes),
26:     Leaf { value: Expr, delayed: bool },
27: }
28:
29: #[derive(Clone, Debug, PartialEq, Eq)]
30: struct NestedPathNodes {
31:     entries: Vec<(Expr, NestedPathNode)>,
32:     positions: Option<HashMap<Expr, usize>>,
33: }
77: pub(super) fn association_key_deflatten_entries(
78:     entries: Vec<AssociationEntryForm>,
79: ) -> Option<Expr> {
80:     let mut nested = NestedPathNodes::new();
81:
82:     for entry in entries {
83:         let Some(path) = path_from_key(entry.key) else {
84:             drop_nested_path_nodes_iterative(nested);
85:             return None;
86:         };
87:         if insert_path(&mut nested, &path, entry.value, entry.delayed).is_none() {
88:             drop_nested_path_nodes_iterative(nested);
89:             return None;
90:         }
91:     }
92:
93:     Some(render_nested_path_entries(nested))
94: }
206: fn nested_path_node_for_missing_suffix(
207:     remaining: &[Expr],
208:     value: Expr,
209:     delayed: bool,
210: ) -> NestedPathNode {
211:     let mut node = NestedPathNode::Leaf { value, delayed };
212:     for key in remaining.iter().rev() {
213:         let mut children = NestedPathNodes::new();
214:         children.push(key.clone(), node);
215:         node = NestedPathNode::Branch(children);
216:     }
217:     node
218: }
219:
220: #[cold]
221: #[inline(never)]
222: fn drop_nested_path_nodes_iterative(mut nodes: NestedPathNodes) {
223:     let mut work = Vec::new();
224:     work.append(&mut nodes.entries);
225:     while let Some((_key, node)) = work.pop() {
226:         if let NestedPathNode::Branch(mut children) = node {
227:             work.append(&mut children.entries);
228:         }
229:     }
230: }
TODO.md
1195: ## Standing engineering debt
1196:
1197: - Deep `Expr` trees constructed directly through the Rust API can overflow the
1198:   native stack on drop. Source-created trees are depth-guarded; three iterative
1199:   `Drop` designs regressed the hot path by 20-50% and were rejected. Keep the
1200:   host limitation documented in
1201:   [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md).
2504: Settled by measurement, 2026-09-19. **A source-language construction exists**, on
2505: the string-replacement resume paths, and the tail grows **per element** — one
2506: suspension, then a node per character — so it does **not** need repeated host
2507: round trips. Two constructions reached tail length 1 000, each verified by
2508: instrumentation planted in the `appended` path and then removed; the tree was
2509: left clean.
2510:
2511: **Nothing caps it.** The worker set the generated-element limit and the
2512: iteration-work limit **both to 8** and the sequence still reached 1 000: those
2513: suffix loops enforce neither. The scalar replacement count defaults to
2514: `usize::MAX` (`crates/rum-stdlib/src/strings/patterns.rs:746`). So the honest row
2515: for the family in brief-476's inventory is **unbounded and reachable**, not
2516: `Unbounded(owed_by = …)` pending an answer.
2525: **Still unmeasured, and first thing next session**: the drop ceiling. The brief
2526: deliberately forbade driving it to a crash — that bracket is the director's —
2527: so we know the chain can be made arbitrarily long and do **not** yet know at
2528: what length its drop stops being survivable. Eight of the eleven families are
2529: also still unexamined.
docs/host-embedding-and-hardening.md
125: - Source-created trees are protected by parser, evaluator, matcher, and native-
126:   stack limits. A host can still construct a pathologically deep `Expr` directly
127:   through the Rust API and overflow the stack during recursive drop. Three safe
128:   iterative-drop designs regressed the normal hot path by 20-50% and were
129:   rejected; hosts must depth-limit directly constructed trees.
crates/rum-stdlib/src/strings/suspended_state.rs
60:     pub(super) fn appended(&self, value: T) -> Self {
61:         Self {
62:             prefix: self.prefix.clone(),
63:             tail: Some(Arc::new(PersistentStringSequenceNode {
64:                 previous: self.tail.clone(),
65:                 value,
66:             })),
67:             len: self.len + 1,
68:         }
69:     }
116:     pub(super) fn appended(mut self, fragment: impl Into<String>) -> Self {
117:         let fragment = fragment.into();
118:         let len = self.len + fragment.len();
119:         self.tail = Some(Arc::new(PersistentStringTextNode {
120:             previous: self.tail,
121:             fragment,
122:         }));
123:         self.len = len;
124:         self
125:     }
crates/rum-stdlib/src/strings/patterns.rs
743: fn parse_string_match_limit_and_options(
744:     extras: &[Expr],
745: ) -> Option<(usize, StringPatternMatchConfig)> {
746:     let mut limit = usize::MAX;
747:     let mut config = StringPatternMatchConfig::default();
crates/rum-stdlib/src/strings/patterns/search/rules.rs
258: fn resume_suspended_replace_string_value_from_index(
259:     session: &mut Session,
260:     state: FrozenStringValue<StringReplaceRuleEvalState>,
261:     mut index: usize,
262:     replacements: usize,
263:     mut literal_match_index: usize,
264:     mut output: PersistentStringText,
265: ) -> Result<String, EvalError> {
313:     while index <= fixed.chars.len() {
314:         if replacements >= fixed.max_replacements {
315:             return Ok(output
316:                 .appended(fixed.chars[index..].iter().collect::<String>())
317:                 .to_string());
318:         }
345:         if index == fixed.chars.len() {
346:             break;
347:         }
348:         output = output.appended(fixed.chars[index].to_string());
349:         index += 1;
350:     }
351:
352:     Ok(output.to_string())
crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs
268: fn resume_suspended_replace_string_value_from_candidate(
269:     session: &mut Session,
270:     state: FrozenStringValue<GuardedStringReplaceRuleEvalState>,
271:     mut index: usize,
272:     replacements: usize,
273:     mut rule_index: usize,
274:     mut output: PersistentStringText,
275: ) -> Result<String, EvalError> {
284:         let Some(rule) = fixed.rules.get(rule_index) else {
285:             if index == fixed.chars.len() {
286:                 return Ok(output.to_string());
287:             }
288:             output = output.appended(fixed.chars[index].to_string());
289:             index += 1;
290:             rule_index = 0;
291:             continue;
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs
62: impl<T: FrameRootTrace> PersistentRewriteQueue<T> {
63:     pub(super) fn freeze(values: Vec<T>) -> Self {
64:         let len = values.len();
65:         let mut head = None;
66:         for value in values.into_iter().rev() {
67:             head = Some(Arc::new(PersistentRewriteQueueNode { value, next: head }));
68:         }
69:         Self { head, len }
70:     }
crates/rum-stdlib/src/structure_ops/suspended_state.rs
71:     pub(super) fn push(&mut self, value: T) {
72:         if let Some(tail) = self.tail.as_mut().and_then(Arc::get_mut) {
73:             if tail.values.len() < PERSISTENT_SEQUENCE_CHUNK_CAPACITY {
74:                 tail.values.push(value);
75:                 self.len += 1;
76:                 return;
77:             }
78:         }
79:
80:         let mut values = Vec::with_capacity(PERSISTENT_SEQUENCE_CHUNK_CAPACITY);
81:         values.push(value);
82:         self.tail = Some(Arc::new(PersistentStructureSequenceNode {
83:             previous: self.tail.take(),
84:             values,
85:         }));
86:         self.len += 1;
87:     }
~~~

#### Evidence B — Enforcement precedent, manifests and source-closure boundaries

<details>
<summary>Read-only command</summary>

~~~bash
python3 -B - <<'PY'
from pathlib import Path
sections = [["crates/rum-runtime/Cargo.toml",[[1,22]]],["crates/rum-stdlib/Cargo.toml",[[12,14],[25,27]]],["crates/rum-derive/Cargo.toml",[[10,16]]],["crates/rum-stdlib/src/lib.rs",[[152,154]]],["crates/rum-stdlib/src/tests.rs",[[7,8]]],["crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs",[[1,9],[20,25],[196,200],[800,801],[831,837]]],["crates/rum-stdlib/src/tests/association_depth_contract/registry.rs",[[39,47],[509,520],[569,581],[591,601]]],["crates/rum-derive/src/lib.rs",[[45,51],[85,98]]],["crates/rum-runtime/src/runtime/model/session.rs",[[95,95],[137,139]]],["crates/rum-runtime/src/runtime/suspension/core.rs",[[9,18],[76,81],[86,98],[654,658]]],["crates/rum-runtime/src/runtime/file_io.rs",[[93,107]]],["crates/rum-runtime/src/runtime/metadata.rs",[[332,332]]],["crates/rum-runtime/src/runtime/model/cache.rs",[[11,19]]],["crates/rum-core/src/expr.rs",[[7,10],[327,332]]],["crates/rum-stdlib/src/tests/association_key_reshape_depth.rs",[[310,325]]],["AGENTS.md",[[36,49],[86,93],[109,122]]],["docs/dev/codex/TEMPLATE.md",[[19,25],[32,36],[99,100],[102,111],[125,129],[151,155]]]]
for name, ranges in sections:
    lines = Path(name).read_text().splitlines()
    print(name)
    for first, last in ranges:
        for number in range(first, last + 1):
            print(f'{number}: {lines[number - 1]}'.rstrip())
PY
~~~

</details>

~~~text
crates/rum-runtime/Cargo.toml
1: [package]
2: name = "rum-runtime"
3: version.workspace = true
4: edition.workspace = true
5: rust-version.workspace = true
6: license.workspace = true
7: repository.workspace = true
8: keywords.workspace = true
9: categories.workspace = true
10: description = "Pure rum kernel for lowering, evaluation, definitions, matching, tracing, and host requests."
11:
12: [features]
13: test-builtin-memory-growth = []
14: test-native-stack-guard-zero = []
15:
16: [dependencies]
17: rum-core = { path = "../rum-core" }
18: rum-derive.workspace = true
19: rum-syntax = { path = "../rum-syntax" }
20: num-integer.workspace = true
21: num-traits.workspace = true
22: thiserror.workspace = true
crates/rum-stdlib/Cargo.toml
12: [features]
13: default = ["hash"]
14: hash = ["dep:sha2", "dep:sha1", "dep:md-5", "dep:crc32fast"]
25: [dev-dependencies]
26: rum-runtime = { path = "../rum-runtime", features = ["test-builtin-memory-growth"] }
27: syn = { version = "2", features = ["full", "visit"] }
crates/rum-derive/Cargo.toml
10: [lib]
11: proc-macro = true
12:
13: [dependencies]
14: proc-macro2 = "1"
15: quote = "1"
16: syn = { version = "2", features = ["full", "visit"] }
crates/rum-stdlib/src/lib.rs
152: #[cfg(test)]
153: #[path = "tests.rs"]
154: mod tests;
crates/rum-stdlib/src/tests.rs
7: #[path = "tests/association_depth_contract/mod.rs"]
8: mod association_depth_contract;
crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs
1: //! Closed source inventory for recursion in the association implementation roots.
2: //!
3: //! The scan intentionally matches the layer-3 implementation boundary. It sees
4: //! direct calls and `self`/`Self` method calls whose caller and callee are
5: //! defined in the same file. These roots implement the catalog association
6: //! surface and its six declared cross-family axes; that correspondence, rather
7: //! than convenience, defines the boundary. The scan cannot see recursion
8: //! reached through a trait object or function pointer, recursion generated by
9: //! a macro, or recursion outside [`SCANNED_ROOTS`].
20: pub(super) const SCANNED_ROOTS: &[&str] = &[
21:     "crates/rum-runtime/src/runtime/kernel_data/association_data.rs",
22:     "crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs",
23:     "crates/rum-stdlib/src/associations",
24:     "crates/rum-stdlib/src/functional_ops/apply_map",
25: ];
196: fn recursive_edges_from_source(path: &str, source: &str) -> Vec<RecursiveEdge> {
197:     let syntax = syn::parse_file(source)
198:         .unwrap_or_else(|error| panic!("recursion inventory should parse {path}: {error}"));
199:     let mut definitions = CallableCollector::default();
200:     definitions.visit_file(&syntax);
800: #[test]
801: fn association_recursion_inventory_is_sorted_exhaustive_and_declared() {
831: #[test]
832: fn synthetic_unregistered_recursion_is_reported_exactly() {
833:     let source = r#"
834:         fn self_recursive() { self_recursive(); }
835:         fn mutual_left() { mutual_right(); mutual_right(); }
836:         fn mutual_right() { mutual_left(); }
837:         fn acyclic_root() { acyclic_leaf(); }
crates/rum-stdlib/src/tests/association_depth_contract/registry.rs
39: struct AssociationDepthRow {
40:     surface: &'static str,
41:     catalog_name: Option<&'static str>,
42:     mechanism: Mechanism,
43:     contract_status: ContractStatus,
44:     failure_direction: Option<FailureDirection>,
45:     owed_by: Option<&'static str>,
46:     recursion_declarations: &'static [&'static str],
47: }
509:         if matches!(row.contract_status, ContractStatus::ShortOfContract) && row.owed_by.is_none() {
510:             return Err(format!(
511:                 "{} is short of the contract but has no owning brief or queue item",
512:                 row.surface
513:             ));
514:         }
515:         if matches!(row.mechanism, Mechanism::UnguardedNative) && row.owed_by.is_none() {
516:             return Err(format!(
517:                 "{} is unguarded but has no owning brief or queue item",
518:                 row.surface
519:             ));
520:         }
569:     let missing = catalog_set
570:         .difference(&registry_set)
571:         .copied()
572:         .collect::<Vec<_>>();
573:     let extra = registry_set
574:         .difference(&catalog_set)
575:         .copied()
576:         .collect::<Vec<_>>();
577:     assert!(
578:         missing.is_empty()
579:             && extra.is_empty()
580:             && catalog_names.len() == registry_catalog_names.len(),
581:         "association depth registry/catalog mismatch; catalog entries without rows: {missing:?}; rows without catalog entries: {extra:?}"
591: #[test]
592: #[should_panic(
593:     expected = "AssociationKeyFlatten declares NoProportionalDescent but recursive edge (\"crates/rum-stdlib/src/associations/key_reshape/traversal.rs\", \"flatten_association_entries_with_front_depth\", \"flatten_association_entries_with_front_depth\", 1)"
594: )]
595: fn real_catalog_misdeclaration_names_association_key_flatten_recursive_edge() {
596:     let mut rows = ASSOCIATION_DEPTH_REGISTRY.to_vec();
597:     let row = rows
598:         .iter_mut()
599:         .find(|row| row.catalog_name == Some("AssociationKeyFlatten"))
600:         .expect("AssociationKeyFlatten should have a registry row");
601:     row.mechanism = Mechanism::NoProportionalDescent;
crates/rum-derive/src/lib.rs
45: #[proc_macro_derive(FrameRootTrace)]
46: pub fn derive_frame_root_trace(input: TokenStream) -> TokenStream {
47:     let input = parse_macro_input!(input as DeriveInput);
48:     expand_frame_root_trace(&input)
49:         .unwrap_or_else(syn::Error::into_compile_error)
50:         .into()
51: }
85: fn expand_frame_root_trace(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
86:     let name = &input.ident;
87:     let mut generics = input.generics.clone();
88:     add_field_type_parameter_bounds(&mut generics, &input.data);
89:     let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
90:
91:     let body = match &input.data {
92:         Data::Struct(data) => trace_struct(data),
93:         Data::Enum(data) => trace_enum(data),
94:         Data::Union(_) => {
95:             return Err(syn::Error::new_spanned(
96:                 name,
97:                 "FrameRootTrace cannot be derived for unions; use a struct or enum with traceable fields",
98:             ));
crates/rum-runtime/src/runtime/model/session.rs
95:     pub(in crate::runtime) print_sink: Option<Box<dyn FnMut(&str) + Send>>,
137:     pub fn set_print_sink(&mut self, sink: Box<dyn FnMut(&str) + Send>) {
138:         self.print_sink = Some(sink);
139:     }
crates/rum-runtime/src/runtime/suspension/core.rs
9: type DynSuspensionFrame = dyn SuspensionFrame;
10:
11: #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
12: pub(in crate::runtime) struct SuspensionGeneration(u64);
13:
14: pub(in crate::runtime) struct SuspendedEvaluation {
15:     pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
16:     generation: SuspensionGeneration,
17:     frames: VecDeque<Arc<DynSuspensionFrame>>,
18: }
76: pub trait SuspensionFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static {
77:     fn resume(
78:         self: Arc<Self>,
79:         session: &mut Session,
80:         outcome: Result<Expr, EvalError>,
81:     ) -> FrameAction;
86: pub trait PureFrameFinalizer: Send + 'static {
87:     fn finish(self: Box<Self>) -> Result<Expr, EvalError>;
88: }
89:
90: /// The closed transition protocol between a named frame and the driver.
91: #[doc(hidden)]
92: pub enum FrameAction {
93:     Complete(Result<Expr, EvalError>),
94:     Evaluate {
95:         expr: Expr,
96:         next_frame: Arc<DynSuspensionFrame>,
97:     },
98:     Finalize(Box<dyn PureFrameFinalizer>),
654: /// Matches a potentially suspended result without constructing frame state on
655: /// the ordinary path. The macro accepts data expressions only, never closures.
656: #[doc(hidden)]
657: #[macro_export]
658: macro_rules! resume_or_suspend {
crates/rum-runtime/src/runtime/file_io.rs
93: #[derive(Clone)]
94: pub struct FileIoProvider {
95:     inner: Arc<dyn FileIo>,
96: }
97:
98: impl FileIoProvider {
99:     pub fn new(provider: impl FileIo + 'static) -> Self {
100:         Self {
101:             inner: Arc::new(provider),
102:         }
103:     }
104:
105:     pub fn from_arc(provider: Arc<dyn FileIo>) -> Self {
106:         Self { inner: provider }
107:     }
crates/rum-runtime/src/runtime/metadata.rs
332: pub type BuiltinFn = fn(&mut Session, &Expr, &SharedArgs) -> Result<Expr, EvalError>;
crates/rum-runtime/src/runtime/model/cache.rs
11: pub(in crate::runtime) type SymbolCountMap =
12:     HashMap<Symbol, u32, BuildHasherDefault<RuntimeCacheHasher>>;
13: pub type SymbolExprMap = HashMap<Symbol, Expr, BuildHasherDefault<RuntimeCacheHasher>>;
14:
15: pub(in crate::runtime) type SymbolSet =
16:     std::collections::HashSet<Symbol, BuildHasherDefault<RuntimeCacheHasher>>;
17:
18: pub(in crate::runtime) type DefinitionMap =
19:     HashMap<Symbol, SymbolDefinition, BuildHasherDefault<RuntimeCacheHasher>>;
crates/rum-core/src/expr.rs
7: use std::sync::{Arc, Mutex, OnceLock, Weak};
8:
9: use num_bigint::BigInt;
10: use serde::{Deserialize, Serialize};
327:
328: #[derive(Clone, Debug, PartialEq, Eq, Hash)]
329: pub enum IntValue {
330:     Small(i64),
331:     Big(Box<BigInt>),
332: }
crates/rum-stdlib/src/tests/association_key_reshape_depth.rs
310: /// The deflatten error path drops a partially built `NestedPathNodes`, whose
311: /// `Branch` arm is recursive and would otherwise drop recursively.
312: ///
313: /// `path_from_key` rejects exactly one shape — an **empty list** key — so a
314: /// deep good path followed by `{}` is the only way to build a deep intermediate
315: /// tree and then abandon it. A non-list key is treated as a one-element path,
316: /// not an error, which is why a fixture built around one measures the ordinary
317: /// success path instead.
318: ///
319: /// The depth is measured, not chosen: on this 2 MiB thread a recursive drop of
320: /// the intermediate tree survives 20 000 and terminates the process at 60 000.
321: const DROP_PATH_LEVELS: usize = 60_000;
322:
323: #[test]
324: #[ignore = "release-only deployment-stack probe; run explicitly with --release"]
325: fn association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack() {
AGENTS.md
36: ## Testing Policy (hard lines)
37:
38: 1. Tests verify `rum`'s **own documented behavior** — the language guide, the
39:    generated builtin reference, and the host crates' rustdoc are the
40:    normative contract. See `docs/testing-strategy.md`.
41: 2. **No black-box differential testing against other vendors' engines.** Do
42:    not add oracle harnesses, comparison lanes, compatibility matrices, or
43:    conformance reports — in code, scripts, fixtures, docs, or CI. Do not
44:    commit outputs generated by third-party engines. This policy is permanent
45:    (`DISCLAIMER.md` records its history).
46: 3. Performance gates are **rum-vs-rum A/B**: run
47:    `scripts/core-language-microbench.sh` on the base and candidate builds and
48:    compare per-label `timings.tsv` rows. A regression on any hot label, or on
49:    a same-tier neighbor of the target label, is a defect to fix or report.
86: Use focused tests while iterating, then run the broader relevant suite before
87: handing work back. For runtime semantics work, `cargo test -p rum-runtime` is
88: the default minimum; cross-crate changes need `--workspace`.
89:
90: For long-running commands or anything that may spawn child processes, do not
91: start them bare. Use `scripts/run-bounded.py --timeout SECONDS -- <command>`
92: (or a repo script that delegates to it); it kills the whole process group on
93: timeout so no orphaned children are left behind.
109: **Safe Rust is a hard convention.** `rum-core` and `rum-runtime` are
110: ZERO-`unsafe` (verified; the only `unsafe` in the repo is the off-by-default
111: alloc-count `GlobalAlloc` in `rum-cli`). Do not introduce `unsafe` into the
112: kernel crates — there is almost always a safe equivalent at the same cost
113: (e.g. `Option<Arc<…>>` niche-packs to `Arc<…>`'s size and gives a safe
114: `.take()` for a manual `Drop`). If you believe `unsafe` is unavoidable, STOP
115: and surface it for a decision rather than shipping it.
116:
117: **Hot-path changes are measured, not asserted.** A change touching an
118: evaluation hot path (eval/dispatch/matcher, `Expr`/`SharedExpr`/`SharedArgs`
119: construction or `Drop`, candidate-value lookup) must be gated with a
120: `scripts/core-language-microbench.sh` base-vs-new A/B — a "no default-path
121: cost" intention is not evidence; an O(N) cost hidden inside a "fast path"
122: only shows up when measured.
docs/dev/codex/TEMPLATE.md
19: - **Design-only brief** — `Scope: NONE — no source change`. Codex surveys the
20:   code and returns a step-by-step *plan*: files/functions per step, how the
21:   invariants stay intact, the gate tests per step, a call-site/ripple
22:   inventory, and risks + open questions — every claim carrying `path:line`
23:   evidence. It must also **draft the follow-up implementation briefs
24:   themselves**, in this template's shape, not merely describe the steps.
25:   Codex writes no code in this shape.
32: **Plan-first is the default (user ruling 2026-07-29).** Any non-trivial or
33: cross-cutting task opens with a design-only brief; Claude then reviews the
34: plan, settles the open questions, and **rewrites** Codex's drafts into his own
35: implementation briefs before dispatching them one at a time. Codex's drafts
36: are raw material, never dispatched as-is. Trivial mechanical jobs may still go
99: 7. **Stop-loss is real.** If the same step fails twice for the same reason, or
100:    the measured ratio does not move, STOP and report — do not keep tuning.
102: ## Verification (MANDATORY — you run it; iterate to green / to evidence)
103:
104: Diagnostic brief: produce the named artifacts and paste the key rows VERBATIM.
105: No source change means no test gate; the evidence IS the deliverable.
106:
107: Implementation brief, run exactly these and paste results verbatim:
108:
109: ```
110: cargo fmt --check
111: cargo test --workspace        # or the narrower -p suite the brief names
125: ## Hard rules
126:
127: - Do NOT `git commit`, branch, push, or touch git config. Leave all changes in
128:   the working tree; Claude runs the authoritative gate and commits.
129: - Do NOT modify files outside Scope. If you believe you must, STOP and write
151: - **Timeout discipline — derive bounds from data, never guess.** Use
152:   `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
153:   could run long or spawn children; size the bound from a prior measured run
154:   of the same shape, and remember a baseline build can be slower than an
155:   optimized one.
~~~

#### Evidence C — Persistent-node lexical shortlist and link fields

<details>
<summary>Read-only command</summary>

~~~bash
python3 -B - <<'PY'
from pathlib import Path
import re
rows = []
for crate in ('rum-core', 'rum-runtime', 'rum-stdlib'):
    for path in sorted(Path('crates', crate, 'src').rglob('*.rs')):
        if 'tests' in path.parts:
            continue
        lines = path.read_text().splitlines()
        for index, line in enumerate(lines):
            if re.match(r'\s*struct Persistent\w*Node(?:<|\s|\{)', line):
                links = [(i + 1, lines[i].strip()) for i in range(index + 1, min(index + 5, len(lines))) if re.match(r'\s*(previous|next):', lines[i])]
                rows.append((path, index + 1, line.strip(), links))
for path, number, declaration, links in rows:
    print(f'{path}:{number}: {declaration}')
    for number, field in links:
        print(f'  {number}: {field}')
print(f'LEXICAL SHORTLIST ONLY: {len(rows)} Persistent*Node declarations in {len({row[0] for row in rows})} files; not an ownership inventory.')
print('Link fields in shortlist: previous=' + str(sum(field.startswith('previous:') for *_, links in rows for _, field in links)) + ' next=' + str(sum(field.startswith('next:') for *_, links in rows for _, field in links)))
PY
~~~

</details>

~~~text
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:65: struct PersistentExprAccumulatorNode {
  66: previous: Option<Arc<PersistentExprAccumulatorNode>>,
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:57: struct PersistentRewriteQueueNode<T: FrameRootTrace> {
  59: next: Option<Arc<PersistentRewriteQueueNode<T>>>,
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/suspended_state.rs:105: struct PersistentRewriteExprNode {
  106: previous: Option<Arc<PersistentRewriteExprNode>>,
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/suspended_state.rs:27: struct PersistentReplacePartTargetNode {
  28: previous: Option<Arc<PersistentReplacePartTargetNode>>,
crates/rum-runtime/src/runtime/pattern_matching/suspended_state.rs:79: struct PersistentMatchResultsNode {
  80: previous: Option<Arc<PersistentMatchResultsNode>>,
crates/rum-runtime/src/runtime/tree_walkers/position.rs:16: struct PersistentPositionNode {
  17: previous: Option<Arc<PersistentPositionNode>>,
crates/rum-stdlib/src/associations/suspended_state.rs:57: struct PersistentAssociationSequenceNode<T: FrameRootTrace> {
  58: previous: Option<Arc<PersistentAssociationSequenceNode<T>>>,
crates/rum-stdlib/src/associations/suspended_state.rs:111: struct PersistentAssociationGroupNode<T: FrameRootTrace> {
  112: previous: Option<Arc<PersistentAssociationGroupNode<T>>>,
crates/rum-stdlib/src/functional_ops/suspended_state.rs:36: struct PersistentFunctionalSequenceNode<T: FrameRootTrace> {
  37: previous: Option<Arc<PersistentFunctionalSequenceNode<T>>>,
crates/rum-stdlib/src/logic/suspended_state.rs:13: struct PersistentLogicSequenceNode<T: FrameRootTrace> {
  14: previous: Option<Arc<PersistentLogicSequenceNode<T>>>,
crates/rum-stdlib/src/predicates/suspended_state.rs:29: struct PersistentPredicateMatchNode {
  30: previous: Option<Arc<PersistentPredicateMatchNode>>,
crates/rum-stdlib/src/strings/suspended_state.rs:45: struct PersistentStringSequenceNode<T: FrameRootTrace> {
  46: previous: Option<Arc<PersistentStringSequenceNode<T>>>,
crates/rum-stdlib/src/strings/suspended_state.rs:101: struct PersistentStringTextNode {
  102: previous: Option<Arc<PersistentStringTextNode>>,
crates/rum-stdlib/src/structure_ops/suspended_state.rs:56: struct PersistentStructureSequenceNode<T: FrameRootTrace> {
  57: previous: Option<Arc<PersistentStructureSequenceNode<T>>>,
crates/rum-stdlib/src/structure_ops/suspended_state.rs:150: struct PersistentStructureCountNode {
  151: previous: Option<Arc<PersistentStructureCountNode>>,
crates/rum-stdlib/src/structure_ops/suspended_state.rs:234: struct PersistentStructureSetNode<T: FrameRootTrace> {
  235: previous: Option<Arc<PersistentStructureSetNode<T>>>,
LEXICAL SHORTLIST ONLY: 16 Persistent*Node declarations in 11 files; not an ownership inventory.
Link fields in shortlist: previous=15 next=1
~~~

### Append-only verification

**M:** the original prefix hash, report section count, citation locations and
working-tree scope were checked after the report was appended. Location checks
verify that cited files/lines exist; they do not substitute for reading the
source excerpts above. This is document verification, not a language test gate
(docs/dev/codex/brief-476-ownership-graph-inventory.md:167).

**M — audit correction:** the first section-count assertion incorrectly searched
the entire file, including the original brief's four numbered questions. It
failed as follows; the corrected audit limits the assertion to the appended
report and finds eight sections. No report content was changed to satisfy it.

~~~text
Traceback (most recent call last):
  File "<stdin>", line 13, in <module>
AssertionError
~~~

Read-only corrected audit command:

~~~bash
python3 -B - <<'PY'
from pathlib import Path
import hashlib, re
path = Path('docs/dev/codex/brief-476-ownership-graph-inventory.md')
data = path.read_bytes()
expected = '1c67ce92c88687630a4ba32984c46c9b0d3807a54e37a966eed81084a6201bb0'
assert hashlib.sha256(data[:9831]).hexdigest() == expected
text = data.decode()
report = text.split('## Report — 2026-09-19', 1)[1].split('### Verbatim evidence for this report', 1)[0]
refs = set(re.findall(r'\b((?:crates|docs)/[\w./-]+\.(?:rs|md|toml)|AGENTS\.md|TODO\.md):(\d+)', report))
for name, number in refs:
    lines = Path(name).read_text().splitlines()
    assert 1 <= int(number) <= len(lines), (name, number)
original_questions = len(re.findall(r'^### [1-8]\. ', text.split('## Report — 2026-09-19', 1)[0], re.M))
report_sections = len(re.findall(r'^### [1-8]\. ', report, re.M))
assert report_sections == 8
assert not re.search(r'[ \t]+$', text, re.M)
print('Original prefix: 9831 bytes; SHA-256 ' + expected + '; unchanged.')
print(f'Section-count audit: original brief questions={original_questions}; appended report sections={report_sections}.')
print(f'Citation locations: {len(refs)} checked; missing files or out-of-range lines: 0.')
print('Builds, tests, runtime probes, benchmarks: not run (design-only).')
PY
~~~

Verbatim output (exit status 0):

~~~text
Original prefix: 9831 bytes; SHA-256 1c67ce92c88687630a4ba32984c46c9b0d3807a54e37a966eed81084a6201bb0; unchanged.
Section-count audit: original brief questions=4; appended report sections=8.
Citation locations: 92 checked; missing files or out-of-range lines: 0.
Builds, tests, runtime probes, benchmarks: not run (design-only).
~~~

Command: GIT_OPTIONAL_LOCKS=0 git diff --check. Exit status 0; output was empty.

Command: GIT_OPTIONAL_LOCKS=0 git status --short. Exit status 0; verbatim output:

~~~text
 M docs/dev/codex/brief-476-ownership-graph-inventory.md
~~~

**M:** no source file changed. No builds, tests, runtime probes, benchmark or
drop-ceiling measurements were run. The initial brief remains byte-for-byte
intact, and this report is the sole appended artifact
(docs/dev/codex/brief-476-ownership-graph-inventory.md:150).
