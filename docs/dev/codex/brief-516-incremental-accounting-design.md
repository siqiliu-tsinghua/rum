# brief-516 — design: what `Incremental` actually has to be

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and drafted implementation briefs. Write no code.

## Goal

`d1e5de5c` made every armed checkpoint walk the session proxy, which fixed a
ceiling that a loop walked straight through. Armed sessions now pay **2.26x
to 43.34x** instructions, and `eval_memory_limit` defaults to `None`, so the
cost lands entirely on hosts that asked for safety. The user ruled
(2026-09-22) to build `Incremental` rather than leave it.

Decide **what `Incremental` is**, and return the briefs to build it.

## Why this is a design brief and not an implementation

Three things are unsettled, and each changes the shape of the work.

### 1. Which design is meant — they are not the same size

- **(a) Per-builtin charging.** What the declaration literally says:
  "retained growth must be charged as data-dependent construction proceeds"
  (`crates/rum-runtime/src/runtime/metadata.rs:346-348`). **165 builtins
  already declare `Incremental`**
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs`), so this is a
  165-site obligation that nothing currently discharges.
- **(b) Incremental session total.** Maintain the session's proxy total as
  session state changes, so `current_memory_proxy_bytes()` becomes O(1) and
  a checkpoint costs O(expr) instead of O(session). Far fewer sites, and it
  is the thing that actually removes the 43x.

They are not alternatives in the obvious way: (b) may remove the *cost* while
leaving the 165 declarations undischarged, and (a) may discharge them without
making a checkpoint cheap. Say which the line is, whether both are needed,
and in what order. **Cost of implementation is not an argument** — this
project does not score an option down for its size — but scope still has to
be the problem's shape.

### 2. The models disagree, and this is the crux

The existing walker charges repeated shared subtrees **by occurrence**. A
counter of unique allocations does not reproduce that. So an incremental
counter is not a faster way to compute the same number — **it computes a
different number**, and the ceiling's meaning changes with it.

Settle it explicitly: does the ceiling mean "bytes this session would occupy
if nothing were shared" (today's model, conservative, occurrence-based) or
"bytes actually allocated" (physical, unique-based)? Say which the
documented contract promises today
(`docs/guide/resource-limits-and-sandbox.md`), which one an incremental
design can maintain cheaply, and what breaks for each existing test and
documented guarantee if the answer changes. **If the two models cannot be
reconciled, that is a director decision — surface it, do not pick silently.**

### 3. What it must survive

Briefs 415-416 made the session walk complete: 52 of `Session`'s 93 fields
walked, the other 41 carrying recorded exemptions, with a macro generating
both the inventory and an exhaustive `let Session { .. }` pattern so a new
field stops compiling until classified. An incremental total must **stay
consistent with that inventory or replace it** — and if it replaces it, it
must keep the property that a new field cannot be added silently.

Enumerate, with `path:line`, every place the total would have to change:
capacity growth, expression import and sharing, retention in definitions and
caches, prints, traces, reaps, suspension, abandonment, and release on every
exit including error and panic paths. A missed release is a monotonically
drifting counter, which is worse than a slow walk because it fails closed on
a session that did nothing wrong.

## What I want back

1. The answer to §1, argued: which design, both or one, and the order.
2. The answer to §2, with the model question named as a ruling if it cannot
   be settled from the existing documented contract.
3. The §3 inventory, produced rather than estimated, with the release paths
   called out separately from the charge paths.
4. **The falsifier design.** A drifting or under-counting total must be
   detectable, and the check must be one that can be watched failing. The
   obvious shape — assert the incremental total equals the walk — is a
   *good* falsifier only if the walk stays; say whether it does.
5. **The cost model**: what a checkpoint costs afterward, what the charge
   paths cost, and which of those numbers are measured, inferred, or guessed.
   The A/B this line will need is the instruction-count recipe, with both
   armed and unarmed workloads, since the unarmed path must not move at all.
6. Drafted follow-up briefs in `TEMPLATE.md` shape, as raw material.

## Settled — do not reopen

- The unstrided armed checkpoint (`d1e5de5c`) stays. This line makes it
  cheap; it does not restore sampling. **No stride, no adaptive policy** —
  brief-513 established last-sample adaptive stride has no useful factor
  bound.
- `Symbol`'s representation and lifetime-free public type are frozen (user
  ruling 2026-09-21).
- The interner quota line is **parked, not cancelled**, and is not this
  line's concern. The user ruled the snapshot format changes instead; that
  is a separate line to be discussed after this one.
- **No default-path cost.** `eval_memory_limit` defaults to `None` and the
  unarmed path must stay exactly as fast.

## Method requirements

- Read `AGENTS.md`, `TODO.md` §User rulings of 2026-09-22 and the
  memory-accounting entries, and briefs 513-515's reports.
- **Re-probe rather than inherit.** The audit's mechanism for the strided
  finding was wrong, and the director has been wrong twice in this line
  already. Verify anything you lean on, including the claims in this brief,
  and say how.
- Where you assert a cost, say whether it is measured, inferred or a guess.
- Name what would falsify your recommendation. **A refutation is an
  acceptable answer**: if an incremental total cannot be maintained without
  either changing the ceiling's meaning or risking silent drift, say so, and
  the armed cost becomes a documented property instead.

## Hard rules

- **No source change.** No `.rs`, no script, no fixture, no doc except
  appending your report to this file. Scratch under `target/brief-516/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. You may read everything under
  `target/brief-5*/`; overwrite nothing there.
- Bound anything long with `scripts/run-bounded.py --timeout SECONDS`, sized
  from a measured run of the same shape.
- **Stop-loss, keyed on progress.** If a probe fails twice and the second
  attempt produced no more information, stop and report what you have.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. Which design, and why.
2. The occurrence-versus-unique model answer, or the ruling it needs.
3. The charge-and-release inventory.
4. The falsifier design.
5. The cost model, labelled measured / inferred / guessed.
6. Drafted follow-up briefs, and any director decisions you hit.
