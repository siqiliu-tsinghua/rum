# brief-490 — the wrapper's drop pays a cold call on every link

A measured regression, its cause identified, and one file to change.
brief-489's sixteen conversions are **in the working tree, uncommitted, and
correct** — the workspace suite is green with them. They are not the defect and
you must not modify them.

## The finding

brief-489's instruction-count gate failed on three ordinary evaluation paths:

| workload | delta | arm spreads |
|---|---:|---|
| `prepared-arguments` | **+0.545%** | ~0.009% |
| `lhs-parents` | **+1.036%** | ~0.011% |
| `value-table-rules` | **+0.741%** | ~0.011% |

Fifty to a hundred times the instrument's own spread, with allocation counters
identical. The worker stopped rather than touch the helper, which was right.

## The cause, already established — do not re-derive it

`crates/rum-runtime/src/runtime/accumulator_chain.rs:74-78`:

```rust
impl<N: AccumulatorChainNode> Drop for AccumulatorChain<N> {
    fn drop(&mut self) {
        drain(self.0.take());
    }
}
```

`drain` is `#[cold] #[inline(never)]`. So **every** drop of a link pays an
out-of-line call — including the empty link and the single-node chain, which is
what almost every drop on these paths actually is. The cold attribute is right
for walking a chain and wrong as the entry condition.

This is the same lesson the traversal-depth line paid for twice: **put the
check where the growth is**, and keep the cold path cold by *branching* into it
rather than *calling* into it unconditionally.

## Goal

The empty link and the single-node chain must be handled **inline, with no
out-of-line call**; only a chain with something left to walk reaches the cold
drain. All three workloads come back within their arm spreads, brief-485's
certificate stays exactly as green as it is now, and nothing else changes.

## The shape, as ruled

Something of this form — you may improve on it, but not weaken what it
guarantees:

```rust
impl<N: AccumulatorChainNode> Drop for AccumulatorChain<N> {
    #[inline]
    fn drop(&mut self) {
        let Some(link) = self.0.take() else { return };
        let Some(mut node) = Arc::into_inner(link) else { return };
        let next = node.take_accumulator_link();
        drop(node);
        if next.is_some() {
            drain(next);
        }
    }
}
```

The invariants this must preserve, all of them already certified:

- **Shared links still stop immediately** and leave the surviving strong holder
  responsible for the tail. Never inspect `strong_count` to decide ownership.
- **Acquisition is still `Arc::into_inner`**, not `try_unwrap(..).ok()`.
- **Release stays immediate** and does not preserve payload order.
- **One node at a time, O(1) auxiliary space.** The fast path must not
  accumulate.
- The rustdoc's recorded premise — immediate release is sound because no
  payload in this family has an observable destructor — stays, and stays
  accurate.

## The gate

1. **brief-485's certificate must stay green unchanged**: destruction nesting
   exactly 1 at chain lengths 16 and 64, the shared-tail case, the live-`Weak`
   case. Do not edit those tests. If the fast path makes any of them pass for a
   different reason, say so — a chain of 16 still reaches the drain, so they
   should be untouched.
2. **The falsifier must still discriminate**: replace `drain`'s body with an
   ordinary recursive release and show the depth assertions reading the chain
   length, then restore **by hand** (`git checkout` restores from the index,
   which holds the pre-brief version) and show green.
3. **Instruction counts**, the same calibrated recipe and the same three
   workloads brief-489 used, so the numbers are comparable:

   ```
   taskset -c 2 perf stat -e instructions:u -x, -- <command>
   ```

   Read `cpu_core/instructions/u`, require `100.00` multiplex coverage, build
   both arms at the same path, release profile, profiling env OFF, three runs
   per arm. **Base is `HEAD` without brief-489's conversions**; candidate is
   the tree with the conversions **and** this fix. That is the comparison that
   matters: it asks whether the conversion is now free, not whether this patch
   improved on the failing candidate. Report both arms' spreads and the delta
   per workload.
4. `cargo fmt --check` and the full workspace suite.

**Acceptance**: all three deltas within the arm spreads. If one remains out,
report it with its number — do not keep tuning. **Stop-loss: two attempts.** If
a second shape also fails, stop and report both, with what you learned about
where the instructions go; the director decides whether ~1% is the price of
bounded destruction or whether the design changes.

## Scope

Files you MAY modify: **`crates/rum-runtime/src/runtime/accumulator_chain.rs`
only** — and within it, the `Drop` implementation and, if needed, `drain`'s
signature. Its tests may be read but not edited.

MUST NOT touch: brief-489's sixteen converted members and their call sites
(they are in the tree, uncommitted, and correct); `crates/rum-derive/**`;
`host_payload.rs`; the suspension traits; the ownership scanner and its
inventories — **this change alters no type, so no inventory row should move;
if one does, STOP and report**; `rum-core`; `SECURITY.md`, `AGENTS.md`,
`PROGRESS.md`, `TODO.md`, other `docs/**`, `docs/dev/codex/**` except your own
report; `.git`, git config, `Cargo.lock`.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run any `git stash`
  operation. The tree carries brief-489's uncommitted work — **do not revert,
  stash, or "clean" it**.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No crash, overflow, past-ceiling, or fatal-process probe.**
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`, and a
  faster destructor is not a reason to introduce the first.
- **No `#[cfg(test)]` divergence in production code.**
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** for weakening a guarantee: do not recover
  instructions by dropping the shared-node stop, the `into_inner` acquisition,
  or the O(1) space bound.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.
- If your sandbox cannot bind loopback, say so and name the skip you used.

## Report format

1. What changed in the one file, with line ranges and why.
2. The three workloads' base/candidate counts, spreads and deltas, against
   brief-489's failing numbers for comparison.
3. brief-485's certificate green, then the falsifier red, then restored green,
   verbatim.
4. `cargo fmt --check` and the workspace total.
5. Confirmation that no inventory row moved.
6. Stop-loss / open questions.
