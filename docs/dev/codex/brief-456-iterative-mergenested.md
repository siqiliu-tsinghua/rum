# brief-456 — iterative `MergeNested`

## Goal

`MergeNested` recurses natively once per association nesting level and now ends
the process at 2,027 wrappers — raw physical height 4,055, just under the ruled
4,097. Remove that recursion so an exact merge completes at raw height 4,097 and
fails in band above it, preserving grouping order, the combiner call schedule,
diagnostics and suspend/resume behaviour exactly.

This is draft B of brief-454, re-derived against measurements taken after
brief-455 landed. **Its numbers and its failure mode both changed** — read the
next section before anything else.

## What I measured — treat as given, do not re-run

Release `rum-cli` at `c19052d6`, `ulimit -s 2048`, 1 MiB evaluator budget.
Heights are **logical association wrappers**; one wrapper is two physical
levels, so the ruled 4,097 physical height is `N=2048`. Do not use `Depth` as
the height unit — it hides the `Rule` layer.

| probe | before brief-455 | now |
|---|---|---|
| `MergeNested[{a, b}, Total]` | 1,153 / 1,161, **in band** | **2,027 / 2,035, process kill** |
| `Map[ff, a, {0, Infinity}]` | 966 / 974, in band | clears to the construction ceiling |
| nested key-path assignment then read | 1,145 / 1,153, in band | clears to the construction ceiling |
| construction alone (the fixture ceiling) | 5,457 / 5,463 | unchanged |

**The failure mode flipped, and that is not a defect brief-455 introduced.**
Before, the association arm of the stability walk consumed the byte budget
first, and because that walk polls, the failure surfaced as an in-band
`General::reslim`. brief-455 made that arm cheap, so `MergeNested`'s own
recursion — which has **no poll point anywhere in
`crates/rum-stdlib/src/associations/`** — now reaches the raw stack limit first.
Verified: at 2,035 the process dies with `thread 'main' has overflowed its
stack`.

So this site moved from "fail-closed and 3.6x short" to "**fail-open and 1.01x
short**", which is why it is dispatched next. Removing a guard-adjacent cost can
convert a fail-closed site into a fail-open one; that is worth knowing generally,
and it is why this line re-probes before briefing.

## The recursion to remove

`crates/rum-stdlib/src/associations/resumable/aggregate.rs`:

- `resume_merge_nested_from_index` (`:122`) calls `evaluate_merge_nested_group`
  at `:134` and `:168`;
- `evaluate_merge_nested_group` (`:199`) calls back into
  `resume_merge_nested_from_index` at `:205` for a nested group.

One mutual recursion, one native level per logical wrapper, measured at about
1,008 B per level: the parent holds its owned combiner, the grouped `Vec`, the
partial merged `Vec`, the index, the key and values, and the
`Result`/suspension match state live across the recursive call.

## Semantics that must not move

- **First-seen key order** in the merged output.
- **Only an all-association value group recurses** (`:199-222`); a mixed group
  goes to the combiner as it stands.
- **The combiner runs only where it runs today** — same call sites, same
  number of calls, same order. A driver that flushes eagerly or defers
  differently changes an observable schedule.
- **Suspension freezes the whole explicit stack.** An `InputRequired` inside a
  combiner must resume at the exact pending combiner with no replay and no
  duplicated side effect. This is the property the resume-frame module exists
  for; extend it, do not bypass it.
- Grouped trees are `Arc`-shared: no deep clones on the task path.

## Deliverable

1. The nesting walk driven by explicit `EnterGroup`/`FinishGroup` work items
   carrying the combiner, grouped entries, index, key and partial output, behind
   a bounded borrowed front if the front is instruction- and
   allocation-neutral. The pattern is the one `Replace`, `D`, `DeleteCases`,
   `Take`/`ReplacePart`, the substitution engines and now the association
   stability arm all use.
2. No new suspension protocol: the existing frames carry the explicit stack.
3. Certificates, following the rule this line settled in briefs 439-442:
   - a release-only `#[ignore]` deep certificate on a 2 MiB declared thread
     merging two `N=2048` chains with a **unique deepest numeric pair** and
     non-matching shallow sentinels, asserting **with an iterative helper** that
     each input is raw physical height 4,097, and asserting the **exact complete
     merged association** ending in the summed deepest value;
   - a three-level shallow companion pinning the nested-association branch
     predicate, the first-seen key order, and that the combiner is called only
     where it is called today;
   - a suspended-combiner companion proving no combiner call is repeated across
     a suspend/resume.
4. One microbench label for a small nested merge, with a sink liveness
   assertion following the convention at
   `scripts/core-language-microbench.sh:546,552,558`.

## The falsifier — required section

Before changing any source, add the shallow companion and the suspended-combiner
companion, run both against unmodified HEAD, paste the passes, then break each
deliberately — a wrong key order for the first, an expectation of a repeated
call for the second — and paste the failures. Restore both before continuing.

Do **not** run the deep certificate against unmodified HEAD: at `N=2048` the
pre-change code terminates the test binary.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/associations/resumable/aggregate.rs`
- `crates/rum-stdlib/src/associations/resumable/aggregate/resume_frames.rs` and
  new files beside them, declared from their existing module roots; you MAY edit
  those roots for the `mod` line only
- focused `MergeNested` tests
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh` — additive, one label

Out of scope: ordinary `Merge`, `grouping.rs` semantics, every runtime crate,
the association stability arm brief-455 just repaired, docs and `Cargo.lock`.

## Explicitly out of scope

Do not remove repeated work or change how many times a group is parsed. This
brief bounds depth and nothing else. **A local poll is not acceptance**: it
would restore the in-band failure this site used to have without reaching the
contract.

## Measurement — instruction counts with allocation counts, not wall time

**A timing percentage is not evidence on this host and will not be accepted.**
P-cores and E-cores are 1.27x apart with no pinning.

1. **Allocation counts**: `--features alloc-count` build, `RUM_PROFILE_ALLOC=1`,
   isolated probe per label, base and candidate executable paths the **same
   length**. Calibrate: one extra loop iteration must move the count.
2. **Instruction counts**: `perf stat -e instructions:u -x,`, minimum of the
   tight clean cluster; discard and name runs that migrate between `cpu_atom`
   and `cpu_core` (they read about 55% high).

**Report both together, and do not attribute a small instruction delta on its
own.** At the brief-455 gate a change that could not touch the generic paths
moved their instruction counts by -7% to -8.7% while their allocation counts
were identical to the byte — codegen in the same module moved, not the work. So:
an allocation increase is a real result; an instruction increase with unchanged
allocations needs a second look before you call it a regression, and an
instruction *decrease* is not a speedup you may claim.

The bar: allocation counts unchanged on a small nested merge and on
`association-small` shapes; instruction movement reported for both with its
cluster spread.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

`cargo test --workspace` may fail nine `rum-jupyter` native ZeroMQ tests if your
sandbox cannot bind loopback — an environment limit, not a result. Say so and
also run `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace`, pasting both.

Then, after the shallow suite is green:

```
cargo test -p rum-stdlib --release -- --ignored
```

That panel takes about 450 s because of the `D` certificate; size the timeout
from a measured run.

Then the allocation and instruction comparisons above.

## Stop-loss

Stop and report if:

- a key-order, branch-predicate or combiner-schedule difference appears twice;
- suspension parity fails twice for the same reason;
- allocation counts rise on a shallow merge and two isolated revisions do not
  remove it;
- the certificate can only pass by lowering the target height or by using
  `Depth` as the unit.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the tree.
- Do NOT modify files outside Scope; nothing outside this repository; no network.
- Testing Policy is a hard line (`AGENTS.md`).
- No new `unsafe`; no default-path cost; no profiling state in a frame or task.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything long.
- Paste real verbatim output.

## Report format

1. What changed — per file and line range, naming both removed recursive edges
   and how the suspension freezes the explicit stack.
2. Verification output verbatim, including both falsifier passes and their
   deliberate failures, and both workspace runs.
3. Safety and contract as separate outcomes: no process termination, and the raw
   physical height reached, measured with an iterative helper.
4. Allocation and instruction numbers with calibration and discarded runs.
5. Stop-loss / open questions.
