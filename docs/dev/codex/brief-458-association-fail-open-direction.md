# brief-458 — the class-wide fail-open direction (design-only)

## Goal

Six association operations now clear the ruled physical height 4,097 and
**terminate the process** above their own ceiling. Not one of them raises an
in-band error. Design the repair that restores a fail-closed outcome for the
class, without rewriting sites that already work, and make the resulting
guarantee **enforced rather than remembered**.

This is the largest open item in the association depth class and it outranks
every remaining individual site, because it is a property of all of them at
once.

**How the class got here.** Before brief-455, these descents failed in band by
*borrowing* a poll point: a fresh deep association result went through the
generic stability walk, and that walk polls. brief-455 replaced the walk's
association arm with a bounded front plus an explicit task stack. The borrowed
poll went with it. brief-456 saw this in one operation — `MergeNested` went from
failing in band at 1,153 wrappers to terminating the process at 2,027 — and the
2026-09-05 brackets show it is the whole class. **Removing a guard-adjacent cost
converted fail-closed into fail-open six times over.**

The contract is met with 1.5x-2x margin. The fail-closed promise is not, and
`SECURITY.md` is the document that has to stay true.

## What is given — do not re-measure, and do not probe past a ceiling

Director-measured 2026-09-05, release CLI at `80c8acce`, `ulimit -s 2048`, 1 MiB
evaluator budget. Heights are **physical expression levels**. A verdict is the
highest height passing 3/3 against the lowest failing 3/3.

| site | at 4,097 | ok <= | fail >= | above the ceiling |
|---|---|---|---|---|
| `DeleteMissing` | pass | 6,272 | 6,304 | **terminates the process** |
| `AssociationKeyDeflatten` | pass | 7,344 | 7,376 | terminates the process |
| `MapIndexed`, association shape | pass | 7,344 | 7,376 | terminates the process |
| `AssociateTo` / `KeyDropFrom`, `Part` target | pass | 8,048 | 8,080 | terminates the process |
| `AssociationKeyFlatten` | pass | 8,292 | 8,334 | terminates the process |
| `Merge` collection parser | pass | 16,072 | 16,128 | terminates the process |
| nested key-path assignment | pass | >= 10,800 | — | **unmeasured** (fixture-bound) |
| `Scan`, association shape | pass | >= 10,800 | — | unmeasured (fixture-bound) |
| `Map`, `MergeNested` (controls) | pass | >= 10,800 | — | unmeasured (fixture-bound) |

The last four are capped by the association fixture, which gives out around
5,455 wrappers; their direction above that is not known.

**A ceiling is not deterministic.** Inside a band a few tens of levels wide the
same height flips run to run, because the stack base moves between processes.
Do not treat any single number above as exact to better than about 50 levels.

## The constraint that shapes the whole design — verified, treat as given

1. **`poll_native_stack_limit` is private to the evaluator.** It is
   `fn poll_native_stack_limit(&mut self)` at
   `crates/rum-runtime/src/runtime/eval_core/mod.rs:68`, and the string occurs
   nowhere outside `crates/rum-runtime/src/runtime/eval_core/`. No stdlib code
   can call it today, and `crates/rum-stdlib/src/associations/` contains zero
   occurrences of it or of `evaluation_depth`.
2. **The only fail-closed precedent in `rum-stdlib` is a fixed depth counter** —
   `MAX_JSON_DEPTH`, `MAX_YAML_DEPTH`, `MAX_XML_DEPTH`, `MAX_TOML_DEPTH`, all
   256, from brief-422 (`crates/rum-stdlib/src/json.rs:11` and siblings). That
   is precisely the shape brief-443 measured and **rejected** for the stability
   walk: "a bare depth counter keeps the certificates but is not stack-size-
   honest". A counter tuned on release frames is wrong in debug, where
   brief-437 measured 20.6 KiB per user-recursion level against 7.5 KiB release.
3. **A seam is architecturally possible but is a decision, not a detail.**
   `rum-stdlib` depends on `rum-runtime` and already imports from
   `rum_runtime::kernel_api` (for example
   `crates/rum-stdlib/src/associations/missing.rs:1`). But brief-390 found that
   export surface permissive enough to forge a typed selector wrapper through
   it, so widening it needs an argument about what a caller can do with the new
   API, not only about what it is for.

## The three questions this brief exists to answer

### 1. What is the mechanism, per site?

Three candidate shapes. Argue each site to one of them, with `path:line`.

- **A local byte poll** — needs a seam per constraint 1. This line's taxonomy
  calls a local poll *safety only*: it converts an out-of-band edge into a
  resource outcome but cannot make work finish at a height it could not reach.
  **Here that is exactly the remaining gap**, because every site in the table
  already completes at 4,097. Safety-only is the whole job for once.
- **A fixed depth counter** — available today with no seam, and dishonest per
  constraint 2. Say plainly whether the class can tolerate that dishonesty, or
  cannot.
- **Conversion to a bounded front plus an explicit work stack** — what every
  closed site in this line used. brief-450's boundary says do not rewrite code
  that already works, and all six of these pass the contract.

**The sub-question I most want settled, from source.** When a site is converted
and becomes heap-bound, *what actually stops it?* For `Replace` and
`DeleteCases` the answer was the shared stability walk at roughly 5,800, which
polls. Post-455 the association tail polls too. So does conversion buy
fail-closed for free on this class as well — and if so, is the cheapest correct
answer for some of these six a conversion rather than a poll? Establish this
from the code path, not by analogy.

### 2. Where does the check go, and what does it cost?

Per site, name the exact recursing branch — the call that increases native
depth, not the function entry. Two measured precedents bound the design:

- brief-445 put the stability counter at the walk's entry, where it also ran for
  atoms and cache hits, and cost `list-literal-small` +7.99%. Moving it to the
  call-cache miss path, immediately before the recursion, removed that entirely
  (brief-446). **Put the check where the growth is.**
- brief-431 propagated exhaustion as a `Result` from every node and cost 6-15%
  on wide traversal normalization, because atoms returned through it too.

`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs` is shared with
`Map` and `Apply`, which are hot. Say which of the six sites sit on a hot path
and which do not; a guard on a cold association operation and a guard on the
`Map` machine are not the same decision.

### 3. How is the guarantee enforced rather than remembered?

**This is the part I care most about.** Six hand-placed guards, each correct on
the day it is written, is the "relies on people remembering" shape this project
replaces with structure — a fix the compiler or a derive enforces beats one that
does not, even at an order of magnitude more cost.

Propose a structure that makes a *new* unpolled association descent fail to
build, or fail a test, rather than ship. In-repository precedents to study and
say which shape fits:

- `SuspensionFrame: FrameRootTrace` — a missing traversal is a compile error
  (briefs 364-375).
- the definition-store sealing inventory test — a handwritten call outside the
  allowlist makes a count assertion fail with the exact number (brief-383).
- the builtin growth declarations — every builtin must declare, and the
  declaration is falsifiable (briefs 417-419).

Whatever you propose, **it ships with a proof it can fire.** A check nobody has
watched fail is not known to work: brief-419 caught five bad declarations the
day the falsifier existed, and every growth contract now carries a checked-in
`#[should_panic]` proof verified by misdeclaring a real catalog builtin. Name
the falsifier for your proposal explicitly.

## Also deliver

4. **The four fixture-bound sites.** `Map`, `MergeNested`, `Scan` and nested
   key-path assignment clear the fixture cap, so their direction above roughly
   10,800 physical levels is unknown. From source: can each still grow native
   stack per level above that height, or is it heap-bound? If any of them can,
   say how it would be certified given that the canonical fixture cannot build a
   value that deep.
5. **A class inventory beyond the six.** brief-457's citation refresh is current
   and accurate; reuse it. Name any association-side descent that is unpolled
   today and would be covered — or missed — by your proposed structure. A repair
   that closes the six sightings rather than the class is the gate miss
   brief-388 made and brief-394 had to clean up.
6. **Drafted implementation briefs** in `TEMPLATE.md` shape, one per step, each
   with its gate tests, its certificate shape (a unique deepest target, an
   assertion on the exact rebuilt value, a shallow companion pinning the
   selector), and its falsifier.

## Out of scope — named so you do not chase them

- **Deep `Expr` construction and drop.** Deep list construction terminates the
  process at 21,416 levels, far above the contract. This is standing engineering
  debt with a decision already on it: three iterative `Drop` designs regressed
  the hot path 20-50% and were rejected. Do not reopen it.
- **The four sites still SHORT of the contract** — multi-spec `GroupBy`, `Keys`
  and `KeyTake` over nested lists, generic-shape `MapIndexed`, and the schema
  validator. They are separate queue items with their own briefs. This brief is
  about sites that *pass* and fail open.
- The construction-ceiling attribution dispute brief-457 raised in its risk 5.

## Context pointers

- `AGENTS.md` first, including the Testing Policy hard lines.
- `TODO.md`, the section "The tail re-probed under the 1 MiB budget" and the
  2026-09-05 re-derivation table that follows it.
- `docs/dev/codex/brief-457-association-tail-rederivation.md` — the current
  citation refresh and per-site poll analysis. Its §1 table is accurate at
  `80c8acce`; do not redo it.
- `docs/dev/codex/brief-443-stability-failclosed-measurement.md` — three
  fail-closed shapes measured and all three rejected, including the bare depth
  counter. Read this before proposing any of them again.
- `docs/dev/codex/brief-445-stability-strided-poll.md` and
  `brief-446-stability-poll-placement.md` — where the check goes, and the
  +7.99% that placement cost.
- `crates/rum-runtime/src/runtime/eval_core/mod.rs:14-68` — the poll and the
  depth increment, and the visibility that keeps stdlib out.
- `crates/rum-stdlib/src/json.rs:11`, `yaml/decode.rs:374` — the fixed-counter
  precedent, and what a stdlib in-band depth refusal looks like today.

## Scope

- Files you MAY modify: **NONE — design-only.** Append your report to this file
  and change nothing else.
- **Do not run any probe that drives a recursion to or past its ceiling**, and
  do not write a reproduction whose purpose is to terminate a process. The
  brackets above are given. Sub-ceiling `/proc` VmStk high-water sampling at
  several heights is permitted and is the right instrument for a slope.
- If a claim cannot be settled without a past-ceiling measurement, say so and
  stop there. That is a correct outcome.

## rum / Rust pitfalls (read twice)

1. **Mark every claim measured or reasoned.** brief-457's reasoned arithmetic
   was within 2% on four operation-local native walks and 19x out, in the wrong
   direction, on the one site that crosses the evaluator per level. Reasoned
   packet arithmetic is reliable for a local native walk and unreliable the
   moment the descent can reach `Session::evaluate`.
2. **A per-level byte cost without its build profile is not a number**
   (brief-437: 4.7 KiB release strict, 7.5 KiB release user recursion, 20.6 KiB
   debug). Any counter threshold you propose has to survive all three.
3. **Logical wrappers are not physical height.** An association wrapper is
   `Association[Rule[...]]`, two physical levels. The ruled 4,097 is `N=2048`
   wrappers for the canonical fixture.
4. **No default-path cost.** Any diagnostic or counter goes behind the env-gated
   cold-load `AtomicU8` pattern; a branch paid on every call when disabled is a
   regression. A guard that must run is not a diagnostic — say which yours is.
5. **Wall-clock timing is not evidence on this host.** Gates use allocation
   counts (`RUM_PROFILE_ALLOC=1` on a `--features alloc-count` build) and
   instruction counts (`perf stat -e instructions:u`, minimum of the tight clean
   cluster, runs migrating between `cpu_atom` and `cpu_core` discarded), with
   **both arms built at the same filesystem path** — a `/tmp` worktree against a
   home-directory build reads 8% apart on untouched probes.
6. **Suspension safety.** Several of these sites are resumable and carry frame
   state (`crates/rum-stdlib/src/associations/missing/resume_frames.rs` and
   siblings). A guard that changes a descent must preserve the suspend/resume
   continuation shape.

## Verification

Design-only: no test gate. The evidence is the deliverable. Every claim carries
`path:line`; paste verbatim the output of any command you rely on.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any file except appending your report to this one. If you
  believe you must, STOP and write that in the report instead.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- Do not fabricate a citation. If you cite evidence it must be at the line you
  name.
- Paste real verbatim output. If something was not established, say so.

## Report format (your final message)

1. Design-only, no source change — state it.
2. Question 1: mechanism per site, including whether conversion buys fail-closed
   for free on this class.
3. Question 2: the exact recursing branch per site, hot-path classification, and
   the expected cost shape.
4. Question 3: the enforcement structure, and the falsifier that proves it fires.
5. Deliverable 4: the four fixture-bound sites.
6. Deliverable 5: the class inventory beyond the six, and what your structure
   would miss.
7. Deliverable 6: the drafted implementation briefs.
8. Risks and open questions, including anything you expect a director
   measurement to contradict.

---

## Appended design report — 2026-09-05

### 1. Design-only, no source change

**Reasoned.** This is a design report only. I changed no Rust, test, benchmark,
configuration, or other documentation file; this append is the sole workspace
change. I ran source-reading/search commands only, no tests, builds, network
access, recursion probes, or git-mutating commands. That is the required shape
for this brief (`docs/dev/codex/brief-458-association-fail-open-direction.md:191-245`;
`docs/dev/codex/TEMPLATE.md:19-25`).

**Measured (director-supplied, not re-measured).** The six target operations all
pass physical height 4,097 and all terminate the process above their supplied
ceilings; the four controls/unknowns are fixture-bound. I use those brackets as
given and do not refine them (`docs/dev/codex/brief-458-association-fail-open-direction.md:27-50`;
`TODO.md:1705-1723`).

In this report, **Measured** means a supplied or previously recorded
measurement. **Reasoned** includes a direct source observation or a conclusion
from cited source. A proposed artifact or acceptance condition is labelled
**Plan**, and is not presented as an established fact.

### 2. Question 1 — mechanism per site

#### Decision

**Reasoned.** Use a **local native-stack byte guard** for each of the six. Do
not use a fixed logical-depth refusal, and do not convert a passing operation
merely to obtain safety. The guard is mandatory correctness work, not optional
diagnostic instrumentation. It must be entered only by the selected handlers
and checked at the calls that increase native depth, so it adds no branch to the
general evaluator (`crates/rum-runtime/src/runtime/eval_core/mod.rs:14-64`), and
no branch at all to `Map`/`Apply`'s separate bounded transformer
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61,94-267`).

| Target | Mechanism | Why this mechanism |
|---|---|---|
| `DeleteMissing` | **Reasoned: local byte guard**, spanning memory projection, recursive and finite-target modes, scan helpers, and every resume re-entry. | All of those walks are native recursion (`crates/rum-stdlib/src/associations/missing.rs:84-230,308-329,356-1003,1038-1053`), while the operation already passes 4,097 (**Measured** at `docs/dev/codex/brief-458-association-fail-open-direction.md:33-40`). Rewriting its resumable semantics would add risk without lifting the required target (`crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-260`). |
| `AssociationKeyDeflatten` | **Reasoned: local byte guard** over insertion and rendering. | Its two native phases are separate and already complete at the target: path insertion is `insert_path`, then `render_nested_path_entries` (`crates/rum-stdlib/src/associations/key_reshape.rs:340-415`). |
| association-shaped `MapIndexed` | **Reasoned: local byte guard** only in the `MapIndexed` recursive machine. | The association child call is native recursion and the handler has a `Session` from which to acquire the guard (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-367`; `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:59-74`). `Map` and `Apply` use other functions (`crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:77-92,134-147`). |
| held `AssociateTo` / `KeyDropFrom` with a `Part` target | **Reasoned: local byte guard** through association-only guarded adapters for exact-path collect/read/rebuild. | The pure exact-path phases run after selector evaluation and before the store (`crates/rum-stdlib/src/associations/mutation.rs:522-560,588-692`). The raw helpers are shared by many non-association consumers, so changing their ordinary entry points would charge unrelated paths (`crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42,88-110,173-201`). |
| `AssociationKeyFlatten` | **Reasoned: local byte guard** over its value descent. | The sole proportional operation walk is the recursive association-value call (`crates/rum-stdlib/src/associations/key_reshape.rs:417-440`); its result is flat, so there is no deep post-result association envelope to borrow (`docs/dev/codex/brief-457-association-tail-rederivation.md:422-423`). |
| ordinary `Merge` collection parser | **Reasoned: local byte guard** over its nested-list edge. | The handler owns a `Session`, parsing finishes before the sibling-group loop, and only the parser follows collection depth (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-56,93-128`; `crates/rum-stdlib/src/associations/data.rs:163-189`). |

#### Why not a fixed counter

**Reasoned.** The class cannot tolerate a fixed refusal. A number calibrated to
release frames is not a stack bound in debug or under a host-lowered declared
stack. The rejected V3 was explicitly not stack-size-honest
(`docs/dev/codex/brief-443-stability-failclosed-measurement.md:69-77`), and the
current contract says a host entering beneath substantial caller frames must
lower the byte budget (`SECURITY.md:60-66`; `crates/rum-runtime/src/runtime/metadata.rs:37-53`).
Logical depth may throttle *when* to read the stack pointer; it must never make
the refusal decision.

#### The byte boundary and the seam

**Reasoned design.** The existing `native_stack_limit` is documented as an
**evaluator** budget, not the whole declared stack
(`crates/rum-runtime/src/runtime/model/config.rs:3-23`; `SECURITY.md:43-58`). A
local operation guard must therefore compare consumption from the request base
against

```text
operation safety boundary = native_stack_limit
                          + NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES
```

which is 1.5 MiB in the default 2 MiB profile. This does not raise either
published constant: the existing derivation subtracts 512 KiB poll headroom and
512 KiB non-evaluator reserve from the declared stack
(`crates/rum-runtime/src/runtime/metadata.rs:10-31,45-53`). If a host lowers
`native_stack_limit` for caller frames, the operation boundary lowers by the
same number. The implementation must use checked arithmetic and retain the
existing caller-declared-stack precondition (`SECURITY.md:60-66`).

**Reasoned safety condition.** A branded `NativeStackDepth<'scope>` starts at
zero. A recursive edge obtains its child token through the guard. The token
uses logical depth only as a stride and calls an actual-pointer
`#[cold] #[inline(never)]` check at the same four-level cadence as the evaluator;
the decision compares bytes from `native_stack_base`. Any
`Session::evaluate` reached while operation frames remain resident receives an
immediate, unstrided guard check first. That preserves the full evaluator poll
headroom at the crossing; the existing evaluator can otherwise advance four
levels between byte checks (`crates/rum-runtime/src/runtime/eval_core/mod.rs:34-43,66-87`;
`crates/rum-runtime/src/runtime/metadata.rs:11-24`). The per-operation
implementation gate must disassemble and measure both debug and release frame
slopes; if any local level can consume the 128 KiB per-level reserve, this
stride is invalid and the step stops (`crates/rum-runtime/src/runtime/metadata.rs:11-22`).

**Reasoned least-authority seam.** Add, under `#[doc(hidden)]`, only an opaque
scoped runner and branded child token to `rum_runtime::kernel_api`; do **not**
make `poll_native_stack_limit`, `native_stack_base`, a raw stack floor, or a
constructor public. The scope owns copied base/boundary values and a sticky
exhaustion bit; callers may ask for a checked child and supply a type-correct
fallback, but cannot inspect or increase the boundary. The runner checks the
sticky bit after the operation and gives `StackLimitExceeded` precedence over
the throwaway fallback. This avoids a `Result` return through every atom, the
shape that cost 6–15% in the supplied precedent
(`docs/dev/codex/brief-458-association-fail-open-direction.md:101-116`). The
kernel facade is semver-relevant, so this narrowness is an acceptance item, not
an implementation detail (`crates/rum-runtime/src/runtime/kernel_api.rs:1-6,67-99`).

**Reasoned suspension property.** The branded lifetime cannot escape the
runner, and neither guard nor token implements `FrameRootTrace`, `Clone`, or a
public constructor. A suspension therefore unwinds the native scope; the resume
method reacquires a fresh scope after the kernel resets `native_stack_base`
(`crates/rum-runtime/src/runtime/suspension/core.rs:482-509`). Attempting to put
the token in an `Arc`-owned suspension frame must fail to compile, while current
frames continue carrying only expression/state roots
(`crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-43,87-125,128-170`).

#### Does conversion buy fail-closed “for free”?

**Reasoned: no borrowed in-band stop, but complete conversion removes the
hazard.** The association stability tail is an explicit `Vec` task machine
(`crates/rum-runtime/src/runtime/eval_core/stability/association_tail.rs:14-43,72-99`).
Its poll is reached through `begin_association_call_cache_visit` and compares
the *current native pointer* with the evaluator byte budget
(`crates/rum-runtime/src/runtime/evaluation_cache/stability/association_front.rs:76-99,103-159`).
Once an operation's recursive frames have unwound, logical association depth
does not move that pointer, so the tail does not impose a heap-depth ceiling.
It cannot retroactively protect a still-recursive operation either. Thus a
complete conversion is safe because proportional native growth is gone; it
does not get an in-band `StackLimitExceeded` merely because the downstream tail
contains a poll. `Map` and `MergeNested` demonstrate the bounded-front/work-list
shape and simply continue until a different resource/fixture boundary
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61,70-267`;
`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:3-13,48-194`).

**Reasoned decision.** None of the six should be converted. Every one already
meets the height contract (**Measured**, brief table), conversion changes more
ordering/ownership/suspension code than a call-edge guard, and conversion does
not supply the imagined downstream error. The one-line `Merge` recursion is
also cheaper to guard once the common seam exists than to introduce a second
parser work machine (`crates/rum-stdlib/src/associations/data.rs:173-189`).

### 3. Question 2 — exact growth sites and cost

#### Required call-edge checks

| Site/phase | Exact native-depth-increasing call(s) | Path class and expected cost |
|---|---|---|
| `DeleteMissing`, recursive association fixture | `delete_missing_from_keyed_data_recursive` from list item and association value at `crates/rum-stdlib/src/associations/missing.rs:965-968,985-992`. | **Reasoned cold operation path.** One token child per actual recursive edge; pointer check only at the stride. |
| `DeleteMissing`, finite/resumable target | Child/sibling transitions at `crates/rum-stdlib/src/associations/missing.rs:368-455,470-565,575-723,773-934`; the canonical association child is `:678-686`, and its suspended twin is `:848-856`. | **Reasoned cold but suspension-sensitive.** Existing `Result`/continuation flow stays intact; resume reacquires a scope. Calls to `Session::evaluate` at `:590-620,798-817` get an immediate pre-crossing check. |
| `DeleteMissing`, preflight/scan | Recursive count edges at `crates/rum-stdlib/src/associations/missing.rs:119-210`, expression projection at `:212-230`, and `contains_missing_within` at `:1038-1053`. | **Reasoned cold.** The projection runs only when a memory limit is active because `:84-103` returns early otherwise; all still enter the structural inventory. |
| `AssociationKeyDeflatten`, insert | Existing-branch and new-branch calls at `crates/rum-stdlib/src/associations/key_reshape.rs:379-388`. | **Reasoned cold.** Token branch per path component; no allocation beyond existing node construction. |
| `AssociationKeyDeflatten`, render | `render_nested_path_entries(children)` at `crates/rum-stdlib/src/associations/key_reshape.rs:392-407`. | **Reasoned cold.** Separate native phase and separate certificate/falsifier. |
| association `MapIndexed` | Association value call at `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:329-342`. | **Reasoned hot-family site.** Restrict token plumbing to `MapIndexed`; `Map`/`Apply` must have byte-identical allocation counts and unchanged instructions within spread. The generic calls at `:369-409` remain the separately queued generic-shape axis unless its own brief elects to use the same token. |
| held association `Part` mutation, plan/read/rebuild | Selector expansion recursion at `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:88-110`, read at `:35-42`, head/argument rebuild at `:173-201`; held call sites are `crates/rum-stdlib/src/associations/mutation.rs:588-603,652-680`. | **Reasoned cold mutator over shared hot helpers.** Add guarded adapters and leave ordinary exports unchanged. A key adds two raw components at `exact_paths.rs:112-126`. |
| `AssociationKeyFlatten` | `flatten_association_entries(path, children)` at `crates/rum-stdlib/src/associations/key_reshape.rs:417-430`. | **Reasoned cold.** One checked child per nested association value. |
| `Merge` parser | `parse_merge_keyed_entries_collection_with_diagnostics(item)` at `crates/rum-stdlib/src/associations/data.rs:173-187`. | **Reasoned cold parser.** Shallow collections pay child-token arithmetic but no heap allocation or pointer check before the stride. |
| fixture-bound `Scan` | Association call at `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:429-436`; standard head/arg calls at `:454-477`. | **Reasoned hot-family site.** Guard the association axis now; guard generic `Scan` too because it is the same passing collector, but do not claim this lifts generic `MapIndexed`'s separate result ceiling (`docs/dev/codex/brief-457-association-tail-rederivation.md:505-527`). |
| fixture-bound nested key assignment | `set_association_key_path` at `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:976-1003`, specifically `:996`. | **Reasoned mutation path.** Selector evaluation is already complete before the recursion and storage follows it at `:771-807,842-878`; finalize the guard before committing. |

**Reasoned cost shape.** Every protected edge pays an opaque-depth increment,
mask comparison, and predicted-not-exhausted branch. Every fourth *resident
depth* pays the cold pointer check; wide siblings at one shallow depth do not
accumulate the token depth. The guard is stack-local, performs no allocation or
locking, and adds no `Session` field. This follows the successful “counter is a
throttle, bytes make the decision” precedent while moving the check from entry
to the growth edge (`docs/dev/codex/brief-445-stability-strided-poll.md:36-77`;
`docs/dev/codex/brief-446-stability-poll-placement.md:21-48`). It is mandatory
runtime work, so it is not hidden behind a profiling flag; only measurement
counters used to inspect it may use the env-gated `AtomicU8` pattern
(`docs/runtime-evaluation-audit.md:28-40`).

**Measured risk basis.** The local release packets range from 128 B for the
`Merge` parser through 816 B for finite-target `DeleteMissing`; these are
measurements, not thresholds (`docs/dev/codex/brief-457-association-tail-rederivation.md:441-462`).
Adding a token parameter can change those packets. Each implementation brief
therefore gates prologue and `/proc` slope in debug and release before accepting
the 4,097 certificate; no arithmetic projection substitutes for that gate.

### 4. Question 3 — enforcement, not memory

#### Three mutually reinforcing layers

1. **Plan — compile-time recursive-group wrapper.** Add a
   `guarded_native_recursion! { ... }` procedural macro in `rum-derive`. It
   parses all functions in one direct/mutual-recursion group, constructs the
   local call graph, and rejects every SCC edge not syntactically wrapped in
   `native_stack_descent!(token, fallback => child_token, call)`; it emits the
   functions unchanged after validation. The derive crate already uses `syn`
   visitors and converts validation errors into compile errors
   (`crates/rum-derive/src/lib.rs:3-11,45-50,85-108`). Adding an unguarded call
   to a migrated group therefore fails the build.
2. **Plan — exhaustive source/call-site inventory.** Add a `syn`-based test that
   scans all production Rust in `rum-runtime` and `rum-stdlib`, computes direct
   self/mutual-recursion SCCs, and compares exact `(path, caller, callee,
   occurrence-count, disposition/site-id)` rows. `GuardedNative` rows must live
   inside the compile-time group macro; `BoundedHeap` rows name their front
   constant and task loop; `EvaluatorFailClosed` and `DeferredShort(<brief>)`
   are explicit, not silent exemptions. The definition-store test demonstrates
   repository source collection and exact call-count inventories
   (`crates/rum-runtime/src/runtime/tests/definition_store_sealing.rs:4-50,158-225`),
   and the frame-root audit demonstrates a sorted, exhaustive derived inventory
   (`crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:313-367`).
3. **Plan — exhaustive association-surface declaration.** Add a sorted
   association-depth registry, separate from `BuiltinSpec`, covering every name
   in `ASSOCIATION_CORE_BUILTINS` and `ASSOCIATION_TRANSFORM_BUILTINS`, plus
   `Map`, `Apply`, `MapIndexed`, `Scan`, `Set`, and keyed `Part` axes. Each row
   declares `NoProportionalDescent`, `GuardedNative(site ids)`, `BoundedHeap`,
   `EvaluatorFailClosed`, or `DeferredShort(brief id)`. A new association
   catalog entry without a row fails exact equality, mirroring the builtin
   growth inventory (`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:3-9,199-215`;
   association catalogs at `crates/rum-stdlib/src/catalog/associations.rs:8-289`
   and `crates/rum-stdlib/src/catalog/associations_transform.rs:10-368`). This
   is test-only metadata, so dispatch and `BuiltinSpec` gain no default-path
   field (`crates/rum-runtime/src/runtime/metadata.rs:374-403`).

**Reasoned guarantee boundary.** A new recursive edge inside an existing group
fails macro expansion; a new recursive group under the audited roots fails the
SCC inventory; a new association builtin or axis fails the surface registry.
This replaces six remembered guards with a closed declaration. It is not a
formal whole-Rust call-graph proof: indirect function-pointer calls, trait
dispatch, recursion generated outside the scanned macro, and code outside the
declared association surfaces require an explicit escape row and review. Those
limits are reported again in section 6.

#### Falsifiers/proof that the checks fire

**Plan.** Ship all four; none is optional.

- A `rum-derive` unit test feeds the macro one directly recursive and one
  mutually recursive unwrapped edge and asserts the exact compile-error text;
  a wrapped twin expands. Existing compile-fail documentation establishes the
  crate's pattern (`crates/rum-derive/src/lib.rs:13-44`).
- A checked-in `#[should_panic]` registry test temporarily presents the real
  `AssociationKeyFlatten` row as `NoProportionalDescent`; validation must report
  its real edge at `crates/rum-stdlib/src/associations/key_reshape.rs:429`. This
  follows the real-catalog falsifier standard at
  `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:253-319`.
- A test-only runtime feature supplies an effective operation-boundary override
  of zero without adding a normal-build `Session` field or branch. Each migrated
  site gets a modest-depth, unique-path test that must return
  `StackLimitExceeded`; no test approaches a supplied ceiling. The underlying
  arithmetic helper also receives synthetic base/current-address unit tests.
- During each implementation brief, remove the wrapper from that brief's
  binding real edge, run its focused low-bound test and capture red, restore it,
  and capture green. If removal does not turn the test red, the certificate is
  rejected before any performance work. This is the same red/green standard as
  the prior stability guard (`docs/dev/codex/brief-446-stability-poll-placement.md:88-97`).

**Reasoned suspension enforcement.** The branded guard cannot satisfy a
`'static` suspension-frame field, and it intentionally lacks `FrameRootTrace`;
that makes accidental persistence a compile error rather than an allowlist
convention. Resume entry points reacquire a new scope, consistent with the base
reset at `crates/rum-runtime/src/runtime/suspension/core.rs:482-509` and the
existing `SuspensionFrame: FrameRootTrace` enforcement pattern
(`crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:313-367`).

### 5. Deliverable 4 — the four fixture-bound sites

| Site | Source direction above the fixture | Certification without a deeper association fixture |
|---|---|---|
| `Map` | **Reasoned heap-bound on the association-value axis.** Native recursion is capped by `RECURSIVE_TRANSFORM_CALL_DEPTH = 16`; exhaustion transfers to `TransformWork` and a `while let Some(task)` loop (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61,70-106`). | **Plan.** Keep the existing 4,097 exact-output certificate (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:243-264`), add a small test-only front limit of zero/one that forces the heap tail on a three-level branched value, and assert the exact rebuilt value and unique selected leaf. The static registry must bind `Map` to that constant, task enum, and loop. This is a structural proof that native depth is constant; no probe above the fixture is needed. |
| `MergeNested` | **Reasoned heap-bound on nested association groups.** Its native front is capped at 16, depth zero calls the iterative machine, and that machine drains `MergeNestedWork` in a loop (`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:3-13,88-139`). Resume restores heap work, not native parents (`:165-202`). | **Plan.** Keep the existing exact 4,097 chain certificate (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:294-335`), retain the iterative-tail suspension/no-replay test (`crates/rum-stdlib/src/tests/associations/associations_key_reshape.rs:195-223`), and force the front to zero in a shallow test with left/deep/right sentinels. The registry binds the bounded declaration to the constant and `while` loop. |
| association-shaped `Scan` | **Reasoned native-stack-growing.** The collector recursively enters every association value before evaluating the collected flat vector (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:411-487`; `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131`). No source bound appears above the fixture. | **Plan.** Install the local guard at `traversal.rs:435` (and the standard collector edges at `:470,476`). Keep a 4,097 certificate whose unique deepest sentinel appends exactly once to an iteratively inspected side-effect log; a three-level companion pins level specification, association position, and order. A modest-depth test with the test-only zero operation boundary must fail in band. Removing the `:435` wrapper must make that test red. |
| nested key-path assignment | **Reasoned native-stack-growing.** `set_association_key_path` calls itself for every selector (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:976-1003`); only after it returns does the symbol store occur (`:842-878,881-919`). No source bound appears above the fixture. | **Plan.** Guard `:996` and finish the scope before either store. Retain the 4,097 exact post-write certificate (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:266-292`), add an iterative unique-deepest-sentinel assertion plus shallow bare-string/`Key[...]` selector companions, and use the zero-bound modest-depth falsifier. No higher fixture is required. |

**Measured.** All four currently pass to the fixture cap supplied by the
director; only the source structure settles their direction beyond it
(`docs/dev/codex/brief-458-association-fail-open-direction.md:41-46`). For the
two heap-bound paths the proof obligation is “bounded front really transfers to
a non-recursive loop”; for the two native paths it is “the actual byte guard can
be forced to refuse.” Neither obligation benefits from a destructive
past-ceiling run.

### 6. Deliverable 5 — class inventory beyond the six

#### Association-value and result axes

| Surface/axis | Current source class | Proposed enforced disposition |
|---|---|---|
| `Map`, `Apply` | **Reasoned:** bounded front and explicit work stack (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6-61,70-267`). | `BoundedHeap`; covered by the registry and forced-tail test. |
| keyed `Part` read | **Reasoned:** iterative key loop (`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:523-541`). | `NoProportionalDescent`; covered by the registry. |
| ordinary `Merge` over association-valued entries | **Reasoned:** groups and iterates; it does not descend values (`crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-56,93-128`). | `NoProportionalDescent`; its distinct nested collection parser is `GuardedNative`. |
| `KeyMap`, `KeySort` | **Reasoned:** top-level entry/key work only (`crates/rum-stdlib/src/associations/transforms.rs:10-46,86-125`). | `NoProportionalDescent`. |
| `AssociationMap`, `KeyGroupBy`, `Counts`, `Catenate`, key-set operations, `JoinAcross` | **Reasoned:** index/loop/resumable drivers do not recurse through association-valued children (`docs/dev/codex/brief-457-association-tail-rederivation.md:427-428`). | `NoProportionalDescent`, with exact handler inventory. |
| `MergeNested` | **Reasoned:** bounded 16-level front plus heap work (`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:3-13,48-194`). | `BoundedHeap`. |
| `Scan`, nested association `Set` | **Reasoned:** still-native fixture-bound walks, detailed in section 5. | `GuardedNative` in the functional and assignment steps. |
| `ConformSchema` / `SchemaErrors` | **Measured:** short but already fail closed at 208/224 physical levels (`TODO.md:1708-1714`); source crosses evaluation during validation (`crates/rum-stdlib/src/schema.rs:177-330`). | `EvaluatorFailClosed("schema-depth queue")`; detected and declared, not changed by this brief. |
| multi-spec `GroupBy` | **Measured:** short and fail open (`TODO.md:1708-1712`); native grouping recursion is in `crates/rum-stdlib/src/associations/grouping/list.rs:257-347` and `association.rs:279-364`. | `DeferredShort("GroupBy queue")`; an explicit red registry row, not a safe classification. Out of scope here (`docs/dev/codex/brief-458-association-fail-open-direction.md:160-169`). |
| generic-shape `MapIndexed` | **Measured:** short but fail closed on its active-result path (`TODO.md:1710-1714`); the extra standard-call recursion is `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:369-409`. | `DeferredShort("brief-439 draft 3")`. The association edge is guarded, but this declaration prevents claiming the generic axis was lifted. |

#### Other association-owned recursive input shapes

**Reasoned.** The current complete inventory also found recursion on collection,
selector, and wrapper axes rather than canonical association-value nesting
(`docs/dev/codex/brief-454-association-depth-class.md:319-339`):

- the stdlib/runtime deep association parsers
  (`crates/rum-stdlib/src/associations/data.rs:234-260`;
  `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:95-123,279-305`);
- nested `Keys`/`Values` lists and `KeyTake`/`KeyDrop` lists
  (`crates/rum-stdlib/src/associations/basic.rs:752-810,818-915`);
- nested `Lookup` key specifications (`crates/rum-stdlib/src/associations/basic.rs:699-721`);
- nested key-collection option lists
  (`crates/rum-stdlib/src/associations/transforms/key_collections.rs:488-517`);
- `Normal` below ordinary wrappers
  (`crates/rum-stdlib/src/associations/basic.rs:549-560`).

**Reasoned coverage.** The SCC/source inventory sees every direct recursive edge
above because it scans the whole two production crates, not only the six files.
`Keys` and `KeyTake` are explicit `DeferredShort` rows because they are named
out of scope and already measured fail open (`TODO.md:1710-1715`;
`docs/dev/codex/brief-458-association-fail-open-direction.md:160-169`). The
other passing/unpriced rows must be `GuardedNative` before the registry is
allowed to call the passing class closed; a final residual-shapes brief is
drafted below. `KeyDrop` shares its helper with short `KeyTake`, so that helper
remains in the deferred collection-shape brief rather than receiving a guard
that would preserve the shortfall (`crates/rum-stdlib/src/associations/basic.rs:818-915`).

**Reasoned parser exception/ripple.** The runtime association parser is a
broad pure API: the source search below finds 116 calls to the two public parser
entry points. A local guard cannot be smuggled into it through ambient/TLS state
without creating re-entrancy and suspension ambiguity, and exposing a raw floor
would violate the least-authority seam. The residual brief therefore requires a
guarded parser entry point/token to be threaded only through Session-evaluation
roots, leaves the current unguarded helper for explicitly trusted internal
callers during migration, and makes every remaining unguarded call an exact
allowlist row. If the measured ripple touches hot callers beyond spread, the
brief stops for a director choice between finishing that token migration and a
bounded-front parser conversion; it does not silently choose conversion against
this brief's “do not rewrite passing sites” boundary
(`crates/rum-runtime/src/runtime/kernel_api.rs:29-34`;
`docs/dev/codex/brief-458-association-fail-open-direction.md:89-99`).

**Reasoned misses.** The structure deliberately does not claim to detect:

- indirect recursion through function pointers or trait/dynamic dispatch unless
  the call is registered as an escape;
- recursion in downstream crates outside the workspace source scan;
- deep `Expr` construction/drop, explicitly out of scope
  (`docs/dev/codex/brief-458-association-fail-open-direction.md:160-165`);
- the known short `GroupBy`, `Keys`, `KeyTake`, generic `MapIndexed`, and schema
  work before their named briefs complete.

Those are visible registry dispositions, not green/safe rows. Any new direct
recursive SCC, new association catalog entry, or changed occurrence count in
the in-repository class fails a build/test as described in section 4.

### 7. Deliverable 6 — drafted implementation briefs

**Plan.** Dispatch in the order below. Each runtime step flips its exact
registry row from `GuardPending` to `GuardedNative`; the final residual step
must leave no `GuardPending` row. `DeferredShort`/`EvaluatorFailClosed` rows stay
visibly non-green and belong to the separately named queue. The drafts are raw
material for director review, as required by the template
(`docs/dev/codex/TEMPLATE.md:32-37`).

#### Draft A — sealed native-descent guard, compile-time group validator, and registry

##### Goal

Add the least-authority, stack-size-honest guard substrate and the exhaustive
association-depth enforcement structure. No production operation uses it yet;
the change proves the mechanism and installs all benchmark labels needed by
the later one-variable steps.

##### Context pointers

- Read `AGENTS.md`, this report, and the rejected/accepted poll designs at
  `docs/dev/codex/brief-443-stability-failclosed-measurement.md:19-77` and
  `brief-445-stability-strided-poll.md:25-81`.
- The current private poll is `crates/rum-runtime/src/runtime/eval_core/mod.rs:66-87`;
  budget derivation is `crates/rum-runtime/src/runtime/metadata.rs:10-53`.
- The kernel facade boundary is
  `crates/rum-runtime/src/runtime/kernel_api.rs:1-6,67-99`; derive validation
  precedent is `crates/rum-derive/src/lib.rs:3-50,85-108`.
- Exact source and declaration inventory precedents are
  `crates/rum-runtime/src/runtime/tests/definition_store_sealing.rs:4-50,158-225`
  and `runtime/tests/memory_growth_contract.rs:3-9,199-215,253-319`.

##### Scope

- Files MAY modify: a new focused runtime native-stack-guard module;
  `crates/rum-runtime/src/runtime/{mod.rs,kernel_api.rs,metadata.rs}` and focused
  runtime tests; `crates/rum-runtime/Cargo.toml` only for a test feature;
  `crates/rum-derive/src/lib.rs` plus a new focused validator module/tests;
  a new `crates/rum-stdlib/src/tests/association_native_depth_contract.rs` and
  its test-module wiring; association/functional catalog files only as needed
  to expose names to the test; `scripts/core-language-microbench.sh` only to add
  the exact labels named below.
- MUST NOT modify operation implementations, public resource-limit values,
  `Expr`, unrelated docs, or `Cargo.lock` unless an actually new dependency is
  unavoidable. `syn` already belongs to `rum-derive`
  (`crates/rum-derive/Cargo.toml:16`).
- The normal `Session` layout and general evaluator path must not change.

##### Deliverable

- An opaque HRTB-scoped runner, branded `NativeStackDepth<'scope>`, sticky
  exhaustion state, cold actual-byte check, and
  `native_stack_descent!` wrapper. No public constructor, raw address, raw
  limit, or method that can relax the boundary.
- Operation boundary derived by checked addition of
  `native_stack_limit + NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES`; an immediate
  pre-evaluator-crossing check; `StackLimitExceeded` on finalization.
- `guarded_native_recursion!` validates every direct/mutual SCC call in its
  group at macro expansion. Add exact unwrapped/wrapped validator tests.
- A sorted exhaustive surface registry and sorted exact SCC/call-site
  inventory. Existing passing sites begin as `GuardPending(<draft>)`; known
  short sites use `DeferredShort(<brief>)`, never a green label.
- Test-only effective-boundary override behind a test feature, absent from the
  normal `Session` layout/build.
- Synthetic certificate: on a 2 MiB release thread, a guarded minimal recursive
  copier rebuilds a physical-height-4,097 one-child tree containing one unique
  deepest sentinel; inspect exact structure iteratively. A three-level branched
  companion proves only the selected child is rebuilt. At modest depth, the
  zero-bound override returns `StackLimitExceeded`.
- Add benchmark labels, with no operation changes yet:
  `delete-missing-association-small`, `association-key-flatten-small`,
  `association-key-deflatten-small`, `mapindexed-association-small`,
  `scan-association-small`, `association-set-key-path-small`,
  `associate-to-part-small`, `key-drop-from-part-small`, and
  `merge-collection-small`, plus `map-small`, `apply-small`, and
  `mapindexed-generic-small` controls. Establish their baseline allocations and
  `instructions:u` now so later arms use the identical harness.

##### rum / Rust pitfalls (read twice)

1. The depth token throttles pointer reads; it never refuses by depth.
2. The cold address-taking helper must return before the recursive call so its
   frame is not resident (`crates/rum-runtime/src/runtime/eval_core/mod.rs:66-78`).
3. Do not implement ambient TLS, panic unwinding, `unsafe`, or a public raw poll.
4. A token must not be storable in a `SuspensionFrame`; add a compile-fail proof.
5. Synthetic deep assertions must be iterative, not `PartialEq`/`Display`
   (`docs/dev/codex/brief-454-association-depth-class.md:1141-1145`).

##### Verification (MANDATORY)

Run bounded `cargo fmt --check`, `cargo test -p rum-derive`,
`cargo test -p rum-runtime`, `cargo test -p rum-stdlib`, and
`cargo test --workspace`. Run the focused synthetic certificate with
`--release --ignored --exact` on its declared 2 MiB thread. Run the registry's
real-catalog `#[should_panic]` falsifier. Temporarily unwrap the synthetic real
edge, capture the focused test red, restore it, and capture green.

Build base and candidate at the same filesystem path. With profiling disabled,
run the core microbench A/B for `list-literal-small`,
`association-small-literal-repeat`, and the twelve new dormant labels; allocation rows
must be byte-identical and `instructions:u` must remain inside the clean
same-binary spread. Any long command uses `scripts/run-bounded.py` with a
timeout derived from the last same-shape run. Paste all decisive output
verbatim.

##### Hard rules

No git mutation, network, out-of-scope edit, `unsafe`, oracle testing, or probe
at/past any supplied ceiling. Do not change the 1 MiB evaluator budget or either
512 KiB reserve. No normal-build `Session` field, allocation, lock, TLS access,
or general-evaluator branch. Stop if the 4,097 synthetic certificate fails, the
token can enter a suspension frame, the falsifier does not fire, or a control
regresses beyond spread.

##### Report format

Per-file changes; opaque API/authority argument; exact operation-boundary
arithmetic; compile-fail and red/green falsifiers; registry rows; certificate
height/sentinel/exact value; base/new allocation and instruction rows; all test
outputs; stop-loss/open questions.

#### Draft B — guard every `DeleteMissing` native descent and resume entry

##### Goal

Make every `DeleteMissing` native recursive phase fail in band without changing
its modes, evaluation order, suspension continuation, diagnostics, or exact
result at physical height 4,097.

##### Context pointers

- Handler/preflight: `crates/rum-stdlib/src/associations/missing.rs:19-103`.
- Recursive edges: `missing.rs:119-230,356-1003,1038-1053`.
- Suspension frames: `crates/rum-stdlib/src/associations/missing/resume_frames.rs:5-260`.
- The finite-target evaluator crossings are `missing.rs:590-620,798-817`.

##### Scope

- Files MAY modify: `crates/rum-stdlib/src/associations/missing.rs`, its
  `missing/resume_frames.rs` only if wrapper entry names require it, focused
  missing tests, iterative depth certificates, and the association-depth
  registry/inventory.
- MUST NOT change builtin semantics/messages, memory projector arithmetic,
  persistent suspension containers, the guard substrate, or unrelated files.
- `missing.rs` is in the 20–35k split-plan range; report why the safety patch
  keeps it intact, with the future boundary `missing/{preflight,traversal}.rs`,
  or perform only that responsibility split
  (`docs/code-organization-guidelines.md:64-81`).

##### Deliverable

- Put all directly/mutually recursive functions in validated groups and wrap
  every count, projection, scan, child, sibling-tail, after-child, and resumed
  recursive edge. Acquire one scope after argument normalization and before the
  armed preflight; reacquire at every resume driver. Never store the token.
- Immediately recheck before the delayed-value `Session::evaluate` calls while
  native parents are resident.
- Give a refused edge a type-correct throwaway result; scoped finalization must
  override it with `StackLimitExceeded`. Do not propagate a new `Result` through
  pure recursive nodes.
- Flip `DeleteMissing` to `GuardedNative` with exact site IDs/counts.
- Certificates on a declared 2 MiB release thread: (1) recursive association
  mode at physical height 4,097, one unique deepest retained sentinel and one
  distinct deepest `Missing` removed; (2) finite-target mode exercising the
  816-byte driver; (3) an armed-memory preflight path. Inspect raw height and
  exact rebuilt association iteratively. Shallow companions pin `Infinity`, a
  finite level/scan selector, `RuleDelayed` evaluation count, order, and message
  attribution.
- Falsifiers: zero-bound modest-depth tests for the recursive edge at
  `missing.rs:988`, finite association edge at `:679`, scan at `:1052`, and an
  armed preflight edge. Temporarily unwrap `:679`; its focused test must turn
  red, then green after restoration.

##### rum / Rust pitfalls (read twice)

Preserve first-invalid behavior and delayed evaluation exactly. Stack-limit
finalization outranks a partial fallback or suspension. Resume starts from a
new request-base scope; it must not replay already completed predicates. Do not
assert deep values through recursive formatting/equality.

##### Verification (MANDATORY)

Run bounded fmt, `cargo test -p rum-stdlib` with all focused missing/suspension
tests, `cargo test -p rum-runtime`, workspace tests, then the three exact release
certificates. Run the zero-bound tests and capture the real-edge removal
red/green proof. Compare base/new allocation and `instructions:u` for
`delete-missing-association-small`, shallow invalid/delayed companions,
`association-small-literal-repeat`, and same-tier `deletecases-*`; run the mandated
core-language microbench A/B at the same filesystem path. Paste exact output.

##### Hard rules

Standard template rules plus: no ceiling probes, no mode conversion/work-stack
rewrite, no guard in a frame, no message/order drift, no raising the boundary,
and no unrelated file split. Stop if any 4,097 certificate fails, suspension
replays, the falsifier stays green when unwrapped, or a label regresses beyond
spread.

##### Report format

Exact guarded edge inventory/counts; resume acquisition points; file-size split
assessment; certificates and shallow selectors; red/green falsifier; allocation,
instruction, microbench, and test output; open questions.

#### Draft C — guard both association key-reshape directions

##### Goal

Make `AssociationKeyDeflatten` insertion/rendering and
`AssociationKeyFlatten` value descent fail in band while preserving exact key
paths, entry order, delayed heads, conflicts, and the 4,097-height results.

##### Context pointers

- Handlers: `crates/rum-stdlib/src/associations/key_reshape.rs:105-143`.
- Deflatten phases: `key_reshape.rs:340-415`; flatten: `:417-440`.
- Current surface tests:
  `crates/rum-stdlib/src/tests/associations/associations_key_reshape.rs`.

##### Scope

- Files MAY modify: `key_reshape.rs`, its focused tests, iterative depth
  certificates, and registry/inventory rows.
- MUST NOT alter `NestedPathNodes`, grouping/`KeyGroupBy`, render
  canonicalization, the common guard, or other association families.
- `key_reshape.rs` is in the 10–20k warning range; report the review outcome
  (`docs/code-organization-guidelines.md:64-81`).

##### Deliverable

- Use the handler's currently unused `Session` to enter a scope. Validate and
  guard both `insert_path` calls at `key_reshape.rs:383,387`, render descent at
  `:406`, and flatten descent at `:429`. Give insertion and rendering distinct
  site IDs because their native phases do not overlap.
- Flip both catalog surfaces to `GuardedNative` with exact edge counts.
- Deflatten certificate: a 2,048-component flat list key produces physical
  height 4,097, one unique deepest sentinel, and the exact association chain;
  inspect iteratively. Flatten certificate: a 2,048-wrapper association with
  one deepest sentinel produces exactly one flat entry whose key is the exact
  2,048-element path and whose value/head is exact. Shallow companions pin
  prefix conflicts, duplicate ordering, mixed key lengths, and `RuleDelayed`.
- Zero-bound modest-depth tests bind separately to existing-branch insertion,
  new-branch insertion, render, and flatten. Temporarily unwrap `:429`, capture
  red, restore and capture green.

##### rum / Rust pitfalls (read twice)

The two deflatten phases need one scope but distinct token roots after the first
phase unwinds; do not carry a fictitious accumulated logical depth between
them. A fallback must never be mistaken for an invalid/conflicting key because
finalization takes precedence. Avoid recursive `Display`/`PartialEq`.

##### Verification (MANDATORY)

Run bounded fmt, focused key-reshape tests, `cargo test -p rum-stdlib`, runtime,
workspace, and both release certificates. Run all zero-bound tests and the real
edge red/green proof. Gate allocations, `instructions:u`, and the core
microbench rows for `association-key-{flatten,deflatten}-small`,
`association-small-literal-repeat`, and the nearest same-tier association
controls at the same path. Paste exact output.

##### Hard rules

No git/network/unsafe/oracle/ceiling probe; no conversion to a work stack; no
conflict, order, delayed-head, or diagnostic drift; no extra shallow allocation;
stop on certificate, falsifier, or performance failure.

##### Report format

Per-phase guarded edges; exact deep/shallow values; conflict/delayed proofs;
red/green output; allocation/instruction/microbench rows; tests and stop-loss.

#### Draft D — guard association `MapIndexed` and the passing `Scan` collector

##### Goal

Make association-value `MapIndexed` and `Scan` collection fail closed while
leaving the already bounded `Map`/`Apply` machines untouched and making no claim
that generic `MapIndexed`'s separate post-result shortfall is fixed.

##### Context pointers

- Handlers: `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:59-92,113-147`.
- Recursive functions/edges:
  `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:301-409,411-487`.
- Bounded `Map`/`Apply`: `traversal.rs:6-61,70-267`.
- The axis split is established at
  `docs/dev/codex/brief-457-association-tail-rederivation.md:505-527`.

##### Scope

- Files MAY modify: `functional_ops/apply_map/{mod.rs,traversal.rs}`, focused
  MapIndexed/Scan tests, iterative depth certificates, and registry/inventory.
- MUST NOT modify the bounded recursive module, `TransformWork`, normalization,
  Map/Apply/MapAll/Comap semantics, evaluator stability, or generic
  `MapIndexed` acceptance expectations.
- `traversal.rs` is in the 10–20k warning range; report the split review.

##### Deliverable

- Acquire scopes only in completed `MapIndexed` and `Scan` call normalization.
  Guard the association `MapIndexed` call at `traversal.rs:336`; do not put a
  branch in `map_parts_with_mode`. Keep generic MapIndexed edges explicitly
  `DeferredShort` unless the director separately expands scope.
- Guard `Scan` at `:435,470,476`, since its passing collector is one direct
  recursion group. Callbacks still execute sequentially only after collection
  (`apply_map/mod.rs:128-130`).
- Flip only association `MapIndexed` and `Scan` rows to `GuardedNative`; retain
  `Map`/`Apply` as `BoundedHeap` and generic MapIndexed as deferred.
- `MapIndexed` certificate: physical-height-4,097 nested association with one
  unique deepest sentinel; iteratively assert the exact rebuilt key/value chain
  and the exact deepest position supplied to the mapping function. A three-level
  branched companion pins levelspec, `Heads`, association index positions, and
  untouched siblings.
- `Scan` certificate: same height, exact `Null` result, and an iteratively
  inspected ordered log containing the unique deepest sentinel exactly once.
  A shallow companion pins preorder/postorder, levelspec, association values,
  and `Heads` selection.
- Zero-bound modest-depth tests bind to `:336` and `:435`; temporarily unwrap
  `:336`, capture red, restore green.

##### rum / Rust pitfalls (read twice)

Do not turn the whole recursive return into `Result`; the guard's discarded
fallback exists to avoid the brief-431 wide-node cost. Do not charge Map/Apply
or claim generic MapIndexed reaches 4,097. Deep result assertions are iterative.

##### Verification (MANDATORY)

Run bounded fmt, focused apply/map tests, stdlib, runtime, workspace, and both
release certificates. Run zero-bound and removal red/green proofs. Gate
allocation counts, `instructions:u`, and the core microbench for
`mapindexed-association-small`, `scan-association-small`, and unchanged
`map-small`, `apply-small`, `mapindexed-generic-small`, `list-literal-small`,
and `association-small-literal-repeat`
same-tier neighbors. Any Map/Apply movement beyond its own spread is a stop.

##### Hard rules

No git/network/unsafe/oracle/ceiling probe; no edits to the bounded machines or
generic evaluator; no extra shallow allocation; no levelspec/path/callback
order drift; stop on any certificate, falsifier, or neighbor regression.

##### Report format

Exact guarded edges and axis declarations; exact MapIndexed value/path and Scan
log; Map/Apply source diff proof; red/green; all allocation, instruction,
microbench, and test output; stop-loss.

#### Draft E — guard nested association key-path assignment before commit

##### Goal

Make both association function-lvalue and `Part`-lvalue key-path assignment fail
closed, with no partial store and no change to selector evaluation, direct-key
fast paths, messages, or the scalar return value.

##### Context pointers

- Selector evaluation and finish methods:
  `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:771-919`.
- Recursive rebuild: `association_data.rs:934-1004`, binding edge `:996`.
- Existing exact certificate:
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:266-292`.

##### Scope

- Files MAY modify: `association_data.rs`, a focused child module if used for
  the key-path rebuild, runtime association-assignment tests, the existing
  iterative certificate, and registry/inventory.
- MUST NOT alter selector evaluation, direct one-key mutation, definition-store
  APIs, messages, `Expr`, or unrelated assignment kinds.
- `association_data.rs` is in the 20–35k split-plan range. Prefer moving only
  the association assignment/rebuild responsibility to a child module; if that
  obscures private definition access, keep the minimal edit and report that
  exact future boundary (`docs/code-organization-guidelines.md:64-81`).

##### Deliverable

- Enter a guard scope only after selectors and the current association value
  are ready. Validate `set_association_key_path` as a guarded recursive group
  and wrap `association_data.rs:996`; both callers at `:859,901` use the guarded
  root.
- Finalize the scope before `definitions.entry`, epoch bump, or message
  emission. Exhaustion returns `StackLimitExceeded`, leaves the old ownvalue and
  epoch intact, and emits no misleading `Set::kval`.
- Flip the two Set axes to `GuardedNative` with one exact site ID.
- Certificate: on a declared 2 MiB release thread, assign through 2,048 keys in
  a physical-height-4,097 association containing one unique deepest old
  sentinel; iteratively assert the exact rebuilt chain, exactly one new deepest
  RHS, unchanged siblings/keys, and scalar RHS return. Shallow companions pin
  bare-string versus `Key[...]`, missing-key diagnostics, function-lvalue versus
  `Part`-lvalue, and the direct one-key fast path.
- A modest-depth zero-bound test snapshots ownvalue/epoch/messages, expects
  `StackLimitExceeded`, and proves the snapshot unchanged. Temporarily unwrap
  `:996`; capture red, restore green.

##### rum / Rust pitfalls (read twice)

Do not retain the guard across selector evaluation or suspension. Sticky
fallback `Unchanged` is never allowed to reach message/store logic; guard
finalization comes first. Preserve delayed-ownvalue clearing only on a real
successful update (`association_data.rs:862-866,903-907`).

##### Verification (MANDATORY)

Run bounded fmt, focused runtime assignment/session tests, runtime, stdlib,
workspace, and the release certificate. Run the atomicity zero-bound test and
real-edge red/green proof. Gate allocations, `instructions:u`, and core
microbench rows for `association-set-key-path-small`,
`association-set-key-incremental-{small,large}`, a direct-one-key control,
`association-part-*`,
and shallow scalar `Set`. Paste exact output.

##### Hard rules

No git/network/unsafe/oracle/ceiling probe; no partial write/epoch/message; no
selector or direct-fast-path change; no budget widening; stop on certificate,
atomicity, falsifier, or performance failure.

##### Report format

Per-file/split changes; acquisition/finalization/commit order; exact deep and
shallow outcomes; unchanged snapshot; red/green; allocation, instruction,
microbench, and test output; stop-loss.

#### Draft F — guard held association `Part` mutation without taxing shared exact paths

##### Goal

Make `AssociateTo` and `KeyDropFrom` on exact `Part` targets fail closed through
path collection, read, and rebuild, while leaving ordinary `Part`,
`ReplacePart`, targeted-map consumers, suspension, and mutation atomicity
unchanged.

##### Context pointers

- Held flow and commit:
  `crates/rum-stdlib/src/associations/mutation.rs:522-692`.
- Exact helper recursion and key expansion:
  `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:35-42,88-126,173-201`.
- Shared callers are listed at
  `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs:19-76` and
  include definition/targeted-map users; the kernel exports are
  `crates/rum-runtime/src/runtime/kernel_api.rs:93-99`.

##### Scope

- Files MAY modify: `structure_data/exact_paths.rs`, `structure_data/mod.rs`,
  `kernel_api.rs`, `associations/mutation.rs`, focused mutation/runtime tests,
  iterative certificates, and registry/inventory.
- MUST NOT change existing unguarded exact-path signatures or their callers,
  generic Part/ReplacePart/MapAt behavior, selector evaluation, store APIs, or
  mutation result conventions.
- `mutation.rs` is in the 20–35k split-plan range; report the boundary
  `mutation/held_part.rs` or perform only that responsibility split
  (`docs/code-organization-guidelines.md:64-81`).

##### Deliverable

- Add association-only guarded adapters for exact selector expansion, exact
  read, and exact rebuild. Share leaf utilities, but do not add an optional
  guard branch to the ordinary hot exports. Validate each adapter's recursive
  group and wrap edges corresponding to `exact_paths.rs:41,103,179,194`.
- After selector evaluation/resume, enter one scope around call sites
  `mutation.rs:588,654,679`; finalize it before the second protection check and
  store at `:682-686`. Exhaustion preserves ownvalue/messages and cannot return
  a partially updated selected part.
- Flip held `AssociateTo` and `KeyDropFrom` Part axes to `GuardedNative`; keep
  direct symbol-only mutation `NoProportionalDescent` (`mutation.rs:268-278`).
- Certificates: for each head on a declared 2 MiB release thread, select a
  2,048-key association path (4,096 raw components), mutate the unique deepest
  target, and iteratively assert exact rebuilt target, unchanged siblings,
  correct returned target/part convention, and exactly one deepest sentinel.
  Shallow companions pin `Key[...]` strictness, multi-selector rejection,
  missing part/key diagnostics, and both result conventions.
- Zero-bound modest-depth atomicity tests bind separately to collect, read, and
  replacement. Temporarily unwrap the binding replacement edge corresponding
  to `exact_paths.rs:194`; capture red, restore green.

##### rum / Rust pitfalls (read twice)

A key selector expands to association argument plus rule RHS, so token depth
tracks raw native calls, not logical association wrappers. Do not place a guard
in selector suspension frames. Do not genericize adapters if monomorphization
changes ordinary exact-path prologues; isolation is the purpose of this step.

##### Verification (MANDATORY)

Run bounded fmt, focused held-mutation/suspension tests, runtime, stdlib,
workspace, and both release certificates. Run all zero-bound atomicity tests
and red/green removal proof. Gate allocations, `instructions:u`, and core
microbench rows for `associate-to-part-small`, `key-drop-from-part-small`, every
existing `part-association-*` and `replacepart-*` same-tier neighbor, plus a
targeted-map control. Ordinary shared-helper rows must remain within spread.

##### Hard rules

No git/network/unsafe/oracle/ceiling probe; no optional branch in ordinary exact
helpers; no partial store/message; no selector/result drift; no budget widening;
stop on shared-neighbor, certificate, atomicity, or falsifier failure.

##### Report format

Guarded-adapter versus untouched-call inventory; commit order; exact
certificates/shallow selectors; atomicity and red/green output; ordinary and
target allocation/instruction/microbench rows; file split assessment; tests and
open questions.

#### Draft G — guard the ordinary `Merge` nested collection parser

##### Goal

Make ordinary `Merge` reject native-stack exhaustion in band while preserving
depth-first left-to-right flattening, exact first-invalid attribution, grouping,
combiner order/suspension, and the physical-height-4,097 parser result.

##### Context pointers

- Handler boundary: `crates/rum-stdlib/src/associations/resumable/aggregate.rs:11-56`.
- Parser/root: `crates/rum-stdlib/src/associations/data.rs:163-189`.
- Sibling merge/combiner loop and suspension:
  `resumable/aggregate.rs:93-128` and its resume frames.

##### Scope

- Files MAY modify: `associations/data.rs`,
  `associations/resumable/aggregate.rs` and its resume frames only if entry
  plumbing requires it, focused Merge/parser tests, iterative certificates,
  and registry/inventory.
- MUST NOT change grouping/canonicalization, combiner evaluation, `MergeNested`,
  the stdlib/runtime general association parsers, or diagnostics.
- `data.rs` is in the 10–20k warning range; report the split review.

##### Deliverable

- Enter a scope only in the two-argument `Merge` arm and pass its token to
  `parse_keyed_entries_list_expr_with_diagnostics`. Validate the parser group
  and wrap `data.rs:186`; parser failure fallback is discarded before diagnostic
  matching because stack finalization has precedence.
- Flip the ordinary-Merge collection axis to `GuardedNative`; retain the
  association-value axis as `NoProportionalDescent` and `MergeNested` as
  `BoundedHeap`.
- Certificate: build (without recursive Rust assertions) a raw-height-4,097
  nested list collection containing left shallow entries, one unique deepest
  rule/sentinel, and right shallow entries; on a declared 2 MiB release thread,
  assert the exact merged association, key order, combined duplicate values,
  and unique sentinel iteratively. Shallow companions pin first-invalid element,
  left-to-right flatten order, duplicate grouping, and combiner suspension/no
  replay.
- A modest-depth zero-bound parser test expects `StackLimitExceeded` and no
  combiner call. Temporarily unwrap `data.rs:186`; capture red, restore green.

##### rum / Rust pitfalls (read twice)

Do not turn invalid input into a stack error unless the actual byte guard fired.
Do not reverse a LIFO traversal because no traversal conversion is authorized.
The guard ends before the combiner loop; no token belongs in a resume frame.

##### Verification (MANDATORY)

Run bounded fmt, focused Merge/parser/suspension tests, stdlib, runtime,
workspace, then the exact release certificate. Run zero-bound and red/green
proofs. Gate allocations, `instructions:u`, and core microbench rows for
`merge-collection-small`, `merge-nested-small`,
`association-small-literal-repeat`, and
invalid/duplicate shallow companions. Paste exact output.

##### Hard rules

No git/network/unsafe/oracle/ceiling probe; no parser conversion, grouping or
diagnostic drift, combiner replay, shallow allocation, or budget widening; stop
on any certificate/falsifier/performance failure.

##### Report format

Exact parser edge and scope lifetime; exact deep output and shallow order/error
proofs; combiner count; red/green; allocation, instruction, microbench, and test
output; split assessment/open questions.

#### Draft H — close the remaining passing association-owned recursive shapes

##### Goal

After drafts B–G, eliminate every remaining `GuardPending` row on passing
association-owned input shapes, without touching the explicitly short/deferred
`Keys`, `KeyTake`/`KeyDrop`, `GroupBy`, generic `MapIndexed`, or schema repairs.
If a supposedly passing row refuses at 4,097 after the guard is active, report
it as a newly measured short lane rather than widening the budget.

##### Context pointers

- Complete residual inventory:
  `docs/dev/codex/brief-454-association-depth-class.md:319-339`.
- Local sites: `crates/rum-stdlib/src/associations/basic.rs:549-560,699-721,781-810`;
  `associations/transforms/key_collections.rs:488-517`;
  `associations/data.rs:234-260`.
- Runtime parser copies:
  `crates/rum-runtime/src/runtime/kernel_data/association_data.rs:57-123,279-305`;
  the facade exports them at `runtime/kernel_api.rs:29-34`.

##### Scope

- Files MAY modify: `crates/rum-stdlib/src/associations/{basic.rs,data.rs,mod.rs}`,
  `associations/transforms/key_collections.rs`, their focused tests;
  `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`,
  `runtime/kernel_api.rs`, focused parser tests; iterative certificates; and
  registry/inventory.
- MUST NOT edit non-association consumers of the public parser APIs, the short
  shared `KeyTake`/`KeyDrop` helper, catalog semantics, or convert a recursive
  parser to a work stack. Any additional call-site file needed is a stop and a
  director scope decision.
- Apply the required split assessments to `basic.rs`/`association_data.rs` if
  their sizes are in the 20–35k band
  (`docs/code-organization-guidelines.md:64-81`).

##### Deliverable

- Add guarded parser entry points rather than widening the raw poll. Within the
  allowed association roots, thread branded tokens through the stdlib/runtime
  deep parser SCCs; leave every temporarily unguarded external raw-parser call
  in an exact, sorted escape inventory. Do not call that escape inventory safe.
- Guard passing residual `Values`, `Lookup` query count/result planning, option
  list collection, and `Normal` wrapper descent at their exact recursive calls.
  Keep `Keys`, the shared `KeyTake`/`KeyDrop` helper, and named short axes
  `DeferredShort`.
- For each row, first make its modest-depth zero-bound test green. Only then run
  a declared-2-MiB physical-height-4,097 certificate. Each fixture has one
  unique deepest sentinel; assert the exact rebuilt/nested value iteratively.
  Shallow companions pin wrapper choice, query result shape/missing order,
  option order/first-invalid attribution, `Normal` target selection, parser
  flatten order, duplicate canonicalization, and `RuleDelayed` preservation.
- If a target completes, flip it to `GuardedNative`. If it returns an in-band
  stack limit at 4,097, leave it `DeferredShort` with the exact outcome and stop
  that row. Never remove the guard to seek a ceiling.
- Falsifier per SCC: unwrap its binding edge on the modest fixture, capture red,
  restore green. Misdeclare one real passing row as
  `NoProportionalDescent`; the checked-in registry falsifier must panic.

##### rum / Rust pitfalls (read twice)

The public parser has many consumers, so no claim may extend beyond the exact
guarded/escape inventories. No ambient TLS and no raw-floor export. Invalid
parse versus exhausted guard must remain distinguishable. Do not probe an
unknown ceiling; a guarded 4,097 run is attempted only after its low-bound
mechanism test proves the process cannot escape.

##### Verification (MANDATORY)

Run one row at a time: bounded fmt, its focused tests and zero-bound falsifier,
then its release certificate, then stdlib/runtime/workspace. Run the registry
falsifier after every disposition flip. Gate allocations, `instructions:u`,
and core microbench rows for shallow `Association`, `Values`, `Lookup`,
`Normal`, key-collection options, parser invalid input, and same-tier controls.
Both arms use the same filesystem path and identical harness. Stop the whole
brief on an out-of-scope caller requirement or repeated performance failure.

##### Hard rules

No git/network/unsafe/oracle/past-ceiling probe; no work-stack conversion,
ambient state, raw stack API, budget widening, or weakening 4,097. Do not touch
the explicitly short helpers. No shallow allocation/instruction regression or
diagnostic/order drift.

##### Report format

Per-row exact SCC/call/escape inventory; guarded versus still-deferred registry;
low-bound proof before each target run; unique deepest/exact shallow outputs;
red/green falsifiers; allocation/instruction/microbench and test output;
file-size assessment; any newly discovered short row and stop reason.

### 8. Risks and open questions

1. **Reasoned — the 1.5 MiB operation boundary is the largest likely director
   contradiction.** It follows the declared-stack reserve arithmetic, but has
   not been installed or measured. `DeleteMissing` has the tightest supplied
   target-to-crash margin: 4,097 versus 6,272/6,304 (**Measured** at
   `docs/dev/codex/brief-458-association-fail-open-direction.md:33-40`). A new
   token slot/prologue could make it refuse before 4,097. The correct response
   is frame reduction or a separately reviewed conversion, not spending poll
   headroom or weakening the certificate.
2. **Reasoned — “non-evaluator reserve may be spent by guarded operation
   descent” needs director ratification.** The formula is consistent with the
   names and derivation at `crates/rum-runtime/src/runtime/metadata.rs:20-31`,
   but no existing API says that reserve is itself a user-visible operation
   budget. If the intended policy is that *all* descent must stop at the 1 MiB
   evaluator value, a local poll will contradict at least the previously priced
   816 B × 2,048 finite `DeleteMissing` path (**Measured packet** at
   `docs/dev/codex/brief-457-association-tail-rederivation.md:455-456`); the
   design would have to reopen frame shrink/conversion rather than pretend both
   contracts fit.
3. **Reasoned — sticky fallback is lower-cost but harder to review than
   `Result`.** Finalization must dominate invalid-input, suspension, and mutation
   outcomes at every root. Proc-macro validation can require a root scope and
   checked SCC edges, but semantic tests are still required. This complexity is
   accepted only to avoid the documented 6–15% every-node `Result` cost
   (`docs/dev/codex/brief-458-association-fail-open-direction.md:101-116`).
4. **Reasoned — token plumbing can enlarge the very frames it protects.** The
   stability V1 precedent grew its hot frame merely by taking a local address
   (`docs/dev/codex/brief-445-stability-strided-poll.md:36-41`). The proposed
   address lives in a cold callee, but branded-token/fallback state remains in
   operation frames. Prologue plus `/proc` release/debug slopes are mandatory;
   reasoned packet arithmetic is not acceptance.
5. **Reasoned — the static audit is intentionally not a sound whole-Rust call
   graph.** Direct and mutual calls in parsed source are enforceable; dynamic
   dispatch/function pointers, macro-generated calls outside the validator, and
   downstream crates are misses. Escape rows prevent accidental green status,
   but review remains necessary.
6. **Reasoned — parser closure may demand broader scope.** The public runtime
   parser entry points are widely called (`crates/rum-runtime/src/runtime/kernel_api.rs:29-34`).
   Draft H stops rather than either infecting unrelated hot paths with token
   plumbing or converting an already passing parser without a director choice.
7. **Reasoned — suspension error precedence must be specified.** If a guard has
   exhausted before code reaches a possible `InputRequired`, return stack limit
   and do not persist partial state. If suspension occurs first, frames unwind
   and resume under a fresh base/scope (`crates/rum-runtime/src/runtime/suspension/core.rs:482-509`).
   Existing side effects completed before an eventual resource error are not
   rolled back; mutation stores specifically remain after successful guard
   finalization.
8. **Reasoned — error text reports the configured evaluator limit.** The
   existing error contains `resource_limits.native_stack_limit`
   (`crates/rum-runtime/src/runtime/eval_core/mod.rs:68-76`). Reusing that value
   in the error while the internal operation boundary is 1.5 MiB is consistent
   with the configured contract but could confuse diagnostics; adding a second
   public limit field would widen API/host serialization and needs a separate
   decision.
9. **Reasoned — forced-tail structural proofs are axis-specific.** `Map` and
   `MergeNested` are heap-bound for canonical association-value depth, but their
   calls to the general association parser retain the separate nested-container
   axis until draft H classifies it (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:123-145`;
   `crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:237-246`).
10. **Measured/open — no new runtime result exists in this report.** I ran no
    test, benchmark, disassembly, `/proc` sample, or depth probe. The supplied
    brackets remain the only measured operation ceilings, exactly as this
    design-only brief requires (`docs/dev/codex/brief-458-association-fail-open-direction.md:27-50,229-245`).

#### Verbatim command evidence used for aggregate claims

The path-and-line citations above are the primary evidence. These are the
verbatim outputs of the three aggregate searches used in the report.

```text
$ rg -n 'NATIVE_STACK_(UNPOLLED_LEVEL_RESERVE_BYTES|DEPTH_POLL_STRIDE|POLL_HEADROOM_BYTES|NON_EVALUATOR_RESERVE_BYTES)|DEFAULT_NATIVE_STACK_LIMIT|fn poll_native_stack_limit|fn native_stack_budget_exhausted' crates/rum-runtime/src/runtime/metadata.rs crates/rum-runtime/src/runtime/eval_core/mod.rs
crates/rum-runtime/src/runtime/eval_core/mod.rs:41:                evaluation_depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
crates/rum-runtime/src/runtime/eval_core/mod.rs:68:    fn poll_native_stack_limit(&mut self) -> Result<(), EvalError> {
crates/rum-runtime/src/runtime/eval_core/mod.rs:82:    pub(in crate::runtime) fn native_stack_budget_exhausted(&self) -> bool {
crates/rum-runtime/src/runtime/metadata.rs:15:pub const NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES: usize = 128 * 1024;
crates/rum-runtime/src/runtime/metadata.rs:17:pub const NATIVE_STACK_DEPTH_POLL_STRIDE: usize = 4;
crates/rum-runtime/src/runtime/metadata.rs:21:pub const NATIVE_STACK_POLL_HEADROOM_BYTES: usize =
crates/rum-runtime/src/runtime/metadata.rs:22:    NATIVE_STACK_DEPTH_POLL_STRIDE * NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES;
crates/rum-runtime/src/runtime/metadata.rs:24:pub const NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES: usize = 512 * 1024;
crates/rum-runtime/src/runtime/metadata.rs:29:pub const DEFAULT_NATIVE_STACK_LIMIT: usize = MIN_SUPPORTED_NATIVE_STACK_BYTES
crates/rum-runtime/src/runtime/metadata.rs:30:    - NATIVE_STACK_POLL_HEADROOM_BYTES
crates/rum-runtime/src/runtime/metadata.rs:31:    - NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES;
crates/rum-runtime/src/runtime/metadata.rs:50:    match stack_bytes.checked_sub(NATIVE_STACK_POLL_HEADROOM_BYTES) {
crates/rum-runtime/src/runtime/metadata.rs:51:        Some(remaining) => remaining.checked_sub(NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES),
```

```text
$ rg -n 'RECURSIVE_TRANSFORM_CALL_DEPTH|transform_parts_iterative|while let Some\(task\)|RECURSIVE_MERGE_NESTED_GROUP_DEPTH|evaluate_merge_nested_group_iterative|resume_merge_nested_work' crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:3:const RECURSIVE_MERGE_NESTED_GROUP_DEPTH: usize = 16;
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:41:                RECURSIVE_MERGE_NESTED_GROUP_DEPTH,
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:97:            return evaluate_merge_nested_group_iterative(session, combiner, grouped);
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:117:fn evaluate_merge_nested_group_iterative(
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:122:    resume_merge_nested_work(
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:134:fn resume_merge_nested_work(
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:139:    while let Some(task) = work.pop() {
crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:202:    resume_merge_nested_work(session, work.as_slice().to_vec(), Some(value))
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:6:const RECURSIVE_TRANSFORM_CALL_DEPTH: usize = 16;
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:20:        RECURSIVE_TRANSFORM_CALL_DEPTH,
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:23:        transform_parts_iterative(
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:52:        RECURSIVE_TRANSFORM_CALL_DEPTH,
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:55:        transform_parts_iterative(
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:94:fn transform_parts_iterative(
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:106:    while let Some(task) = work.pop() {
```

```text
$ rg -n 'association_entries_from_expr\(|association_entry_forms_from_expr\(' crates/rum-runtime/src crates/rum-stdlib/src -g '*.rs' | wc -l
116
```

Final scope/whitespace checks (the second command exited 0 and its verbatim
output is empty):

```text
$ git diff --name-only
docs/dev/codex/brief-458-association-fail-open-direction.md
$ git diff --check
```
