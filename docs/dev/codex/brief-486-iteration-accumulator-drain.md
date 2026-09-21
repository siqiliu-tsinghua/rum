# brief-486 — the iteration accumulator, the family's hottest member

Step 2 of the chain-destruction repair (plan brief-484; mechanism landed in
brief-485, `4c564adc`). It converts **one** member —
`PersistentExprAccumulatorNode` — and nothing else, because that member is on
the `Table`/`Do` collection path and deserves its own A/B and its own
stop-loss room rather than being buried in a batch.

**Do not declare any suspension frame `BoundedDestruction`.** Step D stays
blocked until all 29 members are converted.

**Redispatched 2026-09-21 after brief-488.** The first attempt found a
pre-existing kernel panic, which is now fixed (`8fe40eb0`): `into_vec` moves
out of a uniquely owned node and **clones out of a shared one**. Three things
changed for you:

- This brief's earlier instruction to preserve the uniqueness panic is
  **obsolete**. Preserve brief-488's move-or-clone behavior exactly instead.
- `stash@{0}` holds the first attempt. It is **stale** — it predates the
  repair. Do not apply, inspect-and-copy, or otherwise touch any stash. Redo
  the conversion on the current tree.
- The A/B is re-specified below with a calibrated instrument, because the
  timing panel could not resolve it.

## Why this one is alone, and why it matters most

- **It grows one node per collected element.** `PersistentExprAccumulator::appended`
  (`kernel_data/control_data/branching_loops/iteration/mod.rs:79-89`) allocates
  a node per element on top of the frozen `prefix`.
- **Completion drains iteratively already** — `into_vec` (`:91` onward) walks
  with `Arc::try_unwrap` in a loop. **Abandonment does not.** Dropping the
  accumulator without `into_vec` — an error, a cancellation, an abandoned
  suspension — destroys the chain recursively.
- So an ordinary `Table[i, {i, 1, 100000}]` that is cancelled or fails drops a
  100 000-node chain, against a measured ceiling near **64 000** at the ruled
  2 MiB stack (brief-484). This is the most reachable member of the 29.
- **`PersistentExprAccumulator` derives `Clone`**, and a clone shares the tail.
  The drain's "stop at a shared node" path is therefore load-bearing here, not
  theoretical.

Take that as motivation, not as a task: **do not reproduce the crash.** The
ceiling is measured, past-ceiling probing is the director's, and no crash,
overflow, or process-termination probe belongs in this brief.

## Goal

`PersistentExprAccumulatorNode`'s link travels behind `AccumulatorChain`, so
its destruction is the cold iterative drain. Iteration behavior, suspension and
resume, `Clone` sharing, and `into_vec`'s uniqueness expectation are unchanged,
and the collection path pays nothing.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs`
  — the node, the holder's `tail` field, `appended`, `into_vec`, and the
  in-file `FrameRootTrace` assertions.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs`
  and `.../iteration_aggregate.rs` — only if the field-type change forces a
  call-site edit. Do not restructure either.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and its
  `derived_inventory.rs`, if a recorded traversal changes.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` — under the
  inventory rule below.

MUST NOT touch: `crates/rum-runtime/src/runtime/accumulator_chain.rs` and the
`rum-derive` accessor derive (both finished in brief-485 — if you believe
either needs a change, STOP and report); `host_payload.rs`; the suspension
traits; the other 26 unconverted members; the ownership scanner's `.rs` code;
`rum-core`; `SECURITY.md`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, other
`docs/**`, `docs/dev/codex/**` except your own report; `.git`, git config,
`Cargo.lock`.

## The mechanism — already ruled, do not redesign

Follow brief-485 exactly: the `#[repr(transparent)]` `AccumulatorChain<N>`
wrapper carries the single `Drop`; the derive supplies the node accessor, so
omitting it is a compile error; the drain is `#[cold]`, holds one node at a
time, acquires with `Arc::into_inner`, and stops at a shared node. Release is
immediate and does **not** preserve payload order — that ruling rests on no
payload in this family having an observable destructor, and `Expr` does not.

Two things specific to this member:

1. **`into_vec` must keep brief-488's behavior exactly**: move out of a
   uniquely owned node, clone out of a shared one, never panic. It consumes
   the chain itself, so it must take the wrapper's contents rather than
   fighting the `Drop`.

   One interaction to get right rather than discover: the shared branch clones
   a node, and that clone's link is a wrapper whose `Drop` runs at the end of
   the iteration. That is correct — the clone's link is an `Arc` clone, so
   `Arc::into_inner` fails at once and the drain stops immediately — but say
   in your report that you checked it, because a naive reading suggests the
   tail is drained twice.

3. **No `#[cfg(test)]` divergence in production code.** The first attempt gave
   `into_vec` one body for test builds and another for production. That makes
   the certificate prove something about code production never runs, which is
   a trap this project has been caught by before. Production and test builds
   must execute the same `into_vec`. If an observer cannot be added without
   forking the path, put the observer in the test's own types, or STOP and
   report.
2. **`appended` is the hot path.** The wrapper is `#[repr(transparent)]` and
   its constructor is `#[inline]`, so the collection path should pay exactly
   nothing. That is a claim the A/B has to support, not an assumption.

## The inventory rule — disclosure, not permission

You MAY regenerate any of the six `.tsv` files as discovery requires. List
**every** content change (position drift excepted) with its witness line,
classified as: (a) an attribute/derive row from the new derive; (b) an
ownership-path change from the link moving behind the wrapper; (c) anything
else. **Class (c) is a stop**, as is any new `UnknownHostPayload`, any removed
row, or any reclassification. Never hand-edit a row to make a check pass —
regenerate, then explain.

brief-485's one authorized `UnknownHostPayload` (the generic wrapper
constructor) already exists; this conversion must add **no** further one, since
it reuses that same generic wrapper. If one appears, that is a finding worth
stopping for.

`PersistentExprAccumulatorNode` must remain in `self_reaching.tsv`: this bounds
destruction, it does not remove the ownership cycle.

## The certificate

Reuse brief-485's discriminator rather than inventing one. In the accumulator's
own test module:

- A chain built through `appended`, then **dropped without `into_vec`** — the
  abandonment shape that is the actual hazard — asserting the wrapper's drain
  ran. Where the brief-485 depth observer cannot be reused directly (the
  payload is `Expr`, whose `Drop` you must not touch), assert instead that the
  drop path is the drain: a **shared-tail** case where one clone is dropped
  first and the surviving clone still yields the correct `into_vec` result, and
  a case where the accumulator is dropped mid-iteration.
- `into_vec` still returns exactly the collected elements in order, on a chain
  long enough to have a real tail.

**Falsifier (mandatory):** revert the `tail` field to a raw
`Option<Arc<PersistentExprAccumulatorNode>>` and show which assertions go red,
then restore by hand and show them green. If **nothing** goes red, say so
plainly — that means the certificate does not discriminate and needs
redesigning before this brief can be credited. Restore by hand: `git checkout`
restores from the index, which holds the pre-brief version.

## Verification (MANDATORY)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

**The A/B is the real gate here**, because `appended` runs per collected
element — and the first attempt showed the timing panel cannot resolve it: the
named controls themselves moved +3.70% to +12.45%, and one target read +19.13%
in one pair and -6.34% in the other. Do not try to win that argument with more
timing runs.

**Use instruction counts. The director calibrated the instrument for you:**

```
taskset -c 2 perf stat -e instructions:u -x, -- <command>
```

Read the `cpu_core/instructions/u` row and require its multiplex column to read
`100.00`; pinning to CPU 2 keeps the run on one core type, which this host
needs because its P and E cores count separately. Repeatability measured by the
director on a trivial command: 167 565 / 167 562 / 167 567 instructions, a
spread of 0.003%.

- Build base and candidate **at the same path**, release profile, profiling env
  OFF.
- Drive a fixed `Table`/`Do` workload from a `.rum` script through the release
  `rum-cli`; state the script and keep it fixed across arms.
- **Three runs per arm.** Report every count, each arm's spread, and the delta.
- **The acceptance criterion**: the candidate's delta must not exceed the
  observed within-arm spread. If it does, that is a real cost — the plausible
  mechanism is a `#[cold]` drain call replacing an inlined refcount decrement
  on a per-iteration drop — and it must be fixed or reported, not averaged
  away.
- Allocation counts must also be identical, as they were in the first attempt.
  If counters and instructions both say no change, no timing number overrules
  them; say so and move on rather than running the panel again.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave everything in
  the working tree.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No crash, overflow, past-ceiling, or fatal-process probe.**
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No default-path cost**: no new allocation, counter, branch, or clone on
  `appended`, and test-only observers absent from production builds.
- Preserve suspension and resume behavior; do not collapse a
  `suspend_with_frame*` into an eager call.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27). No stopgap.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output; if a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — every command verbatim, with the workspace total.
3. The certificate — the assertions, then the falsifier red and restored
   green, or an explicit statement that nothing went red.
4. The A/B — allocation counters first, then the two pairs, the named controls
   and the control band.
5. The inventory diff, classified.
6. Stop-loss / open questions, including anything you noticed about the other
   26.
