# brief-511 — make `eval_with_timeout` do what its rustdoc says

## Goal

`HostSession::eval_with_timeout` arms a timeout, evaluates, and clears it
**unconditionally** (`crates/rum-host/src/session.rs:527`). When the
evaluation suspends for host input the deadline is silently gone, and the
resume runs unbounded — director-reproduced at 500 ms armed, 1.39 s actual,
**no termination reason**.

Make the code honour the contract three lines above it: the timeout is
cleared after the request **completes or fails**, and an input suspension is
neither.

## The ruling this implements (director, 2026-09-22)

**The absolute deadline is preserved across the suspension.** A resumed
segment is bounded by the *original* wall-clock deadline, not given a fresh
budget.

The reason is consistency with a kernel precedent, established from source:
`TimeConstrained` computes `next_deadline` once at entry as an absolute
`Instant` (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:55-57`),
`enforce_active_time_limit` compares against `Instant::now()` (`:165`), and
**there is no recompute, pause, or remaining-duration bookkeeping anywhere** —
an `Instant` cannot pause without code that recomputes it, and none exists.
`resume_or_suspend!` sits in that same function, so `TimeConstrained` already
suspends and already charges the suspended wall-clock time.

The consequence is accepted deliberately: **the host's own input latency is
charged to the evaluation.** The host set the budget, a sandbox wants total
wall-clock bounded, and a host that wants otherwise passes a longer timeout
or none. What is not acceptable is today's behavior, where the bound
silently disappears.

## Characterize before you change anything

brief-510 could not verify the precedent empirically — it said so instead of
inferring, and that gap is this brief's first deliverable.

Add a characterization test in the stdlib resource tests
(`crates/rum-stdlib/src/tests/debug_tools_resources.rs`) that pins today's
behavior: `Input` and `InputString` inside an active `TimeConstrained` body,
confirm a real `InputRequired`, wait past the limit using a monotonic
duration, resume, and record what actually happens. Pair it with a
generously-timed immediate-resume control, a following unconstrained request
proving restoration, and a case with two input suspensions inside one
constraint.

**If that baseline disagrees with the ruling above, STOP and report it.** The
ruling rests on the source argument; a measured contradiction falsifies the
premise and the production change must not proceed on it.

## The repair

1. **Clear on completion or failure only.** In
   `HostSession::eval_with_timeout`, do not clear when the outcome is the
   input-suspension variant — `HostEvalOutcome`'s three variants are distinct
   (`crates/rum-host/src/model/outcome.rs:13-21`) and the docs direct the
   host to resume the same evaluation (`crates/rum-host/src/session.rs:320-326`).
2. **Own the armed state privately.** Add a small private marker recording
   that `eval_with_timeout` installed this timeout; initialize it in
   `HostSessionBuilder::build` and clear it in `HostSession::reset`. Do
   **not** store a second independently advancing deadline, and do **not**
   add a public setter or a timed-resume overload — brief-510 established
   neither is necessary for this repair, and both are scope.
3. **Clear it on the resume path** once the resumed evaluation completes or
   fails, so the marker cannot outlive the request that armed it.
4. **Follow `interrupt_armed`.** It is derived from `eval_timeout.is_some()`
   (`crates/rum-runtime/src/runtime/session/session_api.rs:196-197`), so
   leaving the timeout armed across a suspension extends that flag's
   lifetime. Briefs 397-411 closed cancellation erased by its own request
   prologue; show that this repair cannot resurrect that class, and say how
   you showed it.
5. **A timeout during a resumed segment must report the ordinary documented
   termination**, not complete silently. Name which one and assert its exact
   shape.

## Out of scope, recorded not fixed

Nested file-form deadline renewal, which brief-510 diagnosed separately. Do
not expand into it; if the repair collides with it, report and stop.

## Scope

Files you MAY modify:

- `crates/rum-host/src/session.rs`
- `crates/rum-host/tests/suspended_timeout_contract.rs` (new)
- `crates/rum-stdlib/src/tests/debug_tools_resources.rs` (characterization)
- `docs/guide/embedding.md` — the host timeout semantics this settles
- this file, for your report

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. A change that is not compiler-forced and not in this list is a stop.

`session.rs` is 30 KB, in the split-assessment range. **Keep this change
together** — timeout ownership, the panic guard and host entrypoint admission
are one lifecycle responsibility — and put the new tests in their own file
rather than growing it. Do not mix any reorganization into a correctness
patch.

Do NOT touch: kernel production files beyond what is listed, the cancellation
state representation, `Expr`/`SharedExpr`/`SharedArgs`, the builtin catalog or
dispatch, `Cargo.toml`, `Cargo.lock`, `AGENTS.md`, `TODO.md`, `PROGRESS.md`,
`.git`, the ownership-graph inventory fixtures.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

`--no-fail-fast` is required: a plain `cargo test` stops at the first failing
target and silently leaves downstream crates unrun.

**Expected state of the suite on `main`:** 2 465 passed, 57 ignored, and
**nine native ZeroMQ failures only if your sandbox cannot bind loopback** —
run without `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`, report those nine as
environmental, and say plainly that those tests did not execute. Any other
failure is yours. The ownership-inventory test **passes on `main`**; if it
fails you are not on `main`, so stop and say so.

**No microbench A/B is required** and you should not run one: this touches a
host request entrypoint, not an evaluation hot path. Say that in your report
rather than performing the ceremony.

Bound the suite with `scripts/run-bounded.py --timeout 900 --` ; a recorded
run of the same shape took about 160 s.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. Leave changes in the
  working tree; the director gates and commits.
- **No new `unsafe`.**
- **Do not weaken or delete a test to make the suite pass.** If a test
  encodes the old timeout-clearing behavior, that is a contract conflict to
  surface, not to edit away.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- Timing here is a correctness question — you are checking *whether* a
  deadline fires. Derive waits from monotonic durations, never wall-clock
  differencing, and keep them short enough that the suite stays fast.
- **Stop-loss, keyed on progress.** If the same step fails twice and the
  second attempt produced no more information, stop and report.
- Paste real verbatim output, including failures.

## Report format

1. The characterization result — what today's `TimeConstrained` actually does
   across a suspension — and whether it confirms or falsifies the ruling.
2. What changed, per file, with line ranges and a one-line why; plus any
   compiler-forced file and its reason.
3. The `interrupt_armed` / cancellation analysis and how you showed the old
   class cannot return.
4. Verification, verbatim, with the complete failure set.
5. Anything you refused, and open questions.
