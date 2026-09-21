# brief-489 — the remaining sixteen runtime chain members

Step 3 of the chain-destruction repair (plan brief-484). Mechanism in
brief-485 (`4c564adc`); two members converted in brief-485 and brief-486
(`377a0747`). This converts the **sixteen remaining `rum-runtime` members**.
The ten `rum-stdlib` members and the family coverage gate are brief-490.

**Do not declare any suspension frame `BoundedDestruction`.** Step D stays
blocked until all 29 are converted and the coverage gate is in.

## The three enforcement layers — build nothing beyond your layer

This is the main instruction of this brief. The proofs already exist and are
layered; **do not add sixteen copies of a certificate**:

1. **The derive proves the wiring, at compile time.** `AccumulatorChainNode`
   rejects a marked link whose type is not `AccumulatorChain<…>` with a
   compiler error (`rum-derive/src/accumulator_chain.rs:44-57`), and requires
   exactly one `#[accumulator_link]` field. A converted member therefore cannot
   have a raw link.
2. **The helper test proves the mechanism** — brief-485's depth observer,
   asserting destruction nesting of exactly 1. It is already green and covers
   every member through the shared drain.
3. **The coverage gate will prove the family is complete** — that every
   discovered member has the derive at all. That is brief-490's job, **not
   yours**. Do not write a partial or list-based version of it here.

What is left for you per member is therefore small: the conversion itself, and
a behavioural test **only** where a member has a reachable abandonment path
worth pinning. Say per member whether you added one and why, rather than
padding all sixteen.

## The members

Derive the list yourself from
`crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv` —
the rows whose ownership path contains `Option[Own]/arg0/std::sync::Arc` —
minus the three already converted (`WithBindingNode`, `IndexChunk`,
`PersistentExprAccumulatorNode`). **Do not work from a name pattern**; several
members do not start with `Persistent`.

The director's cross-check, which you should confirm rather than trust — if
your derivation differs, report the difference before converting anything:
`PreparedArgumentNode`, `ReapRenderNode`, `TestReportResultNode`,
`ResolvedArgumentNode`, `DefinitionLhsParentNode`, `PreparedValueTableRuleNode`,
`UnevaluatedRestoreNode`, `AssociationSelectorNode`, `PersistentRewriteExprNode`,
`PersistentRewriteQueueNode`, `GuardedTargetChunk`, `GuardedTargetUpdateNode`,
`PersistentReplacePartTargetNode`, `PersistentMatchResultsNode`,
`FixedPointHistoryNode`, `PersistentPositionNode`.

Two known specifics, both settled:

- **`PersistentRewriteQueueNode` links through `next`, not `previous`.**
  `#[accumulator_link]` marks the field whatever it is called; this is not a
  defect and needs no special case. The immediate-release ruling (brief-485)
  already removed the link-first/payload-first distinction.
- **`PersistentMatchResultsNode` is converted too**, although its only
  constructor sets the link to `None` and it has no growth site today.
  "Bounded by how it happens to be constructed" is not an enforced bound, and
  a future growth site would otherwise reintroduce the defect silently.

## What to determine per member, and what it decides

For each of the sixteen, state in your report **whether its chain is built on
an ordinary evaluation path or only after a suspension**, with the
construction site's `path:line`.

This is not bookkeeping: it decides whether a performance gate is needed. The
director's premise for brief-486 was wrong in exactly this way — `appended`
looked like the `Table` hot path and is in fact reached only from the
`Err(EvalError::InputRequired)` arm (`iteration/driver.rs:294-300`), so the
ordinary path never builds a node. Establish it from source per member; do not
assume either answer.

- If **every** member is post-suspension only, say so with the evidence and run
  **no** A/B; a host round trip dominates anything here.
- If **any** member is built on an ordinary evaluation path, the
  instruction-count A/B is required for that member, with the calibrated
  recipe below.

## Expect the destructuring to keep compiling — and report it if it does not

brief-484 found **13 types that move fields out of an unwrapped node across 14
sites** (`call_preparation.rs:78,95`, `eval_call_prepare.rs:72`,
`kernel_data/association_data.rs:629`, and others). The design puts `Drop` on
the wrapper precisely so the node keeps no `Drop` and those moves stay legal.

If a move stops compiling, that is a **finding to report**, not something to
work around — and above all **do not add a `Drop` to a node** to make something
fit. Stop and report instead.

## The A/B, if it is needed at all

The timing panel cannot resolve questions of this size — its own controls have
moved +3.70% to +12.45% on this host. Use instruction counts:

```
taskset -c 2 perf stat -e instructions:u -x, -- <command>
```

Read `cpu_core/instructions/u` and require its multiplex column to read
`100.00`. Director-calibrated repeatability: 167 565 / 167 562 / 167 567 on a
trivial command, a spread of 0.003%. Build both arms at the same path, release
profile, profiling env OFF, three runs per arm, and judge the delta against the
observed within-arm spread. Allocation counters must be identical. If counters
and instructions both say no change, no timing number overrules them.

**A probe that cannot reach the changed path proves nothing** — state what your
workload actually exercises, as brief-486's worker correctly did.

## The inventory rule — disclosure, not permission

You MAY regenerate any of the six `.tsv` files as discovery requires. List
**every** content change (position drift excepted) with its witness line,
classified as: (a) an attribute/derive row from the new derives; (b) an
ownership-path change from a link moving behind the wrapper; (c) anything else.
**Class (c) is a stop**, as is any new `UnknownHostPayload`, any removed row,
or any reclassification. Never hand-edit a row to make a check pass —
regenerate, then explain.

No new `UnknownHostPayload` should appear: every member reuses the one generic
wrapper whose constructor row is already pinned. If one appears, stop.

All sixteen must **remain** in `self_reaching.tsv`: this bounds destruction, it
does not remove the ownership cycles.

## Scope

Files you MAY modify: the sixteen members' own modules and, only where a field
type change forces it, their immediate call sites — do not restructure any of
them; `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and its
`derived_inventory.rs` if a recorded traversal changes; the inventory `.tsv`
files under the rule above.

MUST NOT touch: `crates/rum-runtime/src/runtime/accumulator_chain.rs` and
`crates/rum-derive/**` (finished — if you believe either needs a change, STOP
and report); `host_payload.rs`; the suspension traits; the ten `rum-stdlib`
members; the ownership scanner's `.rs` code; `rum-core`; `SECURITY.md`,
`AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**`, `docs/dev/codex/**`
except your own report; `.git`, git config, `Cargo.lock`.

## Verification (MANDATORY)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

If your sandbox cannot bind loopback, say so explicitly and name the skip you
used — the director re-runs the gate without it.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run any `git stash`
  operation.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No crash, overflow, past-ceiling, or fatal-process probe.**
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No `#[cfg(test)]` divergence in production code** — production and test
  builds must execute the same bodies.
- **No default-path cost**: no new allocation, counter, branch or clone on any
  construction path.
- Preserve suspension and resume behavior; do not collapse a
  `suspend_with_frame*` into an eager call.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27). Sixteen members
  describe the work; they do not argue for a shortcut.
- If the same obstacle occurs twice, STOP and report rather than broadening
  scope.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. Your derived member list and how it compares to the director's cross-check.
2. What changed — per member, line ranges, one line why.
3. The per-member ordinary-path versus post-suspension determination, with
   `path:line`, and what it decided about the A/B.
4. Verification — every command verbatim, with the workspace total.
5. Which behavioural tests you added, per member, and why those and not others.
6. The inventory diff, classified.
7. Stop-loss / open questions, including anything about the ten stdlib members.
