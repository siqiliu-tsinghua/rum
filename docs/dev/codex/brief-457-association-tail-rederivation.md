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
