# brief-472 — convert the shared `KeyTake`/`KeyDrop` collection walk

## Goal

Make `KeyTake` and `KeyDrop` over nested list collections complete at the ruled
physical height 4,097 and stop growing the native stack, then flip both registry
rows to `BoundedHeap` **and `MeetsContract`**.

**This one is short of the contract**, unlike every conversion since brief-462.
Its certificate is not "still completes at 4,097" — it is "completes at 4,097,
which it cannot do today".

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3, each probe carrying a liveness assertion on the exact value.

| operation | ok <= | fail >= | vs 4,097 | direction above |
|---|---|---|---|---|
| `KeyTake` over nested lists | 2,856 | 2,864 | **1.43x short** | **terminates the process** |
| `KeyDrop` over nested lists | 2,856 | 2,872 | **1.43x short** | **terminates the process** |

They bracket identically because they share one walk — which brief-471 confirmed
and which is the reason they are one brief.

A ceiling is not deterministic; the same height flips run to run inside a band
tens of levels wide.

## The site

`keyed_take_drop_result_for_expr` (`crates/rum-stdlib/src/associations/basic.rs:818`),
self-recursive at `:899`, serving both operations through a `keep_selected`
flag. One function, one edge.

brief-471 established that **no function in this file's key family takes a
`Session`** — director-verified, zero occurrences of a session parameter across
`basic.rs:690-920`. So this walker cannot run user code, and the **simple
bounded-front-plus-redo shape applies**: a depth-16 recursive front with a cold
iterative tail that may redo the traversal. Do not build a carried-continuation
handoff; only `DeleteMissing`'s target mode needed one, because it evaluates
user code mid-walk.

## What must not change

`KeyTake` and `KeyDrop` over a nested list return **lists wrapping an
`Association`** — director-verified: `KeyDrop[{{{{"k" -> 7}}}}, {"z"}]` is
`{{{<|"k" -> 7|>}}}`. That conversion of the innermost rule list into an
association is part of the contract, not an incidental detail, and a rewrite
that preserves the values while changing the wrapper shape is a **stop**.

Also preserve: sibling order, every nested `List` wrapper, key order inside the
resulting association, rule heads, and the `keep_selected` distinction between
the two operations — a conversion that made `KeyDrop` behave as `KeyTake` on
some branch would still pass a values-only assertion.

## Deliverable — certificates and the falsifier

**Certificates**, release-only, on declared 2 MiB threads, three-part rule:

- `KeyTake` at raw physical height 4,097 — **which it cannot reach today** —
  with a unique deepest target, asserting the **exact** result including its
  wrapper shape, not just the surviving value.
- The same for `KeyDrop`, asserting the complementary selection, so a walk that
  confused the two flags fails.
- A three-level **shallow companion** pinning wrapper shape, sibling order and
  key order, which is also the falsifier if a deep case aborts the binary.

**Forced-front falsifier**, in-module, no cargo feature and no production
switch — the shape briefs 463, 468, 469 and 470 settled on. A private
`*_with_front_depth`, called from `mod tests`, on a **branched** fixture with
more than one entry per level and a distinct witness per branch:

- front `0` returns `Exhausted`;
- front `16` returns `Complete`;
- **tail == front == exact expected**.

Run it for both `keep_selected` settings; a single-flag fixture leaves half the
shared walk uncovered.

## Registry and inventory

Both rows change **two** fields, which is new for this line: `Mechanism` from
`UnguardedNative` to `BoundedHeap`, **and `ContractStatus` from
`ShortOfContract` to `MeetsContract`**, with
`failure_direction: Some(FailureDirection::Unmeasured)` and `owed_by: None`.
Every previous conversion left the contract status alone because those sites
already met it. Update inventory rows for the converted edge.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/basic.rs`, and a new sibling module
    under `crates/rum-stdlib/src/associations/basic/` if the file needs
    splitting by responsibility — it is large, so say what you measured before
    deciding;
  - `crates/rum-stdlib/src/associations/mod.rs` if exports change;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - a new focused certificate file plus its wiring in
    `crates/rum-stdlib/src/tests.rs`.
- Files you MUST NOT touch: the `Keys`, `Values` and `Lookup` walks in the same
  file — they are separate walks with their own briefs, and brief-471 corrected
  the inventory citation that made `Keys` and `Values` look shared. Also off
  limits: the guard substrate, `kernel_api.rs`, every already-converted site,
  `metadata.rs`, `Expr`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other
  `docs/**`, `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended change is that this walk stops growing the
  native stack and therefore reaches 4,097. Any change to wrapper shape, sibling
  or key order, rule heads, the `keep_selected` distinction, or any other
  builtin is a **stop**.

## rum / Rust pitfalls (read twice)

1. **Do not touch `Keys` or `Values` while you are in this file.** They are
   `keys_result_for_expr` (`:752`) and `values_result_for_expr` (`:781`), two
   separate functions that brief-454 mistakenly cited as one range. They have
   their own conversions coming.
2. **A per-element allocation is a regression even when nothing else changes.**
   The association surface has been caught twice adding one, both times found
   only by counters.
3. `Keys`/`Values` and `KeyTake`/`KeyDrop` produce different result shapes on
   the same input; do not assume symmetry from the names.
4. Assert exact expressions, never `Depth` or `Length`.
5. These fixtures are nested lists — one physical level per wrapper, so raw
   4,097 is `N=4097`, not 2,048.
6. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the two release certificates and the forced-front falsifier for both flags.

Then the cost gate, **counters first**: allocation counts
(`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base against candidate, both
arms built at the same filesystem path, for a shallow `KeyTake` probe, a shallow
`KeyDrop` probe, a **`Keys` probe** and a **`Values` probe** as untouched
same-file controls, and one unrelated control — sensitivity calibrated and the
same-binary spread stated. Then `perf stat -e instructions:u`, minimum of the
tight clean cluster, migrated runs discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling** before
  your conversion is in place; afterwards the 4,097 certificates are the
  intended runs.
- **Stop-loss.** Stop and report if: a certificate does not complete with its
  exact value; the falsifier does not exercise its tail under both flags;
  allocation counts move on the `Keys` or `Values` control; or the same step
  fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The front constant and the tail's work representation, and how the
   `keep_selected` flag travels through the tail.
3. Verification: every command with verbatim output, and the ZeroMQ statement.
4. The two certificates and the falsifier under both flags.
5. Registry and inventory row changes, including the contract-status move.
6. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
7. Stop-loss and open questions.
