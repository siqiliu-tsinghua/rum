# brief-466 — convert `DeleteMissing`'s finite-target/resume family

## Goal

Remove every unbounded native edge from `DeleteMissing`'s finite-target mode and
its resume family, on **both** axes, and flip its registry row to
`BoundedHeap`. This is the last natively recursive path in the builtin and step
B of brief-464's plan.

When you are done, `DeleteMissing[Table[Missing[], {64000}], 1, 0]` completes and
a target-mode traversal at raw physical height 4,097 produces its exact value.

## The acceptance criterion, stated first because it governs everything else

**brief-465's ten tests in
`crates/rum-stdlib/src/tests/delete_missing_target_semantic_locks.rs` are the
contract. They must pass unmodified.**

They were written against the untouched implementation precisely so this step
has a net: they pin visit order, the rule-like short circuit ahead of the scan,
the syntactic-`Missing` drop before any delayed evaluation, **each reached
`RuleDelayed` RHS evaluated exactly once**, the `level - 1` delayed wrapper, leaf
and head preservation, resume without prefix replay at three sibling positions,
two suspensions in different branches, error propagation, and cancellation
discarding frames without callbacks.

**Editing one of those tests is a stop, not a repair.** If your conversion
cannot satisfy a lock, report which lock and why and stop — that is a design
finding and it is mine to settle, not yours to legislate away. The director has
verified the net is selective: perturbing the post-suspension resume to restart
its level at index 0 turns exactly the four resume/replay tests red and leaves
the other six green.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`. Highest value passing 3/3 against lowest failing 3/3.
**Both axes currently terminate the process.**

| axis | shape | ok <= | fail >= |
|---|---|---|---|
| depth | `DeleteMissing[a, n, 0]` over a nested association | 4,944 physical | 4,960 |
| depth, scan `All` | `DeleteMissing[a, n, All]` | 4,928 physical | 4,960 |
| width | `DeleteMissing[Table[Missing[], {N}], 1, 0]` | 2,921 siblings | 2,968 |
| width, association | the same over an `Association` of N entries | same | same |

For reference, the recursive mode converted in this same file by briefs 462-463
now clears **64,000** siblings and its depth certificate at 4,097. One
conversion closed both of its axes; this brief does the same for target mode.

## The shape — and the one place it differs from every previous conversion

1. **Sibling advance becomes an explicit `loop`.** Never rely on tail-call
   elimination for an `index + 1` edge. That assumption is what produced the
   width defect: brief-454 recorded these edges as "loops or tail transitions",
   and Rust does not guarantee it.
2. **Child descent gets a depth-16 recursive front**, decremented only on a
   `level - 1` edge, with a cold explicit work stack beyond it.
3. **The exhausted handoff must carry the live continuation and the completed
   prefixes. Restarting the traversal from the root is forbidden.** Every
   previous conversion in this line — `Replace`, `DeleteCases`, `MergeNested`,
   and `DeleteMissing`'s own recursive mode — lets the cold tail redo the walk,
   which is sound only for a pure walker. Here a target-level delayed evaluation
   runs user code, so a redo would run it twice. Return `Exhausted` carrying
   work **before** an over-front child is visited, and while that unwinds
   through the bounded parents, wrap each live parent cursor and finish task so
   the tail resumes at the first unvisited child with completed prefixes intact.

   This is the crux of the step and the thing brief-465's replay tests exist to
   catch.
4. **On `InputRequired`, freeze the entire work vector in one traceable,
   pure-abandon frame**, and resume the driver with the supplied value as the
   completed result — the shape `MergeNested` uses
   (`crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:134-203`).
5. **Keep mutable `Vec` storage on the ordinary shallow path.** Freeze inputs
   and prefixes only on real suspension or cold-tail handoff.
   `PersistentAssociationSequence::appended` allocates one `Arc` node per
   retained item (`crates/rum-stdlib/src/associations/suspended_state.rs:72-80`),
   which on the wide ordinary path would be a per-element allocation this file
   has already been caught adding once.
6. **Split the module.** `missing.rs` is 26 353 bytes, inside the split-plan
   range, and this work moves the target machine; put it in
   `crates/rum-stdlib/src/associations/missing/target.rs` (or a directory of
   responsibilities if it would exceed the line). Nothing may end up over
   35 000 bytes.

## Certificates — release-only, on declared 2 MiB threads

- **Depth**: a target-mode traversal at raw physical height 4,097 with a unique
  deepest target, asserting the **exact** rebuilt value. Cover both `scan`
  settings. A three-level shallow companion pins the level and scan
  interpretation.
- **Width**: `DeleteMissing` over **64,000** siblings, both the list and the
  association shape, asserting the exact result. That number is not arbitrary —
  it is what the converted recursive mode already clears, so it is the standard
  this conversion is held to.
- No test may sit near a ceiling; a ceiling is not deterministic and flips run
  to run inside a band tens of levels wide.

## The forced-front falsifier

In-module, no cargo feature and no production switch — the shape brief-463
settled on. Add a private `*_with_front_depth` entry and assert, on **one
shallow branched mixed-container fixture** with several entries per association
and a distinct witness per level:

- front `0` returns `Exhausted` **carrying non-empty work**;
- front `16` returns `Complete`;
- **tail == front == exact expected value**.

Then extend it with what this conversion specifically risks: a fixture whose
front exhausts **after** a delayed RHS has already run, asserting the
side-effect log contains that RHS **exactly once**. A handoff that restarts
would show it twice, and this is the only test that can catch that at shallow
depth.

## Registry and inventory

Flip `DeleteMissing` to `Mechanism::BoundedHeap` **only if both axes are
closed**, with `failure_direction: Some(FailureDirection::Unmeasured)` and
`owed_by: None`, matching the `MergeNested` row
(`crates/rum-stdlib/src/tests/association_depth_contract/registry.rs:389-396`).
`Unmeasured` is the honest entry for a heap-bound site: it no longer fails on
this axis at any height we can build. Update the inventory rows for every edge
you change; the inventory is your completeness check that no unbounded edge
survives.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/missing.rs` and new modules under
    `crates/rum-stdlib/src/associations/missing/`;
  - `crates/rum-stdlib/src/associations/missing/resume_frames.rs`, if the frame
    set genuinely changes — say why in the report;
  - `crates/rum-stdlib/src/associations/mod.rs` if exports change;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - a new focused certificate test file plus its wiring in
    `crates/rum-stdlib/src/tests.rs`;
  - `scripts/core-language-microbench.sh`, only to add one target-mode label if
    you need it.
- Files you MUST NOT touch:
  **`crates/rum-stdlib/src/tests/delete_missing_target_semantic_locks.rs`** — the
  locks; the guard substrate; `kernel_api.rs`; the other four conversion sites;
  the already-converted recursive-mode walks under `missing/traversal/`;
  `metadata.rs`; `Expr`; `SECURITY.md`; `TODO.md`; `PROGRESS.md`; other
  `docs/**`; `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended change is that these walks stop growing the
  native stack on either axis. Any change to what `DeleteMissing` removes, to
  result order, to callback count or order, to diagnostics, to suspension
  behaviour, or to any other builtin is a **stop**.

## rum / Rust pitfalls (read twice)

1. **`DeleteMissing[expr, level]` is the recursive mode.** Target is the
   **three-argument** form. A two-argument probe tests code you are not
   changing.
2. **The memory preflight is unreachable unless a limit is armed**
   (`crates/rum-stdlib/src/associations/missing.rs:89-91`), and its walks were
   already converted. Do not re-convert them.
3. **A per-element allocation on a wide path is a regression even if nothing
   else changes.** This file was caught adding one allocation per association
   level two briefs ago, found only by counters. Counters are the acceptance.
4. Assert exact expressions, never `Depth` or `Length`.
5. Logical wrappers are not physical height; the association fixture is two
   physical levels per wrapper.
6. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then: the ten semantic locks green **unmodified** (paste the run showing all ten
by name); the release depth and width certificates; and the forced-front
falsifier including its delayed-RHS-once extension.

Then the cost gate, **counters first**:

- Allocation counts (`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base
  against candidate, both arms built at the same filesystem path, for: a shallow
  target-mode probe, a **wide** target-mode probe well inside the current
  ceiling (say 2,000 siblings, which passes today), and one untouched control.
  Calibrate sensitivity and state the same-binary spread.
- `perf stat -e instructions:u`, minimum of the tight clean cluster, migrated
  runs discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope, and **never** the semantic locks.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling** before
  your conversion is in place; afterwards the 64,000-sibling and 4,097-height
  certificates are the intended runs.
- **Stop-loss.** Stop and report if: a semantic lock cannot be satisfied; the
  handoff cannot preserve completed prefixes without restarting; allocation
  counts move on the wide probe or the control; a certificate does not complete
  with its exact value; or the same step fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The work representation: the cursor/finish enum, where the front decrements,
   and how the exhausted handoff carries live parents and completed prefixes.
3. What happens on `InputRequired`, and which frames exist afterwards.
4. The ten semantic locks, green and unmodified.
5. The certificates and the forced-front falsifier.
6. Registry and inventory rows, and the file sizes after the split.
7. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
8. Stop-loss and open questions.
