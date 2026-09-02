# brief-404 — implement: request-scoped, generation-qualified cancellation

Shape: **implementation**. This supersedes brief-400, whose repair mechanism
brief-401 refuted and whose tree has since moved under it.

## Where this came from

brief-397 found a real host-contract defect. A cancellation is erased by the
evaluation prologue: `begin_top_level_interrupt_budget` clears the flag as its
first statement (`runtime/session/session_helpers.rs:32`), and `eval_request`
calls it immediately (`runtime/session/session_requests.rs:130`). The
`rum-jupyter` adapter takes `session.cancel_handle()` before spawning
(`crates/rum-jupyter/src/adapter/worker.rs:88,99`), so the handle is obtained
at the right moment, the cancel is set at the right moment, and the evaluation
then throws it away. This reproduces in both directions.

brief-398 designed the repair: request-scoped, generation-qualified
cancellation rather than a session-global flag. The user ruled for it (option
乙, host-level request scope, not an adapter latch).

brief-400 began it and stopped correctly on a MAY-list omission of mine. Its
report then surfaced a separate defect, which became briefs 401-403 and is now
landed: the abandonment protocol (`fc959898`) and its caller migration plus the
overlapping-request rejection (`1afdc926`).

Two of brief-401's corrections govern this brief:

- **Do not drive the saved frames with `Err(EvalError::Cancelled)`.** That was
  brief-400's proposed repair and it is unsound: ordinary frames consume errors
  and run work. Use `Session::abandon_pending_suspension`
  (`runtime/suspension/core.rs:484`), which now exists.
- **The accepted-cancellation branch is where the abandonment entrypoint must
  be called** — today's cancellation does not raw-drop the queue.

## Reference material: brief-400's stashed work

`stash@{0}` holds brief-400's WIP. Read it, do not apply it:

```
git stash show -p stash@{0}                                   # the 9-file patch
git show stash@{0}^3:crates/rum-runtime/src/runtime/model/request_cancellation.rs
```

**Read-only. Do not `pop`, `apply`, `drop`, or otherwise mutate the stash or
any git state.** I measured what still fits: six of its nine files apply
cleanly; three conflict — `model/session.rs`, `session_requests.rs`, and
`session_state.rs` — and they conflict exactly at the integration points that
briefs 402 and 403 rewrote. So the state machine module and the 374 lines of
tests in `tests/session_requests/api.rs` are close to reusable; the wiring is
not. Treat the stash as a design reference and a test source, and write the
integration against the current tree.

## The design to implement

From brief-398, as embodied in the stashed `request_cancellation.rs`:

- `RequestInterruptState { phase: AtomicU64 }` encoding either idle or
  `(generation << 1) | cancelled_bit`.
- Accepting a request yields a **non-`Clone`** `AcceptedRequest` ticket
  carrying its generation.
- `RequestCancelHandle` is generation-qualified and cancels by
  compare-exchange, so a handle issued for generation N cannot affect
  generation N+1.
- Request completion is the linearization point.

The property this buys, and the one the tests must prove: **a cancellation
that arrives for the current request is never erased by that request's own
prologue, and a cancellation issued for a finished request never reaches the
next one.**

## Goal

Cancellation becomes request-scoped and generation-qualified; the prologue no
longer erases a live cancel; a cancel accepted while a single-source input
continuation is pending abandons that continuation structurally before any
host input is converted or delivered, returns the documented structured
termination, and leaves the session reusable.

## Do the failing test first

Write the regression that a cancel arriving before `eval_request` survives the
prologue, **run it red, and paste the failure** before any repair. Synchronize
with a channel handshake, never a sleep — see the Pitfalls.

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md`,
  `docs/guide/embedding.md`, brief-397's and brief-398's reports, brief-401's
  report §4 and §6, and brief-403's committed report.
- Defect: `runtime/session/session_helpers.rs:31-39`,
  `runtime/session/session_requests.rs:125-138`.
- Abandonment entrypoint: `runtime/suspension/core.rs:484`, with
  `AbandonReason` in the same file.
- Overlap rejection landed by brief-403:
  `Session::reject_overlapping_evaluation_request`
  (`runtime/session/session_requests.rs:46`), called as `eval_request`'s first
  statement. Cancellation acceptance and overlap rejection are both
  request-boundary preconditions; **their order is a decision you must make
  deliberately and state in the report.**
- Host surface: `crates/rum-host/src/session.rs:118-125,457-460`.
- Stale tracker entry to correct: `TODO.md:701-707` still describes this
  defect as open and cites `session_helpers.rs:30` and four sleeping tests.
  Only `crates/rum-host/tests/stress.rs:124-144` still sleeps.
- **Your sandbox has no network.** Pass `--offline` to every `cargo` command.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model/request_cancellation.rs` (new),
    `runtime/model/mod.rs`, `runtime/model/session.rs`;
  - `runtime/session/session_requests.rs`, `session_helpers.rs`,
    `session_state.rs`, `session_api.rs`, `session_eval.rs`;
  - `runtime/eval_core/mod.rs`;
  - **`runtime/suspension/core.rs`** — for the accepted-cancel call into the
    abandonment entrypoint. *This is the file my brief-400 MAY list omitted,
    which is why that brief stopped. It is in scope now.*
  - `runtime/kernel_api.rs` if the host needs a new export;
  - `crates/rum-host/src/session.rs` and `crates/rum-host/tests/`,
    `crates/rum-host/src/tests/`;
  - `crates/rum-runtime/src/runtime/tests/` (in particular
    `tests/session_requests/api.rs`);
  - `TODO.md`, and `docs/guide/embedding.md` if the cancellation contract's
    wording changes.
- Files/areas you MUST NOT touch:
  - **`HostSession::cancel_handle` must keep working.** Re-express it on the
    new mechanism; do not remove it or migrate its consumers — that is
    brief-405, deliberately separate so the public API change lands after the
    mechanism is proven.
  - `crates/rum-jupyter` — its two pump failures are briefs 405 and 406.
  - `rum-core` representation, `Cargo.lock`, dependency edges, the generated
    builtin reference;
  - any `FrameAbandon` classification, the abandonment protocol's shape, or
    brief-403's rejection contract. If one of them is wrong, **STOP and report
    it** rather than adjusting it here — that would be a gate miss of mine and
    I want to hear about it.
  - ordinary evaluator, dispatch, or matcher logic.
- If you need a file outside this list, **STOP and report**. My MAY lists have
  been short seven times on this project, including the one that stopped
  brief-400.

## The accept-vs-stop criterion

Making cancellation request-scoped is in scope. **Changing what a cancelled
evaluation reports to the host is a stop.** The documented structured
termination stays as it is; cooperative late cancellation stays cooperative.
If the repair appears to require changing the reported outcome, that is a
finding, not a licence.

## Deliverable

1. The generation-qualified state machine, replacing the raw `interrupt_flag`
   clear. `begin_top_level_interrupt_budget` must no longer erase a cancel
   belonging to the request it is beginning.
2. A non-`Clone` `AcceptedRequest`; a `RequestCancelHandle` that cannot affect
   a later generation; completion as the linearization point. `reset()` must
   retire the generation so a handle held across it is inert.
3. **Cancellation while suspended.** A cancel accepted while a single-source
   continuation is pending calls `abandon_pending_suspension` with the
   cancellation reason **before** `input.into()`, before any `Expr` is built
   from host input, and before any queued value is consumed. Prove conversion
   did not happen with a custom `Into<String>` counter, and prove statements
   after the `InputString` did not run with a rum-side sentinel.
4. **State the resulting escape route.** brief-403 documented that a
   script-shaped suspension leaves `reset()` as the only way forward. If a
   cancel can now abandon a pending continuation and leave the session usable,
   that is a second, explicitly named escape — say so in the report and in
   `docs/guide/embedding.md`. Do not invent a new public API for it; use the
   cancellation that already exists.
5. Separate public `Block`, `Module`, and `With` cases plus attributes,
   iterator, option, `Reap`, `Monitor`, trace, constraint, PRNG, and
   `AbortProtect` state proven clean after a cancel-while-suspended, reusing
   brief-403's matrix shapes.
6. Nested `WithCleanup` follows the policy brief-402 landed: cleanup runs,
   nested input is abandoned without a second host request.
7. `TODO.md:701-707` corrected: the defect is closed, and only
   `stress.rs:124-144` still sleeps.

## rum / Rust pitfalls (read twice)

1. Never call the ordinary resume driver with a synthetic `Cancelled`.
2. Do not clone or convert the resume input before checking the accepted
   cancel. Conversion is observable; the counter test will catch it.
3. Do not let a frame transform cancellation into success or run later
   continuation, fallback, or handler work.
4. A stale handle must not leak a cancel into the next request or across
   `reset()`. That is the whole point of the generation.
5. **No sleep-synchronized tests.** Use a channel handshake so the canceller
   and the evaluation rendezvous deterministically. The tracker entry you are
   closing exists because sleeping tests of this exact shape are a latent
   flake, and this session has watched the `rum-jupyter` suite produce three
   different timing failures.
6. Do not change the amortized interrupt-polling cadence in this brief.
7. No new `unsafe`.
8. Everything here is request-boundary and cold-path work. No ordinary
   evaluator-step branch, allocation, or lock.

## Verification (MANDATORY — you run it; iterate to green)

Pass `--offline` to every `cargo` command.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

`--no-fail-fast` is required on the workspace run: `cargo test` stops after a
failing binary and silently under-reports the rest.

**Known-failing, not yours.** The `rum-jupyter` suite fails intermittently at
whole-suite level on the committed tree — I measured roughly three failures in
ten whole-suite samples, with the failing test moving between
`tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle`,
`tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`,
and two `zeromq::tests::native::*` tests, each of which passes 20/30 out of 30
in isolation. Report whatever you see; do not fix it and do not touch that
crate. Any failure outside `rum-jupyter` is yours.

Note that the first of those, finding B, is the defect *this* brief repairs at
the kernel level. If it starts passing, say so — but do not chase it, and do
not modify `rum-jupyter` to make it pass.

No timing A/B: this is a correctness line on cold paths. If you change ordinary
eval, dispatch, or matcher code, the cost premise is exceeded — stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, checkout, or touch git config. Do NOT
  `stash pop`, `apply`, or `drop`. Reading `stash@{0}` with `git stash show -p`
  and `git show` is explicitly permitted and is the only git access granted.
- Leave changes in the working tree; I run the authoritative gate and commit.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. No probe artifacts in the repo.
- **Testing Policy is a hard line**: rum's own documented behavior only.
- **No new `unsafe`.**
- Minimal diff; no drive-by reformatting or renames.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-file change with line ranges and a one-line why; what you reused from
   the stash and what you rewrote, and why.
2. The request-boundary ordering decision: where cancellation acceptance sits
   relative to brief-403's overlap rejection, and why that order is right.
3. The red test verbatim before the repair, then the same test green.
4. Cancel-while-suspended evidence: the conversion counter, the side-effect
   sentinel, and the scope-cleanliness matrix.
5. Verification output, including the repository invariant rows.
6. Cost statement.
7. Stop-loss, risks, open questions.
