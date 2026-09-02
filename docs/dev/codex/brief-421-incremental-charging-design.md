# brief-421 — design: what `Incremental` would have to mean

Shape: **design-only**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

Gap B, step 4. This is design-only because the thing to falsify does not exist
yet, and because the change is cross-cutting enough that plan-first is the
project default.

## The state, measured

`MemoryGrowthContract::Incremental` says "retained growth must be charged as
data-dependent construction proceeds" (`metadata.rs:303-304`).

Director-measured:

- **233 declarations** carry `Incremental` across the two checked-in
  inventories.
- **`enforce_active_resource_limits_for_expr` has five callers**, and only two
  are a builtin charging its own construction: `factorial.rs:27` and `:64`. The
  other three are evaluator internals (`resolution.rs:411,490`,
  `eval_core/mod.rs:59`).
- There is **no dedicated incremental-charging primitive** anywhere in the tree.
- Worse, the one thing that looks like incremental charging is itself
  **strided**: `enforce_active_resource_limits_for_expr` does its full walk only
  on every 64th armed check (`constraints.rs:9,145-159`). So even `Factorial`'s
  in-loop polling does not bound what one step can add.

So building a falsifier now would report that 233 declarations are unbacked.
That is true and worth nothing: there is nothing for them to be backed by.

Eight builtins are already waiting on this. brief-420 retained the six `From*`
codecs plus `RumTokens` and `TokenizeWith`, each with the same recorded reason
in its own words — the output shape is known only after parsing, a per-byte
bound is sound but refuses ordinary long scalars, and what they need is an
incremental, parser-integrated check. Design for them specifically; they are
the hardest case and the one that will expose a design that only works for
loops.

## What I want back

A reviewable plan, plus drafted implementation briefs in `TEMPLATE.md` shape.
**Write no code.** Every claim carries `path:line`.

### 1. The primitive

`IntegerWorkBudget` (`crates/rum-stdlib/src/number_theory.rs:304-333`) is the
existing precedent: a small struct with `limit` and `used`, a `charge(units)`
that fails closed on overflow and on exceeding, threaded by hand through
`number_theory.rs` and `combinatorics.rs`. Start there and say whether a byte
budget should copy it, or whether a different shape — an RAII guard charging on
drop, a session method taking deltas, a builder wrapper around `Vec<Expr>` —
is better, and why. Name what you reject.

### 2. The baseline problem, which I think is the hard one

An integer-work budget is per-call and starts at zero. A **memory** budget
cannot: it must be relative to what the session already holds, and
`current_memory_proxy_bytes` is a full walk — the cost the stride exists to
avoid. Sampling once at handler entry and then charging deltas is the obvious
answer. Say whether it is sound, specifically:

- when nested evaluation inside the handler grows the session by other means;
- when the handler suspends and resumes, so construction spans two requests
  with a different session in between — brief-401 established that saved frames
  are not an unwind, and briefs 397-411 closed three defects of exactly this
  shape;
- when the handler fails partway and its partial construction is dropped.

If sampling at entry is not sound, say what is.

### 3. Parser integration — the eight codecs

`eval_from_json` (`crates/rum-stdlib/src/json.rs:11-15`) has
`session: &mut Session`; its decoder lives in `crates/rum-stdlib/src/json/decode.rs`
as recursive descent. Inventory, for all eight retained builtins, what the
decoder entry point takes today and what it would have to take. Threading a
`&mut Session` through recursive descent is one answer and probably a bad one;
say what you would do instead.

The eight already have depth guards (`MAX_JSON_DEPTH`, `next_depth`,
`MAX_TOML_DEPTH`) — director-verified, they are all guarded for nesting. Is the
byte budget the same shape of thing, threaded the same way, or different?

### 4. Cost

The unarmed path must stay free. A per-node charge is a per-node branch; say
what it costs when no ceiling is armed, and which microbench labels would prove
or disprove a regression. If your design puts a branch in `Vec<Expr>` pushes on
the ordinary path, say so plainly in the risks — that is a stop, not a detail.

### 5. The falsifier

What makes an `Incremental` declaration checkable, once the primitive exists?

Note the attribution problem before you answer: brief-419's falsifier snapshots
a counter around the handler, so charges made by *nested* builtin calls count
toward the outer handler. That was acceptable there because it makes the
instrument conservative — it under-reports rather than raising false alarms.
Say whether the same trade is acceptable here, and whether a stronger
attribution is worth building.

Also say what the rule should be. "Charged at least once" is weak: the promise
is that *un-charged growth between charges is bounded*. Is that checkable at
reasonable cost, or is "at least once" the honest first step? I would rather
have a weak rule you are honest about than a strong one you cannot enforce.

### 6. The stride

brief-414's own answer was to keep `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64`
"only after the structural inventory is enforced and every multiplicative site
is preflighted or incrementally reserved". This work is when that condition
comes due. Does landing incremental charging change the stride answer? Back it
with the recorded A/B (`docs/dev/host-embedding-line.md:107`: armed workload
3.825 s to 0.667 s), and say plainly if you did not remeasure.

### 7. Sequencing and drafted briefs

How many implementation steps, in what order, and which of the 233 declarations
does each cover. **Do not plan to reclassify anything**: brief-420's standing
rule is that a move into an unenforced contract is a deferral wearing a
repair's clothes, and it applies to moves *out* of `Incremental` too.

Draft the briefs as usable briefs, not step descriptions. A MAY list must
include a module root whenever a type it exports changes; that omission has
cost this project eight stop-losses.

## Method requirements

- **Every claim carries `path:line`.**
- **Cross-check any script-derived count against live evidence.** A regex over
  this repository has produced a confidently wrong count three times in its
  history, and I have made the same mistake five times in this line alone —
  most recently by taking the first `grep -rl` hit as a definition site when it
  was only a reference.
- **State first, verbatim, what your environment can actually run.**
- Say plainly where you reason from source rather than observation.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run `cargo` and the release `rum-cli`, and build throwaway
  crates under `/tmp` that you remove. If you believe a file must change, that
  is a finding for the report.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`** in any proposal for `rum-core` or `rum-runtime`.
- **Append your report to this brief file** before finishing. brief-414 and
  brief-418 both left theirs in the dispatch log only; the first cost a wasted
  dispatch and nearly lost the inventory this line has depended on since.

## Report format

1. Environment capability, verbatim.
2. The primitive: options, what you reject, and your recommendation.
3. The baseline problem, with your answer for each of the three cases.
4. Parser integration: the eight-decoder inventory and your design.
5. Cost, with the labels that would prove it.
6. The falsifier and the rule you propose, including its honest limits.
7. The stride answer.
8. Sequencing and the drafted implementation briefs.
9. Risks and open questions — what you could not establish, and why.
