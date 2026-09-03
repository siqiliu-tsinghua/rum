# brief-427 — find the recursion behind the shared 4,500 ceiling

Shape: **diagnostic, source-level**. `Scope: NONE — no source change, not one file.`
Worker: **cdx (Codex)**.

**You do not need to reproduce any boundary.** I measured them all and they are
below. What is missing is the mechanism, and that is a question about the code.
Read, do not probe.

## What is measured, so you can spend the run on the why

brief-425 converted ten traversals to iterative walks. Six of them then share a
ceiling that the other four do not. All figures are mine, release, on a 2 MiB
stack, with values built as `x = 1; Do[x = {x}, {i, N}]`:

| Probe | Ceiling | What it tells us |
|---|---|---|
| `ToString[x]` | 43,000 | rendering is fine |
| `Level[x, {2}]` | >20,000 | shallow walk is fine |
| **`Level[x, Infinity]`** | **>20,000** | **a full-depth walk with no pattern is fine** |
| `Map[f, x]`, `Apply[f, x]` | 20,000 | fine |
| **`MatchQ[x, _]`** | **>20,000** | **one match against a deep value is cheap** |
| `SameQ[x, x]` | >20,000 | structural comparison is cheap |
| **`Cases[x, _, Infinity]`** | **4,500 / 4,700** | trivial pattern, still short |
| `Cases[x, _List, Infinity]` | 4,500 / 4,700 | same |
| `Position[x, 1]` | 4,500 / 4,700 | same |
| `Count`, `MemberQ`, `FreeQ`, `ReplaceAll` | 4,500 / 4,700 | same |

Three eliminations follow and I want them treated as settled:

- **Not the walk.** `Level[x, Infinity]` descends every level and clears 20,000.
- **Not the matcher's cost on a deep value.** `MatchQ[x, _]` clears 20,000.
- **Not the pattern.** `Cases[x, _, Infinity]` with the trivial `_` stops at the
  same 4,500 as `_List`.

What is left is the thing the six do and the four do not: **invoke the matcher
once per node while walking every level.** A single invocation is cheap and
4,500 of them do not fit a 2 MiB stack, which means each per-node invocation leaves
something on the native stack that is not released until the walk finishes.

## Deliverable 1 — the mechanism, from source

Find what that retained thing is. Compare, in the code:

- `Level` (`crates/rum-runtime/src/runtime/tree_walkers/level.rs`) — walks every
  level, no per-node match, clears 20,000;
- `Position` (`tree_walkers/position.rs`) and `count_like.rs` (`MemberQ`,
  `Count`, `FreeQ`) — same module, same post-order cursor
  (`tree_walkers/post_order.rs`, added by brief-425), but short;
- `Cases` (`kernel_data/rewrite_data/cases/collect.rs`) and `ReplaceAll`
  (`definitions/application/replace_all.rs`) — different modules, same ceiling.

Two of those live in the same module and use the same cursor and differ only in
whether they match, so the cursor is not the answer either. Say what is.

Answer concretely: which function, which frame, what it holds, and why the
iterative cursor does not release it per node. `path:line` for each claim.

## Deliverable 2 — whether it is fixable, and how expensive

Given the mechanism, say what would remove it, and price it against what this
line already knows:

- brief-425's conversions were free or better — `position-wide-no-match` got
  23.5% faster — so an iterative rewrite in this area has a good record.
- brief-426's cached height cost up to 30% because it put an O(n) scan in
  `SharedArgs::new`, and the stop-loss fired. **Do not propose anything that
  adds per-construction or per-node work to an ordinary path** without saying
  so plainly in the risks.

If the honest answer is that removing it is not worth what it costs, say that.
A ceiling of 4,500 already clears the ruled bound of 4,097; the reason to lift
it is margin, not correctness, and margin can lose to cost.

## Deliverable 3 — classify the untested tail by the same mechanism

brief-422's appended report §3(e) lists rows that were source-verified and never
measured: string-pattern machinery, calculus and polynomial walkers, schema
conformance, `Flatten[_, Infinity]`, tensor rank recursion, `Merge` nested,
substitution walks, matcher compile and structural match, rewrite traversals,
`lowering.rs`, and suspension continuation encoding.

Now that you know the mechanism, **classify each of those rows against it**:
does it invoke a per-node match inside an all-levels walk, does it recurse for
some other reason, or is it bounded already? This is a source-reading task and
it is what decides the next brief's scope, so I would rather have an honest
"unclear, needs measurement" per row than a confident guess.

## Method requirements

- **Every claim carries `path:line`.**
- **You are reading, not probing.** If you believe a specific claim cannot be
  settled from source, name it and say what measurement would settle it — do not
  construct one.
- **Cross-check any script-derived count against the source.** A regex over this
  repository has produced confidently wrong counts, and I have made the same
  class of error repeatedly on this line, most recently by measuring
  `Level[x, {2}]` and nearly concluding `Level` was fine for the wrong reason.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. If you
  believe a file must change, that is a finding for the report.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config.
  **`stash@{0}` is brief-426's rejected spike and `stash@{1}` is brief-400's
  WIP; leave both alone.**
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Append your report to this brief file** before finishing.

## Report format

1. Environment capability, verbatim.
2. The mechanism: function, frame, what it retains, why the cursor does not
   release it, with `path:line`.
3. What would remove it, and its price, with the brief-426 lesson applied.
4. The §3(e) classification table: row, mechanism, or "unclear — needs
   measurement".
5. Anything you could not establish from source, and what would settle it.
