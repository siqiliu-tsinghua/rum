# brief-473 — convert the `Keys` collection walk

## Goal

Make `Keys` over nested list collections complete at the ruled physical height
4,097 and stop growing the native stack, then flip its registry row to
`BoundedHeap` **and `MeetsContract`**.

Like `KeyTake`/`KeyDrop` before it and unlike everything earlier in this line,
this site is **short of the contract**: the certificate is "completes at 4,097",
not "still completes at 4,097".

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3, with a liveness assertion on the exact value.

| operation | ok <= | fail >= | vs 4,097 | direction above |
|---|---|---|---|---|
| `Keys` over nested lists | 3,056 | 3,080 | **1.34x short** | **terminates the process** |

Confirmed still dying at the contract height after brief-472, which converted
its neighbours and left this walk alone.

A ceiling is not deterministic; the same height flips run to run inside a band
tens of levels wide.

## The site, and why its cost is what it is

`keys_result_for_expr` (`crates/rum-stdlib/src/associations/basic.rs:698`), one
recursive edge at `:722`:

```rust
items.iter()
    .map(|item| keys_result_for_expr(item, wrapper))
    .collect::<Result<Vec<_>, _>>()?
```

**The recursion happens inside the closure passed to `map`**, so the iterator
adaptor, the closure and the `collect` machinery are all live in the frame
across the recursive call. That is the 704 bytes per layer brief-471 measured,
against `values_result_for_expr`'s 336 on the same fixture — and it is why
`Keys` gives out at 3,056 where `Values` reaches 5,896.

**Do not propose a frame shrink instead of a conversion.** Outlining could
plausibly recover most of that 2x, and it would be the wrong repair here: the
site is *short of the contract*, so a fix sized to just clear 4,097 leaves a
margin thinner than the roughly 8% this line has watched brackets drift over
unrelated commits, and a margin thinner than the drift is not a margin. Convert
it; that is structural rather than tuned-to-fit.

brief-471 established, and I verified, that **no function in `basic.rs:690-920`
takes a `Session`**. So this walker cannot run user code and the simple
**bounded-front-plus-redo** shape applies: a depth-16 recursive front with a
cold iterative tail that may redo the traversal.

## What must not change — including one thing a value assertion cannot see

- The result shape: `Keys[{{{{"k" -> 7}}}}]` is `{{{{"k"}}}}` — a `List` wrapper
  per input level, with the innermost entry list becoming a list of keys.
- Sibling order, key order, and the `wrapper` option threading through unchanged.
- **First error wins, in source order.** `collect::<Result<Vec<_>, _>>()?`
  stops at the first `Err`, whose payload is the offending sub-expression
  (`:715-717`), and that payload reaches a user-visible diagnostic. A tail that
  visits items in a different order, or that gathers every error and reports the
  last, returns a *different* expression in the message while every successful
  case still matches. Pin it: a shallow fixture with **two** distinct malformed
  items must report the **first** one.

## Deliverable — certificates and the falsifier

**Certificates**, release-only, on declared 2 MiB threads:

- `Keys` at raw physical height 4,097 — **which it cannot reach today** — with a
  unique deepest target, asserting the **exact** result including its wrapper
  shape.
- A three-level **shallow companion** pinning wrapper shape, sibling order, key
  order, and the first-error payload described above. It is also the falsifier
  if the deep case aborts the binary.

**Forced-front falsifier**, in-module, no cargo feature and no production
switch — the shape briefs 463 and 468-472 settled on. A private
`*_with_front_depth`, called from `mod tests`, on a **branched** fixture with
more than one item per level and a distinct witness per branch:

- front `0` returns `Exhausted`;
- front `16` returns `Complete`;
- **tail == front == exact expected**.

Include an error case in the falsifier's fixture family so the tail's
first-error behaviour is compared against the front's, not only its success
behaviour.

## Registry and inventory

The `Keys` row moves **two** fields: `Mechanism` to `BoundedHeap` and
`ContractStatus` to `MeetsContract`, with `owed_by: None`.

Set `failure_direction: Some(FailureDirection::Unmeasured)`. **Do not try to
measure it** — that needs a probe above the new ceiling, which this brief
forbids. The director measures it at the gate and corrects the row; for
`KeyTake`/`KeyDrop` that measurement turned `Unmeasured` into `FailsClosed`.

Update the inventory row for the converted edge.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/basic.rs`, and a new sibling module
    under `crates/rum-stdlib/src/associations/basic/` if splitting by
    responsibility helps — `basic.rs` is 23 081 bytes;
  - `crates/rum-stdlib/src/associations/mod.rs` if exports change;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - a new focused certificate file plus its wiring in
    `crates/rum-stdlib/src/tests.rs`.
- Files you MUST NOT touch: **`values_result_for_expr` (`:727`)** and the
  `Lookup` phases — they are separate walks with their own briefs. `Values` in
  particular is nearly the same shape, and converting it here would also destroy
  the only untouched same-file allocation control this gate has. Also off
  limits: the just-converted `basic/key_take_drop.rs`, the guard substrate,
  `kernel_api.rs`, every other converted site, `metadata.rs`, `Expr`,
  `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other `docs/**`, `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended change is that this walk stops growing the
  native stack and therefore reaches 4,097. Any change to wrapper shape, sibling
  or key order, the `wrapper` option, the first-error payload, or any other
  builtin is a **stop**.

## rum / Rust pitfalls (read twice)

1. **`collect::<Result<_, _>>()` short-circuits.** Reproducing that in an
   explicit loop is easy to get subtly wrong — gathering all results and then
   checking, or checking after the loop, changes which error surfaces.
2. **A per-element allocation is a regression even when nothing else changes.**
   The association surface has been caught twice adding one, both found only by
   counters. brief-472's conversion of the neighbouring walk *removed* about
   sixteen allocations per call, so a same-shaped conversion adding any is a
   signal something is off.
3. These fixtures are nested lists — one physical level per wrapper, so raw
   4,097 is `N=4097`, not 2,048.
4. Assert exact expressions, never `Depth` or `Length`.
5. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the release certificate and the forced-front falsifier.

Then the cost gate, **counters first**: allocation counts
(`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base against candidate, both
arms built at the same filesystem path, for a shallow `Keys` probe, a
**`Values` probe** and a **`KeyTake` probe** as untouched same-file controls,
and one unrelated control — sensitivity calibrated and the same-binary spread
stated. Then `perf stat -e instructions:u`, minimum of the tight clean cluster,
migrated runs discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling.**
- **Stop-loss.** Stop and report if: the certificate does not complete with its
  exact value; the falsifier does not exercise its tail; the first-error
  behaviour cannot be preserved; allocation counts move on the `Values` or
  `KeyTake` control; or the same step fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The front constant, the tail's work representation, and how the tail
   reproduces `collect::<Result<_, _>>`'s first-error-wins behaviour.
3. Verification: every command with verbatim output, and the ZeroMQ statement.
4. The certificate, the shallow companion including its first-error case, and
   the falsifier.
5. Registry and inventory row changes.
6. The cost gate: allocation rows first with calibration and spread, then
   instruction counts.
7. Stop-loss and open questions.
