# brief-474 — convert the four `Lookup` walks

## Goal

Make `Lookup`'s nested key-specification handling bounded by heap work rather
than by native stack depth, and flip its registry row to `BoundedHeap`.

`Lookup` clears the ruled 4,097 today, but by **1.16x — the thinnest margin
anywhere in this class**, thinner than the `DeleteMissing` target mode this line
treated as urgent — and it fails open. A margin thinner than the drift is not a
margin: this line has watched brackets move about 8% over fourteen unrelated
commits.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3, with a liveness assertion on the exact value.

| axis | ok <= | fail >= | vs 4,097 | direction above |
|---|---|---|---|---|
| `Lookup` with a nested key specification | 4,760 | 4,784 | clears by **1.16x** | **terminates the process** |

The axis is the **key specification argument**, not the association's value
depth — `Lookup[assoc, spec]` where `spec` is a nested list. That is an
argument-shaped axis, like `KeyComplement`'s option walk, and it is reachable
without any deep data.

## The four walks — corrected from brief-471's summary

brief-471 named the four phases correctly in substance but did not cite them by
function, and two of them are not where its wording suggests. Director-verified
by enumerating every self-recursive function in the file:

| phase | function | line | self-call |
|---|---|---|---|
| count | `lookup_query_count` | `:620` | `:626` |
| planning | `lookup_entries_outcome` | `:630` | `:635` |
| owned rendering | `LookupOutcome::into_expr` | `:506` | inside its `List` arm's `.map()` |
| borrowed rendering | `LookupOutcome::to_expr` | `:522` | same shape |

`finish_prepared_lookup` (`:252`) and `finish_suspended_prepared_lookup`
(`:285`) do take a `&mut Session` but are **not recursive** — they are the
callers of the two rendering methods. So brief-471's "none of the workers takes
a `Session`" holds for the four walks that actually recurse, and the simple
**bounded-front-plus-redo** shape applies to all four: a depth-16 front with a
cold iterative tail that may redo the traversal.

All four recurse over the same nested shape — the key specification, and the
`LookupOutcome` tree that mirrors it — so a fixture that drives one drives all
four. Say in your report whether one shared front constant is right or whether
they want separate ones, and why; briefs 468 and 471's workers both independently
chose per-walk constants for genuinely separate walks.

## The hazard this file has already produced once

`LookupOutcome` is a **recursive enum** — `List(Vec<Self>)` at `:502` — so its
derived `Drop` recurses. brief-468 hit exactly this class with
`NestedPathNodes`: one implementation dropped a partially built deep tree
recursively and terminated the process at depth 60,000, and only the
cross-comparison caught it.

`into_expr` consumes the outcome as it renders, so that path destructures rather
than dropping. **`to_expr` borrows**, so on the resume path the outcome is
dropped after rendering — and any early return drops it too.

**Establish from source whether a deep `LookupOutcome` is ever dropped rather
than consumed.** If it is, bound that drop as well and certify it; if it
provably is not, say so with `path:line` and that is a complete answer.

## What must not change

- The result shape: a nested key spec produces a matching nested `List` result;
  `Lookup[<|"k" -> 7|>, {{{"k"}}}]` is `{{{7}}}`.
- Hit versus miss handling: a miss yields the `default` when one was supplied and
  `Missing["KeyAbsent", key]` otherwise, at the position where the key sat.
- The `wrapper` option threading through both rendering methods identically —
  `into_expr` and `to_expr` must stay in agreement, since one serves the ordinary
  path and the other the resume path and a divergence would show only after a
  suspension.
- Query counting: `lookup_query_count` feeds a downstream decision; its result
  must be identical.

## Deliverable — certificates and the falsifier

**Certificates**, release-only, on declared 2 MiB threads:

- `Lookup` at raw physical height 4,097 with a nested key specification, a
  unique deepest target, asserting the **exact** result.
- A certificate covering a **miss** at depth, so the `Missing`/`default`
  substitution is exercised by the deep path and not only by the companion.
- A three-level **shallow companion** pinning result shape, hit/miss handling
  and the wrapper option.

**Forced-front falsifier**, in-module, no cargo feature and no production
switch — the shape briefs 463 and 468-473 settled on. A private
`*_with_front_depth` per walk, called from `mod tests`, on a **branched** fixture
with a distinct witness per branch:

- front `0` returns `Exhausted`;
- front `16` returns `Complete`;
- **tail == front == exact expected**.

For the two rendering methods, additionally assert that `into_expr` and
`to_expr` produce **the same value** on the same outcome under both front
settings. They are the ordinary and resume paths of one contract, and nothing
else in the suite compares them.

## Registry and inventory

Flip the `Lookup` row to `Mechanism::BoundedHeap` with `owed_by: None`. Its
contract status is already `MeetsContract` and stays. Set
`failure_direction: Some(FailureDirection::Unmeasured)` and **do not try to
measure it** — that needs a probe above the new ceiling, which this brief
forbids; the director measures it at the gate and corrects the row, as happened
for `KeyTake`/`KeyDrop` and `Keys`, both of which turned out to be
`FailsClosed`.

Update inventory rows for every edge you convert.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/basic.rs`, and a new sibling module
    under `crates/rum-stdlib/src/associations/basic/` if splitting by
    responsibility helps;
  - `crates/rum-stdlib/src/associations/mod.rs` if exports change;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - a new focused certificate file plus its wiring in
    `crates/rum-stdlib/src/tests.rs`.
- Files you MUST NOT touch: **`values_result_for_expr` (`:673`)** — the last
  unconverted walk in this file, with its own brief coming, and the only
  untouched same-file allocation control this gate has. Also off limits:
  `basic/keys.rs` and `basic/key_take_drop.rs`, the guard substrate,
  `kernel_api.rs`, every other converted site, `metadata.rs`, `Expr`,
  `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other `docs/**`, `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended change is that these walks stop growing the
  native stack. Any change to result shape, hit/miss handling, the wrapper
  option, the query count, agreement between the two rendering methods, or any
  other builtin is a **stop**.

## rum / Rust pitfalls (read twice)

1. **`into_expr` and `to_expr` must stay in agreement.** They are the same
   contract on the owned and borrowed paths; converting one and not the other,
   or converting them differently, produces a divergence that only appears after
   a suspension.
2. **A recursive enum's derived `Drop` recurses.** See the hazard section.
3. **A per-element allocation is a regression even when nothing else changes.**
   The last two conversions in this file *reduced* allocations; one that adds
   any is a signal something is off.
4. The fixture axis is the key-spec argument, not data depth. Calibrate at a
   small size and show the probe reaches the intended function before scaling.
5. Assert exact expressions, never `Depth` or `Length`.
6. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the release certificates and the forced-front falsifiers, including the
`into_expr`-versus-`to_expr` agreement assertion.

Then the cost gate, **counters first**: allocation counts
(`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base against candidate, both
arms built at the same filesystem path, for a shallow `Lookup` probe, a
**`Values` probe** and a **`Keys` probe** as untouched same-file controls, and
one unrelated control — sensitivity calibrated and the same-binary spread
stated. Then `perf stat -e instructions:u`, minimum of the tight clean cluster,
migrated runs discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling.**
- **Stop-loss.** Stop and report if: a certificate does not complete with its
  exact value; a falsifier does not exercise its tail; `into_expr` and `to_expr`
  cannot be kept in agreement; a deep `LookupOutcome` drop cannot be bounded;
  allocation counts move on the `Values` or `Keys` control; or the same step
  fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The four walks: front constants (shared or per-walk, with your reason) and
   each tail's work representation.
3. The `LookupOutcome` drop question, answered with `path:line`.
4. Verification: every command with verbatim output, and the ZeroMQ statement.
5. The certificates and the falsifiers, including the rendering-agreement
   assertion.
6. Registry and inventory row changes.
7. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
8. Stop-loss and open questions.
