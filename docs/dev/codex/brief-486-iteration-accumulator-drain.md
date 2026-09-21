# brief-486 — the iteration accumulator, the family's hottest member

Step 2 of the chain-destruction repair (plan brief-484; mechanism landed in
brief-485, `4c564adc`). It converts **one** member —
`PersistentExprAccumulatorNode` — and nothing else, because that member is on
the `Table`/`Do` collection path and deserves its own A/B and its own
stop-loss room rather than being buried in a batch.

**Do not declare any suspension frame `BoundedDestruction`.** Step D stays
blocked until all 29 members are converted.

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

1. **`into_vec` must keep working unchanged in behavior**, including its
   `Arc::try_unwrap` uniqueness expectation and its existing panic message. It
   consumes the chain itself, so it must take the wrapper's contents rather
   than fighting the `Drop`. If the wrapper makes that panic unreachable or
   changes when it fires, STOP and report — that is a semantic change, not a
   refactor.
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
element. One pair cannot gate this panel — the previous step saw untouched
`table-*` labels move +13.48% and +10.72% on pure layout lottery — so:

- **Allocation counts decide first.** They are load-immune. Report the
  allocation counters for the `table-*` and `do-*` labels; the wrapper adds no
  allocation, so **any change in allocation count is a defect**, and if the
  counters do not move, do not let a timing number overrule them.
- Then **two base/candidate pairs**, alternating order, profiling env OFF,
  machine otherwise idle. Build both arms at the same path.
- Name the untouched control labels **before** judging, and report the control
  band alongside the targets. If the targets sit inside the band, that is no
  regression; say so rather than tuning.
- Report `table-*`, `do-*`, and their same-tier neighbors.

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
