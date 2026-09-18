# brief-475 — convert the `Values` collection walk

## Goal

Make `Values` over nested list collections bounded by heap work rather than by
native stack depth, and flip its registry row to `BoundedHeap`. This is the last
unconverted walk in `crates/rum-stdlib/src/associations/basic.rs`; its three
siblings — `KeyTake`/`KeyDrop`, `Keys` and the four `Lookup` phases — are done.

## What is given — measured, do not re-run, do not probe past a ceiling

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3, with a liveness assertion on the exact value.

| operation | ok <= | fail >= | vs 4,097 | direction above |
|---|---|---|---|---|
| `Values` over nested lists | 5,896 | 5,904 | clears by 1.44x | **terminates the process** |

A ceiling is not deterministic; the same height flips run to run inside a band
tens of levels wide.

## The site

`values_result_for_expr` (`crates/rum-stdlib/src/associations/basic.rs:341`),
one self-recursive edge at `:363`, entered from `:181`.

brief-471 established, and I verified, that no function in this file's key
family takes a `Session`, so the **simple bounded-front-plus-redo** shape
applies: a depth-16 recursive front with a cold iterative tail that may redo the
traversal. `basic/keys.rs`, `basic/key_take_drop.rs` and `basic/lookup.rs` are
three worked examples of exactly this, all gated in the last few days.

## Two differences from `Keys` that a copy would erase

`Keys` and `Values` look like mirror images and are not. Both were measured.

1. **`Values` already uses an explicit loop**, `Vec::with_capacity` plus `push`
   (`:360-367`), where `Keys` used `.map().collect::<Result<_, _>>()`. That is
   why `Values` costs 336 bytes per layer against `Keys`' 704 and reaches 5,896
   against 3,056. **Do not import `Keys`' front/tail code wholesale** — its
   shape exists to undo an iterator-adaptor cost `Values` never had. If your
   conversion makes `Values` allocate or cost more per level, that is the signal.
2. **Their error payloads differ, and the difference is user-visible.**
   Director-verified at the CLI:

   | call | reported argument |
   |---|---|
   | `Keys[{{1, 2}}]` | `1` — the innermost offending element |
   | `Values[{{1, 2}}]` | `List[List[1, …]]` — the whole top-level argument |

   `Values` returns `Err(expr.clone())` at **the level that failed**, not the
   innermost one. A conversion that "harmonises" the two would silently change
   `Values`' diagnostic while every successful case still matched. Pin it: a
   shallow fixture with a malformed item nested two levels down must report the
   **outer** expression.

## What must not change

The result shape (`Values[{{{{"k" -> 7}}}}]` is `{{{{7}}}}`), sibling order, the
`wrapper` option threading through, and the error payload above.

## Deliverable — certificate and falsifier

**Certificate**, release-only, on a declared 2 MiB thread: `Values` at raw
physical height 4,097 with a unique deepest target, asserting the **exact**
result. Plus a three-level **shallow companion** pinning result shape, sibling
order and the outer-expression error payload.

**Forced-front falsifier**, in-module, no cargo feature and no production
switch — the shape briefs 468-474 settled on. A private `*_with_front_depth`
called from `mod tests`, on a **branched** fixture with a distinct witness per
branch: front `0` returns `Exhausted`, front `16` returns `Complete`, and
**tail == front == exact expected**. Include an error case so the tail's payload
is compared against the front's.

## Registry and inventory

Flip the `Values` row to `Mechanism::BoundedHeap` with `owed_by: None`; its
contract status is already `MeetsContract` and stays. Set
`failure_direction: Some(FailureDirection::Unmeasured)` and **do not try to
measure it** — the director measures it at the gate and corrects the row, as
happened for `KeyTake`/`KeyDrop`, `Keys` and `Lookup`, all three of which turned
out to be `FailsClosed`. Update the inventory row for the converted edge.

## Scope

- Files you MAY modify: `crates/rum-stdlib/src/associations/basic.rs` and a new
  sibling module under `crates/rum-stdlib/src/associations/basic/`;
  `crates/rum-stdlib/src/associations/mod.rs` if exports change; the registry
  and recursion inventory (row updates only); a new focused certificate file
  plus its wiring in `crates/rum-stdlib/src/tests.rs`.
- Files you MUST NOT touch: `basic/keys.rs`, `basic/key_take_drop.rs`,
  `basic/lookup.rs`, the guard substrate, `kernel_api.rs`, every other converted
  site, `metadata.rs`, `Expr`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other
  `docs/**`, `Cargo.lock`.
- **Accept-vs-stop:** the only intended change is that this walk stops growing
  the native stack. Any change to result shape, sibling order, the `wrapper`
  option, the error payload, or any other builtin is a **stop**.

## rum / Rust pitfalls (read twice)

1. `Values` is not `Keys` mirrored — see the two differences above.
2. A per-element allocation is a regression even when nothing else changes; the
   last three conversions in this file held counts flat or reduced them.
3. These fixtures are nested lists — one physical level per wrapper, so raw
   4,097 is `N=4097`.
4. Assert exact expressions, never `Depth` or `Length`.
5. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the release certificate and the forced-front falsifier.

Then the cost gate, **counters first**: allocation counts (`RUM_PROFILE_ALLOC=1`,
`--features alloc-count`), base against candidate, both arms built at the same
filesystem path, for a shallow `Values` probe, a **`Keys` probe** and a
**`Lookup` probe** as already-converted same-file controls, and one unrelated
control — sensitivity calibrated and the same-binary spread stated. Then
`perf stat -e instructions:u`, minimum of the tight clean cluster, migrated runs
discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. **No new `unsafe`.** No network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling.**
- **Stop-loss.** Stop and report if: the certificate does not complete with its
  exact value; the falsifier does not exercise its tail; the error payload
  cannot be preserved; allocation counts move on a control; or the same step
  fails twice for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed, per file, with line ranges and a one-line why.
2. The front constant, the tail's work representation, and how the tail
   preserves the outer-expression error payload.
3. Verification with verbatim output, and the ZeroMQ statement.
4. The certificate, the companion including its error case, and the falsifier.
5. Registry and inventory row changes.
6. The cost gate: allocation rows first, then instruction counts.
7. Stop-loss and open questions.
