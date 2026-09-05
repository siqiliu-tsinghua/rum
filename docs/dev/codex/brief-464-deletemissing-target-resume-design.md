# brief-464 — `DeleteMissing`'s finite-target/resume family (design-only)

## Goal

Design the conversion of `DeleteMissing`'s finite-target mode and its resume
family. It is the last natively-recursive path in this builtin and the only
thing between its registry row and `BoundedHeap`. It is also the hardest
conversion this line has attempted: eleven mutually recursive functions, five of
them `resume_suspended_*` variants, all of them able to suspend on user code.

**Scope: NONE — no source change.** The deliverable is a reviewable plan plus
drafted implementation briefs, every claim carrying `path:line`.

## What is given — measured, do not re-run, do not probe past a ceiling

Release CLI, `ulimit -s 2048`, 1 MiB evaluator budget. Highest value passing 3/3
against lowest failing 3/3. **Both axes terminate the process.**

| axis | shape | ok <= | fail >= |
|---|---|---|---|
| **depth** | `DeleteMissing[a, n, 0]` over a nested association | 4,944 physical | 4,960 |
| **depth**, scan `All` | `DeleteMissing[a, n, All]` | 4,928 physical | 4,960 |
| **width** | `DeleteMissing[Table[Missing[], {N}], 1, 0]` | **2,921 siblings** | **2,968** |
| **width**, association | the same over an `Association` of N entries | same bracket | same |

For contrast, the recursive mode — converted by briefs 462-463 in this same file
— now clears **64,000** siblings and its depth certificate passes at 4,097. **One
conversion closed both of its axes.** That is the single most useful datum here.

## The finding that reframes this work

brief-454 wrote off the sibling edges: *"The many `resume_*` calls that advance
`index + 1` are loops or tail transitions over siblings, not tree height."*
**Measurement says otherwise.** Rust does not guarantee tail-call elimination,
and these are not being compiled to loops: a **flat list of 3,000 `Missing[]`
entries terminates the process**. That value has physical height 2. It is not a
deep structure, not an adversarial input, and not near any documented limit —
it is an ordinary list.

So this is not only a 1.20x depth margin. It is a process kill on ordinary
input, and it is the more urgent of the two axes.

Treat that quote as a worked example of the rule this line keeps relearning: a
reasoned claim about what the compiler will do is a hypothesis, and it wants a
measurement before a plan is built on it.

## The four questions this brief exists to answer

### 1. Map both axes precisely

For every function in `crates/rum-stdlib/src/associations/missing.rs:214-818`,
say which of its self/mutual calls advance **sibling index** and which descend
**tree depth**, with `path:line` per edge. My reading is that `:337`, `:350`,
`:598`, `:623` and `:646` are sibling advances in tail position while `:267`,
`:302`, `:526` and `:562` are child descents, but that is a reading of the
source, not a verified claim — correct it.

Then say, per axis, **which edges the measured brackets actually exercise.** A
plan that converts an edge no probe reaches is a plan with an unfalsifiable
step.

### 2. What does suspension capture today, and what would an explicit stack
have to freeze instead?

This is the crux and the reason this brief is design-only. The family evaluates
user code — the scan predicate and delayed values — so it can suspend mid
traversal, and there are eleven entry points to resume into. Establish:

- the frames it uses (`crates/rum-stdlib/src/associations/missing/resume_frames.rs`)
  and exactly which state each one carries;
- what is currently held in **native frames** across a suspension, and what is
  in the heap frame — the split is what an explicit stack has to change;
- whether the five `resume_suspended_*` functions are a **second machine** or
  the same one re-entered, and what distinguishes them.

`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs` is the
worked precedent — brief-456 converted a suspension-bearing nested walk by
freezing the whole work stack. Say plainly whether that shape transfers here or
where it breaks.

### 3. Does one conversion close both axes?

It did for the recursive mode. Establish from the code whether the same is true
here, or whether the width axis needs its own treatment — for instance if
sibling state and child state cannot share one work representation without
changing the order in which user callbacks run.

**Callback order and count are observable.** State explicitly what the conversion
must preserve: which entries are visited, in what order, how many times each
predicate runs, and what a suspension resumes into.

### 4. The plan, and drafted implementation briefs

Ordered steps with the taxonomy this line uses, gate tests per step, the
certificate shape (a unique deepest or widest target, an assertion on the exact
rebuilt value, a shallow companion), and the falsifier each step needs.

**On falsifiers, follow what worked last time rather than what I specified.**
brief-463 was asked for a test-only cargo feature to force the bounded front and
instead called the private `*_with_front_depth` with a front of zero from an
in-module test — no feature, no production branch — asserting that front zero
really reports `Exhausted`, that front sixteen reports `Complete`, and that
**tail == front == expected** on a branched fixture with a per-level witness.
That is the better shape. Draft the equivalent here, and say what it would catch
that the certificates would not.

## Also deliver

5. **Whether the width defect is reachable in other builtins through the same
   idiom.** `index + 1` tail recursion over siblings appears elsewhere in the
   association surface; brief-460's recursion inventory
   (`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs`)
   is the map. Name any other site with the same shape and say whether its
   registry row's `NoProportionalDescent` is still honest — that row means no
   recursion **per association nesting level**, which says nothing about width.
   If any row is wrong, that is a finding, not a follow-up.

## Context pointers

- `AGENTS.md` first, including the Testing Policy hard lines.
- `crates/rum-stdlib/src/associations/missing.rs:214-818` — the family.
- `crates/rum-stdlib/src/associations/missing/resume_frames.rs` — its frames.
- `crates/rum-stdlib/src/associations/missing/traversal/deletion.rs` — the
  conversion briefs 462-463 just landed in the same file, including its
  in-module forced-front test. This is the closest model.
- `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs` — the
  suspension-bearing precedent.
- `TODO.md`, the 2026-09-05 entries for briefs 459-463.

## Scope

- Files you MAY modify: **NONE — design-only.** Append your report to this file.
- **Do not run any probe that drives a recursion to or past its ceiling**, on
  either axis, and do not write a reproduction whose purpose is to terminate a
  process. Every bracket you need is given above. Sub-ceiling `/proc` VmStk
  sampling is permitted if you want a slope.
- If a claim cannot be settled without a past-ceiling measurement, say so and
  stop there.

## rum / Rust pitfalls (read twice)

1. **Mark every claim measured or reasoned.** The claim this brief exists to
   correct was a reasoned one that read as settled.
2. **Do not assume tail calls become loops.** That is the specific error above.
   If you claim an edge is a loop, say how you established it.
3. `DeleteMissing[expr, level]` is the **recursive** mode; `Target` is the
   three-argument form. A two-argument probe measures the wrong walker.
4. The memory preflight is unreachable unless a limit is armed
   (`missing.rs:89-91`).
5. Logical wrappers are not physical height.
6. Wall-clock timing is not evidence on this host.

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
2. The per-edge axis map, and which edges the measured brackets exercise.
3. Suspension: what is captured where today, and what an explicit stack must
   freeze; whether the `resume_suspended_*` set is a second machine.
4. Whether one conversion closes both axes, and what callback-observable
   behaviour must be preserved.
5. The ordered plan and the drafted implementation briefs, with falsifiers in
   the in-module forced-front shape.
6. Deliverable 5: other sites with the same sibling idiom, and whether any
   registry row is wrong.
7. Risks and open questions, including anything you expect a director
   measurement to contradict.
