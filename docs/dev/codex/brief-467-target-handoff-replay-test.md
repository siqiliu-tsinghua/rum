# brief-467 — make the handoff-replay test actually detect a replay

## Situation

**The working tree contains brief-466's uncommitted conversion. Build on it; do
not restart or revert it.** The conversion is correct — the director verified
both axes at the CLI (width 3,000 and 64,000 now complete where 3,000 used to
terminate the process; depth 4,097 and 8,000 both complete), all ten semantic
locks pass unmodified, the four release certificates pass, and allocation counts
are byte-identical to base on a shallow probe, a 2,000-sibling wide probe and an
untouched control.

**One thing is wrong, and it is the thing brief-466 called the crux of the
step.** The handoff that must carry the live continuation and completed
prefixes, rather than restarting the traversal, is **not protected by any
test**.

Director-measured: perturbing
`delete_missing_at_target_level_with_front_depth` so that an `Exhausted`
traversal discards its carried work and re-drives a fresh root cursor with an
unbounded front — the exact failure the design forbids — leaves **890 rum-stdlib
tests green**, including the ten semantic locks, both in-module handoff tests,
and the four release certificates.

## Why the existing tests miss it, precisely

The gap is specific and worth stating because the repair follows from it. **No
test combines a real front exhaustion with an observable side effect in the
completed prefix.**

- `target_front_handoff_does_not_replay_completed_delayed_prefix`
  (`crates/rum-stdlib/src/associations/missing/target.rs:771`) has the side
  effect but **fakes the exhaustion**: it drives the prefix cursor itself, then
  hand-assembles an `OwnedListCursor { index: 1, kept: vec![first_result] }` and
  drives that. It never calls the real entry point, so it cannot observe what
  the real entry point does on exhaustion. It is a valid unit test of
  `resume_delete_missing_target_work` in isolation, and its name claims much
  more than that.
- The release certificates exhaust the front for real, but their fixtures
  contain no side-effecting delayed RHS, so a restart redoes pure work and
  produces the same value. Their assertions cannot see it.

## Deliverable 1 — the integration test

One test, going through the **real entry point**, whose fixture has both
properties at once:

- a **completed prefix with an observable side effect**: an early association
  entry whose `RuleDelayed` RHS appends a distinct token to a log;
- a **later branch deep enough to exhaust the depth-16 front for real** —
  remember target mode descends only to `level`, so the call needs a target
  level above the front (say 24) and a nested structure at least that deep;
- an assertion on the **exact ordered log**, so the prefix token appearing twice
  fails.

Assert the exact resulting value as well, so the test also pins that the restart
is not merely invisible but that the carried path produces the right answer.

Keep the fixture shallow and narrow enough to stay far from both measured
ceilings (depth 4,944 physical, width 2,921 siblings).

## Deliverable 2 — prove it fires, with this exact perturbation

Apply this to `delete_missing_at_target_level_with_front_depth`, capture the new
test red verbatim, then revert it:

```rust
let traversal = drive_owned_target_cursor(session, cursor, remaining_call_depth);
// perturbation: discard carried work and restart from the root
let traversal = match traversal {
    Ok(TargetTraversal::Exhausted(_)) => {
        let restart = if let Some(items) = list_items(expr) {
            OwnedTargetCursor::List(OwnedListCursor {
                items: items.to_vec(), level, scan, index: 0, kept: Vec::new(),
            })
        } else {
            OwnedTargetCursor::Association(OwnedAssociationCursor {
                entries: association_entry_forms_from_expr(expr).unwrap_or_default(),
                level, scan, index: 0, kept: Vec::new(),
            })
        };
        drive_owned_target_cursor(session, restart, usize::MAX)
    }
    other => other,
};
finish_bounded_target_traversal(session, traversal).map(Some)
```

**If your new test does not go red under it, the test does not work and the step
is not done.** Do not adjust the perturbation to make the test fail; adjust the
test.

## Deliverable 3 — rename the misleading test

`target_front_handoff_does_not_replay_completed_delayed_prefix` should keep
existing — it is a sound isolation test — but its name must say what it covers:
that `resume_delete_missing_target_work`, **given** a continuation carrying a
completed prefix, does not re-run that prefix. A name that claims a property the
test cannot observe is worse than no test, because it stops anyone else from
writing the one that can.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/missing/target.rs` — **only** its
    `mod tests` block (the new test, and the rename). Its production code is
    off limits except transiently for the perturbation above, which you revert.
  - `crates/rum-stdlib/src/tests/delete_missing_target_certificates.rs` and its
    wiring, if you prefer the integration test to live there — say which you
    chose and why.
- Files you MUST NOT touch: `delete_missing_target_semantic_locks.rs`, the
  registry, the recursion inventory, every other production file, `metadata.rs`,
  `Expr`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other `docs/**`,
  `Cargo.lock`.
- **Accept-vs-stop:** this brief adds one test and renames another. Any
  production change surviving to the end is a **stop**. `git diff --stat` must
  show brief-466's conversion plus test-only additions.

## rum / Rust pitfalls (read twice)

1. **Target mode is the three-argument form**, and it descends only to `level`.
   A fixture deeper than `level` does not exhaust anything below it.
2. The front constant is 16. A fixture that descends fewer than 17 levels never
   exhausts, and a test that never exhausts cannot see this bug — that is
   exactly how the current tests miss it.
3. Assert the **exact ordered log**, not a count and not a set.
4. Stay far from both measured ceilings.
5. Wall-clock timing is not evidence on this host, and nothing here needs it.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

Then the perturbation red and the restored green, both verbatim, and
`git diff --stat`.

No performance gate: no production behaviour changes in this brief.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT leave any production edit in the tree.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Stop-loss.** Stop and report if the new test cannot be made to fail under
  the given perturbation, or if making it fail would require changing production
  code.
- Paste real verbatim output.

## Report format (your final message)

1. What changed, per file, with line ranges.
2. The new test: its fixture, why it exhausts the front for real, and what it
   asserts.
3. The perturbation red and the restored green, verbatim.
4. The rename, and the property the renamed test actually covers.
5. Verification output and `git diff --stat`.
6. Stop-loss and open questions.
