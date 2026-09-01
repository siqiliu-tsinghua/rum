# brief-400 — runtime-qualified request cancellation (step 1 of 2)

Step 1 of the repair planned by brief-398. **Additive**: the legacy raw
interrupt lane stays intact and every current `rum-host` consumer keeps
compiling and passing unchanged. Step 2 (a later brief) makes the typed host
contract authoritative and removes the legacy lane. Do not do step 2's work
here.

## Goal

Give cancellation a request identity inside `rum-runtime`, so a cancel aimed at
an accepted request is effective from the moment the request is accepted —
including before its evaluation prologue runs — while a cancel from a finished
or unrelated request stays inert. When you are done, a cancel that lands in the
window that today loses it must be observed, proven by a test that fails
against the current code and passes after the change.

## The defect this closes

Verified from source by the director, and made switchable in both directions by
brief-397:

- `crates/rum-runtime/src/runtime/session/session_requests.rs:112` —
  `eval_request`'s first statement is `begin_top_level_interrupt_budget()`.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:31` — that
  function's first line is `self.interrupt_flag.store(false, Ordering::Relaxed)`.
- A consumer can hold a cancel handle before the evaluation it targets has
  started (`crates/rum-jupyter/src/adapter/worker.rs:87-88` takes one before
  spawning the worker thread).

So a cancel arriving after acceptance and before the prologue is erased, with
nothing to re-deliver it. The clear itself is deliberate — a stale cancel must
not kill a later evaluation (`crates/rum-host/tests/lifecycle_contract.rs:107-117`)
— which is exactly why a bare boolean cannot serve both purposes.

**Note on resume, corrected in brief-398**: `resume_eval_request_with_input`
(`session_requests.rs:185-249`) calls `resume_suspended_evaluation` directly and
never calls `begin_top_level_interrupt_budget`, so the resume path has the same
scheduling window but **not** the erasing mechanism today. Your obligation there
is that the identity *survives* suspension and resume, not that you repair an
erase. Say in the report whether the shape you built would also be safe if a
future refactor gave resume a prologue.

## The design (from brief-398, accepted)

The state machine is the requirement; the names are not.

- One session-owned control block holding a single atomic state word: idle, or
  `(generation, cancelled-bit)`.
- A non-`Clone` accepted-request ticket with private fields, consumed by exactly
  one accepted entrypoint; dropping an unexecuted ticket makes its handles inert.
- A cloneable cancel handle obtainable only from that ticket, carrying its
  generation. `cancel()` compare-exchanges only its own exact
  `(generation, not-cancelled)` state; it is idempotent when already cancelled
  and a no-op against idle or any other generation.
- Completion is a **linearization point**: the finishing request atomically
  transitions its own generation back to idle and learns whether cancellation or
  completion won. A cancel that won before that transition still yields the
  ordinary `Cancelled` outcome even if the last amortized poll already ran; a
  cancel that loses is a stale no-op.
- Generation exhaustion must not reuse an identity: on exhaustion while idle,
  replace the control block so old handles address a retired one.
- The active accepted scope is installed **before**
  `begin_top_level_interrupt_budget`, and that prologue must not clear scoped
  cancellation.
- `interrupt_armed` derives from the active accepted scope, **not** from
  `Arc::strong_count`. brief-398 found the strong-count form carries its own
  cancel-and-drop observation race; retiring it is part of this step.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/model/requests.rs`
- `crates/rum-runtime/src/runtime/model/session.rs`
- `crates/rum-runtime/src/runtime/model/mod.rs`
- `crates/rum-runtime/src/runtime/session/session_api.rs`
- `crates/rum-runtime/src/runtime/session/session_helpers.rs`
- `crates/rum-runtime/src/runtime/session/session_requests.rs`
- `crates/rum-runtime/src/runtime/session/session_eval.rs`
- `crates/rum-runtime/src/runtime/session/session_state.rs`
- `crates/rum-runtime/src/runtime/eval_core/mod.rs`
- `crates/rum-runtime/src/runtime/tests/session_requests/api.rs`
- one focused new module under `runtime/model/` or `runtime/session/` if a named
  file would otherwise cross the organization threshold
  (`docs/code-organization-guidelines.md`)
- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` and
  `crates/rum-runtime/src/runtime/tests/part_assignment.rs`, **only** if their
  private interrupt setup must follow the new helper. If you touch either, name
  it in the report and say why — it means the reach is wider than planned.
- this brief, report append only.

MUST NOT touch: `rum-host`, `rum-cli`, `rum-repl`, `rum-jupyter`, `rum-stdlib`,
the `Do` driver, `Cargo.lock`, `.git`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
and all other `docs/**`.

Keep the legacy `interrupt_handle` boolean lane working. Do not reinterpret a
session-scoped handle as "the next request". No new dependency, no public
expression/serde shape change.

## Deliverable

- The control block, ticket, and qualified handle described above, with an
  invariant comment beside the atomic encoding stating why a bare boolean or an
  unconditional clear cannot serve.
- Accepted variants of the request entrypoints that exist today (source text,
  expression, script/file). A ticket is validated against its receiving session
  and consumed exactly once. Script/file accepted requests keep one identity
  across forms and run no later form after cancellation terminates the request.
- Suspension keeps the ticket: a cancel already present at the suspension
  boundary prevents publication of `InputRequired`; a cancel during suspension is
  observed before resume consumes host input; nested resuspension keeps the same
  identity.
- Deterministic tests, **no sleeps** — direct accepted-ticket ordering or a
  channel/barrier acknowledgement:
  1. accept → cancel → eval returns cancellation;
  2. a completed request's handle cannot affect a later ordinary or accepted
     request;
  3. a stale handle cannot overwrite the current request's cancellation;
  4. a cancel between `InputRequired` and resume is observed before the supplied
     input runs;
  5. normal completion and timeout remain usable.
- **Test (1) must be shown to fail against the pre-change code.** Write it
  first, run it, paste the failure, then implement. A test that only passes
  afterwards does not establish that it can catch the defect.

## rum / Rust pitfalls (read twice)

1. A plain `store(generation)` per cancel is not enough — a delayed stale handle
   can overwrite a newer cancellation. Transition only your exact generation.
2. A "cancelled?" load followed by an unrelated idle store loses a cancel
   between the two. Close-and-observe needs one linearization order.
3. Do not infer arming from `Arc::strong_count`; the active scope is the proof.
4. Do not allocate an `Arc<AtomicBool>` per request. Reuse the session control
   block; rotate it only on generation exhaustion.
5. The ticket is linear: no `Clone`/`Copy`, no public fields, never accept a
   caller-supplied generation number.
6. Suspension is the same logical request. Do not finish the ticket at
   `InputRequired` and mint another on resume; preserve the continuation shape.
7. Timeout stays separate. Its monotonic deadline is not a request identity and
   keeps its present start semantics.
8. The ordinary non-cancellable path must gain no allocation, lock, `Arc` clone,
   or atomic operation. The evaluator's `if interrupt_armed` branch and poll
   stride stay as they are.
9. No new `unsafe`. If the safe state machine becomes awkward, STOP and report
   the ownership conflict.
10. Scoped tests must not pass by accidentally setting the legacy flag.

## Verification (MANDATORY — you run it, paste verbatim)

```
cargo fmt --check
python3 scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 1800 -- cargo test --workspace
```

Plus each new deterministic test run by name with `--exact --nocapture`, and the
verbatim pre-change failure of test (1).

### The microbench A/B, and a host-specific requirement you must follow

This step changes the top-level prologue and touches `eval_core`'s amortized
poll, so it carries a rum-vs-rum A/B. Two things about the instrument on this
machine are not negotiable:

**Pin the runs to performance cores.** The host is an Intel i9-14900HX: sixteen
logical P-cores capped at 5.2 GHz (CPUs 0-15, hyperthreaded pairs) and sixteen
E-cores at 4.1 GHz (CPUs 16-31), different microarchitectures with different L2
sharing. An unpinned benchmark measures which core type the scheduler chose.
Prefix **every** microbench invocation with `taskset -c 0,2,4,6` — four distinct
physical P-cores, avoiding hyperthread siblings — and say in the report that you
did.

**Establish the noise floor on this host, in this session, before believing any
delta.** Do not cite a historical noise figure: every per-label number this
project recorded before `3e42c097` (2026-07-29) came from a wall-clock
instrument on a host now known to have had a drifting clock, and the suite's
behaviour on this machine has not been characterised. Run the **base build
against itself** at least three times, report the per-label spread, and judge
your candidate rows against that spread. A separate diagnostic brief will
calibrate this suite properly; until it lands, your own control is the only
floor you may use.

```
python3 scripts/run-bounded.py --timeout 900 -- cargo build --release -p rum-cli
taskset -c 0,2,4,6 bash scripts/core-language-microbench.sh \
  --output-dir /tmp/brief-400-base-1 --rum-cli target/release/rum-cli
# repeat for -base-2, -base-3 on the SAME binary → this is your noise floor
# then build the candidate and run -new-1, -new-2 the same way
```

Paste base/new rows for `literal-integer`, `literal-string`,
`symbol-ownvalue-read` and `list-literal-small`, plus every row that moves
beyond your measured floor. Profiling env vars OFF for every timing run. Say
whether the machine was otherwise idle.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do not modify files outside Scope. If you believe you must, STOP and report.
- **Preserve the pre-existing worktree changes** (five modified files under
  `scripts/` and `.claude/`, plus untracked `scripts/peak-rss.py`). Never reset
  or clean to manufacture a baseline.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  and rum-vs-rum A/B only.
- No new `unsafe`; no allocation or lock on the ordinary request path; no `Do`
  driver change; no extra evaluator poll; no timeout semantic change.
- Benchmark sources you generate are `*.rum`.
- Use `scripts/run-bounded.py --timeout SECONDS --` for long or child-spawning
  commands.
- **Stop-loss**: if the same step fails twice for the same reason, or a hot
  label regresses beyond your measured floor without a clear repair, STOP and
  report rather than continuing to tune.
- Paste real verbatim output. If a gate did not reach green, say so.

## Report format

1. The state machine and the type/API changes, per file and function.
2. The pre-change failure of test (1), verbatim, then the post-change pass.
3. Deterministic ordering evidence, including cancel-versus-completion and
   suspension/resume.
4. Verbatim focused, crate, and workspace test output.
5. The noise floor you measured, then base/new rows and an all-row verdict;
   confirm the `taskset` pinning and machine idleness.
6. What is deliberately left for step 2, stop-loss, and open questions —
   including whether your shape stays safe if resume later gains a prologue.
