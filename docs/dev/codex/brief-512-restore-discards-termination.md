# brief-512 — a truncated restore reports success

## Goal

`HostSession::restore_from_bytes` matches `Ok(_) => Ok(())`
(`crates/rum-host/src/session.rs:213-214`), discarding the entire
`SessionEvalResponse`. That response carries
`termination: Option<EvalTerminationReason>`
(`crates/rum-runtime/src/runtime/model/requests.rs:128-134`). So a restore
whose evaluation was **terminated** — a resource limit, a cancellation —
returns `Ok(())`, and the caller believes it holds the snapshot it asked
for while actually holding a truncated session.

Make a terminated restore report failure.

## The ruling this implements (director, 2026-09-22)

**A terminated restore is a `HostFailure`.** The contract of
`restore_from_bytes -> Result<(), HostFailure>` is "the session now holds
this snapshot", and a restore cut short does not satisfy it. Reporting `Ok`
is the fail-open reading.

This is the third instance of one class in this codebase — cancellation
erased by its own request prologue (briefs 397-411), the deadline dropped
across an input suspension (brief-511), and now this. The common cause is a
boundary whose return type is coarser than the outcome it receives, so the
structured termination has nowhere to go and is dropped rather than
converted.

## Explicitly out of scope

- **Do not make restore transactional.** A failed restore leaves the session
  partially populated — `self.reset()` runs *before* the restore
  (`crates/rum-host/src/session.rs:207`), so this is already true of today's
  `Err` path. Document that state; do not change it. Making restore
  all-or-nothing is a separate, larger decision.
- **Do not change how any other entrypoint handles termination.** D5 ruled
  that the general case is its own item; this brief is the restore instance
  only.
- The response's `messages` and `prints` are also discarded. **Record what
  they would contain on a terminated restore, and do not surface them** —
  widening the payload is a separate decision. One sentence in the report is
  enough.

## The repair

1. In `restore_from_bytes`, inspect the `Ok` response: if `termination` is
   `Some(_)`, convert it to the appropriate `HostFailure` rather than
   `Ok(())`. Reuse the existing termination-to-host mapping if one exists;
   if it does not, say so rather than inventing a parallel one.
2. Update the rustdoc on `restore_from_bytes` to state both halves: a
   terminated restore fails, and a failed restore leaves the session
   partially populated rather than empty or unchanged.
3. Mirror the same statement in `docs/guide/embedding.md` wherever snapshot
   restore is described.

## The test, which is the hard part

You need a restore whose evaluation terminates **deterministically**. Session
resource limits apply to the restore evaluation, so a configured limit plus a
snapshot whose definitions exceed it is the likely route — find the cheapest
deterministic trigger and say why it is deterministic. A timing-based trigger
is not acceptable; this suite must not gain a flake.

Assert the exact failure shape, and pair it with a control: an ordinary
restore of the same snapshot **without** the limit still returns `Ok(())` and
the definitions are present. Without that control the test could pass because
restore is broken in some other way.

If you cannot construct a deterministic termination, **stop and report that**
— an untested contract change is worse than a reported gap, and the trigger
question is exactly the sort of thing this brief exists to settle.

## Scope

Files you MAY modify:

- `crates/rum-host/src/session.rs`
- a new or existing `rum-host` test file for the regression
- `docs/guide/embedding.md`
- this file, for your report

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. A change that is not compiler-forced and not listed is a stop.

Do NOT touch: kernel production files, the cancellation state representation,
`Expr`/`SharedExpr`/`SharedArgs`, the builtin catalog or dispatch,
`Cargo.toml`, `Cargo.lock`, `AGENTS.md`, `TODO.md`, `PROGRESS.md`, `.git`,
the ownership-graph inventory fixtures.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

**Expected state on `main`:** 2 490 passed, 0 failed, 57 ignored on a host
that can bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if
your sandbox denies loopback you will see nine native ZeroMQ failures —
report them as environmental and say plainly those tests did not execute.
Any other failure is yours.

**No microbench A/B**: this is a host snapshot entrypoint, not an evaluation
hot path. Say so in the report rather than performing the ceremony.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`; a recorded run
of the same shape took about 160 s.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. Leave changes in the
  working tree; the director gates and commits.
- **No new `unsafe`.**
- **Do not weaken or delete a test to make the suite pass.** If an existing
  test asserts that a terminated restore succeeds, that is a contract
  conflict to surface, not to edit away — and it would be a significant
  finding.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- **Stop-loss, keyed on progress.** If the same step fails twice and the
  second attempt produced no more information, stop and report.
- Paste real verbatim output, including failures.

## Report format

1. What changed, per file, with line ranges and a one-line why.
2. The termination trigger you used and why it is deterministic — or why you
   could not build one.
3. Verification, verbatim, with the complete failure set.
4. What `messages`/`prints` would have carried, in one sentence.
5. Anything you refused, and open questions.
