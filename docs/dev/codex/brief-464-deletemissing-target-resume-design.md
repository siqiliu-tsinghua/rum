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

---

## Report — 2026-09-05

### 1. Scope and evidence status

**Established, design-only.** No Rust source, test, registry, generated file, or
configuration was changed.  The only change is this appended report, as the
brief requires (`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:11-12,130-138,153-166`).
No ceiling probe, sub-ceiling probe, timing run, or test gate was run.  The four
brackets below are the brief's supplied measurements, not measurements made in
this report (`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:14-24`).

**Reasoned from project policy.** The drafted tests below use only rum's own
documented/runtime behavior; no external engine is an oracle
(`AGENTS.md:36-51`; `docs/testing-strategy.md:3-14`).  Suspension behavior is a
runtime-regression concern and deep/wide limits are stress-test concerns
(`docs/testing-strategy.md:18-37`).

### 2. Exact axis map

#### Classification rule

**Reasoned.** In this map, a **sibling** edge keeps `level` and passes
`index + 1`; a **depth** edge passes `level - 1` and resets `index` and `kept`
to zero/empty.  A call into an `after_*` helper at the same `index` is a
same-node continuation, not motion on either axis.  Those facts are visible in
the argument lists themselves (`crates/rum-stdlib/src/associations/missing.rs:239-350,353-448,450-654,656-817`).

**Reasoned correction to the question's initial reading.** The proposed depth
edges at `:267`, `:302`, `:526`, and `:562` are correct but incomplete: the
post-suspension drivers also descend at `:383`, `:393`, `:722`, and `:732`.
The proposed sibling edges at `:337`, `:350`, `:598`, `:623`, and `:646` are
correct but incomplete: sibling recursion also occurs at `:256`, `:372`,
`:405`, `:440`, `:463`, `:515`, `:671`, `:711`, `:744`, `:787`, and `:809`
(`crates/rum-stdlib/src/associations/missing.rs:251-337,365-447,458-646,664-816`).

#### All eleven functions

Every row in this table is **Reasoned from source**; the cited call is the
evidence.  “Continuation” means it must preserve state/order, but does not by
itself increase sibling count or tree depth.

| function | sibling advances | child descents | same-node/entry dispatch |
|---|---|---|---|
| `delete_missing_at_target_level` | none | none | root list dispatch at `crates/rum-stdlib/src/associations/missing.rs:220-229`; root association dispatch at `:232-236` |
| `resume_delete_missing_list_at_target_level` | level-1 item completion at `crates/rum-stdlib/src/associations/missing.rs:251-263`; non-keyed item completion at `:336-337` | list child at `:266-274`; association child at `:301-309` | completed list child to `after_child` at `:275-284`; completed association child to the same helper at `:310-319` |
| `resume_delete_missing_list_after_child` | append child and advance at `crates/rum-stdlib/src/associations/missing.rs:349-350` | none | none |
| `resume_suspended_delete_missing_list_at_target_level` | level-1 item completion at `crates/rum-stdlib/src/associations/missing.rs:365-379`; non-keyed item completion at `:404-412` | fresh ordinary list child at `:382-390`; fresh ordinary association child at `:392-401` | completed child to suspended `after_child` at `:415-418` |
| `resume_suspended_delete_missing_list_after_child` | append child and advance at `crates/rum-stdlib/src/associations/missing.rs:439-447` | none | none |
| `resume_delete_missing_association_at_target_level` | syntactic `Missing` value at `crates/rum-stdlib/src/associations/missing.rs:462-470`; delayed entry above target at `:506-522`; ordinary leaf at `:597-605` | list-valued entry at `:525-533`; association-valued entry at `:561-569` | target predicate to `after_predicate` at `:473-489`; completed list child to `after_child` at `:534-544`; completed association child to it at `:570-580` |
| `resume_delete_missing_association_after_predicate` | predicate decision then advance at `crates/rum-stdlib/src/associations/missing.rs:619-630` | none | none |
| `resume_delete_missing_association_after_child` | attach child then advance at `crates/rum-stdlib/src/associations/missing.rs:643-653` | none | none |
| `resume_suspended_delete_missing_association_at_target_level` | syntactic `Missing` value at `crates/rum-stdlib/src/associations/missing.rs:670-678`; delayed entry above target at `:703-718`; ordinary leaf at `:743-751` | fresh ordinary list child at `:721-729`; fresh ordinary association child at `:731-740` | target predicate to suspended `after_predicate` at `:681-690`; completed child to suspended `after_child` at `:754-757` |
| `resume_suspended_delete_missing_association_after_predicate` | predicate decision then advance at `crates/rum-stdlib/src/associations/missing.rs:781-794` | none | none |
| `resume_suspended_delete_missing_association_after_child` | attach child then advance at `crates/rum-stdlib/src/associations/missing.rs:808-816` | none | none |

#### Which edges the supplied brackets exercise

**Measured bracket / reasoned route.** The flat-list width bracket is measured
at 2,921/2,968 siblings (`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:23-23`).
For its exact expression, root dispatch is `:221`; every `Missing[]` enters the
`level == 1` branch, is rejected by the structural test, and advances through
the self-call at `:256`; completion uses the `items.get(index)` terminator at
`:247-249` (`crates/rum-stdlib/src/associations/missing.rs:220-229,247-263`).
It does **not** exercise `:337` or `:350`, because those are below the
`level == 1` return, and it exercises no child, association, suspended, or
`after_child` edge (`crates/rum-stdlib/src/associations/missing.rs:251-350`).

**Measured bracket / reasoned route.** The N-entry association width bracket has
the same measured 2,921/2,968 bracket
(`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:24-24`).  For
entries whose values are `Missing[]`, root dispatch is `:235`, the syntactic
test at `:462` succeeds, and every entry advances through the self-call at
`:463`; completion is `:458-460`
(`crates/rum-stdlib/src/associations/missing.rs:232-236,458-470`).  It does
**not** exercise the originally nominated association sibling edges `:598`,
`:623`, or `:646`, nor any delayed evaluation or child edge
(`crates/rum-stdlib/src/associations/missing.rs:473-653`).  Thus the two measured
width failures bind precisely the two sibling edges omitted by the initial
reading: `:256` and `:463`.

**Measured bracket / reasoned route.** Both nested-association depth brackets
terminate at 4,944/4,960 for scan `0` and 4,928/4,960 for scan `All`
(`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:21-22`).  An
immediate association-valued chain necessarily dispatches at `:235`, descends
through `resume_delete_missing_association_at_target_level` at `:562` with
`level - 1`, returns through `resume_delete_missing_association_after_child` at
`:571`, and makes the terminating next-index call at `:646`
(`crates/rum-stdlib/src/associations/missing.rs:232-236,561-580,634-653`).  The
same chain does not exercise list descents `:267`, `:302`, or `:526`.  The
supplied depth-probe notation documents no input request, so no post-suspension
edge can honestly be credited as measured from it
(`crates/rum-stdlib/src/associations/missing.rs:266-333,525-558,656-817`).

**Open, because the supplied fixture is underspecified.** The phrase “over a
nested association” does not state the target association's terminal entries.
Consequently this report cannot honestly choose among terminal sibling edge
`:463` (syntactic `Missing`), `:623` (a non-missing target predicate), or no
terminal sibling edge (empty association), and it cannot prove that the
scan-`All` bracket actually enters `contains_missing_within` rather than merely
passing `All` as an unused value.  Those alternatives are selected at
`crates/rum-stdlib/src/associations/missing.rs:458-503,609-630`; the scan helper
is called at `:619`.  Settling the terminal edge requires the director's exact
fixture/command, not another ceiling run.

**Reasoned probe-coverage conclusion.** A width repair proved only with the
given probes must remove/loop `:256` and `:463`; converting only `:337`, `:350`,
`:598`, `:623`, and `:646` would not touch either measured width path.  A depth
repair proved by the stated association-chain probes must bound `:562` and the
return/advance path through `:571`/`:646`; converting list-only child edges
would not touch that measured depth path
(`crates/rum-stdlib/src/associations/missing.rs:251-350,458-653`).

### 3. Suspension state today and the state an explicit stack must own

#### What can actually suspend in this family

**Reasoned correction/qualification.** `scan` is a depth value, not executable
user code: it is parsed as a non-negative integer/`Infinity`/`All` at
`crates/rum-stdlib/src/associations/missing.rs:171-177`, and the scan predicate
is the pure `contains_missing_within` helper with no `Session` argument
(`crates/rum-stdlib/src/associations/missing/traversal/deletion.rs:325-387`).
The operation-local suspension points in this family are the two calls that
evaluate a target-level `RuleDelayed` RHS, on the ordinary path at
`crates/rum-stdlib/src/associations/missing.rs:473-503` and after a prior
suspension at `:681-700`.  Immediate values are cloned rather than evaluated at
`:476-478` and `:684-686`.

**Reasoned.** A delayed entry above the target does not run user code during
this traversal; it is replaced by a wrapper at
`crates/rum-stdlib/src/associations/missing.rs:506-514,703-710`.  If that
`RuleDelayed` is evaluated later, its wrapper invokes a new `DeleteMissing`
call (`crates/rum-stdlib/src/associations/missing.rs:819-849`), which is a new
machine invocation and may independently suspend.

#### The eight heap frame types

All rows are **Reasoned from the frame definitions and their `resume` targets**.
The list frames never need a current `item`, because the pending child result
replaces it; the association frames retain `entry`, because either the original
rule must be kept after a predicate or its value must be replaced after a child
(`crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-356`).

| frame | state frozen in the heap frame | successful resume target / distinction |
|---|---|---|
| `ResumeDeleteMissingListAtTargetLevelSuspensionFrame1` | `items`, `level`, `scan`, `index`, `kept` (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-14`) | ordinary parent waiting for a **list** child; resume supplies the child to suspended list `after_child` (`:16-43`) |
| `...ListAtTargetLevelSuspensionFrame2` | the same five fields (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:46-55`) | ordinary parent waiting for an **association** child; same continuation (`:57-84`) |
| `ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrame` | the same five fields, already frozen/persistent (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:87-96`) | a previously suspended list parent waiting for either kind of fresh child; same continuation (`:98-125`) |
| `ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1` | `entries`, `level`, `scan`, `index`, `kept`, and original `entry` (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:128-140`) | ordinary target association waiting for its delayed **predicate value**; resume enters suspended `after_predicate` (`:142-171`) |
| `...AssociationAtTargetLevelSuspensionFrame2` | the same six fields (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:174-186`) | ordinary association parent waiting for a **list** child; resume enters suspended `after_child` (`:188-217`) |
| `...AssociationAtTargetLevelSuspensionFrame3` | the same six fields (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:220-232`) | ordinary association parent waiting for an **association** child; same continuation (`:234-263`) |
| `ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame1` | the same six fields, already frozen/persistent (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:266-278`) | previously suspended association waiting for a delayed **predicate value**; resume enters suspended `after_predicate` (`:280-309`) |
| `ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame2` | the same six fields, already frozen/persistent (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:312-324`) | previously suspended association waiting for either fresh **child** kind; resume enters suspended `after_child` (`:326-355`) |

**Reasoned storage semantics.** On first suspension, an owned input `Vec` is
moved into `FrozenAssociationSequence(Arc<[T]>)`, while the completed mutable
prefix becomes `PersistentAssociationSequence`, an `Arc` prefix plus an O(1)
append chain (`crates/rum-stdlib/src/associations/suspended_state.rs:7-23,44-98`).
The macro constructs this state only on `InputRequired`, inserts the named
frame, and returns the same request
(`crates/rum-runtime/src/runtime/suspension/core.rs:654-674`).

#### Native/heap split on an actual suspension

**Reasoned.** Immediately before the innermost delayed evaluation returns
`InputRequired`, every resident native association ancestor owns its input
`Vec<AssociationEntryForm>`, `level`, `scan`, current `index`, completed `kept`
prefix, current `entry`, and the pending child-result match; list ancestors own
the analogous state without `entry`
(`crates/rum-stdlib/src/associations/missing.rs:239-333,450-595`).  The
innermost predicate site freezes one association frame at `:490-500`; as the
error unwinds, each ordinary list parent freezes at `:285-295` or `:320-330`,
and each ordinary association parent freezes at `:545-555` or `:581-591`.

**Reasoned.** Frames are appended to the pending continuation queue as native
parents unwind (`crates/rum-runtime/src/runtime/suspension/core.rs:454-467`).
After unwinding, no traversal-local state remains only on those native frames:
the queue collectively contains one heap frame per waiting traversal ancestor,
with the evaluator's own continuation state.  Resumption consumes queue frames
front-to-back and feeds each completed `Expr` to the next parent continuation
(`crates/rum-runtime/src/runtime/suspension/core.rs:601-632`).  A second input
request splices the new nested continuation ahead of the still-live suffix
(`crates/rum-runtime/src/runtime/suspension/core.rs:634-648`).

**Reasoned.** If a resumed parent launches a fresh child and that child
suspends, the already-frozen parent is copied into the third list frame at
`crates/rum-stdlib/src/associations/missing.rs:419-425` or the second suspended
association-child frame at `:758-764`; a second target-level delayed predicate
uses the suspended predicate frame at `:691-697`.  This is how repeated
suspensions retain earlier prefixes without requiring `Arc::get_mut`.

#### One machine or two?

**Reasoned answer.** The five `resume_suspended_*` functions are **not an
independent semantic machine**, but they are a duplicated storage-mode
implementation of the same depth-first state machine.  They repeat the same
terminal tests, child classification, delayed-wrapper rule, and left-to-right
advance as the owned functions, using `FrozenAssociationSequence` and
`PersistentAssociationSequence` instead of `Vec`
(`crates/rum-stdlib/src/associations/missing.rs:239-350,353-448,450-654,656-817`).
They re-enter the ordinary machine for every fresh list/association child at
`:383-400` and `:722-739`; if that child suspends, the parent returns to the
frozen variant through one of the frames above.  Thus “owned before first
suspension” versus “immutable/persistent after first suspension” distinguishes
the variants, not traversal semantics.

#### What an explicit stack must freeze

**Reasoned requirement.** An explicit tail cannot freeze merely the current
`items`/`entries` cursor.  It must freeze, for **every active ancestor**: the
container kind; immutable input sequence; `level` and `scan`; next/current
sibling index; completed output prefix; the current association entry when a
child or predicate result still has to be attached; and whether the returned
`Expr` is to finish a list child, association child, or predicate
(`crates/rum-stdlib/src/associations/missing.rs:239-350,450-654`).  It also must
preserve the LIFO ordering of those finish continuations, because that ordering
is what attaches a completed child before visiting the parent's next sibling
(`crates/rum-stdlib/src/associations/missing.rs:275-284,310-319,534-544,570-580`).

**Reasoned recommended representation.** Use one traceable work enum with
cursor and finish variants, conceptually:

- `DriveList { items, level, scan, index, kept }`;
- `DriveAssociation { entries, level, scan, index, kept }`;
- `FinishListChild { parent_cursor }`;
- `FinishAssociationChild { parent_cursor, entry }`;
- `FinishAssociationPredicate { parent_cursor, entry }`.

The input/prefix fields should use the already-established frozen/persistent
containers once work crosses to the cold tail
(`crates/rum-stdlib/src/associations/suspended_state.rs:7-23,44-98`).  Before
calling `Session::evaluate`, push `FinishAssociationPredicate`; then a
suspension frame needs only a frozen **whole work vector**, and the resumed
value is the driver's single `completed: Option<Expr>`
(`crates/rum-stdlib/src/associations/missing.rs:473-503,681-700`).  Every work
variant and the containing frame must derive `FrameRootTrace`, as the current
frames do (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-6,46-47,87-88,128-129,174-175,220-221,266-267,312-313`).

**Reasoned precedent transfer.** `MergeNested` transfers directly at the
control-flow level: it represents enter/finish work in one LIFO vector, pushes
the finish before the child, and freezes the entire vector when a leaf callback
suspends (`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:5-13,134-190`).
Its frame later resumes that vector with the callback result
(`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:197-203`).
The shape does **not** transfer as only two variants: `DeleteMissing` has two
container types, a target predicate continuation distinct from a child
continuation, an original association entry whose head must survive, and
deletion rather than one result per input (`crates/rum-stdlib/src/associations/missing.rs:251-253,462-514,619-645`).

### 4. One conversion can close both axes, with an explicit observable contract

#### Verdict

**Reasoned.** One architectural conversion **can and should close both axes**,
provided it removes every same-level recursive call as well as bounding every
`level - 1` child call.  A depth-only bounded front leaves the measured width
edges `:256` and `:463` untouched; conversely, replacing only sibling calls by
loops leaves measured child descent `:562` native
(`crates/rum-stdlib/src/associations/missing.rs:251-267,458-470,561-569`).

**Reasoned.** Sibling and child state can share the cursor/finish representation
above without reordering callbacks: a cursor processes one entry, pushes its
own finish/next-cursor continuation, then pushes the child, so LIFO execution is
the current left-to-right depth-first order.  Sibling advancement itself should
be an explicit Rust `loop`/driver iteration, not a tail-call assumption; the
other association resumable drivers demonstrate this exact loop-backed shape
(`crates/rum-stdlib/src/associations/resumable/map.rs:52-83,86-113`).

#### Behavior that is not negotiable

Each item below is **Reasoned from the current implementation** and therefore a
conversion invariant:

1. Lists and associations are visited in source order; an immediate keyed child
   is completed depth-first before the next sibling
   (`crates/rum-stdlib/src/associations/missing.rs:247-350,458-653`).
2. At target level in a list, an item is dropped when it is a rule-like form
   with a syntactically `Missing` value or when the structural scan finds
   `Missing` within `scan`; the rule-like test short-circuits the scan
   (`crates/rum-stdlib/src/associations/missing.rs:251-254,852-854`).
3. At every visited association level, an entry with a syntactically `Missing`
   RHS is dropped before target-level delayed evaluation; it therefore runs no
   user code (`crates/rum-stdlib/src/associations/missing.rs:458-471,664-679`).
4. At target level in an association, each reached `RuleDelayed` RHS is
   evaluated **exactly once** and each immediate RHS zero times.  The original
   entry, including `Rule` versus `RuleDelayed`, is retained only if the
   resulting value is neither root `Missing` nor structurally missing within
   `scan` (`crates/rum-stdlib/src/associations/missing.rs:473-503,609-630`).
5. Above target level, an immediate list/association RHS is traversed now; a
   delayed RHS is not evaluated and instead receives exactly one delayed
   cleanup wrapper for `level - 1` (`crates/rum-stdlib/src/associations/missing.rs:506-569,819-849`).
6. Non-keyed leaves are retained unchanged; rebuilt lists and associations keep
   input order, association keys, and rule heads
   (`crates/rum-stdlib/src/associations/missing.rs:336-350,597-653`).
7. A suspension resumes **after** the pending evaluation: its returned `Expr`
   feeds the pending predicate/child finish, completed prefixes are not
   revisited, the pending delayed RHS is not reevaluated, and the next callback
   is the next entry in depth-first/source order
   (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:16-43,142-171,188-217,234-263,280-355`).
8. Non-input errors propagate unchanged, and cancellation/abandonment remains a
   pure frame discard because all eight current frames derive
   `PureFrameAbandon` (`crates/rum-stdlib/src/associations/missing.rs:297-298,332-333,427-428,502-503,557-558,593-594,699-700,766-767`; `crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-6,46-47,87-88,128-129,174-175,220-221,266-267,312-313`).

### 5. Ordered plan and drafted implementation briefs

#### Overall order

**Reasoned plan.** Use two review steps: first lock the shallow observable
semantics against the untouched implementation; then land one unified
`BoundedHeap` conversion that loops siblings and hands depth overflow to a
cold explicit work stack.  Do not reclassify the registry between those steps;
today it correctly remains `UnguardedNative` and names this work
(`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:163-171`).

#### Draft implementation brief A — semantic and suspension locks (test-only)

**Goal.** Add small, non-ceiling tests that distinguish every observable branch
before production code moves.  Existing surface coverage pins several target
level/scan and delayed-head cases
(`crates/rum-stdlib/src/tests/associations_missing_surface.rs:78-223`), but the
current request suite has only one eager delayed predicate suspension and one
nested association suspension
(`crates/rum-stdlib/src/tests/session_requests/transforms_resume_associations.rs:508-558`).

**Scope.** Modify only focused rum-stdlib tests and test wiring.  Do not touch
production source, the recursion inventory, or the registry in this brief.
Use no external oracle, consistent with `docs/testing-strategy.md:3-14`.

**Fixtures and gates.** All are **Reasoned requirements** from the observable
contract in section 4:

1. Add a shallow mixed list/association tree with multiple siblings at each of
   three levels, immediate and delayed rules, syntactic `Missing`, scanned
   nested `Missing`, and a distinct per-level survivor.  Assert the exact value,
   order, and rule heads, not `Depth`/`Length
   (`crates/rum-stdlib/src/associations/missing.rs:251-350,458-653`).
2. Add a two-input suspension fixture whose target-level delayed RHS appends
   `before-i`, requests input, then appends `after-i`.  Put the two callbacks in
   different branches with a completed sibling before, between, and after them.
   Assert prompt order, exact final value, and the exact side-effect log once;
   compare queued-input and explicit-resume outcomes, following the existing
   request-test idiom
   (`crates/rum-stdlib/src/tests/session_requests/transforms_resume_associations.rs:11-47`).
3. Add the same schedule with a list parent around an association child so both
   ordinary parent suspension sites `:285-295`/`:320-330` and association
   parent sites `:545-591` have coverage
   (`crates/rum-stdlib/src/associations/missing.rs:266-333,525-594`).
4. Run the focused tests against untouched source, then deliberately make the
   expected callback log contain a replay and deliberately transpose two
   expected siblings; each mutation must fail, then be restored.  Those are the
   falsifiers for count and order.  This follows the principle that small
   helper/runtime tests should directly reach high-risk policy branches
   (`docs/testing-strategy.md:18-26`).

**Acceptance/stop.** Accept only if all new tests pass on the old machine and
both deliberate expectation mutations fail.  Stop rather than bless any
existing behavior conflict; weakening a contract to clear a test is prohibited
(`AGENTS.md:46-51`).  No deep or wide pre-change run belongs in this step,
because the supplied brief forbids reproductions intended to kill a process
(`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:130-138`).

#### Draft implementation brief B — unified finite-target `BoundedHeap` conversion

**Goal.** Remove all unbounded `delete-missing` edges in
`crates/rum-stdlib/src/associations/missing.rs:214-817`, close the measured
width paths at `:256`/`:463`, bound all eight ordinary/resumed child descents,
preserve brief A's exact schedule, and only then flip the registry row to
`BoundedHeap` (`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:487-612`; `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:163-171`).

**Implementation taxonomy.** This is **Reasoned design**, patterned on existing
bounded/heap code:

1. Move the finite-target machine into a cohesive
   `associations/missing/target.rs` module; `missing.rs` is 26,353 bytes in the
   measured source-audit output below and therefore is already in the
   split-plan range whose assess-on-touch rule is
   `docs/code-organization-guidelines.md:64-81`.
2. Use a depth-16 recursive front for shallow child descent, but implement each
   list/association sibling cursor with a real `loop`.  Decrement the front only
   when following one of the eight `level - 1` edges; never rely on tail-call
   elimination for an `index + 1` edge
   (`crates/rum-stdlib/src/associations/missing.rs:266-309,382-401,525-569,721-740`).
3. Return `BoundedTraversal::Exhausted(work)` (or an equivalent target-specific
   enum carrying work) **before** an over-front child is visited.  While that
   result unwinds through the bounded parents, prepend/wrap each live parent
   cursor and finish task so the tail begins at the first unvisited child with
   all already-completed prefixes intact.  Restarting from the root is
   forbidden because earlier delayed predicates may already have run
   (`crates/rum-stdlib/src/associations/missing.rs:473-503`).
4. Drive the cold tail with the cursor/finish enum from section 3.  Its one-item
   cursor discipline covers width without pre-expanding all siblings and its
   finish-before-child LIFO discipline preserves depth-first order.  This is the
   same bounded-front/cold-tail taxonomy as recursive `DeleteMissing`
   (`crates/rum-stdlib/src/associations/missing/traversal/deletion.rs:14-23,25-50,139-194`).
5. At a tail delayed predicate, push `FinishAssociationPredicate` before
   `Session::evaluate`; on `InputRequired`, freeze the **entire** work vector in
   one traceable, pure-abandon frame.  Resume the driver with the supplied value
   as `completed`, exactly as `MergeNested` does
   (`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:134-203`).
6. The bounded front may retain the current per-ancestor frames, but every
   re-entry must call the bounded/looping driver and each uninterrupted native
   segment must remain at most 16 child descents.  Alternatively, collapse them
   to the single whole-work frame only if brief A's schedule and the allocation
   gate remain identical.  No unbounded `resume_suspended_*` child or sibling
   cycle may survive (`crates/rum-stdlib/src/associations/missing.rs:353-448,656-817`).
7. Preserve the ordinary shallow path's mutable `Vec` storage.  Freeze inputs
   and prefixes only on actual suspension or cold-tail handoff, matching the
   current cold allocation boundary
   (`crates/rum-stdlib/src/associations/suspended_state.rs:7-13,44-80`).

**Required in-module forced-front falsifier.** Add a private
`delete_missing_at_target_level_with_front_depth` plus a directly testable
bounded helper; add no cargo feature and no production switch.  On one shallow,
branched mixed-container fixture with three entries per association and a
different numeric/string witness per level:

- assert the bounded helper with front `0` returns `Exhausted` and carries
  nonempty work;
- assert the bounded helper with front `16` returns `Complete`;
- compute through `_with_front_depth(..., 0)` and through
  `_with_front_depth(..., 16)` and assert **tail == front == exact expected**;
- iteratively inspect each level's keys, order, rule head, witness, and child
  attachment.

This is **Reasoned to be the exact equivalent** of the existing recursive-mode
test, which proves zero-front exhaustion, normal-front completion, equality,
exact expected output, and a per-level witness
(`crates/rum-stdlib/src/associations/missing/traversal/deletion.rs:410-477`).
The new fixture must additionally contain both a list child and an association
child because the finite-target family has distinct edges for them
(`crates/rum-stdlib/src/associations/missing.rs:266-319,525-580`).

**What that falsifier catches that certificates do not.** **Reasoned:** a deep
single association chain can pass while list work variants are dead, sibling
order is reversed, a completed prefix is dropped at handoff, a child is
attached to the wrong entry, or the supposed tail is never selected.  Direct
`Exhausted`/`Complete` assertions prove branch selection; front/tail/exact
equality and per-level witnesses prove balance and order
(`crates/rum-stdlib/src/associations/missing/traversal/deletion.rs:431-477`).
Add a second front-zero version of brief A's two-input test; it proves that the
whole explicit stack survives repeated suspension without replay, a property
no non-suspending depth/width certificate can establish
(`crates/rum-runtime/src/runtime/suspension/core.rs:634-648`).

**Certificates.** These are **Reasoned gate designs**, and they run only after
the conversion is present:

1. Finite-target depth, scan `0`: release-only ignored test on a declared 2 MiB
   thread, raw physical height 4,097 verified iteratively.  The unique deepest
   target association has one `Missing` entry and one distinguishing survivor;
   every outer association has a per-level witness.  Invoke the three-argument
   form `DeleteMissing[input, target_level, 0]` and iteratively assert the exact
   rebuilt chain, not merely completion.  The three-argument dispatch is the
   target mode at `crates/rum-stdlib/src/associations/missing.rs:43-60,207-210`.
2. Finite-target depth, scan `All`: same physical-height discipline, but the
   unique target is a non-root nested `Missing` under a distinguishable call so
   `contains_missing_within` must run; assert its exact removal and a sibling
   survivor.  A shallow companion compares scan `1` with `2`/`All`, extending
   the existing companion at
   `crates/rum-stdlib/src/tests/delete_missing_depth.rs:276-286`.
3. List width: 64,000 target-level items on a declared 2 MiB thread, mostly
   `Missing[]` with unique first/middle/last survivors; assert the exact survivor
   list and order.  This directly covers measured edge `:256`
   (`crates/rum-stdlib/src/associations/missing.rs:251-263`).
4. Association width: 64,000 entries, mixing syntactic `Missing` values with
   unique immediate survivors at known positions; assert the exact keys,
   values, heads, and order.  This directly covers measured edge `:463` and the
   non-missing predicate advance `:623`
   (`crates/rum-stdlib/src/associations/missing.rs:462-503,609-630`).
5. Each certificate gets a three-level/short-width shallow companion that pins
   target versus recursive arity, scan depth, list versus association behavior,
   and delayed evaluation count.  Existing shallow assertions show the expected
   style but do not replace these axis-specific companions
   (`crates/rum-stdlib/src/tests/associations_missing_surface.rs:78-223`).

**Inventory and registry gate.** Remove the unbounded `delete-missing` rows for
the direct cycles at
`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:487-612`.
Declare only any remaining depth-16 front edges under a new bounded-front site
ID, retain the already-converted helper declaration, then set the
`DeleteMissing` row to `BoundedHeap`, keep `MeetsContract`, record an honest
post-tail failure direction, and clear `owed_by`
(`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:163-171,462-518`).
The flip is rejected if any reachable target/resume cycle remains.

**Correctness and cost gates.** Run focused brief-A tests and the forced-front
tests first, then `cargo fmt --check`, `cargo test -p rum-stdlib`, and
`cargo test --workspace` under `scripts/run-bounded.py` for long runs, as
required by `AGENTS.md:75-93`.  Run the four post-conversion certificates only
after shallow green.  Compare allocation counters first on shallow target list,
immediate association, delayed association, and an untouched control; then run
rum-vs-rum A/B microbench rows for small target list/association shapes and
same-tier neighbors (`AGENTS.md:46-49,117-128`; `docs/testing-strategy.md:74-94`).
Any shallow-path allocation increase, callback replay/reorder, exact-value
difference, non-traceable frame, failure of front zero to report `Exhausted`,
or surviving unbounded inventory edge is a stop, not a follow-up.

### 6. Other association-surface `index + 1` sites and registry honesty

#### Source survey result

**Reasoned, not measured.** No other surveyed association builtin has the same
ordinary-width native recursion defect.  Several suspension continuations have
the same **textual** `index + 1` tail call, but each lands in a function whose
body contains an explicit `loop`; one continuation frame calls one loop, and
the loop advances all subsequent siblings without recursive self-calls.  This
is established by source control flow, not by assuming LLVM tail-call
elimination.

| surface/site | textual continuation | explicit loop that receives it | registry result |
|---|---|---|---|
| `Association` key evaluation | `crates/rum-stdlib/src/associations/basic.rs:106-116` | suspended driver loop at `:74-104` (ordinary loop at `:45-72`) | already `UnguardedNative` for its separate deep parsers, so no new row finding (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:79-90`) |
| `Counts` | `crates/rum-stdlib/src/associations/aggregate.rs:238-253` | suspended loop at `:201-235` | `NoProportionalDescent` remains honest (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:145-152`) |
| `Catenate` | continuations at `crates/rum-stdlib/src/associations/aggregate.rs:328-343,484-493` | loops at `:294-325,433-481` | `NoProportionalDescent` remains honest (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:136-143`) |
| `KeyGroupBy` key and reducer phases | `crates/rum-stdlib/src/associations/key_reshape.rs:215-233,315-325` | loops at `:182-213,284-313` | `NoProportionalDescent` remains honest (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:244-251`) |
| ordinary `Merge` | `crates/rum-stdlib/src/associations/resumable/aggregate.rs:177-187` | suspended loop at `:144-175` | row is `GuardedNative` for a different recursive parser; no new finding (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:379-387`) |
| `CountsBy` | `crates/rum-stdlib/src/associations/resumable/aggregate.rs:267-276` | suspended loop at `:231-265` | `NoProportionalDescent` remains honest (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:154-161`) |
| both `AssociationMap` shapes | `crates/rum-stdlib/src/associations/resumable/map.rs:116-131,202-219` | suspended loops at `:86-113,169-199` | `NoProportionalDescent` remains honest (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:109-117`) |
| `KeySelect` | `crates/rum-stdlib/src/associations/resumable/key_select.rs:110-125` | suspended loop at `:75-107` | `NoProportionalDescent` remains honest (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:280-287`) |
| `KeyUnion` fill phase | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:398-415` | suspended loop at `:362-395` | `NoProportionalDescent` remains honest (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:316-323`) |
| `GroupBy` list/association group siblings | list continuation `crates/rum-stdlib/src/associations/grouping/list.rs:397-421`; association continuation `crates/rum-stdlib/src/associations/grouping/association.rs:413-438` | list/association suspended loops at `list.rs:350-393` and `association.rs:367-410` | the row is already `UnguardedNative`/`ShortOfContract` for the distinct multi-spec recursion (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:172-180`) |

**Reasoned separate finding, already represented rather than newly discovered.**
`GroupBy` does have a width-like unbounded axis over the number of specs:
`finish_group_by_grouped` increments `spec_index` and enters the nested driver,
which calls back into the next-spec worker
(`crates/rum-stdlib/src/associations/grouping/list.rs:257-277,299-347`); the
association version is the same at
`crates/rum-stdlib/src/associations/grouping/association.rs:279-299,316-365`.
The recursion inventory declares both cycles
(`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:417-472`),
and the registry already classifies `GroupBy` as unguarded, fail-open, and short
of contract (`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:172-180`).
It is therefore not a wrong `NoProportionalDescent` row.

**Reasoned inventory qualification.** The inventory deliberately sees only
same-file direct call cycles and documents trait-object, function-pointer,
macro-generated, and out-of-root blind spots
(`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:1-9,20-25`).
The one-way post-resume calls above do not form a direct source cycle because a
`SuspensionFrame::resume` callback mediates re-entry; the explicit target loops
are why that blind spot does not hide sibling stack growth.  `DeleteMissing`,
in contrast, has direct cycles and is exhaustively present in the inventory
(`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs:487-612`).

**Registry verdict.** **Reasoned:** no current `NoProportionalDescent` row is
wrong on this survey.  That label speaks only to recursion proportional to
association nesting, as this brief specifies
(`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:109-116`), and
the named NoProportional sites also use explicit loops for their sibling axes.
This is a source verdict, not a claim that arbitrary wide inputs cannot hit a
different evaluator, memory, or generated-element limit.

### 7. Risks, open questions, and expected measurement challenges

1. **Open — exact depth-probe leaf.** The supplied depth notation is not enough
   to identify the terminal target branch, as section 2 explains
   (`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:21-22`;
   `crates/rum-stdlib/src/associations/missing.rs:458-503`).  The director should
   attach the exact existing fixture/command to the implementation brief; no
   repeat ceiling run is needed.
2. **Open — suspended depth/width ceiling.** The supplied probe descriptions
   document no input request, so the five post-suspension functions and eight
   heap frames have no measured bracket here
   (`docs/dev/codex/brief-464-deletemissing-target-resume-design.md:19-24`;
   `crates/rum-stdlib/src/associations/missing.rs:353-448,656-817`).  Correctness
   must come from forced-tail multi-suspension tests; this report makes no
   numerical suspended-ceiling claim.
3. **Reasoned risk — replay at bounded handoff.** A pure walker may restart an
   iterative tail after front exhaustion, but this one cannot, because a
   target-level delayed evaluation can mutate state before a later child
   exhausts (`crates/rum-stdlib/src/associations/missing.rs:473-503`).  The
   exhausted value must carry the live continuation and prefixes.
4. **Reasoned risk — allocation shape.** Using
   `PersistentAssociationSequence::appended` on the ordinary wide path would
   allocate one `Arc` node per retained item (`crates/rum-stdlib/src/associations/suspended_state.rs:72-80`).
   Keep mutable vectors on the unsuspended front/loop and measure the tail;
   otherwise the width fix may trade stack failure for avoidable heap growth.
5. **Reasoned risk — frame rooting and abandonment.** A whole-work frame owns
   more heterogeneous `Expr` roots than any current frame.  Every enum field
   must trace, and abandonment must remain pure
   (`crates/rum-runtime/src/runtime/suspension/core.rs:68-82`;
   `crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-6`).
6. **Reasoned risk — duplicated owned/suspended semantics.** Retaining two
   cursor implementations makes predicate-order drift possible; fully sharing
   transition helpers lowers that risk, but any abstraction that allocates or
   clones on the shallow path fails the cost gate
   (`crates/rum-stdlib/src/associations/missing.rs:239-448,450-817`;
   `AGENTS.md:117-128`).
7. **Expected director-measurement challenge.** I expect the converted
   64,000-wide Missing-only fixtures to stop showing native-stack growth once
   `:256` and `:463` are real loops; if they fail, the likely competing causes
   are construction, memory, or evaluator/result processing, not those removed
   recursive edges.  That attribution is a reasoned prediction and must be
   checked by the exact post-conversion value/certificate, not wall time
   (`crates/rum-stdlib/src/associations/missing.rs:247-263,458-470`;
   `AGENTS.md:124-128`).
8. **Expected director-measurement challenge.** The scan-`All` depth bracket may
   not move in lockstep with scan `0`, because `contains_missing_within` has its
   own bounded front/tail and structural workload
   (`crates/rum-stdlib/src/associations/missing/traversal/deletion.rs:325-387`).
   One conversion removes target-driver native growth; it does not promise
   identical instruction/allocation profiles for different scan depths.
9. **Reasoned organization risk.** The current `missing.rs` implementation is
   in the 20–35k split-plan range, while `resume_frames.rs` is already in the
   warning range (verbatim size output below); adding the machine inline would
   worsen reviewability.  A `target.rs` responsibility boundary follows the
   repository's split policy (`docs/code-organization-guidelines.md:64-81`).

#### Verbatim source-audit output relied on above

No probe or test output exists for this design-only report.  The following is
the complete relevant output of the focused source-audit commands.

```text
$ rg -n '^(fn (delete_missing_at_target_level|resume_(suspended_)?delete_missing_(list|association)_(at_target_level|after_child|after_predicate)))|resume_(suspended_)?delete_missing_(list|association)_(at_target_level|after_child|after_predicate)\(' crates/rum-stdlib/src/associations/missing.rs
214:fn delete_missing_at_target_level(
221:        return resume_delete_missing_list_at_target_level(
235:    resume_delete_missing_association_at_target_level(session, entries, level, scan, 0, Vec::new())
239:fn resume_delete_missing_list_at_target_level(
256:        return resume_delete_missing_list_at_target_level(
267:        let child_result = resume_delete_missing_list_at_target_level(
276:            Ok(child_value) => resume_delete_missing_list_after_child(
302:        let child_result = resume_delete_missing_association_at_target_level(
311:            Ok(child_value) => resume_delete_missing_list_after_child(
337:    resume_delete_missing_list_at_target_level(session, items, level, scan, index + 1, kept)
340:fn resume_delete_missing_list_after_child(
350:    resume_delete_missing_list_at_target_level(session, items, level, scan, index + 1, kept)
353:fn resume_suspended_delete_missing_list_at_target_level(
372:        return resume_suspended_delete_missing_list_at_target_level(
383:        Some(resume_delete_missing_list_at_target_level(
393:            resume_delete_missing_association_at_target_level(
405:        return resume_suspended_delete_missing_list_at_target_level(
416:        Ok(value) => resume_suspended_delete_missing_list_after_child(
431:fn resume_suspended_delete_missing_list_after_child(
440:    resume_suspended_delete_missing_list_at_target_level(
450:fn resume_delete_missing_association_at_target_level(
463:        return resume_delete_missing_association_at_target_level(
480:            Ok(predicate_value) => resume_delete_missing_association_after_predicate(
515:        return resume_delete_missing_association_at_target_level(
526:        let child_result = resume_delete_missing_list_at_target_level(
535:            Ok(child_value) => resume_delete_missing_association_after_child(
562:        let child_result = resume_delete_missing_association_at_target_level(
571:            Ok(child_value) => resume_delete_missing_association_after_child(
598:    resume_delete_missing_association_at_target_level(
609:fn resume_delete_missing_association_after_predicate(
623:    resume_delete_missing_association_at_target_level(
634:fn resume_delete_missing_association_after_child(
646:    resume_delete_missing_association_at_target_level(
656:fn resume_suspended_delete_missing_association_at_target_level(
671:        return resume_suspended_delete_missing_association_at_target_level(
688:            Ok(value) => resume_suspended_delete_missing_association_after_predicate(
711:        return resume_suspended_delete_missing_association_at_target_level(
722:        Some(resume_delete_missing_list_at_target_level(
732:            resume_delete_missing_association_at_target_level(
744:        return resume_suspended_delete_missing_association_at_target_level(
755:        Ok(value) => resume_suspended_delete_missing_association_after_child(
771:fn resume_suspended_delete_missing_association_after_predicate(
787:    resume_suspended_delete_missing_association_at_target_level(
798:fn resume_suspended_delete_missing_association_after_child(
809:    resume_suspended_delete_missing_association_at_target_level(
```

```text
$ rg -n 'struct Resume|pub\(super\) (items|entries|level|scan|index|kept|entry):|resume_suspended_delete_missing_(list|association)_after_(child|predicate)' crates/rum-stdlib/src/associations/missing/resume_frames.rs
6:pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrame1 {
7:    pub(super) items:
9:    pub(super) level: usize,
10:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
11:    pub(super) index: usize,
12:    pub(super) kept:
36:                resume_suspended_delete_missing_list_after_child(
47:pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrame2 {
48:    pub(super) items:
50:    pub(super) level: usize,
51:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
52:    pub(super) index: usize,
53:    pub(super) kept:
77:                resume_suspended_delete_missing_list_after_child(
88:pub(super) struct ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrame {
89:    pub(super) items:
91:    pub(super) level: usize,
92:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
93:    pub(super) index: usize,
94:    pub(super) kept:
118:                resume_suspended_delete_missing_list_after_child(
129:pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1 {
130:    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
133:    pub(super) level: usize,
134:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
135:    pub(super) index: usize,
136:    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
139:    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
164:                resume_suspended_delete_missing_association_after_predicate(
175:pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame2 {
176:    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
179:    pub(super) level: usize,
180:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
181:    pub(super) index: usize,
182:    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
185:    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
210:                resume_suspended_delete_missing_association_after_child(
221:pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame3 {
222:    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
225:    pub(super) level: usize,
226:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
227:    pub(super) index: usize,
228:    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
231:    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
256:                resume_suspended_delete_missing_association_after_child(
267:pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame1 {
268:    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
271:    pub(super) level: usize,
272:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
273:    pub(super) index: usize,
274:    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
277:    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
302:                resume_suspended_delete_missing_association_after_predicate(
313:pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame2 {
314:    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
317:    pub(super) level: usize,
318:    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
319:    pub(super) index: usize,
320:    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
323:    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
348:                resume_suspended_delete_missing_association_after_child(
```

```text
$ rg -n 'index\s*\+\s*1' crates/rum-stdlib/src/associations/{aggregate.rs,basic.rs,key_reshape.rs,missing.rs} crates/rum-stdlib/src/associations/grouping/{association.rs,list.rs} crates/rum-stdlib/src/associations/resumable/{aggregate.rs,key_select.rs,map.rs} crates/rum-stdlib/src/associations/transforms/key_collections.rs
crates/rum-stdlib/src/associations/resumable/map.rs:129:        index + 1,
crates/rum-stdlib/src/associations/resumable/map.rs:217:        index + 1,
crates/rum-stdlib/src/associations/transforms/key_collections.rs:203:                        indices.as_ref().map(|indices| &indices[rest_index + 1]),
crates/rum-stdlib/src/associations/transforms/key_collections.rs:413:        index + 1,
crates/rum-stdlib/src/associations/resumable/key_select.rs:125:    resume_suspended_key_select_from_index(session, entries, criterion, index + 1, selected)
crates/rum-stdlib/src/associations/resumable/aggregate.rs:187:    resume_suspended_merge_from_index(session, combiner, grouped, index + 1, merged)
crates/rum-stdlib/src/associations/resumable/aggregate.rs:276:    resume_suspended_counts_by_from_index(session, function, items, index + 1, counts)
crates/rum-stdlib/src/associations/key_reshape.rs:231:        index + 1,
crates/rum-stdlib/src/associations/key_reshape.rs:325:    resume_suspended_key_group_by_reduce_from_index(session, reducer, grouped, index + 1, reduced)
crates/rum-stdlib/src/associations/basic.rs:116:    resume_suspended_association_key_evaluation(session, entries, evaluated_entries, index + 1)
crates/rum-stdlib/src/associations/aggregate.rs:157:            .map(|(index, item)| (item.clone(), Expr::integer(BigInt::from(index + 1))))
crates/rum-stdlib/src/associations/aggregate.rs:250:        index + 1,
crates/rum-stdlib/src/associations/aggregate.rs:341:        index + 1,
crates/rum-stdlib/src/associations/aggregate.rs:493:    resume_suspended_catenate_steps_from_index(session, call_args, steps, index + 1, flattened)
crates/rum-stdlib/src/associations/grouping/list.rs:234:        index + 1,
crates/rum-stdlib/src/associations/grouping/list.rs:266:    if spec_index + 1 < specs.len() {
crates/rum-stdlib/src/associations/grouping/list.rs:272:            spec_index + 1,
crates/rum-stdlib/src/associations/grouping/list.rs:419:        index + 1,
crates/rum-stdlib/src/associations/grouping/list.rs:519:        index + 1,
crates/rum-stdlib/src/associations/grouping/association.rs:256:        index + 1,
crates/rum-stdlib/src/associations/grouping/association.rs:288:    if spec_index + 1 < specs.len() {
crates/rum-stdlib/src/associations/grouping/association.rs:294:            spec_index + 1,
crates/rum-stdlib/src/associations/grouping/association.rs:436:        index + 1,
crates/rum-stdlib/src/associations/grouping/association.rs:532:        index + 1,
crates/rum-stdlib/src/associations/missing.rs:261:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:337:    resume_delete_missing_list_at_target_level(session, items, level, scan, index + 1, kept)
crates/rum-stdlib/src/associations/missing.rs:350:    resume_delete_missing_list_at_target_level(session, items, level, scan, index + 1, kept)
crates/rum-stdlib/src/associations/missing.rs:377:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:410:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:445:        index + 1,
crates/rum-stdlib/src/associations/missing.rs:468:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:520:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:603:        index + 1,
crates/rum-stdlib/src/associations/missing.rs:628:        index + 1,
crates/rum-stdlib/src/associations/missing.rs:651:        index + 1,
crates/rum-stdlib/src/associations/missing.rs:676:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:716:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:749:            index + 1,
crates/rum-stdlib/src/associations/missing.rs:792:        index + 1,
crates/rum-stdlib/src/associations/missing.rs:814:        index + 1,
```

```text
$ wc -c crates/rum-stdlib/src/associations/missing.rs crates/rum-stdlib/src/associations/missing/resume_frames.rs crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs
26353 crates/rum-stdlib/src/associations/missing.rs
13718 crates/rum-stdlib/src/associations/missing/resume_frames.rs
 7866 crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs
47937 total
```
