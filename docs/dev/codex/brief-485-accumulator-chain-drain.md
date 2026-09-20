# brief-485 — the accumulator-chain drain, proved on two members

Step 1 of the chain-destruction repair planned by brief-484, which is itself
step D's prerequisite. It builds the mechanism and adopts it for **two**
members — one per crate — and nothing else. The remaining 27 are later steps.

**Do not declare any suspension frame `BoundedDestruction` in this brief.**
Step D stays blocked until the family is covered.

## What is settled — do not re-derive or re-litigate

- **The hazard, measured** (director, brief-484): a faithful replica of the
  `Option<Arc<Self>>` shape at `ulimit -s 2048`, three runs per point, survives
  **64 000 nodes 3/3** and dies **66 000 3/3** with `fatal runtime error: stack
  overflow` — about 32 bytes per level. The tail grows one node per element on
  the string-replacement resume paths (brief-477), so the length is reachable.
  **Do not reproduce this.** No crash, overflow, or past-ceiling probe belongs
  in this brief; past-ceiling probing is the director's.
- **The family is 29 types**, 18 in `rum-runtime` and 11 in `rum-stdlib`, per
  brief-484's derivation from `self_reaching.tsv`. Two qualifications it found
  and this brief keeps: `PersistentRewriteQueueNode` links through `next`, not
  `previous`, and `PersistentMatchResultsNode`'s only constructor sets its link
  to `None` with no growth site.
- **`Drop` goes on a link wrapper, not on the node.** brief-484 found that
  **13 types move fields out of an unwrapped node across 14 sites** (e.g.
  `call_preparation.rs:78,95`, `eval_call_prepare.rs:72`,
  `kernel_data/association_data.rs:629`); a `Drop` on the node itself makes
  those moves illegal. This is the finding the design rests on.
- **The acquisition idiom** is `Arc::into_inner`, detaching each acquired
  node's link and **stopping at a shared node** — the surviving strong holder
  becomes responsible for the rest of that tail. Never inspect `strong_count`
  to decide ownership; that is an observation, not an acquisition. Prefer
  `into_inner` over `try_unwrap(..).ok()`.

## Director's ruling — the drain is O(1) and does not preserve payload order

brief-484 recommended preserving the recursive payload release order, at the
cost of collecting detached nodes in a cold `Vec` — O(k) temporary heap for an
exclusive suffix. **Overruled: release each detached node immediately.**

Two reasons, and the first is the load-bearing one:

1. **The order is unobservable.** The director enumerated every payload type in
   the family — `Expr`, `String`, `Symbol`, `SharedArgs`, `usize`,
   `Vec<usize>`, `DefinitionLhsParentContext`, `GuardedTargetUpdate`,
   `PersistentStructureCountUpdate`, `PreparedValueTableRule`, and the `Arc`
   collections — and **none of them appears among the tree's production `Drop`
   implementations** (`LookupOutcome`, `AssociationRuleMutationRollback`,
   `Session`, the four profiling guards, `SnapshotRestoreMode`). Payload
   destruction here is pure memory reclamation, so its order is not a
   semantic.
2. **Allocating inside a destructor is a new failure mode**, and doing it on
   the very path that exists to make destruction safe is backwards: an
   allocation failure aborts, and the 65 000-node case would ask for a
   65 000-element `Vec` at exactly the wrong moment.

It also collapses the link-first/payload-first distinction: **one algorithm**
serves both `previous`- and `next`-linked nodes, so the queue needs no special
case.

**The ruling's premise must be recorded where the next person will find it**:
state in the drain helper's rustdoc that immediate release is sound *because*
no payload in this family has an observable destructor, so that adding such a
payload is visibly a decision. If you find a payload type whose `Drop` is
observable, STOP and report — that changes my ruling, it does not get worked
around.

## Scope

Files you MAY modify:

- **new** `crates/rum-runtime/src/runtime/accumulator_chain.rs` — the wrapper,
  the node-access trait, the cold drain; plus an in-module test module.
- `crates/rum-runtime/src/runtime/mod.rs`, `.../kernel_api.rs` — wiring and
  re-export (`rum-stdlib` members need the wrapper across the crate edge).
- **new** `crates/rum-derive/src/accumulator_chain.rs` and the entry point in
  `crates/rum-derive/src/lib.rs` — the accessor derive.
- **new** `crates/rum-derive/tests/accumulator_chain.rs` — derive fixtures.
- The two proving-ground owners and their call sites:
  `crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs` (`IndexChunk`)
  and `crates/rum-runtime/src/runtime/scoping/lexical/with.rs`
  (`WithBindingNode`).
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and its
  `derived_inventory.rs`, if the wrapper changes a recorded traversal.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` — **position
  drift only**, plus any row whose *ownership path* legitimately changes
  because the link now sits behind the wrapper. Report every content change
  with its witness; a new `UnknownHostPayload` row, a removed row, or a
  reclassification is a **stop**, not a snapshot update.

MUST NOT touch: `rum-core` and `Expr`/`Symbol` representation or destruction
(`Expr`'s iterative drop was rejected on a measured 20-50% hot-path regression
and stays standing debt); `host_payload.rs`; the suspension traits;
`SECURITY.md`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**`,
`docs/dev/codex/**` except your own report; the other 27 members;
the ownership scanner's `.rs` code; `.git`, git config, `Cargo.lock`.

## The mechanism, as ruled

1. **A transparent wrapper** over the existing `Option<Arc<N>>` with a private
   field, carrying the single `Drop`. Its definition requires `N` to implement
   the node-access trait. Consuming extraction takes the inner option; nothing
   moves a field out of the wrapper's own type.
2. **A derive generates the accessor** from exactly one explicitly marked link
   field, and **rejects an unsupported arrangement with a compiler
   diagnostic** — including zero marked fields and more than one. Forgetting
   the derive where the wrapper is used must be a compile error. The accessor
   takes the marked wrapper's contents; it must never manufacture or clone a
   link.
3. **The drain is `#[cold] #[inline(never)]`** and holds at most one node at a
   time. No allocation, no counter, no payload clone.
4. **No automatic `BoundedDestruction`.** The marker is never derived, never
   blanket-implemented, and no frame gains a declaration in this brief.
5. **Weak handles do not make a node shared for this purpose** — acquisition is
   about strong ownership. Cover that in the tests rather than assuming it.

## The certificate — prove the drain is iterative, not that it survived

A test that drops a long chain and does not crash is **too weak**: it passes on
a recursive drop with a slightly larger stack. The certificate must
discriminate the *mechanism*.

Required, in the helper's own test module, over a synthetic node type built on
the wrapper (not a production type):

- A payload whose `Drop` records the **maximum simultaneously-active
  destruction nesting depth** through a test-local counter (increment on
  entry, decrement on exit, keep the max).
- **Iterative drain ⇒ the observed maximum is 1** at chain lengths 16 and 64;
  a recursive drop would read the chain length. Assert the exact value, not a
  bound.
- Exactly-once destruction of every node and payload.
- A **shared-tail** case: two prefixes over one tail, the first drop stopping
  at the shared node, the second later draining its own prefix and the
  now-exclusive tail, with the observed depth still 1 in both.
- A case with a live `Weak` handle present.

**Falsifier (mandatory):** replace the drain's body with an ordinary recursive
release, show the depth assertion reading the chain length and the test going
**red**, then restore by hand and show it green. Paste both. `git checkout`
restores from the index, which holds the pre-brief version, and would silently
revert your own work.

Note the premise honestly in the report: this certificate is about the
**helper**, and it cannot exist on the pre-brief tree because the helper does
not. Family coverage — that every member actually uses the wrapper — is a
later step's gate, not this one's; say plainly that 27 members remain
unconverted.

## Verification (MANDATORY — you run it; iterate to green)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

Both bounds come from this session's own full-workspace gate runs on this host,
which completed well inside them; they are upper bounds, not estimates.

**Microbench A/B is required**, because `WithBindingNode` is constructed on the
`With` binding path. Baseline before any production edit, candidate after:

```
CARGO_NET_OFFLINE=true bash scripts/core-language-microbench.sh --output-dir target/chain-a/base
CARGO_NET_OFFLINE=true bash scripts/core-language-microbench.sh --output-dir target/chain-a/new
```

Report the `with-*` labels, their same-tier neighbors, and — before drawing any
conclusion — **name the untouched control labels you are judging against**. If
the controls move as much as the target, that is the noise floor: report no
regression rather than tuning. Profiling env OFF. One baseline, one candidate;
do not silently re-run.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave everything in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No crash, overflow, past-ceiling, or fatal-process probe.** The number
  exists; reproducing it is not this brief's work.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`, and an
  iterative destructor is not a reason to introduce the first. If you are
  convinced one is unavoidable, STOP and report.
- **No default-path cost**: no new per-push allocation, counter, branch or
  payload clone on the construction path. Test-only observers must be wholly
  absent from production builds.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27). No stopgap, no
  compatibility shim.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — every command, verbatim, with the workspace total.
3. The certificate — the depth observations at both lengths, the shared-tail
   and weak cases, then the falsifier red and restored green.
4. The A/B — target labels, same-tier neighbors, and the named controls.
5. The inventory diff — position drift, and any ownership-path change with its
   witness.
6. Stop-loss / open questions, including anything you found about the other 27.
