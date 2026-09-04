# brief-441 — take the cold path back off the hot one

**Shape: implementation.** Continuation of brief-440, whose work is **already
in the working tree**. One variable: the shape of the cached-lookup control
flow. No semantic change at all.

## Goal

brief-440's conversion is functionally correct and complete — every invariant
test passes, the twelve release certificates pass including the three repaired
ones, and the counters are byte-identical between base and candidate
(`stability_checks=664`, `call_stability_hits=132`,
`call_stability_misses=216`, `insertions=138`). No extra work is being done.

It was nevertheless rejected at the A/B, and correctly stopped there:

| label | base same-binary spread | candidate | layout control |
|---|---:|---:|---:|
| `downvalue-exact-miss-repeat` | **0.062%** | **+6.283% / +4.790%** | +0.469% / −0.976% |
| `downvalue-pattern-miss-repeat` | **0.105%** | **+4.358% / +2.926%** | +0.349% / +0.076% |

Those movements are roughly fifty times their own noise floor and are not
reproduced by the layout-only control, so they are real.

When you are done, both labels are back inside their same-binary spread, every
test and certificate still passes, and the counters are still identical.

## The diagnosis — this is not yours to re-derive

The cost is code shape on the cached-lookup path, not work.

`begin_call_stability_check` returns `CallStabilityCheck`, an enum whose size is
that of its `Miss(PendingCallStabilityCheck)` variant. Before brief-440 the
lookup returned `(bool, StabilityBloom)` directly on a hit and fell straight
through to the compute-and-insert on a miss, all in one function
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs` before the
change). Now:

- **every** lookup, hit or miss, moves the larger enum;
- **every miss** additionally constructs `PendingCallStabilityCheck` — key,
  profile key, `Option<Instant>`, miss outcome — and moves it across two
  function boundaries where the code used to be straight-line, then destructures
  it in `finish_call_stability_check`.

`downvalue-exact-miss-repeat` and `downvalue-pattern-miss-repeat` are
miss-heavy by construction, which is why they and only they show it.

My brief-440 said the precedents "keep the borrow in the bounded front and only
pay for owned state in the cold fallback." That was applied to `Expr` clones but
not to the control flow, and the control-flow split is what costs.

## Deliverable

**The bounded recursive front keeps its original single-function shape.** One
function performs key construction, entry validation, the hit return,
the uncached compute, and the insert, straight-line, returning
`(bool, StabilityBloom)` — exactly as before brief-440, plus the depth
parameter and the `Option` return that makes the fallback reachable.

`begin_call_stability_check` / `finish_call_stability_check` and
`PendingCallStabilityCheck` survive **only for the cold iterative machine**,
which is entered after the recursive call budget is exhausted. Nothing on the
bounded path may construct or move them.

That means the lookup and insert logic exists in two places. **That duplication
is the point and is accepted**; do not undo it to satisfy a tidiness instinct.
If you would rather not duplicate, factor the *pieces* — key construction, entry
validation, the insertion decision — into `#[inline]` helpers that both the hot
function and the cold machine call, so the hot path is still straight-line
through them. Splitting the *control flow* is what must not happen.

Constraints:

- **No semantic change whatsoever.** The counters above must come back
  byte-identical, and every equivalence test brief-440 added must pass
  unmodified. If you find yourself editing an equivalence test, stop and report.
- The five invariants (pointer-key lookup and stable-only insertion, the
  early-unstable short-circuit, bloom union order, data-constructor and
  head-effect order before the argument walk, cache insertion order) are already
  covered by those tests. Do not weaken them.
- Leave the memory-proxy conversion in
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` alone. It
  passed, and it cannot reach the default path because it runs only under an
  armed memory constraint.
- Leave the repaired certificates and their shallow companions alone.
- Keep the `#[cfg(test)]` insertion trace test-only.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
cargo test -p rum-stdlib --release --lib tests::iterative_traversal_depth -- --ignored
```

Run the workspace gate **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`. If loopback
binding is unavailable, report the nine failures as environmental and say
plainly that those tests did not execute.

**Counters before timings.** Produce the counter line first and show it matches
the values quoted above. If it does not, the change is not semantically neutral
and you should stop rather than time it.

**A/B, same protocol as brief-440** — four passes base/candidate/base/candidate
in one window, same-binary spread reported first, layout-control binary with the
hashes checked, within-run quotients if control adjustment moves the panel.
Same panel. The base for this brief is **HEAD `384df8aa`**, i.e. the tree
*without* brief-440's changes, so the comparison answers the question that
matters: does the conversion as a whole now cost anything?

The acceptance criterion is specific: `downvalue-exact-miss-repeat` and
`downvalue-pattern-miss-repeat` must both fall inside their own same-binary
spread, and no other panel label may regress beyond its own spread after the
layout control and quotient checks.

**Stop-loss.** Stop and report if: the counters change; any equivalence test,
certificate, or companion fails; the two labels are still outside their spread
after one revision; or a different label regresses in exchange. Do not tune a
third time — report what the shape costs and let me decide.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs`,
  `crates/rum-runtime/src/runtime/eval_core/stability.rs`.
- **General accept-vs-stop criterion**: a module root that must change only
  because a type or function it re-exports changed is in scope. Any other
  non-test source file is a stop — write it in the report.
- Do not modify any test file. brief-440's tests are the gate; changing them
  changes the gate.
- **Do not split any file.** Three files are now over the organization
  threshold (`eval_core/stability.rs` 21,529 bytes,
  `evaluation_cache/tests.rs` 40,113, `memory.rs` 35,654). That is real and I
  will brief it separately. A file split changes code layout, which this
  project has measured moving labels by up to 4.45%, and it would confound this
  A/B.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. The working tree
  already contains brief-440's changes; build on them and leave everything
  uncommitted.
- **Do not write, generate, or run any program whose purpose is to exhaust the
  stack.**
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`.
- No default-path cost for diagnostics or off-by-default features.

## Report format

The diff of the shape change; the counter line proving semantic neutrality;
verbatim `cargo fmt --check`, the three `cargo test` runs, and the release
certificates; the full A/B panel against HEAD `384df8aa` with the same-binary
column first, plus the layout control and quotients where used; and an explicit
verdict on the two named labels.
