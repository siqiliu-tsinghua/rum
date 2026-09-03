# brief-436 — why some heads cannot be nested past a few hundred levels

**Shape: diagnostic. `Scope: NONE — no source change.`** I have done the
measurement; the source is yours. Return the mechanism and a repair plan.

## What I measured, so you do not need a single probe of your own

Release CLI, `ulimit -s 2048` (2 MiB, Rust's default for a spawned thread),
values built with `s = "a"; Do[s = HEAD[s, "a"], {i, N}]`. Every failure below
was re-run and confirmed to be `thread 'main' has overflowed its stack`, not an
in-band error and not a timeout.

At N = 3,000:

| Head | Result |
|---|---|
| `foo` (undefined Global symbol) | completes, `Depth` 3,001 |
| `List` | completes, 3,001 |
| `Rule` | completes, 3,001 |
| `Sin` | completes, 3,001 |
| `StringExpression` | **stack overflow** |
| `Alternatives` | **stack overflow** |
| `StartOfString` | **stack overflow** |
| `WhitespaceCharacter` | **stack overflow** |
| `DigitCharacter` | **stack overflow** |
| `Blank` | **stack overflow** |
| `Association` | **stack overflow** |

`Hold` and `Times` were also tried and are not valid probes for this: the
resulting value does not nest (`Depth` 2), because one holds and the other
flattens.

Bracketed precisely for `StringExpression`: **completes at 375, overflows at
550**, and the failure happens *inside the construction loop* — a progress
print shows `built:400` and then the overflow, with no traversal involved. A
plain `foo` spine reaches 20,001 on the same stack. That is a 45x difference
between two heads doing what should be the same amount of work.

`StringExpression` carries only `{Protected}` and is registered as an inert
`BuiltinSpec::with_attributes` with no handler
(`crates/rum-stdlib/src/catalog/strings.rs:95-100`), so nothing about a
*handler* explains it.

## What I ruled out, with evidence

- **Not string-pattern parsing.** Every caller of `parse_string_pattern` is a
  string *operation* — `StringMatchQ`, `StringReplace`, tokenize
  (`crates/rum-stdlib/src/strings/patterns/search.rs:61,386,429,510,552`,
  `crates/rum-stdlib/src/strings/tokenize.rs:131`). None runs at construction,
  and `Association` and `Blank` are not string patterns at all.
- **Not the pattern-head registry.** Only `Alternatives` of the failing set
  appears in `crates/rum-runtime/src/runtime/pattern_syntax.rs:53`;
  `StringExpression` and `StartOfString` do not.
- **Not "is a System builtin".** `Sin`, `Rule` and `List` are System heads and
  all complete.

## My hypothesis, which I want confirmed or refuted from source

The evaluation-stability analysis. brief-428's survey recorded that a
stability-cache hit requires exact head/args pointer identity
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-243,332-349`)
and that **on a miss the uncached analysis recursively visits heads and args**
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33`). If some heads
take a cached or short-circuited path and others re-walk the whole spine on
every construction, that is both the 45x and the recursion — O(depth) per step,
recursive, with the failure landing inside the loop exactly where I see it.

**Refute it if the source says otherwise.** A refutation with the real
mechanism is worth more than a confirmation; brief-427 refuted the previous
mechanism hypothesis on this line and that was its most valuable result.

## Deliverables

1. **The mechanism**, with `path:line` for every step: what runs per
   construction for `StringExpression` that does not run for `List`, why it
   recurses, and what the recurrence is per level. Name the exact predicate or
   table that sorts these heads into the two groups — I could not find it, and
   a predicate that splits `Association` and `Blank` from `Rule` and `List` is
   the centre of this.
2. **The full membership of the slow group.** If it is a table or an attribute,
   enumerate it. If it is a structural property, state it and name three more
   heads it predicts — I will test them. **A prediction I can falsify is the
   deliverable here**, not a list of examples.
3. **Whether the cost is also asymptotic, not just deep.** If every
   construction re-walks the spine, then building N levels is O(N²) as well as
   O(N)-deep. Say whether that follows, because it changes what a repair has to
   fix: an iterative walk removes the crash but leaves the quadratic.
4. **The repair options**, with what each costs and what enforces it. The
   line's precedents are the depth-16 recursive front path with an
   explicit-stack fallback
   (`crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-87`,
   `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-61`,
   and now `crates/rum-runtime/src/runtime/resolution/match_normalization.rs`),
   and the cheaper one where it applies: not doing the work at all.
5. **Drafted implementation briefs** in `docs/dev/codex/TEMPLATE.md` shape, one
   per step, each naming its A/B panel and its stop-loss. Any brief that
   touches an evaluation path needs the microbench gate; say which do.

## Method requirements

- Every claim carries `path:line`. A claim without evidence is a guess and
  should be labelled one.
- **Open the file that defines a thing, not the first file that mentions it.**
  A `grep -rl | head -1` returning a reference rather than a definition is a
  mistake this line has already made.
- If you cannot separate two candidate mechanisms from source, say so and name
  the probe that would separate them. I will run it — do not write it.

## Scope

- Files you MAY modify: **NONE**, except appending your report to
  `docs/dev/codex/brief-436-construction-depth-class.md`.
- No source change, no test, no probe program, no benchmark run.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No network. Touch nothing outside this repository.
- **Write no probe that builds a deep value to see it fail.** Every number this
  brief needs is already above; there is no reproduction to write and none is
  wanted.
- The Testing Policy in `AGENTS.md` is a hard line.
- **Append your report to this file** before you finish.

## Report format

1. The mechanism, with the predicate that splits the two groups.
2. The membership of the slow group, and three falsifiable predictions.
3. Whether the cost is quadratic as well as deep.
4. Repair options, priced, with what enforces each.
5. The drafted implementation briefs.
6. What you could not establish from source, and the probe that would settle it.
