# brief-457 — re-derive drafts C-J of the association depth class (design-only)

## Goal

brief-454 produced a ten-step repair plan for the association-side depth class.
Steps 1 and 2 shipped as briefs 455 and 456. **Every remaining verdict in that
plan is now stale in a way that changes the queue**, and this brief re-derives
them against `HEAD` = `80c8acce`.

Two things moved the ground under drafts C-J:

1. **brief-455 removed the shared envelope those drafts were priced against.**
   Read brief-454 §4's verdict column: `MapIndexed`, nested association `Set`,
   `AssociateTo`/`KeyDropFrom` on a `Part` target, `AssociationKeyDeflatten`,
   `DeleteMissing` recursive mode, `ConformSchema`, and multi-spec `GroupBy` are
   every one of them recorded as "short **through stability**" — they hit the
   roughly 1,170-logical fresh-association envelope before their own recursion
   ever bound. That envelope is gone: an association-rooted chain now takes over
   the whole subtree at the first `Association`
   (`crates/rum-runtime/src/runtime/evaluation_cache/stability/association_front.rs`,
   `crates/rum-runtime/src/runtime/eval_core/stability/association_tail.rs`), and
   `Map` over a deep association went 966 -> raw physical height 4,097. So each
   of those sites is now bound by its **own** local recursion, at a ceiling
   nobody has stated.
2. **brief-456 proved the failure direction can flip when that happens.**
   `MergeNested` went from 1,153 wrappers failing **in band** to 2,027
   **terminating the process**: the association stability arm used to exhaust
   the byte budget first, and that arm polls; once it was cheap, `MergeNested`'s
   own unpolled recursion reached the raw stack instead. brief-454 §3 answered
   "why do these fail closed at all?" for the three sites it measured. That
   answer is now wrong for at least one of them, and the same reasoning applies
   to every site in §4 whose descent was polling only by accident.

The deliverable is a re-derived, re-ordered queue with each claim marked
measured or reasoned — **not** rewritten implementation briefs. Those come in a
follow-up once the director's brackets land; drafting them now would be drafting
against projections a third time, which is the failure this brief exists to
stop.

## What is given — do not re-measure, and do not probe past a ceiling

- brief-454's measured table (`docs/dev/codex/brief-454-association-depth-class.md:19-33`)
  and its per-site release packets in §4. Those packets were measured on
  functions that briefs 455 and 456 did not touch, so they stand.
- **brief-455 outcome** (`9f8e89b2`): `Map` over a deep association and a read
  after a nested key-path assignment both complete at raw physical height 4,097
  (`N=2048` wrappers), up from 966 and 1,145. `Map` then runs to the
  construction ceiling at 5,457 wrappers.
- **brief-456 outcome** (`d0e80b1b`): `MergeNested` completes at raw 4,097 and
  runs to the construction ceiling at 5,457. Its walk is explicit enter/finish
  work items behind a bounded front, with suspension freezing the whole stack.
- **The budget**: 2 MiB thread, 1 MiB evaluator budget
  (`2 MiB - 512 KiB poll headroom - 512 KiB non-evaluator reserve`, brief-438).
  A per-level cost is only a number when it names which budget it is against
  (brief-437: 4.7 KiB release strict, 7.5 KiB release user recursion, 20.6 KiB
  debug user recursion).
- The ruled contract is `MAX_PHYSICAL_EXPR_DEPTH >= 4097` on **physical
  expression height**, not the `Depth` builtin. The canonical fixture
  `a = 1; Do[a = Association["k" -> a], {i, N}]` costs two physical levels per
  wrapper, so raw 4,097 is `N=2048`.

The director is measuring the actual brackets for these sites in parallel. Your
job is the source-side derivation that says what to expect and why, not the
bracket itself.

## Deliverables

### 1. Citation refresh

briefs 455 and 456 changed eight and six files. `MergeNested` in particular
moved out of `crates/rum-stdlib/src/associations/resumable/aggregate.rs` into a
new `aggregate/merge_nested.rs`. Walk every `path:line` citation in brief-454 §4
and §5 and report, as a table, which are still accurate at `80c8acce`, which
moved (with the new location), and which no longer exist. A stale citation in a
plan is how a repair gets briefed at the wrong layer.

### 2. Per-site poll analysis, post-455

For each site in brief-454 §4's first table — the per-nesting-level table —
answer, from source:

- **Does its descent still cross the evaluator, and where?** Name the exact
  function and line whose call reaches a frame that calls
  `poll_native_stack_limit` or advances `evaluation_depth`. brief-454 §3
  established that `crates/rum-stdlib/src/associations/` contains zero
  occurrences of either, so every fail-closed outcome in this class is borrowed
  from a frame further down.
- **Was that crossing the association stability arm?** If the site's polling
  frame was the pre-455 generic walk over a fresh association result, then
  brief-455 removed it and the site's predicted failure direction is now
  out-of-band, exactly as `MergeNested`'s was. Say so explicitly per site.
- **Predicted failure direction at `N=2048`**: in band, or terminating the
  process. Mark it reasoned; the director's probes will confirm or refute it.

This is the single most valuable thing in the brief. The line's own taxonomy
ranks fail-open above fail-closed, so a site that silently changed direction has
moved up the queue whether or not its height moved.

### 3. Restated ceiling per site, with the envelope removed

For each site, redo the arithmetic **without** the roughly 1,170-logical
stability envelope, using its already-measured release packet from §4:

| what to state | note |
|---|---|
| packet bytes per logical level | from §4, cite it |
| which budget it consumes | 1 MiB evaluator, or the 2 MiB physical stack — say which and why |
| projected levels | packet vs that budget, fixed entry overhead named as omitted |
| verdict at raw 4,097 (`N=2048`) | clears / short, and by what factor |

Then say plainly which sites **you now expect to already clear the contract**.
This is the question with precedent behind it: brief-450 probed all nine
consumers draft 5 named and found **six already clear 4,097**, so its shared
refactor across about fifteen files would have mostly rewritten working code;
brief-449 found draft 6's bundle was three different problems and one of them
was not short at all. Expect the same here and look for it.

### 4. Re-ordered queue

Reorder drafts C-J using this line's established taxonomy and ranking:

- **Ranking**: fail-open outranks fail-closed; within each, order by shortfall.
- **Taxonomy**: a local poll is *safety only*; a bounded recursive front plus an
  explicit work stack is *safety plus the 4,097 contract*; only an evaluator
  task stack lifts an *active-result ceiling*. Say which each surviving site
  needs — and note that step 1 of brief-454's plan was the evaluator task stack
  and it has shipped, so that category should now be empty or nearly so.

State explicitly, per draft: **KEEP, DROP, MERGE (into which), or SPLIT (into
what)**, with the reason. A draft whose site now clears 4,097 and fails closed
is a DROP, not a low-priority KEEP. A draft covering two sites that turn out to
have different failure directions is a SPLIT.

Include drafts C through J. Also fold in one site from outside brief-454 that
belongs to the same queue and is currently tracked separately: brief-439's
draft 3, `MapIndexed[ff, _, {0, Infinity}]`, measured 2,694 / 2,702 in band
before brief-455. It is the same function as brief-454's draft I, so say whether
they are one item or two.

### 5. Risks and open questions

Anything you could not settle from source, and anything where you think the
director's parallel measurement is likely to contradict a reasoned verdict here.
Name it rather than smoothing it over — a reasoned ceiling that turns out wrong
is useful when it was labelled reasoned.

## Context pointers

- `AGENTS.md` first, including the Testing Policy hard lines.
- `docs/dev/codex/brief-454-association-depth-class.md` — the plan being
  re-derived. §4 is the inventory, §6 the ordering, §7 the drafts.
- `TODO.md`, the section "The tail re-probed under the 1 MiB budget" and the
  paragraphs on briefs 455-456 that follow it.
- The two commits that moved the ground: `9f8e89b2` (brief-455) and `d0e80b1b`
  (brief-456). Read the association stability front and tail modules they added;
  what they take over is the crux of deliverable 2.
- The conversions already done, for the shape a KEEP would follow:
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/traversal.rs`,
  `crates/rum-stdlib/src/calculus/differentiation/iterative.rs`,
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/iterative.rs`,
  `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs`
  (the `LexicalWalk` context struct carrying the depth),
  `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs`.

## Scope

- Files you MAY modify: **NONE — design-only.** Append your report to this file
  and change nothing else.
- Do not write, redraft, or update any implementation brief. That is deliberately
  held back to a follow-up.
- **Do not run any probe that drives a recursion to or past its ceiling**, and do
  not write a reproduction whose purpose is to terminate a process. The brackets
  are the director's to measure and are being measured in parallel. Sub-ceiling
  `/proc` VmStk high-water sampling at several heights — brief-443's method, the
  one brief-454 used — is permitted and is the right instrument if you want a
  slope, because it produces one with no failure at all.
- If a claim genuinely cannot be settled without a past-ceiling measurement, say
  so in deliverable 5 and stop there. That is a correct outcome, not a shortfall.

## rum / Rust pitfalls (read twice)

1. **Mark every claim measured or reasoned**, as brief-454 §4 did. That
   convention is the reason this re-derivation is possible at all: the reasoned
   column is what went stale and the measured column held.
2. **A per-level byte cost without its budget is not a number** (brief-437).
   Say 1 MiB evaluator or 2 MiB physical, every time.
3. **Logical wrappers are not physical height.** The fixture costs two physical
   levels per wrapper. brief-454 caught itself on this; do not reintroduce it.
4. **The construction ceiling caps what any repair here can be certified to**,
   and brief-454 §5 attributes it to recursive `Expr` drop at teardown, not to
   the stability walk. It sits at 5,455-5,457 wrappers, above the 2,048 the
   contract needs. Do not propose a repair that can only be demonstrated above
   it.
5. **Wall-clock timing is not evidence on this host.** Nothing in this brief
   needs a timing number; if you find yourself wanting one, you have left scope.
6. **`Merge` and `MergeNested` are different operations** with different
   descent behavior (§4 records `Merge` as not descending association-valued
   entries). Keep them apart.

## Verification

Design-only: no test gate. The evidence is the deliverable. Every claim carries
`path:line`. Where you ran a command — a grep for poll points, a VmStk slope —
paste the verbatim output lines that support the claim.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any file except appending your report to this one. If you
  believe you must, STOP and write that in the report instead.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- Do not fabricate a citation. If you cite evidence, it must be in the file at
  the line you name — a dangling citation costs more than an omission, because
  the director has to go read the source to find out which it was.
- Paste real verbatim output. If something was not established, say so.

## Report format (your final message)

1. Design-only, no source change — state it.
2. Deliverable 1: the citation refresh table.
3. Deliverable 2: per-site poll analysis with predicted failure direction.
4. Deliverable 3: the restated ceiling table, and the list of sites you expect
   already clear the contract.
5. Deliverable 4: the re-ordered queue, with KEEP/DROP/MERGE/SPLIT per draft.
6. Deliverable 5: risks and open questions, including anything you think the
   director's brackets will contradict.

---

## Codex report — 2026-09-05

### 0. Scope and evidence labels

**Measured (workspace): design-only; no source changed.**  The only edit is
this appended report.  No test, timing run, VmStk probe, ceiling probe, or
failure reproduction was run.
`HEAD` is the documentation-only commit `7996387a`; relative to the requested
source base `80c8acce`, it adds only this brief, so the source examined is the
requested source (`docs/dev/codex/brief-457-association-tail-rederivation.md:5-8`).

I use **Measured** only for a supplied result or literal command output, and
**Reasoned** for source call-graph conclusions and packet arithmetic.  Every
quotient below omits fixed evaluator/operation entry overhead and is therefore
an optimistic ceiling, as brief-454 required
(`docs/dev/codex/brief-454-association-depth-class.md:290-296`).

The source-base check was:

```text
$ git diff --name-status 80c8acce..HEAD
A	docs/dev/codex/brief-457-association-tail-rederivation.md
```

### 1. Citation refresh

**Measured (tree diff).**  Of all source files cited in brief-454 sections 4
and 5, only the resumable aggregate implementation changed between the source
measured by brief-454 and `80c8acce`; the new `MergeNested` module was added
(`docs/dev/codex/brief-457-association-tail-rederivation.md:40-51`).  The exact
output lines from the path-restricted
`git diff --name-status a7474d94..80c8acce` were:

```text
M	crates/rum-stdlib/src/associations/resumable/aggregate.rs
A	crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs
```

**Measured (citation audit).**  The table lists every `path:line` citation in
those two sections.  Multiple ranges in one row are separate citations to the
same unchanged file.

| brief-454 section 4/5 citation(s) | status at `80c8acce` | current location / note |
|---|---|---|
| `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61,70-267`; `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:8-29,94-203`; `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-409`; `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:411-487` | **Accurate (Measured by diff).** | The bounded `Map`/`Apply` machine, recursive `MapIndexed`, and recursive `Scan` collector remain at those exact ranges. |
| `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131` | **Accurate (Measured by diff).** | `eval_scan` still collects first and then calls `evaluate_scan_parts`. |
| `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:934-1004`; `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:523-541`; `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:95-123,279-305` | **Accurate (Measured by diff).** | The key-path setter, iterative keyed read, and runtime deep association parsers remain at those ranges. |
| `crates/rum-stdlib/src/associations/mutation.rs:560-692`; `crates/rum-stdlib/src/associations/mutation.rs:268-278,401-429` | **Accurate (Measured by diff).** | Held `Part` mutation and direct-symbol mutation remain at those ranges. |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:112-126`; `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42,88-109,173-200` | **Accurate (Measured by diff).** | Key expansion and exact collect/read/replace remain at those ranges. |
| `crates/rum-stdlib/src/associations/resumable/aggregate.rs:122-218` (`MergeNested`) | **Moved and replaced; the cited recursion no longer exists (Measured by diff/source).** | Entry and bounded front are now `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:15-113`; the explicit work loop is `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:115-203`, and grouping helpers are `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:237-246`.  The old unbounded driver/helper mutual recursion was removed. |
| `crates/rum-stdlib/src/associations/resumable/aggregate.rs:8-53,224-258` (ordinary `Merge`) | **Moved within the file (Measured by diff/source).** | `eval_merge` is now `crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-56`; its sibling-group loop is `crates/rum-stdlib/src/associations/resumable/aggregate.rs:93-128`.  Lines 224-258 now belong to `CountsBy`, so the old second range must not be reused for `Merge`. |
| `crates/rum-stdlib/src/associations/key_reshape.rs:340-415`; `crates/rum-stdlib/src/associations/key_reshape.rs:417-440`; `crates/rum-stdlib/src/associations/key_reshape.rs:62-103,145-325` | **Accurate (Measured by diff).** | Deflatten, flatten, and the non-value-depth `KeyGroupBy` loops remain at those ranges. |
| `crates/rum-stdlib/src/associations/missing.rs:84-230`; `crates/rum-stdlib/src/associations/missing.rs:936-1003`; `crates/rum-stdlib/src/associations/missing.rs:355-934`; `crates/rum-stdlib/src/associations/missing.rs:1038-1053`; `crates/rum-stdlib/src/associations/missing.rs:590-620,798-817` | **Accurate (Measured by diff).** | Memory preflights, recursive mode, finite-target drivers, scan, and delayed-value evaluator calls remain there. |
| `crates/rum-stdlib/src/schema.rs:55-157`; `crates/rum-stdlib/src/schema.rs:177-301`; `crates/rum-stdlib/src/schema.rs:303-330` | **Accurate (Measured by diff).** | Public drivers, recursive association/list conformance, and leaf `MatchQ` evaluation remain there. |
| `crates/rum-stdlib/src/associations/grouping.rs:135-206`; `crates/rum-stdlib/src/associations/grouping/list.rs:257-347`; `crates/rum-stdlib/src/associations/grouping/association.rs:278-364`; `crates/rum-stdlib/src/associations/grouping/list.rs:239-255` | **Accurate (Measured by diff).** | Spec parsing, list/association nested-group recursion, and list leaf reduction remain at those ranges. |
| `crates/rum-stdlib/src/associations/transforms.rs:10-46,86-125` | **Accurate (Measured by diff).** | `KeyMap`/`KeySort` remain top-level transformations. |
| `crates/rum-stdlib/src/associations/resumable/map.rs:52-199`; `crates/rum-stdlib/src/associations/key_reshape.rs:62-103,145-325`; `crates/rum-stdlib/src/associations/aggregate.rs:162-493`; `crates/rum-stdlib/src/associations/join_across.rs:213-337` | **Accurate (Measured by diff).** | The cited sibling/index loops still do not descend canonical association-value depth. |
| `crates/rum-stdlib/src/associations/data.rs:234-260`; `crates/rum-stdlib/src/associations/data.rs:163-189` | **Accurate (Measured by diff).** | The stdlib deep association parser and ordinary-`Merge` collection parser remain there. |
| `crates/rum-stdlib/src/associations/basic.rs:752-810`; `crates/rum-stdlib/src/associations/basic.rs:818-915`; `crates/rum-stdlib/src/associations/basic.rs:699-721`; `crates/rum-stdlib/src/associations/basic.rs:549-560` | **Accurate (Measured by diff).** | Nested-list `Keys`/`Values`, `KeyTake`/`KeyDrop`, `Lookup` planning, and `Normal` recursion remain there. |
| `crates/rum-stdlib/src/associations/transforms/key_collections.rs:488-517` | **Accurate (Measured by diff).** | Nested option-list collection remains there. |
| `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:25-66,69-100` | **Accurate (Measured by diff).** | The `RenderTask` implementation remains there. |
| `crates/rum-stdlib/src/structure_ops/mod.rs:115-140` | **Accurate (Measured by diff).** | Association-aware `Depth` remains there. |
| `crates/rum-core/src/expr.rs:573-775` | **Location accurate (Measured by diff); section 5's inference is disputed.** | The `Expr`/shared representation is still there.  The later director gate explicitly refutes the report's teardown attribution (`docs/dev/codex/brief-454-association-depth-class.md:1178-1205`); see risk 5 below. |
| `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:540-568` | **Accurate (Measured by diff).** | The iterative cleanup versus natural-drop test distinction remains there; it does not by itself establish which phase caused the supplied construction bracket. |

**Measured (diff/source).**  No other section 4/5 citation disappeared.  The
only source claim that no longer exists is the old `MergeNested` unbounded
mutual recursion; ordinary `Merge` remains a separate operation and still does
not descend association-valued entries
(`crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-56,93-128`).

### 2. Per-site poll analysis after brief-455

#### Shared downstream machinery

**Reasoned (source).**  An ordinary returned result reaches the next fixed-point
stability check at
`crates/rum-runtime/src/runtime/resolution.rs:305-326`.  On a generic call-cache
miss, `call_stability_with_bloom` polls the byte budget every 16 misses at
`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:266-311` and
`crates/rum-runtime/src/runtime/metadata.rs:19-19`.  Once the
generic data-constructor arm sees `Association`, it enters a depth-16
association-rooted front at
`crates/rum-runtime/src/runtime/eval_core/stability.rs:6-6,229-275`; exhausted front
depth hands the entire remaining call subtree to the explicit
`AssociationStabilityTask` loop at
`crates/rum-runtime/src/runtime/evaluation_cache/stability/association_front.rs:78-99`
and
`crates/rum-runtime/src/runtime/eval_core/stability/association_tail.rs:72-99,121-132,194-299`.
That is the removed 1,170-logical native envelope.

**Reasoned (source).**  A real nested `Session::evaluate` reaches
`evaluate_profiled_fixed_point` at
`crates/rum-runtime/src/runtime/resolution.rs:288-303`; an unstable evaluation
checks `poll_native_stack_limit`, increments `evaluation_depth`, and restores it
afterward at `crates/rum-runtime/src/runtime/eval_core/mod.rs:14-53`.  A call to
`Session::evaluate` while operation frames are live is therefore a possible
poll opportunity, but brief-456 proves that a leaf-only opportunity is not a
bound: `MergeNested` still reached the physical stack first
(`docs/dev/codex/brief-457-association-tail-rederivation.md:25-32`).

**Measured (search).**  The exact poll/handoff search output was:

```text
crates/rum-runtime/src/runtime/eval_core/stability/association_tail.rs:72:    pub(in crate::runtime) fn association_stability_tail_with_bloom(
crates/rum-runtime/src/runtime/eval_core/mod.rs:42:            self.poll_native_stack_limit()?;
crates/rum-runtime/src/runtime/eval_core/mod.rs:51:        self.evaluation_depth = evaluation_depth + 1;
crates/rum-runtime/src/runtime/eval_core/mod.rs:68:    fn poll_native_stack_limit(&mut self) -> Result<(), EvalError> {
crates/rum-runtime/src/runtime/eval_core/mod.rs:82:    pub(in crate::runtime) fn native_stack_budget_exhausted(&self) -> bool {
crates/rum-runtime/src/runtime/evaluation_cache/stability/association_front.rs:78:    pub(in crate::runtime) fn association_rooted_cached_expr_stability_with_bloom(
crates/rum-runtime/src/runtime/evaluation_cache/stability/association_front.rs:84:            return self.association_stability_tail_with_bloom(expr.clone());
crates/rum-runtime/src/runtime/evaluation_cache/stability/association_front.rs:152:            && self.native_stack_budget_exhausted()
crates/rum-runtime/src/runtime/eval_core/stability.rs:47:            self.association_rooted_cached_expr_stability_with_bloom(head, remaining_depth);
crates/rum-runtime/src/runtime/eval_core/stability.rs:69:                self.association_rooted_cached_expr_stability_with_bloom(arg, remaining_depth);
crates/rum-runtime/src/runtime/eval_core/stability.rs:120:                self.association_data_stability_with_bloom_association_rooted(
crates/rum-runtime/src/runtime/eval_core/stability.rs:180:                self.association_rooted_cached_expr_stability_with_bloom(arg, remaining_depth);
crates/rum-runtime/src/runtime/eval_core/stability.rs:235:        self.association_data_stability_with_bloom_association_rooted(
crates/rum-runtime/src/runtime/eval_core/stability.rs:243:    fn association_data_stability_with_bloom_association_rooted(
crates/rum-runtime/src/runtime/eval_core/stability.rs:274:                self.association_rooted_cached_expr_stability_with_bloom(arg, remaining_depth);
crates/rum-runtime/src/runtime/eval_core/stability.rs:314:            self.association_rooted_cached_expr_stability_with_bloom(lhs, remaining_depth);
crates/rum-runtime/src/runtime/eval_core/stability.rs:320:            self.association_rooted_cached_expr_stability_with_bloom(rhs, remaining_depth);
crates/rum-runtime/src/runtime/eval_core/stability.rs:350:            self.association_rooted_cached_expr_stability_with_bloom(lhs, remaining_depth);
crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:304:            && self.native_stack_budget_exhausted()
```

**Measured (search).**  The requested operation directories still contain no
local byte poll or evaluation-depth update:

```text
$ rg -n 'poll_native_stack_limit|native_stack_budget_exhausted|evaluation_depth' crates/rum-stdlib/src/associations crates/rum-stdlib/src/schema.rs crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs crates/rum-runtime/src/runtime/kernel_data/association_data.rs crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs --stats

0 matches
0 matched lines
0 files contained matches
34 files searched
0 bytes printed
443172 bytes searched
0.000441 seconds spent searching
0.002315 seconds total
```

**Measured (search).**  The evaluator crossings relevant to the recursive
sites are, verbatim:

```text
crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:452:        let result = session.evaluate(Expr::call(operator.clone(), vec![part]));
crates/rum-stdlib/src/associations/missing.rs:592:            session.evaluate(entry.value.clone())
crates/rum-stdlib/src/associations/missing.rs:800:            session.evaluate(entry.value.clone())
crates/rum-stdlib/src/associations/grouping/association.rs:56:        let key = match session.evaluate(Expr::call(
crates/rum-stdlib/src/associations/grouping/association.rs:81:                    .evaluate(Expr::call(value_function, vec![entry.value.clone()]))
crates/rum-stdlib/src/associations/grouping/association.rs:137:        let key = match session.evaluate(Expr::call(
crates/rum-stdlib/src/associations/grouping/association.rs:157:                    .evaluate(Expr::call(value_function, vec![entry.value.clone()]))
crates/rum-stdlib/src/associations/grouping/association.rs:204:    let value = match session.evaluate(Expr::call(value_function, vec![entry.value.clone()])) {
crates/rum-stdlib/src/associations/grouping/association.rs:275:        Some(reducer) => session.evaluate(Expr::call(reducer, vec![group])),
crates/rum-stdlib/src/associations/grouping/association.rs:456:        let value_result = session.evaluate(Expr::call(reducer.clone(), vec![group]));
crates/rum-stdlib/src/associations/grouping/association.rs:494:        let value_result = session.evaluate(Expr::call(reducer.clone(), vec![group]));
crates/rum-stdlib/src/associations/grouping/list.rs:56:        let key = match session.evaluate(Expr::call(spec.key_function.clone(), vec![item.clone()]))
crates/rum-stdlib/src/associations/grouping/list.rs:78:                match session.evaluate(Expr::call(value_function, vec![item])) {
crates/rum-stdlib/src/associations/grouping/list.rs:132:        let key = match session.evaluate(Expr::call(spec.key_function.clone(), vec![item.clone()]))
crates/rum-stdlib/src/associations/grouping/list.rs:149:                match session.evaluate(Expr::call(value_function, vec![item])) {
crates/rum-stdlib/src/associations/grouping/list.rs:193:    let value = match session.evaluate(Expr::call(value_function, vec![item])) {
crates/rum-stdlib/src/associations/grouping/list.rs:253:        Some(reducer) => session.evaluate(Expr::call(reducer, vec![group])),
crates/rum-stdlib/src/associations/grouping/list.rs:438:        let value_result = session.evaluate(Expr::call(
crates/rum-stdlib/src/associations/grouping/list.rs:478:        let value_result = session.evaluate(Expr::call(
crates/rum-stdlib/src/schema.rs:65:        let value_result = session.evaluate(args_slice[0].clone());
crates/rum-stdlib/src/schema.rs:116:        let value_result = session.evaluate(args_slice[0].clone());
crates/rum-stdlib/src/schema.rs:161:                .session
crates/rum-stdlib/src/schema.rs:246:                        self.session.evaluate(default).map_err(SchemaAbort::Eval)?;
crates/rum-stdlib/src/schema.rs:324:            .session
crates/rum-stdlib/src/schema.rs:325:            .evaluate(Expr::call(
```

#### Per-site verdicts

“Completes” below is deliberately neither “in band” nor “terminates”: the
source arithmetic or supplied measurement says the operation reaches the
`N=2048` contract point.

| site from brief-454 section 4's first table | evaluator crossing in the depth-growing phase | was the old binding the association stability arm? | predicted result at association `N=2048` |
|---|---|---|---|
| `Map` | **Reasoned:** no operation-local crossing.  `map_parts_with_mode` uses a depth-16 front/explicit fallback and returns before the outer fixed-point check (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-6,40-61,94-267`; `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:77-92`). | **Reasoned: yes.**  The fresh `ff[Association[...]]` result now transfers at its first `Association` to the bounded association front/tail cited above. | **Measured: completes**, and continues to the 5,457-wrapper construction ceiling (`docs/dev/codex/brief-457-association-tail-rederivation.md:45-48`). |
| `Apply` | **Reasoned:** no local crossing; it shares the bounded transform and returns its result (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-29,94-203`; `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:134-147`). | **Reasoned: conditional.**  Preserved association ancestry enters the new tail; a result whose association heads were all replaced is a generic call chain and uses the 192 B generic stability packet instead (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:266-311`). | **Reasoned: completes.** |
| association `MapIndexed` | **Reasoned:** `map_indexed_parts_recursive` has no `Session` and returns before the outer check (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-409`; `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:59-74`). | **Reasoned: yes.**  It rebuilds every association and the first rebuilt `Association` transfers the remaining `ff`/`Rule` subtree to the new tail (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:329-358`). | **Reasoned: completes** for the association fixture.  The generic standard-call shape has a different local packet; its supplied active-result shortfall is carried into section 4 below. |
| `Scan` | **Reasoned:** the collector itself does not cross.  Only after it returns does `evaluate_scan_parts` evaluate the flat vector sequentially (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:411-487`; `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131`; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:446-462`). | **Reasoned: no.**  It returns `Null`, not a deep fresh association. | **Reasoned: completes**; brief-439 also measured the generic 4,097 case passing (`docs/dev/codex/brief-439-deep-rebuild-class-design.md:227-231`). |
| nested association `Set` | **Reasoned:** selector evaluation finishes before `set_association_key_path`; the recursive rebuild contains no evaluator call and the mutation returns the scalar RHS (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:771-807,842-878,934-1004`). | **Reasoned: yes, but only in the certificate's later read.**  The rebuilt symbol read used to enter the recursive association stability walk after the setter had unwound. | **Measured: completes**, including the post-write read (`docs/dev/codex/brief-457-association-tail-rederivation.md:45-48`). |
| `AssociateTo` / `KeyDropFrom` on a `Part` target | **Reasoned:** target/selector evaluation is completed before `finish_held_association_part_mutation`; exact collect/read/rebuild are pure recursive helpers (`crates/rum-stdlib/src/associations/mutation.rs:215-244,522-560,588-692`; `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42,88-109,173-200`). | **Reasoned: yes** for the returned or subsequently read fresh target; that check occurs after all exact-path frames have unwound. | **Reasoned: completes.** |
| keyed `Part` read | **Reasoned:** there is no depth-growing call chain; the key loop advances iteratively (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:523-541`). | **Reasoned: no operation-created fresh ancestry.** | **Measured: completes** to the supplied construction bracket (`docs/dev/codex/brief-454-association-depth-class.md:28-35`). |
| `MergeNested` | **Reasoned:** the retained native front is capped at 16; a leaf calls `Session::evaluate` at `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:109-112`, while the tail leaf does so with heap work at `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:165-181` (the depth constant and complete front/tail are `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:3-3,48-194`). | **Measured: it was**, and removing that old binding exposed a process termination before brief-456 converted the local walk (`docs/dev/codex/brief-457-association-tail-rederivation.md:25-32`). | **Measured: completes** to the construction ceiling after brief-456 (`docs/dev/codex/brief-457-association-tail-rederivation.md:49-51`). |
| ordinary `Merge` | **Reasoned:** it parses once and loops sibling groups; combiner evaluation at `crates/rum-stdlib/src/associations/resumable/aggregate.rs:105-108` is not nested by association-value depth (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-56,93-128`). | **Reasoned: no.**  Deep values are reused, not descended. | **Reasoned: completes / no per-association-depth site.**  Its nested-list parser is accounted for separately under draft J. |
| `AssociationKeyDeflatten` | **Reasoned:** `insert_path` and `render_nested_path_entries` recurse without a `Session`; `eval_association_key_deflatten` merely returns the result (`crates/rum-stdlib/src/associations/key_reshape.rs:105-123,340-415`). | **Reasoned: yes.**  The fresh deep association result used to meet the shared envelope only after both local phases returned. | **Reasoned: completes.** |
| `AssociationKeyFlatten` | **Reasoned:** `flatten_association_entries` recursively visits values without an evaluator call (`crates/rum-stdlib/src/associations/key_reshape.rs:126-142,417-440`). | **Reasoned: no proportional result crossing.**  Its result is flat. | **Reasoned: completes.** |
| `DeleteMissing` | **Reasoned:** recursive mode has no evaluator (`crates/rum-stdlib/src/associations/missing.rs:308-328,936-1003`).  Finite-target mode descends at `crates/rum-stdlib/src/associations/missing.rs:356-723`; only a delayed entry at the selected leaf calls `Session::evaluate` at `crates/rum-stdlib/src/associations/missing.rs:590-620` (and `crates/rum-stdlib/src/associations/missing.rs:798-817` after suspension).  Preflights and `contains_missing_within` are also pure recursion (`crates/rum-stdlib/src/associations/missing.rs:84-230,1038-1053`). | **Reasoned: yes** for recursive mode's fresh deep result.  The finite-target delayed leaf is only a single opportunity and is not a reliable local bound, by the `MergeNested` precedent. | **Reasoned: completes** for the priced immediate association paths.  Armed-memory and scan qualifications are in risk 3. |
| `ConformSchema` / `SchemaErrors` | **Reasoned:** their initial value evaluations occur before validation (`crates/rum-stdlib/src/schema.rs:55-75,106-126`).  `conform_value` then recurses at `crates/rum-stdlib/src/schema.rs:177-301`; only the deepest leaf calls `Session::evaluate(MatchQ[...])` at `crates/rum-stdlib/src/schema.rs:322-330`. | **Reasoned:** `ConformSchema`'s returned association used the old arm; `SchemaErrors` discards the conformed tree and never depended on that post-result arm.  The one deepest-leaf evaluator call does not bound either local walk. | **Reasoned: terminates the process.**  The 2 MiB physical-stack packet quotient is already below 2,048 pairs before fixed overhead. |
| multi-spec `GroupBy` (list and association inputs) | **Reasoned:** each spec level evaluates a key at `crates/rum-stdlib/src/associations/grouping/list.rs:55-73` or `crates/rum-stdlib/src/associations/grouping/association.rs:55-75`; the next-spec recursion is `crates/rum-stdlib/src/associations/grouping/list.rs:257-347` or `crates/rum-stdlib/src/associations/grouping/association.rs:279-364`.  Those short nested evaluations restore `evaluation_depth` before the next spec (`crates/rum-runtime/src/runtime/eval_core/mod.rs:19-53`); there is no operation-local poll. | **Reasoned: yes** for the prior fresh nested association result.  The per-spec callback is an opportunity, not a source-enforced bound; exactly that inference failed for pre-conversion `MergeNested`. | **Reasoned: terminates the process** on a shallow callback/reducer probe before `N=2048`; callback shape can change this direction (risk 1). |
| `KeyMap` / `KeySort` | **Reasoned:** only top-level keys/entries are transformed or sorted; values are retained (`crates/rum-stdlib/src/associations/transforms.rs:10-46,86-125`). | **Reasoned: no proportional crossing.** | **Reasoned: completes / no per-association-depth site.** |
| `AssociationMap`, `KeyGroupBy`, `Counts`, `Catenate`, key-set operations, `JoinAcross` | **Reasoned:** the cited drivers advance sibling indices and retain resumable state, rather than recursing through association values (`crates/rum-stdlib/src/associations/resumable/map.rs:52-199`; `crates/rum-stdlib/src/associations/key_reshape.rs:62-103,145-325`; `crates/rum-stdlib/src/associations/aggregate.rs:162-493`; `crates/rum-stdlib/src/associations/join_across.rs:213-337`). | **Reasoned: no operation-depth crossing.**  Callback-produced deep values are a downstream result issue. | **Reasoned: completes / no per-association-depth site.** |

### 3. Restated ceilings without the old association envelope

**Given/Reasoned.**  The budgets are **1,048,576 B** only where evaluation
depth itself is the resident recursion, and **2,097,152 B** for an unpolled
operation-local native walk
(`docs/dev/codex/brief-457-association-tail-rederivation.md:52-60`).  Merely
calling the evaluator once at a leaf does not change an operation-local packet
into a 1 MiB packet; brief-456 is the measured counterexample
(`docs/dev/codex/brief-457-association-tail-rederivation.md:25-32`).  All
projections below omit fixed entry overhead.

| site | packet retained per depth level | budget and projected ceiling | verdict at raw physical height 4,097 |
|---|---:|---|---|
| `Map` | **Measured old:** 1,075.5 B/logical level, but that was the now-replaced post-result stability packet, not the bounded transformer (`docs/dev/codex/brief-454-association-depth-class.md:302-302`). | **Measured current:** no proportional operation-native packet after the 16-level front; it reaches the 5,457-wrapper construction ceiling (`docs/dev/codex/brief-457-association-tail-rederivation.md:45-48`). | **Measured clear**, at least `5457 / 2048 = 2.66x` wrapper headroom before the fixture ceiling. |
| `Apply` | **Measured/reused:** 192 B per generic stable output call; association-containing results use the explicit association tail (`docs/dev/codex/brief-454-association-depth-class.md:303-303`; `crates/rum-runtime/src/runtime/eval_core/stability/association_tail.rs:72-99`). | **Reasoned:** 1 MiB generic-stability budget / 192 B = about **5,461 ordinary calls**. | **Reasoned clear** for raw height 4,097 by about **1.33x**; preserved associations are bounded rather than packet-limited. |
| association `MapIndexed` | **Measured packet:** **560 B/logical association level** (`docs/dev/codex/brief-454-association-depth-class.md:304-304`). | **Reasoned:** unpolled local recursion consumes the 2 MiB physical stack: `2,097,152 / 560 = 3,745` logical levels. | **Reasoned clear by 1.83x** (`3,745 / 2,048`). |
| generic-call `MapIndexed` (brief-439 draft 3) | **Open locally / Reasoned downstream:** the 560 B packet was measured on the association branch, which calls `map_indexed_parts_recursive` directly; a generic call routes through the additional `map_indexed_parts_on_standard_call` frame, so 560 B cannot price its local walk (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:329-365,369-408`; `docs/dev/codex/brief-454-association-depth-class.md:304-304`).  Downstream, all-level selection puts a rebuilt generic call and an `ff[...]` wrapper on each deep-path level, for **2 × 192 = 384 B/input level** in generic stability (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:387-405`; `docs/dev/codex/brief-446-stability-poll-placement.md:115-120`). | **Reasoned local budget:** its unpolled walker consumes the 2 MiB physical stack, but no current generic packet is supplied, so there is no defensible local quotient.  **Reasoned/Measured downstream:** 1 MiB evaluator budget / 384 B projects about **2,730 input levels**, with fixed entry overhead omitted; the supplied bracket is 2,694/2,702 in band (`docs/dev/codex/brief-457-association-tail-rederivation.md:134-138`; `docs/dev/codex/brief-439-deep-rebuild-class-design.md:227-227`). | **Measured downstream shortfall: 1.52x** (`4,097 / 2,694`) and fail-closed.  **Local stable-result verdict: Open**—source proves the unpolled direction, not whether its 2 MiB physical ceiling is below 4,097. |
| `Scan` | **Measured packet:** **336 B/logical association level** (`docs/dev/codex/brief-454-association-depth-class.md:305-305`). | **Reasoned:** 2 MiB physical / 336 B = about **6,242 logical levels**. | **Reasoned clear by 3.05x**; its callbacks run only after collection. |
| nested association `Set` | **Measured/static packet:** **360.9/368 B per logical level** (`docs/dev/codex/brief-454-association-depth-class.md:306-306`). | **Reasoned:** 2 MiB physical / 368 B = about **5,699 logical levels**. | **Measured clear** with the later read; local arithmetic is **2.78x** over 2,048 (`docs/dev/codex/brief-457-association-tail-rederivation.md:45-48`). |
| held `AssociateTo` / `KeyDropFrom` exact paths | **Measured packets:** **336 B/selector** collect, **96 B/raw component** read, **256 B/raw component** replace; phases are separate (`docs/dev/codex/brief-454-association-depth-class.md:307-307`). | **Reasoned, 2 MiB physical:** about **6,242 selectors**, **21,845 read components**, or **8,192 replace components**.  A key contributes two raw components (`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:112-126`). | **Reasoned clear.**  Required is 2,048 selectors / 4,096 raw components; the binding replacement phase has **2.00x** headroom. |
| keyed `Part` read | **Measured:** no operation packet; the key traversal is iterative (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:523-541`). | **Measured:** construction/fixture ceiling 5,455/5,463 logical wrappers (`docs/dev/codex/brief-454-association-depth-class.md:28-35`). | **Measured clear**, about **2.66x** wrapper headroom. |
| `MergeNested` | **Measured old packet:** **1,007.2/1,008 B/logical level**, now removed from the tail (`docs/dev/codex/brief-454-association-depth-class.md:309-309`). | **Measured current:** bounded 16-level front plus explicit work stack (`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:3-13,48-194`). | **Measured clear** to the 5,457-wrapper fixture ceiling (`docs/dev/codex/brief-457-association-tail-rederivation.md:49-51`). |
| ordinary `Merge` | **Reasoned:** no packet per association-value level (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-56,93-128`). | No association-value quotient. | **Reasoned clear**; keep its collection parser separate. |
| `AssociationKeyDeflatten` | **Measured packets:** **224 B/key** insertion, then **256 B/rendered branch**, in separate phases (`docs/dev/codex/brief-454-association-depth-class.md:311-311`). | **Reasoned, 2 MiB physical:** about **9,362 keys** or **8,192 rendered branches**. | **Reasoned clear by at least 4.00x**. |
| `AssociationKeyFlatten` | **Measured packet:** **496 B/logical association level** (`docs/dev/codex/brief-454-association-depth-class.md:312-312`). | **Reasoned:** 2 MiB physical / 496 B = about **4,228 logical levels**. | **Reasoned clear by 2.06x**. |
| `DeleteMissing`, recursive mode | **Measured packet:** **160 B/logical association level** (`docs/dev/codex/brief-454-association-depth-class.md:313-313`). | **Reasoned:** unpolled 2 MiB physical / 160 B = about **13,107 logical levels**. | **Reasoned clear by 6.40x** once fresh-result association stability is removed. |
| `DeleteMissing`, finite-target association driver | **Measured packet:** **816 B/logical association level** (`docs/dev/codex/brief-454-association-depth-class.md:313-313`). | **Reasoned:** the immediate path has no reliable per-level evaluator poll, so 2 MiB physical / 816 B = about **2,570 logical levels**. | **Reasoned clear by 1.25x**.  This is the thinnest projected passing margin in drafts D, F, G, H, and J. |
| `ConformSchema` / `SchemaErrors` | **Measured packet:** **1,056 B/nested value/schema pair** (`docs/dev/codex/brief-454-association-depth-class.md:314-314`). | **Reasoned:** the evaluator call is leaf-only, so 2 MiB physical / 1,056 B = about **1,986 pairs**. | **Reasoned short by 1.031x**; 2,048 pairs retain **2.0625 MiB** before fixed overhead, so process termination is predicted. |
| multi-spec `GroupBy` | **Measured packet:** roughly **1.9-2.0 KiB/spec** (`docs/dev/codex/brief-454-association-depth-class.md:315-315`). | **Reasoned:** no source-enforced local poll, so the 2 MiB physical quotient is about **1,024-1,078 specs**. | **Reasoned short by about 1.90-2.00x**, with process termination predicted for shallow callbacks. |
| `KeyMap` / `KeySort` | **Reasoned:** no packet per association-value level (`crates/rum-stdlib/src/associations/transforms.rs:10-46,86-125`). | No quotient. | **Reasoned clear.** |
| other sibling/index association operations | **Reasoned:** no packet per association-value level in the cited loops (`crates/rum-stdlib/src/associations/resumable/map.rs:52-199`; `crates/rum-stdlib/src/associations/aggregate.rs:162-493`; `crates/rum-stdlib/src/associations/join_across.rs:213-337`). | No quotient. | **Reasoned clear.** |
| draft J stdlib/runtime deep association parsers | **Measured packet:** **272 B/nested collection container** (`docs/dev/codex/brief-454-association-depth-class.md:328-328`). | **Reasoned:** 2 MiB physical / 272 B = about **7,710 containers**. | **Reasoned clear for raw 4,097 by 1.88x**.  This is a one-physical-level-per-container shape, not the `N=2048` rule-value fixture. |
| draft J ordinary-`Merge` collection parser | **Measured packet:** **128 B/nested list layer** (`docs/dev/codex/brief-454-association-depth-class.md:329-329`). | **Reasoned:** 2 MiB physical / 128 B = **16,384 list layers**. | **Reasoned clear for raw 4,097 by 4.00x**. |

**Expected already to clear the ruled contract:** `Map`, `Apply`, association
`MapIndexed`, `Scan`, nested association `Set`, held-part `AssociateTo` and
`KeyDropFrom`, keyed `Part`, `MergeNested`, ordinary `Merge`, both key-reshape
operations, the priced `DeleteMissing` association modes, `KeyMap`, `KeySort`,
the non-descending association operations, and all three draft-J parser copies.
This list is **Measured** for `Map`, post-assignment read, keyed `Part`, and
`MergeNested`, and **Reasoned** for the rest from the preceding cited rows.

**Expected still short:** multi-spec `GroupBy` and both schema entry points by
**Reasoned** local ceilings, plus the generic all-level `MapIndexed`
post-result path by its **Measured** 2,694/2,702 bracket
(`docs/dev/codex/brief-457-association-tail-rederivation.md:134-138`).  The
generic walker's own stable-result ceiling is **Open** because the supplied
association packet does not price its additional standard-call helper.

### 4. Re-ordered queue and draft dispositions

**Given/Reasoned.**  The queue applies fail-open before fail-closed, then larger
shortfall before smaller shortfall, exactly as required
(`docs/dev/codex/brief-457-association-tail-rederivation.md:118-132`).  A local
poll would make an operation safe but cannot make work complete; a bounded
front plus explicit work stack supplies local safety and the height contract;
only an evaluator task stack changes an active-result ceiling
(`docs/dev/codex/brief-454-association-depth-class.md:439-445`).

#### Surviving work, in order

1. **Draft E — KEEP (Reasoned), first: multi-spec `GroupBy`.**  Its unpolled
   local packet projects only 1,024-1,078 specs on the 2 MiB physical stack,
   about 1.90-2.00x short.  List and association variants have the same nested
   driver shape and the same shallow per-spec evaluator opportunity
   (`crates/rum-stdlib/src/associations/grouping/list.rs:257-347`;
   `crates/rum-stdlib/src/associations/grouping/association.rs:279-364`).  It
   needs a bounded recursive front plus suspension-safe explicit work state; a
   poll is safety only and would not reach 2,048 specs.
2. **Draft C — KEEP (Reasoned), second: `ConformSchema`/`SchemaErrors`.**  Both
   share `SchemaValidator::conform_value`; the 2 MiB physical quotient is about
   1,986 pairs, 1.031x short, and the only nested evaluator call is at the leaf
   (`crates/rum-stdlib/src/schema.rs:177-330`).  They need one bounded front plus
   an explicit, suspension-safe visit/finish stack.  A local poll would only
   make the predicted process termination recoverable below the contract.
3. **brief-439 draft 3's generic post-result concern — SPLIT from the local
   walker and KEEP (Measured/Reasoned), third.**  The all-level
   `MapIndexed[ff, _, {0, Infinity}]` result remains the one clear exception to
   “the evaluator-task category is empty”: its 2,694/2,702 in-band bracket is
   **Measured**, a 1.52x shortfall
   (`docs/dev/codex/brief-457-association-tail-rederivation.md:134-138`).  If
   that generic call-spine result contains no `Association`, the
   association-only task stack does not take it over
   (`crates/rum-runtime/src/runtime/eval_core/stability.rs:90-103,229-240`).
   Only an evaluator-owned task-stack extension on the generic post-result path
   can lift that ceiling; converting `map_indexed_parts_recursive` cannot.  For
   a stable `ff` this is the generic stability path; an actually active `ff`
   would retain the separate whole-evaluator limit.  The local generic walk is
   unpolled, but its current 2 MiB physical-stack packet/ceiling is **Open**, so
   it has no evidence-based insertion point in this queue.

**Reasoned.**  The `MapIndexed` halves of draft I and brief-439 draft 3 name
**one source walker, not two**
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-487`), but the
surviving measured defect is not in that walker.  The correct split is one
dropped association-local/`Scan` draft, one kept generic evaluator item, and
one explicitly open generic local ceiling, rather than a merged walker
implementation that claims to lift the post-result ceiling.

#### Per-draft disposition, including drops

| draft | disposition | reason |
|---|---|---|
| **C — schema validator** | **KEEP (Reasoned).** | Both entry points are locally fail-open and about 1.031x short on the 2 MiB physical stack (`crates/rum-stdlib/src/schema.rs:177-330`; `docs/dev/codex/brief-454-association-depth-class.md:314-314`). |
| **D — all-mode `DeleteMissing`** | **DROP as drafted (Reasoned).** | Removing the result envelope leaves the recursive association mode at 6.40x headroom and the finite-target association driver at 1.25x.  The broad implementation draft would rewrite passing modes; unpriced armed-memory/scan routes stay an explicit measurement question, not an implementation projection (`crates/rum-stdlib/src/associations/missing.rs:84-230,308-328,936-1053`). |
| **E — multi-spec `GroupBy`** | **KEEP (Reasoned).** | About 1.90-2.00x short and predicted fail-open for shallow callbacks; it needs explicit suspension-safe work state (`crates/rum-stdlib/src/associations/grouping/list.rs:257-347`; `crates/rum-stdlib/src/associations/grouping/association.rs:279-364`). |
| **F — association key-path assignment** | **DROP (Measured/Reasoned).** | The post-write exact read now completes at `N=2048` **Measured**; the local 368 B packet has 2.78x reasoned headroom and returns the scalar before any read (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:842-919,976-1004`; `docs/dev/codex/brief-457-association-tail-rederivation.md:45-48`). |
| **G — held-mutation exact paths** | **DROP (Reasoned).** | The binding replacement phase retains 1 MiB at `N=2048` but consumes the 2 MiB physical stack, leaving 2.00x headroom; all evaluator work precedes the pure exact-path phases (`crates/rum-stdlib/src/associations/mutation.rs:522-692`; `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42,88-126,173-200`).  This is the same “do not refactor already-clearing consumers” boundary learned in brief-450 (`docs/dev/codex/brief-457-association-tail-rederivation.md:111-116`). |
| **H — key reshape** | **DROP (Reasoned).** | Deflatten has at least 4.00x headroom and flatten 2.06x after result stability is removed; neither recursive helper crosses the evaluator (`crates/rum-stdlib/src/associations/key_reshape.rs:105-142,340-440`). |
| **I — `MapIndexed` and `Scan`** | **SPLIT, then DROP both local halves (Reasoned).** | Association `MapIndexed` has 1.83x projected headroom and `Scan` 3.05x; the latter's callbacks are sequential after collection.  Do not merge either passing association target into brief-439's post-result concern (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-487`; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:446-462`). |
| **J — collection parser tails** | **DROP (Reasoned).** | The 272 B copies project 7,710 raw container levels and the 128 B `Merge` parser 16,384, both above raw 4,097.  No below-contract shortfall remains (`crates/rum-stdlib/src/associations/data.rs:163-189,234-260`; `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:95-123,279-305`). |
| **brief-439 draft 3 — generic `MapIndexed`/`Scan`** | **SPLIT: DROP the priced association/`Scan` overlap; KEEP the post-result concern as an evaluator item; leave the generic local ceiling OPEN (Measured/Reasoned).** | It is the same source walker as draft I, so there is not a second walker implementation.  The supplied generic all-level result is 1.52x short in band and only an evaluator-owned generic post-result task stack can lift that ceiling; the local generic walker's own packet is not supplied (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-487`; `docs/dev/codex/brief-439-deep-rebuild-class-design.md:819-836`). |

**Reasoned.**  This disposition deliberately does **not** turn every
source-unbounded helper into an implementation item.  A passing
source-unbounded route remains a residual safety observation, but the old
broad drafts are dropped where their priced target now clears, matching the
measurement-scoped precedent recorded for briefs 449 and 450
(`docs/dev/codex/brief-457-association-tail-rederivation.md:111-116`).

### 5. Risks and open questions

1. **GroupBy's failure direction is the largest likely contradiction
   (Reasoned).**  Source proves a `Session::evaluate` call at every spec level,
   but not that a particular key/value/reducer evaluation reaches the next
   depth-strided byte poll before returning.  `evaluation_depth` is restored
   after each nested evaluation (`crates/rum-runtime/src/runtime/eval_core/mod.rs:19-53`),
   and there is no local poll in the grouping drivers.  I therefore predict a
   shallow-callback probe reaches the 2 MiB physical stack and terminates.  A
   callback with enough evaluator depth can instead fail in band; the
   director's exact callback shape can refute the representative verdict
   without refuting the local packet arithmetic.
2. **Schema has only 3.1% negative margin (Reasoned).**  Fixed entry overhead
   can only lower the 1,986-pair quotient, so the direction is likely robust,
   but LLVM layout drift and the leaf `MatchQ` poll phase can change the first
   observed bracket or produce an earlier in-band result
   (`crates/rum-stdlib/src/schema.rs:177-330`).  The source settles neither
   poll-counter phase nor an empirical bracket.
3. **DeleteMissing's broad draft contains routes not separately priced in
   brief-454 (Open).**  Section 4 supplies 160 B for recursive mode and 816 B
   for the finite-target association driver, but no standalone packet for
   `projected_expr_memory_proxy_bytes`, delayed-wrapper preflight, or
   `contains_missing_within` (`docs/dev/codex/brief-454-association-depth-class.md:313-313`;
   `crates/rum-stdlib/src/associations/missing.rs:84-230,1038-1053`).  The DROP
   verdict is for the priced association routes.  An armed-memory or scan-depth
   bracket below 2,048 would reopen a narrower item; source alone does not
   justify drafting it now.
4. **The generic `MapIndexed` measured fixture is underspecified here (Open).**
   The supplied notation uses `_`, not the concrete input shape
   (`docs/dev/codex/brief-457-association-tail-rederivation.md:134-138`).  The
   single-source-walker conclusion is solid because both association and
   standard calls enter `map_indexed_parts_recursive`, but the standard-call
   route adds `map_indexed_parts_on_standard_call`; the association-measured
   560 B packet cannot price it
   (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:329-408`).
   Whether the old 2,694/2,702 active bracket itself moves after brief-455 also
   depends on whether the concrete result reaches an `Association`.  The
   director's retained active probe and a stable generic-call bracket are
   needed to settle those two questions; neither was reconstructed or rerun
   here.
5. **The construction-ceiling attribution is internally contradictory
   (Measured documentation conflict).**  This brief instructs treating
   5,455-5,457 as recursive `Expr` drop at teardown
   (`docs/dev/codex/brief-457-association-tail-rederivation.md:189-193`), while
   brief-454's accepted director gate says the opposite: the liveness marker is
   never printed and explicitly refutes teardown
   (`docs/dev/codex/brief-454-association-depth-class.md:1178-1205`).  I obeyed
   the only conclusion common to both records—the ceiling is above `N=2048` and
   caps certification—and did not use or remeasure its mechanism.  That
   attribution should be reconciled before a future brief cites it as causal.
6. **All projected ceilings omit fixed entry overhead (Reasoned).**  This is
   most material for passing finite-target `DeleteMissing` (1.25x headroom),
   then association `MapIndexed`/draft-J parsers (1.83-1.88x).  It is least
   capable of changing the schema verdict because schema is already over the
   full 2 MiB quotient (`docs/dev/codex/brief-454-association-depth-class.md:290-296`).
7. **No claim is made above the construction ceiling (Reasoned).**  In
   particular, the 5,699/6,242/8,192/13,107 quotients are arithmetic used to
   show target headroom, not certifiable operation brackets
   (`docs/dev/codex/brief-457-association-tail-rederivation.md:187-193`).
