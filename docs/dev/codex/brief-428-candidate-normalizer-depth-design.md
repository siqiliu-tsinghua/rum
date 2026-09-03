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
