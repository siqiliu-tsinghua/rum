# brief-434 — decompose the height cache's cost before redesigning it

**Shape: diagnostic. Nothing you build here is kept.** The deliverable is a
cost attribution and one measured alternative.

## Why this brief exists

brief-426 built a cached physical height, measured `set-part-fixed100-len800`
at −30.04% and repeated list append at 8.5-8.9%, fired the stop-loss, and
concluded "the height is not affordable in this layout". That conclusion is
sound as far as it goes, but **the spike changed three things at once and the
verdict attributed the cost to one of them without separating them**:

1. **Allocation growth.** Both private Arc allocations grew 8 bytes —
   `SharedExpr::new` 48→56, `SharedArgs::new` 40→48 (brief-426 §2).
2. **A constructor scan.** `SharedArgs::new` went from `Arc::new(args)` to
   `max_physical_height(&args)`, an O(arity) pass in which every non-atom child
   costs two Arc dereferences to read its cached height.
3. **A rescan on every mutation.** `mutate_shared_args` installs a
   `RefreshHeightOnDrop` guard that recomputes the full O(width) maximum after
   *any* mutable access, and `get_mut`/`make_mut` are the closure-passing API
   every in-place edit goes through.

The label that regressed 30% is a Part assignment into an 800-element list —
the in-place `make_mut` path that plan-297 made O(1) per assignment. Under the
spike each of those assignments ends in an 800-element rescan. **My hypothesis
is that (3) is nearly all of the cost and (1) and (2) are small**, which would
mean the design is affordable and only its maintenance strategy was wrong. I do
not want that hypothesis confirmed by agreement; I want it measured, and I want
to be told if it is wrong.

## Your starting point

The working tree already contains the complete brief-426 spike, restored from
the stash. That is deliberate, not a scope violation. It compiles and its own
tests pass.

## Deliverable — five binaries, one window

Build these and measure them against each other. Nothing here is committed;
work in the tree and report numbers.

| Variant | What it is |
|---|---|
| **V0** | the committed tree, no spike (you will need to set the spike aside to build this — see below) |
| **V1** | spike with the height field present but **never computed**: `SharedArgs::new` does not scan, the mutation guard does not rescan, and every height is a fixed constant. The accessor still returns it so nothing is dead-code eliminated. Isolates allocation growth. |
| **V2** | V1 **plus** the constructor scan in `SharedArgs::new`. Isolates the constructor scan. |
| **V3** | V2 **plus** the mutation rescan — i.e. the unmodified spike. Isolates the mutation rescan. |
| **V4** | V2 plus an **O(1) conservative mutation update**: instead of rescanning, raise the cached maximum to `max(old_max, height_of_what_was_written)`. |

V4 is the design question. A cached maximum that only ever rises is still a
sound admission bound — it can refuse slightly early after a value shrinks, but
it can never admit something too deep. If V4 lands near V2, the height is
affordable and brief-426's verdict was about its maintenance strategy, not the
representation.

For V4 you need the height of what was written, which is available at the write
site. Where an edit does not expose that cheaply, take the conservative route
(leave the maximum unchanged only if you can argue the write cannot raise it;
otherwise saturate to the limit). Say exactly what you did per call site.

**Building V0.** You may not run git commands. Instead: build V0 **first**,
before you touch anything — the tree as you received it minus the spike is not
available to you, so build the spike variants first and tell me you need V0,
**or** simply report V1-V4 against each other and I will supply V0 myself. Do
not guess at V0's numbers.

## The measurement protocol — this is the part that matters

brief-431's gate produced a stop-loss that was half noise because a single
base/candidate pair cannot resolve a few percent on this panel. Use all three
instruments:

1. **Two passes per variant, all ten passes in one window**, so every
   comparison is within-session. Report the same-binary spread per label from
   the two passes of the same variant, first.
2. **Within-run quotients.** For each affected label, divide it by a same-tier
   untouched neighbour *from the same run*, then compare quotients across
   variants. This is immune to the whole-run offset by construction, and it is
   what settled the brief-433 gate when control-median adjustment moved the
   whole panel one way. Same-tier neighbours to consider: `table-count-null`,
   `list-literal-small`, `string-repeat-small`, `keysort-callback-large`,
   `sortby-callback-small`, `table-iterator-plus`, `module-chain-escape-large`.
3. **Report a movement as real only if it exceeds that label's own same-binary
   spread**, and say so per row rather than reporting every delta as a finding.

**The panel that decides this brief**: `set-part-fixed100-len100`,
`set-part-fixed100-len800`, `appendto-list-incremental-small`,
`appendto-list-incremental-large`, `appendto-part-outer1000-small`,
`appendto-part-outer1000-large`, plus every list-construction row and the
`matcher-single-*` and `cases`/`count`/`position`-wide rows as neighbours.
Controls: `literal-integer`, `literal-string`, `symbol-ownvalue-read`,
`loop-empty`, `sameq-integer`, `head-list`, `length-list`, `part-list`,
`if-true`, `module-local`, `block-local`, `with-local`, `table-small`,
`do-small`.

**Allocation counters before timings.** Counters are load-immune; if the
allocation counts do not move between two variants, say so and treat any timing
difference between them with suspicion.

## What I want concluded

1. **The attribution**: how much of brief-426's regression is allocation
   growth, how much the constructor scan, how much the mutation rescan. Numbers,
   per label, with the same-binary spread beside them.
2. **Whether V4 is affordable** — near V2, or not.
3. **Any call site where the conservative O(1) update cannot be applied**, with
   `path:line`, and what it would cost there.
4. If my hypothesis is wrong — if the constructor scan or the allocation growth
   carries the cost — **say so plainly**. A refuted hypothesis here is the most
   valuable result this brief can produce, because it closes the design rather
   than sending me around the loop again.

## Scope

- You may modify anything needed to build the five variants, **but nothing is
  kept**: the tree you leave behind should be V4 (the most promising variant),
  and I will decide what survives. Do not delete or rewrite tests to make a
  variant compile — if a test blocks a variant, say so and skip that variant.
- Do not touch `docs/**` except to append your report to
  `docs/dev/codex/brief-434-height-cost-decomposition.md`.
- **No enforcement, no limit constant, no refusal path, no new public API.**
  This brief measures maintenance cost only. The bound itself is a later
  decision.

## Hard rules

- Do NOT `git commit`, branch, push, stash, checkout, restore, or otherwise run
  git. The working tree's contents are deliberate.
- No network. Touch nothing outside this repository.
- **No new `unsafe`**, including `unsafe` attributes.
- **Write no probe that builds a deep value to see it fail.** This brief is
  timing and counters only; there is no stack-overflow reproduction anywhere in
  it, and none is wanted.
- Profiling env OFF for every timing number. Bound long commands with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run.
- Keep the probe count bounded and state the cap. If a number looks wrong,
  report it rather than silently re-measuring everything.
- Paste real verbatim output. If a variant did not build or a run did not
  complete, say so.
- **Append your report to this file** before you finish.

## Report format

1. The five variants — what each one changed, per file and line range.
2. Allocation counters per variant, for the deciding panel.
3. The timing table — same-binary spread column first, then per-variant
   within-run quotients.
4. The attribution: the three components, with numbers.
5. V4's verdict, and any call site where the O(1) update could not be applied.
6. Whether my hypothesis survived. Say it in one sentence before explaining.
