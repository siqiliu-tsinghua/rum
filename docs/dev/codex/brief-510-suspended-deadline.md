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

---

## Executing-agent report — 2026-09-22

Design only; no implementation was changed. **One required observation remains
unverified:** I could not run a delayed-input **resume** of `TimeConstrained`
through the available executables without writing a new Rust harness, which
the Hard rules prohibit. The CLI probes below establish actual suspension,
and existing tests establish ordinary timeout, cancellation, overlap and
poison behavior; neither substitutes for that missing observation. The
absolute-deadline recommendation below is therefore source-supported, not a
claim that the requested suspension/resume precedent was experimentally
confirmed. A question requesting an existing local executable was sent; no
path had been supplied when this report was written.

All new scratch, `.rum` probes and captured output are under
`target/brief-510/`. `brief-prefix.txt` preserves this file before the append;
`source-hashes.json` records the principal source files examined.

### 1. Rustdoc reading: confirmed, with a correction to the proposed mechanism

The public contract distinguishes suspension from failure explicitly:
`crates/rum-host/src/model/outcome.rs:13-21` defines three separate variants,
including “Evaluation suspended for host-provided input.”
`crates/rum-host/src/session.rs:320-326` says to resume that same evaluation.
Consequently, “cleared after the request completes or fails” at
`crates/rum-host/src/session.rs:511` does not describe `InputRequired`.
The runtime's internal `Result::Err` transport for input does not turn that
public suspension into a failed host request.

Current source, verbatim (`crates/rum-host/src/session.rs:509-512,525-528`):

```text
    /// Evaluates source with a wall-clock timeout for this request.
    ///
    /// The timeout is cleared after the request completes or fails.
    /// A pending input suspension is rejected without replacing its continuation.
                session.session.set_eval_timeout(Some(timeout));
                let outcome = session.eval_request_unprotected(request);
                session.session.set_eval_timeout(None);
                outcome
```

Thus the primary finding is a rustdoc violation, not merely an unspecified
policy. The line numbers have moved from those quoted in the brief: the
unconditional clear is now line 527.

**The current code does not have the suggested per-resume lazy recompute.**
`set_eval_timeout` clears `eval_deadline`, but the only production assignment
that computes a deadline is in `begin_top_level_interrupt_budget`:

```text
crates/rum-runtime/src/runtime/session/session_helpers.rs:30:    pub(in crate::runtime) fn begin_top_level_interrupt_budget(&mut self) {
crates/rum-runtime/src/runtime/session/session_helpers.rs:32:        self.eval_deadline = self.eval_timeout.map(|timeout| Instant::now() + timeout);
```

Its production callers are source-request entry
(`session/session_requests.rs:158`), expression-request entry (`:186`) and
text/script-form entry (`session/session_eval.rs:215`), all beneath
`crates/rum-runtime/src/runtime/`. Neither
`resume_eval_request_with_input` (`session/session_requests.rs:265`) nor
`resume_suspended_evaluation` (`suspension/core.rs:482`) calls it. `evaluate`
(`resolution.rs:292`) does not recompute the host deadline either.
Simply preserving **both** host timeout fields across an ordinary suspension
therefore retains the already-computed absolute `Instant`. Preserving only
the duration while clearing its deadline would leave this resume path
unprotected. A fresh full budget per segment would require an explicit new
rearm, not fall out of today's resume implementation.

There are also **two distinct deadline lanes**, not one shared field:

- The host uses `eval_timeout` and `eval_deadline`
  (`crates/rum-runtime/src/runtime/model/session.rs:121-122`).
- `TimeConstrained` uses `current_time_deadline`
  (`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:55-57`).
  `restore_time_deadline` in `suspension/core.rs:343-344` restores this
  **language constraint** during abandonment; it is not a host-timeout
  preservation API.

### 2. Suspension exits, cancellation, arming and poison

The shipped runtime has one input-suspension producer:
`crates/rum-runtime/src/runtime/builtins_input.rs:231-245`,
`drive_input_request`. Its non-queued branch installs the suspension and
returns `EvalError::InputRequired`. Searching for `begin_input_suspension`
finds its definition at `suspension/core.rs:436` and this producer call at
`builtins_input.rs:243`. This makes the inventory exhaustive by producer
and public request exit; the many continuation frames are propagators, not
additional host suspension kinds.

| Path from a timed source request | Source evidence | Does line 527 run? |
| --- | --- | --- |
| `Input[]`, with an empty response queue | `crates/rum-runtime/src/runtime/builtins_input.rs:87-101`; the parsed response is evaluated at `:301-314` | Yes, when the initial request returns `InputRequired`. |
| `InputString[]`, including prompt and `FieldMasked` variants | `crates/rum-runtime/src/runtime/builtins_input.rs:78-84,180-229,231-245` | Yes; it uses exactly the same producer. |
| `Dialog[]` or `Dialog` with input in its body/prolog/bindings | `crates/rum-runtime/src/runtime/builtins_input.rs:41-75`; omitted body becomes `Input[""]` at `:281-283` | Yes. `Dialog` is an input wrapper, not a fourth public outcome. |
| Input reached inside an argument, condition, matcher callback, definition, scope, loop or other saved continuation | Fixed-point propagation: `crates/rum-runtime/src/runtime/resolution.rs:351-360`; frame insertion: `suspension/core.rs:654` and following | Yes, if it escapes to the initial source-request exit. No builtin-specific exception exists in the host wrapper. |
| Input while evaluating a `TimeConstrained` limit, its active body, or its fallback | `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:25-36`, `:59-72`, `:237-245` respectively | Yes in all three cases. The language deadline is not installed yet in the limit case, remains installed in the body case, and has been restored at `:233` before fallback evaluation. The host budget is independent of these distinctions. |
| Input during `WithCleanup` cleanup after the primary expression has returned or failed | `crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:80-93` | Yes if reached during the initial timed call; a cleanup suspension is still a live request. |
| Initial public source-request suspension exit | `crates/rum-runtime/src/runtime/session/session_requests.rs:233-243`; `crates/rum-host/src/model/outcome.rs:409-423`; `crates/rum-host/src/session.rs:409-413` | Yes. This is the common funnel for all preceding rows. |
| A second or later suspension, including input supplied to `Input` that itself evaluates another `Input` | `crates/rum-runtime/src/runtime/builtins_input.rs:306-314`; `suspension/core.rs:634-648`; `session/session_requests.rs:297-313,328-338`; `crates/rum-host/src/session.rs:551-562` | No new execution of line 527: resume never enters `eval_with_timeout`. The first suspension has already lost the host budget. The repair must retain it at every later suspension too. |

Pre-queued responses take `builtins_input.rs:232-240` and do not suspend.
`Pause` uses `thread::sleep` (`:114-133`); it is not a host-input suspension.
`Return`, `Abort`, `Exit`, ordinary syntax/evaluation errors, resource
termination and panic are likewise not input suspension exits.

Other public evaluation shapes are important to the cleanup inventory:

- `call` can suspend through `eval_expr_request`, but has no timeout-taking
  counterpart (`crates/rum-host/src/session.rs:349-357,455-459`).
- `eval_script` and `eval_file` can report input, but are not entrypoints to
  this timed wrapper and have the documented non-resumable script limitation
  (`crates/rum-host/src/session.rs:360-405`). They must not gain accidental
  time limits leaked from an earlier timed request.
- `eval`, `call`, `eval_script`, `eval_file`, `eval_with_timeout` and
  `resume_with_input` can all be the host call that finishes a previously
  **cancelled** pending evaluation. Source/script/file request preflights are
  at `crates/rum-runtime/src/runtime/session/session_requests.rs:154-158`,
  `:366-369`, `:399-402`; typed-call preflight is at
  `crates/rum-host/src/session.rs:416-426`; resume preflight is at runtime
  `session_requests.rs:269-273`. Clearing only in `resume_with_input` would
  miss these alternate completion paths.

**Overlap is an additional concrete trap.** Today's timed wrapper calls
`set_eval_timeout(Some(timeout))` before the runtime's pending-request guard.
A rejected new timed request consequently replaces and then clears timeout
state belonging to the old request. A conditional clear on `InputRequired`
alone does not repair that: the rejected call is a `Failure`, although the
original continuation still exists. Run the existing cancellation-first
preflight and overlap guard **before** installing the new timeout. On
rejection, retain the original budget and continuation. Do not decide
liveness from `pending_input_request()`:
`queue_input_response` clears that display marker at
`crates/rum-runtime/src/runtime/session/session_api.rs:266-268` without
removing the continuation. The existing overlap query uses the suspension
generation (`session/session_requests.rs:70-84`) and remains authoritative.

**Cancellation and `interrupt_armed`.**

1. `set_eval_timeout` now arms with `eval_timeout.is_some() ||
   active_accepted_request.is_some()` (`session_api.rs:193-197`), not just
   the timeout predicate quoted in the brief. Without a cancel handle,
   retaining the timeout keeps the flag true throughout suspension and
   resumed work. With an active accepted ticket, the existing unconditional
   clear already leaves cancellation polling armed; losing the timeout does
   not itself erase that ticket.
2. The cold poll checks cancellation first, then `eval_deadline`, every
   1,024 poll calls (`crates/rum-runtime/src/runtime/eval_core/mod.rs:11,91-104`).
   Loop and mutation callers also use the same armed flag; for example
   `kernel_data/control_data/branching_loops.rs:150,237,297`.
   Keeping the current deadline and poll state does not require modifying
   any of these hot callers.
3. The accepted ticket and generation-qualified cancellation bit must remain
   untouched. `RequestCancelHandle::cancel` and completion compare/exchange
   only their own generation
   (`crates/rum-runtime/src/runtime/model/request_cancellation.rs:181-214`).
   The existing budget prologue does not clear cancellation; the proposed
   host repair must not add such a clear or start a new request on resume.
4. Cancellation accepted while suspended is handled before `input.into()`
   (`session/session_requests.rs:269-286`). It takes the accepted ticket,
   structurally abandons the pending frames and finishes that same ticket
   (`:46-67`). Normal completion closes it at `:315-319`; a late cancel
   can also win at `:298-310`. Preserve those ordering rules, including
   cancellation precedence over an already-expired deadline.
5. Taking the accepted ticket sets `interrupt_armed = eval_deadline.is_some()`
   (`session/session_helpers.rs:84-93`). After this repair it can therefore
   stay true during cancellation cleanup, until the host clears the timer
   after the cancellation outcome. Structural scope restoration is not a
   fresh guest continuation. `WithCleanup` does evaluate cleanup guest code
   (`suspension/core.rs:558-577`); that code remains subject to the existing
   cooperative limits. Do not clear the deadline early just to make cleanup
   unlimited. Test restoration, no continuation execution, a short cleanup,
   nested cleanup suspension and final `Cancelled` classification; surface
   any disagreement with the existing cleanup contract rather than weakening
   its assertions.
6. Completion/failure must clear the host-owned timeout after runtime
   completion has retired cancellation. This removes the deadline and
   recalculates the armed flag without writing the cancellation phase word.
   Reset rebuilds the runtime and retires the old control block
   (`crates/rum-host/src/session.rs:162-175`).

**Panic already has a separate cleanup hole:** a panic inside the current
guard body bypasses line 527. `guard` catches it, marks the session poisoned
and returns the panic outcome (`crates/rum-host/src/session.rs:661-672`).
Later guest requests are rejected until reset. The repair should clear its
host-owned timeout in that caught-panic branch too, including a panic during
resume/input conversion. It must not run guest cleanup or claim the poisoned
kernel is reusable. Retained guest state is discarded by the existing reset
contract. A normal timeout remains recoverable and does not poison.

### 3. Contract recommendation, evidence and host API

| Policy | Multi-tenant host allowing input | Interactive host/REPL |
| --- | --- | --- |
| (i) Fresh full duration per resumed segment | Bounds each segment cooperatively, but repeated suspensions multiply the total allowance. Needs a separate total-request quota/watchdog. | Does not charge human waiting, but also forgives all earlier compute at every prompt. This is not a cumulative active-compute budget. |
| (ii) Retain the original absolute deadline | Repeated prompts do not renew the budget. Host wait consumes it too, so queue/input latency belongs in admission and scheduling policy. | A slow human can return after the deadline. Interactive hosts should normally use untimed `eval` plus explicit cancellation unless they intentionally want a whole-request expiry. |

**Recommend (ii) for `eval_with_timeout`, with cooperative enforcement.** It
fits “for this request,” preserves the already-computed deadline, and does
not let guest-controlled prompts renew host authorization. The current
source also points at (ii) for `TimeConstrained`: the body suspension leaves
`current_time_deadline` installed (`constraints.rs:70-72`), resumed fixed-point
evaluation checks active resource limits (`resolution.rs:400`), the time
check compares against the existing `Instant` (`constraints.rs:164-168`),
and the constraint frame restores the prior deadline only when finishing or
abandoning (`constraints.rs:227-249,343-364`). Nested constraints take the
earlier deadline (`:296-300`). This is source analysis pending the empirical
resume check described below.

Retaining a deadline is not a background timer: while the host is waiting,
no worker is being preempted and no unsolicited result is delivered. Expiry
is observed when resumed execution reaches a cooperative check. Short
segments below a host polling threshold can still finish; blocking callbacks
and `Pause` are not made preemptible by this repair. Do not promise a strict
process wall-clock cap or add an eager resume poll without separately
deciding that semantic change.

**Host API answer:** add neither `HostSession::set_eval_timeout` nor
`resume_with_input_and_timeout` in this repair. Existing callers chose a
request timeout; ordinary `resume_with_input` must honor that choice without
extra host work. A mutable session setter would blur request ownership and
an implicit rearm-on-resume would quietly choose (i). A host can already
choose between timed and untimed evaluation and can use its qualified cancel
handle for an external scheduling policy. If a later product requirement
needs both timed policies, expose an explicitly named request policy, with
separate rules for active-compute accounting; do not overload this fix with
that API. No adapter changes are required by the recommendation.

**Ordinary unhandled timeout shape:**

- `HostEvalOutcome::Success`;
- `value.text == "TerminatedEvaluation[\"TimeConstraint\"]"`;
- `termination == Some(HostTerminationReason::TimeConstraint)`;
- a `General::timelim` message, with the existing structured outputs;
- no pending input, no poison, and a usable subsequent untimed request.

The resumed result passes through the same finalizer at
`crates/rum-runtime/src/runtime/session/session_requests.rs:322-359` as the
initial value at `:226-262`. Conversion is at
`session/session_eval.rs:37-60`, the reason/message mapping is at
`session/session_helpers.rs:327,358`, and host conversion is at
`crates/rum-host/src/model/outcome.rs:388-405`.
Do not return `HostFailureCode::TimeConstraintExceeded` for this ordinary
source/resume case. `TimeConstrained` has different surrounding behavior:
its local handler consumes `TimeConstraintExceeded` and returns its fallback
or `$Aborted` (`constraints.rs:235-249`), so such a handled outcome need not
carry a top-level termination reason. Cancellation remains `Cancelled` when
its existing precedence applies.

#### What was actually run

Reused existing binaries; no build or new harness was written:

```text
target/brief-506/build/debug/deps/rum_host-a119f062096badf7 9c25c8d877824b3f521d05a28410ad67d270bc2e05e2b1ad437a720cf60cec12
target/brief-506/build/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1 5778eeb50336f461554734b7a6ffaca706e75ff3d6cdccd5c741762b859ca403
target/brief-507/build/release/rum-cli eb306640457ca20fdb6dd6fcebb2dd9ddb7abca453e85f88575a39aa29b1abb6
```

The runtime and adapter test executables used were also from
`target/brief-506/build/debug/deps/`, named
`rum_runtime-562b258c6eef9b02` and `adapter_contract-36e2ac93dcf516de`.
These are corroborating existing-build results, not a fresh authoritative
gate on a candidate implementation. Complete commands, exits, monotonic
elapsed values and combined stdout/stderr are in `logs/*.json`; matching
`.log` files contain the output. There were twelve child invocations: six
test selections and six CLI probes, with no retries. Profiling variables
were removed and `TMPDIR` pointed inside `target/brief-510/tmp`.

Every child used `scripts/run-bounded.py`. Test selections had 3-second
bounds; the earlier corresponding full host and abandonment binaries are
recorded at 0.42 and 0.16 seconds in
`target/brief-506/logs/workspace.log`, and the current timeout selection
completed in the captured 0.27428159000010055 seconds. CLI probes had
2-second bounds, following the same CLI shape recorded in
`target/brief-509/logs/*.json`; its documented calibration was
0.06303024099997856 seconds. Sequential Python command orchestrations used
10-second outer bounds. These durations size watchdogs only; they are not
performance comparisons. No microbench panel ran.

Verbatim ordinary-timeout evidence (`logs/host-timeout.log`):

```text
running 1 test
test tests::eval_contract::host_eval_with_timeout_interrupts_running_eval ... host_eval_with_timeout_interrupts_running_eval elapsed_ms=200
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.21s
```

The existing test asserts all three value/reason/message fields above at
`crates/rum-host/src/tests/eval_contract.rs:325-339`. This verifies the
ordinary shape, **not** the missing timed-resume regression.

Verbatim cancellation and overlap evidence:

```text
test runtime::tests::session_requests::api::accepted_cancel_abandons_input_before_conversion_or_continuation ... ok
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... ok
test runtime::tests::session_requests::api::stale_cancel_cannot_overwrite_current_request_cancel ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 996 filtered out; finished in 0.01s
```

```text
test public_cancel_handle_acquired_while_suspended_abandons_before_input_conversion ... cancelLateSentinel404=Success(value=0, termination=None, messages=0, prints=[], trace=0)
InputString[]=Success(value="queued-value-survives-cancel", termination=None, messages=0, prints=[], trace=0)
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s
```

```text
Do[Null, {1000}]; "done"=Success(value="done", termination=None, messages=0, prints=[], trace=0)
cancelSentinel404=Success(value=0, termination=None, messages=0, prints=[], trace=0)
Length[Range[20000]]=Success(value=20000, termination=None, messages=0, prints=[], trace=0)
cancelSentinel404=Success(value=0, termination=None, messages=0, prints=[], trace=0)
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.10s
```

The last excerpt is from the existing cancellation/constraint-restoration
matrix (`logs/cancel-constraints.log`), not from resuming a timed request.

```text
test every_public_evaluation_entrypoint_rejects_a_pending_continuation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s
```

Verbatim poison evidence (`logs/poison.log`, expected caught panic):

```text
host_boundary_catches_panic_and_poisons_session caught_code=Panic caught_message="host session caught panic: host panic boundary test" followup_code=Poisoned followup_message="host session is poisoned after a previously caught panic"
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.00s
```

The six actual `.rum` probes were passed as a single source to CLI `--eval`
(not `--file`, which would change the request shape). The control
`TimeConstrained[Pause[1/50]; 7, 1/100, "expired"]` returned, verbatim:

```text
"expired"
```

Both `TimeConstrained[Input["deadline: "]; 7, 1/20, "expired"]` and its
`InputString` counterpart returned exit 1. The JSON capture preserves the
trailing prompt spaces; its output field is, verbatim:

```json
  "output": "! input required: deadline: \nerror: input required: deadline: \n"
```

`Dialog[]` returned exit 1 and:

```text
! input required
error: input required
```

`TimeConstrained[7, Input["limit: "]]` returned exit 1; captured output field:

```json
  "output": "! input required: limit: \nerror: input required: limit: \n"
```

`TimeConstrained[Pause[1/50]; 7, 1/100, InputString["fallback: "]]` returned
exit 1; captured output field:

```json
  "output": "! input required: fallback: \nerror: input required: fallback: \n"
```

These expected input exits are successful observations, not failed attempts
being retried. The CLI exits instead of resuming. The REPL's
`evaluate_input` only renders the suspension
(`crates/rum-repl/src/lib.rs:141-150`) and contains no call to
`resume_with_input`; piping another line would start a rejected overlapping
evaluation, not answer the input request. The existing tests found for
`TimeConstrained` cover its limit-argument resume and deadline restoration
on abandonment, not an active body delayed across resume. The old audit
harness is described as deleted in
`docs/dev/codex/brief-413-glm-security-audit.md:164`; its former path is
outside this brief's filesystem scope and was not accessed. No network or
alternative source harness was used to bypass these restrictions.

### 4. Draft follow-up implementation brief (TEMPLATE.md shape)

This is review material only, not authorization to implement during brief-510.
The director must settle the missing characterization and the policy choice
before dispatch. The proposed production change is confined to the host
boundary; broader deadline weaknesses in section 5 are not silently included.

#### Goal

Make `HostSession::eval_with_timeout` retain its already-armed absolute host
deadline across single-source input suspension and every ordinary resume.
Clear its timeout on terminal completion/failure, cancellation completion,
caught panic and reset; preserve it on a rejected overlapping request.
Do not renew the budget when input arrives. Resumed unhandled expiry must
use the existing `TimeConstraint` terminated-success shape and leave the
session reusable. Document that suspended host waiting counts and that
enforcement remains cooperative.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/host-embedding-and-hardening.md` and this report.
- `crates/rum-host/src/session.rs`: timer admission, the six evaluation
  entrypoints, panic guard, builder and reset.
- `crates/rum-runtime/src/runtime/session/session_requests.rs:46-84,150-171,265-359`:
  cancellation-first admission, overlap rejection and continuation lifetime.
- `crates/rum-runtime/src/runtime/session/session_api.rs:193-198` and
  `session_helpers.rs:30-37,84-93`: the two host timer fields and arming.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`:
  separate language-deadline precedent; no production edit is proposed here.
- Existing gates: `crates/rum-host/src/tests/eval_contract.rs:308`,
  `crates/rum-host/tests/adapter_contract.rs:130`, and
  `crates/rum-host/tests/suspension_abandonment_contract.rs:185,485,591`.

#### Scope

Files that the follow-up MAY modify:

- `crates/rum-host/src/session.rs`.
- `crates/rum-host/tests/suspended_timeout_contract.rs` (new, cohesive
  public-host regression suite, automatically discovered by Cargo).
- `crates/rum-stdlib/src/tests/debug_tools_resources.rs` (the language
  constraint characterization/regression below).
- `docs/guide/embedding.md` (the selected host timeout semantics).
- `docs/guide/evaluation.md` (the experimentally confirmed language
  constraint behavior, if promoted to a lasting regression contract).
- The follow-up's own report append and scratch under its assigned target
  directory.

Do not modify kernel production files, cancellation state representation,
`Expr`/`SharedExpr`/`SharedArgs`, builtin catalog/dispatch, dependencies,
`Cargo.lock`, git metadata, `AGENTS.md`, `TODO.md`, or unrelated docs.
If observation requires a larger repair, report it before expanding scope.

`session.rs` is currently 30,142 bytes, in the split-assessment range. Keep
this change together because timeout ownership, panic guard and host
entrypoint admission are the same lifecycle responsibility. Put the new
tests in their own behavior-family file rather than growing it. The later
production split boundary is evaluation/request lifecycle versus
editor/introspection/snapshot services; do not mix that reorganization into
the correctness patch.

#### Deliverable

1. **Characterize before changing production.** Add/run
   `time_constrained_deadline_counts_suspended_input_wait` in the stdlib
   resource tests. Exercise both `Input` and `InputString` in an active
   `TimeConstrained` body, confirm an actual `InputRequired`, wait beyond
   its limit using a monotonic host duration, then resume and require its
   fallback. Pair it with a generously timed immediate-resume control, and
   a subsequent unconstrained request proving restoration. Include a second
   input suspension within one constraint. Record actual values/reasons;
   if the baseline disagrees, stop and return the evidence for a contract
   decision. Do not claim the diagnostic probes above already passed this.
2. **Add explicit private host ownership.** In `HostSession` add a small
   private marker for a timeout installed by `eval_with_timeout`. Initialize
   it in `HostSessionBuilder::build` and clear it in `HostSession::reset`.
   Do not store a second independently advancing deadline or expose a new
   public setting. A marker prevents generic cleanup from erasing timeout
   settings installed by a downstream runtime installer outside this API.
3. **Admit before arming.** In `HostSession::eval_with_timeout`, inside the
   panic boundary, first use the existing
   `finish_cancelled_pending_evaluation_request` and
   `reject_overlapping_evaluation_request`, in that order. Return a completed
   cancellation immediately; reject a live overlap without touching either
   timer field. Only then install `Some(timeout)`, mark ownership, and call
   `eval_request_unprotected`. Remove the unconditional post-call clear.
4. **Centralize evaluation-outcome cleanup.** Add private host helpers such
   as `guard_eval` and `clear_owned_request_timeout`, keeping `guard<T>` as
   the generic panic boundary. Route `eval`, `call`, `eval_script`,
   `eval_file`, `eval_with_timeout` and `resume_with_input` through the shared
   evaluation helper. If this host owns the timer: retain it on
   `InputRequired`; clear it on success/termination and terminal failure;
   preserve it on a failure rejecting another call while the original
   suspension generation is still live. Use the runtime overlap predicate,
   not the nullable input-display marker, for the rejection distinction.
   This catches cancelled-pending completion through any of the six APIs,
   not just resume. Preserve existing return values and I/O accumulation.
5. **Clear on panic without resuming.** In `guard<T>`'s caught-panic branch,
   after recording poison, clear the owned host timeout and marker. Perform
   no guest evaluation. This covers panic in initial evaluation, resume,
   input conversion and another guarded operation while a timeout is owned.
   Reset still rebuilds the session and invalidates old handles.
6. **Document and test the final behavior.** Clarify `eval_with_timeout` and
   `resume_with_input` rustdoc plus the embedding guide: suspension retains
   the armed deadline, waiting consumes it, input does not rearm, overlap
   cannot replace it, and unhandled expiry uses the existing structured
   termination. State cooperative enforcement and avoid claiming the
   unrelated nested-file budget reset has been repaired. Document the
   confirmed `TimeConstrained` precedent in the evaluation guide before
   treating its test as a normative regression.

No new `set_eval_timeout` or timed-resume host API. No request prologue on
resume; no clearing, reserving or replacing a cancellation generation.

#### rum / Rust pitfalls (read twice)

- `InputRequired` is a live request even though the runtime transports it
  in `Err`. Another caller's overlap failure is not that request's failure.
- `set_eval_timeout(Some(same_duration))` also clears the current absolute
  deadline. Do not call it on suspension or ordinary resume.
- A queued response can clear `pending_input_request` while the continuation
  remains live. Test this exact case.
- Accepted cancellation must win before `input.into()` and any continuation
  side effects, including when the deadline is already expired. Keep stale
  handles inert after completion/reset.
- Assert value, structured termination, message tag and recovery, not just
  elapsed duration. Do not use a giant allocation to manufacture elapsed
  time or accept `IterationWorkLimit` as a timeout.
- Use finite, poll-reaching work after a host-side wait. `Pause` alone does
  not guarantee a host-timeout poll. Do not add a tight upper-time assertion
  sensitive to CI scheduling.
- Language `TimeConstrained` may handle the timeout; use an unwrapped loop
  for the host termination regression and a distinct fallback assertion for
  the language constraint test.
- Preserve typed suspension frames, cleanup ordering, safe Rust and the
  existing disabled diagnostic paths. No performance instrumentation is
  needed for this patch.

#### Verification (MANDATORY)

The test that would have caught the finding is
**`eval_with_timeout_survives_input_suspension_and_terminates_resumed_work`**
in `crates/rum-host/tests/suspended_timeout_contract.rs`:

- Start a timed single-source request `Input[]; Do[x = 1, 4096]; 7` with a
  nonzero duration. Assert `InputRequired` before proceeding.
- After receiving the suspension, wait longer than that duration on the
  host using `Duration`/`Instant`, then call plain `resume_with_input("0")`.
- Require the exact terminated-success value/reason/message described above,
  cleared pending input, and successful subsequent untimed poll-reaching
  work. The loop is finite and below an explicitly configured iteration
  allowance. Against today's code it instead completes with `7` and no
  termination, so it is a useful red/green regression, not a speed test.
- Also cover an immediately resumed segment that crosses its deadline while
  running: a safe test-installed signal/wait builtin or a synchronized
  print-sink barrier can let the host wait past expiry after resume has
  actually begun, followed by the same finite loop. Keep the mechanism
  inside this test file and honor the bounded host-payload declarations.

The new test file must additionally cover these distinct lifecycle cases
(separate named tests or a clearly labeled table):

| Test | Required assertion |
| --- | --- |
| `eval_with_timeout_keeps_one_deadline_across_multiple_inputs` | Immediate first resume reaches a second input; waiting past the original deadline then resuming times out. Parameterize direct `InputString`, `Input` whose reply contains another input, `Dialog`, and a suspending cleanup where appropriate. |
| `eval_with_timeout_overlap_preserves_suspended_budget` | Rejected timed and untimed calls neither remove nor replace the old timer. Test both a long attempted replacement (must not extend an expired old budget) and zero replacement (must not prematurely expire a generous old budget). |
| `eval_with_timeout_queued_input_does_not_hide_live_continuation` | Clearing the display marker by queuing input does not clear the budget or admit overlap; the queued response remains available according to the existing contract. |
| `eval_with_timeout_clears_after_resume_success_or_failure` | Fast completion, malformed `Input` response, ordinary terminal failure and timeout each leave later untimed poll-reaching work usable. |
| `eval_with_timeout_cancelled_suspension_preserves_cancel_precedence` | Cancellation before input conversion wins even after deadline expiry; no primary continuation side effect, scope restoration and existing cleanup behavior hold, and the stale handle cannot affect the next request. |
| `eval_with_timeout_cancelled_pending_completion_clears_budget_at_every_entrypoint` | Complete cancellation through each of `eval`, `call`, `eval_script`, `eval_file`, `eval_with_timeout` and resume; no budget leaks to a later request. Use a non-accessed dummy file path for the cancellation-before-file-read case. |
| `eval_with_timeout_panic_and_reset_preserve_poison_contract` | Initial/resumed panic returns `Panic`, later guest requests return `Poisoned`, reset restores use and invalidates the old handle. Include a panic on host input conversion inside resume. |
| `eval_with_timeout_preserves_existing_precancel_and_untimed_behavior` | A pre-entry cancel survives admission; untimed suspend/resume still works; no timer becomes a persistent default for later requests. |

Run, under `scripts/run-bounded.py`, in this order:

```text
cargo test --offline -p rum-stdlib time_constrained_deadline_counts_suspended_input_wait -- --nocapture --test-threads=1
cargo test --offline -p rum-host --test suspended_timeout_contract -- --nocapture --test-threads=1
cargo test --offline -p rum-host --test adapter_contract
cargo test --offline -p rum-host --test suspension_abandonment_contract
cargo test --offline -p rum-runtime
cargo fmt --check
cargo test --offline --workspace --no-fail-fast
```

Measure the first focused characterization before sizing its later runs.
Derive build/full-suite bounds from the latest equivalent recorded gate;
the existing `target/brief-506/logs/workspace.log` records a 32.93-second
build and 124.96 seconds summed over its 32 test-result duration rows, useful
as a conservative sizing reference, not a claim that its full workspace
gate was green. It contains unrelated native-adapter failures. Keep all
new artifacts and temporary test files inside the follow-up's target
directory, including by setting `TMPDIR`; use offline Cargo. Paste actual
red/green regression output and final suite output. Report any native
skip/failure honestly.

**No performance A/B is required for this proposed patch.** It changes host
request admission/finalization and tests/docs, not evaluator steps,
dispatch, matching, expression ownership or candidate lookup. Run no
microbench panel for brief-510. If the follow-up expands into those hot
paths, stop for a newly scoped brief with the repository A/B requirement.

#### Hard rules

- Modify only the follow-up scope above; no drive-by changes or weakened
  contracts. Its new tests/code are not authorized during this design brief.
- No git commit, branch, switch, merge, push, stash, checkout, restore, index
  edits or git configuration changes. Leave the implementation uncommitted
  for the director's gate.
- No network or writes outside the repository. Use offline Cargo and bound
  long/child-spawning commands with measured bounds.
- No `unsafe`. No third-party engine/oracle comparisons or artifacts.
- Preserve the cancellation generation protocol and typed continuation
  shapes. No new cost per evaluator step or diagnostic counters.
- Two failed attempts with no additional information on the second trigger
  stop-loss. Report actual observations, including unavailable verification.

#### Report format

1. Per-file changes and scope/split assessment.
2. Verbatim baseline characterization, red/green host regression, and named
   gate output; identify any skipped or failing checks.
3. Final deadline policy, outcome shape and cancellation/poison invariants.
4. Risks, unresolved observations and any expansion requiring director scope.

### 5. Severity and director decisions

**Keep the finding MEDIUM on the evidence available.** It is a real host
resource-control bypass whenever a host allows interactive suspension and
resumes a timed request. Even the first resumed segment loses the timeout;
the problem is not limited to repeated-prompt amplification. The public API
cannot simply rearm it, and a one-line conditional clear would leave
completion, overlap and poison holes.

Factors limiting the scope:

- `HostConfig::sandboxed()` is `{Pure}` and does not permit `Input` or
  `InputString` (`docs/guide/resource-limits-and-sandbox.md:262-267`). The
  affected multi-tenant example therefore means a sandbox extended to allow
  `Interactive` or an equivalent downstream suspension path. This is not a
  demonstrated bypass of the default pure capability policy.
- Qualified cancellation, other resource limits and an external watchdog
  remain available. Preserving the deadline does not turn cooperative checks
  into preemption. The observed panic cleanup hole leaves a poisoned session
  rejected until reset rather than silently allowing another guest request.
- The request already promises completion/failure cleanup, and the runtime
  already retains the host deadline if the facade does not clear it. The
  primary repair can stay outside evaluation hot paths.

**Additional source finding affecting any stronger absolute guarantee:**
`Get` calls `Session::eval_file`
(`crates/rum-runtime/src/runtime/builtins_context_loader.rs:282`), which
evaluates script forms (`session/session_eval.rs:378,309-324`). Each form
calls the prologue at `session_eval.rs:215`, which unconditionally recomputes
`eval_deadline` at `session_helpers.rs:32`. Thus even with suspension fixed,
nested file evaluation can renew a host budget. This is a separate,
source-established reinitialization path, not a measured overrun in this
report, and it is absent from a `{Pure, Interactive}` host without file
capability. Do not advertise the small host repair as a proof that the
original `Instant` survives every possible nested loader operation. If the
director requires that stronger whole-request invariant now, the draft's
host-only production scope must be expanded with an explicit runtime
request-lifetime design and its own characterization.

Director decisions/open evidence:

1. Accept or reject the recommendation of an absolute deadline across
   suspension, explicitly counting host waiting. The current resume source
   makes this smaller than the per-segment alternative, contrary to the
   brief's lazy-recompute premise.
2. Supply an existing allowed local resume probe, or authorize the proposed
   characterization test in the follow-up scope. The required empirical
   `TimeConstrained` delayed-resume precedent and the new resumed host
   timeout regression were **not** run here. I did not repeat the audit's
   overrun numbers as new measurements or infer a probe result from source.
3. Keep nested file-form deadline renewal as a separately diagnosed issue,
   or expand the next design before promising an absolute deadline across
   all nested evaluation. Neither a new host setter nor a timed-resume
   overload is necessary for the primary suspension repair.

No stop-loss was triggered: each executed probe produced the intended
observation, and the missing resume probe is a scope/tool-surface limitation,
not a twice-retried run. No source, script, fixture or other documentation
was changed; only this report was appended.
