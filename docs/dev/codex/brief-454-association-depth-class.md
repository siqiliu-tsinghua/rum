# brief-454 — the association-side depth class (design-only)

## Goal

Three association operations fall short of the ruled 4,097 physical height and
bracket suspiciously close together. Find out whether they share one cause,
explain why their per-level stack cost is roughly four times that of every
traversal this line has already converted, produce a complete inventory of the
class, and draft the implementation briefs that would close it.

**Scope: NONE — no source change.** The deliverable is a reviewable diagnosis
plus drafted follow-up briefs, every claim carrying `path:line` evidence.

## What I measured — treat as given, do not re-run

Release `rum-cli` at `a7474d94`, `ulimit -s 2048`, 1 MiB evaluator budget. The
fixture is `a = 1; Do[a = Association["k" -> a], {i, N}]`, and every probe
asserts the exact value it produces.

**Short of the contract, all failing in band:**

| probe | last ok / first fail | vs 4,097 |
|---|---|---|
| `Map[ff, a, {0, Infinity}]` over a deep association | **966 / 974** | **4.2x short** |
| nested part assignment `b[["k", …]] = 8` | **1,145 / 1,153** | 3.6x short |
| `MergeNested[{a, b}, Total]` | **1,153 / 1,161** | 3.6x short |

**Clearing the contract, and all four bracketing identically:**

| probe | last ok / first fail |
|---|---|
| construction alone | 5,455 / 5,463 |
| `Part @@ Prepend[Table["k", {N}], a]` (deep keyed read) | 5,455 / 5,463 |
| `ToString[a, InputForm]` | 5,455 / 5,463 |
| `Depth[a]` | 5,455 / 5,463 |

Do not re-measure any of this, and do not write a probe that drives a recursion
past its ceiling.

## The three questions this brief exists to answer

1. **One cause or three?** `MergeNested` (1,153) and nested part assignment
   (1,145) are eight levels apart, which looks like one shared recursion.
   `Map` over an association is 180 levels lower. Establish from source whether
   the three descents pass through a common function, and if so name it; if
   `Map`'s extra cost is additional frames on the same path, say which.
2. **Why is the per-level cost about four times everything else?** 1 MiB over
   1,150 levels is roughly 912 bytes per level. The stability walk costs 192 B
   after brief-444, and the converted traversals sit around 250 B. The four
   probes that clear the contract bracket at 5,455, which is 1 MiB / 5,455 ≈
   192 B — **exactly the stability walk**, i.e. those are bound by the shared
   walk and not by anything association-specific. So the short three pay roughly
   4.7x more per level than the walk that binds their own fixture. Find where
   that goes: frame size of the recursive function(s), what they hold live
   across the recursive call, whether an `Option`/`Result` return or a large
   local is resident. brief-443's method applies — `/proc` VmStk high-water at
   several heights gives a slope with no abort at all, and gdb ptrace is denied
   in your sandbox.
3. **Why do these fail closed at all?** Every file under
   `crates/rum-stdlib/src/associations/` has **zero** occurrences of
   `poll_native_stack_limit` and `evaluation_depth`, yet all three raise an
   in-band error rather than terminating the process. Establish which frame in
   the descent crosses the evaluator and therefore polls. That answer decides
   whether a repair must add a bound or only shrink a frame.

## Also deliver

4. **A complete class inventory.** Which association operations recurse per
   nesting level, with `path:line` for each, and a measured or reasoned ceiling
   for each. Candidates to check at minimum: `MergeNested`, `Merge`,
   `AssociationKeyDeflatten`, `DeleteMissing`, `ConformSchema`/`SchemaErrors`,
   `KeyMap`, `KeySort`, association `Map`/`Apply`/`Scan`, nested part
   assignment and the `AssociateTo`/`KeyDropFrom` mutation paths, and the
   association parser `associations/data.rs`. Say plainly which you measured and
   which you reasoned about.
5. **The construction ceiling's own status.** Construction gives out at 5,455
   and every deep-association probe inherits that limit, so no association
   operation can be tested above it. State whether that is the stability walk
   (the arithmetic says yes) or something association-specific, because it caps
   what any repair in this class can be certified to.
6. **A repair plan**, ordered, with the three-way taxonomy this line uses: a
   local poll is safety only; a bounded front plus explicit stack is safety plus
   the 4,097 contract; only an evaluator task stack lifts an active-result
   ceiling. Say which each site needs.
7. **Drafted implementation briefs** in `TEMPLATE.md` shape, one per step,
   including per-step gate tests, the certificate shape (unique deepest target,
   assertion on the exact rebuilt value, a shallow companion pinning the
   selector), and the falsifier each step would need.

## Context pointers

- `AGENTS.md` first, including the Testing Policy hard lines.
- `crates/rum-stdlib/src/associations/` — the whole directory; `mutation.rs`,
  `key_reshape.rs`, `missing.rs`, `aggregate.rs` and `data.rs` are the likely
  centres.
- The kernel side of nested part assignment:
  `crates/rum-runtime/src/runtime/definitions/part_assignment.rs` and
  `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`.
- What this line has already done, for the patterns to reuse:
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/traversal.rs`,
  `crates/rum-stdlib/src/calculus/differentiation/iterative.rs`,
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/iterative.rs`,
  `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs` (the
  `LexicalWalk` context struct that carries the depth).
- The measurement rules that now apply to any follow-up brief you draft:
  wall-clock timing is **not** evidence on this host; gates use allocation
  counts (`RUM_PROFILE_ALLOC=1` on a `--features alloc-count` build) and
  instruction counts (`perf stat -e instructions:u`, minimum of the tight clean
  cluster, runs that migrate between `cpu_atom` and `cpu_core` discarded).

## Scope

- Files you MAY modify: **NONE — diagnostic only.** Append your report to this
  file and change nothing else.
- Do not run a probe that drives any recursion past its measured ceiling; the
  numbers above are given so you do not have to.

## rum / Rust pitfalls (read twice)

1. **Every deep-association probe is capped by construction at 5,455.** A
   measurement that reports a ceiling near 5,455 is measuring the fixture, not
   the operation — the four clearing probes above all show exactly that. Bracket
   the construction first whenever you quote a number; that trap already hid
   `Merge`'s ceiling once (`TODO.md`, the 2026-09-03 probe table).
2. A probe must assert the exact value it claims to produce. Several probes in
   this line reported a ceiling for an operation that never ran.
3. `Depth` on an association omits heads and treats associations specially, so
   it certifies a different tree than physical height. Say which you mean.
4. The three short probes fail **in band**, so they are a contract shortfall,
   not a safety defect. Do not describe them as crashes.

## Report format (append to this file)

1. The mechanism — one cause or three, named in source with `path:line`, and
   where `Map`'s extra 180 levels come from.
2. The per-level cost — measured bytes per level for each descent, and what
   occupies the frame.
3. Why the descent polls, and which frame does it.
4. The class inventory, measured versus reasoned marked per row.
5. The construction ceiling's status.
6. The ordered repair plan with the three-way taxonomy.
7. The drafted implementation briefs.
8. Risks and open questions.

---

## Codex report — 2026-09-04

All source references below are to the current tree.  `HEAD` differs from the
measured `a7474d94` only by this brief, so the executable source is the source
that was measured.  No supplied ceiling was re-run.  Every additional stack
sample was below the supplied last-success height and exited normally.

### 1. The mechanism — one cause or three

#### First, the height unit needs correcting

The fixture counter `N` is **logical association nesting**, not physical
expression height.  `render_association` represents one logical entry as an
`Association` call containing a `Rule` call
(`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:125-133`).
Consequently the unique root-to-leaf path of the stated fixture has

```text
physical height = 2 * N + 1
```

when atoms and calls are each counted as one node.  `Depth` deliberately hides
the `Rule` layer and follows association values directly
(`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`); for example, two
wrappers have `Depth` 3 but raw physical height 5.  Therefore the ruled raw
physical height 4,097 is a fixture with **2,048 logical association wrappers**,
not 4,097 wrappers.  The supplied failure brackets translated to raw input
height are 1,933/1,949 for `Map`, 2,291/2,307 for assignment, and
2,307/2,323 for `MergeNested`.  They are still contract shortfalls, but the
reported 4.2x/3.6x factors compare unlike units; the physical shortfalls are
about 2.10x, 1.78x, and 1.76x respectively.

#### Finding

There is **not one common operation recursion**.  There are three paths, two of
which later converge on the same evaluator stability walk:

| Probe | Native descent that grows | What actually binds the supplied exact-value probe |
|---|---|---|
| association `Map` | None in the transform after level 16: `Map` first tries a depth-16 front and then uses `TransformWork::{Visit,FinishCall,FinishAssociation}` (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61,70-106,123-141,205-267`). `eval_map` simply returns that result (`crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:77-92`). | The returned tree is fresh at every selected level, so the next fixed-point pass calls `cached_expr_is_evaluation_stable` (`crates/rum-runtime/src/runtime/resolution.rs:305-326`). Its cache-miss recursion is `cached_expr_stability_with_bloom` -> `call_stability_with_bloom` -> `call_stability_uncached_with_bloom` (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:200-264`; `crates/rum-runtime/src/runtime/eval_core/stability.rs:5-34). |
| nested association part assignment | `set_association_key_path` calls itself once per key and rebuilds while unwinding (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:934-940,976-1004`). | The mutation itself returns the scalar right-hand side (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:910-920`). The exact-value certificate's subsequent read of the freshly rebuilt symbol enters the same fixed-point stability walk. The operation-only slope below confirms that the assignment recursion is not the 1,145/1,153 binding. |
| `MergeNested` | `resume_merge_nested_from_index` calls `evaluate_merge_nested_group`, which calls the former again for nested associations (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-153,199-218`). The parent owns its grouped and partial-output vectors across that call. | This native mutual recursion is already about 1,008 B/logical level. At the leaf, `evaluate_merge_nested_group` crosses back into `Session::evaluate` for the combiner (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:204-217`), so the evaluator can observe the retained native frames. The fresh merged output would subsequently encounter the 896 B association stability path as well. |

`Map`'s extra roughly 180-level loss is now exact in source.  The iterative
association finish rebuilds an association and then wraps every selected level
in `operator[rebuilt]` (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:205-245`).
For `{0, Infinity}`, a logical layer in the result is therefore
`ff[Association[Rule[key, child]]]`.  In addition to the `Association` and
`Rule` cache-miss packets, it adds one ordinary `ff[...]` cache-miss packet per
logical level.  The output path has physical height `3*N + 2`, including the
mapped leaf.  The added generic packet is 192 B, turning the 896 B fresh
association packet into a predicted 1,088 B `Map` packet.  That prediction
matches the measured slope below.

### 2. Per-level cost and resident frames

#### Safe `/proc` measurements

These are `VmStk` high-water samples on the supplied release binary and 2 MiB
thread.  All heights are logical association wrappers and are below the given
last-success point for that operation.

| Path | `(N, VmStk)` samples | Slope |
|---|---|---:|
| `Map[ff, a, {0, Infinity}]` | `(128,168 KiB)`, `(256,300 KiB)`, `(512,572 KiB)`, `(768,836 KiB)`, `(900,980 KiB)` | **1,075.5 B/logical level** OLS; 1,077.1 B by endpoints |
| `MergeNested[{a,b}, Total]` | `(128,172 KiB)`, `(256,304 KiB)`, `(512,548 KiB)`, `(768,804 KiB)`, `(1000,1032 KiB)` | **1,007.2 B/logical level** OLS; 1,009.9 B by endpoints |
| assignment followed by exact rebuilt-target read | `(128,148 KiB)`, `(256,264 KiB)`, `(512,484 KiB)`, `(768,712 KiB)`, `(1000,920 KiB)` | **904.0 B/logical level** OLS; 906.6 B by endpoints |
| assignment expression returning only `8` | `(128,136 KiB)`, `(256,136 KiB)`, `(512,228 KiB)`, `(768,324 KiB)`, `(1000,400 KiB)` | page-floor distorted globally; **360.9 B/logical level** on the 512-to-1000 segment |

The release prologues explain all four slopes.  Counts below include saved
registers, `sub $N,%rsp`, and the 8-byte recursive return address.  The generic
stability packet was independently established as a 184 B prologue and
192 B/level (`docs/dev/codex/brief-446-stability-poll-placement.md:115-120`).

| Resident packet | Static bytes | Source-level live state |
|---|---:|---|
| generic cached call stability | **192** | The 48-byte ABI result area for `(bool, StabilityBloom)`, 16-byte pointer cache key, bloom, miss outcome/profile token, and live session/head/args pointers. The irreducible result/key pieces are documented at `docs/dev/codex/brief-443-stability-failclosed-measurement.md:370-389`; the current return and key construction are at `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:204-226`. |
| association data helper | **400** = six pushes + `0x158` + return | A linear `Vec<Expr>`, optional `HashSet<Expr>`, parsed `AssociationEntryForm`, iterator position, and accumulated bloom remain live while the rule child is checked (`crates/rum-runtime/src/runtime/eval_core/stability.rs:97-134`). |
| rule data helper | **112** = six pushes + `0x38` + return | LHS result/bloom and parent bloom remain live while the RHS is checked (`crates/rum-runtime/src/runtime/eval_core/stability.rs:137-153`). |
| `MergeNested` driver | **656** = six pushes + `0x258` + return | Owned combiner, grouped `Vec`, partial merged `Vec`, index, key/values and the `Result`/suspension match state (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-153`). |
| `MergeNested` group helper | **352** = six pushes + `0x128` + return | Owned combiner and values, parsed nested-association vectors, grouped child state, and the `Result<Expr, EvalError>` across the recursive driver call (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:199-218`). |
| association key-path set | **368** = six pushes + `0x138` + return | Owned current expression, parsed entry vector, selector tail, RHS, and `AssociationSetUpdate`; the parent entries remain live until the child is rebuilt (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:976-1003`). |

Thus the resident logical packets are:

```text
fresh association = generic Association 192
                  + association helper   400
                  + generic Rule         192
                  + rule helper          112
                  =                       896 B/level

Map result        = fresh association     896
                  + generic ff call       192
                  =                     1,088 B/level

MergeNested       = driver                656
                  + group helper          352
                  =                     1,008 B/level

assignment alone = set-key-path packet    368 B/level
```

The observed 1,075.5, 1,007.2, 904.0, and 360.9 B slopes agree within page
quantization, fixed entry overhead, and optimized-call alignment.  The “about
four times 192 B” mystery is therefore two generic call packets **plus** two
data-constructor helpers, not one unexpectedly large stability frame.  The
`MergeNested` similarity is coincidental: it is a different two-frame packet.

### 3. Why the descents fail in band, and which frame polls

There are two polling routes, and the premise that every operation-local
descent crosses the evaluator is false.

1. Every call-stability **miss** increments a counter and, every 16 misses,
   calls `native_stack_budget_exhausted`; exhaustion conservatively returns an
   unstable result (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-264`).
   The next unstable evaluation enters the ordinary evaluator, whose
   depth-strided check returns `StackLimitExceeded`
   (`crates/rum-runtime/src/runtime/eval_core/mod.rs:14-43,66-77`).
2. Independently, any operation that calls `Session::evaluate` while native
   parents remain live gives that nested evaluation a chance to take the same
   stability check and then the evaluator byte poll.  A nested evaluate does
   not reset the native-stack base unless `evaluation_depth == 0`
   (`crates/rum-runtime/src/runtime/resolution.rs:288-320`).

For `Map`, the transform neither polls nor re-enters the evaluator.  It returns
to `evaluate_profiled_fixed_point`; the **post-result stability miss frame** is
the first frame that notices the budget.  For assignment, the key-path rebuild
also neither polls nor re-enters; the assignment returns `8`, and the
certificate's later **symbol-result stability miss frame** notices the freshly
rebuilt target.  For `MergeNested`, the retained driver/helper pair reaches
`session.evaluate` at its leaf, so the **combiner evaluation's stability/eval
frame** can poll while all native merge parents are live
(`crates/rum-stdlib/src/associations/resumable/aggregate.rs:199-217`).  Its
fresh result has the post-result route too.

This is why all three supplied short outcomes are in-band without a single
`poll_native_stack_limit` in `associations/`.  It does **not** make their native
walks bounded: changing the combiner/certificate shape could remove the lucky
evaluator crossing.  They are contract shortfalls, not crashes and not proof of
operation-local safety.

### 4. Complete class inventory

“Measured” below means this report's safe slope or a supplied bracket.
“Reasoned” means release prologue arithmetic plus the cited call graph; it is
not presented as a measured ceiling.  Approximate ceilings divide the relevant
1 MiB evaluator budget or 2 MiB physical stack by the resident packet and omit
fixed entry overhead, so they are deliberately upper estimates.

#### Per logical association nesting or deep association result

| Surface/path | Status and source | Current ceiling / verdict |
|---|---|---|
| `Map` | Depth-16 front plus explicit transform stack already (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61,70-267`); fresh association and `ff` at every level. | **Measured:** 966/974 logical, slope 1,075.5 B; post-result stability binds. Short of raw height 4,097. |
| `Apply` | Shares the same bounded front/explicit stack (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:8-29,94-203`). Applying a head need not preserve a deep association; unchanged subtrees retain pointer-cache hits. | **Reasoned:** no association-local native ceiling. A result with one ordinary call per logical layer pays the established 192 B generic walk and clears 4,097 raw calls; a preserved fresh association is governed by the shared stability row. |
| `MapIndexed` | Direct recursion through association values and ordinary calls (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-409`); release packet **560 B/logical layer**. It also creates a fresh selected result. | **Reasoned:** current fresh-result envelope is no better than `Map`'s roughly 970 logical levels; local physical-stack projection is about 3,700 logical levels. Short now through stability; unbounded locally. |
| `Scan` | Recursive collector over association values and calls (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:411-487`), then callbacks are evaluated sequentially and the operation returns `Null` (`crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131`). Release packet **336 B/logical layer**. | **Reasoned:** raw-height-4,097 input (`N=2048`) costs about 688 KiB and should clear; nominal 2 MiB ceiling is about 6,200 logical levels, so fixture teardown likely binds first. Still unbounded and unpolled while collecting. |
| nested association `Set` | Recursive reverse rebuild (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:934-1004`). | **Measured:** operation-only 360.9 B/logical versus 368 B static; exact-target read 904.0 B and supplied 1,145/1,153. Step 1 lifts the certificate; the local walk itself should clear `N=2048` but is unbounded. |
| `AssociateTo` / `KeyDropFrom` on a `Part` target | Exact selectors are planned, read, then reverse rebuilt at `crates/rum-stdlib/src/associations/mutation.rs:560-692`. A key selector expands to the association argument and rule RHS components (`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:112-126`); exact collection/read/rebuild recurse at `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42,88-109,173-200`. Release packets are 336 B/selector, 96 B/raw component, and 256 B/raw component respectively. | **Reasoned:** the rebuilt target first hits the ~1,170-logical fresh-association envelope. After that is fixed, raw height 4,097 is locally near 1 MiB for replacement but under the 2 MiB thread; the path remains unbounded. Direct symbol-only mutation changes only the root (`crates/rum-stdlib/src/associations/mutation.rs:268-278,401-429`) and is excluded. |
| keyed `Part` read | Iterates keys (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:523-541`). | **Measured supplied:** 5,455/5,463 logical, identical to construction. No operation recursion. |
| `MergeNested` | Native mutual recursion at `crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-218`; 1,008 B/logical measured/static. | **Measured:** 1,153/1,161 supplied; 1,007.2 B/logical. At `N=2048` it would need about 2.06 MiB before fixed overhead, so it needs a local explicit stack even after stability is fixed. |
| ordinary `Merge` | Parses inputs, groups once, then processes sibling groups in loops (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:8-53,224-258`). It does not descend association-valued entries. | **Reasoned:** canonical nested associations reuse child pointers and are construction-bound. Deeply nested *list collection syntax* reaches the parser row below, a different depth axis. |
| `AssociationKeyDeflatten` | Recurses down a key path in `insert_path`, then recursively renders its `NestedPathNodes` (`crates/rum-stdlib/src/associations/key_reshape.rs:340-415`). Release packets are 224 B/key and 256 B/rendered branch, in separate phases. | **Reasoned:** local work for a 2,048-component path is at most about 512 KiB, but the fresh depth-2,048 association result hits the ~1,170-logical stability envelope. Short now through stability; locally unbounded. |
| `AssociationKeyFlatten` | Recurses down association values (`crates/rum-stdlib/src/associations/key_reshape.rs:417-440`); release packet **496 B/logical layer** and result is flat. | **Reasoned:** `N=2048` is about 992 KiB of native frames and should fit the 2 MiB thread, so it clears raw 4,097 but has little evaluator-budget margin and no poll. Nominal physical-stack ceiling is about 4,200 logical layers, below the construction fixture's logical cap. |
| `DeleteMissing` | Memory preflights recurse at `crates/rum-stdlib/src/associations/missing.rs:84-230`; recursive mode rebuilds at `crates/rum-stdlib/src/associations/missing.rs:936-1003`; finite-target mode is mutually recursive/resumable at `crates/rum-stdlib/src/associations/missing.rs:355-934`; scan uses `contains_missing_within` (`crates/rum-stdlib/src/associations/missing.rs:1038-1053`). Release packets include 160 B for recursive mode and **816 B** for the finite-target association driver. | **Reasoned:** recursive mode's deep fresh output hits ~1,170 logical levels; finite-target mode reaches roughly 1,285 logical levels per MiB before fixed overhead and can evaluate a delayed entry while parents are live (`crates/rum-stdlib/src/associations/missing.rs:590-620,798-817`). Both are short today; the armed-memory preflight is an additional unbounded route. |
| `ConformSchema` / `SchemaErrors` | Both call the same `SchemaValidator::conform_value` (`crates/rum-stdlib/src/schema.rs:55-157`). Association/list conformance recurses at `crates/rum-stdlib/src/schema.rs:177-301`; a leaf evaluates `MatchQ` at `crates/rum-stdlib/src/schema.rs:303-330`. Release packet **1,056 B/nested value/schema pair**. | **Reasoned:** about 993 logical pairs per MiB, raw height about 1,987, before fixed overhead. `ConformSchema` also returns a fresh deep association; `SchemaErrors` discards it but still pays the validator recursion. Both are short. |
| multi-spec `GroupBy` | Spec lists select nested grouping (`crates/rum-stdlib/src/associations/grouping.rs:135-206`). Each grouped child calls the group driver again for the next spec (`crates/rum-stdlib/src/associations/grouping/list.rs:257-347`; association variant `crates/rum-stdlib/src/associations/grouping/association.rs:278-364`). Release parent/child packets are roughly 1.9-2.0 KiB/spec; leaf reducers cross the evaluator (`crates/rum-stdlib/src/associations/grouping/list.rs:239-255`). | **Reasoned:** roughly 500 nested specs per MiB, producing raw association height around 1,000. This newly found association-result producer is below contract and needs its own task stack. |
| `KeyMap`, `KeySort` | Only top-level entries/keys are mapped or sorted; values are reused (`crates/rum-stdlib/src/associations/transforms.rs:10-46,86-125`). Custom ordering is an index loop, not a value-depth walk. | **Reasoned:** no per-association-nesting recursion. With shallow keys/functions, deep values keep cache identities; construction/teardown binds. |
| `AssociationMap`, `KeyGroupBy`, `Counts`, `Catenate`, key-set operations, `JoinAcross` | Their data loops advance indices and store resumable state rather than descending association-valued children (`crates/rum-stdlib/src/associations/resumable/map.rs:52-199`; `crates/rum-stdlib/src/associations/key_reshape.rs:62-103,145-325`; `crates/rum-stdlib/src/associations/aggregate.rs:162-493`; `crates/rum-stdlib/src/associations/join_across.rs:213-337`). | **Reasoned:** no recursion per canonical association-value level. A callback may independently return a deep fresh value, which is an active-result stability case, not an operation walk. |

#### Recursive association-owned input shapes that are not the canonical fixture

These matter for safety completeness, but their recurrence follows nested
collection/spec syntax rather than a `Rule` value.  The canonical fixture's
nested association is behind a rule, so these parsers stop at the rule and do
not add a resident frame to the slopes above.

| Site | Shape and source | Reasoned ceiling |
|---|---|---|
| stdlib and runtime association deep parsers | Mutual `Association`/`List` item flattening at `crates/rum-stdlib/src/associations/data.rs:234-260` and `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:95-123,279-305`; **272 B/nested container** in both release copies. | About 7,700 containers on 2 MiB nominally; raw 4,097 should clear, but there is no bound/poll. |
| ordinary `Merge` collection parser | Nested list collection recursion at `crates/rum-stdlib/src/associations/data.rs:163-189`; **128 B/list layer**. | About 16,000 layers nominally; clears 4,097 by arithmetic, unbounded. |
| `Keys` / `Values` over nested lists | Recursive projection at `crates/rum-stdlib/src/associations/basic.rs:752-810`; **336 B/list layer**. | About 6,200 layers nominally; likely clears 4,097, unbounded. |
| `KeyTake` / `KeyDrop` over nested lists | Recursive projection at `crates/rum-stdlib/src/associations/basic.rs:818-915`; **720 B/list layer**. | About 2,900 layers nominally: below raw 4,097 for this *list-shape* contract, but not association-value nesting. |
| nested `Lookup` key specifications | Recursive count and result planning at `crates/rum-stdlib/src/associations/basic.rs:699-721`. | Unmeasured/reasoned only; output depth follows the nested key-list shape and then reaches ordinary result stability. |
| nested option lists for key collections | `collect_option_names_from_expr` at `crates/rum-stdlib/src/associations/transforms/key_collections.rs:488-517`; **96 B/list layer**. | About 21,800 nominal layers; clears 4,097, unbounded. |
| `Normal` below non-association wrappers | `normalize_association_expr` recursively rebuilds ordinary calls but stops and converts as soon as it reaches an association (`crates/rum-stdlib/src/associations/basic.rs:549-560`); **112 B/wrapper**. | About 18,700 wrappers nominally; clears 4,097, unbounded. A root association does not recurse into its association-valued entries. |

No other source recursion in `crates/rum-stdlib/src/associations/` follows
association value depth.  The many `resume_*` calls that advance `index + 1`
are loops or tail transitions over siblings; generated suspension-frame
`resume` methods restore those drivers but do not recurse by tree height.

### 5. Construction ceiling status

The common 5,455/5,463 bracket is **not the stability walk during
construction**, despite the apparent `1 MiB / 5,455 ~= 192 B` arithmetic.  A
safe, sub-ceiling live-process sample separated the operation from teardown:

| live operation at `N=4000` | `VmStk` high-water |
|---|---:|
| build complete, paused with `a` still live | **136 KiB** |
| read symbol, then pause | **136 KiB** |
| keyed `Part`, then pause | **724 KiB** |
| `Depth`, then pause | **412 KiB** |
| `ToString[..., InputForm]`, then pause | **396 KiB** |

All five shapes also returned normally at `N=5400`, below the supplied 5,455
last-success point.  Construction therefore has no depth-growing native stack
while the loop is building the value.  Keyed lookup is explicitly iterative
(`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:523-541`), InputForm rendering uses a
`RenderTask` vector (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:25-66,69-100`), and
`Depth` uses its separate association-value recursion
(`crates/rum-stdlib/src/structure_ops/mod.rs:115-140`).  Their different live peaks cannot produce
one identical operational bracket.

The common point is teardown.  `Expr` owns `Arc`-backed `SharedExpr` and
`SharedArgs`, derives its ordinary drop glue, and has no manual `Drop`
(`crates/rum-core/src/expr.rs:573-775`).  The runtime tests explicitly distinguish
an iterative cleanup helper from a naturally recursive deep-`Expr` drop
(`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:540-568`).  When
the last root goes away, dropping the unique association chain recursively
drops `Association`, then `Rule`, then the next `Association`.

This also resolves the misleading arithmetic: `N=5455` contains **10,910 call
nodes** on its deep path.  `2 MiB / 10,910` is again about 192 B/raw call.  The
factor of two in physical nodes cancels the factor of two between the 2 MiB
thread and 1 MiB evaluator budget, making teardown look numerically identical
to a 192 B/logical-level stability walk.

The exact optimized drop packet was not directly sampled because ptrace is
unavailable, so “natural recursive drop” is the strongly supported attribution,
not a claimed exact disassembly proof.  It does **not** cap certification of the
ruled target: raw height 4,097 needs `N=2048`, and even the alternate (but
incorrectly named) target of 4,097 logical wrappers is below 5,455.  Fixing drop
would raise the usable ceiling beyond the contract, but it is a separate core
ownership/destruction lane and not a prerequisite for this repair sequence.

### 6. Ordered repair plan and taxonomy

The ordering follows the first binding on a correct exact-result certificate.

1. **Evaluator-owned association-result stability task stack.** Trigger a cold
   explicit machine only after a bounded association-specific recursive front;
   do not put the rejected depth/`Option` shape back on every generic stability
   query.  This is the only category that lifts an **active-result ceiling**.
   It unlocks `Map`, exact reads after nested assignment, recursive
   `DeleteMissing`, `AssociationKeyDeflatten`, and every other producer of a
   fresh deep association.  The two earlier generic bounded/iterative attempts
   were functionally correct but regressed hot labels 2-7%
   (`docs/dev/codex/brief-442-stability-frame-shrink.md:26-32`), so association-
   gated entry and the full instruction/allocation gate are mandatory.
2. **`MergeNested` bounded front plus explicit DFS/suspension state.** Existing
   evaluator crossings make current failure safe, but a local poll would only
   preserve that safety and fail near the same 1 MiB point.  An explicit stack
   gives safety **and** the 4,097 contract.
3. **Schema validator explicit task stack.** Same classification: its leaf
   evaluator call happens with 1,056 B native parents.  A poll is safety only;
   bounded front plus explicit tasks is safety plus contract.
4. **`DeleteMissing` explicit traversal and iterative preflights.** Convert the
   finite-target/resume path, recursive path, missing scan, delayed-wrapper
   count, and armed-memory proxy coherently.  A poll in any one path is safety
   only and leaves alternate modes unbounded; explicit visit/finish tasks close
   both safety and contract.
5. **Nested multi-spec `GroupBy` task stack.** Encode spec-depth parents and
   suspension continuations on the heap.  Again, local polling is safety only;
   a bounded front plus explicit state reaches the contract.
6. **Association key-path assignment.** Convert
   `set_association_key_path` to an explicit descent/reverse rebuild.  Step 1 is
   enough for the measured assignment certificate at raw 4,097, but this local
   walk remains stack-size-dependent.  An explicit stack provides the safety/
   contract result; adding a poll would only fail in band.
7. **Held-mutation exact paths.** Separately convert shared exact-path collect,
   read, and replace used by `AssociateTo`/`KeyDropFrom` Part targets.  These
   helpers have different selector and overlap invariants from association
   assignment and need an independent semantic/performance gate.  Explicit
   stacks give safety plus contract; polls give safety only.
8. **Key reshape.** Make deflatten insertion/rendering and flatten traversal
   iterative with ordered finish tasks.  Step 1 removes the current result
   ceiling; this step removes the remaining local safety ceiling.
9. **Residual functional association walkers.** Convert `MapIndexed` and
   `Scan` using the existing `Map`/`Apply` visit/finish machine.  A local poll is
   safety only; the existing bounded-front pattern is safety plus contract.
10. **Association-owned collection parsers.** Deduplicate or independently
   convert the stdlib/runtime deep parsers and the `Merge` collection parser.
   They are reasoned to clear raw 4,097 today, so this is last: it closes an
   unpolled safety tail, not a currently demonstrated association-value
   shortfall.  The nested-list `Keys`/`Values`/`KeyTake`/`KeyDrop`/`Lookup`
   family should be briefed in its generic collection-shape lane rather than
   silently bundled here.

A **local poll is safety only** in every operation above: it changes an
out-of-band edge into a resource outcome but cannot make work finish at 4,097.
A **bounded recursive front plus explicit work stack is safety plus the 4,097
contract** for an operation-owned walk.  **Only an evaluator task stack** (step
1) lifts the ceiling of an already-built result that remains active throughout
fixed-point evaluation.  Shrinking native frames can add margin but does not
change any of those categories.

### 7. Draft implementation briefs

These are raw drafts in `TEMPLATE.md` shape.  Each deep certificate is run only
after its conversion is present; no red phase drives a known or unknown native
recursion beyond its ceiling.  Every fixture has one distinguishable deepest
leaf, every gate validates the entire rebuilt value (using InputForm text or an
iterative Rust structural checker), and every selector/descent rule has a
three-level companion.

#### Draft A — association-rooted evaluator stability tasks

##### Goal

Make stability checking of a fresh association-rooted result
native-stack-bounded without changing stability verdicts, bloom union order,
cache keys/insertion order, profiling counters, or the generic hit/miss hot
shape.  `Map`, nested association assignment followed by a read, and
`AssociationKeyDeflatten` must complete on inputs/results whose asserted raw
physical height is 4,097.

##### Context pointers

Read `AGENTS.md`, briefs 440-446, and
`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:200-275`,
`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-59,97-153`,
`crates/rum-runtime/src/runtime/resolution.rs:305-326`, and
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:205-259`.

##### Scope

May modify the two runtime stability modules, their module roots if
required, runtime cache tests, `rum-stdlib` depth-certificate tests, and
`scripts/core-language-microbench.sh` only for named labels.  Do not touch
`Expr` representation, stdlib operation implementations, documented semantics,
serde, or `Cargo.lock`.

##### Deliverable

Keep the current straight-line generic cached lookup.  Give
association stability a depth-16 borrowed front; only its exhausted tail may
allocate an evaluator-owned `Visit`/`FinishCall` machine.  The machine must
preserve head-before-args, first-unstable short circuit, bloom order,
stable-only insertion, and exact postorder cache insertion/size-cap behavior
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33`;
`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-264`).  Use
the previously rejected begin/finish state only inside the cold association
tail, never on generic hits or ordinary misses.  Add cache-trace equivalence
tests covering duplicate keys, `RuleDelayed`, dirty generations, unstable
early children, and cache-cap insertion order.

Once entered at an association root, the bounded front and task machine must
own the **entire rooted subtree** and carry their depth through ordinary calls
such as `ff[...]`; restarting a fresh depth counter at each `Association` would
miss exactly the interleaved `ff[Association[Rule[...]]]` Map path diagnosed
above.

Certificates: construct `N=2048` and assert with an iterative helper that the
input has raw height 4,097.  Put a unique sentinel only at the deepest value.
For `Map[ff, a, {0, Infinity}]`, assert the exact full InputForm string, including
every `ff`, association delimiter, key, and final sentinel; for assignment, read `b`
and assert every ancestor plus the final `8`; for deflatten, assert the entire
association produced from a 2,048-component key.  Three-level companions pin
`{0,Infinity}`, the full key selector chain, and key-list order respectively.

##### rum / Rust pitfalls (read twice)

Do not repeat briefs 440/441's generic depth argument
or `Option` return.  Heap tasks must use Arc-cheap clones and preserve early
termination; an eager postorder traversal changes cache contents.  The Map
result is taller than its input (`3*N+2`), so clean it up without asking core
`Display` or equality to recurse (`crates/rum-core/src/expr.rs:755-775,850-873`).

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run bounded `cargo fmt --check`, `cargo test -p rum-runtime`,
`cargo test -p rum-stdlib`, `cargo test --workspace`, then the release ignored
certificate target.  Before performance work, require byte-identical stability
counters and cache-insertion traces on the equivalence corpus.  A/B default and
candidate allocation counts with `RUM_PROFILE_ALLOC=1` on an `alloc-count`
build, then `perf stat -e instructions:u` on `association-small-literal-repeat`,
`list-literal-small`, both downvalue-miss-repeat labels, evaluator/rewrite
neighbors, and a new shallow association-stability label.  Use the minimum of a
tight clean cluster and discard `cpu_atom`/`cpu_core` migrations; wall time is
not evidence.

##### Falsifier

Under `cfg(test)`, force the front budget to one and assert a
tail-entry trace.  Temporarily disable the tail transition and show that test
red, then restore it and show green.  Separately point each shallow companion at
a non-descending selector/wrong key order and paste its exact-value failure.
Never run the deep certificates before the implementation exists.

##### Hard rules

No network, third-party oracle, `unsafe`, git mutation, or files outside Scope.
Stop after one revision if counters/cache order differ or any instruction/alloc
label regresses beyond same-binary spread.

##### Report format

Report the diff, red/green falsifier, exact certificate outcomes, verbatim test
output, counter/cache traces, allocation rows, instruction clusters, and open
issues.

#### Draft B — `MergeNested` explicit nesting stack

##### Goal

Remove native recursion by association nesting from `MergeNested`
while preserving grouping order, combiner call order, diagnostics, and
suspend/resume behavior; exact merge at raw input height 4,097 must complete.

##### Context pointers

`crates/rum-stdlib/src/associations/resumable/aggregate.rs:55-85,122-218`, its
`crates/rum-stdlib/src/associations/resumable/aggregate/resume_frames.rs`,
grouping state in `crates/rum-stdlib/src/associations/grouping.rs:10-108`, and the
post-result repair from Draft A.

##### Scope

The resumable aggregate implementation/frame module, focused
`MergeNested` tests, the shared depth-certificate module, and a microbench label
only.  Ordinary `Merge`, grouping semantics, runtime evaluator code, docs, and
`Cargo.lock` are out.

##### Deliverable

Retain a small borrowed/native front if it is instruction-
neutral, then represent deeper parents as explicit `EnterGroup`/`FinishGroup`
tasks carrying combiner, grouped entries, index, key, and partial output.  Any
`InputRequired` suspension must freeze the whole explicit stack and resume at
the exact pending combiner without replay.  Preserve first-seen key order and
the rule that only an all-association value group recurses
(`crates/rum-stdlib/src/associations/resumable/aggregate.rs:199-222`).

Certificate: merge two `N=2048` chains with a unique deepest numeric pair and
nonmatching shallow sentinels; iteratively assert each input raw height 4,097
and assert the exact complete merged association ending in the summed deepest
value.  A three-level companion pins the nested-association branch predicate,
key order, and that `Total` is called only at the leaf.  Add a suspended
combiner companion proving no call is repeated.

##### rum / Rust pitfalls (read twice)

A local poll is not acceptance.  Do not turn suspension into eager evaluation
or deep-clone grouped trees.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt,
`cargo test -p rum-stdlib`, workspace tests, and the release certificate after
the change.  Gate alloc counts and `instructions:u` for shallow/wide ordinary
`Merge`, shallow/wide `MergeNested`, and same-tier grouping neighbors.

##### Falsifier

On the shallow companion, temporarily classify a mixed scalar/
association group as nested (or reverse finish order) and capture the exact
failure; then restore.  A test-only forced-front depth of one must record use of
the heap path.  No pre-change deep run.

##### Hard rules

Standard template hard rules apply: no files outside Scope and no unsafe,
network, third-party oracle, or git action.  Stop for replayed callbacks,
changed ordering, or a coherent alloc/instruction regression.

##### Report format

Report frame removal, continuation shape, red/green falsifier, exact values,
tests, and measured gates.

#### Draft C — iterative schema validator

##### Goal

Make `ConformSchema` and `SchemaErrors` conform nested association/list
value-schema pairs with bounded native stack through raw height 4,097, preserving
error order/cap, defaults, extra-key policy, leaf evaluation, and suspension.

##### Context pointers

`crates/rum-stdlib/src/schema.rs:12-53,55-166,177-339,431-520` and its existing
suspension frames/tests; Draft A for fresh conformed-result stability.

##### Scope

`crates/rum-stdlib/src/schema.rs`, a responsibility-based new
`crates/rum-stdlib/src/schema/` traversal/frame module if needed to keep files
below organization thresholds, schema tests, depth certificates, and one
benchmark label.  Catalog/reference semantics and matcher/evaluator internals
are out.

##### Deliverable

Replace recursive `conform_value` with bounded visit/finish
tasks.  A finish task owns matched bits, conformed slots, defaults, path, and
source order; leaf `MatchQ` and default evaluation must suspend with the entire
task stack and resume once.  `SchemaErrors` must perform identical validation
work but return the exact ordered capped errors.

Certificate: value and schema each have one unique deepest key and are sized so
an iterative raw-tree counter—not `Depth`—asserts physical height 4,097; account
for any `Required`, pattern, or default wrapper at the leaf rather than assuming
2,048 association wrappers.  A leaf conversion/default makes the successful
rebuilt result distinguishable.  Assert the exact whole conformed
association; a second exact certificate asserts `SchemaErrors`' full empty or
single-error list and path.  Three-level companions pin key selection, list
index paths, default evaluation, and extra-key behavior.

##### rum / Rust pitfalls (read twice)

Preserve the 64-error cap and message order; do not evaluate unmatched defaults
or leaf matchers early.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt, stdlib,
workspace, focused suspension tests, and release depth certificates.  Gate
alloc counts and `instructions:u` for shallow association/list schemas,
defaulted schemas, error paths, and matcher neighbors.

##### Falsifier

Temporarily schedule association finishes in reverse schema
order; the shallow exact error-order/result companion must fail.  Force heap
entry at depth one and require its trace.  Do not run the deep case pre-fix.

##### Hard rules

Standard template hard rules apply; stop on callback replay, error-order drift,
or measured regression.

##### Report format

Report exact red/green output, continuation invariants, test output,
allocation/instruction rows, and risks.

#### Draft D — complete `DeleteMissing` traversal stack

##### Goal

Bound every `DeleteMissing` depth-dependent path—preflight, recursive
mode, finite target mode, scan, and delayed-rule wrapper planning—while
preserving exact level/scan semantics and suspension.

##### Context pointers

`crates/rum-stdlib/src/associations/missing.rs:62-230,307-328,330-934,936-1053`,
`crates/rum-stdlib/src/associations/missing/resume_frames.rs`, memory-limit
tests, and Draft A.

##### Scope

Split `missing.rs` by traversal responsibility if needed, its resume
frames/module root, DeleteMissing tests/depth certificates, and a benchmark
label.  Resource-limit policy, `Missing` semantics, and other association
operations are out.

##### Deliverable

Use one explicit ordered walk model for value traversal and
separate allocation-free/heap-bounded counters for the armed-memory preflight.
Encode list versus association finishes, current logical level, scan depth,
delayed flag, source index, and kept entries.  Preserve delayed-entry evaluation
and `InputRequired` without replay.  Remove every source recursion listed in
the inventory, including `contains_missing_within`.

Certificate: an association sized by an iterative raw-tree counter to physical
height 4,097, with exactly one deepest `Missing`, a surviving sibling at every
level, and a delayed-rule control.  Account for the non-atom `Missing` leaf
rather than assuming 2,048 wrappers.  Assert the exact complete rebuilt value for recursive and finite
target modes.  Three-level companions pin the selected deletion level and scan
depth.  Repeat under an armed memory limit and with a once-suspending delayed
entry.

##### rum / Rust pitfalls (read twice)

Do not change delayed evaluation, source order, memory projection, or the point
at which a resource error is reported.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run
fmt, stdlib/workspace tests, memory/suspension tests, and release certificates.
Gate alloc counts and `instructions:u` for shallow recursive/finite modes with
memory limit off and on plus association neighbors.

##### Falsifier

Point the shallow companion at the adjacent level and show the
exact survivor/deletion assertion red; force heap entry at one and trace it.
For preflight, perturb one delayed-wrapper count and show the focused projected-
bytes assertion red.

##### Hard rules

Standard template rules apply; no weakened memory contract or eager callbacks.
Stop on semantic/callback drift or a default-path regression.

##### Report format

Report each removed recurrence, falsifiers, exact outputs, tests, and gates.

#### Draft E — nested multi-spec `GroupBy`

##### Goal

Make list and association `GroupBy` with deeply nested spec lists use
bounded native stack while preserving spec order, grouping order, value
functions, reducers, and continuation behavior.

##### Context pointers

`crates/rum-stdlib/src/associations/grouping.rs:110-272`,
`crates/rum-stdlib/src/associations/grouping/list.rs:5-513`,
`crates/rum-stdlib/src/associations/grouping/association.rs:5-526`, and both
grouping `resume_frames.rs` modules; Draft A for the nested association result.

##### Scope

GroupBy implementation/frame modules, focused tests, depth
certificate, and benchmark label.  `KeyGroupBy`, `MergeNested`, generic
evaluator code, and docs are out.

##### Deliverable

Turn spec-depth recursion into an explicit stack of grouped
parents and finish tasks shared in shape between list and association inputs.
Freeze/trace it on suspension and never replay key/value functions or reducers.
Keep a shallow front only if the instruction gate supports it.

Certificate: use 2,048 distinguishable constant-key grouping specs, a minimal
input, and a `First` reducer so the leaf is atomic; assert with an iterative
counter that the result has raw association height 4,097 and one unique deepest
payload, then assert the exact whole result.  Three-level companions pin
outer-to-inner spec order, Rule/RuleDelayed value-function meaning, and reducer
placement for both list and association input.

##### rum / Rust pitfalls (read twice)

The large `finish_*` and child driver frames are both resident today; shrinking
one is insufficient.  Preserve source-order groups and suspension frame root
tracing.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt, stdlib/workspace,
suspension, and release certificate tests; gate alloc counts and
`instructions:u` for one-spec, three-spec, wide-group, and both input variants.

##### Falsifier

Reverse two shallow specs and show the exact nesting assertion
red; use a once-suspending key function with a call counter to detect replay.
Force the heap path at one in a test-only build.

##### Hard rules

Standard template rules and stop-loss apply.

##### Report format

Report exact values, callback counts, red/green falsifier, tests, and
allocation/instruction gates.

#### Draft F — association key-path assignment

##### Goal

Replace `set_association_key_path` native recursion with a bounded
descent and explicit reverse rebuild, preserving missing-key diagnostics,
`RuleDelayed` normalization, duplicate-key canonicalization, and scalar return.

##### Context pointers

`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:934-1020`,
`crates/rum-runtime/src/runtime/definitions/part_assignment.rs:10-54`,
association assignment tests, and Draft A for the later target read.

##### Scope

Kernel association-data implementation and runtime association-set
tests/depth certificate.  Generic exact paths, stdlib held mutation, evaluator
stability, and docs are out.

##### Deliverable

Descend iteratively while storing the selected entry index and
owned entry vector per ancestor; rebuild in reverse only after the leaf update.
Return the same `AssociationSetUpdate` variant and preserve the first missing
key.  A bounded borrowed front may retain the shallow fast path if neutral.

Certificate: assign `8` to the unique deepest value of an `N=2048` chain,
assert raw height 4,097 before/after, assert the scalar return, then iteratively
assert every key/rule and the exact complete stored value.  Three-level
companions pin bare string and `Key[...]` selector semantics, a missing middle
key, and conversion of the updated delayed entry to immediate.

##### rum / Rust pitfalls (read twice)

Avoid deep clones; each ancestor should be parsed and retained once.  Do not
change epoch/cache invalidation.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt, runtime,
workspace, and release certificate tests; gate allocations and
`instructions:u` on one-level updates, existing incremental association-set
labels, missing keys, and deep-tail setup excluded from timed statements.

##### Falsifier

Omit the middle selector in the shallow companion and capture
the exact-value failure; force fallback at depth one and assert its trace.

##### Hard rules

Standard template rules apply; stop for diagnostic/epoch drift or a hot-path
regression.

##### Report format

Report rebuilt identity behavior, tests, falsifier, and measured gates.

#### Draft G — shared exact paths for held association mutation

##### Goal

Bound exact path collection, lookup, and reverse replacement used by
`AssociateTo`/`KeyDropFrom` Part targets, preserving multi-selection order,
head selection, key-to-Rule-RHS mapping, diagnostics, and stored/returned value
contracts.

##### Context pointers

`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:3-200`,
`crates/rum-stdlib/src/associations/mutation.rs:45-130,521-692`, existing
ReplacePart/Part and held mutation tests, and Draft A.

##### Scope

Exact-path module/root, association mutation tests, affected runtime
exact-path tests, release certificate, and benchmark labels.  Direct
symbol-only association mutation and general selector semantics are out.

##### Deliverable

Use explicit selector expansion tasks for path collection, an
iterative exact lookup, and a recorded-ancestor reverse rebuild.  Preserve the
ordered application of multiple replacement paths and all COW sharing.  Keep
three helpers separately testable even if they share a frame type.

Certificates: at a unique deepest Part target in an `N=2048` association,
`AssociateTo` adds one distinguishable entry and returns/asserts the exact
whole target; `KeyDropFrom` removes one deepest entry and then the stored owner
is asserted exactly.  Raw height and every untouched ancestor are checked.
Three-level companions pin `Key[k]` expanding to association argument then
Rule RHS, numeric/head selector behavior, and return-value differences.

##### rum / Rust pitfalls (read twice)

The path contains two raw components per logical key; tests must count raw
calls, not use `Depth`.  Preserve replacement overlap order and messages.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt, runtime/stdlib/workspace tests and release
certificates.  Gate alloc counts and `instructions:u` for shallow Part,
ReplacePart, AssociateTo/KeyDropFrom, and the existing association read labels.

##### Falsifier

In a test-only mutation, omit the Rule-RHS component `2`; the
three-level exact owner/return assertions must fail.  Force each heap helper at
depth one and require its trace.

##### Hard rules

Standard template rules apply; stop on selector, ordering, sharing, or
performance drift.

##### Report format

Report each helper independently and paste all gates.

#### Draft H — iterative association key reshape

##### Goal

Make `AssociationKeyFlatten` and `AssociationKeyDeflatten` bounded by
heap work rather than native association/path depth while preserving path
order, overwrite/conflict behavior, rule heads, and exact inverse cases.

##### Context pointers

`crates/rum-stdlib/src/associations/key_reshape.rs:105-143,340-440`, its tests,
and Draft A for the deep deflattened result.

##### Scope

Key-reshape implementation, a responsibility-based helper module if
needed, key-reshape tests/depth certificates, and one benchmark label.  GroupBy
code in the same file and generic stability are out.

##### Deliverable

Replace `insert_path` recursion with an explicit path cursor
that preserves `NestedPathNodes` indexing, render branches with finish tasks,
and flatten with ordered visit/finish tasks.  Preserve conflicts between a leaf
and branch and the original entry order.

Certificates: deflatten one 2,048-component unique key path into a raw-height
4,097 association and assert every key plus deepest value exactly; flatten that
value and assert the exact original flat association.  Include siblings and a
`RuleDelayed` leaf.  Three-level companions pin left-to-right key order,
scalar-key behavior, and leaf/branch conflict results.

##### rum / Rust pitfalls (read twice)

LIFO scheduling must reverse pushes to preserve source order; do not clone
every path prefix quadratically.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt,
stdlib/workspace and release certificates; gate allocations and
`instructions:u` for shallow and wide flatten/deflatten plus KeyGroupBy
neighbors.

##### Falsifier

Schedule the three shallow keys without reverse-pushing; the
exact round-trip/order companion must fail.  Force heap fallback at one.

##### Hard rules

Standard template rules apply; stop on conflict/order/delayed-head or
performance drift.

##### Report format

Report exact red/green and all gates.

#### Draft I — residual `MapIndexed` and `Scan` traversal

##### Goal

Give `MapIndexed` and `Scan` the same depth-16 front plus explicit
visit/finish fallback already used by `Map`/`Apply`, with unchanged level,
heads, position, visit-order, and callback semantics.

##### Context pointers

`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-299,301-487`,
`crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:59-74,113-131`, traversal
tests, and Draft A.

##### Scope

Apply/map traversal module (split by responsibility if required),
focused traversal/depth tests, and benchmark labels.  Normalization and level-
spec parsing are out.

##### Deliverable

Reuse or generalize `TransformWork` without changing the
already-fast `Map`/`Apply` path.  `MapIndexed` finish tasks carry the exact
logical position path; `Scan` tasks append selected nodes in current traversal
order before sequential evaluation.  No callbacks may run inside structural
collection.

Certificates: `MapIndexed` transforms one unique deepest sentinel in an
`N=2048` association, input raw height 4,097, and the exact whole rebuilt value
including the deepest position is asserted.  `Scan` returns exact `Null` and an
exact full visit log (its semantic output is not a rebuilt tree).  Three-level
companions pin the descending level selector, association-value position
convention, Heads option, and Scan order.

##### rum / Rust pitfalls (read twice)

Do not regress existing Map/Apply allocations or instructions; path
construction must not become quadratic.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt, stdlib,
workspace, release certificates; gate alloc counts and `instructions:u` for all
four traversal families and same-tier rewrite neighbors.

##### Falsifier

Give the shallow companions the old non-descending selector and
show exact output/log failures; force fallback at one and trace it.

##### Hard rules

Standard template rules apply; stop if Map/Apply move beyond their same-binary
spread or visit semantics change.

##### Report format

Report all four families' exact results, tests, allocation counts, instruction
clusters, and stop-loss verdict.

#### Draft J — association collection parser tails

##### Goal

Remove unbounded native recursion from both copies of deep
association-entry parsing and from the ordinary `Merge` nested-list collection
parser without changing flattening, error attribution, rule-head preservation,
or duplicate canonicalization.

##### Context pointers

`crates/rum-stdlib/src/associations/data.rs:163-189,204-260`,
`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:57-133,263-316`,
and parser consumers.

##### Scope

The two association data modules, their module roots if needed,
focused parser/association/Merge tests, and depth certificates.  Do not alter
public expression representation or Merge aggregation.

##### Deliverable

Prefer one kernel API if dependency direction permits without
widening the facade; otherwise implement matching explicit container stacks in
each owner.  Preserve depth-first left-to-right flattening, first invalid
element attribution, `Rule` versus `RuleDelayed`, and canonicalization timing.

Certificates: after conversion only, parse a raw-height-4,097 alternating
`List`/`Association` item nest with one unique deepest rule and assert the exact
flat association; parse a 4,097-high nested Merge collection and assert its
exact merged output.  Three-level companions pin container flatten order,
invalid-element identity, duplicate-key result, and delayed rule head.

##### rum / Rust pitfalls (read twice)

A parser work stack must borrow while possible and must not reverse diagnostics.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run fmt, runtime, stdlib, workspace, and release
certificates.  Gate allocation counts and `instructions:u` for shallow
association literals, ordinary Merge, and invalid-input neighbors.

##### Falsifier

Deliberately push shallow siblings in forward LIFO order; exact
order and first-invalid companions must fail.  Force the heap path at one.

##### Hard rules

Standard template rules apply; stop on dependency-boundary widening,
diagnostic drift, extra shallow allocation, or instruction regression.

##### Report format

Report whether code was shared, red/green companions, exact outputs, and all
gates.

### 8. Risks and open questions

1. **Contract nomenclature must be fixed before dispatch.** This brief calls
   4,097 “physical height,” so the certificates above use 2,048 association
   wrappers and independently count raw `Expr::Call` nodes.  Existing prose
   that labels `N=4097` physical is off by one whole `Rule` layer.  If the
   intended product contract is instead 4,097 *logical association* levels,
   say so explicitly; the implementation stacks still work, but gate sizes and
   Map output height change.
2. **Draft A is the highest performance risk.** Two general iterative stability
   shapes already passed semantics and lost 2-7% on hot miss families
   (`docs/dev/codex/brief-442-stability-frame-shrink.md:26-32`).  An association-only exhausted-tail transition avoids
   their generic control-flow cost, but cache insertion order, early-unstable
   behavior, and heap allocation remain acceptance-critical.
3. **Suspension state is the highest correctness risk.** `MergeNested`, schema,
   `DeleteMissing`, and `GroupBy` can all call the evaluator while parent work
   is pending.  Explicit tasks must be `FrameRootTrace`-complete, freeze without
   deep clones, and resume without replay.
4. **Reasoned ceilings are not measurements.** Optimizer inlining, fixed entry
   stack, page quantization, and polling phase explain small discrepancies.
   They select implementation order; only post-conversion exact certificates
   decide acceptance.  No pre-fix probe should seek an unknown ceiling.
5. **Construction teardown is a separate latent boundary.** The evidence points
   strongly to natural recursive `Expr` drop, but no direct stack trace was
   taken.  A separate design brief should decide between a safe manual iterative
   drop, a host-owned teardown strategy, or a documented maximum.  Any `Expr`
   `Drop` change is core hot-path work and requires allocation/instruction plus
   full rum-vs-rum A/B gates; it is not needed to certify raw height 4,097.
6. **Exact certificates must not introduce their own recursion.** Avoid deep
   `PartialEq`/`Display` (`crates/rum-core/src/expr.rs:755-775,850-873`).  Assert raw height and exact
   structure with an iterative Rust helper or compare InputForm strings built
   iteratively.  Teardown of Map's taller output must stay below the known
   construction/drop bracket or use the existing test-only iterative dismantler
   (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:552-559`).
7. **Resource accounting can move from native stack to heap.** Each explicit
   machine needs allocation-count gates and, where user-controlled breadth can
   enlarge pending state, existing memory-limit enforcement.  “No stack growth”
   is not permission for unbounded unaccounted heap growth.
8. **Adjacent nested-list surfaces remain.** `KeyTake`/`KeyDrop` are reasoned
   below raw-list height 4,097 and should receive a separate generic collection-
   shape brief.  They do not explain any supplied association fixture and
   should not be smuggled into an association-value patch.

## Director gate — 2026-09-04

**Scope**: clean, this file only, 1,011 insertions, no source change.

### Verified

**The height-unit correction is right, and it is the most consequential thing
in this report.** `Association["k" -> a]` renders as `Association[Rule[…]]`, so
one logical wrapper is two physical levels. Confirmed directly: at `N=2` the
`FullForm` is `Association[Rule["k", Association[Rule["k", 1]]]]` — physical
height 5 = 2N+1 — against `List[List[1]]` = N+1 for the list fixture, and
`Depth` reports 3 for both. My brackets were logical wrappers compared against a
physical-height contract, so the shortfalls I quoted were overstated about
twofold. The corrected figures are 2.10x (`Map`), 1.78x (assignment) and 1.76x
(`MergeNested`).

**Nested part assignment is bound by the read, not by its own recursion.**
Confirmed with an operation-only probe that returns the assigned scalar and
never reads the rebuilt value: it clears `N=2500` (physical 5,001), where the
read-the-result form dies at 1,145. `set_association_key_path` recursion is real
but is not what the certificate was measuring.

### Refuted — section 5's teardown attribution

The common 5,455/5,463 bracket is **the construction loop**, in band, not
teardown.

- At the first failing height the probe emits
  `evaluation exceeded native stack limit (1048576 bytes)` and **never prints
  its liveness line**, so the failure precedes any output.
- Splitting the probe — build, `Print["BUILT"]`, then `Depth` — never reaches
  `BUILT`, so it is not `Depth` either.
- Construction alone, with no operation and no `Depth`, brackets at
  **5,457 / 5,463**: the same bracket as all four probes that "clear the
  contract". They share it because they share the fixture, not because they
  share a teardown.

`1 MiB / 5,460 ≈ 192.0 B` per logical wrapper, so association construction pays
one generic stability packet per wrapper. The report's own observation that all
five shapes returned normally at `N=5400` is explained by this and does not
require a drop hypothesis; its `/proc` samples measured live operations, which
are indeed cheap, and did not sample the construction loop itself.

The report was careful to mark this as "strongly supported attribution, not a
claimed exact disassembly proof", which is the right register — but the
conclusion it supports is wrong, and it would have sent a later reader to
optimize `Expr::drop`, a lane the standing engineering debt already records as
tried and rejected. **Section 5's practical conclusion is unaffected**: the
ceiling does not cap certification of the ruled target, because raw height 4,097
needs only `N=2048`.

### Verdict

Accepted as a diagnosis, with section 5 corrected. The mechanism finding — that
`Map` and post-assignment reads are bound by the **association arm** of the same
stability walk this line already shrank on its generic arm, and that the arm
costs a 400 B association helper plus a 112 B rule helper on top of the 192 B
generic packet — is the finding that reorganizes the work, and it is consistent
with every bracket I measured. `MergeNested` is a genuinely separate native
recursion at about 1,008 B per logical level.

Draft A is dispatched first and alone: it is the shared root, it unlocks `Map`,
post-assignment reads and every other producer of a fresh deep association, and
it touches the hottest path in the system, so it carries the full instruction
and allocation gate. Drafts B through J stay drafts and will be re-derived
against post-A measurements rather than dispatched from projections — the same
rule that turned draft 5 into two sites and draft 6 into two briefs.
