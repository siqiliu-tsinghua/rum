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

---

## Appended diagnostic report — 2026-09-03

### 1. Mechanism and the exact split

**Verdict: the stability-cache hypothesis is half confirmed and half
refuted.** Exact pointer-identity stability hits are what make the fast group
incremental, but the native recursion in the slow generic group is not the
uncached stability walker recursively visiting the old spine. For a registered
head such as `StringExpression`, stability returns `false` *before* visiting
the arguments; ordinary evaluator argument preparation then recursively
evaluates the old spine. The early return is
`call_head_has_evaluation_effect` at
`crates/rum-runtime/src/runtime/eval_core/stability.rs:21-23`; the argument
stability walk it skips is at
`crates/rum-runtime/src/runtime/eval_core/stability.rs:25-31`.

The path for one `s = StringExpression[s, "a"]` iteration is:

1. `Set` evaluates every non-self-evaluating RHS by calling `Session::evaluate`
   (`crates/rum-runtime/src/runtime/definitions/definitions.rs:22-56`). Reading
   `s` during that evaluation finds its stored immediate own value and evaluates
   that value again if it is a call
   (`crates/rum-runtime/src/runtime/eval_core/symbol.rs:71-75,122-158`). Thus the
   preceding spine is an evaluator input, not merely copied into the new call.
2. Every `Session::evaluate` fixed-point pass asks
   `cached_expr_is_evaluation_stable` first and only calls the unstable
   evaluator if the answer is false
   (`crates/rum-runtime/src/runtime/resolution.rs:305-340`). A call-cache lookup
   is keyed by the `SharedExpr` head pointer plus the `SharedArgs` pointer
   (`crates/rum-runtime/src/runtime/model/cache.rs:480-500`), and a hit additionally
   requires the same exact pointers, generation, and a non-dirty dependency
   bloom (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-243,332-349`).
3. On a cache miss, the stability classifier first checks the head, then the
   data-constructor table, then the head-effect predicate, and only if all three
   allow it does it inspect arguments
   (`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33`). For
   `StringExpression`, the data-constructor table has no matching arm
   (`crates/rum-runtime/src/runtime/eval_core/stability.rs:36-79`).
4. The **exact generic split predicate** is currently “does this System symbol
   have *any* registered `BuiltinSpec`?”:
   `cached_builtin_dispatch_spec_for_symbol(symbol).is_some()` appears in both
   the argument-effect shortcut and the cached head-effect computation
   (`crates/rum-runtime/src/runtime/eval_core/stability.rs:249-293,295-346`). The
   lookup returns the whole registered spec without considering its dispatch or
   handler (`crates/rum-runtime/src/runtime/resolution.rs:68-95`). This is wider
   than actual dispatch: `BuiltinDispatch` explicitly distinguishes `None`,
   `SpecialForm`, and `Ordinary`, while `BuiltinSpec::with_attributes` sets
   `dispatch: None` and `handler: None`
   (`crates/rum-runtime/src/runtime/metadata.rs:283-288,334-343,381-396`).
5. The real call planner always schedules argument preparation for a
   non-special call, but schedules an ordinary builtin stage only for
   `BuiltinDispatch::Ordinary`
   (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:42-99`). A missing
   handler is also a no-op at dispatch
   (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:173-187`). Consequently,
   registration alone is an evaluation effect only in the stability model, not
   in the dispatch model.
6. `StringExpression` has only `Protected` and is constructed with
   `BuiltinSpec::with_attributes`
   (`crates/rum-stdlib/src/catalog/strings.rs:89-100`), so its call plan eagerly
   prepares both arguments and has no builtin stage. A non-held argument is
   assigned `CallArgumentMode::Evaluate`
   (`crates/rum-runtime/src/runtime/call_preparation.rs:115-167`); preparation
   invokes `evaluate_atom_fast_path`, and that function invokes full
   `Session::evaluate` for every call argument
   (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`,
   `crates/rum-runtime/src/runtime/eval_core/mod.rs:136-145`). This is the native
   recursion down the first-argument spine.
7. Evaluation leaves each inert call unchanged. When an argument compares equal,
   preparation retains the original `SharedArgs`
   (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:262-328`), and the final
   dispatch stage rebuilds the call from those shared parts
   (`crates/rum-runtime/src/runtime/eval_core/call.rs:120-135`). `Expr` equality
   itself short-circuits on those same shared pointers
   (`crates/rum-core/src/expr.rs:749-775`), so the fixed-point loop then returns
   the unchanged value (`crates/rum-runtime/src/runtime/resolution.rs:362-366`).
   The walk has nevertheless already consumed one native evaluator frame per
   spine level.
8. No unstable verdict is cached: the call-stability cache inserts only when
   `stable` is true
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:293-329`). Thus
   none of the unchanged `StringExpression` nodes acquires the proof that would
   stop the next iteration at the old root.
9. `Set` finally stores the evaluated RHS as `s`'s own value and performs a
   symbol-scoped stability invalidation
   (`crates/rum-runtime/src/runtime/definitions/definitions.rs:454-493`). That
   invalidation removes `s`'s symbol caches and records its bloom bit rather
   than clearing every call proof
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:49-89,91-164`).
   **Source-derived inference:** after RHS evaluation, a fast group's stored
   spine contains the prior *value* rather than the symbol `s`, so its bloom
   has no semantic dependency on `s`. A bloom is a hashed conservative summary,
   so a bit collision can still cause a false-positive invalidation
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:18-38`); absent
   such a collision—as the measured fast controls demonstrate—the exact-pointer
   proof survives the write under the dirty-delta test
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:332-358`).

That explains the controls:

| Head | Classifier route | Why construction is fast or slow |
|---|---|---|
| `foo` | No data-constructor arm, no registered spec, and no definition | The generic classifier reaches its argument walk; once `s` has been replaced, the new root is stable and is cached. The next iteration hits the old root by pointer (`crates/rum-runtime/src/runtime/eval_core/stability.rs:21-33,249-346`; `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-243,293-321`). |
| `List` | Same generic route as `foo` | `List` is a canonical System head, but the runtime structure builtin table is empty, so it has no `BuiltinSpec` solely by being canonical (`crates/rum-core/src/heads.rs:112-115`; `crates/rum-runtime/src/runtime/builtin_catalog/structure.rs:1-3`). The `Nothing` special case is the only listed List effect and is absent from this workload (`crates/rum-runtime/src/runtime/eval_core/stability.rs:213-224`). |
| `Rule` | Data-constructor override before the generic registered-builtin test | An exact two-argument `Rule` checks and accumulates stability for its lhs and rhs, allowing each completed root to be cached (`crates/rum-runtime/src/runtime/eval_core/stability.rs:47-69`). Its registered special-form handler therefore does not put a stable constructed rule into the generic slow class (`crates/rum-runtime/src/runtime/builtin_catalog/rules.rs:407-416`). |
| `StringExpression`, `Alternatives`, `Blank`, and the other metadata-only strict heads below | No data-constructor override; registered-spec predicate is true | Stability returns false without looking at the old child, no false verdict is cached, and eager argument preparation recursively evaluates it (`crates/rum-runtime/src/runtime/eval_core/stability.rs:15-31,249-346`; `crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`). |
| `Association` for the exact measured two-argument shape | `Association` data-constructor override, but that override returns false immediately | Its stability loop accepts only a direct two-argument `Rule`/`RuleDelayed`; the first nested `Association` is not such an entry, so it returns false before checking or caching that child (`crates/rum-runtime/src/runtime/eval_core/stability.rs:113-149`; `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:245-260`). Eager preparation then recurses, and the ordinary handler additionally deep-parses nested Association/List items (`crates/rum-stdlib/src/catalog/associations.rs:8-14`; `crates/rum-stdlib/src/associations/basic.rs:13-34`; `crates/rum-stdlib/src/associations/data.rs:234-260`). |

`Sin` needs a correction. **Source-derived inference from a negative catalog
survey, not a runtime observation:** this clean checkout does not install or
recognize an unqualified `Sin` as a System builtin. A System candidate exists
when `symbol_exists` finds session metadata/definitions or the known-System
fallback (`crates/rum-runtime/src/runtime/resolution.rs:116-153`); that fallback
is the canonical-head list, core catalog, and explicit extra-name list
(`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:347-417`). `Sin` is
absent from the two explicit name enumerations
(`crates/rum-core/src/heads.rs:112-194` and
`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:375-417`), and the source
survey found its only crate-source uses in polynomial tests rather than a
catalog definition (`crates/rum-stdlib/src/tests/polynomials.rs:16-40,72,281-293`).
A missing unqualified name is created in the current context
(`crates/rum-runtime/src/runtime/resolution.rs:97-123`). Therefore the source
predicts that the measured `Sin` was an undefined ``Global`Sin``, semantically
the same control as `foo`, unless that CLI session had prior state not recorded
in the brief. This is not evidence that a registered System builtin follows the
fast route.

### 2. Full membership and falsifiable predictions

There is **no immutable head-only slow list**. The exact membership of this
construction class is structural and session-relative because builtin specs are
inserted into each `Session` by `register_builtin`, which can merge a fallback
and change attributes (`crates/rum-runtime/src/runtime/session/session_api.rs:265-281`),
while definitions and attributes also participate in the two call-effect
predicates (`crates/rum-runtime/src/runtime/eval_core/stability.rs:249-346`).

For the exact workload `s = "a"; Do[s = H[s, "a"], ...]`, a head is in the
slow/deep class precisely while all three properties hold at every retained
level:

1. `H[old, "a"]` evaluates to a residual call retaining the next `H` node,
   rather than flattening, holding away the read of `s`, or rewriting to another
   value; actual dispatch is determined by the staged plan at
   `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:42-99`.
2. The stability route returns false for each residual node and therefore does
   not install a call proof
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:293-329`). For
   the generic observed heads, the sufficient branch is: stable symbolic head,
   no data-constructor override, and either a registered spec or another effect
   reported by `call_head_has_evaluation_effect`
   (`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33,213-346`). For the
   measured `Association`, malformed direct-entry classification supplies the
   false verdict instead (`crates/rum-runtime/src/runtime/eval_core/stability.rs:113-149`).
3. The retained child is in an eagerly evaluated argument position, so argument
   preparation calls back into `Session::evaluate`
   (`crates/rum-runtime/src/runtime/call_preparation.rs:115-167`;
   `crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`). A
   `BuiltinDispatch::None` spec alone is not sufficient: `HoldForm` also has no
   dispatch, but its `HoldAll` attribute makes ordinary arguments use Hold mode
   (`crates/rum-runtime/src/runtime/builtin_catalog/functional.rs:16-21`;
   `crates/rum-runtime/src/runtime/call_preparation.rs:120-152`).

The data-constructor exception table is exhaustive and contains:
`Association`, `Failure`, `Missing`, `Success`, `TerminatedEvaluation`,
`HoldAllComplete`, `HoldComplete`, `Function`, `Hold`, `HoldAll`, `HoldPattern`,
`Rule`, and `RuleDelayed`
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:36-79`). Membership still
depends on the named arm's shape predicate: `Rule[_, _]` can become stable,
whereas the measured malformed `Association[old, "a"]` cannot
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:57-76,113-149`).

The complete metadata-only string-symbol table installed by the default
runtime/stdlib pair is a concrete strict, residual subclass of the slow group:

`Blank`, `BlankSequence`, `BlankNullSequence`, `Pattern`, `Alternatives`,
`Except`, `StringExpression`, `RegularExpression`, `Whitespace`,
`WhitespaceCharacter`, `LetterCharacter`, `DigitCharacter`, `WordCharacter`,
`NumberString`, `PunctuationCharacter`, `IgnoreCase`, `Overlaps`,
`StartOfString`, `EndOfString`, `StartOfLine`, `EndOfLine`, `WordBoundary`,
`Repeated`, `RepeatedNull`, `Shortest`, `Longest`, `TemplateObject`,
`DiffObject`, `ByteArray`, `InputForm`, `OutputForm`, `StandardForm`,
`TraditionalForm`, and `Automatic`.

All 34 are defined as protected `BuiltinSpec::new` entries with no dispatch in
`crates/rum-runtime/src/runtime/builtin_catalog/strings.rs:4-179`; 18 receive
equivalent protected `with_attributes` overlay specs from
`crates/rum-stdlib/src/catalog/strings.rs:8-118`. Core specs are registered
first, the stdlib string specs later, and fallback merging preserves dispatch
only when one side supplies it
(`crates/rum-stdlib/src/lib.rs:42-45,116-118`;
`crates/rum-runtime/src/runtime/metadata.rs:527-542`). None is in the
data-constructor exception table
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:36-79`).

Three cross-catalog predictions, all falsifiable with the brief's exact N=3,000
release/2-MiB construction, are:

- `General` will preserve the spine and stack-overflow. It is a protected
  no-dispatch metadata head with messages but no handler
  (`crates/rum-runtime/src/runtime/builtin_catalog/inert_heads.rs:3-59`).
- `UpTo` will preserve the spine and stack-overflow. It is the second protected
  no-dispatch metadata head in that catalog
  (`crates/rum-runtime/src/runtime/builtin_catalog/inert_heads.rs:60-75`).
- `XmlElement` will preserve the two-argument spine and stack-overflow. Its
  catalog entry is protected `BuiltinSpec::with_attributes` with no handler
  (`crates/rum-stdlib/src/catalog/xml.rs:41-46`), hence it has the same generic
  stability and eager-argument path as `StringExpression`
  (`crates/rum-runtime/src/runtime/eval_core/stability.rs:21-33,249-346`).

These predictions are about the exact residual two-argument construction, not
about uses in which another evaluator consumes those heads; a rewriting
handler or a hold/flattening attribute can make the same symbol leave the class
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:51-99`;
`crates/rum-runtime/src/runtime/call_preparation.rs:120-167`).

### 3. Depth and asymptotic cost

For a strict metadata-only head such as `StringExpression`, let `T(d)` be the
work to evaluate one already-built residual spine of depth `d`. At each node,
stability rejects in constant work, eager preparation recursively evaluates the
one nested child, and the unchanged call returns in constant additional work;
therefore

```text
T(0) = Theta(1)
T(d) = T(d - 1) + Theta(1) = Theta(d)
native stack(d) = native stack(d - 1) + Theta(1) = Theta(d)
```

The premises are the pre-argument false return
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:21-23`), recursive call
argument evaluation (`crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`),
and unchanged shared-argument reconstruction
(`crates/rum-runtime/src/runtime/eval_call_prepare.rs:262-328`;
`crates/rum-runtime/src/runtime/eval_core/call.rs:120-135`). The RHS may walk a
spine a fixed number of times across symbol substitution and the final
fixed-point pass, but that changes only the constant because the loop exits on
the first equal result (`crates/rum-runtime/src/runtime/resolution.rs:305-366`).
This is the supplied unconstrained CLI path: the fixed-point resource precheck
performs a full expression proxy walk only while an explicit memory constraint
is active (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:139-161`).
Summing one `Theta(d)` construction over `d = 1..N` gives
**Theta(N^2) total construction work and Theta(N) peak native stack**. An
iterative evaluator would remove only the latter.

The measured malformed `Association` case is worse in time. Its stability
check rejects the nested first item in constant work
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:113-124`), but after
eagerly evaluating that child, each ancestor's ordinary handler recursively
parses its first nested Association down to the invalid atom before returning
the original call
(`crates/rum-stdlib/src/associations/basic.rs:18-34`;
`crates/rum-stdlib/src/associations/data.rs:234-260`). Hence
one spine evaluation has

```text
T_assoc(0) = Theta(1)
T_assoc(d) = T_assoc(d - 1) + Theta(d) = Theta(d^2)
```

and the sequential measured construction is **Theta(N^3) total work with
Theta(N) peak native stack**. The cubic statement applies to the exact
`Association[s, "a"]` ordering: the deep parser visits the nested first item
before it could reject the top-level second string
(`crates/rum-stdlib/src/associations/data.rs:234-260`).

For the fast group in the measured range, each new outer node is certified and
the prior child is an exact-pointer hit, so the incremental step is constant
work (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-243,293-321`).
This statement is deliberately bounded to the measured range: the pointer
cache has a fixed default cap of 16,384 entries and clears on overflow
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:6-16,301-304,361-367`),
so an unbounded-asymptotic claim for the fast group would require a separate
cache-eviction analysis.

### 4. Repair options, price, and enforcement

The recommended sequence is A then B. Both use the cheaper precedent—prove
that evaluation has no work to do—rather than first making needless work
iterative.

| Option | Change and price | What enforces it | Verdict |
|---|---|---|---|
| **A. Make builtin-effect classification dispatch-aware** | **Small, low semantic risk, hot path.** Replace both registered-spec `is_some()` tests with one helper that treats only `BuiltinDispatch::SpecialForm`/`Ordinary` as builtin execution. Apply the new no-effect result only to ordinary strict calls (`HoldAttribute::None`); keep held metadata heads on their current conservative route, and retain the existing Listable/Orderless/Flat, definition, upvalue, and data-constructor checks (`crates/rum-runtime/src/runtime/eval_core/stability.rs:249-346`; `crates/rum-runtime/src/runtime/metadata.rs:283-288`). On a newly constructed inert node, stability will then inspect the arguments, hit the old root's pointer proof, and cache the new root (`crates/rum-runtime/src/runtime/eval_core/stability.rs:25-33`; `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:293-321`). The added miss-path work is an enum/hold comparison and no allocation; that cost characterization is an implementation estimate grounded in the copyable `BuiltinSpec` lookup (`crates/rum-runtime/src/runtime/resolution.rs:83-95`). | Unit tests must distinguish `None` from both real dispatch variants, keep held metadata calls conservative, preserve invalidation through argument blooms, and prove an exact stable-cache entry. A release-only 2-MiB construction certificate belongs beside the existing harness (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-29,31-46`). Because this changes evaluator stability, the rum-vs-rum microbench A/B is mandatory (`docs/testing-strategy.md:74-94`). | **Do first.** It fixes all strict metadata-only heads without entering the evaluator at all. |
| **B. Share a sound shallow no-op proof for malformed Association** | **Medium, low semantic risk if deliberately narrow, cross-crate hot path.** Recognize a *trailing top-level* self-evaluating non-symbol atom as a conclusive deep-parse failure (`crates/rum-runtime/src/runtime/expr_helpers.rs:34-39`; `crates/rum-stdlib/src/associations/data.rs:234-260`). Checking only `last()` keeps the negative/valid-path preflight O(1). Use the same pure predicate in two places: stability calls `all_args_stability_with_bloom`, so completed residual nodes become cacheable; after ordinary argument preparation, the handler returns the exact input call before invoking the recursive deep parser (`crates/rum-runtime/src/runtime/eval_core/stability.rs:82-95,113-149`; `crates/rum-stdlib/src/associations/basic.rs:18-31`). Both halves are required: the cache prevents recursive evaluation of the old spine on the next iteration, while the handler preflight prevents the current resolved outer call from recursively parsing that spine. This does **not** claim that every malformed nested Association can be certified shallowly. | Unit tests must cover the trailing invalid string, an unstable argument, valid nested flattening, duplicate-key normalization, and agreement between the runtime proof and stdlib preflight; the existing distinct/duplicate stability tests are at `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:120-129`. Add the same 2-MiB release certificate and a separate A/B panel including the existing association neighbor (`scripts/core-language-microbench.sh:728-735`). | **Do second, after A.** Together, the two no-work checks change the measured Association build from cubic/deep to incremental linear work. |
| **C. Cache validated fixed-point/no-op results** | **Medium-to-large, high invalidation risk, hot path.** When `next == current`, install a proof for the exact shared call only if evaluation collected a complete dependency bloom. Equality alone is not a future-stability proof: call entries are valid only under generation, pointer, and dirty-bloom checks (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:332-358`), while the fixed-point branch currently has no such dependency product (`crates/rum-runtime/src/runtime/resolution.rs:339-366`). This could cover rejected invalid forms with real handlers, but it expands cache semantics and retention. | Mutation tests for definitions, attributes, upvalues, capabilities, suspension, and cache-size clearing; target plus all evaluation neighbors in an A/B. The cache pins head/args trees and is intentionally bounded (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:9-16,296-321`). | **Not needed for this measured class.** Open a design-only brief before choosing it. |
| **D. Bound recursive evaluation and parsing, then fall back to explicit owned stacks** | **Large, cross-cutting, highest risk, hot path.** Follow the established recursive-front depth of 16 and cold explicit-stack fallback used by `Level`, `Map`/`Apply`, and match normalization (`crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-87`; `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-61`; `crates/rum-runtime/src/runtime/resolution/match_normalization.rs:5-8,47-80,127-193`). The evaluator work stack must cover fixed-point evaluation, call dispatch stage/cursor, and argument preparation rather than only a pure tree walk; current suspension points retain exactly those states (`crates/rum-runtime/src/runtime/eval_core/call.rs:111-199`; `crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-234`). Association's independent recursive parser also needs a bounded/iterative fallback (`crates/rum-stdlib/src/associations/data.rs:234-260`). | 2-MiB deep certificates, recursion/resource-limit behavior, cancellation, input suspend/resume, frame-root tracing, parser equivalence, and a full evaluator A/B panel. Existing suspended call/argument state is the semantic boundary (`crates/rum-runtime/src/runtime/eval_core/call.rs:183-199`; `crates/rum-runtime/src/runtime/eval_call_prepare.rs:221-234`). | **Safety backstop only, not the first repair.** It removes native depth but leaves StringExpression Theta(N^2) and this Association Theta(N^3) work. It requires its own design-only brief under the plan-first rule (`docs/dev/codex/TEMPLATE.md:32-37`). |

Converting only `parse_association_item_forms_deep` to an explicit stack is also
insufficient: it removes that parser's recursive frames
(`crates/rum-stdlib/src/associations/data.rs:234-260`) but
does not remove recursive evaluator argument preparation
(`crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`) or either
asymptotic re-walk. It should not be shipped as the repair for this brief.

### 5. Draft follow-up implementation briefs

These are raw implementation-brief drafts in the required template shape; the
template says the director reviews and rewrites such drafts before dispatch
(`docs/dev/codex/TEMPLATE.md:17-37`). They intentionally split the generic
metadata fix from the Association-specific proof so each A/B changes one
variable (`docs/dev/codex/TEMPLATE.md:96-100`).

#### Draft brief-437 — dispatch-aware stability for metadata-only heads

##### Goal

Make registration metadata with `BuiltinDispatch::None` cease to count as a
builtin evaluation effect for an ordinary strict (`HoldAttribute::None`) call.
A strict no-handler head whose stable arguments make the call a fixed point must
receive the same pointer-keyed stability proof as an undefined strict head,
while held metadata heads, `SpecialForm`/`Ordinary`, attributes, definitions,
upvalues, and the data-constructor table retain their current semantics
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33,36-79,249-346`;
`crates/rum-runtime/src/runtime/metadata.rs:283-288,381-396`).

##### Context pointers

- `crates/rum-runtime/src/runtime/eval_core/stability.rs:213-346` — both
  registered-spec effect checks and the surrounding invariants.
- `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-349` — exact
  pointer key, stable-only insertion, and invalidation validation.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:42-99` — authoritative
  distinction between metadata and actual dispatch.
- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:1-23,217-258` — cache
  assertion helpers and invalidation tests.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-46` — existing
  release/2-MiB certificate harness.
- `scripts/core-language-microbench.sh:456-507` — same-tier loop, Set,
  own-value, literal-call, and definition-miss neighbors.

##### Scope

- Files MAY modify:
  `crates/rum-runtime/src/runtime/eval_core/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`,
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`, and
  `scripts/core-language-microbench.sh`; append the execution report only to
  the dispatched brief.
- Do not touch public `Expr`/`Symbol` representation, serde shapes, generated
  docs, `Cargo.lock`, or unrelated evaluator code. The shared-pointer
  representation being relied upon is defined at
  `crates/rum-core/src/expr.rs:573-708,749-775`.
- No new unconditional allocation, lock, or instrumentation in the evaluation
  path; the existing spec cache already returns a copied spec
  (`crates/rum-runtime/src/runtime/resolution.rs:83-95`).

##### Deliverable

- Introduce one private predicate for “registered builtin has executable
  dispatch” and use it at both current `is_some()` sites; permit the generic
  no-effect route only for `BuiltinDispatch::None` plus `HoldAttribute::None`,
  and do not special-case string head names
  (`crates/rum-runtime/src/runtime/eval_core/stability.rs:260-273,326-338`).
- Add focused runtime tests proving: a manually registered `None`/no-handler
  strict call is stable and has an exact cache entry; `Ordinary` and
  `SpecialForm` specs remain effectful before their handlers run; `HoldForm` or
  an equivalent no-dispatch held test spec remains conservative; a symbol write
  affecting an argument invalidates the proof; and `Rule`/Association table
  routing is unchanged. Stable-cache test helpers already expose the exact key
  assertion (`crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:7-23`).
- Add an ignored release-only 2-MiB test constructing 4,096
  `StringExpression` levels and assert `Depth` 4,097, using the existing thread
  harness (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-29`).
- Before changing evaluator source, add and freeze two exact benchmark labels:
  `construction-inert-stringexpression-depth100` with
  `Module[{s = "a"}, Do[s = StringExpression[s, "a"], {i, 100}]]`, and the same
  statement at depth 300. Both are below the supplied base completion bound of
  375 (`docs/dev/codex/brief-436-construction-depth-class.md:33-35`). Run the
  unchanged labels on base and candidate.
- A/B panel: those two targets plus `do-count-null`, `do-iterator-null`,
  `symbol-ownvalue-read`, `set-ownvalue`, `list-literal-small`,
  `downvalue-exact-miss-repeat`, and `downvalue-pattern-miss-repeat`, whose
  definitions are at `scripts/core-language-microbench.sh:456-507`. The
  candidate depth-300/depth-100 ratio must be no greater than 4.5 and must be
  materially below the base ratio; 4.5 is a conservative implementation-gate
  estimate for the expected linear threefold-size change.

##### rum / Rust pitfalls

- Compare `dispatch`, not `handler`: the planner keys stages from dispatch and
  the metadata constructors currently keep the two consistent
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:46-85`;
  `crates/rum-runtime/src/runtime/metadata.rs:381-509`). Add a debug/test
  invariant if consistency cannot otherwise be assumed; do not add a release
  hot-path branch.
- Preserve bloom accumulation by falling through to the existing argument walk;
  do not insert a dependency-free “stable” entry
  (`crates/rum-runtime/src/runtime/eval_core/stability.rs:25-33`;
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:293-321`).
- Do not alter held-call logic or suspension paths
  (`crates/rum-runtime/src/runtime/call_preparation.rs:120-167`;
  `crates/rum-runtime/src/runtime/eval_call_prepare.rs:221-234`).
- Read-only diagnosis measured `eval_core/stability.rs` at 12,056 characters,
  in the organization guide's warning range. Keep the helper within its
  cohesive stability-policy responsibility and include a split assessment in
  the report; do not opportunistically reorganize it
  (`docs/code-organization-guidelines.md:64-81`).
- Benchmark the exact frozen statements with profiling variables off; the suite
  uses in-kernel monotonic timing and per-label rows
  (`docs/testing-strategy.md:74-94`; `docs/dev/codex/TEMPLATE.md:76-100`).

##### Verification (mandatory)

Use `scripts/run-bounded.py` with bounds derived from the preceding run for all
commands that can spawn children. Run, in order:

```text
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test -p rum-stdlib --release tests::iterative_traversal_depth::string_expression_construction_reaches_physical_height_4097_on_two_mib -- --ignored --exact
cargo test --workspace
```

For timing, add the labels first, build/run the base panel, make only the
stability change, then run the identical candidate panel on an otherwise idle
machine. Use best-of-2 base and best-of-2 candidate (four suite runs total),
persist each output directory, and paste the exact target and neighbor
`timings.tsv` rows. This follows the repository's A/B contract
(`docs/testing-strategy.md:74-94`) and the suite's existing `rumMicroOnce`
min-of-reps harness (`scripts/core-language-microbench.sh:315-355`).

**Stop-loss:** reject the patch if either real dispatch variant becomes stable,
if invalidation fails, if the 2-MiB certificate fails twice for the same reason,
if depth-300 does not improve beyond base run-to-run spread, if the target-size
ratio misses the stated 4.5 ceiling, or if any named same-tier neighbor
regresses beyond its own best-of-2 spread. If the same
failure occurs twice or the target ratio does not move, stop and report; do not
tune further (`docs/dev/codex/TEMPLATE.md:96-100`).

##### Hard rules

- No git mutation, network, unsafe code, third-party oracle, or files outside
  Scope; preserve unrelated user changes. These are the template and repository
  hard lines (`docs/dev/codex/TEMPLATE.md:125-159`;
  `AGENTS.md:36-51,90-132,134-154`).
- Leave the tree uncommitted. Paste real command output and real per-label rows;
  do not claim an unrun gate (`docs/dev/codex/TEMPLATE.md:107-123,156-159`).

##### Report format

Report per-file changes, verbatim verification results, base/candidate rows and
ratios for every panel label, semantic verdict, and any stop-loss/open question
(`docs/dev/codex/TEMPLATE.md:161-170`).

#### Draft brief-438 — share a conclusive malformed-Association no-op proof

##### Goal

For an `Association` call ending in a top-level self-evaluating non-symbol atom,
prove the handler's parse failure from `last()` without recursively exploring
an earlier nested item; short-circuit that parse in the ordinary handler and,
when all arguments are stable, cache that exact unchanged call.
The measured `Association[old, "a"]` construction must become incremental while
valid entry flattening, key evaluation, canonicalization, duplicate handling,
and other malformed shapes retain current behavior
(`crates/rum-stdlib/src/associations/basic.rs:13-34`;
`crates/rum-stdlib/src/associations/data.rs:204-260`;
`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:245-260`;
`crates/rum-runtime/src/runtime/eval_core/stability.rs:113-149`).

##### Context pointers

- `crates/rum-runtime/src/runtime/eval_core/stability.rs:82-149` — all-argument
  proof and Association-specific classifier.
- `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:245-260` —
  direct-entry grammar used by runtime stability.
- `crates/rum-stdlib/src/associations/data.rs:204-260` — direct-entry and
  recursive deep-item grammar used by the ordinary handler.
- `crates/rum-stdlib/src/associations/basic.rs:13-34` — handler returns the
  original shared call when that grammar rejects.
- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:55-129` — existing
  Association proof helpers and distinct/duplicate verdict tests.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-46` — deployment
  stack harness.
- `scripts/core-language-microbench.sh:456-507,728-735` — evaluator neighbors
  and the existing small-Association neighbor.

##### Scope

- Files MAY modify:
  `crates/rum-runtime/src/runtime/eval_core/stability.rs`,
  `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`,
  `crates/rum-runtime/src/runtime/kernel_api.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`,
  `crates/rum-stdlib/src/associations/basic.rs`,
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`, and
  `scripts/core-language-microbench.sh`; append the report only to the
  dispatched brief.
- Do not rewrite the recursive Association parser, canonicalizer, the remainder
  of the handler, `Expr` representation, docs, or unrelated builtins. The only
  kernel-API expansion is the shared pure rejection predicate, and the handler
  boundary is already a shared-argument no-op on parse failure
  (`crates/rum-stdlib/src/associations/basic.rs:18-31`).
- No unsafe or default-path instrumentation. Any new check is borrowed and
  allocation-free, and the inconclusive path examines only `last()`.

##### Deliverable

- Add a narrowly named pure predicate, colocated with the runtime Association
  grammar and re-exported through `kernel_api`, which proves rejection only for
  trailing top-level self-evaluating non-symbol atoms. Do not classify symbols,
  nested failures, earlier items, or arbitrary non-entry calls without an
  equivalent proof
  (`crates/rum-runtime/src/runtime/expr_helpers.rs:34-39`;
  `crates/rum-runtime/src/runtime/kernel_api.rs:29-34`;
  `crates/rum-stdlib/src/associations/data.rs:204-260`).
- Run that preflight before the current direct-entry loop. On a conclusive
  rejection, call `all_args_stability_with_bloom`; only its true result may be
  cached (`crates/rum-runtime/src/runtime/eval_core/stability.rs:82-95,113-149`;
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:293-321`).
- In `eval_association`, apply the same preflight to `source_items` after the
  singleton-List unwrap and before `parse_association_entry_forms_deep`; return
  the existing shared input call on a conclusive rejection
  (`crates/rum-stdlib/src/associations/basic.rs:18-31`). Do not duplicate the
  predicate in the stdlib.
- Add focused tests for: `Association[Association["a", "a"], "a"]` acquiring an
  exact stable proof bottom-up; a conclusive-invalid call with an unstable
  argument remaining unstable; valid nested List/Association flattening still
  reaching the handler; duplicate keys remaining unstable before
  canonicalization; and a unary nested malformed chain remaining on the old
  conservative route. Existing duplicate assertions are at
  `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:120-129`.
- Add an ignored release-only 2-MiB test constructing 4,096 exact
  `Association[s, "a"]` levels and asserting `Depth` 4,097, using
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-29`.
- Before the source change, freeze two conservative base-safe labels:
  `construction-malformed-association-depth32` and
  `construction-malformed-association-depth96`, each timing the exact
  `Module[{s = "a"}, Do[s = Association[s, "a"], {i, DEPTH}]]` statement.
  **Estimate:** 32/96 should stay far below the supplied N=3,000 failure while
  still exposing the predicted cubic slope; if either base label fails, stop
  rather than lowering and silently redefining the panel. The only supplied
  Association bound is the N=3,000 failure at
  `docs/dev/codex/brief-436-construction-depth-class.md:13-27`.
- A/B panel: both targets plus `do-count-null`, `do-iterator-null`,
  `symbol-ownvalue-read`, `set-ownvalue`, `list-literal-small`, and
  `association-small-literal-repeat`
  (`scripts/core-language-microbench.sh:456-478,728-735`). This is an evaluator
  hot-path change, so the A/B is mandatory (`docs/testing-strategy.md:74-94`).
  The candidate depth-96/depth-32 ratio must be no greater than 4.5 and must be
  materially below the base ratio; 4.5 is a conservative implementation-gate
  estimate for the expected linear threefold-size change.

##### rum / Rust pitfalls

- “Parser will eventually fail” is insufficient; the proof must identify the
  trailing top-level item whose value cannot change and which the defining
  grammar rejects. Otherwise skipping the handler could skip key evaluation or
  canonicalization (`crates/rum-stdlib/src/associations/basic.rs:29-34`;
  `crates/rum-stdlib/src/associations/data.rs:244-260`).
- Even on conclusive parse failure, accumulate every argument's dependency bloom
  with the existing helper; a stable cache entry with an incomplete bloom is a
  semantic bug (`crates/rum-runtime/src/runtime/eval_core/stability.rs:82-95`;
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:332-358`).
- Preserve exact `SharedArgs`; do not clone the tree merely to classify it
  (`crates/rum-core/src/expr.rs:637-708`).
- Read-only diagnosis measured runtime `association_data.rs` at 32,178
  characters and stdlib `associations/basic.rs` at 28,418, both in the
  assess-on-touch split-plan range (`docs/code-organization-guidelines.md:64-81`).
  Keep this brief's edits surgical and require its report to defer or propose
  concrete future boundaries: parsing/rendering versus lvalue mutation in the
  runtime file, and construction versus lookup/query operations in the stdlib
  file. Do not add out-of-Scope split files merely to satisfy the measurement.
- This brief does not make the recursive deep parser iterative and does not
  claim to solve unary malformed chains
  (`crates/rum-stdlib/src/associations/data.rs:234-260`).

##### Verification (mandatory)

Use bounded commands and run:

```text
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test -p rum-stdlib --release tests::iterative_traversal_depth::malformed_association_construction_reaches_physical_height_4097_on_two_mib -- --ignored --exact
cargo test --workspace
```

Run best-of-2 base and candidate suites (four total) with the frozen labels and
profiling variables off, on an idle machine, and paste exact `timings.tsv` rows
for every panel label. The base for this brief is the accepted brief-437 result,
so the Association classifier is the only evaluator variable; same-label A/B
and one-variable discipline are required by
`docs/testing-strategy.md:74-94` and `docs/dev/codex/TEMPLATE.md:96-100`.

**Stop-loss:** reject if any valid/duplicate/key-evaluation outcome changes, if
an unstable dependency receives a proof, if the 2-MiB test fails twice, if the
depth-96 target does not improve beyond base spread, or if any named neighbor
regresses beyond its best-of-2 spread, or if the target-size ratio misses the
stated 4.5 ceiling. Stop after the same failure twice; do not broaden the
predicate to make a benchmark pass
(`docs/dev/codex/TEMPLATE.md:96-100`).

##### Hard rules

- No git mutation, network, unsafe, external oracle, or out-of-Scope changes;
  leave the tree uncommitted and preserve suspension behavior. The governing
  lines are `docs/dev/codex/TEMPLATE.md:125-159` and
  `AGENTS.md:36-51,90-132,134-154`.
- Paste every real test result and A/B row; report any failed/unrun gate
  explicitly (`docs/dev/codex/TEMPLATE.md:107-123,156-159`).

##### Report format

Report per-file changes, verbatim verification output, the full A/B panel and
ratios, the semantic/no-op-proof verdict, and stop-loss/open questions
(`docs/dev/codex/TEMPLATE.md:161-170`).

### 6. Source limits and probes that would settle them

The source is sufficient to distinguish the proposed mechanisms: the generic
slow heads return before argument stability recursion, and their recursion is
in evaluator argument preparation
(`crates/rum-runtime/src/runtime/eval_core/stability.rs:21-31`;
`crates/rum-runtime/src/runtime/eval_call_prepare.rs:203-220`). No runtime probe
is needed for that verdict.

What source alone cannot establish is:

- **The exact 375/550 cutoff or 45x depth ratio.** Native frame size and
  optimizer layout are build/platform facts, while the source establishes only
  linear stack and quadratic work. The supplied release/2-MiB experiment is the
  probe that settles those deployment numbers, and it is already complete
  (`docs/dev/codex/brief-436-construction-depth-class.md:6-37`); it should not be
  repeated for this diagnosis. If a non-failing count-level corroboration is
  later wanted, run the existing evaluation-cache profiler at a safely completed
  depth for identical `List` and `StringExpression` constructions and compare
  `evaluate_entries`, per-head checks/hits/misses, and insertions. The env gate
  and counters are defined at
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:29-55`, and the
  snapshot prints those fields plus per-head rows at
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/report.rs:6-41,328-358`.
  This probe is optional because it would corroborate, not separate, the source
  paths.
- **The context of `Sin` in the exact measured CLI process.** The source predicts
  ``Global`Sin`` for a clean session, but prior definitions/context state can
  alter resolution (`crates/rum-runtime/src/runtime/resolution.rs:97-153`). The
  separating probe is a context query for unevaluated `Sin` in that same clean
  CLI startup; the `Global` context confirms the `foo`-equivalent control,
  while the `System` context would require identifying extra session
  installation not present
  in `install_core` (`crates/rum-stdlib/src/lib.rs:42-150`).
- **Membership under future or host-added registrations.** `register_builtin`
  is session mutation, so no source-time name list can remain complete
  (`crates/rum-runtime/src/runtime/session/session_api.rs:265-281`). The
  separating behavioral probes for the present default catalogs are the three
  exact N=3,000 constructions predicted above: `General`, `UpTo`, and
  `XmlElement`. A failure of any to retain depth before overflow would falsify
  one of the residual/eager/no-proof premises, not merely shift a timing
  constant.
- **Whether `s`'s bloom bit collides with a newly cacheable target head in a
  particular build/session.** Bloom bits deliberately hash symbols into 128
  positions (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:18-38`),
  and a rewritten-dirty overlap suppresses otherwise-stable insertion
  (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:296-328`). The
  brief-437/438 focused cache-entry assertions settle this directly; the
  optional profiler probe above would show `rewritten_dirty_skips` if a
  collision defeated the proposed incremental path
  (`crates/rum-runtime/src/runtime/evaluation_cache/profile/report.rs:21-31`).

No Rust source file, test, probe, or benchmark was written or run for this
diagnostic; only this report was appended, as required by
`docs/dev/codex/brief-436-construction-depth-class.md:109-123`.

---

## Director gate — 2026-09-04

**Scope**: clean. `git diff --stat` = this file only, 669 insertions, no source
change, as `Scope: NONE` required.

**Claims reproduced.** All on the release `rum-cli` under `ulimit -s 2048`,
`N = 3000`, stdin from `/dev/null`.

The three offered predictions all hold, against the two calibration controls:

| Head | Predicted | Observed |
|---|---|---|
| `List` (control, fast) | completes | `DONE height=3001` |
| `StringExpression` (control, slow) | overflow | `thread 'main' has overflowed its stack` |
| `General` | overflow | overflow |
| `UpTo` | overflow | overflow |
| `XmlElement` | overflow | overflow |

The `Sin` correction is right and my original probe was invalid.
`Context[Sin]` is ``Global` `` on a clean session, while `StringExpression` and
`List` are both ``System` ``. `Sin` was never a System control; it was a second
copy of the `foo` control.

**The report's reach is half the class.** Its recommended repair A exempts
`BuiltinDispatch::None`, so it cannot touch a head that carries real dispatch.
Those heads are in the class too:

| Head | Result | Head | Result |
|---|---|---|---|
| `Power` | OVERFLOW | `Equal` | OVERFLOW |
| `Mod` | OVERFLOW | `Less` | OVERFLOW |
| `Max` | OVERFLOW | `Not` | OVERFLOW |
| `Min` | OVERFLOW | `Repeated`, `OutputForm` | OVERFLOW |
| `Missing` | fast (spine 3001) | `Complex`, `Rational`, `Optional`, `PatternTest` | fast (spine 3001) |

`Divide`, `Subtract`, `And`, `Or`, `Condition`, `Cases`, `Position`,
`StringJoin` are **invalid probes** for this workload, not fast results: they
never build a spine (final `Depth` 1 or 2). The probe reports the height so the
two are distinguishable; without that check four of them would have been
recorded as passes.

Ordinary user code does not reach the class. A `Global` head with a downvalue,
an ownvalue, an attribute, or an upvalue all complete at height 3001.

**The finding the diagnosis stopped one step short of.** The kernel already owns
the right instrument for this and it is mis-calibrated:

- `poll_native_stack_limit` (`crates/rum-runtime/src/runtime/eval_core/mod.rs:68-79`)
  measures *bytes actually consumed* from `native_stack_base`, polled every
  `NATIVE_STACK_DEPTH_POLL_STRIDE = 4` evaluation levels. That is the correct
  shape: it is stack-size-honest by construction.
- Its default is `DEFAULT_NATIVE_STACK_LIMIT = 6 * 1024 * 1024`
  (`crates/rum-runtime/src/runtime/metadata.rs:6`) — **three times the ruled
  2 MiB embedder stack**. On the ruled configuration the guard can never fire.
- `DEFAULT_RECURSION_LIMIT = 1024` (`.../metadata.rs:5`) is a depth proxy that
  cannot be stack-honest, and at the measured frame cost it corresponds to
  ≈4.8 MiB — consistent with the 6 MiB sibling, and equally unreachable.

Both defaults are internally consistent with each other and with an 8 MiB
stack. Measured:

```
default stack (8 MiB), N=3000:
  Power, StringExpression, General
      -> exit=1  "evaluation exceeded recursion limit (1024)"   [in-band, correct]
ruled stack (2 MiB), N=3000:
  same three
      -> exit=134  "thread 'main' has overflowed its stack"     [process killed]
```

**Cost per evaluation level, by bisected ceiling.** The model is linear and the
frame chain is head-independent:

| Head | stack | last ok | first fail |
|---|---|---|---|
| `Power` | 1024 KiB | 202 | 218 |
| `Power` | 2048 KiB | 421 | 437 |
| `Power` | 4096 KiB | 875 | 890 |
| `StringExpression` | 2048 KiB | 421 | 437 |

Slope `(4096-1024) KiB / (875-202) levels` = **≈4.7 KiB of native stack per
evaluation level**. `Power` and `StringExpression` have identical ceilings, so
the cost is in the shared evaluate/dispatch/prepare chain, not in any handler.
Reaching height 4,097 on 2 MiB needs ≤512 B per level, a 9.1x reduction.

**Verdict: report accepted, plan split.** The diagnosis of the mechanism, the
split predicate, the data-constructor table, and the asymptotic analysis are
correct and are what made the class findable. Its repair options are ranked as
performance work, and they are worth doing on those merits, but A + B close the
heads that were measured rather than the class — the recorded brief-388 failure
shape. Sequencing is therefore:

1. **The guard, first.** A process kill on the ruled configuration is a safety
   defect and it covers every head at once. The budget must be derived from the
   stack the evaluator actually has, not from a constant chosen for a larger
   one.
2. **The frame,** once its floor is known: 4.7 KiB per evaluation level is the
   number that decides between shrinking the chain and rewriting the evaluator
   iteratively (option D). No repair should be chosen before that is measured.
3. **A and B** as the performance repairs they are (`Theta(N^2)` and
   `Theta(N^3)` to incremental), after the depth contract is settled.

Options C and D are not taken up on this report's recommendation.
