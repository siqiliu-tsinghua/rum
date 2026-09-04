# brief-444 — take the off-by-default profiling state out of the hot frame

**Shape: implementation.** One variable: where the stability walk's profiling
state lives. No poll, no bound, no control-flow change.

## Goal

`call_stability_with_bloom` computes two locals at its top and holds them across
the recursive call
(`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:224-225`):

```rust
let profile_key = call_stability_source_head_key(head.as_ref());
let timing_start = call_stability_timing_start(profile_key);
```

`call_stability_source_head_key` returns `None` immediately when profiling is
off (`crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:157-165`),
so no *work* is done on the default path — but the locals still occupy the
frame. `Symbol` is 24 bytes and the key pairs one with a head shape
(`crates/rum-core/src/expr.rs:36-40`;
`crates/rum-runtime/src/runtime/evaluation_cache/mod.rs:30-33`), so roughly 72
bytes of instrumentation state for an off-by-default feature is resident in the
hot recursive frame on every default-path call.

`AGENTS.md` and `CLAUDE.md` state the line as **no default-path cost for
diagnostics or off-by-default features**. On a recursive walk, stack bytes per
level are not a lesser kind of cost than CPU cycles — they are exactly what the
depth contract is made of. Repair it.

**Target: measured slope ≤192 bytes per level**, from today's 255.6. brief-443
established that as the reachable floor and closed 128 as structurally
unreachable, because the 48-byte ABI result area for `(bool, StabilityBloom)`
plus the 16-byte pointer cache key already exceed the remaining budget. Do not
try for it.

## Context pointers

Read `AGENTS.md`, then `docs/dev/codex/brief-443-stability-failclosed-measurement.md`
(the measurement, and my gate note naming this defect) and
`docs/dev/codex/brief-442-stability-frame-shrink.md` (how the frame got to 248).

- `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:218-302` — the
  hot function, the recording sites, and `cache_call_stability_result`, which
  also takes the key.
- `.../evaluation_cache/stability.rs:344-390` — `record_call_stability_hit`,
  `record_present_call_stability_miss`, `record_absent_call_stability_miss`.
- `.../evaluation_cache/profile/source.rs:157-175` — the key constructor and its
  already-`#[cold]` enabled half.
- `crates/rum-runtime/src/runtime/profiling/timing.rs:96-125` —
  `runtime_path_timing_start` and `RUNTIME_PATH_TIMING_STACK`, the thread-local
  stack pattern this codebase already uses for exactly this shape. Follow it.
- `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:143-165` — the
  two gates: `configured` is env-driven and loads once, `active` is toggleable.

## Deliverable

### 1. Remove both locals from the recursive frame

- **The key is derivable, so do not carry it.** It is a pure function of
  `head`. Give each recording site the `head` it already has in scope and let it
  construct the key inside its own cold, profiling-enabled path. Recomputing
  per site when profiling is on is the right trade: that path is off by default
  and its cost is not the hot path's business.
- **The start instant is not derivable, so move it off the frame.** Use a
  thread-local stack pushed by a `#[cold] #[inline(never)]` start and popped by
  the recording site, mirroring `RUNTIME_PATH_TIMING_STACK`. Spans here nest
  with the recursion, so a stack is the correct structure. When profiling is
  off, nothing is pushed and no local exists.
- `cache_call_stability_result` takes the key too
  (`stability.rs:269,285,302`); give it the same treatment.

### 2. Report which frames are actually resident

The slope is what matters, not one prologue. Use the method brief-443
established — a gdb breakpoint on the recursive symbol sampling `$rsp` minima,
cross-checked against `/proc` high-water — to say which frames are resident per
level and what each contributes, before and after. `all_args_stability_with_bloom`
is 120 bytes and it is not obvious from the prologues alone whether it is
resident; settle it rather than assuming.

### 3. Give the profiling output a gate, and watch it first

**No test anywhere enables `RUM_PROFILE_EVALUATION_CACHE`.** The per-head
profiling surface you are about to refactor has no coverage at all, so nothing
would notice if this change silently emptied it.

Add an `#[ignore]`d test that must be run with the env var set — the same shape
as the release-only certificates — evaluating a fixed script and asserting the
per-head rows and outcome counts.

**Run it against HEAD before you change anything and paste the output.** If it
does not produce per-head rows at HEAD, it is not testing what it claims and
you should fix the test before touching the source. Then run it after and paste
identical output. A test written after the refactor, against the refactor, is
not a gate.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
cargo test -p rum-stdlib --release --lib tests::iterative_traversal_depth -- --ignored
RUM_PROFILE_EVALUATION_CACHE=1 cargo test -p rum-runtime --release <the new test> -- --ignored --exact
```

Run the workspace gate **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if loopback
binding is unavailable, report the nine failures as environmental and say
plainly that those tests did not execute.

All twelve release certificates must still pass. A smaller frame gives more
headroom, so a failure there means something else broke.

**Counters first.** With profiling enabled the counter line for the standard
probe must be byte-identical to HEAD's:
`stability_checks=508 call_stability_hits=128 call_stability_misses=145`.
Produce it before any timing number.

**A/B, same protocol.** Four passes base/candidate/base/candidate in one
window, same-binary spread reported first; a layout-control binary with hashes
checked if a label still looks real; within-run quotients if the whole panel
shifts. Base is HEAD. Panel: `downvalue-exact-miss-repeat`,
`downvalue-pattern-miss-repeat`, `matcher-single-tiny`, `cases-wide-no-match`,
`rewrite-guarded-hit`, `replaceall-simple`, plus the untouched index
`do-count-null`, `do-iterator-null`, `symbol-ownvalue-read`, `set-ownvalue`,
`list-literal-small`, `replace-wide-no-match`.

Expect neutral or better: this removes state from the hot path rather than
adding any. A regression here would mean the recomputation leaked into the
disabled path, which is the thing to check first if one appears.
`cases-wide-no-match` is a noisy label in this panel — inconclusive in both
windows at the brief-442 gate — so weigh it accordingly rather than treating it
as a clean baseline.

**Deep behaviour.** Measuring the new slope by bisection is **not** safe here:
the walk still has no bail-out, so a value past the ceiling aborts. Use the gdb
`$rsp` method instead, which measures without a single abort.

**Stop-loss.** Stop and report, without a second tuning revision, if: the
measured slope does not reach 192 bytes per level — report the floor you did
reach, it is a useful number either way; the profiling test's output differs
before and after; the counters change; any certificate fails; or a panel label
regresses beyond its own same-binary spread.

## Explicitly out of scope

- Any bail-out, poll, depth parameter or bound. brief-443 measured three and
  none fit; the poll is re-measured against the smaller frame in a later brief.
- The same defect in the evaluator's own chain. brief-437 projected 0.55-0.85
  KiB per evaluator level from making `RuntimePathTimingToken` a one-word cookie
  instead of carrying an `Instant` and a full key
  (`crates/rum-runtime/src/runtime/profiling/mod.rs:411-415,469-472`). That is
  the same hard-line violation on the 4.7 KiB/level path and is worth more than
  this one, but it is a different panel and a different brief. Record it, do not
  touch it.
- Splitting any file, though three are over the organization threshold.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs`,
  and the test file the new profiling test lives in.
- **General accept-vs-stop criterion**: a module root that must change only
  because a symbol it re-exports moved is in scope; a signature change to a
  `record_*` helper inside the evaluation-cache profile cluster is in scope. Any
  other non-test source change is a stop.
- Do not modify `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`, the
  `Expr`/`Symbol` representation, serde shapes, or `Cargo.lock`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do not apply, drop or
  otherwise touch the stashes.
- **Do not write, generate, or run any program whose purpose is to exhaust the
  stack**, and do not bisect a ceiling on a walk that still aborts.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`.
- The repair must not move cost from the frame into the disabled path. A branch
  that runs when profiling is off is a regression even if the frame shrinks.

## Report format

The before/after resident-frame table with the gdb-measured slope; the
profiling test's HEAD output and post-change output, both verbatim; the counter
line; verbatim `cargo fmt --check`, the test runs and the release certificates;
the full A/B panel with the same-binary column first; and an explicit verdict
against the 192-byte target.
