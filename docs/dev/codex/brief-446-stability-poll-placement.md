# brief-446 — move the poll to where the stack actually grows

**Shape: implementation.** Continuation of brief-445, whose work is **already in
the working tree**. Two changes: where the throttle sits, and two tests that now
declare the stack they need.

## What brief-445 proved, and what it cost

The mechanism works and the contract holds. Measured on the tree you have:

- bail-out at **5,380 levels**, against the ruled 4,097 — a real margin of
  1,283 levels (31.3%), matching the 5,386 projection;
- height 20,001 returns in band — `TerminatedEvaluation["StackLimit"]`, exit 0,
  `General::reslim: evaluation exceeded native stack limit (1048576 bytes)`;
- frame unchanged (184 B prologue, 192 B/level), counters byte-identical, all
  twelve release certificates green.

Two things blocked it, and both have a repair that does not touch any of the
above.

## Change 1 — the throttle is in the wrong place

It sits at the top of `cached_expr_stability_with_bloom`
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:205-212`), which
runs for **every** stability query: atoms, and calls that hit the cache. Neither
of those can grow the stack. Only a cache **miss** recurses.

`list-literal-small` regressed **+7.99% / +7.98%** — two passes agreeing to
0.01%, against a 1.2% same-binary spread, and the layout control moved it the
other way (−1.49% / −3.78%), so it is real. That label is atom-heavy by
construction: evaluating a small list literal is one call plus several atoms,
and after the first evaluation the call itself is a cache hit — so nearly every
increment it pays is on a path that cannot recurse.

**Move the counter and its stride test into the miss path**, immediately before
`call_stability_uncached_with_bloom`
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:261`), after the
hit return. Nothing else changes: the same monotonic wrapping counter, the same
stride, the same cold `native_stack_budget_exhausted`, the same conservative
`(false, bloom)` return.

This is also the more honest meter. The throttle then counts recursive descents
rather than lookups, which is what the stride's overshoot arithmetic already
assumes when it multiplies by 192 bytes per level.

Re-derive the bail-out height after the move and report it. It should land in
the same place, because the levels that were not counted were the ones that
consumed no stack; if it moves materially, say so rather than smoothing it.

## Change 2 — two codec tests must declare their stack

`from_toml_depth_limit_accepts_each_descent_at_the_limit`
(`crates/rum-stdlib/src/tests/codecs_toml.rs:445-458`) and
`from_yaml_depth_limit_accepts_every_collection_descent_at_the_limit` both now
fail with `StackLimitExceeded { limit: 1048576 }`.

That is not a regression to avoid. They run in **debug** on a 2 MiB libtest
thread, where per-level costs are several times the release figures, and they
were passing by consuming more stack than the declared budget permits — with
nothing between them and the edge. The budget is now enforced on this path, so
they surface. Their XML sibling is the same test at the same
`CODEC_DEPTH_LIMIT`, and brief-438 already repaired it exactly this way:

```rust
// crates/rum-stdlib/src/tests/codecs_xml.rs:307-322
std::thread::Builder::new()
    .stack_size(XML_DEPTH_ACCEPTANCE_STACK_BYTES)
    .spawn(|| {
        let mut session = core_session();
        session.set_native_stack_limit(
            native_stack_limit_for_declared_stack(XML_DEPTH_ACCEPTANCE_STACK_BYTES)
                .expect("XML acceptance stack should satisfy the supported minimum"),
        );
        …
    })
```

Give both the same treatment: a declared thread and a budget derived from that
same declared size, so the declaration is true by construction rather than a
number matched by hand.

**Three things are stops, not options:** lowering `CODEC_DEPTH_LIMIT` or the
depth either test accepts; raising `DEFAULT_NATIVE_STACK_LIMIT` or widening a
reserve; and changing either test's expected outcome to a resource error. These
tests assert that the codec accepts its documented depth, and that is what they
must keep asserting.

## Change 3 — watch the fail-closed test fail

brief-445 added a focused fail-closed test
(`crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:908`). Per the
standing rule, a check nobody has seen fail is not known to work.

Delete the strided check temporarily, run that test alone, and paste the
failure. Then restore the check and paste it green. If it passes with the
mechanism removed, it is not testing the mechanism and must be fixed before
anything else in this brief is measured.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
cargo test -p rum-stdlib --release --lib tests::iterative_traversal_depth -- --ignored
RUM_PROFILE_EVALUATION_CACHE=1 cargo test -p rum-runtime --lib runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes -- --ignored --exact --nocapture
```

Run the workspace gate **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if loopback
binding is unavailable, report the nine failures as environmental and say
plainly that those tests did not execute. `cargo test -p rum-stdlib` must be
green, which it is not today.

**Counters first**, before any timing:
`stability_checks=508 call_stability_hits=128 call_stability_misses=145`.

**Frame and slope** unchanged: 184 B prologue, 192 B/level by the `/proc` VmStk
method. gdb ptrace is denied in this sandbox; do not spend time on it and do not
quote a number you could not take.

**Bail-out** re-measured by bisection, which is safe because the walk returns
rather than aborting. Report the last success and first bail, and the margin
over 4,097.

**Height 20,001** message and exit code verbatim, naming the guard.

**A/B, same protocol.** Four passes base/candidate/base/candidate in one window,
same-binary spread first; layout control with hashes checked if a label still
looks real; within-run quotients if the panel shifts. Base is HEAD. Same panel
as brief-445. `list-literal-small` is the label this brief exists to recover —
report it first. `cases-wide-no-match` has been inconclusive in four separate
windows now; treat it as noisy rather than as a clean baseline.

**Stop-loss.** Stop and report, without a second tuning revision, if:
`list-literal-small` still regresses beyond its own same-binary spread after the
move; the bail-out drops to or below 4,097; the frame or counters move; any
certificate fails; the fail-closed test passes with the mechanism removed; or a
different panel label regresses in exchange.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`,
  `crates/rum-stdlib/src/tests/codecs_toml.rs`,
  `crates/rum-stdlib/src/tests/codecs_yaml.rs`.
- The other five files brief-445 touched are already correct; leave them alone
  unless the move requires it, and say so if it does.
- **General accept-vs-stop criterion**: a test that declares a thread and
  derives its budget from that same size is in scope even where I did not name
  it. Any other non-test source change is a stop — write it in the report.
- Do not modify `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`, the
  `Expr`/`Symbol` representation, serde shapes, or `Cargo.lock`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. The working tree
  already contains brief-445's changes; build on them and leave everything
  uncommitted. Do not apply, drop or otherwise touch the stashes.
- **Do not write, generate, or run any program whose purpose is to exhaust the
  stack.** Bisecting the bail-out is permitted because the walk returns; an
  abort anywhere is a result to report, not something to explore.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`.

## Report format

The fail-closed test's red run and green run verbatim; the counter line; the
frame and slope; the re-measured bail-out with its margin; the height-20,001
outcome; verbatim `cargo fmt --check`, all test runs, the twelve certificates
and the profiling certificate; the full A/B panel with `list-literal-small`
first and the same-binary column before any cross-binary number; and an explicit
verdict on whether the regression is gone.
