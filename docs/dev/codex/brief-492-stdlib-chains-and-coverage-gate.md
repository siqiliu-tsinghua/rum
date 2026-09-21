# brief-492 — the last ten members, and the gate that keeps the family closed

The final step of the chain-destruction repair (plan brief-484). Nineteen of
twenty-nine members drain iteratively (briefs 485, 486, 489 — `d67e9f9b`).
This converts the **ten remaining `rum-stdlib` members** and installs the
**family coverage gate**, after which step D is unblocked.

**Do not declare any suspension frame `BoundedDestruction`.** That is step D
and it is a separate brief.

## The ten members

Derive the list yourself from
`crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv`: the
rows whose ownership path contains `Option[Own]/arg0/std::sync::Arc` **and does
not contain** `AccumulatorChain`. The director's count is **exactly 10**, all
in `rum-stdlib`; report any difference before converting anything.

Cross-check: `PersistentAssociationGroupNode`,
`PersistentAssociationSequenceNode`, `PersistentFunctionalSequenceNode`,
`PersistentLogicSequenceNode`, `PersistentPredicateMatchNode`,
`PersistentStringSequenceNode`, `PersistentStringTextNode`,
`PersistentStructureCountNode`, `PersistentStructureSequenceNode`,
`PersistentStructureSetNode`.

**`PersistentStringTextNode` is the one with a demonstrated reachable chain.**
brief-477 drove it past length 1 000 through the string-replacement resume
path, one node per character, with the generated-element and iteration-work
limits both set to 8. It is the reason this line exists. Convert it like the
rest; do not build a special case for it, and **do not reproduce that
construction** — no crash, overflow or past-ceiling probe belongs here.

`rum-stdlib` reaches the wrapper through `rum_runtime::kernel_api`, exactly as
`IndexChunk` already does (`structure_ops/resumable_merge_sort.rs`) — follow
that file as the in-crate pattern.

## The coverage gate — the point of this brief

Until now nothing forces a member to participate: the derive proves a
*converted* member's link is a wrapper, and brief-485's helper test proves the
drain is iterative, but a type could simply not be converted at all.

Install, in the ownership-graph test module, a check with this shape:

> Every self-reaching type whose ownership path goes
> `Option[Own]/arg0/std::sync::Arc` back to itself must carry
> `AccumulatorChain` on that path.

Requirements, and they matter more than the implementation:

- **It runs against discovery, not against the pinned `.tsv`** — over
  `graph::discover(source::production())`, the same source the exact-inventory
  test uses. A check that reads the snapshot could be satisfied by editing the
  snapshot.
- **No name list, no allowlist, no exemption table.** The family is whatever
  discovery finds. A future type of this shape must be converted or the gate is
  red, and anyone who genuinely needs an exception changes the gate in a
  visible diff.
- **Its failure message must name the type, its file and line, and its
  ownership path**, so the next person sees what to convert rather than a
  boolean.
- **It proves participation only.** Say so in a comment: the derive proves the
  link's type, the helper test proves the drain's mechanism, and this proves
  the family is complete. Three layers, one claim each.

**Falsifier (mandatory).** Revert one converted member to a raw
`Option<Arc<Self>>` link, show the gate red and its message naming that type,
then restore **by hand** and show it green. `git checkout` restores from the
index and would delete work — the director has made that mistake twice this
session; use a scratchpad copy instead.

Also state plainly whether the gate would have caught the pre-conversion tree —
it should have found all twenty-nine — and whether it currently finds zero.

## What to determine per member

For each of the ten, state whether its chain is built on an **ordinary
evaluation path** or only **after a suspension**, with the construction site's
`path:line`.

That decides the performance gate. brief-489 found three ordinary-path members
and they cost +0.567%, +1.044% and +0.774%; the iteration accumulator was
post-suspension and cost nothing measurable. Establish it from source per
member and do not assume either answer.

- Every member post-suspension: say so with evidence and run **no** A/B.
- Any member on an ordinary path: the instruction-count A/B is required for it.

```
taskset -c 2 perf stat -e instructions:u -x, -- <rum-cli> <workload>.rum < /dev/null
```

Read `cpu_core/instructions/u`, require `100.00` multiplex coverage, build both
arms at the same path, release profile, profiling env OFF, three runs per arm,
judge the delta against the observed within-arm spread (~0.019% on this host).
Run the CLI with stdin closed or the probes hang. Allocation counters must be
identical.

**Do not tune the wrapper.** Its shape is settled: three drain variants were
measured and the cost is intrinsic to carrying a `Drop`, not to the drain body.
If a member costs instructions, **report it** — the director accepts a measured
cost on this line's precedent and does not want it optimized away silently.

## The inventory rule — disclosure, not permission

You MAY regenerate any of the six `.tsv` files as discovery requires. List
**every** content change (position drift excepted) with its witness line,
classified as: (a) an attribute/derive row from the new derives; (b) an
ownership-path change from a link moving behind the wrapper; (c) anything else.
**Class (c) is a stop**, as is any new `UnknownHostPayload`, any removed row,
or any reclassification. Never hand-edit a row to make a check pass —
regenerate, then explain. All ten must **remain** self-reaching: this bounds
destruction, it does not remove the cycle.

## Scope

Files you MAY modify: the ten members' own modules and, only where a field type
change forces it, their immediate call sites — do not restructure any of them;
the ownership-graph test module for the new gate
(`crates/rum-stdlib/src/tests/ownership_graph/`, including `mod.rs` wiring and
a new file for the check); the inventory `.tsv` files under the rule above;
`crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and its
`derived_inventory.rs` if a recorded traversal changes.

MUST NOT touch: `crates/rum-runtime/src/runtime/accumulator_chain.rs` and
`crates/rum-derive/**` (finished — if you believe either needs a change, STOP
and report); the nineteen already-converted members; `host_payload.rs`; the
suspension traits; `rum-core`; `SECURITY.md`, `AGENTS.md`, `PROGRESS.md`,
`TODO.md`, other `docs/**`, `docs/dev/codex/**` except your own report; `.git`,
git config, `Cargo.lock`.

## Verification (MANDATORY)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

If your sandbox cannot bind loopback, say so and name the skip you used.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run any `git stash`
  or `git checkout` operation on a modified file.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No crash, overflow, past-ceiling, or fatal-process probe.**
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No `#[cfg(test)]` divergence in production code.**
- **No default-path cost**: no new allocation, counter, branch or clone on any
  construction path.
- Preserve suspension and resume behavior; do not collapse a
  `suspend_with_frame*` into an eager call.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27).
- If the same obstacle occurs twice, STOP and report rather than broadening
  scope.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. Your derived list of ten and how it compares to the cross-check.
2. What changed — per member, line ranges, one line why.
3. The gate: how it selects the family, its failure message, and the falsifier
   red then restored green, verbatim.
4. The per-member ordinary-path versus post-suspension determination with
   `path:line`, and what it decided about the A/B — plus any measured cost,
   reported rather than tuned.
5. Verification — every command verbatim, with the workspace total.
6. The inventory diff, classified.
7. Stop-loss / open questions.
