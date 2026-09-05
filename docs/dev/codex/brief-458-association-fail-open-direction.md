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
