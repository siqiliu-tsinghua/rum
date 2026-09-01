# brief-398 — design: request-scoped cancellation at the host boundary

**Scope: NONE — design and plan only, no source change.** Reading, grepping and
throwaway probes are in scope; revert any temporary instrumentation before
reporting and name it. Plan-first is the standing rule for cross-cutting work,
and this is cross-cutting: it touches the runtime's interrupt flag, the host's
public cancel handle, a documented contract test, and every embedder.

**Write your findings and design first, before running any verification
commands.**

## The defect, already established

brief-397 diagnosed it and I verified the mechanism from source myself. Three
facts, all confirmed:

- `crates/rum-jupyter/src/adapter/worker.rs:87-88` — `spawn_eval` obtains
  `session.cancel_handle()` *before* it spawns the worker thread, so the handle
  is live and callable the instant the worker struct exists.
- `crates/rum-runtime/src/runtime/session/session_requests.rs:112` —
  `eval_request`'s first statement is `begin_top_level_interrupt_budget()`.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:31` — that
  function's first line is `self.interrupt_flag.store(false, Ordering::Relaxed)`.

So a cancel that lands after the handle exists but before the worker's
evaluation prologue runs is **erased**, and nothing re-delivers it. brief-397
made this switchable in both directions with forced ordering (5/5 failures with
the worker delayed, 5/5 passes with the parent delayed), and traced
`begin_top_level_interrupt_budget clears prior=true` on the failing side.

The user-visible consequence is not a test artifact: a Jupyter frontend can
send `interrupt_request`, receive an acknowledgement, and the accepted execute
runs to completion anyway.

## The ruling — this is decided, do not re-open it

brief-397 proposed repairing this inside the `rum-jupyter` adapter, by latching
`interrupted` and reasserting the cancel handle on each completion poll.
**That is rejected: it fixes at the wrong layer.** The mechanism lives in
`rum-runtime` (an unconditional clear) and `rum-host` (a handle that is live
before the evaluation it targets has started). Any embedder that cancels from
another thread inside that window loses the cancel; `rum-jupyter` is merely the
first consumer to reach it. Patching one adapter leaves the hole open for every
other embedder of a project whose stated product is an embeddable, sandbox-first
host contract.

The user's ruling (2026-09-01) invoked the project's two standing principles
directly: **pick the best and most correct option, cost arguments do not count**,
and **a fix the compiler or the type system enforces beats one that relies on
people remembering**. Design accordingly. Do not return a variant of the
adapter latch, and do not argue for it on effort grounds.

## The real constraint you must satisfy

The clear is deliberate, not an oversight. `rum-host`'s contract test requires
that a cancel issued *before* an evaluation is a **no-op for a later
evaluation** (`crates/rum-host/tests/lifecycle_contract.rs:107-117`), so a stale
cancel from a finished request cannot kill an unrelated later one. That intent
must survive.

So the design problem is exactly: **distinguish a stale cancel from a cancel
aimed at the request that is starting right now.** A cancel that names no
request cannot be told apart from one that names a finished one, which is why
the current code can only choose between two wrong answers. The obvious shape
is to give cancellation a request identity — a generation, epoch, or token
minted when a request is accepted and carried by the handle — but the shape is
yours to design and justify; I am naming the problem, not the solution.

Whatever you choose, state precisely:

1. **Where the identity is minted, and by whom.** The window opens when the
   embedder takes a handle for a request that has not yet begun evaluating, so
   the identity has to exist before `eval_request` runs. Say what creates it and
   what happens to a handle taken outside any request.
2. **What the new documented contract sentence is**, replacing the current
   "a cancel before an evaluation is a no-op". It must still make a stale cancel
   inert, and it must make a cancel for an accepted-but-unstarted request
   effective. Write the sentence.
3. **How it is enforced structurally rather than by convention.** A rule that a
   future caller must remember is not acceptable here; prefer a typed handle
   that cannot be constructed without an identity, in the spirit of the
   `EvaluatedPartSelectors` and `TrustedSystemDownValueLhs` repairs.
4. **What `interrupt_armed` does.** It is computed in the same prologue
   (`session_helpers.rs:31-39`) from the deadline and the flag's strong count.
   Say whether the arming decision has the same window, and whether the elapsed
   timeout shares the defect.

## What to inventory

- **Every construction and use of a cancel handle**, across `rum-runtime`,
  `rum-host`, `rum-jupyter`, `rum-cli`, `rum-repl` and the tests, with
  `path:line` and a one-line role. Include `spawn_resume`
  (`crates/rum-jupyter/src/adapter/worker.rs:98-105`), which brief-397 reports
  has the identical startup window, and say whether resume needs the same
  identity or a different one.
- **Every test that synchronizes a cross-thread cancel with a sleep or a print.**
  `TODO.md` already records four sites the consistency review flagged as this
  exact shape and never observed failing
  (`crates/rum-host/tests/lifecycle_contract.rs:118`,
  `crates/rum-host/src/tests/eval_contract.rs:210`,
  `crates/rum-host/tests/stress.rs:52`,
  `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:502`).
  For each, say whether the new design lets it become deterministic by
  construction, and whether it was passing for the right reason before.
- **Every place the interrupt flag is stored, cleared, or read**, so the plan
  can state which of them the identity has to reach.

## Required properties of the repair you plan

- A cancel for the current request is never lost, whether it arrives before the
  evaluation prologue, during evaluation, or between a suspension and its
  resume.
- A cancel from a finished or unrelated request never affects a later one.
- No new unconditional cost on the evaluation hot path. The prologue runs on
  every top-level request; whatever you add there must be a couple of stores,
  not an allocation or a lock. If the design cannot avoid hot-path cost, say so
  and quantify it — that becomes a microbench-gated question.
- No new `unsafe`.
- The determinism test must hold **by construction** — a worker-start gate,
  barrier, or channel handshake — never a sleep. brief-397's own repro used a
  100 ms sleep as a probe; that is a probe, not an acceptable test.

## Deliverable

- The design, with the four numbered answers above.
- The three inventories.
- A step-by-step plan, one step per independently gateable change, each naming
  files, functions, its gate test, and what it must not break. Say explicitly
  which step first makes the failing ordering deterministic, because that test
  is the acceptance evidence for the whole line.
- **Drafts of the follow-up implementation briefs**, in `TEMPLATE.md` shape,
  one per step — full drafts, not step descriptions.
- Risks, open questions, and any correction to the background above.

## Explicitly out of scope

- Finding A from brief-397 (the immediate-completion assertion in
  `tests/pump.rs:37`). It is independent and gets its own brief; do not repair
  or plan it here.
- The `rum-jupyter` native-ZeroMQ skip mechanism.
- Any change to the `Do` driver or the evaluator's amortized poll path.
  brief-397 established the absence of a `Do`-driver poll is not what loses this
  request.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- **No source change.** Temporary instrumentation only if reverted before
  reporting and named; verify with `git status` and paste it.
- Do not modify `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` outside this
  file, or anything under `docs/dev/codex/` except appending your report.
- Leave no probe files in the repository.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Paste verbatim output for every claim; if a run did not produce the evidence,
  say so.

## Report format

1. The design, answering the four numbered questions.
2. The three inventories.
3. The plan, step by step.
4. The drafted implementation briefs.
5. Stop-loss, risks, open questions, and corrections to my background.

---

## Report — 2026-09-01

### 1. Design

#### Decision

Use a request ticket plus a generation-qualified cancel handle, backed by one
session-owned atomic state word.  Do not keep a session-scoped cancel handle and
do not add an adapter latch.

The runtime state is conceptually:

```rust
struct RequestInterruptState {
    // 0 is idle; otherwise (generation << 1) | cancelled_bit.
    phase: AtomicU64,
}

struct AcceptedRequest {
    generation: NonZeroU64,
    state: Arc<RequestInterruptState>,
    // private, non-Clone, consumed by exactly one accepted entrypoint
}

struct RequestCancelHandle {
    generation: NonZeroU64,
    state: Arc<RequestInterruptState>,
    // private fields; Clone + Send + Sync
}
```

The names are recommendations, not a requirement; the state machine is the
requirement.  `cancel()` performs a compare/exchange from this handle's exact
`(generation, not-cancelled)` state to `(generation, cancelled)`.  It is
idempotent for the already-cancelled state and a no-op for idle or any other
generation.  Completing or abandoning the request compare/exchanges either of
that generation's states back to idle.  Thus an old handle cannot write a bare
boolean over a newer request's cancellation.  Generation exhaustion must not
reuse an identity: when the private 63-bit counter is exhausted while idle,
replace the session's `Arc<RequestInterruptState>` and restart its counter;
old handles then address the retired control block.

Completion must be a linearization point, not a load followed by an unrelated
clear.  The finishing request atomically transitions its own uncancelled or
cancelled state to idle and learns which state won.  If cancellation won before
that transition, ordinary finalization returns the normal `Cancelled`
termination even if the last amortized evaluator poll already ran.  If
completion wins first, the later handle CAS sees idle and is correctly a stale
no-op.  This closes the smaller cancel-versus-final-return window as well as the
reported startup window.

The runtime keeps the existing unarmed synchronous path for callers that do not
request a cancel capability.  The new accepted-request path installs the
non-cloneable ticket before calling `begin_top_level_interrupt_budget`; a normal
completion consumes it, an input suspension retains it in the session's
pending-request state, and the final resume consumes it.  A cancellation that
is already set when a request tries to publish `InputRequired` wins rather than
publishing a prompt; a cancellation set after suspension remains latched and is
checked before a resume consumes input or re-enters the continuation.

At the host boundary, replace `HostSession::cancel_handle()` with an explicit
accepted-request API of this shape:

```rust
let accepted = session.accept_cancellable_request()?;
let cancel = accepted.cancel_handle();
let outcome = session.eval_accepted(accepted, request);
```

`HostAcceptedRequest` has private fields, is non-`Clone`, and is consumed by one
of the accepted guest-code entrypoints (`eval_accepted`,
`eval_script_accepted`, `eval_file_accepted`, or `call_accepted`).  It contains
the runtime ticket; `HostCancelHandle` contains only the matching qualified
runtime handle.  Each accepted entrypoint validates that the ticket's control
block is the one owned by that `HostSession`, so moving a ticket to another
session fails closed instead of redirecting cancellation.  Dropping an
unexecuted ticket abandons its identity and makes all of its handles inert.
Ordinary `eval`/script/file/call remain available but cannot manufacture a
cancel handle.

The accepted ticket is payload-independent so the CLI can accept a request,
install a print sink capturing that request's handle, and then consume the
ticket through the appropriate source/script/file entrypoint.  This is still a
real accepted request: only one ticket can reserve a session at a time, the
ticket is linear, and dropping it closes that reservation.

For input suspension, the identity belongs to the entire logical evaluation,
not to an individual worker segment.  The initial accepted Eval ticket moves
into the runtime pending request on `InputRequired`.  The embedder retains its
handle.  `resume_with_input` continues that stored identity and does not mint a
new one.  In Jupyter, `ExecuteContext` (or a dedicated pending-execute control
object) must retain the same handle while waiting for stdin and pass a clone to
`spawn_resume`; `spawn_resume` must stop calling `session.cancel_handle()`.

#### The four required answers

1. **Where identity is minted.** `HostSession::accept_cancellable_request`, by
   way of a runtime `Session::accept_cancellable_request`, reserves the next
   nonzero generation before the worker is spawned and returns the linear
   accepted ticket from which the handle is derived.  The session, not the
   adapter and not the worker prologue, is the authority.  There is no handle
   outside an accepted request after the old `HostSession::cancel_handle()` and
   raw `Session::interrupt_handle()` APIs are removed.  Dropping an accepted
   ticket without executing it closes the generation; dropping or invoking its
   handle later cannot target another request.

2. **Replacement contract sentence.** The exact proposed normative sentence
   is: **“A `HostCancelHandle` affects only the accepted evaluation request that
   produced it: cancellation is effective from acceptance, through every
   suspension and resume of that request, until its completion, and calls made
   after that request completes never affect another request.”**  The API
   remains cooperative: while a request is suspended and no runtime thread is
   executing, the bit is latched; the next host re-entry observes it before
   consuming input.

3. **Structural enforcement.** The handle's fields and constructor stay
   private and include the identity.  The only public constructor path also
   returns a private-field, non-cloneable accepted ticket; accepted evaluation
   consumes that ticket.  The handle CAS names that ticket's generation, and
   the runtime validates the ticket's `Arc` identity against the session.  No
   caller can construct an unqualified handle, reuse a ticket, retarget a
   handle, or make a stale generation store over the current generation.  This
   is enforced by types plus the atomic state transition, not by a comment that
   an adapter must remember to reassert cancellation.

4. **`interrupt_armed` and timeout.** Today the prologue clears the flag and
   then derives arming from `eval_deadline` or `Arc::strong_count` at
   `session_helpers.rs:31-38`; `set_eval_timeout` recomputes the same shape at
   `session_api.rs:140-144`.  In the reported Jupyter ordering the worker owns a
   handle throughout startup, so arming remains true even though the cancel bit
   is erased.  In the general cancel-then-drop ordering, both the bit and the
   strong-count evidence can disappear.  The repair makes arming
   `deadline.is_some() || active_accepted_request.is_some()`.  The accepted
   scope is installed before the prologue and remains present across input
   suspension, so arming has no startup/drop race.  The evaluator's existing
   `if interrupt_armed` sites stay unchanged; the cold poll compares the atomic
   state with the active generation.  Elapsed timeout does **not** share this
   defect: its deadline is session-local and is first created in the prologue,
   not delivered by another thread and then cleared.  Preserve the present
   timeout start point and OR it into arming; do not give timeout a cancellation
   generation.

#### Cost and safety

The ordinary non-cancellable path gains no allocation, lock, Arc clone, or
atomic operation.  Its prologue replaces the current unconditional
`AtomicBool::store(false)` plus `Arc::strong_count` with ordinary checks of the
deadline and the optional active accepted scope.  Accepting a cancellable
request performs a generation increment, one atomic reservation, and Arc
clones for the ticket/handle; those are on the opt-in host control path before
evaluation.  An armed cancellation poll performs one atomic load/compare, as
the current flag poll does.  No `unsafe` is needed.  Because the evaluator
prologue and cold poll are still touched, both implementation steps below carry
the required rum-vs-rum microbench A/B gate rather than assuming the cost is
zero.

### 2. Inventories

#### A. Every cancel-handle construction and use

Production and public surface:

| Site | Current role | Required disposition |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/model/session.rs:112-115` | Stores the session-wide `Arc<AtomicBool>` and poll counter/deadline state. | Replace the boolean with the shared qualified-request control state plus optional active accepted identity. |
| `crates/rum-runtime/src/runtime/session/session_api.rs:136-138` | `Session::interrupt_handle` clones the unqualified flag. | Remove after the additive migration lane; only an accepted runtime ticket may yield a qualified handle. |
| `crates/rum-host/src/session.rs:113-126` | Defines `HostCancelHandle`; `cancel` stores bare `true`. | Keep the public handle name, but wrap the qualified runtime handle and delegate its exact-generation CAS. |
| `crates/rum-host/src/session.rs:164-176` | Reset rustdoc and implementation promise a fresh flag invalidates old handles. | Update wording to request identity; rebuilding the runtime control block still invalidates every old handle. |
| `crates/rum-host/src/session.rs:440-447` | `HostSession::cancel_handle` constructs a handle at arbitrary session lifetime. | Remove; replace with accepted ticket creation and `HostAcceptedRequest::cancel_handle`. |
| `crates/rum-host/src/lib.rs:54-69` | Re-exports request/session facade types and `HostCancelHandle`. | Re-export the accepted ticket; retain `HostCancelHandle`, with its new request-qualified meaning. |
| `crates/rum-jupyter/src/adapter/worker.rs:7-9,26-30` | Imports and stores a handle beside each worker. | Store the initial accepted request's qualified handle. |
| `crates/rum-jupyter/src/adapter/worker.rs:47-53` | `ExecuteState::cancel` marks `interrupted` and invokes the handle. | Keep one immediate call; no reassertion loop.  The generation latch makes it durable. |
| `crates/rum-jupyter/src/adapter/worker.rs:87-95` | `spawn_eval` takes a session handle before spawning the Eval worker. | Accept a cancellable request before spawn, put its ticket in `ExecuteSegment::Eval`, and retain its handle in logical execute context. |
| `crates/rum-jupyter/src/adapter/worker.rs:98-105` | `spawn_resume` takes a new session handle before spawning Resume. | Do not mint anything: use the initial logical execute's handle and the runtime's stored pending identity. |
| `crates/rum-jupyter/src/adapter/mod.rs:154-160,360-363` | Control dispatch acknowledges interrupt and routes it to `ExecuteState::cancel`. | Also locate the retained pending-execute handle while awaiting input; keep Control non-blocking. |
| `crates/rum-cli/src/main.rs:246-253,327-343` | Each CLI request installs a streaming print sink; the sink's pre-request session handle cancels after EPIPE. | Accept one ticket first, capture its handle in the sink, and consume that ticket in the selected eval/script/file call. |
| `crates/rum-repl/**` | The repository search finds no cancel-handle construction or use. | No production migration; workspace compilation is the gate. |

Direct tests and compile-time uses:

| Site | Current role | Migration |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:512-544` | Takes `interrupt_handle`, clones it into a canceller, and stores `true` after a test builtin signals evaluation start. | Use an accepted runtime ticket/qualified handle; retain the post-start handshake as the during-evaluation case. |
| `crates/rum-host/src/tests/eval_contract.rs:212-278` | Constructs and clones a host handle; cancels after a print handshake. | Use the accepted ticket and its handle; retain the timing/termination/session-reuse assertions. |
| `crates/rum-host/src/tests/eval_contract.rs:322-340` | Keeps an unused handle alive while exercising a generous timeout. | Recast as an accepted-but-not-cancelled request plus timeout, proving the OR arming path. |
| `crates/rum-host/src/tests/session_lifecycle.rs:7-12` | Compile-time `Send` assertion for `HostCancelHandle`. | Add the accepted ticket's intended `Send` assertion if it is part of worker transfer; retain handle `Send + Sync`. |
| `crates/rum-host/tests/lifecycle_contract.rs:107-117` | Requires a bare cancel before each later eval to be cleared. | Replace with two new contract halves: accepted-but-unstarted cancel is effective; a completed request's handle is inert for a separately accepted later request. |
| `crates/rum-host/tests/lifecycle_contract.rs:119-164` | Cancels a running eval after a print/channel handshake and proves recovery. | Migrate to accepted ticket/handle without weakening the running and recovery assertions. |
| `crates/rum-host/tests/lifecycle_contract.rs:166-179` | Calls a pre-reset handle after rebuilding the session. | Retain with a handle from a real accepted/completed request; prove the old control block cannot affect the rebuilt session. |
| `crates/rum-host/tests/lifecycle_contract.rs:320-338` | Asserts `HostSession: Send` and handle `Send + Sync`, then moves a session through a worker. | Retain and add the accepted ticket trait/worker transfer expected by the public API. |
| `crates/rum-host/tests/stress.rs:54-98` | Cancels a rewrite-heavy running eval after a print handshake. | Migrate to the accepted ticket while keeping load, latency, and recovery coverage. |
| `crates/rum-host/tests/stress.rs:120-145` | Ignored ten-cycle test sleeps 25 ms before each cancel. | Remove the sleep ordering; use a per-cycle acknowledged start channel or a deliberately pre-start accepted cancel, according to the property named by the test. |

The Jupyter protocol tests do not construct `HostCancelHandle` directly, but
they exercise its complete use chain: `tests/pump.rs:90,243`,
`tests/adapter/dispatch.rs:341`, `tests/bootstrap.rs:526`, and
`zeromq/tests/native/frontend/control.rs:39` construct or dispatch
`KernelInterruptRequest`.  The deterministic startup acceptance test belongs in
the in-memory pump/adapter layer; the native transport test remains only a
transport gate.

#### B. Cross-thread cancel tests synchronized by sleep or print

| Test/site | Current synchronization | Was it passing for the right reason? | Deterministic construction under the new design |
| --- | --- | --- | --- |
| `crates/rum-host/tests/lifecycle_contract.rs:120-161` | `Print["evaluation-started"]` reaches an exclusive sink, which sends on a zero-capacity channel; the canceller receives before calling `cancel`. | **Yes for its stated running-evaluation property.** The print executes after the top-level prologue, so it deliberately excludes the lost pre-start ordering. It is not evidence for accepted-but-unstarted cancellation. | Keep this as the during-evaluation case; add a separate accepted ticket, cancel it before calling `eval_accepted`, and assert cancellation with no sleep. |
| `crates/rum-host/src/tests/eval_contract.rs:213-267` | Same print plus zero-capacity channel, with a cloned handle. | **Yes for running cancellation**, for the same reason; it duplicates a crate-local layer of the contract. | Migrate the handle to the ticket and retain the channel. The new pre-start unit test is simply `accept -> cancel -> eval_accepted`. |
| `crates/rum-host/tests/stress.rs:55-97` | Same print/channel handshake inside the rewrite-heavy workload. | **Yes for cancellation under load.** It proves the rewrite path observes a post-prologue cancel, not startup durability. | Retain it as the stress layer with a qualified handle. No fixed duration is needed. |
| `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:512-544` | `TestEvaluationStarted[]` calls `push_print`; the sink acknowledges builtin dispatch through a zero-capacity channel before the raw flag store. | **Yes for the file-request during-evaluation path.** It cannot expose the startup clear because dispatch is necessarily after it. | Migrate to the scoped runtime API; add a separate accepted-before-start file/eval case that cancels before entering the request. |
| `crates/rum-host/tests/stress.rs:124-144` | The ignored stability loop sleeps 25 ms, then cancels. | **No.** It normally passes only when evaluation crosses its prologue before the duration expires and can fail if scheduling reverses that order. | Replace the duration with a per-cycle print/channel acknowledgement for running-cancel stability, or explicitly cancel the accepted ticket before execution if the desired property is pre-start durability. Do not keep both meanings in one test. |

`TODO.md:701-707` is stale relative to the current source: it still says the
four named tests start sleeping cancellers, but those four now use the
print/channel or test-builtin/channel handshakes above.  The ignored stress loop
is the remaining sleep-based cancel test found by the repository search.

#### C. Interrupt flag, clear, read, and arming inventory

Direct flag lifecycle:

| Site | Operation | Identity reach in the repair |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/session/session_state.rs:632,660-663` | Initializes `interrupt_armed = false`, `Arc<AtomicBool>(false)`, deadline, and poll counter. | Initialize idle qualified control state and generation counter; keep the cheap armed/counter/deadline fields. |
| `crates/rum-host/src/session.rs:122-126` | `HostCancelHandle::cancel` stores `true`. | Exact-generation compare/exchange only. |
| `crates/rum-runtime/src/runtime/session/session_helpers.rs:31-38` | Clears the flag, resets counters/budgets, creates deadline, computes arming from deadline or Arc strong count. | Never clear scoped cancellation. Derive arming from deadline or active accepted identity. |
| `crates/rum-runtime/src/runtime/eval_core/mod.rs:84-97` | Every 1024 armed polls, loads the boolean, then checks `Instant` deadline. | Compare the control word with `(active_generation, cancelled)`; keep deadline check independent. |
| `crates/rum-runtime/src/runtime/session/session_api.rs:136-144` | Clones the raw handle; timeout mutation recomputes armed from timeout or strong count. | Remove raw clone; timeout recomputes from timeout or active accepted identity. |
| `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:529-537` | Test obtains raw Arc and stores `true`. | Qualified test handle. |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:699-719` | Test manually arms, positions counter, and stores `true` before a direct association mutation. | Use a test-only scoped request helper/qualified handle; do not expose the control word. |
| `crates/rum-runtime/src/runtime/tests/part_assignment.rs:380-400` | Same direct setup for positional Part mutation. | Same migration. |

Every `interrupt_armed` consumer is already centralized through
`poll_host_interrupt_if_due`: evaluator entry at `eval_core/mod.rs:16-24`,
association direct update at `kernel_data/association_lvalue.rs:46-48`, direct
Part assignment at `definitions/part_assignment.rs:171-173`, incremental append
at `definitions/incremental_append.rs:40-42`, and the `While`, `Until`, and `For`
drivers at `kernel_data/control_data/branching_loops.rs:150-152,237-239,297-299`.
Those call sites do not carry identities themselves; the identity lives in the
session's active accepted scope and is consumed by the single poll helper.

The only writers of `interrupt_armed` are initialization
(`session_state.rs:632`), timeout mutation (`session_api.rs:140-144`), and the
top-level prologue (`session_helpers.rs:31-38`), plus the two direct unit-test
setups above.  No other hidden flag store/read was found in the named crates.

### 3. Step-by-step plan

#### Step 1 — add the qualified runtime request state alongside the legacy lane

- Touch `crates/rum-runtime/src/runtime/model/requests.rs` (or a focused new
  sibling module), `model/session.rs`, `session/session_state.rs`,
  `session/session_api.rs`, `session/session_helpers.rs`,
  `session/session_requests.rs`, `session/session_eval.rs`, and
  `eval_core/mod.rs` to add the private generation/control-word state,
  non-cloneable accepted ticket, cloneable qualified runtime handle, atomic
  cancel/completion linearization, and accepted variants of eval/expr/script/
  file request entrypoints.  Retain the old boolean API temporarily so the step
  compiles without changing `rum-host` consumers.
- Carry the accepted scope through `PendingEvalRequest`/resume.  Do not mint a
  resume generation.  Check a latched cancel before publishing input and before
  consuming resume input.  A scoped script/file request retains one identity
  across forms and stops evaluating later forms once cancellation terminates
  the request.
- Add deterministic runtime tests in
  `runtime/tests/session_requests/api.rs`: accepted cancel before entry is
  effective; completed/stale handle is inert for a later accepted and ordinary
  request; a stale handle cannot overwrite a current cancel; cancellation
  survives input suspension and wins before resume input is consumed.  Update
  the two direct poll-before-mutation tests only if the additive implementation
  removes their private setup path in this step.
- Gate with focused runtime tests, `cargo test -p rum-runtime`, workspace tests,
  and a base/candidate `core-language-microbench.sh` A/B.  Inspect every row and
  paste at least `literal-integer`, `literal-string`,
  `symbol-ownvalue-read`, and `list-literal-small`, the closest short default
  path neighbors.
- Must not change the existing public host contract yet, allocate or lock on an
  ordinary request, add `unsafe`, change `Do`, or make the temporary legacy path
  behave as request-scoped by convention.

This step is independently gateable and gives the lower layer a deterministic
`accept -> cancel -> enter` regression.  It does not yet repair Jupyter because
Jupyter still uses the legacy handle.

#### Step 2 — make the typed host contract authoritative and migrate every consumer

- Touch `crates/rum-host/src/model/requests.rs`, `model/mod.rs`, `session.rs`,
  and `lib.rs` to publish the linear `HostAcceptedRequest`, accepted guest-code
  entrypoints, and generation-qualified `HostCancelHandle`; remove
  `HostSession::cancel_handle`.  Update the host rustdoc contract sentence and
  preserve reset, poison, timeout, and input continuation behavior.
- Migrate all host cancellation tests named in inventories A/B.  Replace
  `cancel_handle_before_or_after_eval_is_a_noop_for_later_evals` with explicit
  accepted-before-start and stale-after-completion halves.  Remove the ignored
  stress test's 25 ms ordering.  Add suspension/resume and stale-versus-current
  ordering tests.
- Touch `crates/rum-cli/src/main.rs` so `eval_request` accepts one request ticket
  before installing the EPIPE print sink, captures only that ticket's handle,
  and consumes the same ticket through source/script/file evaluation.  Preserve
  successful broken-pipe exit and all rendering/exit-code behavior.
- Touch `crates/rum-jupyter/src/adapter/worker.rs` and `adapter/mod.rs` so Eval
  accepts before spawn, the logical execute retains its handle across
  `InputRequired`, Resume reuses it, and Control can latch cancellation while a
  request is running or pending input.  Do not add reassertion.
- Add an instance-scoped, `#[cfg(test)]` channel/barrier worker-start gate and
  use it in `tests/pump.rs` to force `cancel-before-eval-prologue`; this is the
  **first step that makes the actual brief-397 failing Jupyter ordering
  deterministic, and it is the acceptance evidence for the whole line**.  Add
  the corresponding `tests/stdin.rs` Resume-start test using the same logical
  handle, plus a cancel-while-suspended-before-resume assertion.  Preserve all
  current Control reply, routing, deferral, lifecycle, cancellation outcome,
  and follow-up-session assertions.  Finding A at `tests/pump.rs:37` remains out
  of scope.
- Remove the legacy runtime `Arc<AtomicBool>`, raw `interrupt_handle`,
  strong-count arming, and compatibility branch; migrate direct runtime tests.
  Search the named crates to prove no unqualified construction remains.
- Update `docs/guide/embedding.md`,
  `docs/guide/resource-limits-and-sandbox.md`,
  `docs/host-embedding-and-hardening.md`, and
  `docs/runtime-evaluation-audit.md`, plus the stale `TODO.md:701-707` entry, in
  the same contract change.  `rum-repl` should require no source edit; prove it
  with the workspace gate.
- Gate focused runtime/host/Jupyter/CLI tests, 30 capped release repetitions of
  each deterministic Eval and Resume startup test, `cargo test --workspace`,
  and a Step-1/candidate full microbench A/B with the same named rows and all-row
  regression review.
- Must not block the Control reply, allocate/lock on ordinary eval, add an
  evaluator/`Do` poll, reassert cancellation from adapter polling, conflate
  timeout with request identity, or weaken post-cancel usability.

This is one atomic public-API migration because removing the session-scoped
constructor and moving every embedder to the typed ticket must compile and land
together.  Splitting removal from consumer migration would deliberately leave
an unqualified escape hatch, contrary to the ruling.

### 4. Drafted follow-up implementation briefs

#### Draft 1 — add runtime-qualified request cancellation

##### Goal

Add an opt-in runtime cancellation lane whose handle is bound to a unique,
accepted request identity before evaluation starts.  Cancellation must survive
the gap before the top-level prologue, remain attached across input suspension
and resume, linearize correctly against request completion, and become inert
after completion.  This step is additive: the existing raw boolean lane remains
temporarily for unchanged host consumers, and ordinary non-cancellable requests
gain no allocation, lock, or new hot-path atomic work.

##### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/host-embedding-and-hardening.md`, and brief 398's accepted design before
  editing.
- `crates/rum-runtime/src/runtime/model/session.rs:77-115` owns transient
  evaluation/interrupt state.
- `crates/rum-runtime/src/runtime/model/requests.rs:221-227` owns pending eval
  request state.
- `crates/rum-runtime/src/runtime/session/session_api.rs:136-144` owns the
  current raw interrupt handle and timeout-side arming.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:31-39` owns the
  top-level interrupt/deadline prologue.
- `crates/rum-runtime/src/runtime/session/session_requests.rs:108-249` owns
  eval, input suspension, and resume request lifetimes.
- `crates/rum-runtime/src/runtime/session/session_eval.rs:195-375` owns legacy
  text/script/file top-level forms and their per-form budget setup.
- `crates/rum-runtime/src/runtime/eval_core/mod.rs:13-97` owns amortized polling.
- `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:512-544` is the
  current runtime cross-thread cancellation contract test.
- Acceptance is mechanical: the deterministic identity tests, runtime and
  workspace suites, and the named microbench rows decide.

##### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model/requests.rs`
  - `crates/rum-runtime/src/runtime/model/session.rs`
  - `crates/rum-runtime/src/runtime/model/mod.rs`
  - one focused new runtime model/session module if keeping either named file
    below its organization threshold requires it
  - `crates/rum-runtime/src/runtime/session/session_api.rs`
  - `crates/rum-runtime/src/runtime/session/session_helpers.rs`
  - `crates/rum-runtime/src/runtime/session/session_requests.rs`
  - `crates/rum-runtime/src/runtime/session/session_eval.rs`
  - `crates/rum-runtime/src/runtime/session/session_state.rs`
  - `crates/rum-runtime/src/runtime/eval_core/mod.rs`
  - `crates/rum-runtime/src/runtime/tests/session_requests/api.rs`
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` and
    `crates/rum-runtime/src/runtime/tests/part_assignment.rs` only if their
    private interrupt setup must follow the new scoped helper now
  - this dispatched brief, report append only
- Files/areas you MUST NOT touch:
  - `rum-host`, `rum-cli`, `rum-repl`, `rum-jupyter`, `rum-stdlib`, the `Do`
    driver, `Cargo.lock`, `.git`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, and all
    other `docs/**`.
- Keep the legacy `interrupt_handle`/boolean behavior intact in this additive
  step.  Do not silently reinterpret a session-scoped handle as “the next
  request.”
- No new dependency and no public expression/serde shape change.

##### Deliverable

- A private session control block with an idle state and non-reused nonzero
  request generations.  Encode cancellation with the generation in a single
  atomic state transition, or use an equivalently proven state machine that
  prevents stale stores from overwriting the current request.
- A non-`Clone` accepted runtime request ticket with private fields, and a
  cloneable qualified runtime cancel handle obtainable only from that ticket.
  Dropping an unexecuted ticket makes its handles inert.
- Accepted variants of source and expression request evaluation and the
  script/file paths needed by `rum-host` in the next brief.  A ticket is
  validated against the receiving session and consumed exactly once.
- An active accepted identity is installed before
  `begin_top_level_interrupt_budget`.  Scoped cancellation is never cleared by
  that prologue.  `interrupt_armed` includes the active scope independent of Arc
  strong count and remains armed through suspension/resume.
- Completion atomically closes the exact generation and observes whether
  cancellation or completion won.  A winning cancel returns the ordinary
  `Cancelled` request outcome even after the final amortized evaluator poll; a
  cancel whose CAS loses to completion is a stale no-op.
- Input suspension retains the ticket.  A cancel already present before the
  suspension boundary prevents publication of `InputRequired`; a cancel during
  suspension is checked before resume consumes host input.  Nested resuspension
  retains the same identity.
- Script/file accepted requests retain one identity across forms and execute no
  later forms after cancellation terminates the request.
- Deterministic tests, with no sleep:
  1. `accept -> cancel -> eval` returns cancellation;
  2. a completed request's handle cannot affect a later ordinary or accepted
     request;
  3. a stale-handle call after a current-handle call cannot overwrite the
     current cancellation;
  4. cancel between `InputRequired` and resume is observed before the supplied
     input/continuation runs;
  5. normal completion and timeout remain usable.
- A short invariant comment beside the atomic encoding/CAS states why a bare
  boolean or unconditional clear is insufficient.

##### rum / Rust pitfalls (read twice)

1. One `AtomicU64::store(generation)` for every cancel is not sufficient: a
   delayed stale handle can overwrite a newer cancellation.  The handle must
   conditionally transition only its exact active generation.
2. A load of “cancelled?” followed by an unrelated idle store loses a cancel
   between the two.  Close-and-observe must have one atomic linearization order.
3. Do not infer arming from `Arc::strong_count`.  A cancel-and-drop can race the
   prologue.  The active accepted scope itself is the arming proof.
4. Do not allocate one `Arc<AtomicBool>` per request.  Reuse the session control
   block and rotate it only on otherwise-impossible generation exhaustion.
5. The ticket is linear.  Do not derive `Clone`/`Copy`, expose its fields, or
   accept a numeric generation supplied by a caller.
6. Suspension is the same logical request.  Do not finish the ticket on
   `InputRequired` and mint another on resume; preserve continuation shape.
7. Keep timeout separate.  Its monotonic deadline is not a request identity and
   must retain the present start semantics.
8. The ordinary path must not allocate, lock, clone an Arc, or add an atomic
   operation.  The existing evaluator `if interrupt_armed` branch and poll
   stride remain unchanged.
9. No new `unsafe`; if the safe state machine becomes awkward, stop and report
   the ownership conflict rather than bypassing it.
10. Keep the legacy raw lane only as an explicit compatibility path for the next
    brief.  Scoped tests must not pass by accidentally setting the legacy flag.

##### Verification (MANDATORY — run and paste verbatim)

Before editing, with profiling environment variables off, build and save the
base timings:

```bash
python3 scripts/run-bounded.py --timeout 900 -- cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh \
  --output-dir /tmp/brief-399-request-cancel-base \
  --rum-cli target/release/rum-cli
```

After editing:

```bash
cargo fmt --check
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-runtime \
  runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed \
  -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-runtime \
  runtime::tests::session_requests::api::stale_cancel_cannot_overwrite_current_request_cancel \
  -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-runtime \
  runtime::tests::session_requests::api::accepted_cancel_survives_input_suspension \
  -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 1200 -- cargo test --workspace

python3 scripts/run-bounded.py --timeout 900 -- cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh \
  --output-dir /tmp/brief-399-request-cancel-new \
  --rum-cli target/release/rum-cli
```

Use the final test names if implementation-appropriate names differ, but list
the substitutions.  Paste exact test result lines.  Compare every `timings.tsv`
row and paste base/new rows for `literal-integer`, `literal-string`,
`symbol-ownvalue-read`, and `list-literal-small`, plus every row that regresses.
Any regression on a hot label or same-tier neighbor is a defect to fix or
report.

##### Hard rules

- Do NOT commit, branch, push, modify git config, or touch files outside Scope.
- Preserve all pre-existing worktree changes.  Do not use a cleanup/reset to
  manufacture a baseline.
- Testing Policy is a hard line: rum-native contract tests and rum-vs-rum A/B
  only; no third-party engine or output.
- No new `unsafe`, allocation/lock on the ordinary request path, adapter change,
  `Do` change, extra evaluator poll, or timeout semantic change.
- The deterministic ordering tests use direct accepted-ticket order or channel/
  barrier acknowledgement, never a sleep, retry, or “long enough” workload.
- Use `scripts/run-bounded.py` for long/spawning commands.  If the same failure
  occurs twice for the same reason or an A/B row regresses without a clear
  repair, stop and report.
- Append real outputs to the dispatched brief.  Do not claim a gate or timing
  row that was not produced.

##### Report format

1. State machine and type/API changes, per file and function.
2. Deterministic ordering evidence, including cancellation-versus-completion
   and suspension/resume.
3. Verbatim focused, crate, and workspace test output.
4. Base/new microbench rows and all-row regression verdict.
5. Compatibility lane left for Draft 2, stop-loss, and open questions.

#### Draft 2 — migrate the host boundary and every embedder

##### Goal

Make request-qualified cancellation the only cancellation API exposed by
`rum-host` and used in-tree.  An interrupt for an accepted but unstarted Eval
worker must be deterministic and effective; stale handles must be inert; the
same identity must survive Jupyter input suspension and Resume.  Migrate CLI,
host/runtime tests, and Jupyter atomically, remove the temporary raw runtime
lane, update the normative documentation, and preserve ordinary-path
performance, Control responsiveness, session recovery, and all existing
protocol routing/lifecycle assertions.

##### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/host-embedding-and-hardening.md`, `docs/guide/embedding.md`,
  `docs/guide/resource-limits-and-sandbox.md`, Draft 1's report, and brief 398's
  accepted design before editing.
- `crates/rum-host/src/session.rs:101-127,318-448,485-536` owns the current
  handle, guest request entrypoints, timeout, and resume facade.
- `crates/rum-host/src/model/requests.rs:40-66` owns `HostEvalRequest`; put new
  public request-control types at the model boundary rather than growing the
  already split-plan-range `session.rs` without a module boundary.
- `crates/rum-host/tests/lifecycle_contract.rs:107-179` is the normative stale,
  running-cancel, recovery, and reset contract.
- `crates/rum-cli/src/main.rs:246-253,327-343` owns streaming Print/EPIPE
  cancellation for eval, script, and file requests.
- `crates/rum-jupyter/src/adapter/worker.rs:21-115,168-186` owns worker
  acceptance, handle lifetime, Eval/Resume execution, readiness, and join.
- `crates/rum-jupyter/src/adapter/mod.rs:36-47,154-160,273-318,386-464` owns
  running/pending execute state, Control, input suspension, completion, and
  resume.
- `crates/rum-jupyter/src/tests/pump.rs:174-343` owns the full interrupted Eval
  transport contract; `tests/stdin.rs:127-236` owns the primary Resume contract;
  `tests/mod.rs:27,66-87` owns bounded async test support.
- `crates/rum-runtime/src/runtime/session/session_api.rs:136-144`,
  `session_helpers.rs:31-39`, and `eval_core/mod.rs:84-97` contain the temporary
  legacy lane to remove after migration.
- Acceptance is mechanical: forced worker ordering, focused host/runtime tests,
  capped repeated release tests, workspace tests, documentation compile/drift
  gates, and the named A/B rows decide.

##### Scope

- Files you MAY modify:
  - the Draft-1 `rum-runtime` model/session/poll files and the three runtime test
    files named in brief 398's inventories
  - `crates/rum-host/src/model/requests.rs`
  - `crates/rum-host/src/model/mod.rs`
  - one focused new host request-control module if required by organization
  - `crates/rum-host/src/session.rs`
  - `crates/rum-host/src/lib.rs`
  - `crates/rum-host/src/tests/eval_contract.rs`
  - `crates/rum-host/src/tests/session_lifecycle.rs`
  - `crates/rum-host/tests/lifecycle_contract.rs`
  - `crates/rum-host/tests/stress.rs`
  - `crates/rum-cli/src/main.rs` and its existing tests if signatures ripple
  - `crates/rum-jupyter/src/adapter/worker.rs`
  - `crates/rum-jupyter/src/adapter/mod.rs`
  - `crates/rum-jupyter/src/tests/mod.rs`
  - `crates/rum-jupyter/src/tests/pump.rs`
  - `crates/rum-jupyter/src/tests/stdin.rs`
  - other existing Jupyter Control tests only for mechanical API/lifecycle
    assertion ripple; list each
  - `docs/guide/embedding.md`
  - `docs/guide/resource-limits-and-sandbox.md`
  - `docs/host-embedding-and-hardening.md`
  - `docs/runtime-evaluation-audit.md`
  - `TODO.md` only to remove/correct the stale cancellation-test entry
  - this dispatched brief, report append only
- Files/areas you MUST NOT touch:
  - `rum-core` expression representation, `rum-stdlib`, the `Do` driver,
    Jupyter native-ZeroMQ skip behavior, Finding A at
    `crates/rum-jupyter/src/tests/pump.rs:37`, public Jupyter wire/model serde
    shapes, `Cargo.lock`, `.git`, `AGENTS.md`, `PROGRESS.md`, and unrelated docs.
- No adapter-side cancellation reassertion/latch and no global test hook.

##### Deliverable

- Public `HostAcceptedRequest`: private fields, non-`Clone`, tied to its session
  control block, and consumed by accepted eval/script/file/call entrypoints.
  `HostCancelHandle` wraps only the matching generation-qualified runtime
  handle.  Remove `HostSession::cancel_handle()`.
- The host rustdoc contains this exact contract sentence: “A
  `HostCancelHandle` affects only the accepted evaluation request that produced
  it: cancellation is effective from acceptance, through every suspension and
  resume of that request, until its completion, and calls made after that
  request completes never affect another request.”
- Ordinary non-cancellable entrypoints remain source-compatible except for code
  that requested a session-scoped handle.  A ticket moved to the wrong session
  fails closed with a structured host failure; dropping it abandons the request
  and invalidates its handles.
- Host contract tests prove accepted-before-start cancellation, stale
  post-completion inertness, stale/current ordering, during-evaluation
  cancellation, suspension/resume retention, reset invalidation, timeout OR
  arming, `Send`/`Sync`, and post-cancel usability.  No cancellation ordering is
  based on a sleep.
- CLI accepts exactly one ticket before installing its print sink and captures
  that ticket's handle.  EPIPE still cancels the active eval/script/file request
  and exits successfully; setup-binding and error/rendering behavior are
  unchanged.
- Jupyter accepts the Eval request before `thread::spawn`, retains one handle in
  logical execute state across `InputRequired`, and gives Resume a clone of that
  same handle.  `spawn_resume` does not mint an identity.  Control remains
  immediate and can latch the current identity while worker-running or
  input-suspended; no polling reassertion is added.
- An instance-scoped test-only start gate made from channels or a barrier, owned
  by exactly one worker/test and compiled out of production.  It acknowledges a
  pause immediately before the host Eval/Resume entrypoint and releases exactly
  that worker.
- `control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle`
  uses the gate to force: worker accepted and paused -> deferred Shell asserted
  -> Control delivered and replied -> worker released -> cancellation reply.
  Preserve every existing routing ID, IOPub, lifecycle, deferred `42`, and
  session-reuse assertion.
- A Resume test forces the same pre-entry ordering while reusing the original
  identity, and a suspension test cancels after `InputRequired` but before
  resume, then proves resume observes cancellation before the input/continuation
  side effects.  Both use bounded completion waits, never ordering sleeps.
- Remove the temporary legacy boolean, raw runtime handle, Arc-strong-count
  arming, and compatibility poll branch.  Direct mutation tests use a private
  qualified test helper rather than exposing atomic state.
- Update every current `cancel_handle()` documentation reference and the stale
  TODO entry.  The resource-limit RSS example obtains the handle from an
  accepted request.  Reset documentation says both completed-request and
  pre-reset handles are inert.
- Repository searches prove no unqualified handle constructor/use remains in
  runtime, host, Jupyter, CLI, REPL, or tests.

##### rum / Rust pitfalls (read twice)

1. The worker-start gate proves ordering; a 100 ms delay, longer workload,
   repeated cancel, or retry is not a substitute.
2. `KernelAdapter::interrupt` must not wait for guest cooperation.  Store the
   qualified cancellation and emit Control reply immediately.
3. Do not use the existing `interrupted` boolean as a cancellation latch.  It
   may remain protocol bookkeeping/late-success defense only if still needed,
   but correctness must come from the runtime identity.
4. Eval and Resume are segments of one logical execute.  A new generation in
   `spawn_resume` breaks cancel-during-suspension and is forbidden.
5. A cancel racing final success is decided by the runtime close CAS.  Do not
   recreate `force_cancelled_success` as the primary correctness mechanism.
6. Do not consume or replay the suspended continuation merely to manufacture a
   test.  Resume must preserve the existing saved-continuation shape and prefix
   side-effect guarantees.
7. Multi-form script/file cancellation stops later forms for that accepted
   request.  Do not clear the scoped bit at each form budget.
8. Timeout stays monotonic and independent.  Removing strong-count arming must
   not disarm an active timeout or an accepted request whose external handle was
   dropped after cancelling.
9. The default path pays no request-ticket allocation, Arc clone, lock, or new
   atomic.  Cancellation-enabled requests may pay their explicit cold-control
   setup cost.
10. No new `unsafe`; preserve one-session/one-worker ownership and do not share
    `HostSession` mutably.
11. Finding A and ZeroMQ skip behavior are explicitly out of scope even if their
    tests appear during the workspace gate.

##### Verification (MANDATORY — run and paste verbatim)

Before editing Draft 2, save a Step-1 timing baseline with profiling variables
off:

```bash
python3 scripts/run-bounded.py --timeout 900 -- cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh \
  --output-dir /tmp/brief-400-request-cancel-base \
  --rum-cli target/release/rum-cli
```

After editing, run:

```bash
cargo fmt --check
python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 600 -- \
  cargo test -p rum-host --test lifecycle_contract cancel -- --nocapture
python3 scripts/run-bounded.py --timeout 600 -- \
  cargo test -p rum-host --test stress cancel_interrupts_heavy_rewrite_loop_and_session_recovers -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 600 -- \
  cargo test -p rum-host host_cancel -- --nocapture
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-jupyter --lib \
  tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle \
  -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-jupyter --lib <exact-qualified-resume-start-test> \
  -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 180 -- \
  cargo test -p rum-jupyter --lib <exact-cancel-during-suspension-test> \
  -- --exact --nocapture
python3 scripts/run-bounded.py --timeout 600 -- cargo test -p rum-cli
python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-jupyter
python3 scripts/run-bounded.py --timeout 1200 -- cargo test --workspace
```

After the exact test names are known, run each forced Eval and Resume start test
30 times in release mode under one 900-second bounded process and print a
verbatim `pass=30 fail=0 total=30` summary.  A failure is evidence; do not retry
it away.

Then build and measure the candidate:

```bash
python3 scripts/run-bounded.py --timeout 900 -- cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh \
  --output-dir /tmp/brief-400-request-cancel-new \
  --rum-cli target/release/rum-cli

rg -n "cancel_handle\(|interrupt_handle\(|Arc<AtomicBool>|interrupt_flag" \
  crates/rum-runtime crates/rum-host crates/rum-jupyter crates/rum-cli crates/rum-repl
```

The final `rg` is expected to find neither an unqualified constructor nor the
legacy runtime flag; list any intentional documentation/test-name matches
separately.  Compare every timing row and paste base/new rows for
`literal-integer`, `literal-string`, `symbol-ownvalue-read`, and
`list-literal-small`, plus every regression.  If a native ZeroMQ test skips,
state that it skipped; it is not cancellation evidence.

##### Hard rules

- Do NOT commit, branch, push, modify git config, or touch files outside Scope.
- Preserve all pre-existing worktree changes.  No cleanup/reset of unrelated
  files and no generated artifact/probe left in the repository.
- Testing Policy is a hard line: only rum's documented contract and rum-vs-rum
  measurements; no vendor oracle, compatibility output, or external engine.
- No new `unsafe`, adapter latch/reassertion, fixed-duration ordering, extra
  `Do`/evaluator poll, Control blocking, timeout semantic change, or
  ordinary-path allocation/lock/atomic cost.
- Test-only start control is instance-scoped, acknowledged, and compiled out of
  production.  No global mutable hook or environment variable.
- Use bounded runners for long/spawning commands.  If the same failure occurs
  twice for the same reason or any hot/same-tier A/B row regresses, stop and
  report rather than weakening coverage or silently remeasuring.
- Update normative docs in the same change as the API.  Never leave the old
  “cancel before evaluation is a no-op” or bare `cancel_handle()` guidance.
- Append real outputs to the dispatched brief and name exact substituted test
  names.  Do not claim skipped tests or absent timing rows as evidence.

##### Report format

1. Public contract and per-file API/state changes.
2. Forced Eval ordering, Resume identity, and suspension-latch evidence.
3. Host, CLI, Jupyter, runtime, and documentation migration inventory; final
   no-legacy `rg` output.
4. Verbatim focused/crate/workspace tests and 30/30 summaries.
5. Base/new microbench rows and all-row verdict.
6. Stop-loss, skips, risks, and open questions.

### 5. Stop-loss, risks, open questions, and corrections

#### Stop-loss

No source instrumentation or probe files were needed.  No test or benchmark was
run because this is a design-only brief and the template says the evidence is
the deliverable.  After this report was written, only read-only inventory/status
verification is permitted; its verbatim output is appended below.

#### Risks and resolved edge cases

- **Cancel versus completion:** a separate load/clear is racy.  The exact-
  generation close CAS is required and defines whether `cancel()` was accepted
  while the request was still current.
- **Stale handle versus current cancel:** a shared `store(generation)` is racy.
  Exact-generation CAS prevents an older handle from overwriting the current
  cancelled state.
- **Generation wrap:** identity reuse would make an ancient handle relevant
  again.  Rotate the control-block Arc while idle instead of wrapping.
- **Abandoned acceptance:** dropping the linear ticket must close its generation
  so a forgotten prepared request cannot reserve the session forever.
- **Cancel before syntax/lowering completes:** cancellation and terminal return
  use the same completion linearization.  If cancel wins before the request
  closes, cancellation is the result; if syntax/failure closes first, the later
  cancel is stale.  This is deterministic and does not depend on whether an
  evaluator poll occurred.
- **Suspension:** returning `InputRequired` is not completion.  Keep the scope
  active, reject a cancel already latched at the suspension boundary, and check
  again before resume consumes input.  The handle alone cannot make progress
  while no runtime thread is executing; that is the existing cooperative model,
  not a lost signal.
- **Multi-form script/file:** a request identity cannot be cleared at each form.
  A cancellation termination stops later forms; otherwise “request-scoped”
  would still lose the request between forms.
- **Public API break:** removing `HostSession::cancel_handle()` is intentional.
  Keeping it as an inert or “next request” compatibility shim would restore the
  ambiguity the repair is meant to remove.
- **Performance:** the accepted path may pay cold setup costs; the ordinary path
  may not.  Both implementation steps are microbench-gated because the
  prologue/poll layout changes.

There is no blocking design question.  Type names and whether the atomic state
lives in `model/requests.rs` or a focused new module can be chosen to respect
file-size boundaries; the lifecycle and public contract above should not vary.

#### Corrections to the supplied background

1. `spawn_resume` has the same handle-before-thread scheduling window as
   `spawn_eval`, but it does **not** currently have the same erasing mechanism.
   `resume_eval_request_with_input` at
   `session_requests.rs:185-249` calls `resume_suspended_evaluation` directly and
   never calls `begin_top_level_interrupt_budget`; therefore a cancel delivered
   just before the current Resume entry is not cleared today.  Resume still
   needs the same logical identity so cancellation cannot be lost while the
   request is suspended, so the adapter cannot mint a different request on
   resume, and so a future prologue refactor cannot reopen the class.
2. `TODO.md:701-707` accurately describes the pre-brief-396 test shape but not
   the current source.  The four named tests now acknowledge a Print/test
   builtin through channels before cancelling.  The ignored ten-cycle stress
   test at `stress.rs:124-144` is the remaining fixed-sleep cancellation test.
3. `interrupt_armed` is not erased in the demonstrated Jupyter Eval ordering:
   the worker struct retains the handle, so `Arc::strong_count > 1` remains true.
   The cancellation **bit** is erased.  Strong-count arming nevertheless has a
   separate cancel-and-drop observation race and should be replaced by the
   active accepted scope as part of the same structural repair.
4. The elapsed timeout has no pre-prologue signal to erase.  It shares the
   `interrupt_armed` gate and amortized poll helper, but not this defect.

### Read-only verification evidence (run after writing the design)

No cargo test or benchmark was run: this brief is design-only, and the report
plus source inventory is its evidence.  The following outputs are verbatim.

Command:

```bash
rg -n "cancel_handle\(|HostCancelHandle|interrupt_handle\(|\.cancel\(\)" \
  crates/rum-runtime crates/rum-host crates/rum-jupyter crates/rum-cli crates/rum-repl \
  --glob '*.rs'
```

Output:

```text
crates/rum-cli/src/main.rs:330:    let cancel = session.cancel_handle();
crates/rum-cli/src/main.rs:340:                cancel.cancel();
crates/rum-jupyter/src/adapter/worker.rs:8:    HostCancelHandle, HostEvalOutcome, HostEvalRequest, HostEvalSuccess, HostFailure,
crates/rum-jupyter/src/adapter/worker.rs:28:    cancel: HostCancelHandle,
crates/rum-jupyter/src/adapter/worker.rs:52:        worker.cancel.cancel();
crates/rum-jupyter/src/adapter/worker.rs:88:        let cancel = session.cancel_handle();
crates/rum-jupyter/src/adapter/worker.rs:99:        let cancel = session.cancel_handle();
crates/rum-jupyter/src/adapter/mod.rs:156:        self.execute_state.cancel();
crates/rum-host/src/tests/eval_contract.rs:219:    let cancel = session.cancel_handle();
crates/rum-host/src/tests/eval_contract.rs:236:            cancel.cancel();
crates/rum-host/src/tests/eval_contract.rs:325:    let _cancel = session.cancel_handle();
crates/rum-host/src/tests/session_lifecycle.rs:12:    is_send::<HostCancelHandle>();
crates/rum-host/src/lib.rs:69:pub use session::{HostCancelHandle, HostSession, HostSessionBuilder};
crates/rum-host/src/session.rs:118:pub struct HostCancelHandle {
crates/rum-host/src/session.rs:122:impl HostCancelHandle {
crates/rum-host/src/session.rs:171:    /// [`HostCancelHandle`], because the rebuilt session has a fresh interrupt
crates/rum-host/src/session.rs:444:    pub fn cancel_handle(&self) -> HostCancelHandle {
crates/rum-host/src/session.rs:445:        HostCancelHandle {
crates/rum-host/src/session.rs:446:            flag: self.session.interrupt_handle(),
crates/rum-host/tests/stress.rs:57:    let cancel = session.cancel_handle();
crates/rum-host/tests/stress.rs:72:        cancel.cancel();
crates/rum-host/tests/stress.rs:128:        let cancel = session.cancel_handle();
crates/rum-host/tests/stress.rs:131:            cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:2:    HostCancelHandle, HostConfig, HostDefinitionSummary, HostEvalOutcome, HostEvalSuccess,
crates/rum-host/tests/lifecycle_contract.rs:110:    let cancel = session.cancel_handle();
crates/rum-host/tests/lifecycle_contract.rs:112:    cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:115:    cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:122:    let cancel = session.cancel_handle();
crates/rum-host/tests/lifecycle_contract.rs:137:        cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:169:    let old_cancel = session.cancel_handle();
crates/rum-host/tests/lifecycle_contract.rs:174:    old_cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:326:    assert_send::<HostCancelHandle>();
crates/rum-host/tests/lifecycle_contract.rs:327:    assert_sync::<HostCancelHandle>();
crates/rum-runtime/src/runtime/tests/session_requests/api.rs:529:    let cancel = session.interrupt_handle();
crates/rum-runtime/src/runtime/session/session_api.rs:136:    pub fn interrupt_handle(&self) -> Arc<AtomicBool> {
```

Command:

```bash
rg -n "interrupt_flag|interrupt_armed|begin_top_level_interrupt_budget" \
  crates/rum-runtime crates/rum-host --glob '*.rs'
```

Output:

```text
crates/rum-runtime/src/runtime/definitions/part_assignment.rs:171:        if self.interrupt_armed {
crates/rum-runtime/src/runtime/definitions/incremental_append.rs:40:        if self.interrupt_armed {
crates/rum-runtime/src/runtime/eval_core/mod.rs:22:        if self.interrupt_armed {
crates/rum-runtime/src/runtime/eval_core/mod.rs:89:        if self.interrupt_flag.load(Ordering::Relaxed) {
crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:707:    session.interrupt_armed = true;
crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:710:        .interrupt_flag
crates/rum-runtime/src/runtime/model/session.rs:84:    pub(in crate::runtime) interrupt_armed: bool,
crates/rum-runtime/src/runtime/model/session.rs:112:    pub(in crate::runtime) interrupt_flag: Arc<AtomicBool>,
crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs:46:        if self.interrupt_armed {
crates/rum-runtime/src/runtime/tests/part_assignment.rs:392:    session.interrupt_armed = true;
crates/rum-runtime/src/runtime/tests/part_assignment.rs:395:        .interrupt_flag
crates/rum-runtime/src/runtime/session/session_eval.rs:215:        self.begin_top_level_interrupt_budget();
crates/rum-runtime/src/runtime/session/session_helpers.rs:31:    pub(in crate::runtime) fn begin_top_level_interrupt_budget(&mut self) {
crates/rum-runtime/src/runtime/session/session_helpers.rs:32:        self.interrupt_flag.store(false, Ordering::Relaxed);
crates/rum-runtime/src/runtime/session/session_helpers.rs:37:        self.interrupt_armed =
crates/rum-runtime/src/runtime/session/session_helpers.rs:38:            self.eval_deadline.is_some() || std::sync::Arc::strong_count(&self.interrupt_flag) > 1;
crates/rum-runtime/src/runtime/session/session_api.rs:137:        Arc::clone(&self.interrupt_flag)
crates/rum-runtime/src/runtime/session/session_api.rs:143:        self.interrupt_armed =
crates/rum-runtime/src/runtime/session/session_api.rs:144:            self.eval_timeout.is_some() || Arc::strong_count(&self.interrupt_flag) > 1;
crates/rum-runtime/src/runtime/session/session_state.rs:632:            interrupt_armed: false,
crates/rum-runtime/src/runtime/session/session_state.rs:660:            interrupt_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
crates/rum-runtime/src/runtime/session/session_requests.rs:112:        self.begin_top_level_interrupt_budget();
crates/rum-runtime/src/runtime/session/session_requests.rs:130:        self.begin_top_level_interrupt_budget();
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:150:            if self.interrupt_armed {
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:237:            if self.interrupt_armed {
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:297:            if self.interrupt_armed {
```

Command:

```bash
rg -n "evaluation-started|thread::sleep\(Duration::from_millis\(25\)\)|cancel\.cancel\(\)|cancel\.store\(true" \
  crates/rum-host/tests/lifecycle_contract.rs \
  crates/rum-host/src/tests/eval_contract.rs \
  crates/rum-host/tests/stress.rs \
  crates/rum-runtime/src/runtime/tests/session_requests/api.rs
```

Output:

```text
crates/rum-runtime/src/runtime/tests/session_requests/api.rs:12:    session.push_print("evaluation-started");
crates/rum-runtime/src/runtime/tests/session_requests/api.rs:522:        assert_eq!(line, "evaluation-started");
crates/rum-runtime/src/runtime/tests/session_requests/api.rs:536:            cancel.store(true, Ordering::Relaxed);
crates/rum-host/tests/stress.rs:12:const CANCELLABLE_REWRITE_LOOP_UNDER_LOAD: &str = "Print[\"evaluation-started\"]; expr = 0; \
crates/rum-host/tests/stress.rs:61:        assert_eq!(line, "evaluation-started");
crates/rum-host/tests/stress.rs:72:        cancel.cancel();
crates/rum-host/tests/stress.rs:130:            thread::sleep(Duration::from_millis(25));
crates/rum-host/tests/stress.rs:131:            cancel.cancel();
crates/rum-host/src/tests/eval_contract.rs:7:const CANCELLABLE_LONG_RUNNING_EVAL: &str = "Print[\"evaluation-started\"]; Do[x = 1, 50000000]";
crates/rum-host/src/tests/eval_contract.rs:223:        assert_eq!(line, "evaluation-started");
crates/rum-host/src/tests/eval_contract.rs:236:            cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:11:const CANCELLABLE_LONG_RUNNING_EVAL: &str = "Print[\"evaluation-started\"]; Do[x = 1, 50000000]";
crates/rum-host/tests/lifecycle_contract.rs:112:    cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:115:    cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:126:        assert_eq!(line, "evaluation-started");
crates/rum-host/tests/lifecycle_contract.rs:137:        cancel.cancel();
crates/rum-host/tests/lifecycle_contract.rs:174:    old_cancel.cancel();
```

Key source excerpt command:

```bash
nl -ba crates/rum-jupyter/src/adapter/worker.rs | sed -n '47,105p;168,186p'
nl -ba crates/rum-runtime/src/runtime/session/session_requests.rs | sed -n '108,134p;185,249p'
nl -ba crates/rum-runtime/src/runtime/session/session_helpers.rs | sed -n '31,39p'
nl -ba crates/rum-runtime/src/runtime/session/session_api.rs | sed -n '136,145p'
```

Key output (verbatim lines bearing on startup, Resume, clear, and arming):

```text
    47	    pub(super) fn cancel(&mut self) -> bool {
    48	        let Self::Running(worker) = self else {
    49	            return false;
    50	        };
    51	        worker.interrupted = true;
    52	        worker.cancel.cancel();
    53	        true
    54	    }
    56	    pub(super) fn start_eval(&mut self, session: HostSession, context: ExecuteContext) {
    57	        debug_assert!(!self.is_running());
    58	        *self = Self::Running(ExecuteWorker::spawn_eval(session, context));
    59	    }
    61	    pub(super) fn start_resume(
    62	        &mut self,
    63	        session: HostSession,
    64	        context: ExecuteContext,
    65	        input: String,
    66	    ) {
    67	        debug_assert!(!self.is_running());
    68	        *self = Self::Running(ExecuteWorker::spawn_resume(session, context, input));
    69	    }
    71	    pub(super) fn take_finished(&mut self) -> Option<ExecuteWorker> {
    72	        let Self::Running(worker) = self else {
    73	            return None;
    74	        };
    75	        if !worker.handle.is_finished() {
    76	            return None;
    77	        }
    79	        let Self::Running(worker) = std::mem::replace(self, Self::Idle) else {
    80	            unreachable!("checked running state before replace");
    81	        };
    82	        Some(worker)
    83	    }
    84	}
    86	impl ExecuteWorker {
    87	    fn spawn_eval(session: HostSession, context: ExecuteContext) -> Self {
    88	        let cancel = session.cancel_handle();
    89	        let code = context.request.code.clone();
    90	        Self {
    91	            handle: spawn_worker(session, ExecuteSegment::Eval { code }),
    92	            cancel,
    93	            context,
    94	            interrupted: false,
    95	        }
    96	    }
    98	    fn spawn_resume(session: HostSession, context: ExecuteContext, input: String) -> Self {
    99	        let cancel = session.cancel_handle();
   100	        Self {
   101	            handle: spawn_worker(session, ExecuteSegment::Resume { input }),
   102	            cancel,
   103	            context,
   104	            interrupted: false,
   105	        }
   168	fn spawn_worker(
   169	    mut session: HostSession,
   170	    segment: ExecuteSegment,
   171	) -> JoinHandle<(HostSession, KernelExecuteOutcome)> {
   172	    thread::spawn(move || {
   173	        let outcome = match catch_unwind(AssertUnwindSafe(|| run_segment(&mut session, segment))) {
   174	            Ok(outcome) => outcome,
   175	            Err(payload) => KernelExecuteOutcome::Failure(worker_panic_failure(&*payload)),
   176	        };
   177	        (session, outcome)
   178	    })
   179	}
   181	fn run_segment(session: &mut HostSession, segment: ExecuteSegment) -> KernelExecuteOutcome {
   182	    let outcome = match segment {
   183	        ExecuteSegment::Eval { code } => session.eval(HostEvalRequest::new(code)),
   184	        ExecuteSegment::Resume { input } => session.resume_with_input(input),
   185	    };
   186	    host_outcome_to_kernel(outcome)
   108	    pub fn eval_request(
   109	        &mut self,
   110	        input: &str,
   111	    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
   112	        self.begin_top_level_interrupt_budget();
   113	        let checkpoint = RequestCheckpoint::new(self);
   114	        let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
   115	        let expr = self
   116	            .lower_syntax_expr(parsed.syntax())
   117	            .map_err(|err| self.finish_request_failure(checkpoint, err))?;
   119	        self.eval_expr_request_from_checkpoint(expr, checkpoint)
   120	    }
   122	    /// Evaluates an already-built expression through the top-level request protocol.
   123	    ///
   124	    /// The expression is evaluated as-is: no lowering pass runs, so callers are
   125	    /// responsible for supplying resolved symbols.
   126	    pub fn eval_expr_request(
   127	        &mut self,
   128	        expr: Expr,
   129	    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
   130	        self.begin_top_level_interrupt_budget();
   131	        let checkpoint = RequestCheckpoint::new(self);
   133	        self.eval_expr_request_from_checkpoint(expr, checkpoint)
   134	    }
   185	    pub fn resume_eval_request_with_input(
   186	        &mut self,
   187	        input: impl Into<String>,
   188	    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
   189	        let input = input.into();
   190	        let checkpoint = self
   191	            .pending_eval_request
   192	            .take()
   193	            .map(RequestCheckpoint::from)
   194	            .ok_or_else(|| SessionRequestFailure {
   195	                error: SessionRequestError::Other {
   196	                    message: "no suspended evaluation is waiting for host input".to_string(),
   197	                },
   198	                messages: Vec::new(),
   199	                prints: Vec::new(),
   200	                trace: Vec::new(),
   201	            })?;
   203	        let previous_message_list_checkpoint = self
   204	            .current_message_list_checkpoint
   205	            .replace(checkpoint.message_checkpoint);
   206	        let previous_message_list_dirty = std::mem::replace(
   207	            &mut self.current_message_list_dirty,
   208	            self.messages.len() > checkpoint.message_checkpoint,
   209	        );
   210	        let outcome = self.resume_suspended_evaluation(input.clone());
   211	        let finalized =
   212	            self.finalize_top_level_eval_value(outcome, checkpoint.handled_error_checkpoint);
   213	        self.current_message_list_checkpoint = previous_message_list_checkpoint;
   214	        self.current_message_list_dirty = previous_message_list_dirty;
   215	        let (value, termination) = match finalized {
   216	            Ok(finalized) => finalized,
   217	            Err(EvalError::InputRequired { .. }) => {
   218	                self.pending_eval_request = Some(checkpoint.into());
   219	                return Err(self.finish_request_failure(
   220	                    checkpoint,
   221	                    EvalError::InputRequired {
   222	                        request: self
   223	                            .pending_input_request()
   224	                            .cloned()
   225	                            .expect("nested input suspension should record the pending request"),
   226	                    },
   227	                ));
   228	            }
   229	            Err(err @ EvalError::Syntax(_)) => {
   230	                return Err(self.finish_completed_request_failure(checkpoint, Some(&input), err));
   231	            }
   232	            Err(err) => return Err(self.finish_request_failure(checkpoint, err)),
   233	        };
   235	        let messages = self.messages.split_off(checkpoint.message_checkpoint);
   236	        let prints = self.prints.split_off(checkpoint.print_checkpoint);
   237	        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
   238	        self.last_message_list = messages.clone();
   239	        self.handled_non_loader_eval_errors
   240	            .truncate(checkpoint.handled_error_checkpoint);
   242	        Ok(SessionEvalResponse {
   243	            value,
   244	            termination,
   245	            messages,
   246	            prints,
   247	            trace,
   248	        })
   249	    }
    31	    pub(in crate::runtime) fn begin_top_level_interrupt_budget(&mut self) {
    32	        self.interrupt_flag.store(false, Ordering::Relaxed);
    33	        self.interrupt_poll_counter = 0;
    34	        self.eval_deadline = self.eval_timeout.map(|timeout| Instant::now() + timeout);
    35	        self.current_memory_limit = self.resource_limits.eval_memory_limit;
    36	        self.memory_limit_check_counter.set(0);
    37	        self.interrupt_armed =
    38	            self.eval_deadline.is_some() || std::sync::Arc::strong_count(&self.interrupt_flag) > 1;
    39	    }
   136	    pub fn interrupt_handle(&self) -> Arc<AtomicBool> {
   137	        Arc::clone(&self.interrupt_flag)
   138	    }
   140	    pub fn set_eval_timeout(&mut self, timeout: Option<Duration>) {
   141	        self.eval_timeout = timeout;
   142	        self.eval_deadline = None;
   143	        self.interrupt_armed =
   144	            self.eval_timeout.is_some() || Arc::strong_count(&self.interrupt_flag) > 1;
   145	    }
```

Final report diff check command:

```bash
git diff --check -- docs/dev/codex/brief-398-request-scoped-cancellation.md
```

Output: no output (success).

Final worktree audit command:

```bash
git status --short
```

Output:

```text
 M .claude/hooks/guard-dispatch.py
 M docs/dev/codex/brief-398-request-scoped-cancellation.md
 M scripts/README.md
 M scripts/core-memory-microbench.sh
 M scripts/dispatch-brief.sh
 M scripts/guarded-test-run.sh
?? scripts/peak-rss.py
```

Only `docs/dev/codex/brief-398-request-scoped-cancellation.md` was changed by
this brief.  The hook/script modifications and `scripts/peak-rss.py` were
present in the initial status and were left untouched.  No temporary
instrumentation or probe file was created.
