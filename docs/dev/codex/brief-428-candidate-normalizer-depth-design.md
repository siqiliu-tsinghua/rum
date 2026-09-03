# brief-428 — design: bound the candidate normalizer's native recursion

**Shape: design-only. `Scope: NONE — no source change.`** You return a plan and
the drafted follow-up briefs. You write no code.

## Goal

brief-427 established from source that the 4,500-height ceiling shared by six
converted traversals is not in the traversals at all: it is the complete
recursive normalization of the match candidate that
`Session::match_expr_owned` performs before dispatch
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`, worker report
at `docs/dev/codex/brief-427-per-node-match-recursion.md:139-231`). I have since
confirmed that mechanism by measurement (§ *What I measured*, below).

Produce a **step-by-step plan** to bound that recursion — a depth-limited
recursive front path with an explicit-stack fallback, following the precedent
already in the tree at `crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-87`
and `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-61` — with a
call-site inventory, an exact statement of the copy-on-write contract that must
survive, the gate per step, and the risks. Then **draft the follow-up
implementation briefs themselves**, in `docs/dev/codex/TEMPLATE.md` shape.

This is the matcher's hot entry. The plan must be gateable by a rum-vs-rum A/B,
and it must not move any work into `Expr`/`SharedArgs` construction — that is
exactly what brief-426 tried and what cost 30.04% on `set-part-fixed100-len800`
(`docs/dev/codex/brief-426-height-cache-spike.md:352-425`).

## What I measured, so you do not have to reproduce it

Release `rum-cli`, `ulimit -s 2048` (2 MiB, Rust's default for a spawned
thread — `docs/guide/resource-limits-and-sandbox.md:122`). Candidate values are
`x = 1; Do[x = {x}, {i, N}]`, physical height N+1, confirmed by `Depth[x]`
printing N+1 in every run before the operation under test.

| Probe | Result |
|---|---|
| `MatchQ[x, _?IntegerQ]` — one match, **no traversal** | OK at height 4,601; stack overflow at 4,651 |
| `Count[x, _, Infinity]` | OK at 4,501; overflow at 4,701 |
| `Cases[x, _, Infinity]` | OK at 4,501; overflow at 4,701 |
| `Position[x, 1]` | OK at 4,501; overflow at 4,701 |
| `MatchQ[x, _]` (blank fast path, never enters the normalizer) | OK at 20,001 |
| `Depth[x]` | OK at 4,701 |
| dropping a height-20,001 value (`x = 0`) | no overflow, exit 0 |

**The decisive row is the first one.** A single general match against a deep
candidate dies in the same bracket as an all-level traversal over the same
value. The traversal contributes nothing to the depth; the whole ceiling is one
candidate normalization. That also means the eventual acceptance probe is a
single `MatchQ` with a non-fast pattern, not a traversal.

`TODO.md` records this probe as giving out "at 8,000", which was a test point
rather than a bracket; the bracket is 4,601/4,651. I will correct the entry.

## The mechanism, as I read it

`normalize_match_expr` → `normalize_match_expr_changed` →
`normalize_match_arg_list_changed` → `normalize_match_expr_changed` …
(`crates/rum-runtime/src/runtime/resolution.rs:62-65,99-118,120-153`). Per call
node two activations stay live while the sole argument is descended, and
neither can return until the leaf returns. The `Option<Expr>` "changed"
protocol is what makes this a copy-on-write normalizer, and it is the part a
rewrite is most likely to break.

Two ordering facts that constrain any rewrite:

- The head is normalized **before** the arguments, because
  `attribute_set_for_head` is consulted with the normalized head
  (`resolution.rs:103-105,137`). A head is itself an arbitrary expression, so
  head chains are a second depth axis, not a special case.
- `flatten_flat_call_args` flattens **one level only**
  (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`). Full
  flattening of a nested `Flat` chain is an emergent property of the bottom-up
  order: children are attribute-normalized before the parent flattens. An
  iterative version that changes that order silently stops fully flattening.

## Deliverable 1 — the plan

Step-by-step, each step independently verifiable and revertible. Per step:
files and functions touched, what the step preserves, its gate, and how a
reviewer can tell it worked. State explicitly which step, if any, is the first
one whose A/B can move a hot label.

## Deliverable 2 — the copy-on-write contract, stated precisely enough to test

Write down, as a specification a test could be built from, what the current
code returns and when. At minimum: when `None` is returned versus `Some`; the
three rebuild arms at `resolution.rs:107-117`; and the case at
`resolution.rs:151-153` where attribute normalization compares its result
against the original arguments and **collapses a changed prefix back to
`None`**. An iterative rewrite is correct only if it reproduces this exactly,
including which `Arc`s are shared rather than rebuilt. Say how the plan proves
that — an equivalence test over a corpus of shapes is likelier to hold than an
argument.

## Deliverable 3 — the call-site inventory

Every caller of `normalize_match_expr`, `normalize_match_expr_changed`, and
`normalize_match_arg_list_changed`, with `path:line`. Classify each by whether
it can receive a deep candidate.

Include `normalize_definition_identity_for_matching`
(`resolution.rs:66-95`): it is the same recursion reached from definition
storage, and the plan should say whether one shared iterative core serves both
or whether they must stay separate, with a reason.

## Deliverable 4 — the questions I want settled, with evidence

1. **The front-path constant.** 16, as the precedent uses, or derived from
   something? What does the fallback cost when it fires — does it restart the
   normalization from the root, as `Level` does, and is that acceptable here?
2. **Head chains.** Does the plan bound a deep head spine as well as a deep
   argument spine? If the answer is "heads are shallow in practice", say so
   and mark it an assumption rather than a fact.
3. **Suspension.** `normalize_match_expr` takes `&self` and cannot evaluate,
   so I believe no suspension can occur inside it. Confirm from source, and
   say what in the plan would break if that were ever to change.
4. **What is behind this ceiling.** If a second recursion sits at a similar
   height, the repair buys little and I want to know before dispatching it.
   Answer from source — name the next recursion the same probe would hit and
   estimate its per-level cost relative to the normalizer's. **Do not write an
   executable crash probe for this**; the post-repair bracket is mine to
   measure at the gate.

## Deliverable 5 — the drafted implementation briefs

In `TEMPLATE.md` shape, one per step. Each must carry:

- **The A/B panel, named.** At minimum `matchq-integer-success`,
  `matchq-integer-fail`, `matcher-single-tiny`, `matcher-single-wide-small`,
  `matcher-single-wide-large`, `replaceall-simple`, `rewrite-guarded-hit`,
  `rewrite-guarded-decline-fallback`, `cases-wide-no-match`,
  `count-wide-no-match`, `position-wide-no-match`,
  `deletecases-wide-no-match`, `replace-wide-no-match`
  (`scripts/core-language-microbench.sh:538-596`), plus **named untouched
  control labels** from which the whole-run offset is computed. A raw
  direction split is not evidence until the controls are consulted — see the
  +1.81% offset recorded at `TODO.md` under family A's gate note.
- **A falsifier for its own gate.** The height certificate the step adds must
  be shown to FAIL on the pre-repair binary. A check nobody has watched fail
  is not known to work; both memory-growth falsifiers ship with checked-in
  proofs, and this brief inherits that requirement.
- **A stop-loss.** State the label movement that ends the step. Any coherent
  ordinary-path regression stops it rather than being tuned around.

## Out of scope — do not fold these in, price them separately

brief-427 noted two work-removal opportunities in the same code
(`docs/dev/codex/brief-427-per-node-match-recursion.md:296-317`): `Cases`
normalizes each candidate twice, and the pattern is recompiled per node. Both
are real and neither is this brief's target. Mixing work removal into a
semantics-neutral restructure contaminates the A/B and makes the result
unattributable. Inventory them at the end of your plan with a price and a
recommended order; do not put them in a step.

Likewise out of scope: the construction-side height bound (brief-426, stashed),
any new knob or configuration surface (ruled out 2026-09-03 — the design
extends the existing `recursion_limit` + `poll_native_stack_limit` pair rather
than adding a third), and everything in brief-422 §3(e)'s tail.

## Scope

- Files you MAY modify: **NONE**, except appending your report to
  `docs/dev/codex/brief-428-candidate-normalizer-depth-design.md`.
- You write no source, no test, and no probe program.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` and every other brief are
  read-only.

## Method requirements

- Every claim carries `path:line`. A claim without evidence is a guess and
  should be labelled one.
- **Calibrate before believing.** If you cite a constant, open the file that
  defines it rather than the first file that mentions it — a `grep -rl | head -1`
  returning a reference rather than a definition is a mistake this line has
  already made.
- If you find that my reading of the mechanism is wrong, say so and show the
  source. brief-427 refuted the previous mechanism hypothesis and that was the
  most valuable thing it did.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No network. Touch nothing outside this repository.
- No source change of any kind. If you believe a step cannot be planned
  without an experiment, STOP and write that in the report.
- The Testing Policy in `AGENTS.md` is a hard line.
- **Append your report to this file** before you finish. A report that exists
  only in the dispatch log is one reboot from gone.

## Report format

1. The plan — steps, each with files/functions, invariants, and gate.
2. The copy-on-write specification.
3. The call-site inventory.
4. Answers to the four questions, with evidence.
5. The drafted implementation briefs.
6. The out-of-scope inventory, priced.
7. What you could not establish from source, and what would settle it.

## Worker report (brief-428)

### 1. The plan

#### Design verdict

The proposed repair is viable without changing `Expr`, `SharedExpr`, or
`SharedArgs`: keep a depth-16 recursive front path, return a distinct internal
"depth exhausted" outcome when that path encounters its seventeenth call node,
and restart that normalization once through a bottom-up explicit-stack engine.
The recursive and iterative paths must share the argument-attribute finishing
logic and the four-way call rebuild logic, because those are where the current
copy-on-write decisions are made
(`crates/rum-runtime/src/runtime/resolution.rs:99-153`). This adds no constructor
work: the three constructors retain their current allocation behavior
(`crates/rum-core/src/expr.rs:813-828`), and the explicit `Vec` work stacks exist
only after a candidate exceeds the recursive front-path budget.

There are two independently revertible implementation steps. Step 1 is
test-only and does not alter a release path. Step 2 moves the normalizer into a
cohesive child module while replacing its recursion and adding the deployment
certificate. **Step 2 is the first step whose A/B can causally move a hot
label**: it adds the depth check to ordinary normalization and changes the deep
path. Step 1's additions are all `#[cfg(test)]`, so any release A/B movement in
that step is run noise rather than execution of the patch. The distinction
matters because matcher/candidate paths require a rum-vs-rum A/B
(`AGENTS.md:117-122`).

#### Step 1 — lock the existing copy-on-write contract in tests

**Files and functions.** Add
`crates/rum-runtime/src/runtime/resolution/match_normalization_contract.rs` as a
`#[cfg(test)]` child of `resolution`, register it from
`crates/rum-runtime/src/runtime/resolution.rs`, and add only a `#[cfg(test)]`
adapter that exposes the current `normalize_match_expr_changed` outcome to that
child. The production functions remain byte-for-byte unchanged at this step
(`crates/rum-runtime/src/runtime/resolution.rs:62-154`). A focused unit test is
the appropriate layer for a normalization policy branch
(`docs/testing-strategy.md:18-23`).

The test module should contain a frozen test-only transcription of the current
recursive algorithm, then compare production with that reference over a
deterministic corpus. The comparison must cover the `Option` discriminant, the
logical expression, and an Arc-origin signature relative to the input, using
`SharedExpr::ptr_eq` and `SharedArgs::ptr_eq`
(`crates/rum-core/src/expr.rs:596-602,673-679`). The corpus must include atoms;
empty, unary, and wide calls; a first/middle/last changed argument; head-only,
args-only, and both-changed calls; `Flat`, `Orderless`, and combined attributed
heads; nested `Flat` chains; normalized call heads; depth-17 and depth-65
argument and head spines; and held definition identities. The current code
normalizes heads before arguments and looks attributes up from that normalized
head (`crates/rum-runtime/src/runtime/resolution.rs:103-106,137`), while the
one-level flatten helper makes the nested-`Flat` cases a necessary bottom-up
ordering check (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`).

Add targeted Arc assertions for all three rebuild arms and for unchanged calls.
Also make the test-only reference's argument finisher directly callable inside
the test module and inject a synthetic `Some(arg.clone())` result: that is the
only reliable source-only way found to force the branch where a logically
unchanged attribute result collapses an already-created changed prefix back to
`None` (`crates/rum-runtime/src/runtime/resolution.rs:142-153`). Natural
reachability of that exact subcase was not established; this direct helper test
locks the specified branch rather than pretending the corpus reaches it.

**Preserved invariants.** There is no production behavior change. The tests pin
head-before-args order, left-to-right argument visitation, bottom-up
`Flat`/`Orderless` processing, exact `None`/`Some` outcomes, and Arc reuse
(`crates/rum-runtime/src/runtime/resolution.rs:99-153`). The definition-identity
cases separately pin the hold-none early return and its own four rebuild arms
(`crates/rum-runtime/src/runtime/resolution.rs:67-96`).

**Gate and falsifier.** Run the focused contract suite, then
`cargo test -p rum-runtime`, both through the bounded runner because Cargo may
spawn child processes (`AGENTS.md:86-93`). Calibrate the focused gate before
accepting it: temporarily change the in-scope test adapter so an unchanged
result is reported as `Some(expr.clone())`; the new discriminant test must fail,
and the temporary mutation must be reverted before the green run. This proves
the test detects the most dangerous “always rebuild/always Some” failure rather
than merely comparing equal values.

Although this step is test-only, its drafted brief carries the full required
A/B panel. The named untouched controls are `literal-integer`,
`literal-string`, `symbol-ownvalue-read`, `loop-empty`, `sameq-integer`,
`head-list`, `length-list`, `part-list`, `sameq-list`, `if-true`,
`module-local`, `block-local`, `with-local`, `table-small`, and `do-small`—the
same predeclared control group used by the height-cache gate
(`docs/dev/codex/brief-426-height-cache-spike.md:317-326`). Compute the run
offset as the median delta of those named controls, never from the suite median;
the repository records why a raw direction split is misleading
(`TODO.md:115-119`; `PROGRESS.md:1376-1385`). The required affected/neighbor
panel is the thirteen labels at
`scripts/core-language-microbench.sh:538-598`, plus the definition-pattern rows
at `scripts/core-language-microbench.sh:502-536`. If a coherent family has an
offset-adjusted slowdown above 3%, or one label is above 5% in both bounded A/B
panels and outside the base/base spread, stop and report. Since release code is
unchanged, do not tune around such a result.

**Reviewer proof.** The final diff for this step contains only `#[cfg(test)]`
registration/adapter code and the new test module; all production lines
`crates/rum-runtime/src/runtime/resolution.rs:62-154` remain unchanged. Reverting
the step removes only tests.
A reviewer can tell the contract lock works from the recorded red calibration
and the subsequent green focused run, not from prose.

#### Step 2 — install the bounded front path and explicit-stack fallback

**Files and functions.** Move the normalization cluster now at
`crates/rum-runtime/src/runtime/resolution.rs:62-154` into a new cohesive module,
`crates/rum-runtime/src/runtime/resolution/match_normalization.rs`, declared by
`resolution.rs`. This is a real responsibility boundary, and it avoids growing
a file already locally measured at 20,037 bytes; the repository requires an
assess-on-touch split plan at 20k and prefers normalization as a module boundary
(`docs/code-organization-guidelines.md:64-79,124-140`). Preserve
`normalize_match_expr` and `normalize_definition_identity_for_matching` at
`pub(in crate::runtime)` visibility so all current callers remain unchanged.

Implement these pieces in the new module:

1. `MATCH_NORMALIZER_RECURSIVE_CALL_DEPTH: usize = 16`. Sixteen is the actual
   defining constant for `Map`/`Apply`, not an inferred reference
   (`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-7`), and
   `Level` passes the same literal to its bounded walker
   (`crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-80`).
2. A small internal depth-exhaustion type, so “completed unchanged” remains
   `Ok(None)` and cannot be confused with “retry iteratively.” The bounded
   recursive worker decrements once for every `Expr::Call`, regardless of
   whether that call was reached through a head or an argument; atoms complete
   even with zero call budget. This mirrors the checked decrement used by the
   `Map`/`Apply` front path
   (`crates/rum-stdlib/src/functional_ops/apply_map/traversal/recursive.rs:3-14,70-99`).
3. The recursive front path, retaining lazy prefix construction: do not allocate
   an args vector until the first child returns `Some`, exactly as today
   (`crates/rum-runtime/src/runtime/resolution.rs:125-135`). On depth exhaustion,
   propagate the distinct outcome to the root; do not return a partial result.
4. A cold, non-inlined iterative fallback with `Visit` and `FinishCall` work
   items and a `Vec<Option<Expr>>` completion stack. `Visit(Call)` schedules one
   finish item, then arguments in reverse, then the head, so actual visitation
   remains head first and arguments remain left-to-right. `FinishCall` consumes
   exactly one head result and `args.len()` argument results and produces one
   `Option<Expr>`. This follows the tree's owned work-stack precedent
   (`crates/rum-runtime/src/runtime/tree_walkers/post_order.rs:4-23,40-108,110-135`).
5. One shared argument finisher for attribute lookup, one-level flatten then
   canonical sort, and equality collapse; and one shared call finisher for the
   `(None,None)`, head-only, args-only, and both-changed arms. Both engines call
   those helpers, preventing semantic drift at the exact high-risk branches
   (`crates/rum-runtime/src/runtime/resolution.rs:108-115,137-153`;
   `crates/rum-runtime/src/runtime/call_preparation.rs:340-372`).
6. A complete-worker entry used by both public wrappers. It first runs the
   bounded worker from the requested subtree root; only depth exhaustion causes
   a single iterative restart from that same root. Keep
   `normalize_definition_identity_for_matching` as a distinct outer wrapper,
   but route each head/argument child through this shared complete worker. Its
   hold gate and outer rebuild semantics must not be replaced by normalizing the
   whole identity (`crates/rum-runtime/src/runtime/resolution.rs:67-96`).

No `Expr` constructor, representation, or `SharedArgs::new` implementation is
touched; those constructors currently only create the Arc wrappers
(`crates/rum-core/src/expr.rs:637-645,813-828`). No resource-limit field or
configuration surface is added: `recursion_limit` and `native_stack_limit`
remain the existing pair (`crates/rum-runtime/src/runtime/model/config.rs:3-30`),
and this pure structural walk becomes stack-bounded internally.

**Certificates and equivalence tests.** Extend the Step-1 corpus to run the new
bounded entry at budgets immediately around the boundary and to run the
iterative engine directly, comparing both with the frozen reference. Add a
depth-65 unchanged argument spine, a changed nested-`Flat` spine, a deep head
spine, and both hold-none and held definition-identity cases. The direct head
case is mandatory because `Expr::Call` permits any `Expr` as its head
(`crates/rum-core/src/expr.rs:749-753`).

Add one ignored release-only deployment certificate to
`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`, using its existing
2 MiB thread harness (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-28`):
build physical height 20,001 and evaluate exactly
`MatchQ[x, _?IntegerQ]`, asserting `False`. Do not put `Depth[x]` in this
certificate. Height 4,097 would not falsify the old implementation—the supplied
measurement already clears 4,601—whereas 20,001 is above the measured failure
and is already known to be constructible and droppable by the supplied probe
(`docs/dev/codex/brief-428-candidate-normalizer-depth-design.md:28-47`). This is
a single non-fast general match: `PatternTest` does not take the blank-only fast
path, which otherwise returns before `match_expr_owned`
(`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:106-131`).

**Gate and falsifier.** Build the base release CLI before production edits. Add
the ignored 20,001-height test first and run that one test against the old
normalizer on its 2 MiB thread; record the nonzero stack-overflow failure. If it
passes, stop—the certificate has not been calibrated. After the repair, the
same exact test must pass. Then run `cargo fmt --check`, the focused contract
suite, `cargo test -p rum-runtime`, and `cargo test --workspace`, plus the
ignored release certificate, all with derived bounded-runner timeouts
(`AGENTS.md:75-93`).

Run the required A/B panel from Step 1 and use the same named-control median as
the whole-run offset. Add a supplemental in-kernel `AbsoluteTiming` panel whose
timed statements are exactly `MatchQ[deep65, _?IntegerQ]`,
`MatchQ[deep65, _]` (fast-path control), and `Depth[deep65]` (unmodified-walker
control), with setup outside the timed statements and any temporary source
named `*.rum` (`AGENTS.md:124-127,140-149`). The stock required rows are defined
at `scripts/core-language-microbench.sh:538-598`; do not substitute a raw
direction count for the named-control adjustment (`TODO.md:115-119`).

The stop-loss is: stop on an offset-adjusted coherent slowdown above 3% in any
ordinary matcher/rewrite/traversal family, or on a repeated single-label
slowdown above 5% outside base/base spread. Also stop and report if the
depth-65 general-match fallback is more than 35% slower after its controls;
the root restart can explain roughly one extra 16-node prefix on a unary
65-node spine, but not an unbounded multiplier. Do not change the constant,
move work into constructors, or mix in work removal to tune around a failed
gate (`AGENTS.md:46-50,117-122`).

**Risks and reviewer proof.** The ordinary path pays one decrement/branch per
call node; only the A/B can decide whether that is affordable. A deep candidate
is walked twice up to the point of exhaustion: for the unary certificate the
wasted prefix is sixteen calls, but a broad tree with its first over-depth
branch late can have almost one full traversal discarded before the iterative
restart. That adds no new traversal asymptotic and performs fewer than two
complete structural visits, but attribute sorting/comparison retains its own
cost and the restart can still cause nearly 2x cold-path traversal work plus
transient copy-on-write allocations. A local
subtree fallback avoids the restart but can allocate a fallback stack for many
siblings and creates a mixed recursive/iterative unwind contract; it is not the
first implementation because the root-restart precedent is smaller and easier
to prove (`crates/rum-runtime/src/runtime/tree_walkers/level.rs:76-87`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:14-29,46-61`).

A reviewer can tell the step worked when: the pre-repair certificate is visibly
red and the identical post-repair certificate is green; the reference corpus
agrees on value, `Option` tag, and Arc-origin signature; all direct caller lines
in §3 are unchanged; `crates/rum-core/src/expr.rs` has no diff; and the named
A/B panel clears the stop-loss. Reverting Step 2 restores the old functions and
leaves Step 1's contract tests green against them.

### 2. Copy-on-write specification

This section specifies the current behavior to be tested, not merely the final
logical value.

#### 2.1 Worker result

For any input `expr`, `normalize_match_expr_changed(expr)` returns:

- `None` for every atom (`crates/rum-runtime/src/runtime/resolution.rs:99-102`).
- For a call, `None` only when the head worker returns `None` and the final
  argument-list worker returns `None` (`crates/rum-runtime/src/runtime/resolution.rs:103-110`).
- For a call, `Some(rebuilt)` when either the head or final argument-list result
  is `Some`, using the exact rebuild arms below
  (`crates/rum-runtime/src/runtime/resolution.rs:108-115`).

`normalize_match_expr(expr)` returns the worker's `Some` value verbatim and,
for `None`, returns `expr.clone()` (`crates/rum-runtime/src/runtime/resolution.rs:62-65`).
Because `Expr`, `SharedExpr`, and `SharedArgs` clones are shallow Arc clones,
that unchanged result retains the input call's head and args allocations
(`crates/rum-core/src/expr.rs:573-574,637-638,749-753`).

#### 2.2 Order and argument-prefix protocol

For a call, normalize the head completely before visiting any argument. Use the
normalized head for attribute lookup if it is `Some`; otherwise use the
original head (`crates/rum-runtime/src/runtime/resolution.rs:103-106,137`). Then
visit arguments from index zero upward. Until the first child returns `Some`,
hold no output vector. At the first changed child at index `i`, allocate a vector
containing shallow clones of `args[..i]`, then append that changed child. For
every later child, append its changed result or a shallow clone of the original
child (`crates/rum-runtime/src/runtime/resolution.rs:125-135`). Thus unchanged
prefixes and suffixes inside a rebuilt argument vector retain their descendant
Arcs even though the parent `SharedArgs` allocation is new
(`crates/rum-core/src/expr.rs:573-574,637-645,749-753`).

After all children complete, look up attributes on the normalized head. If it
has neither `Flat` nor `Orderless`, return the lazy prefix vector as-is, which
is `None` when no child changed (`crates/rum-runtime/src/runtime/resolution.rs:137-140`).
If either attribute is present, materialize `args.to_vec()` only when no child
already caused a vector, then run attribute normalization: one-level `Flat`
flattening, followed by `Orderless` canonical sorting
(`crates/rum-runtime/src/runtime/resolution.rs:142-147`;
`crates/rum-runtime/src/runtime/call_preparation.rs:340-365`). The one-level
flatten operation extends only a direct same-head child
(`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`); complete
flattening of a chain therefore depends on this bottom-up order.

Finally compare the attribute-normalized vector with the **original** argument
slice by logical `Expr` equality, not with the recursively normalized vector and
not by Arc identity (`crates/rum-runtime/src/runtime/resolution.rs:149-153`;
`crates/rum-core/src/expr.rs:755-775`). If equal, return `None`, discarding the
materialized vector and every changed-prefix result it owns. This collapse
applies even when `normalized_args` was already `Some`; consequently the parent
must behave as though its original argument allocation never changed. If the
normalized vector differs, return `Some(attribute_normalized_args)`
(`crates/rum-runtime/src/runtime/resolution.rs:142-153`).

#### 2.3 Exact call rebuild arms

Given `(normalized_head, normalized_args)` after the preceding protocol:

| Worker pair | Return | Required sharing |
|---|---|---|
| `(None, None)` | `None` | The wrapper later shallow-clones the original call, so both original root Arcs survive (`crates/rum-runtime/src/runtime/resolution.rs:108-110,62-65`). |
| `(Some(head), None)` | `Some(Expr::call_with_shared_args(head, args.clone()))` | Allocate a new `SharedExpr` for the head; reuse the exact original `SharedArgs` Arc (`crates/rum-runtime/src/runtime/resolution.rs:110`; `crates/rum-core/src/expr.rs:824-828`). |
| `(None, Some(args))` | `Some(Expr::call_shared(head.clone(), SharedArgs::new(args)))` | Reuse the exact original `SharedExpr` Arc; allocate a new `SharedArgs` Arc (`crates/rum-runtime/src/runtime/resolution.rs:111-113`; `crates/rum-core/src/expr.rs:640-645,820-822`). |
| `(Some(head), Some(args))` | `Some(Expr::call(head, args))` | Allocate both root wrappers (`crates/rum-runtime/src/runtime/resolution.rs:114`; `crates/rum-core/src/expr.rs:813-817`). |

No iterative implementation may replace these with an unconditional
`Expr::call`, even when the logical result compares equal. Pointer identity is
an observable internal performance contract: the evaluation-stability cache
validates entries with exact `SharedExpr`/`SharedArgs` pointer equality
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:332-349`).

#### 2.4 Definition identity

`normalize_definition_identity_for_matching(expr, attributes)` has a separate
outer contract. With `HoldAttribute::None`, return `expr.clone()` without
walking it. For a non-call, also return `expr.clone()`. Otherwise normalize the
outer head and each outer argument with the same changed-worker protocol, then
use the same four rebuild shapes, returning an `Expr` rather than an `Option`
(`crates/rum-runtime/src/runtime/resolution.rs:67-96`). It does **not** apply the
supplied outer attributes to that call's args; the definition-preparation path
has already normalized the outer args before constructing `identity`
(`crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:598-624`).

One shared bounded/iterative subtree core should serve ordinary normalization
and every head/argument normalization requested by this wrapper. Replacing the
wrapper itself with `normalize_match_expr(&identity)` would apply a different
outer-call policy, while keeping a second recursive core would leave definition
storage exposed to the same stack failure and invite semantic drift
(`crates/rum-runtime/src/runtime/resolution.rs:67-96`).

#### 2.5 Proof strategy

The proof is an equivalence suite, not a reasoning-only assertion. The frozen
recursive reference and both new engines must agree on the `Option` tag and
logical output over the corpus; targeted tests must compare each output call's
head/args pointer origin with its input using the public pointer predicates
(`crates/rum-core/src/expr.rs:596-602,673-679`). Boundary budgets force both
paths on the same shallow shapes. Direct helper injection forces the
changed-prefix-to-`None` collapse. End-to-end `Flat` cases prove bottom-up full
flattening, normalized-Function-head cases prove attributes are selected after
head completion, and definition-identity cases prove the separate hold gate
(`crates/rum-runtime/src/runtime/resolution.rs:67-153`;
`crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs:92-123`).

### 3. Call-site inventory

The following is every source occurrence that calls one of the three requested
functions; declarations are not counted as callers. “Deep” means the signature
or upstream construction can supply an arbitrary nested `Expr`, not that every
ordinary invocation is deep.

#### 3.1 `normalize_match_expr`

| Caller | Call site | Deep-candidate classification |
|---|---|---|
| `Session::match_expr_owned` | `crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-31` | **Yes, the measured subject-value path.** `value` is owned and unconstrained; normalization precedes compiled dispatch. |
| `Session::resolve_matcher_pattern_expr_owned` | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:46-52` | **Deep expression possible, but it is a pattern, not the subject candidate.** The preceding resolver itself recursively descends pattern-bearing syntax at `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:54-74`. |
| `Session::try_collect_case_owned` | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:342-360` | **Yes.** `expr` is the current traversal candidate. The pattern arm then sends the original candidate through `match_expr_owned`, causing the out-of-scope second normalization. |
| `Session::try_collect_case_suspended` | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:376-393` | **Yes.** This is the resume-side mirror for the same current candidate. |
| `Session::match_expr_interpreted_owned` (pattern) | `crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:325-336` | **Deep pattern possible.** The entry is currently marked dead-code but its signature accepts arbitrary `Expr`. |
| `Session::match_expr_interpreted_owned` (value) | `crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:325-336` | **Deep candidate possible.** Same dormant interpreted entry. |
| `Session::resolve_cases_spec_expr_for_matching_owned` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:142-167` | **Deep resolved spec possible, not a subject candidate.** An evaluated symbol/rule candidate can resolve to an arbitrary non-rule expression. |
| `Session::compile_matcher_for_expr` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:304-306` | **Deep pattern possible.** The normalized expression is then recursively compiled. |
| `ResolveCasesSpecExprForMatchingOwnedSuspensionFrame::resume` | `crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:330-350` | **Deep resolved spec possible.** This is the generated/checked-in resume-side mirror of the live cases-spec branch. |
| `Session::resume_replace_list_after_rule_exprs` | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:108-135` | **Yes.** The entire arbitrary `ReplaceList` target is normalized once before rule scanning. |
| `Session::normalize_definition_candidate_for_matching` | `crates/rum-runtime/src/runtime/definitions/application/fast_match.rs:4-13` | **Yes when the outer definition head has a hold attribute.** Down-, up-, and sub-value lookup construct a call from arbitrary head/args before this gate (`crates/rum-runtime/src/runtime/definitions/application/down_values.rs:85-98`; `crates/rum-runtime/src/runtime/definitions/application/up_values.rs:41-49`; `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:48-56`). Hold-none candidates are shallow-cloned instead. |
| `Session::apply_rewrite_rule_plan_once` | `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:122-134` | **Yes.** `expr` is an arbitrary root or traversal node normalized before the rewrite-plan scan. |

#### 3.2 `normalize_match_expr_changed`

| Caller | Call site | Deep-candidate classification |
|---|---|---|
| `Session::normalize_match_expr` | `crates/rum-runtime/src/runtime/resolution.rs:62-65` | **Yes.** This is the ordinary public-to-runtime wrapper for every input above. |
| `Session::normalize_definition_identity_for_matching` (head) | `crates/rum-runtime/src/runtime/resolution.rs:67-82` | **Yes.** A stored definition's outer head is an arbitrary `Expr`. |
| `Session::normalize_definition_identity_for_matching` (each arg) | `crates/rum-runtime/src/runtime/resolution.rs:80-89` | **Yes.** Any held identity argument can contain a deep subtree. |
| `Session::normalize_match_expr_changed` (head recursion) | `crates/rum-runtime/src/runtime/resolution.rs:99-105` | **Yes; this is the head-spine recurrence.** |
| `Session::normalize_match_arg_list_changed` (argument recursion) | `crates/rum-runtime/src/runtime/resolution.rs:120-135` | **Yes; this is the argument-spine recurrence.** |

#### 3.3 `normalize_match_arg_list_changed`

| Caller | Call site | Deep-candidate classification |
|---|---|---|
| `Session::normalize_match_expr_changed` | `crates/rum-runtime/src/runtime/resolution.rs:99-117` | **Yes.** Every call delegates its complete argument list after its head returns. This is the only caller. |

#### 3.4 Definition-storage entry

`normalize_definition_identity_for_matching` itself has one caller:
`finish_definition_lhs_call`, after hold-aware preparation and outer argument
normalization (`crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:598-625`).
It reaches the same recursive worker separately for the identity head and each
identity argument (`crates/rum-runtime/src/runtime/resolution.rs:76-89`). The
plan therefore uses one shared complete subtree core for matching and
definition storage, while retaining the distinct definition wrapper for the
reasons in §2.4.

### 4. Answers to the four questions

#### 4.1 Front-path constant and fallback cost

Use **16 call nodes**. There is no source-derived formula for this normalizer.
The calibrated facts are that `RECURSIVE_TRANSFORM_CALL_DEPTH` is defined as 16
for `Map`/`Apply` and passed to both front paths
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-7,14-21,46-53`),
and `Level` independently passes 16 before falling back
(`crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-80`). Therefore 16 is
a precedent-based engineering choice, not a claim that measurements derived an
optimal number. For this normalizer it bounds the source recurrence to at most
sixteen call nodes (with both worker activations per argument-spine node), far
below the measured thousands-deep failure chain
(`crates/rum-runtime/src/runtime/resolution.rs:99-153`).

Yes: the first implementation should restart from the requested subtree root,
as `Level`, `Map`, and `Apply` do after the bounded attempt returns failure
(`crates/rum-runtime/src/runtime/tree_walkers/level.rs:76-87`;
`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:14-29,46-61`). On
the measured unary spine that discards only sixteen visited calls before the
full iterative walk. On a broad tree whose first depth-17 branch occurs late,
it can discard almost a complete prefix, so the cold path can approach two
visits and transiently discard rebuilt prefixes. That is acceptable only as a
first, gateable implementation: it preserves O(nodes), keeps all shallow
candidates off the heap work stack, matches established code, and has an
explicit depth-65 cost gate. If that gate fails, stop and redesign the fallback
as a local continuation; do not tune the constant blindly.

The value 16 is an internal implementation threshold, not a third resource
knob. `ResourceLimits` already exposes `recursion_limit` and
`native_stack_limit` (`crates/rum-runtime/src/runtime/model/config.rs:3-30`),
and evaluator recursion checks both the semantic depth and the native-stack
poll stride (`crates/rum-runtime/src/runtime/eval_core/mod.rs:34-43,66-77`).
Normalization cannot simply call that poll because it is an `&self` pure
transform while the poll mutates session state; explicit fallback removes its
input-proportional native recursion instead
(`crates/rum-runtime/src/runtime/resolution.rs:62-153`).

#### 4.2 Head chains

The plan bounds both axes. Every call consumes one front-path unit before either
its head or arguments are descended, and the iterative `Visit(Call)` schedules
the head as a normal child before all args. That preserves the current
head-first recurrence (`crates/rum-runtime/src/runtime/resolution.rs:99-106`).
This is not based on an assumption that heads are shallow: the core type permits
an arbitrary shared expression as a call head
(`crates/rum-core/src/expr.rs:749-753`). A direct depth-65 head-spine equivalence
test is mandatory, in addition to the language-level argument-spine
certificate.

#### 4.3 Suspension

Confirmed: no suspension can occur inside the current normalizer. Both public
entries take `&self` and return `Expr`, while the workers return `Option` rather
than `Result` (`crates/rum-runtime/src/runtime/resolution.rs:62-124`). Their only
session operations are `attribute_set_for_head`, which parses a pure function
or reads attributes, and `normalize_call_args_by_attributes`, which flattens,
sorts, and filters in memory
(`crates/rum-runtime/src/runtime/call_preparation.rs:316-326,340-372`). Neither
calls `evaluate` or produces `EvalError::InputRequired`.

This is a precondition of the proposed local `Vec` work stacks. If normalization
ever evaluates or returns `InputRequired`, returning from the function would
drop the work/completion stacks and lose the exact continuation point. Such a
change would require owned, `FrameRootTrace`-covered state plus explicit
suspend/resume frames, like the matcher paths that currently preserve state on
`InputRequired` (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:34-49,184-209`).
It must not be folded into this refactor.

#### 4.4 What sits behind this ceiling

For the **measured command as described**, the next input-height recursion is
the liveness prelude `Depth[x]`, not the matcher: `eval_depth` calls the direct
recursive `structural_depth` helper
(`crates/rum-stdlib/src/structure_ops/surface.rs:33-62`;
`crates/rum-stdlib/src/structure_ops/mod.rs:115-142`). On a unary argument spine,
`structural_depth` has one explicit recursive activation per call level,
whereas the normalizer has the two alternating source activations
`normalize_match_expr_changed` and `normalize_match_arg_list_changed` per level
(`crates/rum-runtime/src/runtime/resolution.rs:99-153`). That suggests a smaller
source-level cost per level for `Depth`, but its iterator/closure frames and the
optimizer's layout prevent a byte estimate from source. The supplied data says
only that `Depth` succeeds at 4,701 while the normalizer fails by 4,651; it does
not bracket `Depth`'s failure and therefore cannot establish whether its ceiling
is similar or much higher
(`docs/dev/codex/brief-428-candidate-normalizer-depth-design.md:38-45`). It is
nevertheless the next unbounded recurrence in that compound probe, which is why
the new certificate must omit it.

Inside the MatchQ-only certificate, `_?IntegerQ` compiles through one
`PatternTest` layer to a blank (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:39-43,67-71`),
the inner blank inspects only the candidate root
(`crates/rum-runtime/src/runtime/pattern_matching/core.rs:504-515`), and
`IntegerQ` also inspects only its one argument's root
(`crates/rum-stdlib/src/predicates/numeric.rs:4-17`). Source therefore shows no
second candidate-height matcher recurrence for this exact pattern after
normalization. This conclusion depends on preserving the unchanged candidate's
Arc identity: evaluation-stability cache hits require exact head/args pointers
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:219-243,332-349`).
If that cache misses, its uncached stability analysis recursively visits heads
and args (`crates/rum-runtime/src/runtime/eval_core/stability.rs:5-33`), so the
copy-on-write gate is also what keeps that latent recursion behind the exact
probe.

Two attributed-shape recursions remain outside the traversal rewrite itself:
`Orderless` comparison recursively compares call heads/args, and the final
equality-collapse comparison recursively compares independently allocated
logical trees (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:113-173,236-240`;
`crates/rum-core/src/expr.rs:755-775`). The unary `List` certificate exercises
neither. Their deployment thresholds cannot be inferred from source and are
recorded in §7 rather than silently claimed fixed.

### 5. Drafted implementation briefs

The numbers below are provisional follow-up numbers. Per the template, these
drafts are raw material for director review rather than ready-to-dispatch
authority (`docs/dev/codex/TEMPLATE.md:24-43`).

---

# brief-429 — lock the match normalizer's copy-on-write contract

**Shape: implementation, test-only foundation.** No release behavior change.

## Goal

Add a focused, calibrated unit-test contract for match normalization before its
implementation changes. The suite must distinguish `None` from `Some`, prove
the three rebuild arms' Arc reuse, cover head-first/bottom-up attribute
normalization, and freeze the definition-identity wrapper. Production
normalization remains unchanged at
`crates/rum-runtime/src/runtime/resolution.rs:62-154`.

## Context pointers

- Read `AGENTS.md:36-51,75-93,95-132,134-157` and
  `docs/testing-strategy.md:16-37,74-94` before editing.
- Current contract: `crates/rum-runtime/src/runtime/resolution.rs:62-154`.
- Arc identity and constructors: `crates/rum-core/src/expr.rs:573-608,637-708,749-775,813-828`.
- Attribute normalization and one-level flattening:
  `crates/rum-runtime/src/runtime/call_preparation.rs:316-372` and
  `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`.
- Function-head attributes:
  `crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs:92-123`.
- Acceptance is mechanical: the red calibration, focused green suite, runtime
  suite, and named rum-vs-rum panel decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution.rs`, only for `#[cfg(test)]`
    module registration and a `#[cfg(test)]` outcome adapter;
  - new
    `crates/rum-runtime/src/runtime/resolution/match_normalization_contract.rs`.
- Do not change the production bodies at
  `crates/rum-runtime/src/runtime/resolution.rs:62-154`.
- Do not modify `Expr`, `SharedExpr`, `SharedArgs`, Cargo metadata, docs, or any
  caller. Constructor behavior is fixed at `crates/rum-core/src/expr.rs:813-828`.
- Do not add a source-language probe or fixture; this step is a crate-local
  helper contract, the preferred layer at `docs/testing-strategy.md:18-23`.

## Deliverable

- A test-only recursive reference transcribed from
  `crates/rum-runtime/src/runtime/resolution.rs:62-154`, with no production use.
- A deterministic corpus covering atoms; empty/unary/wide calls;
  first/middle/last changed args; unchanged/head-only/args-only/both-changed
  roots; `Flat`, `Orderless`, both attributes, nested `Flat`, normalized
  Function heads, argument and head spines at depths 17 and 65, and held
  definition identities.
- Per case, compare the exact `Option` discriminant, logical output, and Arc
  origin relative to the input. Use `SharedExpr::ptr_eq` and
  `SharedArgs::ptr_eq` (`crates/rum-core/src/expr.rs:596-602,673-679`).
- Direct assertions for all four call outcomes at
  `crates/rum-runtime/src/runtime/resolution.rs:108-115` and the equality
  collapse at `crates/rum-runtime/src/runtime/resolution.rs:142-153`. Inject a synthetic
  logically-equal `Some` child into the test-only argument finisher so the
  changed-prefix collapse is actually executed.
- Definition-identity assertions for the hold-none early return and all rebuild
  arms at `crates/rum-runtime/src/runtime/resolution.rs:67-96`.
- Recorded red calibration and green verification output.

## rum / Rust pitfalls (read twice)

1. Logical `Expr::eq` is insufficient; it has Arc fast paths but deliberately
   hides whether a wrapper was rebuilt (`crates/rum-core/src/expr.rs:755-775`).
2. `None` and `Some(expr.clone())` can produce the same wrapper-level value;
   compare the worker discriminant directly
   (`crates/rum-runtime/src/runtime/resolution.rs:62-65,99-118`).
3. The args vector is lazy until the first changed child, and later unchanged
   children are shallow clones
   (`crates/rum-runtime/src/runtime/resolution.rs:125-135`). Pin that allocation
   boundary through pointer origin, not an allocation timing.
4. `flatten_flat_call_args` is one-level only; a deep flat-chain expected value
   proves child-before-parent completion
   (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`).
5. Do not encode third-party behavior. Tests verify rum's current internal
   contract, consistent with `AGENTS.md:36-51`.

## Verification (MANDATORY — run it; iterate to green)

First build/store a base release `rum-cli` before edits. Keep all build and
timing processes bounded as required by `AGENTS.md:90-93`.

Calibrate the new focused gate: temporarily make the in-scope test adapter map
an unchanged `None` to `Some(expr.clone())`, run the focused test, and paste the
failure proving the `Option`/sharing contract noticed. Revert that temporary
mutation. Set `brief_timeout_seconds` from the recorded duration of a completed
same-shape command, then run and paste:

```text
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo fmt --check
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo test --offline -p rum-runtime match_normalization_contract
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo test --offline -p rum-runtime
```

Run `scripts/core-language-microbench.sh` on the stored base and candidate
release binaries. Name before looking:

- mandatory affected/neighbors: `matchq-integer-success`,
  `matchq-integer-fail`, `matcher-single-tiny`,
  `matcher-single-wide-small`, `matcher-single-wide-large`,
  `replaceall-simple`, `rewrite-guarded-hit`,
  `rewrite-guarded-decline-fallback`, `cases-wide-no-match`,
  `count-wide-no-match`, `position-wide-no-match`,
  `deletecases-wide-no-match`, `replace-wide-no-match`;
- additional definition neighbors: `downvalue-pattern-hit`,
  `downvalue-pattern-miss`, `downvalue-pattern-miss-repeat`,
  `downvalue-guarded-hit`, `downvalue-guarded-decline-fallback`,
  `upvalue-guarded-hit`, `subvalue-unguarded-hit`,
  `subvalue-unguarded-miss`, `subvalue-guarded-hit`;
- untouched controls: `literal-integer`, `literal-string`,
  `symbol-ownvalue-read`, `loop-empty`, `sameq-integer`, `head-list`,
  `length-list`, `part-list`, `sameq-list`, `if-true`, `module-local`,
  `block-local`, `with-local`, `table-small`, `do-small`.

The stock target rows are defined at
`scripts/core-language-microbench.sh:502-598`, and the controls at
`scripts/core-language-microbench.sh:456-500`. Compute the whole-run offset as
the median delta of the named controls, not the suite median; paste every named
row and the offset calculation. A raw direction split is not evidence
(`TODO.md:115-119`).

Probe budget: one base/base calibration pair followed by at most two
interleaved base/candidate pairs (six stock-suite executions total), using the
same scale, repetitions, environment, and exact binaries throughout. Derive
each bounded-runner timeout from the first completed run of the same shape; do
not silently add reruns when a number looks odd. Use the better of the two
candidate comparisons per label only after reporting both, consistent with the
template's best-of-2 cap (`docs/dev/codex/TEMPLATE.md:147-155`).

**Stop-loss:** stop and report if any coherent named family is more than 3%
slower after subtracting the control offset, or if one label is more than 5%
slower in both bounded panels and outside the base/base spread. This step has no
release-path change, so do not tune around movement.

## Hard rules

- Do not commit, branch, push, stash, checkout, or touch git config.
- No network; use `--offline` for Cargo. Touch nothing outside the repo except
  `/tmp` outputs. Any generated rum source is `*.rum` (`AGENTS.md:140-149`).
- Do not change production normalization or any caller. If the contract cannot
  be tested within Scope, stop and report.
- No external oracle and no weakening of rum's contract (`AGENTS.md:36-51`).
- No `unsafe` (`AGENTS.md:109-115`).
- Profiling variables are off for timing; use the script's in-kernel monotonic
  rows (`docs/testing-strategy.md:74-94`).
- Preserve unrelated user changes (`AGENTS.md:150-157`).

## Report format

1. What changed, per file and line.
2. Falsifier: exact temporary mutation and verbatim failing test output.
3. Verification: verbatim focused/runtime/fmt output.
4. A/B: all named rows, control-offset calculation, and verdict.
5. Stop-loss/open questions.

---

# brief-430 — bound match-candidate normalization recursion

**Shape: implementation. Matcher hot path; correctness, deployment-stack, and
rum-vs-rum A/B gates are mandatory.**

## Goal

Replace input-proportional native recursion in match normalization with a
depth-16 recursive front path and a bottom-up explicit-stack fallback, while
preserving the exact Step-1 copy-on-write contract. A single non-fast
`MatchQ[x, _?IntegerQ]` must complete at physical height 20,001 on a 2 MiB
thread. Definition-storage normalization must use the same bounded core. Do not
change constructors, semantics, evaluation/suspension, or repeated-work policy.

## Context pointers

- Read `AGENTS.md:36-51,75-132,134-157`, the Step-1 tests, and this plan's
  copy-on-write specification before editing.
- Current normalizer and definition wrapper:
  `crates/rum-runtime/src/runtime/resolution.rs:62-154`.
- Measured general-match entry:
  `crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`.
- Depth-16 precedents:
  `crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-100` and
  `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-61`.
- Explicit-stack scheduling precedent:
  `crates/rum-runtime/src/runtime/tree_walkers/post_order.rs:4-135`.
- Existing 2 MiB certificate harness:
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:3-28`.
- Attribute order and rebuild contract:
  `crates/rum-runtime/src/runtime/resolution.rs:99-153`,
  `crates/rum-runtime/src/runtime/call_preparation.rs:316-372`, and
  `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`.
- Acceptance is mechanical: the pre-repair red certificate, post-repair green
  certificate, reference equivalence, workspace tests, and named A/B decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution.rs`;
  - new `crates/rum-runtime/src/runtime/resolution/match_normalization.rs`;
  - `crates/rum-runtime/src/runtime/resolution/match_normalization_contract.rs`;
  - `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`.
- Do not modify any caller listed in brief-428 §3.
- Do not modify `crates/rum-core/src/expr.rs`, any constructor, public type,
  serde shape, `ResourceLimits`, or configuration surface. Constructors remain
  as at `crates/rum-core/src/expr.rs:813-828`.
- Do not modify microbench scripts; supplemental timing source is temporary
  under `/tmp` and must use `.rum`.
- Do not fold in Cases' duplicate normalization or per-node pattern compilation
  (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:342-360`;
  `crates/rum-runtime/src/runtime/pattern_matching/core.rs:29-31`).

## Deliverable

1. Move the cohesive normalization cluster from
   `crates/rum-runtime/src/runtime/resolution.rs:62-154` into
   `crates/rum-runtime/src/runtime/resolution/match_normalization.rs`; keep the
   runtime-visible API names and all caller lines unchanged.
2. Define `MATCH_NORMALIZER_RECURSIVE_CALL_DEPTH = 16`, calibrated to the actual
   precedent definition at
   `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-7`.
3. Add a distinct depth-exhaustion result. `Ok(None)` means completed unchanged;
   exhaustion alone selects fallback.
4. Preserve the current lazy recursive front path through 16 call nodes. Count
   every call reached through either heads or args. Do not allocate a work stack
   on a completed shallow candidate.
5. Add a cold, non-inlined iterative engine with owned `Visit`/`FinishCall`
   tasks and a completion stack. Schedule head first and args left-to-right;
   finish bottom-up.
6. Share the attribute finisher and call-rebuild finisher between recursive and
   iterative engines. Preserve all `None`/`Some` and Arc outcomes specified by
   brief-428 §2.
7. Keep `normalize_definition_identity_for_matching` distinct, including its
   hold-none early return, but route its head/arg subtrees through the same
   complete bounded/iterative worker
   (`crates/rum-runtime/src/runtime/resolution.rs:67-96`).
8. Extend the reference corpus across budgets 0/1/15/16/17, depth-65 arg/head
   spines, nested Flat and Orderless forms, and held definition identities.
9. Add an ignored release-only 2 MiB certificate evaluating exactly
   `MatchQ[x, _?IntegerQ]` at physical height 20,001 and asserting `False`. Do
   not call `Depth` in that test; `Depth` has its own recursion
   (`crates/rum-stdlib/src/structure_ops/mod.rs:115-142`).
10. Paste the pre-repair failure, post-repair pass, tests, A/B rows, controls,
    and stop-loss verdict verbatim.

## rum / Rust pitfalls (read twice)

1. The `Option<Expr>` is copy-on-write state, not merely an optimization hint.
   `None` and `Some(equal_value)` are not interchangeable
   (`crates/rum-runtime/src/runtime/resolution.rs:62-65,99-153`).
2. A two-layer outcome is mandatory: never overload `None` to mean depth
   exhaustion.
3. Heads precede args and can themselves be deep
   (`crates/rum-runtime/src/runtime/resolution.rs:103-106`;
   `crates/rum-core/src/expr.rs:749-753`).
4. Full Flat-chain normalization emerges only from bottom-up processing because
   the helper flattens one level
   (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:97-111`).
5. Attribute normalization compares with the original args and can collapse a
   changed prefix (`crates/rum-runtime/src/runtime/resolution.rs:142-153`). Do
   not compare against the rebuilt
   vector or use pointer equality.
6. Preserve the three asymmetric constructors: head-only reuses args,
   args-only reuses head, both allocates both
   (`crates/rum-runtime/src/runtime/resolution.rs:108-115`;
   `crates/rum-core/src/expr.rs:813-828`).
7. The fallback restarts once from the requested root. Do not add a preflight
   height scan or any constructor-side cache; the rejected cache regressed
   `set-part-fixed100-len800` by 30.04%
   (`docs/dev/codex/brief-426-height-cache-spike.md:352-425`).
8. Normalization cannot suspend today
   (`crates/rum-runtime/src/runtime/resolution.rs:62-153`;
   `crates/rum-runtime/src/runtime/call_preparation.rs:316-372`). Do not add
   evaluation or continuation state.
9. `Orderless` comparison and logical equality contain separate recursion
   (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:113-173`;
   `crates/rum-core/src/expr.rs:755-775`).
   Do not claim those independent paths are repaired.
10. Use safe Rust only (`AGENTS.md:109-115`).

## Verification (MANDATORY — run it; iterate to green/evidence)

Before production edits, build and retain a base release `rum-cli`. Add the
ignored height test first, while the old normalizer is still present, and run
only that test through `scripts/run-bounded.py`. It must fail by stack overflow
on its own 2 MiB thread. Paste the failure. If it passes, stop: a certificate
not observed failing is not a gate.

After implementation, set `brief_timeout_seconds` from the recorded duration
of a completed same-shape command, then run and paste verbatim:

```text
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo fmt --check
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo test --offline -p rum-runtime match_normalization_contract
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo test --offline --workspace
scripts/run-bounded.py --timeout "${brief_timeout_seconds}" -- cargo test --offline --release -p rum-stdlib matchq_pattern_test_completes_at_physical_height_20001_on_two_mib -- --ignored
```

Run the base/candidate core microbench A/B with profiling variables off. The
mandatory panel is:

- `matchq-integer-success`, `matchq-integer-fail`, `matcher-single-tiny`,
  `matcher-single-wide-small`, `matcher-single-wide-large`,
  `replaceall-simple`, `rewrite-guarded-hit`,
  `rewrite-guarded-decline-fallback`, `cases-wide-no-match`,
  `count-wide-no-match`, `position-wide-no-match`,
  `deletecases-wide-no-match`, `replace-wide-no-match`;
- `downvalue-pattern-hit`, `downvalue-pattern-miss`,
  `downvalue-pattern-miss-repeat`, `downvalue-guarded-hit`,
  `downvalue-guarded-decline-fallback`, `upvalue-guarded-hit`,
  `subvalue-unguarded-hit`, `subvalue-unguarded-miss`,
  `subvalue-guarded-hit`;
- untouched controls `literal-integer`, `literal-string`,
  `symbol-ownvalue-read`, `loop-empty`, `sameq-integer`, `head-list`,
  `length-list`, `part-list`, `sameq-list`, `if-true`, `module-local`,
  `block-local`, `with-local`, `table-small`, `do-small`.

Those rows are defined at `scripts/core-language-microbench.sh:456-598`.
Compute and paste the median delta of the named controls as the whole-run
offset, then show raw and offset-adjusted target deltas. Do not use the suite
median or direction split (`TODO.md:115-119`; `PROGRESS.md:1376-1385`).

Probe budget: one base/base calibration pair followed by at most two
interleaved base/candidate pairs (six stock-suite executions total), all with
the same scale, repetitions, environment, and retained binaries. Derive the
bounded timeout from the first completed run of the same shape. Report both
candidate comparisons before using their better per-label result; do not rerun
selectively. This is the template's best-of-2 stop, not an invitation to sample
until the direction changes (`docs/dev/codex/TEMPLATE.md:147-155`).

Also run a temporary, in-kernel `AbsoluteTiming` panel with setup outside the
timed region and exact timed statements:

```text
MatchQ[deep65, _?IntegerQ]
MatchQ[deep65, _]
Depth[deep65]
```

Name the temporary file `*.rum`, give every statement a liveness assertion,
and report the same iteration count/repetitions for base and candidate. Do not
wrap a timed expression in an assignment or sink; the benchmark rule is
documented at `docs/dev/codex/TEMPLATE.md:70-91`.

**Stop-loss:** end the step without tuning if an ordinary named family has a
coherent offset-adjusted slowdown above 3%, if one label is above 5% in both
bounded panels and outside base/base spread, or if the depth-65 general-match
row is more than 35% slower after its two controls. Report any failed row; do
not alter constructors, the constant, or adjacent work to hide it.

## Hard rules

- Do not commit, branch, push, stash, checkout, or touch git config.
- No network; Cargo is `--offline`. Touch nothing outside the repository except
  `/tmp` build/timing outputs. Use `scripts/run-bounded.py` for all Cargo,
  release, and benchmark commands (`AGENTS.md:90-93`).
- Do not modify outside Scope. If a caller must change, stop and report.
- Testing Policy is a hard line; no vendor oracle (`AGENTS.md:36-51`).
- No new `unsafe` (`AGENTS.md:109-115`).
- No constructor scan, height cache, resource knob, profiling counter, or
  unconditional heap allocation on the shallow path.
- Do not mix in Cases work removal or pattern compile-once work.
- Preserve unrelated user changes and leave the tree uncommitted
  (`AGENTS.md:150-157`).
- Paste real output only; a missing/failing gate must be reported as such.

## Report format

1. What changed, per file and line; confirm caller inventory and
   `crates/rum-core/src/expr.rs` are untouched.
2. Copy-on-write equivalence: corpus, `Option` results, Arc-origin checks.
3. Falsifier: verbatim pre-repair height failure and identical post-repair pass.
4. Verification: fmt, focused, runtime, workspace, and ignored-release output.
5. A/B: all named stock rows, named-control offset, supplemental depth-65 rows,
   and stop-loss verdict.
6. Open risks, including residual comparator/equality recursion.

### 6. Out-of-scope inventory, priced

1. **Cases normalizes pattern candidates twice.** `try_collect_case_owned` and
   its resume mirror normalize `expr` before branching, but the pattern arm then
   calls `match_expr_owned` with the original `expr`, whose entry normalizes it
   again (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:342-360,376-393`;
   `crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-32`). Price:
   **small, one implementation brief** after brief-430. Move normalization into
   the rule-only branch while leaving the pattern arm's single general entry
   intact; gate Cases value/order/suspension and the full matcher panel. It can
   remove close to one full candidate scan in pattern Cases, but must not share
   brief-430's A/B because that would make the recursion restructure's cost
   unattributable.
2. **Traversal patterns are recompiled per selected node.** The general entry
   calls `compile_matcher_for_expr` on every invocation
   (`crates/rum-runtime/src/runtime/pattern_matching/core.rs:23-31`), and the
   `Cases`, count-like, `Position`, and `DeleteCases` loops invoke that entry per
   selected node (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:349-353`;
   `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:302-329`;
   `crates/rum-runtime/src/runtime/tree_walkers/position.rs:198-230`;
   `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:4-53`).
   Price: **medium, one design brief plus probably two implementation briefs**:
   first create/verify a compile-once traversal plan, then thread its owned or
   Arc-backed matcher through live and suspended traversal state. Suspension is
   the width driver because matcher state must survive `InputRequired`, as the
   loops currently preserve their raw pattern in frames
   (`crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:292-326`;
   `crates/rum-runtime/src/runtime/tree_walkers/position.rs:219-228`).

Recommended order is brief-430 first, Cases' duplicate scan second, and
compile-once traversal plans third. The small Cases removal gives a clean
one-variable A/B before the wider state-shape change. Combining the last two
could make the final architecture smaller, but would violate the explicit
attribution requirement of this brief.

Also excluded and not priced as follow-ups here: constructor-side height
maintenance, which already hit a 30.04% stop-loss
(`docs/dev/codex/brief-426-height-cache-spike.md:352-425`); a new resource knob,
because the existing limit pair is in `ResourceLimits`
(`crates/rum-runtime/src/runtime/model/config.rs:3-30`); and the independent
recursion tail catalogued by brief-427
(`docs/dev/codex/brief-427-per-node-match-recursion.md:317-382`).

### 7. What source could not establish, and what would settle it

1. Source cannot show that the extra shallow-path decrement/branch is
   performance-neutral, nor whether code layout moves the same hot rows. Only
   the named base/candidate `timings.tsv` panel and named-control offset can
   decide (`AGENTS.md:46-50,117-127`). No timing was run in this design-only
   brief.
2. Sixteen is precedent, not a measured optimum for these frames. Optimized
   inlining and frame bytes are not encoded in Rust source
   (`crates/rum-runtime/src/runtime/resolution.rs:99-153`). A disassembly/stack
   profile could describe them, but is unnecessary if the constant-16 A/B and
   deployment certificate pass; do not tune before that evidence.
3. I could not construct from current normalizer semantics a natural tree whose
   child worker returns a logically-equal `Some` and whose parent then exercises
   the “changed prefix but final args equal original” subcase. The branch is
   nevertheless explicit and must be preserved
   (`crates/rum-runtime/src/runtime/resolution.rs:142-153`). A direct unit test of
   the factored argument finisher with a synthetic logically-equal `Some` child
   settles the implementation contract without claiming natural reachability.
4. The post-repair 2 MiB ceiling cannot be established from source. The
   test-first 20,001-height `MatchQ` certificate settles the required path; the
   director's post-repair bracket remains the authority for the ultimate
   ceiling. This report deliberately ran no crash probe, as required by the
   brief.
5. Deep `Orderless` comparisons and the final logical-equality check retain
   their own direct recursion (`crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:113-173`;
   `crates/rum-core/src/expr.rs:755-775`). Focused attributed-candidate 2 MiB
   measurements would establish their separate thresholds; the unary `List`
   certificate cannot.
6. A head-spine deployment threshold was not measured. The explicit-stack
   design covers it by construction and the depth-65 equivalence corpus checks
   fallback selection, but an additional ignored 20,001-height direct
   normalizer head-spine test would be needed if the reviewer wants the same
   deployment-strength certificate for that axis. It should dismantle the
   result iteratively so test cleanup does not become the measured operation;
   the repository already has such a cleanup helper
   (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:552-559`).
