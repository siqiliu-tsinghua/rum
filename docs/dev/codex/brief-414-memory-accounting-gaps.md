# brief-414 — design: two accounting gaps in the modelled memory ceiling

Shape: **design-only**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

## Goal

`eval_memory_limit` installs a modelled memory ceiling per request, and
`docs/guide/resource-limits-and-sandbox.md` states that a breach returns
`TerminatedEvaluation["MemoryLimit"]` in band. Two categories of allocated
bytes escape that ceiling: one the model never walks at all, one it walks too
late. Produce a reviewable plan that closes both — options with a
recommendation for the first, a complete inventory plus a structural fix for
the second — and draft the follow-up implementation briefs. **Write no code.**

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines) first.
- `docs/guide/resource-limits-and-sandbox.md` — what the ceiling promises.
- `crates/rum-core/src/expr.rs:42-96` — the symbol interner.
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50` —
  `current_memory_proxy_bytes`, the model itself.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:9,139-186`
  — the stride, the strided check, the projected-bytes helper.
- `docs/dev/sealed-scoping-design.md` — prior art on symbol identity lifetime.

## Gap A — the symbol interner is outside the model and never shrinks

`symbol_interner()` (`expr.rs:47-55`) is a process-wide
`OnceLock<Mutex<SymbolInterner>>`. `intern_text` (`:57-70`) `Box::leak`s every
name that is not one of the two canonical statics, and `intern_symbol_text`
(`:80-96`) `Box::leak`s a `SymbolText` per (context, name) pair. Neither
`strings` nor `pairs` has a removal or clear path — grep the file.

`current_memory_proxy_bytes` (`memory.rs:50`) sums session-owned structures:
builtins, the association cache, contexts, the evaluation stack, definitions,
attributes and so on. It never mentions the interner, so those bytes are
invisible to `enforce_active_resource_limits_for_expr`
(`constraints.rs:145-159`), which is the only thing the ceiling consults.
`Session::reset` rebuilds the session and does not touch it either.

**Director-measured, so you do not have to re-derive it** (verify if you want
to, do not assume): under a `{Pure}`-only policy with a memory ceiling armed,
eight consecutive requests each interning 20 000 fresh symbol names took RSS
from 5.6 MB to 57.5 MB monotonically, and `reset()` returned only to 47.0 MB.
Every request stayed in band; no ceiling fired.

Note `intern_symbol_text:88` also `expect`s the pair count to fit in `u32`.

## Gap B — the ceiling is sampled every 64 evaluation points

`ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64` (`constraints.rs:9`). The strided
branch (`:145-159`) does a full walk on every 64th armed check and only
decrements a counter in between. The comment above the constant claims the
interval is safe because "overshoot is bounded by the existing per-step
element, integer, and iteration budgets". **Those budgets bound counts, not
the bytes one step can add**, and a step whose output size is a multiple of
its input size is not bounded by any of them.

The discipline for this already exists and is applied in **18 call sites**:
`enforce_active_memory_limit_for_projected_bytes` (`constraints.rs:175`) is
called from `random.rs`, `combinatorial_lists.rs`, `list_plumbing.rs`,
`sequence/range.rs`, `arrays/generators.rs`, `sequence/select.rs`,
`iteration/spec.rs` and `strings/text/builtins.rs`. `StringRepeat`
(`strings/text/builtins.rs:664-668`) prechecks projected bytes *and* the
generated-element count. `Power` has a bit-length precheck of its own
(`rum-runtime/src/runtime/arithmetic/canonical.rs:99-136`), and
`IntegerWorkBudget` threads a budget through `combinatorics.rs` and
`number_theory.rs`.

Two operations that grow bytes multiplicatively in one step and pass through
none of it:

- `eval_string_join` (`crates/rum-stdlib/src/strings/text/builtins.rs:13-42`)
  takes no `session` at all — its first parameter is `_session` — and ends in
  a bare `pieces.concat()`. `s = s <> s <> s` triples per step.
- Big-integer `Times`. `x = x*x` squares per step; the `Power` precheck above
  does not cover it.

Reported (**not** director-reproduced): an 8 MB ceiling reached a 2.04 GB peak
RSS over 20 tripling steps before terminating in band, and a 64 MB ceiling
reached 1.19 GB by repeated squaring. Treat the mechanism as established and
the magnitudes as unverified.

## What I want back

### 1. Gap A — options, then a recommendation

Enumerate the ways to bring interned bytes under the ceiling. At least these,
and **say plainly if there is a fourth I have not named** — that is the most
valuable thing you can add here:

- **A1 — session-scoped interner.** Symbol text lives with the session and
  dies with it.
- **A2 — account and refuse.** Keep the interner process-global, but make its
  bytes visible to `current_memory_proxy_bytes` and refuse to intern past the
  ceiling.
- **A3 — reclaimable interner.** Reference-counted or epoch-based entries that
  a session teardown can release.

For each, answer concretely and with `path:line`:

- What the `Symbol` identity contract actually requires. Is `&'static str`
  load-bearing beyond convenience — where does the `'static` escape into a
  signature that cannot take a lifetime? How many such signatures?
- Size and `Copy`-ness of `Symbol` today, and whether the option changes them.
  This type is in every `Expr`; a size change is a hot-path change.
- Interaction with sealed scoping's bit-62 identities and `symbol_generations`
  (`docs/dev/sealed-scoping-design.md`). **Precedent: that line already ruled
  symbol identities session-scoped and numeric reuse across sessions sound.**
  Does A1 follow from that ruling or conflict with it?
- Interaction with snapshot / `restore_from_bytes`, and with `Send`/`Sync`
  across the `rum-jupyter` threads.
- Cross-crate reach: how many call sites, in which crates, per option.
- Hot-path cost. `Expr` construction is the hottest path in the tree. State
  what the implementation's microbench A/B would have to show, and which
  labels are the right ones.

Then **recommend one and argue for it**. Cost is not an argument here — this
project's standing ruling is that the best and most correct option wins, and
"398 call sites" describes work rather than counting against it. What does
count: correctness, semantic loss, whether the fix is structurally
unavoidable, and enforceability.

### 2. Gap B — the class, not the two instances

I do not want "add a precheck to `StringJoin` and to bigint `Times`". That is
the failure brief-388 made: it closed the two instances that had been *seen*
rather than the class, because diagnosis and repair both looked only at
recorded sightings. Deliver instead:

- **A complete inventory** of every operation reachable from source-language
  evaluation whose output byte count can be a multiple of its input byte count
  in a single evaluation step, and which does not pass a projected-bytes,
  element-count, or integer-work precheck. Strings, big integers, lists,
  associations, byte arrays — wherever it goes. Give the inventory as a table:
  `path:line`, operation, growth factor per step, which precheck (if any) it
  currently passes.
- **A structural proposal** that makes an omission from that inventory fail
  loudly rather than rely on someone remembering. The standard here is
  `FrameRootTrace` (`docs/dev/plan-364-frame-roots.md`): a missing traversal
  is a *compile error*, and an invariant test records every handwritten impl.
  A fix the compiler or a derive enforces beats one that relies on people
  remembering, even an order of magnitude more expensive. Say honestly if you
  think no such structure is available for this shape, and why.
- A separate answer on the stride itself: is 64 defensible once the
  multiplicative sites are guarded, should it be adaptive, or should the
  comment simply be corrected to state what it really guarantees? Back it with
  the microbench cost of a smaller stride if you argue for keeping it.

### 3. Sequencing

Are A and B one line of work or two? Which order, and does either change the
other's design? Say so explicitly.

### 4. Drafted implementation briefs

In `TEMPLATE.md` shape, one per step, each with its own Scope (MAY-modify file
list), Deliverable, Verification and Hard rules. These are raw material — I
rewrite them before dispatch — but draft them as usable briefs, not step
descriptions. **A MAY list must include a module root whenever a type it
exports changes**; that omission has cost this project eight stop-losses.

## Method requirements

- **Every claim carries `path:line`.** A claim without one is a guess and I
  will treat it as one.
- **Cross-check any script-derived count against live probes before reporting
  it.** A regex over this repository has produced a confidently wrong count
  three separate times (attributes counted 527 where the truth was 483; two
  extraction passes that silently missed multi-line and `pub(super)` entries).
  If you report "N call sites", say how you counted and what you checked it
  against.
- **State first, verbatim, what your environment can actually run.** If a test
  or build cannot run here, say so rather than reporting a result you did not
  observe.
- Say plainly where you reason from source rather than from observation.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run `cargo` and the release `rum-cli`, and build throwaway
  crates under `/tmp` that you delete afterwards. If you believe a file must
  change, that is a finding for the report, not an edit.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Reading history with `git log` / `git show` /
  `git diff` is expected.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. Pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No vendor engine, oracle harness, or compatibility matrix.
- **No new `unsafe`** in any proposal for `rum-core` or `rum-runtime`. If you
  are convinced a design needs it, that is an open question for me, not a
  decision for you.
- **No default-path cost.** Any proposal that adds an unconditional per-call
  branch, allocation, or lock to `Expr` construction or the evaluation loop
  must say so in its own risks section, with the microbench labels that would
  prove or disprove it.
- Paste genuine verbatim output for anything you ran.

## Report format

1. Environment capability, verbatim.
2. Gap A: the options table, the answers above, the recommendation and its
   argument.
3. Gap B: the inventory table, the structural proposal, the stride answer.
4. Sequencing.
5. Drafted implementation briefs.
6. Risks and open questions — what you could not establish, and why.
