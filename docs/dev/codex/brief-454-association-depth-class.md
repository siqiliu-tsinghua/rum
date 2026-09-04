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
