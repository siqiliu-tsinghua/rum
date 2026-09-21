# brief-515 — finish the unstrided armed ceiling

## Goal

brief-514 built the repair and it works: an 8 MB `MemoryConstrained` now
aborts the doubling that previously returned 65 536 000 characters, and the
falsifier was watched failing with the stride restored. **Its default-path
gate has since passed** (below). Two mechanical blockers remain. Clear them
and bring the change to a green workspace.

## Where the work is

**brief-514's changes are already staged in your working tree** — the
director applied them from branch `spike/brief-514-unstride` (`86aad05d`).
Do not reconstruct them, and do not use `git` to switch, merge, cherry-pick
or restore. Inspect what is there with `git diff --cached` / `git diff` and
build on it.

The files carrying the repair are:

- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`
- `crates/rum-runtime/src/runtime/model/session.rs`
- `crates/rum-runtime/src/runtime/session/session_helpers.rs`
- `crates/rum-runtime/src/runtime/session/session_state.rs`
- `crates/rum-runtime/src/runtime/tests/session_memory.rs`
- `docs/guide/resource-limits-and-sandbox.md`

brief-514's report in `docs/dev/codex/` describes each change with line
ranges if you need the reasoning behind one.

## The default-path gate is settled — do not re-litigate it

**Director-established 2026-09-22.** Instruction counts are flat:
−0.000387% to +0.000948% across six calibrated workloads, once the armed
walk sits in a cold helper. The timing panel's apparent regression is
**indistinguishable from the panel compared with itself**: splitting
brief-514's own four base runs two ways gives 69/120 and 72/120 positive
medians, 28 and 30 labels consistently slower, and a worst outlier of
**+18.43%** — against the candidate's 64/120, 9, and +16.66%. The label
flagged hardest against the candidate moves +18.43% between two runs of the
*identical* binary.

So: **do not re-run the timing panel, and do not tune for it.** If you change
anything that could move the default path, say so and stop; otherwise the
gate stands.

## Blocker 1 — the ownership inventory

Three `rum-stdlib` ownership-graph tests fail because source locations moved.
Regenerate the inventory. **This is a disclosure duty, not a free hand:**
list every content change with its witness, classified, and **anything you
cannot classify is a stop**. A line-number-only shift and a genuinely new or
vanished construct are different things and must be reported differently.

Scope exception granted for this: the inventory fixtures under
`crates/rum-stdlib/src/tests/ownership_graph/` and whatever the regeneration
tooling writes.

## Blocker 2 — the falsifier's home

brief-514's falsifier proves the repair and has no permanent home, because
that brief excluded `rum-stdlib`. Give it one. Scope exception granted for
the runtime or stdlib test module that owns resource-limit behavior; put it
where the neighbouring resource-limit tests live rather than creating a new
top-level file if one already fits.

It must keep both halves of what makes it a falsifier: it fails if the
repair is absent. Keep the fixture small — the seed and doubling count only
need to cross the ceiling, not reach 65 MB — and state the fixture's runtime.

## Also finish

- **The false comment** at `constraints.rs:5-8`, if brief-514 did not already
  replace it: it claims per-step element, integer and iteration budgets bound
  the overshoot. They bound counts; the quantity at risk is bytes. Replace it
  with the guarantee the new code actually makes, **including** that a single
  construction larger than the limit is still only caught where a projected
  precheck exists.
- Confirm whether removing the countdown field changed `Session`'s layout,
  and say so either way.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

**Target: 0 failures** other than environmental. On `main` the suite is
2 491 passed, 0 failed, 57 ignored on a host that can bind loopback. Run
**without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if your sandbox denies loopback
you will see nine native ZeroMQ failures — report them as environmental and
say plainly those tests did not execute. **Any other failure is yours**, and
the three inventory tests must now pass.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`; a recorded
run of the same shape took about 160 s.

**No timing panel.** See above.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. Leave changes in the
  working tree; the director gates and commits.
- **No new `unsafe`.**
- **No default-path cost** and no new sampling policy. The repair is the one
  brief-514 built; you are finishing it, not redesigning it.
- **Do not weaken or delete a test to make the suite pass**, and do not
  silence an inventory row you cannot explain.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Do not overwrite anything
  under `target/brief-501/`, `target/brief-502/`, `target/brief-508/`,
  `target/brief-513/` or `target/brief-514/`.
- **Stop-loss, keyed on progress.** If the same step fails twice and the
  second attempt produced no more information, stop and report.
- Paste real verbatim output, including failures.

## Report format

1. How you reconstructed brief-514's changes, and anything that differed.
2. The inventory regeneration: every content change with its witness,
   classified; anything unclassifiable, named.
3. The falsifier's home, its runtime, and proof it still fails without the
   repair.
4. The comment replacement and the `Session` layout answer.
5. Verification, verbatim, with the complete failure set.
6. Anything you refused, and open questions.
