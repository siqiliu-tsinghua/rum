# brief-423 — design: bounding traversal over deep values

Shape: **design-only**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

Opened by brief-422, which bounded four recursive descents and found that the
class behind them is not a missing depth guard but a structural gap. Deciding
between the available shapes is a design question, and plan-first is this
project's default for anything cross-cutting.

## The state, measured

The evaluator's own recursion is bounded: `recursion_limit` 1024 plus a 6 MiB
native-stack poll every four levels (`eval_core/mod.rs:34,39-42,68-78`). That
guard covers the evaluator descending into an expression it is evaluating. It
does **not** cover a traversal walking a value it was handed.

Two facts combine badly.

**A source-language script builds arbitrarily deep values.** `x = 1;
Do[x = {x}, {i, N}]` reaches N of roughly 80 000. The wrap is iterative and the
stored value is never re-descended, so neither the iteration limit nor the
recursion limit applies. (`Nest` does not do this — it re-evaluates and stops at
the iteration limit, which is why an earlier probe of mine wrongly concluded the
encoders were unreachable.)

**Every per-node traversal then walks that value with nothing counting.**
Measured stack-exhaustion thresholds, release build, from brief-422's inventory
and my own reproduction:

| Traversal | Exhausts the native stack around |
|---|---|
| `ToString`, `Print`, `FullForm` | 6 000 – 20 000 |
| `Map`, `Apply`, `Scan`, `ReplaceAll`, `Cases`, `Level`, `MemberQ`, `Count`, `FreeQ`, `Position`, `Save`, `DeleteMissing[_, Infinity]` | 20 000 – 50 000 |
| `Depth`, `LeafCount`, `ByteCount`, sort comparators, drop glue | survive 80 000; lean, but equally unbounded |

brief-422's report §3(e) lists many more source-verified rows it did not probe
individually: string-pattern machinery, calculus and polynomial walkers, schema
conformance, `Flatten[_, Infinity]`, tensor rank recursion, kernel-side
`PartialEq`/`Clone`/`Hash`/`Serialize` on `Expr`, substitution walks, matcher
compile and structural match, rewrite traversals, `lowering.rs`, and suspension
continuation encoding. Read that section; do not re-derive it.

## A documented claim that is now false

`TODO.md:1061-1065` and, more importantly,
`docs/host-embedding-and-hardening.md:113-117` tell embedders:

> Source-created trees are protected by parser, evaluator, matcher, and
> native-stack limits. A host can still construct a pathologically deep `Expr`
> directly through the Rust API and overflow the stack during recursive drop.
> … hosts must depth-limit directly constructed trees.

The first sentence does not hold. A source-created tree reaches 80 000 levels
through the loop above, and the traversals fail far below the depth at which
drop does — I confirmed drop itself survives 80 000. So the document tells a
host the risk is confined to trees it builds in Rust, and that is the wrong
advice. Your plan must include correcting both texts, and the correction has to
follow whatever the design decides rather than being written first.

## The prior art that constrains the answer

**Three safe iterative-`Drop` designs regressed the ordinary hot path by 20-50%
and were rejected** (`TODO.md:1061-1065`,
`docs/host-embedding-and-hardening.md:115-117`). So "make the traversals
iterative" is not an untested idea here; the closest previous attempt has a
measured counterexample.

**But plan-330 made sixteen operator chains iterative successfully**
(`docs/dev/codex/plan-330-left-chain-lowering.md`), with a 4 096-link ceiling
proved through the full round trip. So the technique is not disqualified either.

Explain why those two outcomes differ before you recommend anything. That
explanation is the most useful thing you can give me, because it predicts which
traversals can be converted cheaply and which cannot.

## What I want back

A reviewable plan plus drafted implementation briefs in `TEMPLATE.md` shape.
**Write no code.** Every claim carries `path:line`.

### 1. The options

At least these, and say if there is a fourth:

- **A — admit depth at construction.** Bound how deep a value may become, so
  every traversal downstream is safe by construction. State where the check
  would live, what it costs on `Expr` construction — the hottest path in the
  tree — and how it interacts with `SharedExpr` reuse, where a subtree's depth
  is not known from the node being built.
- **B — make the traversals iterative.** Per the prior art above, with an
  estimate of how many traversals that is, drawn from brief-422 §3(e).
- **C — a shared depth-counting guard traversals opt into.** Cheaper than
  converting them, more invasive than A. Say what it costs per node and how an
  omission would be caught, since a guard nothing enforces is the failure this
  line keeps repairing.

For each: what breaks, what it costs, what enforces it, and whether an omission
is a compile error, a test failure, or nothing.

### 2. The behaviour question, which is mine but needs your evidence

`Depth[x]` returns 80001 today for the loop above, and `Depth`, `LeafCount` and
`ByteCount` survive it. A construction-side bound would refuse to build that
value at all; a traversal-side bound would keep building it and refuse to
render it.

Tell me what each option does to currently-working behaviour, concretely: which
expressions that succeed today would stop succeeding, and at what depth. I will
make the ruling; I need the consequences priced first.

Note also that the renderers are **error-message and trace infrastructure**, not
only user-facing builtins. Say what a refusal even means for `ToString` when it
is called to format a message about another failure.

### 3. Cost

Whatever you recommend, name the microbench labels that would prove or disprove
a regression, and say which of them the rejected iterative-`Drop` designs moved
by 20-50%, if that is recoverable from the repository. Profiling env off for any
timing number you quote; say plainly if you did not measure.

### 4. Sequencing and drafted briefs

Ordered steps, what each covers, and drafted briefs. A MAY list must include a
module root whenever a type it exports changes; that omission has cost this
project eight stop-losses.

Include the documentation correction as an explicit step, not a footnote.

## Method requirements

- **Every claim carries `path:line`.**
- **Cross-check any script-derived count against live evidence.** A regex over
  this repository has produced a confidently wrong count three times, and I have
  made the same class of error six times in this line — including summing two
  inventories without checking their overlap, and taking the first `grep -rl`
  hit as a definition site when it was only a reference.
- **State first, verbatim, what your environment can actually run.**
- Say plainly where you reason from source rather than observation.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run `cargo` and the release `rum-cli`, and build throwaway
  crates under `/tmp` that you remove. If you believe a file must change, that
  is a finding for the report.
- **Do not propose raising the native stack or moving work onto a bigger
  thread.** That trades the threshold for nothing structural.
- **No new `unsafe`** in any proposal for `rum-core` or `rum-runtime`.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Append your report to this brief file** before finishing.

## Report format

1. Environment capability, verbatim.
2. Why iterative `Drop` failed and plan-330 succeeded.
3. The options, with cost, enforcement, and what breaks.
4. The behaviour consequences of each, priced concretely.
5. Cost: the labels, and any recoverable numbers from the rejected designs.
6. Sequencing and the drafted implementation briefs, including the
   documentation correction.
7. Risks and open questions — what you could not establish, and why.

---

## Codex report — design only

### 1. Environment capability, verbatim

The dispatcher supplied this capability statement verbatim:

> Filesystem sandboxing defines which files can be read or written. `sandbox_mode` is `workspace-write`: The sandbox permits reading files, and editing files in `cwd` and `writable_roots`. Editing files in other directories requires approval. Network access is restricted.
>
> Approval policy is currently never. Do not provide the `sandbox_permissions` for any reason, commands will be rejected.
>
> The writable roots are `/home/remi/dev/rum`, `/tmp`.

That is the environment I actually used: repository and `/tmp` reads/writes,
ordinary subprocesses, `cargo` with `--offline`, and the release `rum-cli`; it
matches the dispatcher's documented `workspace-write` meaning
(`docs/dev/codex/TEMPLATE.md:8-15`). I did not use the network, elevated access,
another engine, a stash, or any Git mutation; those would also violate this
brief's fence (`docs/dev/codex/brief-423-traversal-depth-design.md:143-156`).

The only Cargo command was the following; this was a build, not a test or a
timing run (`docs/dev/codex/TEMPLATE.md:102-105`):

```text
$ scripts/run-bounded.py --timeout 300 -- cargo build --offline --release -p rum-cli
    Finished `release` profile [optimized] target(s) in 0.02s
```

`git status --short` was empty before the work. Per the design-only exception,
the only intended final worktree change is this appended report
(`docs/dev/codex/brief-423-traversal-depth-design.md:3,145-157`). No test suite
was run because no implementation exists to test; the template says the
evidence is the gate for a no-source diagnostic/design result
(`docs/dev/codex/TEMPLATE.md:19-25,102-105`).

Evidence labels below are literal: **observed** means I ran the shown command on
this checkout; **source** means I inspected the cited implementation; and
**inference** means a design consequence derived from those cited facts. I do
not treat an observed stack threshold as portable; plan-330 likewise selected a
policy only after bracketing the repository-owned consumer on a 2 MiB stack
(`docs/dev/codex/plan-330-left-chain-lowering.md:271-289`).

### 2. Why iterative `Drop` failed and plan-330 succeeded

The two results differ in **where the iteration tax is paid**, not in whether an
explicit work stack is intrinsically good or bad.

The rejected `Drop` work intercepted a ubiquitous ownership primitive. The
current wrappers are one `Arc` each, their hot read path is direct `AsRef`/
`Deref`, and the current source has no manual expression-tree `Drop`
(`crates/rum-core/src/expr.rs:573-635,637-747`). The safe `Option<Arc>` attempt
put a branch on every dereference; the repository records +8–44%. The sentinel
attempt preserved dereference layout but made every wrapper destruction do the
manual extraction/drain machinery; it records +22–49%, including after the
`strong_count` fast path (`docs/dev/codex/brief-057-safe-iterative-drop-design-A.md@cb4c9d33:5-13,19-53`;
`PROGRESS.md@cb4c9d33:21-34`). The first, reverted unsafe attempt also performed
an O(N) depth probe before list drop and recorded about a 10% list-operation
regression (`docs/dev/codex/brief-056-safe-iterative-drop.md@cb4c9d33:5-12`).

That cost is broad because ordinary evaluation repeatedly creates, shares,
unwraps, and destroys `SharedExpr`/`SharedArgs`; even a shallow unique call paid
the custom drain instead of the standard `Arc` destructor. The historical safe
design explicitly required `try_unwrap` plus a work vector for unique children,
while shared children still needed refcount decisions
(`docs/dev/codex/brief-056-safe-iterative-drop.md@cb4c9d33:20-50`). In other
words, the benefit existed only on a rare deep unique chain, but the trigger ran
on every drop; the recorded conclusion was that no cheap trigger existed
(`PROGRESS.md@cb4c9d33:27-34`).

Plan-330 instead isolated a syntax shape behind an existing lowering dispatch.
Only sixteen named binary operators enter the iterative path
(`crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs:86-106`); that
path collects one lhs spine into a vector and folds it once
(`crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs:108-163`). It does
not change `Expr`, either shared wrapper, dereference, equality, or destruction.
The vector cost is therefore paid only while lowering an affected source chain,
and the nine flat families reduce the number of emitted runtime call nodes
instead of adding per-node lifetime work
(`docs/dev/codex/plan-330-left-chain-lowering.md:39-44,79-118,228-256`).

Plan-330 also kept a finite 4,096-link resource ceiling after removing native
recursion, because iteration removes stack growth but not linear allocation or
work (`docs/dev/codex/plan-330-left-chain-lowering.md:271-279`). Its current
release-only tests prove parse/lower/format/drop on a 2 MiB thread and separately
prove natural default drop for all seven still-nested operators
(`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:319-382,552-587`).
That is a targeted, opt-in traversal with a natural owned accumulator; manual
`Drop` is an implicit, universal traversal with no error return and no opt-in
site.

The predictive rule is consequently:

- iteration is a good candidate where the operation is already explicit and
  relatively cold, already allocates output/plans, and owns a natural frame
  stack—for example lowering, formatting, and transformation walkers
  (`crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs:108-163`;
  `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-90`);
- iteration is highest-risk where Rust invokes the operation implicitly or the
  current common case is one pointer operation—`Drop`, derived `Debug`/`Hash`,
  and equality are the clearest recursive examples
  (`crates/rum-core/src/expr.rs:573-574,605-609,637-638,705-708,749-775`).

### 3. Options: cost, enforcement, breakage, and omission detection

#### The depth being bounded

Any structural safety bound must use **physical expression height**, not the
language-level `Depth` helper. Define `h(Atom) = 1` and `h(Call) = 1 +
max(h(head), h(each raw argument))`. The current `Depth` implementation can omit
heads and gives associations special value-only treatment, whereas `Drop`,
derived traits, and raw renderers own/walk every head and argument edge
(`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`;
`crates/rum-core/src/expr.rs:749-775,850-870`). Reusing the user-visible `Depth`
number would therefore certify the wrong tree.

#### A — admit physical depth at construction

**Mechanism.** Put a saturating cached height in each private `Arc` allocation:
an inner `{ expr, height }` for `SharedExpr` and `{ args, max_child_height }` for
`SharedArgs`, with `WeakSharedArgs` pointing at the latter inner allocation.
Keep each strong/weak wrapper itself pointer-sized. All recursive edges in a
call already pass through the two private strong tuple wrappers
(`crates/rum-core/src/expr.rs:573-574,637-638,749-753`). A fixed core constant
`MAX_PHYSICAL_EXPR_DEPTH = L` can then be enforced without making `Expr` opaque:
the wrappers admit only children whose height is `< L`, so even a caller that
directly writes the public `Expr::Call { head, args }` variant can produce at
most height `L`. This is an inference from the current edge layout and private
wrapper fields (`crates/rum-core/src/expr.rs:573-574,637-638,749-753`).

**Shared reuse and cost.** `SharedExpr` reuse becomes O(1): the height travels
with the shared allocation. `SharedArgs::new` must scan only its immediate
`Vec<Expr>` once to cache the maximum; a call then performs one maximum and one
limit comparison. It must never scan the whole subtree, because repeated
`x={x}` construction would otherwise become quadratic. This adds O(arity)
loads/comparisons to every call construction plus metadata bytes to each shared
allocation; construction is already the explicitly named hot path
(`crates/rum-core/src/expr.rs:576-603,640-683,813-829`;
`docs/dev/codex/TEMPLATE.md:140-146`). Current observed representation baselines
are `Expr=32` bytes and pointer-sized shared wrappers; the latter were pinned as
8 bytes in the rejected work (`docs/dev/codex/brief-414-memory-accounting-gaps.md:249`;
`docs/dev/codex/brief-056-safe-iterative-drop.md@cb4c9d33:24-35,69-70`). The
inner-allocation size and hot-path effect of the proposed cache are unknown
until measured.

**What must break/change.** `SharedArgs::get_mut` and `make_mut` currently expose
a raw mutable `Vec`, so they can invalidate cached metadata and must be replaced
by checked transactional edits or a guard that cannot commit an over-limit
child (`crates/rum-core/src/expr.rs:660-670`). `SharedExpr::new`,
`SharedArgs::new`, `SharedArgs::default`, `From<Expr>`, `From<Vec<Expr>>`, and
the three infallible call constructors need checked implementations or
fallible counterparts/cutover; otherwise the only possible over-limit behavior
is a panic or silent substitution, neither of which is an acceptable guest
failure (`crates/rum-core/src/expr.rs:576-615,640-715,813-829`). A naive derive
would also serialize the new metadata, while derived deserialization bypasses
the checked constructors. Both directions therefore need manual logical-shape
implementations, with a depth-aware `DeserializeSeed` checking before recursive
descent; the current types derive both serde traits
(`crates/rum-core/src/expr.rs:573-574,637-638,749-753`). `WeakSharedArgs` and
its liveness/identity users must follow the changed Arc target
(`crates/rum-core/src/expr.rs:681-697`;
`crates/rum-runtime/src/runtime/model/cache.rs:100-109`). This is a public Rust
API and serde-contract change, not a local evaluator patch; `rum-core` is the
stable cross-layer value owner (`ARCHITECTURE.md:184-195`).

**Enforcement and omissions.** Once the tuple wrappers are the only recursive
edge constructors, their privacy plus safe Rust enforces the height invariant.
The constructor `Result` cutover and removal of raw mutation produce compile
errors at known call sites; boundary and serde tests catch arithmetic mistakes.
A later traversal can omit a guard and still cannot receive a value taller than
`L`, so that omission is harmless with respect to depth. This is the only option
of A/B/C that turns an omitted traversal check into a type invariant rather than
an audit convention (`crates/rum-core/src/expr.rs:573-574,637-638,749-753`). A
per-session limit alone cannot provide this property to hosts constructing
`rum_core::Expr` without a `Session`; the hard maximum must be fixed in
`rum-core` (`crates/rum-core/src/lib.rs:1-14`).

**Verdict.** Structurally complete, including default `Drop`, but highest API
blast radius and an unconditional construction/allocation cost. It is viable
only if the metadata A/B is neutral and the director accepts a hard language
value-depth boundary.

#### B — make traversals iterative

**Mechanism and cost.** Replace recursive calls with explicit visit/finish
frames. This preserves existing deep-value construction and can preserve exact
operation results, but each converted operation pays heap-stack push/pop work
proportional to visited nodes and needs its own ordering, association, `Heads`,
early-exit, and suspension invariants. The four `apply_map` engines alone have
different rebuild/path/result state (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-90,92-194,196-304,306-381`), and the runtime's count/position
walkers combine recursive depth descent with explicit sibling and suspension
state (`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:291-475`;
`crates/rum-runtime/src/runtime/tree_walkers/position.rs:199-455`).

**Work estimate.** I could not establish an exact function count: mutual
recursion, shared helpers, derived trait glue, and one engine serving several
builtins make a regex count actively misleading. I manually reconciled the
already-recorded §3(e) families, counting an algorithm once at its owner rather
than summing search hits. The result is **roughly 40–50 independent conversion
units; the conservative worksheet below contains 45**. This is a planning
estimate, not a claim that the repository has exactly 45 recursive functions
(`docs/dev/codex/brief-422-xml-depth-and-the-class.md:267-280,356-360`).

| Manually reconciled bucket | Conversion units | Ownership evidence |
|---|---:|---|
| Infallible core operations: `PartialEq`, `Debug`, `Hash`, `Serialize`, `Deserialize`, `Display`, `Drop` | 7 | `crates/rum-core/src/expr.rs:573-574,637-638,749-775,850-873` |
| Renderer/persistence infrastructure beyond core `Display`: syntax-aware render, transparent-hold transform, messages, trace layout, host conversion, Save/snapshot | 6 | `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:34-407,409-441`; `crates/rum-runtime/src/runtime/messages/message_format.rs:34-81,124-178`; `crates/rum-runtime/src/runtime/debug_tools/trace.rs:777-813`; `crates/rum-host/src/model/outcome.rs:355-397`; `crates/rum-runtime/src/runtime/session/session_api.rs:390-448` |
| General transforms/walkers: Apply, Map/MapAll, MapIndexed, Scan, Cases, DeleteCases, Replace, Level, Count/MemberQ/FreeQ, Position | 10 | `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-381`; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-321`; `crates/rum-runtime/src/runtime/tree_walkers/level.rs:77-165`; `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:291-475`; `crates/rum-runtime/src/runtime/tree_walkers/position.rs:199-455` |
| Kernel internals: fingerprints; binding, lexical, and slot substitution; matcher compilation; structural match; generic lowering; continuation encoding | 8 | `crates/rum-runtime/src/runtime/profiling/fingerprints.rs:35-109`; `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:47-79`; `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:366-476`; `crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:3-36`; `crates/rum-runtime/src/runtime/matcher_and_symbols.rs:3-114`; `crates/rum-runtime/src/runtime/lowering.rs:7-49`; `crates/rum-runtime/src/runtime/pattern_matching/continuation.rs:50-123` |
| Lean structural operations: `Depth`, `LeafCount`, memory/`ByteCount`, canonical comparator | 4 | `crates/rum-stdlib/src/structure_ops/mod.rs:115-149`; `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-400`; `docs/dev/codex/brief-422-xml-depth-and-the-class.md:274-278` |
| Specialist stdlib: string-pattern compile/search, calculus, polynomial predicates, polynomial transforms, schema, Flatten, tensor/rank, nested Merge, DeleteMissing | 10 | `docs/dev/codex/brief-422-xml-depth-and-the-class.md:278`; `crates/rum-stdlib/src/calculus.rs:88-253`; `crates/rum-stdlib/src/predicates.rs:275-374`; `crates/rum-stdlib/src/schema.rs:169-303`; `crates/rum-stdlib/src/structure_ops/flatten.rs:224-370`; `crates/rum-stdlib/src/associations/missing.rs:936-1055` |

Brief-422's prose also names `Clone`, but I did **not** count ordinary
`Expr::clone` as a recursive conversion unit: `Expr` clones `SharedExpr` and
`SharedArgs`, and both derive `Clone` over `Arc`, so a call clone increments the
two Arc counts rather than walking descendants (`crates/rum-core/src/expr.rs:573-574,637-638,749-753`). I could not establish a depth-recursive ordinary
clone path from current source; any narrower clone-on-write path should be
identified separately in 423A. This is one place where source inspection narrows
the prior inventory rather than repeating it confidently.

The count was not produced by a script. I used `rg` only to locate definitions,
opened the owners, and cross-checked reachability with live CLI probes. On a
2 MiB release stack, the same 4,096-wrap value aborted in six distinct family
heads (`Map`, `Apply`, `ReplaceAll`, `Count`, `Position`, `Level`), while all six
returned normally at 1,024 wraps; the source owners above are distinct. This
does not validate the count, but it does validate that the table is not merely
counting dead references (`docs/dev/codex/brief-422-xml-depth-and-the-class.md:269-278`).

**What breaks.** In principle no successful language result must change; an
explicit stack consumes heap instead of native stack, so a separate work/memory
ceiling may still terminate hostile breadth/depth. In practice, the core trait
set is the hard part: `Eq`, `Hash`, and `Drop` have no `Result` channel, while
`Display` cannot carry a domain-specific refusal; the prior `Drop` conversion
already failed the hot-path gate
(`crates/rum-core/src/expr.rs:573-574,749-778`;
`PROGRESS.md@cb4c9d33:21-34`). Suspension-aware transformation machines must
also preserve their live/resumed frame shape
(`docs/dev/codex/TEMPLATE.md:90-95`).

**Enforcement and omissions.** Converting a named helper removes recursion from
that helper, and its deep boundary test catches a regression. It does not make a
new recursive helper a compile error; an omitted/new traversal is **nothing**
until an inventory, review, or targeted test notices it. Making raw child access
unavailable would improve that answer, but doing so is effectively the public
representation change priced under A (`crates/rum-core/src/expr.rs:749-753`).

**Verdict.** Best option if retaining 80,001-deep values is mandatory, and a good
selective tool for renderers. It is not a credible one-shot repair of the whole
class without dozens of independently gated conversions, including another
solution for the historically expensive default `Drop`.

#### C — shared opt-in depth guard

**Mechanism and per-node cost.** Pass a copyable `TraversalDepth { remaining }`
or equivalent child token through recursive helpers; `descend()` does one
zero/limit comparison and one decrement/add per traversed edge. A session-owned
limit would be added beside the existing resource fields, which currently have
no value/traversal-depth member
(`crates/rum-runtime/src/runtime/model/config.rs:3-18,20-33`). Helpers that
currently return `Expr`, `usize`, `bool`, or tuples must become `Result`-bearing
or define an explicit refusal result; `apply_map`, `structural_depth`, and
`structural_leaf_count` show those current infallible signatures
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-18,92-115,306-314`;
`crates/rum-stdlib/src/structure_ops/mod.rs:115-149`).

That is cheaper per migrated walker than designing its full iterative frame
machine, but it must alter most recursive signatures and their callers, so it is
more source-location-invasive than A's central edge invariant. A remains more
invasive at the public representation/semver boundary; these are different cost
axes (`crates/rum-core/src/lib.rs:1-14`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-381`).

**What breaks.** At guard `G`, deep values still exist, but guarded operations
stop at the first edge beyond `G`. Error propagation must add a runtime error,
termination reason, continuation codec arms, request mapping, and host mapping;
those are exhaustive surfaces today
(`crates/rum-runtime/src/runtime/session/session_state.rs:498-540`;
`crates/rum-runtime/src/runtime/model/requests.rs:56-85,157-211`;
`crates/rum-runtime/src/runtime/suspension/error.rs:269-339,430-539`;
`crates/rum-host/src/model/outcome.rs:41-64,338-351`). Core `Eq`, `Hash`, `Debug`,
`Display`, serde, and `Drop` cannot accept that session token, and several cannot
report a domain refusal at all (`crates/rum-core/src/expr.rs:573-574,749-778,850-873`).
Thus C cannot close the class alone.

**Enforcement and omissions.** Once a helper signature requires the token,
existing callers of that helper fail to compile until migrated. Nothing stops a
new helper from recursing directly over public `Expr::Call`, however, so an
omission is still **nothing** unless a manually maintained inventory/test names
it (`crates/rum-core/src/expr.rs:749-753`). A custom lint could turn a syntactic
subset into a test failure, but it could not recognize trait-derived recursion,
mutual recursion, or semantic child access reliably; I do not recommend calling
that structural enforcement.

**Renderer refusal.** The current renderer API returns `String`, not `Result`,
and is re-exported to stdlib/host (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:19-31`;
`crates/rum-runtime/src/runtime/kernel_api.rs:121-124`). It formats Print,
messages, traces, snapshots, and every host success, not just `ToString`
(`crates/rum-runtime/src/runtime/builtins_input.rs:104-111,430-434`;
`crates/rum-runtime/src/runtime/messages/message_format.rs:124-178`;
`crates/rum-host/src/model/outcome.rs:355-397`). Therefore a C-only strict
renderer cannot safely format its own failure. C needs two policies: a strict
user renderer that returns `TraversalDepthLimit`, and a bounded infrastructure
renderer that stops at `G` and substitutes a fixed, depth-independent marker.
The primary error text must contain only numbers/static text, never recursively
render the offending value. Alternatively, making infrastructure rendering
fully iterative is a B tranche that preserves all bytes. Changing host success
text to a marker is itself a contract change because `HostValue.text` promises
canonical rendering
(`crates/rum-host/src/model/outcome.rs:164-173`).

**Verdict.** Lowest implementation cost for a selected fallible builtin, but
not an acceptable systemic answer. It neither covers infallible traits nor
catches future omissions.

#### D — fourth option: hard construction invariant plus selective iteration

The useful fourth option is a hybrid, not another guard: use A to establish one
fixed maximum physical height, and use B only for (1) rendering/diagnostic
infrastructure that must be total and (2) any traversal whose measured safe
boundary is below the chosen `L`. This avoids a manual iterative `Drop`—default
drop is safe because the value is bounded—while avoiding forty-plus conversions
when a low enough `L` makes the remaining recursion safe. It inherits A's
construction cost and behavior change, but its omission answer remains the A
type invariant (`crates/rum-core/src/expr.rs:573-753`;
`docs/dev/codex/brief-422-xml-depth-and-the-class.md:348-353`).

**Recommendation.** Choose D if the director accepts a hard value-depth
contract. Do not ship A's cache before its hot-path A/B is neutral. Do not ship
a numeric `L` until a dedicated 2 MiB release/debug matrix covers every bucket
above. If the director instead rules that the currently working 80,001-deep
value must remain valid, choose B and accept that it is a multi-brief conversion
program; C alone is not shippable. This recommendation follows the project's
sandbox-first goal and requirement that normal resource failures leave the
session usable (`README.md:3-8`; `docs/host-embedding-and-hardening.md:61-71`).

`L = 1,024` is the best **candidate to measure**, not an established safe
constant. It is high enough for the current 256-level codec contracts—the live
full-depth results have user-visible `Depth` values 513 for XML and 257 for JSON,
and those atom-headed shapes follow the same deepest raw ownership path—and low
enough that the sampled renderer and six generic traversal heads
survived one level beyond it on a 2 MiB release stack. But it would supersede the
documented 4,096-link acceptance for seven nested left-chain operators, whose
test intentionally builds a 4,096-deep runtime `Expr`
(`crates/rum-stdlib/src/tests/codecs_xml.rs:276-321`;
`crates/rum-stdlib/src/xml/encode.rs:250-269`;
`docs/guide/resource-limits-and-sandbox.md:125-135`;
`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:562-587`). Preserving
that contract requires `L >= 4,097` and, from the live evidence below, iterative
conversion of at least the renderer and six generic traversal families before
the cap can be called safe.

### 4. Concrete behavior consequences

#### Live boundary evidence

The current release CLI still builds and measures the source-created 80,000-wrap
value as brief-422 reported (`docs/dev/codex/brief-422-xml-depth-and-the-class.md:284-290`):

```text
$ scripts/run-bounded.py --timeout 40 -- ./target/release/rum-cli --eval 'x = 1; Do[x = {x}, {i, 80000}]; {Depth[x], LeafCount[x], ByteCount[x]}'
List[80001, 80001, 13680032]
```

On a deliberately reduced 2 MiB process stack, current `ToString` brackets as
follows. This is process behavior, not a timing measurement or portable limit;
the source renderer is recursive (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:34-407`):

Each row was a fresh bounded subprocess using the following command, with `N`
replaced by the row value
(`docs/host-embedding-and-hardening.md:67-71`):

```text
scripts/run-bounded.py --timeout 40 -- bash -c 'ulimit -s 2048; exec ./target/release/rum-cli --eval "$1"' _ 'x = 1; Do[x = {x}, {i, N}]; Head[ToString[x]]'
```

```text
wraps=256 exit=0 stdout='String' stderr=''
wraps=512 exit=0 stdout='String' stderr=''
wraps=1024 exit=0 stdout='String' stderr=''
wraps=2048 exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
wraps=4096 exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
```

The cross-family live check at 4,096 wraps returned normally for `Depth` but
aborted for `Map`, `Apply`, `ReplaceAll`, `Count`, `Position`, and `Level`; all
six returned normally at both 512 and 1,024 wraps. Those heads map to distinct
recursive owners in the source
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-381`;
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-321`;
`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:291-475`;
`crates/rum-runtime/src/runtime/tree_walkers/position.rs:199-455`;
`crates/rum-runtime/src/runtime/tree_walkers/level.rs:77-165`):

```text
probe=Depth exit=0 stdout='4097' stderr=''
probe=Map exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
probe=Apply exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
probe=ReplaceAll exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
probe=Count exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
probe=Position exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
probe=Level exit=250 stdout='' stderr="thread 'main' has overflowed its stack | fatal runtime error: stack overflow"
```

At 1,024 wraps the same bounded invocations returned, respectively, `f`, `f`,
`List`, `0`, `List`, and `List`, all with exit 0. The exact probe expressions
were `Head[Map[f,x,{0,Infinity}]]`, `Head[Apply[f,x,{0,Infinity}]]`,
`Head[x /. 999 -> 0]`, `Count[x,999,{0,Infinity}]`,
`Head[Position[x,999,{0,Infinity}]]`, and
`Head[Level[x,{0,Infinity}]]`; their owners are the paths cited above.

The codec-floor check was:

```text
$ ./target/release/rum-cli --eval 'sx = StringJoin[StringRepeat["<a>", 256], StringRepeat["</a>", 256]]; sj = StringJoin[StringRepeat["[", 256], "0", StringRepeat["]", 256]]; {Depth[FromXml[sx]], Depth[FromJson[sj]]}'
List[513, 257]
```

That result is consistent with XML's nested `XmlElement[..., {child}]` builder
and the pinned 256-level accept boundary
(`crates/rum-stdlib/src/xml/encode.rs:253-269`;
`crates/rum-stdlib/src/tests/codecs_xml.rs:276-321`).

#### Per-option consequences

| Option | Exact consequence for `x=1; Do[x={x},{i,N}]` | Other currently successful behavior | Failure/reporting behavior |
|---|---|---|---|
| A, maximum physical height `L` | `N=L-1` constructs height `L`; iteration `N=L` attempts height `L+1` and must fail before assignment. The prior `x` remains height `L` and a later request must still work. This follows from the current `Depth` recurrence and Set receiving an evaluated rhs (`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`). | `Depth[x]==80001`, its `LeafCount`, and `ByteCount` cease to be reachable when `L<80001`. With candidate `L=1024`, 1,023 wraps succeed and the 1,024th is refused; codec depth 256 remains representable, but seven 4,096-link nested chains no longer lower to runtime values (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:562-587`). | Core returns `ExprDepthError { limit, attempted }`; runtime maps it to a specific `EvalError`/termination reason and fixed `General::reslim` text. No offending value is embedded in the message, so rendering the failure cannot repeat the traversal (`crates/rum-runtime/src/runtime/session/session_helpers.rs:311-383`). |
| B, all relevant traversals iterative | Construction and `Depth[x]==80001` remain valid. Each converted operation keeps working at 80,000 subject to its existing work/memory/output limits; the operation should not acquire a new depth refusal merely because its implementation changed (`docs/testing-strategy.md:39-55`). | All currently successful deep operations are intended to stay successful. Heap work stacks add O(depth) temporary storage; width and output size remain independently bounded concerns (`docs/guide/resource-limits-and-sandbox.md:71-88`). | `ToString`, message, trace, and host rendering remain total; no depth error is needed. Resource ceilings may still terminate through their existing structured paths (`crates/rum-runtime/src/runtime/model/requests.rs:56-95`). |
| C, guard `G` | The 80,001-deep value remains stored. A guarded operation accepts paths through depth `G` and returns `TraversalDepthLimit` before edge `G+1`; an iterative `Depth` could still return 80,001. An unguarded operation can still abort (`crates/rum-core/src/expr.rs:749-753`). | With `G=1024`, shallow `Head`/`Length` continue to work on the deep root, while guarded `ToString`, Map/rewrite/walkers refuse it. This is a new per-operation split, not a value invariant (`crates/rum-stdlib/src/structure_ops/mod.rs:98-113`). | User `ToString` needs a real error channel. Message/trace infrastructure needs a separate bounded-total renderer with a fixed omission marker; preserving canonical host text instead requires the B-style iterative renderer (`crates/rum-runtime/src/runtime/messages/message_format.rs:34-81,124-178`; `crates/rum-host/src/model/outcome.rs:164-173,355-397`). |
| D, hard `L` plus selective iteration | Same construction semantics as A. Candidate `L=1024` has the exact A consequence above; preserving the 4,096-link nested-chain contract instead requires `L>=4097` (`docs/guide/resource-limits-and-sandbox.md:125-135`; `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:562-587`). | Iterated renderer/infrastructure continues to work for every admitted value. Any traversal observed to fail at or below `L` must join the iterative tranche before the cap ships; the 4,096 live check already names at least seven such families when `L>=4097`. | The construction error is fixed/numeric; infrastructure rendering is total. This prevents the “error while rendering another error” recursion without inventing truncated canonical host text (`crates/rum-host/src/model/outcome.rs:164-173,386-397`). |

The candidate boundary is therefore a product ruling, not an engineering freebie:
`L=1024` preserves the codec floor but changes the 4,096-link nested-chain
contract; `L>=4097` preserves that contract but requires a substantial B tranche.
The current repository already treats held/inert shape and documented limits as
contract axes, so the change must be explicit in tests and documentation
(`docs/testing-strategy.md:39-55,60-72`).

### 5. Cost gates and recoverable rejected-`Drop` evidence

I ran **no microbench timing** in this design-only brief: there is no candidate
binary, and ad-hoc wall timing would not satisfy the project gate
(`docs/testing-strategy.md:74-85`). The CLI runs above are only pass/abort
observations. No profiling environment was used for a timing claim.

The repository preserves only aggregate historical `Drop` ratios, not the raw
per-label rows:

- safe `Option<Arc>`: recorded +8–44%, attributed to the hot dereference branch
  (`docs/dev/codex/brief-057-safe-iterative-drop-design-A.md@cb4c9d33:5-9`;
  `PROGRESS.md@cb4c9d33:21-29`);
- safe sentinel/manual drain: recorded +22–49%, including the `strong_count`
  fast path (`PROGRESS.md@cb4c9d33:24-29`);
- first reverted unsafe/O(N)-probe design: about +10% on list operations
  (`docs/dev/codex/brief-056-safe-iterative-drop.md@cb4c9d33:5-12`).

The historical briefs name the watched labels but do not retain their seconds or
map a particular ratio to a particular label: `length-list`, `part-list`,
`head-list`, `sameq-list`, `do-small`, `table-small`, `module-local`, and (for
the sentinel rerun) `downvalue-exact-miss-repeat`
(`docs/dev/codex/brief-056-safe-iterative-drop.md@cb4c9d33:72-84`;
`docs/dev/codex/brief-057-safe-iterative-drop-design-A.md@cb4c9d33:74-81`).
Those labels still exist in the current suite
(`scripts/core-language-microbench.sh:473-506`). I could not recover the old
`measurements.tsv` files from repository history or `/tmp`, nor could I establish
the historical ambient profiling environment. The aggregate ranges are thus a
repository-recorded prior, not measurements I independently certify.

For A/D cached construction, the mandatory panel is:

- `literal-integer`, `literal-string`, `list-literal-small`, `head-list`,
  `length-list`, `part-list`, and `sameq-list`, covering atom/call construction,
  wrapper access, and structural equality
  (`scripts/core-language-microbench.sh:463-478`);
- `module-local`, `with-local`, `block-local`, `table-small`, and `do-small`,
  covering repeated short-lived call construction/destruction
  (`scripts/core-language-microbench.sh:480-500`);
- `downvalue-exact-hit`, `downvalue-exact-miss-repeat`,
  `matchq-integer-success`, `matcher-single-tiny`, and `replaceall-simple`,
  covering dispatch/matcher/rewrite neighbors
  (`scripts/core-language-microbench.sh:502-507,538-558`);
- `cases-wide-no-match`, `count-wide-no-match`, `position-wide-no-match`, and
  `replace-wide-no-match`, because cached metadata/mutation changes shared
  argument construction along these traversals
  (`scripts/core-language-microbench.sh:569-598`).

The representation gate must additionally print `size_of::<Expr>()`,
`size_of::<SharedExpr>()`, `size_of::<SharedArgs>()`, and
`size_of::<WeakSharedArgs>()`, run the core memory panel, and update
modeled-memory formulas if the Arc allocations grow. The
runtime and four stdlib proxy walkers currently calculate expression memory in
separate owners
(`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-400`;
`crates/rum-runtime/src/runtime/arithmetic/canonical.rs:393-416`;
`crates/rum-stdlib/src/arithmetic/stats.rs:52-74`;
`crates/rum-stdlib/src/associations/missing.rs:212-227`;
`crates/rum-stdlib/src/strings/similarity/mod.rs:472-485`;
`crates/rum-stdlib/src/structure_ops/surface.rs:261-282`).

For the renderer tranche, the current core panel has no direct
`ToString`/`FullForm`/`Print` labels in the relevant label blocks, so the brief
must add exact-statement rows such as `tostring-shallow`, `fullform-shallow`,
`print-shallow`, and `tostring-depth-256`, then compare their same-tier
neighbors; existing traversal rows alone do not measure rendering
(`scripts/core-language-microbench.sh:463-613`). For later B tranches, use the
existing rewrite/cases/count/position labels above and add exact Map, Apply,
Scan, Level, Depth, LeafCount, and ByteCount rows before claiming neutrality.
Every number must come from the in-kernel suite with profiling variables off
(`docs/dev/codex/TEMPLATE.md:78-100`).

### 6. Sequence and drafted follow-up briefs

The recommended sequence is deliberately gated:

1. Measure every inventory bucket and obtain the director's numeric/behavior
   ruling. The present evidence supports testing `L=1024`, but does not prove it
   (`docs/dev/codex/brief-423-traversal-depth-design.md:101-110,131-141`).
2. Make rendering and trace/message infrastructure total and iterative first;
   this removes the lowest observed failure and gives every later failure a safe
   reporting path (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:19-34`;
   `crates/rum-runtime/src/runtime/messages/message_format.rs:124-178`).
3. Add cached physical-height metadata as a behavior-preserving measured spike.
   Stop the A/D line if construction or memory labels regress; the project does
   not trade a hot-path regression for a pathological-path benefit
   (`docs/dev/codex/TEMPLATE.md:93-100,140-146`).
4. Introduce checked core construction, then migrate runtime and stdlib/host in
   compiler-driven tranches. Do not claim the invariant until the legacy
   infallible/raw-mutation routes are removed
   (`crates/rum-core/src/expr.rs:576-715,813-829`).
5. Prove the chosen boundary through core, source, suspension, and host surfaces;
   only then correct the false documentation. The documentation correction is a
   named final step because behavior and its normative text must change together
   (`docs/testing-strategy.md:60-72`).

The drafts below are raw `TEMPLATE.md`-shape material for director review, not
dispatch-ready briefs; that is the repository's plan-first workflow
(`docs/dev/codex/TEMPLATE.md:17-37`). `[DIRECTOR-SET L]` must be replaced after
the behavior ruling.

#### Draft brief 423A — diagnose the admissible physical-height boundary

##### Goal

Produce the complete evidence table needed to choose a fixed
`MAX_PHYSICAL_EXPR_DEPTH`: for every manually reconciled traversal bucket,
bracket pass/failure on an explicit 2 MiB thread in debug and release, establish
the minimum height required by current documented contracts, and report a
recommended `L` without changing the repository. Existing evidence is only a
representative release bracket, not full coverage
(`docs/dev/codex/brief-422-xml-depth-and-the-class.md:267-280,348-360`).

##### Context pointers

- Start from the reconciled inventory in this report and brief-422 §3(e); do not
  rerun a recursive-function regex as an inventory
  (`docs/dev/codex/brief-422-xml-depth-and-the-class.md:232-280`).
- Preserve the codec accept boundaries and explicitly price the current
  4,096-link nested-expression contract
  (`crates/rum-stdlib/src/tests/codecs_xml.rs:276-321`;
  `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:562-587`).
- Use the repository's 2 MiB test-thread precedent rather than assuming the
  release main-thread stack (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:375-382`).

##### Scope

- Files MAY modify: **NONE — diagnostic only**. A throwaway crate under
  `/tmp/rum-423a-*` is allowed and must be removed.
- MUST NOT modify repository source, tests, docs, `Cargo.lock`, `.git`, or
  `stash@{0}`; do not run another vendor engine
  (`docs/dev/codex/brief-423-traversal-depth-design.md:143-156`).

##### Deliverable

- A manual owner/function manifest for all 45 planning units above, with
  overlaps explicitly reconciled and each row `path:line` sourced.
- One-process-per-probe pass/failure brackets at physical heights 513, 1,024,
  1,025, 2,048, and 4,097 where the owner is reachable; use a throwaway Rust
  harness for core traits and direct APIs and the release CLI for guest heads.
- Exact debug/release outputs for `Clone`, equality, hashing, serde
  encode/decode, `Display`, natural drop, renderer, every generic walker family,
  and one representative of each specialist stdlib bucket.
- A contract-floor table: XML 256 nesting, JSON/YAML/TOML 256 nesting, parser
  depth 128, host typed depth 64, and nested left-chain 4,096
  (`crates/rum-host/src/model/outcome.rs:11,473-560`;
  `docs/guide/resource-limits-and-sandbox.md:111-153`).
- A recommendation of either a numeric `L`, a list of iterative prerequisites
  for a higher `L`, or “could not establish”; never infer a threshold from one
  stack size.

##### rum / Rust pitfalls

- Physical height includes raw heads, association keys/rules, and every owned
  argument; user `Depth` is not the metric
  (`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`).
- A stack overflow aborts the process, so isolate every potentially failing
  point in its own subprocess; `catch_unwind` is not a safety boundary for it
  (`docs/host-embedding-and-hardening.md:67-71`).
- Do not raise a native stack or move work to a larger thread. The only explicit
  stack setting is the 2 MiB lower-bound probe required by this design
  (`docs/dev/codex/brief-423-traversal-depth-design.md:149-150`).
- Count owners manually and cross-check representative rows live; do not sum
  regex results (`docs/dev/codex/brief-423-traversal-depth-design.md:131-141`).

##### Verification

Run bounded commands only, with every Cargo invocation offline:

```text
scripts/run-bounded.py --timeout 300 -- cargo build --offline --release -p rum-cli
scripts/run-bounded.py --timeout 600 -- cargo run --offline --manifest-path /tmp/rum-423a-probe/Cargo.toml --release -- <one isolated row>
scripts/run-bounded.py --timeout 600 -- cargo run --offline --manifest-path /tmp/rum-423a-probe/Cargo.toml -- <one isolated row>
```

Paste every boundary row verbatim, state the finite probe count before running,
and remove `/tmp/rum-423a-*`. No test-suite claim is required for a no-source
diagnostic (`docs/dev/codex/TEMPLATE.md:102-105,147-159`).

##### Hard rules

- No repository changes, Git mutations, network, larger-stack workaround, or
  `unsafe`; use only safe Rust and isolated bounded probes
  (`docs/dev/codex/brief-423-traversal-depth-design.md:143-156`).
- Profiling variables are off for any timing; this brief needs pass/failure and
  memory/layout evidence, not ad-hoc timing
  (`docs/dev/codex/TEMPLATE.md:78-100`).
- Tests/probes establish rum's own contract only
  (`docs/testing-strategy.md:3-14`).

##### Report format

1. Environment and finite probe budget, verbatim.
2. Reconciled manifest and live boundary matrix.
3. Contract floor and recommended `L` (or explicit unknown).
4. `/tmp` cleanup, risks, and director rulings required.

#### Draft brief 423B — make all expression rendering infrastructure iterative and total

##### Goal

Remove native recursion from core `Expr` display, the syntax-aware input/output
renderer, transparent-hold handling, and structured trace-line collection while
preserving every in-bound byte of current output. `ToString`, `FullForm`, Print,
message formatting, Save/snapshot, trace conversion, and host success rendering
must not abort while traversing the 80,000-wrap source-built value on the normal
release process, and must pass the isolated 4,096-wrap renderer boundary on a
2 MiB stack; this brief introduces no depth refusal and no expression
representation change
(`crates/rum-core/src/expr.rs:850-891`;
`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:19-34,409-441`).

##### Context pointers

- The public renderer signatures and export stay `String`-returning
  (`crates/rum-runtime/src/runtime/kernel_api.rs:121-124`).
- Preserve every precedence/special-form branch in the current worker, including
  `MessageName`, slots, functions, holds, rules, and compound expressions
  (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:34-115,380-407`).
- Message, Print, snapshot, trace, and host conversions are infrastructure
  consumers and therefore part of the end-to-end gate
  (`crates/rum-runtime/src/runtime/builtins_input.rs:104-111,430-434`;
  `crates/rum-host/src/model/outcome.rs:355-397`).

##### Scope

- MAY modify:
  - `crates/rum-core/src/expr.rs` and its tests;
  - `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs`;
  - `crates/rum-runtime/src/runtime/debug_tools/trace.rs` and
    `debug_tools/shared.rs`;
  - `crates/rum-runtime/src/runtime/scoping/packages.rs`,
    `session/session_api.rs`, and `package_tools.rs` for the sealed-symbol and
    persistence walks used by Save/snapshot;
  - renderer/message/trace/session tests under
    `crates/rum-runtime/src/runtime/tests/**`;
  - `crates/rum-stdlib/src/strings/text/builtins.rs`,
    `crates/rum-stdlib/src/structure_ops/surface.rs`, and focused stdlib tests;
  - `crates/rum-host/src/model/outcome.rs` and focused host tests;
  - `scripts/core-language-microbench.sh` for the named renderer labels.
- MUST NOT change `Expr`/`SharedExpr`/`SharedArgs` representation or public
  signatures, add truncation/refusal behavior, change canonical text, edit docs,
  or add `unsafe` (`crates/rum-core/src/expr.rs:573-753`).

##### Deliverable

- One explicit action/frame stack for `Expr::fmt` that emits the current order
  without recursively formatting children.
- One explicit precedence-aware render machine; fold transparent-hold behavior
  into that machine or convert its rebuild to explicit frames, so it cannot
  remain the hidden recursive prepass
  (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:409-441`).
- Iterative structured trace-line collection
  (`crates/rum-runtime/src/runtime/debug_tools/trace.rs:777-813`).
- Iterative sealed-symbol discovery so Save/snapshot does not retain a recursive
  pre-render traversal (`crates/rum-runtime/src/runtime/scoping/packages.rs:16-34`;
  `crates/rum-runtime/src/runtime/session/session_api.rs:390-448`).
- Exact-output tests for every renderer branch, message placeholders, Print,
  snapshot text, trace text, and `HostValue.text`.
- Normal-release probes at 80,000 wraps plus dedicated 2 MiB release
  tests/probes at 4,096 wraps for `ToString`, `FullForm`, Print, a message
  containing the deep value, trace conversion, Save/snapshot, and host success
  conversion. The smaller 2 MiB case isolates rendering while current natural
  drop is independently known to pass at that boundary
  (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:562-587`).
- New exact microbench labels `tostring-shallow`, `fullform-shallow`,
  `print-shallow`, and `tostring-depth-256`, with base/new rows.

##### rum / Rust pitfalls

- `fmt::Display` has no domain-error channel; it must be total for the admitted
  value rather than use `fmt::Error` as depth control
  (`crates/rum-core/src/expr.rs:850-873`).
- Preserve left-to-right output and all parentheses. Use visit/emit frames; do
  not build a recursively nested intermediate string vector
  (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:385-404`).
- A deep value in a diagnostic must not cause a second render traversal through
  a fallback path (`crates/rum-runtime/src/runtime/messages/message_catalog.rs:143-169`).
- Rendering work is not permission to raise the stack or change host canonical
  text (`crates/rum-host/src/model/outcome.rs:164-173`).

##### Verification

Before editing, build/copy a baseline CLI without using a stash. Then run:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-core
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-stdlib
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
scripts/run-bounded.py --timeout 1800 -- bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum-423b-base --output-dir /tmp/rum-423b-base-rows
scripts/run-bounded.py --timeout 1800 -- bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum-423b-new --output-dir /tmp/rum-423b-new-rows
```

Run the deep deployment probes on explicit 2 MiB subprocesses and paste their
exit/output lines. Compare all four new labels plus same-tier string, list, and
message neighbors; profiling variables must be off
(`docs/dev/codex/TEMPLATE.md:78-100,107-123`).

##### Hard rules

- Safe Rust only; no stack-size increase, background larger-stack thread,
  output truncation, or behavior change
  (`docs/dev/codex/brief-423-traversal-depth-design.md:149-156`).
- Any exact-output mismatch or reproducible microbench regression is a defect,
  not an allowed safety trade (`docs/testing-strategy.md:60-85`).
- No Git mutation, network, or files outside the MAY list and `/tmp` artifacts,
  which must be removed.

##### Report format

1. Frame-machine design and files changed.
2. Exact-output and 2 MiB evidence, verbatim.
3. Base/new renderer labels and same-tier neighbors.
4. Remaining recursive renderer entrypoints (must be empty) and open risks.

#### Draft brief 423C — cached physical-height metadata, measured before enforcement

##### Goal

Add O(1) physical-height lookup to `Expr` through metadata in the private shared
allocations, preserve all current construction behavior and serialized logical
shape, and measure representation, modeled-memory, and hot-path cost. This is a
stop-loss spike: it does **not** enforce `L`; if the cache is not neutral, remove
it and report that D is rejected
(`crates/rum-core/src/expr.rs:573-715,749-829`).

##### Context pointers

- `SharedExpr`, `SharedArgs`, and `WeakSharedArgs` are exported pointer wrappers
  and `Expr` is the cross-crate value model
  (`crates/rum-core/src/lib.rs:1-14`).
- Raw shared-argument mutation currently bypasses any cache maintenance
  (`crates/rum-core/src/expr.rs:660-670`).
- Memory proxy formulas have multiple owners and must reflect any new allocation
  bytes (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-400`).

##### Scope

- MAY modify:
  - `crates/rum-core/src/expr.rs` and **module root
    `crates/rum-core/src/lib.rs`** because exported value types change;
  - rum-core tests;
  - `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs`,
    `crates/rum-runtime/src/runtime/definitions/part_assignment.rs`, and
    `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` for the current
    raw mutation callers;
  - the runtime and stdlib modeled-memory owners listed in section 5;
  - focused memory tests and both core microbench scripts.
- MUST NOT reject any value depth, change `Expr::call` return types, add a runtime
  error, change logical serde data, touch renderer semantics, or add `unsafe`.

##### Deliverable

- Saturating cached raw physical height in Arc inner allocations, with
  `Expr::physical_height()` O(1) over the call's two wrapper metadata values.
- A mutation API that cannot leave stale metadata; replace raw `&mut Vec<Expr>`
  exposure and migrate its current production/test callers
  (`crates/rum-core/src/expr.rs:660-670`).
- Manual serde implementations/tests proving the logical serialized form is
  unchanged and the cache is recomputed rather than trusted from input.
- Property-style core tests comparing cached height with an independent
  iterative walker across atoms, heads, wide calls, associations, shared DAGs,
  clone/unwrap, mutation, and serialization round trips.
- Verbatim sizes for `Expr`, both strong wrappers, `WeakSharedArgs`, both Arc
  inner allocations, and the modeled-memory deltas.
- Core-language and core-memory A/B rows. At minimum include every A/D label in
  section 5.

##### rum / Rust pitfalls

- Do not compute a child's height by recursive scan; the Do-wrapping path would
  become quadratic. Shared subtree height must be a metadata read
  (`crates/rum-core/src/expr.rs:576-603,640-683`).
- Do not use user `Depth`; physical ownership includes all heads/arguments
  (`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`).
- Preserve pointer identity, weak-pointer behavior, clone-on-write, and
  `ptr_key`; evaluation caches depend on those APIs
  (`crates/rum-core/src/expr.rs:596-603,673-697`).
- Metadata cannot add a branch to every dereference; that exact shape previously
  recorded +8–44% (`docs/dev/codex/brief-057-safe-iterative-drop-design-A.md@cb4c9d33:5-9`).

##### Verification

Build the baseline before editing; profiling variables are explicitly unset for
all timing runs. Then run bounded offline format/tests, full workspace tests,
the core-language A/B, and the core-memory A/B
(`docs/testing-strategy.md:74-85`):

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-core
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-stdlib
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
scripts/run-bounded.py --timeout 1800 -- bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum-423c-base --output-dir /tmp/rum-423c-base-rows
scripts/run-bounded.py --timeout 1800 -- bash scripts/core-language-microbench.sh --engine rum --rum-cli /tmp/rum-423c-new --output-dir /tmp/rum-423c-new-rows
scripts/run-bounded.py --timeout 1800 -- bash scripts/core-memory-microbench.sh --rum-cli /tmp/rum-423c-base --output-dir /tmp/rum-423c-base-memory
scripts/run-bounded.py --timeout 1800 -- bash scripts/core-memory-microbench.sh --rum-cli /tmp/rum-423c-new --output-dir /tmp/rum-423c-new-memory
```

##### Hard rules

- Stop and remove the spike after two representation attempts with the same
  hot-path regression, or on any reproducible hot/same-tier regression; do not
  carry a cost forward on the promise that the later cap justifies it
  (`docs/dev/codex/TEMPLATE.md:96-100,140-150`).
- Safe Rust only. No `unsafe`, subtree scan, stale cache, serde-shape drift,
  network, Git mutation, or larger stack.
- Leave the tree uncommitted and paste real per-label rows and size outputs.

##### Report format

1. Metadata/mutation/serde design and exact sizes.
2. Correctness and workspace outputs.
3. Language and memory A/B rows, including worst neighbor.
4. Verdict: neutral and usable, or reverted/rejected with exact evidence.

#### Draft brief 423D — introduce checked core construction at the director-set limit

##### Goal

On top of a neutral 423C cache, add `ExprDepthError` and parallel fallible
constructors/conversions that reject a child at physical height `L` before an
outer height `L+1` can exist. Preserve the legacy APIs temporarily so the
workspace can migrate in reviewable tranches; do not yet claim the global
invariant (`crates/rum-core/src/expr.rs:576-715,813-829`).

##### Context pointers

- All recursive call edges are `SharedExpr`/`SharedArgs`, but direct enum
  construction remains public (`crates/rum-core/src/expr.rs:749-753`).
- Derived deserialization and raw mutation are construction paths too
  (`crates/rum-core/src/expr.rs:573-574,637-670,749-753`).
- The core crate must remain runtime-neutral; the error carries only limit and
  attempted height, not `EvalError` or a `Session`
  (`ARCHITECTURE.md:184-195`).

##### Scope

- MAY modify `crates/rum-core/src/expr.rs`, **module root
  `crates/rum-core/src/lib.rs`** (new exported error/API on exported types), and
  rum-core tests.
- MAY modify no runtime, stdlib, host, syntax, docs, or scripts except the
  brief's own report; if another file is required, stop and report the missing
  scope.
- MUST NOT remove or silently change legacy infallible APIs in this tranche,
  panic/substitute on overflow, alter atom semantics, or add `unsafe`.

##### Deliverable

- Public `ExprDepthError { limit, attempted }` with no embedded `Expr` and a
  depth-independent `Display`.
- Checked shared-edge constructors, `try_call` variants, checked serde seeds,
  and checked transactional argument edits using `[DIRECTOR-SET L]`.
- Proof tests: atom, exact `L`, `L+1`, direct public `Expr::Call` using bounded
  wrappers, wide/shallow calls, shared subtree reuse, mutation rollback, and
  malicious nested deserialization. No test may require a larger stack.
- A repository call-site manifest separating provably shallow legacy calls from
  guest-data-dependent calls for briefs 423E/423F.

##### rum / Rust pitfalls

- A wrapper must admit children only below `L`; checking only `Expr::try_call`
  leaves direct `Expr::Call` as a one-level bypass
  (`crates/rum-core/src/expr.rs:749-753`).
- `From<Expr>`/`From<Vec<Expr>>` cannot report failure and therefore need
  `TryFrom` replacements in the final cutover
  (`crates/rum-core/src/expr.rs:611-615,711-715`).
- The deserializer must check before its next recursive seed, not build an
  arbitrary tree and validate afterward
  (`crates/rum-core/src/expr.rs:573-574,637-638,749-753`).

##### Verification

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-core
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
```

Rerun the full 423C hot panel against its retained baseline; no new
construction-path regression is allowed (`docs/dev/codex/TEMPLATE.md:140-146`).

##### Hard rules

- Safe Rust, fixed hard maximum, no session/global mutable policy, no panic or
  sentinel expression, no Git/network/larger-stack action.
- Do not claim source safety or correct the docs while legacy infallible paths
  remain; this brief is migration scaffolding.
- Every exported-type change keeps `crates/rum-core/src/lib.rs` in the MAY list,
  as required by brief-423 (`docs/dev/codex/brief-423-traversal-depth-design.md:123-129`).

##### Report format

1. Checked API and invariant proof.
2. Exact boundary/serde/mutation outputs.
3. Retained A/B rows.
4. Complete migration manifest and remaining bypasses.

#### Draft brief 423E — migrate runtime construction and add a structured depth termination

##### Goal

Migrate every runtime construction path that can wrap guest-controlled values to
the checked core API, propagate `ExprDepthError` as a specific
`EvalError::ExpressionDepthLimitExceeded`, and expose one structured
`ExpressionDepthLimit` termination through suspension and request APIs. A
source Do loop must fail before storing height `L+1`, preserve the height-`L`
ownvalue, and leave the session usable
(`crates/rum-runtime/src/runtime/session/session_state.rs:498-540`;
`crates/rum-runtime/src/runtime/session/session_helpers.rs:311-383`).

##### Context pointers

- The call evaluator and preparation paths directly construct shared heads/args
  (`crates/rum-runtime/src/runtime/eval_core/call.rs:34-135`;
  `crates/rum-runtime/src/runtime/eval_call_prepare.rs:328-456`).
- Error propagation has exhaustive runtime, continuation, request, and exported
  termination surfaces (`crates/rum-runtime/src/runtime/suspension/error.rs:269-339,430-539`;
  `crates/rum-runtime/src/runtime/model/requests.rs:56-85,157-211`).
- The error message must not contain the rejected expression; current top-level
  limit mapping uses fixed error text and structured reasons
  (`crates/rum-runtime/src/runtime/session/session_helpers.rs:322-383`).

##### Scope

- MAY modify runtime construction owners in:
  - `runtime/eval_core/**`, `eval_call_prepare.rs`, `call_preparation.rs`,
    `resolution.rs`, and `lowering/**`;
  - `runtime/substitution/**`, `pattern_matching/**`, `kernel_data/**`,
    `tree_walkers/**`, `suspension/**`, and `debug_tools/**` only where the 423D
    manifest identifies a guest-data-dependent constructor;
  - `runtime/session/session_state.rs`, `session/session_helpers.rs`,
    `session/session_requests.rs`, and `model/requests.rs`;
  - **module roots `crates/rum-runtime/src/runtime/mod.rs`,
    `crates/rum-runtime/src/runtime/kernel_api.rs`, and
    `crates/rum-runtime/src/lib.rs`**, because exported error/termination types
    change;
  - focused runtime tests and the core-language microbench script.
- MUST NOT modify stdlib/host/core representation, remove legacy core APIs,
  correct docs, change unrelated language semantics, or add `unsafe`.

##### Deliverable

- Exact `ExprDepthError -> EvalError -> EvalTerminationReason ->
  SessionRequestError` propagation with numeric/static messages.
- Continuation encode/decode round trips for the new error/reason
  (`crates/rum-runtime/src/runtime/suspension/error.rs:269-339,430-539`).
- Runtime construction call sites migrated or explicitly proven shallow in the
  report; no blanket `expect` on guest-controlled children.
- Exact boundary runtime tests: `L-1` wraps succeed, wrap `L` returns
  `TerminatedEvaluation["ExpressionDepthLimit"]`, `x` remains height `L`, and a
  following `1+1` returns 2. Cover held input, definitions, rewriting,
  suspension/resume, trace, and request APIs.
- If `[DIRECTOR-SET L] < 4097`, deliberately update the left-chain runtime tests
  in this tranche only after the director has ruled that behavior
  (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:220-239,562-587`).

##### rum / Rust pitfalls

- Preserve suspend/resume frames while adding `Result` propagation
  (`docs/dev/codex/TEMPLATE.md:90-95`).
- The depth error itself and its continuation representation must stay shallow;
  never attach the rejected value.
- An infallible helper may use an assertion only when its inputs are statically
  bounded and the proof is documented at that call site; guest data propagates
  `Result`.
- The renderer from 423B is a prerequisite, not a reason to stringify the
  rejected value.

##### Verification

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
```

Run exact `L`/`L+1` release and 2 MiB probes and the A/D hot-label panel; paste
termination value, message, retained `x`, and session-survival output verbatim.

##### Hard rules

- Safe Rust; no stack workaround, recursion-limit reuse, generic panic, offender
  rendering, Git/network action, or docs change.
- Keep all three runtime roots in scope whenever exported types change
  (`crates/rum-runtime/src/runtime/kernel_api.rs:76-92`;
  `crates/rum-runtime/src/runtime/mod.rs:70-73`;
  `crates/rum-runtime/src/lib.rs:7-9`).
- Stop on suspension-shape loss, an unscoped construction owner, or a hot-path
  regression; report instead of broadening scope silently.

##### Report format

1. Runtime call-site disposition and error flow.
2. Exact boundary/suspension/request evidence.
3. A/B rows and 2 MiB outputs.
4. Remaining legacy core/stdlib/host bypasses.

#### Draft brief 423F — migrate stdlib and host, then seal the core invariant

##### Goal

Migrate every stdlib and host construction path to checked edges, add the host
termination mapping, remove the legacy infallible shared/call conversions, and
make physical height `<= L` a safe-Rust invariant for every constructible
`rum_core::Expr`. After this brief, omitted traversal guards are safe by
construction and direct hosts receive `ExprDepthError` rather than a panic
(`crates/rum-core/src/lib.rs:1-14`; `crates/rum-host/src/model/outcome.rs:41-64`).

##### Context pointers

- Stdlib builtins generally already return `Result<Expr, EvalError>`, as
  `ToString` illustrates, but pure recursive helpers need signature propagation
  (`crates/rum-stdlib/src/strings/text/builtins.rs:624-645`).
- Typed host input already rejects container depth 64; preserve that lower
  contract while mapping the core hard limit
  (`crates/rum-host/src/model/outcome.rs:473-565`).
- Host success/trace conversion and public termination enums are exhaustive
  exported surfaces (`crates/rum-host/src/model/outcome.rs:24-64,338-397`).

##### Scope

- MAY modify:
  - stdlib implementation modules and tests identified by the 423D manifest;
  - `crates/rum-host/src/config.rs`, `model/outcome.rs`, session/tests, and
    **module root `crates/rum-host/src/lib.rs`** because exported host types
    change;
  - `crates/rum-core/src/expr.rs` and **module root
    `crates/rum-core/src/lib.rs`** to remove legacy exported constructors/
    conversions and seal mutation/serde;
  - runtime error mapping/tests and **runtime roots** only for compiler-forced
    final cutover adjustments;
  - affected syntax/lowering tests if the director selected `L < 4097`;
  - microbench scripts and focused tests.
- MUST NOT edit normative docs yet, add compatibility shims that permit an
  unbounded edge, use `unsafe`, or change unrelated builtin results.

##### Deliverable

- Zero remaining uses of removed infallible `SharedExpr`/`SharedArgs`/call edge
  construction; the compiler, not a regex count, is the final migration check.
- Custom checked deserialization and checked mutation with adversarial tests.
- `HostTerminationReason::ExpressionDepthLimit` and exact conversion/tests;
  typed depth 64 remains unchanged
  (`crates/rum-host/src/model/outcome.rs:11,41-64,338-351,554-560`).
- Core invariant tests on direct public `Expr::Call`, shared DAG reuse, serde,
  mutation, clone/hash/equality/display/drop, exact `L`, and `L+1` rejection.
- One 2 MiB matrix at exact `L` for every inventory bucket. Any row that fails
  is a blocker requiring its own iterative-conversion brief; do not lower `L`
  silently after the director ruling.
- Full language/memory A/B rerun against the pre-423C baseline.

##### rum / Rust pitfalls

- `Expr::Call` may stay public only if both wrapper values are impossible to
  construct with a child height `>= L`; test that exact bypass
  (`crates/rum-core/src/expr.rs:749-753`).
- Host `HostValue.text` remains canonical because 423B made rendering total; do
  not introduce an omission marker in the A/D design
  (`crates/rum-host/src/model/outcome.rs:164-173`).
- Preserve Arc pointer identity/COW behavior and memory accounting
  (`crates/rum-core/src/expr.rs:596-603,647-697`).
- A failed constructor must not partially mutate a definition, association, or
  ownvalue; boundary tests must inspect state afterward
  (`docs/testing-strategy.md:24-31,50-55`).

##### Verification

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-core
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 1200 -- cargo test --offline -p rum-stdlib
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
```

Run the exact-`L` 2 MiB matrix and both retained A/B panels with profiling off.
Paste all per-label rows and any ignored release-only test output verbatim
(`docs/dev/codex/TEMPLATE.md:102-123,140-150`).

##### Hard rules

- Do not declare the invariant while any legacy edge constructor/raw mutation/
  unchecked serde path remains. A compiler-clean migration plus adversarial
  tests is mandatory.
- Safe Rust only; no native-stack increase or larger-stack worker, no Git/network
  action, no external oracle, and no hot-path regression.
- The MAY list explicitly contains every module root exporting a changed type,
  per this brief's stop-loss rule
  (`docs/dev/codex/brief-423-traversal-depth-design.md:123-129`).

##### Report format

1. Final constructor API and proof that all recursive edges are bounded.
2. Runtime/stdlib/host behavior and exact boundary matrix.
3. Workspace, language A/B, memory A/B, and sizes.
4. Blocked iterative rows or remaining risks (empty before proceeding to docs).

#### Draft brief 423G — correct the public depth contract and close the false claim

##### Goal

Make the documentation state the implemented source **and** host behavior:
remove the false claim that source-created trees are already guarded, document
the fixed physical expression-height boundary and exact failure/termination,
and reconcile the parser/left-chain text with the director-selected `L`. This is
an explicit deliverable, not a footnote
(`docs/dev/codex/brief-423-traversal-depth-design.md:44-59,123-129`).

##### Context pointers

- The two mandated false copies are `TODO.md:1061-1065` and
  `docs/host-embedding-and-hardening.md:112-118`.
- A third current copy says source-built trees are guarded and must be corrected
  in the same step (`docs/runtime-evaluation-audit.md:165-167`).
- The guide currently promises exact 4,096-link parse/lower behavior and must
  describe any lower runtime-value ceiling honestly
  (`docs/guide/resource-limits-and-sandbox.md:111-153`).
- Documentation and tests are the normative contract and change together
  (`docs/testing-strategy.md:60-72`).

##### Scope

- MAY modify:
  - `TODO.md`;
  - `docs/host-embedding-and-hardening.md`;
  - `docs/runtime-evaluation-audit.md`;
  - `docs/guide/resource-limits-and-sandbox.md`;
  - README/host rustdoc only if 423F's exported surface requires a pointer;
  - affected documentation-contract tests.
- MUST NOT change implementation, limit values, generated builtin reference
  unless an already-merged catalog change mechanically requires regeneration,
  `.git`, or unrelated roadmap prose.

##### Deliverable

- Exact definition of physical height, fixed `L`, accepted/rejected boundary,
  source Do behavior, direct Rust constructor error, termination tag/message,
  session survival, and distinction from evaluator recursion/native-stack
  limits.
- Honest left-chain wording: either retain 4,096 with the iterative prerequisites
  proved, or distinguish parse acceptance from a lower runtime expression limit.
- Remove all three false “source-built trees are guarded” statements and replace
  host advice with the actual checked-constructor contract.
- Link the standing debt item to the completed design/implementation record
  without claiming compatibility with another engine
  (`DISCLAIMER.md:1-20`).

##### rum / Rust pitfalls

- Do not call physical height the `Depth` builtin; associations/heads differ
  (`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`).
- Do not describe a configurable native stack or larger-thread workaround; the
  safety property is the construction invariant
  (`docs/dev/codex/brief-423-traversal-depth-design.md:149-151`).
- Keep host panic/process-abort language accurate: stack overflow is not caught
  by the panic boundary (`docs/host-embedding-and-hardening.md:67-71`).

##### Verification

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
rg -n "Source-created trees are protected|Source-created trees are depth-guarded|Source-built trees are guarded" TODO.md docs
```

The `rg` command must print no stale claim; paste its empty output and the exact
documentation-contract test result. If generated reference drift appears, stop
and report rather than broadening scope silently
(`docs/testing-strategy.md:64-72`).

##### Hard rules

- Documentation only, except already-listed contract tests; no code behavior
  changes, Git/network action, external comparisons, or invented verification.
- Use the exact constant/outcome proven by 423F; do not write ahead of the
  implementation.
- Leave the tree uncommitted for the authoritative gate.

##### Report format

1. Each corrected contract location and its new truth.
2. Drift/workspace verification, verbatim.
3. Confirmation that all stale phrases are gone.
4. Open documentation questions (empty if none).

### 7. Risks and open questions

1. **The safe numeric `L` is not established.** I observed only release behavior
   for the renderer and six generic families on an explicit 2 MiB stack. The
   specialist stdlib, core derived traits, suspension codecs, debug builds, and
   different tree shapes can use different frame sizes; brief-422 already found
   that XML's shape costs roughly two evaluator cycles per level while JSON/TOML
   list chains do not (`crates/rum-stdlib/src/tests/codecs_xml.rs:304-313`). The
   1,024 candidate is therefore a hypothesis for 423A, not a safety claim.

2. **There is a real contract fork at 4,096.** Seven operators deliberately
   retain a 4,096-deep runtime tree and naturally drop it on a 2 MiB release
   thread (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:562-587`).
   A lower global value bound changes that documented acceptance; a bound high
   enough to retain it currently crosses observed failures in renderer/map/
   rewrite/tree-walker families. The director must choose which behavior is the
   contract before implementation (`docs/guide/resource-limits-and-sandbox.md:125-135`).

3. **A/D may fail the performance gate.** The cache avoids an O(subtree) scan,
   but it still adds per-call immediate-child work and allocation metadata at the
   hottest representation boundary. The previous `Drop` line demonstrates that
   apparently small wrapper costs can move list/dispatch labels by double digits
   (`PROGRESS.md@cb4c9d33:21-34`). No source argument can substitute for 423C's
   A/B.

4. **The public Rust/serde break needs an explicit versioning ruling.** `Expr`,
   `SharedExpr`, `SharedArgs`, and `WeakSharedArgs` are public cross-crate values;
   their constructors, `From` conversions, mutation/weak APIs, derives, and enum
   matching are visible now
   (`crates/rum-core/src/lib.rs:1-14`;
   `crates/rum-core/src/expr.rs:573-715,749-829`). I have not established whether
   downstream users require serde wire compatibility or an infallible builder.
   A compatibility shim that can create an unbounded edge would invalidate A's
   enforcement.

5. **The 45-unit B estimate is deliberately not exact.** It is a manually
   reconciled implementation-work estimate from brief-422's verified inventory,
   not a regex result or a promise that each unit maps to one function
   (`docs/dev/codex/brief-422-xml-depth-and-the-class.md:232-280,356-360`). 423A
   must turn it into an owner manifest before dispatching conversion tranches.

6. **Historical per-label `Drop` data are gone.** Repository history retains the
   aggregate +8–44%, +22–49%, and about-10% statements and the watched label
   names, but not seconds, individual ratios, or a recorded profiling-environment
   block (`docs/dev/codex/brief-056-safe-iterative-drop.md@cb4c9d33:72-84`;
   `docs/dev/codex/brief-057-safe-iterative-drop-design-A.md@cb4c9d33:74-81`;
   `PROGRESS.md@cb4c9d33:21-34`). I could not responsibly assign one old ratio to
   one label.

7. **Depth safety does not bound width, output bytes, or total traversal work.**
   A shallow expression may still be enormous, and an iterative renderer can
   allocate an enormous string. Current host resource documentation distinguishes
   modeled memory from process RSS and requires external process limits for hard
   RSS isolation (`docs/host-embedding-and-hardening.md:52-59,119-121`). This
   report closes only native recursion over height; output/work accounting is a
   separate design.

8. **Physical-height metadata must cover every ownership edge.** Association
   helpers present a logical value view that omits some raw structure, while
   traits/drop traverse the stored head and argument graph
   (`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`;
   `crates/rum-core/src/expr.rs:749-775`). The cache must be defined over raw
   storage, and its mutation/serde proof is a prerequisite, not a cleanup.

9. **Failure atomicity is not yet proved.** The desired Do behavior assumes the
   checked call fails before `Set` commits and that errors survive suspension and
   request conversion. Those are test requirements in drafts 423E/423F, not
   observations about code that does not yet exist. Current runtime errors and
   suspension codecs are exhaustive, which is why their full ripple is listed
   (`crates/rum-runtime/src/runtime/session/session_state.rs:498-540`;
   `crates/rum-runtime/src/runtime/suspension/error.rs:269-339,430-539`).

10. **The false docs remain false until the implementation decision lands.** I
    did not edit `TODO.md`, host docs, or the additional runtime-audit copy in
    this design-only scope. Draft 423G explicitly corrects all three after the
    boundary is real (`TODO.md:1061-1065`;
    `docs/host-embedding-and-hardening.md:112-118`;
    `docs/runtime-evaluation-audit.md:165-167`).

Final verdict: **D is the recommended structural route, conditional on a neutral
height-cache A/B and the director's depth ruling.** A alone forces the cap below
currently documented deep shapes or leaves known traversals unsafe; B alone is
a forty-plus-unit program with a known `Drop` regression; C alone has no answer
for infallible traits or omissions. If the director will not accept construction
refusal, the honest fallback is the full B program—there is no source-supported
cheap guard that preserves 80,001-deep values and closes this class
(`docs/dev/codex/brief-423-traversal-depth-design.md:61-75,82-110`;
`docs/dev/codex/brief-422-xml-depth-and-the-class.md:267-280`).
