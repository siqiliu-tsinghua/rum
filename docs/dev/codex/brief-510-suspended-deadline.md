# brief-510 — design: the deadline that disappears across an input suspension

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and a drafted implementation brief. Write no code.

## Goal

The 2026-09-02 audit's MEDIUM finding, director-re-probed 2026-09-22 and
still present verbatim: `HostSession::eval_with_timeout` arms a timeout,
evaluates, and then clears it **unconditionally**
(`crates/rum-host/src/session.rs:519-524`). If the evaluation *suspends* —
on `Input[]`, say — the timeout is gone, and `resume_eval_request_with_input`
never re-arms it. Director-reproduced at the time: a 500 ms deadline,
suspended, resumed, ran 1.39 s to completion with **no termination reason**.

Design the repair and the contract it should have.

## The finding is smaller than it looks, and that matters

**The code already contradicts its own rustdoc.** Three lines above the bug,
the documentation says the timeout "is cleared after the request completes or
fails" (`crates/rum-host/src/session.rs:511`) — and a suspension is neither.
So the primary repair needs **no new contract**: make the code do what the
documentation already promises, clearing on completion or failure and not on
suspension.

Confirm that reading before you build on it. If the rustdoc is ambiguous
rather than violated, say so.

## What the kernel already decided, so you do not invent a contract

- `Session` keeps **two** fields: `eval_timeout: Option<Duration>` (the
  configured budget) and `eval_deadline: Option<Instant>` (the computed
  absolute deadline). `set_eval_timeout` stores the duration and **clears the
  computed deadline** (`crates/rum-runtime/src/runtime/session/session_api.rs:193-198`).
- `TimeConstrained` saves the prior absolute deadline, installs its own, and
  restores it (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:55-57,165,233`).
- Suspension has `restore_time_deadline`
  (`crates/rum-runtime/src/runtime/suspension/core.rs:343-344`), so deadlines
  already travel across suspension machinery.

Establish from that what the **current** behavior of a `TimeConstrained`
spanning a suspension actually is — probe it, do not infer it — because
whatever the kernel already does is the precedent the host layer should match
rather than contradict.

## The one contract choice, to be surfaced not silently taken

Once the timeout survives the suspension, a resumed evaluation needs a
budget, and there are two honest readings:

- **(i) Per-segment.** Each resumed evaluation gets a fresh full `Duration`.
  This falls out of the existing lazy `eval_deadline` recompute and is the
  smaller change. Its weakness: a program that suspends repeatedly can run
  unboundedly in total, N segments times the budget.
- **(ii) Whole-request absolute deadline.** The original `Instant` persists
  across the suspension, so total wall-clock is bounded. Its weakness: time
  spent *waiting for the host to supply input* is charged to the evaluation,
  so a slow human times out a computation that did nothing wrong.

Give each its consequence for a sandboxed multi-tenant host and for an
interactive REPL, say which the existing kernel precedent points at, and
**recommend one**. Note whether a host should be able to choose — the audit
observed there is no `set_eval_timeout` on `HostSession` and no timeout
variant of `resume_with_input`, and whether to add either is part of your
recommendation, not a given.

## What else to establish

1. **Every suspension exit from a timed request**, not just `Input[]`.
   Enumerate them with `path:line`, and say for each whether the same clear
   fires.
2. **Cancellation and poisoning interactions.** The suspension line
   (briefs 397-411) closed cancellation erased by its own request prologue;
   check that the repair cannot resurrect that class. `interrupt_armed` is
   set from `eval_timeout.is_some()` (`session_api.rs:196-197`), so leaving
   the timeout armed across a suspension changes that flag's lifetime —
   follow that through.
3. **What a timeout during a resumed segment reports.** The finding says the
   overrun completed with *no termination reason*; the repair must produce
   the ordinary documented termination, and you should say which one and
   verify the shape.
4. **The test that would have caught it.** Name it, and include it in the
   drafted brief's gate.

## What I want back

1. Confirmation or correction of the rustdoc-violation reading.
2. The suspension-exit inventory and the cancellation/`interrupt_armed`
   analysis.
3. The (i)-versus-(ii) recommendation with its reasoning, and the host-API
   question answered.
4. A drafted implementation brief in `TEMPLATE.md` shape: exact files and
   functions, the test gate including the regression test from §4, and
   whether any A/B is needed (this is not an evaluation hot path, so say so
   plainly if none is).
5. Anything you found that changes the finding's severity in either
   direction.

## Hard rules

- **No source change.** No `.rs`, no script, no fixture, no doc except
  appending your report to this file. Scratch and `.rum` probes under
  `target/brief-510/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Bound anything long with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run of
  the same shape.
- Timing probes are a correctness question here, not a performance one — you
  are checking *whether* a deadline fires, not how fast anything is. Do not
  run the microbench panel.
- **Stop-loss, keyed on progress.** If a probe fails twice and the second
  attempt produced no more information, stop and report what you have.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. The rustdoc reading, confirmed or corrected.
2. Suspension-exit inventory and the cancellation/`interrupt_armed` analysis.
3. The contract recommendation, with the host-API answer.
4. The drafted implementation brief.
5. Severity changes and any director decisions you hit.
