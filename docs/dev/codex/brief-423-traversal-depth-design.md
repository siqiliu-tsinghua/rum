# brief-423 — design: bounding traversal over deep values

Shape: **design-only**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

Opened by brief-422, which bounded four recursive descents and found that the
class behind them is not a missing depth guard but a structural gap. Deciding
between the available shapes is a design question, and plan-first is this
project's default for anything cross-cutting.

## The state, measured

The evaluator's own recursion is bounded: `recursion_limit` 1024 plus a 6 MiB
native-stack poll every four levels (`eval_core/mod.rs:34,39-42,68-78`). That
guard covers the evaluator descending into an expression it is evaluating. It
does **not** cover a traversal walking a value it was handed.

Two facts combine badly.

**A source-language script builds arbitrarily deep values.** `x = 1;
Do[x = {x}, {i, N}]` reaches N of roughly 80 000. The wrap is iterative and the
stored value is never re-descended, so neither the iteration limit nor the
recursion limit applies. (`Nest` does not do this — it re-evaluates and stops at
the iteration limit, which is why an earlier probe of mine wrongly concluded the
encoders were unreachable.)

**Every per-node traversal then walks that value with nothing counting.**
Measured stack-exhaustion thresholds, release build, from brief-422's inventory
and my own reproduction:

| Traversal | Exhausts the native stack around |
|---|---|
| `ToString`, `Print`, `FullForm` | 6 000 – 20 000 |
| `Map`, `Apply`, `Scan`, `ReplaceAll`, `Cases`, `Level`, `MemberQ`, `Count`, `FreeQ`, `Position`, `Save`, `DeleteMissing[_, Infinity]` | 20 000 – 50 000 |
| `Depth`, `LeafCount`, `ByteCount`, sort comparators, drop glue | survive 80 000; lean, but equally unbounded |

brief-422's report §3(e) lists many more source-verified rows it did not probe
individually: string-pattern machinery, calculus and polynomial walkers, schema
conformance, `Flatten[_, Infinity]`, tensor rank recursion, kernel-side
`PartialEq`/`Clone`/`Hash`/`Serialize` on `Expr`, substitution walks, matcher
compile and structural match, rewrite traversals, `lowering.rs`, and suspension
continuation encoding. Read that section; do not re-derive it.

## A documented claim that is now false

`TODO.md:1061-1065` and, more importantly,
`docs/host-embedding-and-hardening.md:113-117` tell embedders:

> Source-created trees are protected by parser, evaluator, matcher, and
> native-stack limits. A host can still construct a pathologically deep `Expr`
> directly through the Rust API and overflow the stack during recursive drop.
> … hosts must depth-limit directly constructed trees.

The first sentence does not hold. A source-created tree reaches 80 000 levels
through the loop above, and the traversals fail far below the depth at which
drop does — I confirmed drop itself survives 80 000. So the document tells a
host the risk is confined to trees it builds in Rust, and that is the wrong
advice. Your plan must include correcting both texts, and the correction has to
follow whatever the design decides rather than being written first.

## The prior art that constrains the answer

**Three safe iterative-`Drop` designs regressed the ordinary hot path by 20-50%
and were rejected** (`TODO.md:1061-1065`,
`docs/host-embedding-and-hardening.md:115-117`). So "make the traversals
iterative" is not an untested idea here; the closest previous attempt has a
measured counterexample.

**But plan-330 made sixteen operator chains iterative successfully**
(`docs/dev/codex/plan-330-left-chain-lowering.md`), with a 4 096-link ceiling
proved through the full round trip. So the technique is not disqualified either.

Explain why those two outcomes differ before you recommend anything. That
explanation is the most useful thing you can give me, because it predicts which
traversals can be converted cheaply and which cannot.

## What I want back

A reviewable plan plus drafted implementation briefs in `TEMPLATE.md` shape.
**Write no code.** Every claim carries `path:line`.

### 1. The options

At least these, and say if there is a fourth:

- **A — admit depth at construction.** Bound how deep a value may become, so
  every traversal downstream is safe by construction. State where the check
  would live, what it costs on `Expr` construction — the hottest path in the
  tree — and how it interacts with `SharedExpr` reuse, where a subtree's depth
  is not known from the node being built.
- **B — make the traversals iterative.** Per the prior art above, with an
  estimate of how many traversals that is, drawn from brief-422 §3(e).
- **C — a shared depth-counting guard traversals opt into.** Cheaper than
  converting them, more invasive than A. Say what it costs per node and how an
  omission would be caught, since a guard nothing enforces is the failure this
  line keeps repairing.

For each: what breaks, what it costs, what enforces it, and whether an omission
is a compile error, a test failure, or nothing.

### 2. The behaviour question, which is mine but needs your evidence

`Depth[x]` returns 80001 today for the loop above, and `Depth`, `LeafCount` and
`ByteCount` survive it. A construction-side bound would refuse to build that
value at all; a traversal-side bound would keep building it and refuse to
render it.

Tell me what each option does to currently-working behaviour, concretely: which
expressions that succeed today would stop succeeding, and at what depth. I will
make the ruling; I need the consequences priced first.

Note also that the renderers are **error-message and trace infrastructure**, not
only user-facing builtins. Say what a refusal even means for `ToString` when it
is called to format a message about another failure.

### 3. Cost

Whatever you recommend, name the microbench labels that would prove or disprove
a regression, and say which of them the rejected iterative-`Drop` designs moved
by 20-50%, if that is recoverable from the repository. Profiling env off for any
timing number you quote; say plainly if you did not measure.

### 4. Sequencing and drafted briefs

Ordered steps, what each covers, and drafted briefs. A MAY list must include a
module root whenever a type it exports changes; that omission has cost this
project eight stop-losses.

Include the documentation correction as an explicit step, not a footnote.

## Method requirements

- **Every claim carries `path:line`.**
- **Cross-check any script-derived count against live evidence.** A regex over
  this repository has produced a confidently wrong count three times, and I have
  made the same class of error six times in this line — including summing two
  inventories without checking their overlap, and taking the first `grep -rl`
  hit as a definition site when it was only a reference.
- **State first, verbatim, what your environment can actually run.**
- Say plainly where you reason from source rather than observation.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run `cargo` and the release `rum-cli`, and build throwaway
  crates under `/tmp` that you remove. If you believe a file must change, that
  is a finding for the report.
- **Do not propose raising the native stack or moving work onto a bigger
  thread.** That trades the threshold for nothing structural.
- **No new `unsafe`** in any proposal for `rum-core` or `rum-runtime`.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Append your report to this brief file** before finishing.

## Report format

1. Environment capability, verbatim.
2. Why iterative `Drop` failed and plan-330 succeeded.
3. The options, with cost, enforcement, and what breaks.
4. The behaviour consequences of each, priced concretely.
5. Cost: the labels, and any recoverable numbers from the rejected designs.
6. Sequencing and the drafted implementation briefs, including the
   documentation correction.
7. Risks and open questions — what you could not establish, and why.
