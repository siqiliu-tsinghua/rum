# brief-427 — find the recursion behind the shared 4,500 ceiling

Shape: **diagnostic, source-level**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

**You do not need to reproduce any boundary.** I measured them all and they are
below. What is missing is the mechanism, and that is a question about the code.
Read, do not probe.

## What is measured, so you can spend the run on the why

brief-425 converted ten traversals to iterative walks. Six of them then share a
ceiling that the other four do not. All figures are mine, release, on a 2 MiB
stack, with values built as `x = 1; Do[x = {x}, {i, N}]`:

| Probe | Ceiling | What it tells us |
|---|---|---|
| `ToString[x]` | 43,000 | rendering is fine |
| `Level[x, {2}]` | >20,000 | shallow walk is fine |
| **`Level[x, Infinity]`** | **>20,000** | **a full-depth walk with no pattern is fine** |
| `Map[f, x]`, `Apply[f, x]` | 20,000 | fine |
| **`MatchQ[x, _]`** | **>20,000** | **one match against a deep value is cheap** |
| `SameQ[x, x]` | >20,000 | structural comparison is cheap |
| **`Cases[x, _, Infinity]`** | **4,500 / 4,700** | trivial pattern, still short |
| `Cases[x, _List, Infinity]` | 4,500 / 4,700 | same |
| `Position[x, 1]` | 4,500 / 4,700 | same |
| `Count`, `MemberQ`, `FreeQ`, `ReplaceAll` | 4,500 / 4,700 | same |

Three eliminations follow and I want them treated as settled:

- **Not the walk.** `Level[x, Infinity]` descends every level and clears 20,000.
- **Not the matcher's cost on a deep value.** `MatchQ[x, _]` clears 20,000.
- **Not the pattern.** `Cases[x, _, Infinity]` with the trivial `_` stops at the
  same 4,500 as `_List`.

What is left is the thing the six do and the four do not: **invoke the matcher
once per node while walking every level.** A single invocation is cheap and
4,500 of them do not fit a 2 MiB stack, which means each per-node invocation leaves
something on the native stack that is not released until the walk finishes.

## Deliverable 1 — the mechanism, from source

Find what that retained thing is. Compare, in the code:

- `Level` (`crates/rum-runtime/src/runtime/tree_walkers/level.rs`) — walks every
  level, no per-node match, clears 20,000;
- `Position` (`tree_walkers/position.rs`) and `count_like.rs` (`MemberQ`,
  `Count`, `FreeQ`) — same module, same post-order cursor
  (`tree_walkers/post_order.rs`, added by brief-425), but short;
- `Cases` (`kernel_data/rewrite_data/cases/collect.rs`) and `ReplaceAll`
  (`definitions/application/replace_all.rs`) — different modules, same ceiling.

Two of those live in the same module and use the same cursor and differ only in
whether they match, so the cursor is not the answer either. Say what is.

Answer concretely: which function, which frame, what it holds, and why the
iterative cursor does not release it per node. `path:line` for each claim.

## Deliverable 2 — whether it is fixable, and how expensive

Given the mechanism, say what would remove it, and price it against what this
line already knows:

- brief-425's conversions were free or better — `position-wide-no-match` got
  23.5% faster — so an iterative rewrite in this area has a good record.
- brief-426's cached height cost up to 30% because it put an O(n) scan in
  `SharedArgs::new`, and the stop-loss fired. **Do not propose anything that
  adds per-construction or per-node work to an ordinary path** without saying
  so plainly in the risks.

If the honest answer is that removing it is not worth what it costs, say that.
A ceiling of 4,500 already clears the ruled bound of 4,097; the reason to lift
it is margin, not correctness, and margin can lose to cost.

## Deliverable 3 — classify the untested tail by the same mechanism

brief-422's appended report §3(e) lists rows that were source-verified and never
measured: string-pattern machinery, calculus and polynomial walkers, schema
conformance, `Flatten[_, Infinity]`, tensor rank recursion, `Merge` nested,
substitution walks, matcher compile and structural match, rewrite traversals,
`lowering.rs`, and suspension continuation encoding.

Now that you know the mechanism, **classify each of those rows against it**:
does it invoke a per-node match inside an all-levels walk, does it recurse for
some other reason, or is it bounded already? This is a source-reading task and
it is what decides the next brief's scope, so I would rather have an honest
"unclear, needs measurement" per row than a confident guess.

## Method requirements

- **Every claim carries `path:line`.**
- **You are reading, not probing.** If you believe a specific claim cannot be
  settled from source, name it and say what measurement would settle it — do not
  construct one.
- **Cross-check any script-derived count against the source.** A regex over this
  repository has produced confidently wrong counts, and I have made the same
  class of error repeatedly on this line, most recently by measuring
  `Level[x, {2}]` and nearly concluding `Level` was fine for the wrong reason.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. If you
  believe a file must change, that is a finding for the report.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config.
  **`stash@{0}` is brief-426's rejected spike and `stash@{1}` is brief-400's
  WIP; leave both alone.**
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Append your report to this brief file** before finishing.

## Report format

1. Environment capability, verbatim.
2. The mechanism: function, frame, what it retains, why the cursor does not
   release it, with `path:line`.
3. What would remove it, and its price, with the brief-426 lesson applied.
4. The §3(e) classification table: row, mechanism, or "unclear — needs
   measurement".
5. Anything you could not establish from source, and what would settle it.

## Worker report (brief-427)

### 1. Environment capability, verbatim

```text
Filesystem sandboxing defines which files can be read or written. `sandbox_mode` is `workspace-write`: The sandbox permits reading files, and editing files in `cwd` and `writable_roots`. Editing files in other directories requires approval. Network access is restricted.

Approval policy is currently never. Do not provide the `sandbox_permissions` for any reason, commands will be rejected.

The writable roots are `/home/remi/dev/rum`, `/tmp`.
```

That capability matches the repository's documented `workspace-write`
dispatcher envelope of repository-plus-`/tmp` writes
(`docs/dev/codex/TEMPLATE.md:8-15`). I ran no runtime probe and no Cargo command:
this brief says “read, do not probe” and permits only this required appended
report (`docs/dev/codex/brief-427-per-node-match-recursion.md:6-8,89-94,101-111`).

### 2. The mechanism

**Finding: the source disproves the proposed retained-per-node-frame
mechanism.** A traversal invokes one match synchronously and handles its result
before the loop can ask the cursor for another node; this is explicit in
`Position`, `Count`/`MemberQ`/`FreeQ`, and `Cases`
(`crates/rum-runtime/src/runtime/tree_walkers/position.rs:198-231`;
`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:302-330`;
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:223-254`).
Consequently, completed invocations do not leave a growing chain of native
matcher frames behind; the one traversal-loop frame remains, and prior match
calls have returned at the end of each iteration
(`crates/rum-runtime/src/runtime/tree_walkers/position.rs:198-231`;
`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:302-330`;
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:223-254`).

The shared source-level recursion is instead **complete normalization of the
current match candidate**. On the raw-pattern paths, `Session::match_expr_owned`
compiles the pattern, recursively normalizes the complete candidate, and only
then enters the compiled matcher
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`). Pattern
compilation itself normalizes the pattern before calling `compile_matcher`
(`crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:304-306`),
but the measured `_`, `_List`, and integer patterns are shallow; the candidate
passed at the upper levels of the unary list is not
(`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:67-79`).

The exact recursive chain is:

| Function/frame | Live state while its child is visited | Recurrence |
|---|---|---|
| `normalize_match_expr` | The wrapper retains `expr` until `normalize_match_expr_changed` returns, then either accepts the rebuilt value or shallow-clones the original (`crates/rum-runtime/src/runtime/resolution.rs:62-65`). | One call into the recursive worker (`crates/rum-runtime/src/runtime/resolution.rs:62-65`). |
| `normalize_match_expr_changed` | For a call, the source frame has the borrowed `expr`/`head`/`args`, then `normalized_head: Option<Expr>` and `head_for_attributes` while it waits for argument normalization (`crates/rum-runtime/src/runtime/resolution.rs:99-115`). | It recursively normalizes the head, then calls the argument-list worker (`crates/rum-runtime/src/runtime/resolution.rs:103-106`). |
| `normalize_match_arg_list_changed` | The source frame retains borrowed `args`, borrowed `normalized_head`, the `enumerate` loop position, and `normalized_args: Option<Vec<Expr>>`; on a changed prefix that option also owns the rebuilt prefix (`crates/rum-runtime/src/runtime/resolution.rs:120-135`). | Each argument calls back into `normalize_match_expr_changed` before the loop can advance (`crates/rum-runtime/src/runtime/resolution.rs:127-135`); only after every child returns does the frame fetch `Flat`/`Orderless` attributes and normalize the call's arguments (`crates/rum-runtime/src/runtime/resolution.rs:137-153`). |

For a unary `List` spine, every call node therefore leaves one
`normalize_match_expr_changed` activation and one
`normalize_match_arg_list_changed` activation waiting while its sole argument
is descended; neither activation can return until the leaf has returned
(`crates/rum-runtime/src/runtime/resolution.rs:99-115,120-153`). The phrase
“frame” here means the source-level live activation; exact optimized frame
layout, inlining, and bytes per level are not derivable from Rust source alone
(`crates/rum-runtime/src/runtime/resolution.rs:99-153`).

The post-order cursor does not contribute a native recursion chain. Its pending
ancestors and computed depths are `Vec` fields, and `next` pops a work item in a
loop, scheduling children back into that heap-owned work vector
(`crates/rum-runtime/src/runtime/tree_walkers/post_order.rs:4-23,40-108,110-135`).
The path-bearing cursor used by `Position` has the same heap-owned `work`,
`depths`, and `paths`, and its `next` method is likewise a loop
(`crates/rum-runtime/src/runtime/tree_walkers/post_order.rs:138-186,189-269`).
By the time the caller starts a match, `next` has returned; the cursor can
neither flatten nor release frames created later inside the synchronous
normalizer (`crates/rum-runtime/src/runtime/tree_walkers/position.rs:198-209`;
`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:306-317`). Thus the
cursor *does* release its own native `next` activation per node; what remains
until the leaf is the current candidate's normalizer chain, not history from
earlier candidates
(`crates/rum-runtime/src/runtime/tree_walkers/position.rs:198-209`;
`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:306-317`;
`crates/rum-runtime/src/runtime/resolution.rs:99-153`).

The operation comparison closes the loop:

- `Level` first tries only sixteen levels of bounded recursion, then uses
  `PostOrderTraversal` and selects nodes without calling any matching entry
  point (`crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-87`).
- `Position` matches every selected node through `match_expr_owned`
  (`crates/rum-runtime/src/runtime/tree_walkers/position.rs:188-231`), and the
  default configuration selects all levels including heads
  (`crates/rum-runtime/src/runtime/model/traversal.rs:98-103`).
- The count-like driver uses the same cursor and the same general-match entry
  (`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:286-317`). `Count`
  and `MemberQ` default to exact level 1, not all levels
  (`crates/rum-runtime/src/runtime/model/traversal.rs:16-19,67-87`;
  `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:126-143,166-183`),
  so a single match of the outer list's sole child is already a match against
  an almost-full-depth candidate. Per-node accumulation and an all-level walk
  are therefore not even necessary to reach this recursion
  (`crates/rum-runtime/src/runtime/model/traversal.rs:16-19`;
  `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:302-317`).
- `Cases` uses the post-order cursor and calls `try_collect_case_owned` for each
  selected node (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:214-254`).
  That helper normalizes the candidate once before branching; its pattern arm
  then calls `match_expr_owned`, which normalizes the original candidate a
  second time (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:342-360`;
  `crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`). The scans are
  sequential, so this duplicates work but does not double the maximum live
  recursion depth (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:347-360`).
- `ReplaceAll` holds traversal work and completed results in explicit vectors
  (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:18-34`).
  Its first `Visit` synchronously calls `apply_rewrite_rule_plan_once`, which
  recursively normalizes that node before scanning compiled rules; the initial
  node is the deep root itself
  (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:122-134,168-190`).

The `MatchQ[x, _]` control does not exercise this path. `finish_match_q_with_pattern`
checks `fast_match_q_resolved_pattern` first and returns before
`match_expr_owned`; `pattern_blank_head` sends both `_` and head-constrained
blanks directly to `match_blank`
(`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:106-131`).
If a blank reaches the compiled matcher, its actual match is also constant-shape
work, but only *after* the general entry has normalized the candidate
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32,504-515`). Thus
the measured `MatchQ[x, _]` success is valid, but its interpretation as evidence
about the general matcher's deep-candidate path is not
(`docs/dev/codex/brief-427-per-node-match-recursion.md:22,29-34`;
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:106-131`).

`SameQ[x, x]` is likewise not evidence that every structural comparison walks
the tree: `SameQ` delegates to `Expr::eq`, whose call case first accepts pointer-
identical shared heads and argument vectors before recursive logical comparison
(`crates/rum-stdlib/src/predicates/ordering.rs:561-569`;
`crates/rum-core/src/expr.rs:755-775`). It may therefore be constant-depth for
shared clones while independently allocated equal trees still take the
recursive logical path (`crates/rum-core/src/expr.rs:573-608,637-708,755-775`).

**Mechanism verdict.** The common 4,500/4,700 figure is consistent with the
deepest selected candidate's single recursive normalization, not with 4,500
completed matches remaining on the stack: post-order callers eventually supply
an increasingly deep node (or, for level-1 `Count`/`MemberQ`, the one deep
child), while `ReplaceAll` supplies the root immediately
(`crates/rum-runtime/src/runtime/tree_walkers/post_order.rs:40-108`;
`crates/rum-runtime/src/runtime/model/traversal.rs:16-19`;
`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:168-190`).
Source establishes that call chain; it does not by itself establish the exact
numerical stack threshold (`crates/rum-runtime/src/runtime/resolution.rs:99-153`).

### 3. What would remove it, and the price

The narrow structural repair is to make
`normalize_match_expr_changed`/`normalize_match_arg_list_changed` a bottom-up
explicit-stack transform, with visit/finish tasks and completed child results,
while preserving all current copy-on-write outcomes and performing
`Flat`/`Orderless` normalization only after the normalized head and children
are available (`crates/rum-runtime/src/runtime/resolution.rs:99-153`). A bounded
recursive front path with iterative fallback is preferable to an always-
iterative rewrite: `Level` already uses a sixteen-call bounded front path before
its explicit cursor, and `Map`/`Apply` use the same depth-16/fallback structure
(`crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-87`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-61`). This would
bound native recursion in candidate normalization without changing expression
construction (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`;
`crates/rum-core/src/expr.rs:813-829`).

The price is not zero. An always-iterative normalizer would allocate and push/
pop heap work on every general match; a bounded front path avoids that allocation
for shallow candidates but still adds a remaining-depth decrement/branch at
each ordinary normalization call and restarts the deep candidate when it falls
back (`crates/rum-runtime/src/runtime/resolution.rs:99-153`;
`crates/rum-runtime/src/runtime/tree_walkers/level.rs:76-80`). This is a matcher
hot path and therefore requires a base/candidate A/B, not an assertion of “no
default-path cost” (`AGENTS.md:117-122`). The minimum panel should include every
`matchq-*` and `matcher-single-*` row plus `cases-wide-no-match`,
`count-wide-no-match`, `position-wide-no-match`, `replaceall-simple`, shallow
controls, and depth-65 certificates, matching the neighbor discipline used by
brief-425 (`docs/dev/codex/brief-425-iterative-traversals.md:432-468,476-509`).

This repair removes stack growth, not repeated work. The general entry
normalizes the entire candidate on each invocation
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`), so an
all-level unary traversal still performs a sum of successively larger subtree
scans: `Cases`/`Position` and no-match `ReplaceAll` remain quadratic in physical
depth even if each scan is iterative
(`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:223-232`;
`crates/rum-runtime/src/runtime/tree_walkers/position.rs:198-209`;
`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:176-190`).
Compiling the pattern once per traversal and removing `Cases`' first redundant
normalization would reduce repeated ordinary work, but neither change alone
removes the candidate-normalizer recursion
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:29-31`;
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:342-360`).
Normalizing a whole traversal once and carrying each normalized node beside the
original could also remove the quadratic rescan, but it would need explicit
original-to-normalized bookkeeping when `Flat` flattens and `Orderless`
reorders arguments, substantially widening semantics, memory, and suspension
risk (`crates/rum-runtime/src/runtime/resolution.rs:137-153`;
`crates/rum-runtime/src/runtime/tree_walkers/post_order.rs:4-23`).

No height cache or constructor scan is warranted for this repair. The rejected
brief-426 representation enlarged each private Arc allocation by eight bytes
and put unconditional height work before allocation
(`docs/dev/codex/brief-426-height-cache-spike.md:214-218,278-284`); its stabilized
`set-part-fixed100-len800` row regressed 30.04%, firing the construction
stop-loss (`docs/dev/codex/brief-426-height-cache-spike.md:352-425,565-575`). By
contrast, the current `SharedArgs::new` only wraps the supplied vector and
records the existing gated allocation counter
(`crates/rum-core/src/expr.rs:637-645`;
`crates/rum-core/src/profiling.rs:23-38,76-88`). The proposed work belongs solely in the
general match-normalization path, with the deep heap fallback cold
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`).

Brief-425 shows that an explicit-stack traversal can be neutral or better:
`position-wide-no-match` improved 23.55%, the directly exercised collectors
improved or stayed neutral, and its stop-loss did not fire
(`docs/dev/codex/brief-425-iterative-traversals.md:453-474,496-517`). That record
makes a bounded-normalizer spike reasonable, but not automatically affordable
on this hotter shared matcher entry (`AGENTS.md:117-122`). Since the observed
ceiling already exceeds the required 4,097 height, this is margin rather than a
current correctness failure; the honest recommendation is **try only the
bounded-front/iterative-fallback spike and keep it only if the full matcher-
neighbor A/B is neutral** (`docs/dev/codex/brief-427-per-node-match-recursion.md:64-73`).
Any coherent ordinary-path regression should stop that spike rather than move
work into `Expr`/`SharedArgs` construction
(`docs/dev/codex/brief-426-height-cache-spike.md:420-425,565-575`).

### 4. §3(e) classification

Brief-427's shorthand omits several entries present in the original §3(e), so
this table classifies the complete original tail rather than silently dropping
`predicates`, `apply_map`, core traits, fingerprints, or `Expr` Display/drop
(`docs/dev/codex/brief-422-xml-depth-and-the-class.md:267-280`;
`docs/dev/codex/brief-427-per-node-match-recursion.md:75-87`).

| §3(e) row | Classification against the 4,500 mechanism | Source basis |
|---|---|---|
| String-pattern machinery | **Other recursion.** String-pattern parsing recurses over the string-pattern expression AST, and the string matcher recurses over nested `StringPattern` nodes; the scan over input positions/repetitions is loop-based rather than a symbolic per-node all-level `match_expr_owned` walk (`crates/rum-stdlib/src/strings/patterns.rs:365-478`; `crates/rum-stdlib/src/strings/patterns/matcher/engine.rs:118-224,255-338,341-433`). Rule replacement additionally recurs once after each evaluated replacement; its explicit guard compares replacement count with configured `max_replacements`, not a fixed native-depth cap (`crates/rum-stdlib/src/strings/patterns/search/rules.rs:96-151,435-459`). |
| Calculus walkers | **Other recursion.** `differentiate_expr` recursively descends operands, and its exact-subexpression precheck is another direct tree recursion; neither is the general symbolic matcher (`crates/rum-stdlib/src/calculus.rs:88-112,114-156,198-225,253-267`). |
| Predicate walkers (present in the original §3(e)) | **Other recursion, with an isolated shallow match.** Array-shape and lexicographic comparison are direct recursive shape/comparison walks, and array-leaf collection is direct recursion (`crates/rum-stdlib/src/predicates.rs:102-129,144-162`; `crates/rum-stdlib/src/predicates/arrays.rs:247-255`). `ArrayQ` invokes `MatchQ` once on the integer rank, not once per input-tree node (`crates/rum-stdlib/src/predicates/arrays.rs:88-115,144-150`). |
| Polynomial walkers | **Other recursion.** Polynomial and rational-function conversion recurse directly through `Plus`/`Times`/`Power`, while pattern-language detection is its own expression walk (`crates/rum-stdlib/src/polynomials/core.rs:60-110`; `crates/rum-stdlib/src/polynomials/rational_function.rs:22-67`; `crates/rum-stdlib/src/polynomials/mod.rs:144-153`). Multivariate coefficient-grid construction also recurses over the variable slice rather than invoking the matcher (`crates/rum-stdlib/src/polynomials/core.rs:204-228`). |
| Schema conformance | **Other primary recursion; same normalizer is reachable only at a leaf.** Association/list schemas call `conform_value` recursively per nested schema/value pair (`crates/rum-stdlib/src/schema.rs:177-192,194-250,283-301`). A leaf evaluates one `MatchQ`, so a non-fast leaf pattern against a deep value can enter the same general normalizer, but this is not a per-node match inside the schema walk (`crates/rum-stdlib/src/schema.rs:303-331`; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:106-131`). |
| `apply_map` traversals | **Mixed: bounded already for `Map`/`MapAll`/`Apply`; other recursion remains for `MapIndexed` and `Scan`.** Apply and Map use a depth-16 recursive front path and explicit-stack fallback (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-61,70-106`). `MapIndexed` still calls itself for values, heads, and arguments, and `Scan` still collects recursively; neither path invokes the symbolic matcher (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-408,411-486`). |
| `Flatten[_, Infinity]` | **Other recursion.** `Infinity` selects the unbounded depth enum, and `flatten_matching_arg` recursively descends same-head arguments; level-group helpers contain separate direct recurrences (`crates/rum-stdlib/src/structure_ops/flatten.rs:166-180,224-272,274-290,344-397`). |
| Tensor/rank recursion | **Other recursion.** Tensor parsing, shape validation, scalar flattening, index enumeration, and rebuilding recurse over the tensor or dimension slice, with no per-node symbolic match (`crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:593-639,673-689,709-733`). The generated-node preflight itself recurses over the dimension list before its caller applies resource verdicts, so the source does not provide a native-stack bound on a long rank list (`crates/rum-stdlib/src/functional_ops/arrays/generators.rs:205-217,450-460`; `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:173-187,350-360`). |
| `Merge` nested | **Other recursion.** Sibling groups are processed in a loop, but a group whose values are associations re-enters `resume_merge_nested_from_index` through `evaluate_merge_nested_group`, leaving the parent group evaluation waiting per association depth (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-154,199-218`). Suspension converts state into a frame only on `InputRequired`; ordinary successful nested descent remains native (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:134-149`). |
| Kernel `PartialEq`/`Clone`/`Hash`/`Serialize` | **Mixed bounded/other recursion; not the match-walk mechanism.** Ordinary `Expr::clone` clones two Arc-backed wrappers and is depth-constant (`crates/rum-core/src/expr.rs:573-574,637-638,749-753`). `PartialEq` has pointer fast paths but recursively compares logical contents when those miss (`crates/rum-core/src/expr.rs:605-608,705-708,755-775`); derived `Hash`/serde traverse the nested logical values and have no local depth bound (`crates/rum-core/src/expr.rs:573-574,637-638,749-753`). |
| Structural/profiling fingerprints | **Other recursion, diagnostics-gated.** `hash_expr_text_and_exact` directly recurses through head and arguments (`crates/rum-runtime/src/runtime/profiling/fingerprints.rs:72-109`), while source-fingerprint collection is behind the cold-loaded profiling flag (`crates/rum-runtime/src/runtime/profiling/config.rs:65-82,174-188`). It never calls the symbolic matcher in that walk (`crates/rum-runtime/src/runtime/profiling/fingerprints.rs:86-109`). |
| Substitution walks | **Other recursion.** Binding substitution calls itself for heads/arguments, rule substitution does the same through its scoped dispatcher, and lexical substitution dispatches back through `substitute_arg_list_changed` (`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:47-77,125-160,280-329`; `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:287-363,386-427`). Slot substitution is another direct head/argument recurrence (`crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:3-34`). |
| Matcher compile and structural match | **Mixed: it contains the exact candidate-normalizer mechanism, plus other pattern-depth recursion; it is not itself an all-level traversal.** `match_expr_owned` normalizes the candidate before dispatch (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`). Matcher compilation recursively compiles nested pattern nodes (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:3-80,93-114`), and structural matching recursively invokes `match_compiled_owned` for nested matcher nodes/call heads (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:76-128,256-315,504-540`). The simple `Blank` dispatch is constant after normalization (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:510-515`). |
| Rewrite traversals | **Same mechanism for the measured collectors/`ReplaceAll`; mixed with other recursion elsewhere.** `Cases` and `ReplaceAll` feed each candidate through `normalize_match_expr` as established above (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:214-254,342-360`; `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:122-190`). `DeleteCases` both invokes `match_expr_owned` at selected nodes and retains a direct recursive child walk, while `Replace` combines `apply_rewrite_rule_plan_once` with direct recursive child descent (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:4-53,55-95,220-268`; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:270-319,321-365`). |
| `lowering.rs` | **Bounded already for parser-produced script source; other recursion remains for host-constructed ASTs.** Generic lowering recursively dispatches nested `SyntaxExpr` values (`crates/rum-runtime/src/runtime/lowering.rs:13-49,125-145`). Script parsing bounds delimiter and operator-chain depth at 128 and enforces both guards (`crates/rum-syntax/src/parser/front.rs:6-29,117-150`), while the 4,096-link left-associative case is collected/lowered iteratively (`crates/rum-syntax/src/lib.rs:17-20`; `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs:108-163`). The public `SyntaxExpr` constructors and public lowerer still permit an embedder to supply a deeper AST directly, so that host path has no local lowering bound (`crates/rum-syntax/src/ast.rs:5-13,28-30,103-110`; `crates/rum-runtime/src/runtime/lowering.rs:7-16`). |
| Suspension continuation encoding | **Other recursion; reachable boundary unclear — needs measurement.** Matcher continuation encode/decode recurse through `MatcherNode` and nested matcher arguments with no local depth counter (`crates/rum-runtime/src/runtime/pattern_matching/continuation.rs:50-120,123-219,222-301,303-343,384-412`). A representative compiled-except suspension finalizer performs that encoding and its resume frame decodes it, so activation depends on an actual suspended match rather than an all-level candidate walk (`crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:326-375`). |
| `Expr` Display + drop glue | **Other recursion.** Core `Display` recursively formats a call head and each argument (`crates/rum-core/src/expr.rs:850-873`). `Expr` has no manual iterative `Drop`; the deployment test explicitly distinguishes an iterative test cleanup helper from a naturally recursive deep-`Expr` drop (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:540-568`). The runtime's ordinary non-flat, non-`FullForm` text renderer now uses an explicit `RenderTask` vector, so that fixed `ToString` path does not classify core `Display` as bounded (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:25-66,69-84`). |

### 5. Not established from source, and what would settle it

1. Source identifies the recursive normalizer but cannot assign the observed
   4,500/4,700 split to an exact optimized frame size or compiler inlining
   decision (`crates/rum-runtime/src/runtime/resolution.rs:99-153`). A future
   release build with symbols, a 2 MiB thread, and a direct **non-fast-path**
   general match of a shallow pattern against a swept unary candidate—paired
   with a backtrace or disassembly at the boundary—would settle the numerical
   attribution; `MatchQ[deep, _]` is specifically unsuitable because it exits
   through the blank fast path
   (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:106-131`).
2. Source cannot establish whether a bounded-front/iterative-fallback
   normalizer is performance-neutral (`AGENTS.md:117-122`). Only a base/candidate
   `core-language-microbench.sh` A/B over the matcher and traversal neighbors
   named in §3, plus allocation counters and depth-65/deployment-stack
   certificates, can decide whether the margin is worth buying
   (`AGENTS.md:46-49`;
   `docs/dev/codex/brief-425-iterative-traversals.md:432-468,476-509`).
3. The §3(e) categories above are source-settled, but their individual 2 MiB
   failure thresholds were deliberately not measured in this brief
   (`docs/dev/codex/brief-427-per-node-match-recursion.md:89-99`). Focused future
   sweeps would be needed for the remaining direct-recursion families; matcher
   compile, structural match, and candidate normalization must be isolated with
   distinct pattern/value shapes so one does not mask another
   (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:3-80`;
   `crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32,256-315`).
4. Suspension continuation encoding has the most specific reachability unknown:
   settling it requires a runtime-built deep matcher that reaches an
   `InputRequired` path, followed by continuation encode and resume/decode on a
   2 MiB thread; the encode/decode recursion and representative suspension hook
   are at `crates/rum-runtime/src/runtime/pattern_matching/continuation.rs:50-219`
   and `crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:326-375`.
5. Parser-produced lowering is bounded, but the stack threshold for a public,
   host-constructed deep `SyntaxExpr` is unknown; a focused host-side 2 MiB
   lower-and-drop test would settle that residual API path
   (`crates/rum-syntax/src/ast.rs:5-13,103-110`;
   `crates/rum-runtime/src/runtime/lowering.rs:7-49`).
