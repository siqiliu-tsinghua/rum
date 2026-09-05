# brief-462 — convert `DeleteMissing`'s recursive-mode and preflight walks

## Goal

Make `DeleteMissing`'s recursive mode, its delayed-wrapper counters, its memory
preflight projection and its missing scan bounded by heap work rather than by
native stack depth, so that none of them can terminate the process. The
operation must still produce the exact same values at raw physical height 4,097
and above.

**The finite-target/resume family is NOT in this brief.** It is
suspension-bearing and gets its own step. Its registry row therefore stays
`UnguardedNative` when you are done, with `owed_by` naming the follow-up; do not
flip it.

## Why this is a conversion and not a guard

`DeleteMissing` does not fit the 1 MiB operation boundary at contract height —
director-measured, 3/3 kills — so a guard would convert a contract-meeting site
into a contract-failing one. A guard refuses at the budget; it cannot make work
fit inside it. Conversion is the repair for all five remaining
`UnguardedNative` sites, and this is the first of them.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest height passing 3/3 against
lowest failing 3/3. **Every mode fails open — it terminates the process.**

| mode | invocation | ok <= | fail >= | vs 4,097 |
|---|---|---|---|---|
| Recursive | `DeleteMissing[a, Infinity]`, `[a, All]`, `[a, n]` | 6,272 | 6,304 | clears 1.53x |
| Recursive, memory limit armed | the same under `MemoryConstrained` | 6,240 | 6,288 | clears 1.52x |
| Target (out of scope here) | `DeleteMissing[a, n, 0]` | 4,944 | 4,960 | clears 1.21x |
| Target, scan `All` (out of scope) | `DeleteMissing[a, n, All]` | 4,928 | 4,960 | clears 1.20x |

**Three things that measurement settled and that the earlier plans got wrong.**

1. **`DeleteMissing[expr, level]` is the *recursive* mode, not the target mode.**
   `Target` is the **three-argument** form. A two-argument "finite level" probe
   measures the recursive walker; brief-454's and brief-457's per-mode reasoning
   both rest on that distinction and a probe that gets it wrong measures the
   wrong function.
2. **The memory preflight is unreachable unless a limit is armed.**
   `enforce_delete_missing_memory_constraint` returns early when
   `session.active_memory_limit().is_none()`
   (`crates/rum-stdlib/src/associations/missing.rs:89-91`), so an ordinary probe
   cannot see `projected_expr_memory_proxy_bytes` at all. Armed, it costs
   essentially nothing extra: 6,240 against 6,272, inside the uncertainty band.
   It is still an unbounded recursion and still in scope — it is simply not the
   binding one, which is the opposite of what brief-457's risk 3 suspected.
3. **The target mode is 1.27x costlier per level than the recursive mode**, not
   the 5x that brief-454's 816 B against 160 B packets implied.

A ceiling is not deterministic: the same height flips run to run inside a band
tens of levels wide. No test may depend on one.

## Scope of the conversion

Convert every depth-recursive walk in this set. The recursion inventory built in
brief-460 is your completeness check — when you are done, none of these may
appear as an unconverted recursive edge.

- **Recursive mode**: `delete_missing_from_keyed_data_recursive`
  (`missing.rs:936`), `delete_missing_from_list_recursive` (`:950`),
  `delete_missing_list_item_recursive` (`:960`),
  `delete_missing_from_association_entries_recursive` (`:974`).
- **Delayed-wrapper counters**: `recursive_delayed_wrapper_count` (`:119`),
  `recursive_association_item_wrapper_count` (`:139`),
  `target_delayed_wrapper_count` (`:167`),
  `target_association_item_wrapper_count` (`:187`). The two `target_*` counters
  are pure counters with no suspension, so they belong here even though the
  target *driver* does not.
- **Preflight projection**: `projected_expr_memory_proxy_bytes` (`:212`).
- **Missing scan**: `contains_missing_within` (`:1038`).

Use the shape this line has settled on and that `MergeNested` and `Replace`
already demonstrate: a **bounded recursive front** plus an explicit work/finish
stack for the tail. Do not add a byte guard to these paths — the guard is for
sites that fit the budget, and these do not.

**Do not touch** `delete_missing_at_target_level` (`:331`) or any `resume_*`
function in `:356-934`. If a shared helper you convert is also called from that
family, keep the family compiling and behaviourally unchanged, and say in your
report exactly which helpers are shared and how you preserved them.

## Deliverable

### 1. The conversions

Each converted walk keeps its exact semantics: which entries are removed, in
what order, the level and scan-depth interpretation, the `Missing` predicate,
rule and `RuleDelayed` head preservation, and the delayed-wrapper accounting the
preflight depends on.

### 2. Certificates, release-only, on declared 2 MiB threads

Follow the three-part rule; `crates/rum-stdlib/src/tests/merge_native_stack_guard.rs`
and `iterative_traversal_depth.rs` are the models.

- A **unique deepest target that must propagate**: the fixture's innermost
  association carries both the `Missing` entry and a distinguishing survivor, so
  the deletion has to reach the deepest level for the assertion to hold. Assert
  the **exact** deepest value, not `Depth` or `Length`.
- Certificates at raw physical height 4,097 for: the recursive mode
  unarmed, the recursive mode **under an armed `MemoryConstrained`** so the
  preflight and counters actually run, and the missing scan.
- A **shallow companion** for each, pinning the level and scan-depth
  interpretation on a three-level fixture — that is also the falsifier if a deep
  case ever aborts the test binary.

### 3. The falsifier for the conversion itself

A conversion has no boundary override to force, so its proof is different: show
that the **bounded front actually hands off**. Force the front constant to zero
or one behind a test-only path and assert that a *shallow* branched fixture
still produces the exact same value through the heap tail, with a distinguishing
leaf on more than one branch so that a tail which drops or reorders a branch
fails. brief-458 named this shape for `Map`; it is what proves the tail is
exercised at all rather than dead code that the front always skips.

### 4. Registry and inventory

- Update the recursion inventory rows for every edge you convert.
- **The `DeleteMissing` registry row stays `UnguardedNative`** because the
  target/resume family is untouched. Update its `owed_by` to name the follow-up
  step. A row that says `BoundedHeap` while a reachable path still recurses is
  the reclassification failure this project has already paid for once.

## Scope (files)

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/missing.rs`, and a new module under
    `crates/rum-stdlib/src/associations/missing/` if the file needs splitting by
    responsibility to stay under the organization thresholds (it is 33 236 bytes
    today, inside the split-plan range);
  - `crates/rum-stdlib/src/associations/mod.rs` if exports change;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs` and/or a new
    focused test file plus its wiring in `crates/rum-stdlib/src/tests.rs`;
  - `scripts/core-language-microbench.sh`, only to add one
    `delete-missing-association-small` label if you need it for the A/B.
- Files you MUST NOT touch: the guard substrate, `kernel_api.rs`, the other four
  conversion sites, `metadata.rs`, `Expr`, `SECURITY.md`, `TODO.md`,
  `PROGRESS.md`, other `docs/**`, `Cargo.lock`.
- **The accept-vs-stop criterion, general so a file I failed to list still
  cannot admit a regression:** the ONLY intended change is that these walks stop
  growing the native stack. Any change to which entries `DeleteMissing` removes,
  to the order of its results, to its diagnostics, to the delayed-wrapper counts
  the preflight computes, or to any other builtin, is a **stop**.

## rum / Rust pitfalls (read twice)

1. **Suspension safety.** The target/resume family shares this file and its
   frames. Do not collapse a `suspend_with_frame*`; do not change any frame
   type. If a conversion would require touching a frame, STOP and report.
2. **The preflight's arithmetic must not change.** It feeds an in-band memory
   outcome; a different projected byte count is a behavior change, not a
   refactor. Assert the counts, do not eyeball them.
3. **A depth parameter on a hot function is not free** (briefs 440-442 cost
   2-7% that way). These are not evaluator-hot paths, but measure rather than
   assume.
4. Assert deep structure iteratively, never through `PartialEq`/`Display`.
5. Logical wrappers are not physical height: this fixture is two physical levels
   per wrapper, so raw 4,097 is `N=2048`.
6. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; if so, say
plainly that the nine native tests did not run in your gate.

Then the release certificates on their declared 2 MiB threads, and the
forced-front falsifier.

Then the cost gate, **counters first**:

- Allocation counts (`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base
  against candidate, for a shallow `DeleteMissing` recursive probe, an armed
  `MemoryConstrained` probe, and one untouched control. Calibrate sensitivity
  and state the same-binary spread before believing a delta.
- `perf stat -e instructions:u`, minimum of the tight clean cluster, migrated
  runs discarded, **both arms built at the same filesystem path**.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling.**
- **Stop-loss.** Stop and report if: a 4,097 certificate does not complete with
  its exact value; the forced-front falsifier does not exercise the tail; the
  preflight's projected byte counts change; a conversion would require touching
  a suspension frame; or the same step fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. Every walk you converted, and for each the front constant and the tail's work
   representation.
3. The shared helpers between your conversions and the untouched target/resume
   family, and how you preserved that family.
4. Verification: every command with verbatim output, and the ZeroMQ statement.
5. The certificates and the forced-front falsifier.
6. The registry and inventory row changes, and confirmation that the
   `DeleteMissing` row is still `UnguardedNative` with an updated `owed_by`.
7. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
8. Stop-loss and open questions.
