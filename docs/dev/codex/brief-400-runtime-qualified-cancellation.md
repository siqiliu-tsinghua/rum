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

---

## Report — 2026-09-02 — STOPPED on scope conflict

This brief did **not** reach completion. I stopped under the hard rule “Do not
modify files outside Scope. If you believe you must, STOP and report.” The
partial implementation remains in the worktree together with a deterministic
red test that demonstrates the ownership conflict described in section 6.

### 1. Partial state machine and API changes

- `runtime/model/request_cancellation.rs` (new) contains the reusable
  `AtomicU64` control block (`0` idle, otherwise
  `(generation << 1) | cancelled`), linear non-`Clone` `AcceptedRequest`,
  cloneable exact-generation `RequestCancelHandle`, idempotent exact-generation
  cancel CAS, and close-and-observe completion CAS. Its encoding comment states
  why a bare boolean and unconditional clear are unsound. Dropping a ticket
  closes only its generation. Generation exhaustion uses `0` as the private
  counter sentinel so the next idle acceptance rotates the `Arc` control block.
- `runtime/model/mod.rs` exports the ticket, handle, and acceptance error and
  keeps the control implementation runtime-private.
- `runtime/model/session.rs` and `session/session_state.rs` add one
  session-owned control block, the generation counter, the active accepted
  ticket, and a compatibility-only legacy arming cell. The legacy
  `Arc<AtomicBool>` remains.
- `session/session_api.rs::accept_cancellable_request` reserves only idle exact
  generations and rotates on exhaustion. `interrupt_handle` still returns the
  legacy raw handle; it now records legacy arming without `Arc::strong_count`.
  Timeout remains separate.
- `session/session_helpers.rs` installs and validates a ticket by `Arc`
  identity before the prologue, checks the active generation, and atomically
  closes it. `begin_top_level_interrupt_budget` clears only the legacy boolean
  and does not touch scoped cancellation. Arming derives from timeout, the
  active accepted scope, or the explicit legacy compatibility lane; no strong
  count remains.
- `eval_core/mod.rs::poll_host_interrupt_if_due` keeps the existing branch and
  1024-call stride. An accepted request performs one qualified atomic load;
  legacy and timeout-only behavior stay separate. No extra evaluator poll was
  added.
- `session/session_requests.rs` adds accepted source, expression, script, and
  file entrypoints; validates/consumes tickets; linearizes completion against
  cancel; retains the active ticket across ordinary input suspension and nested
  resuspension; and checks cancellation before converting supplied resume
  input. Script/file entrypoints keep one active identity.
- `session/session_eval.rs` observes a scoped cancellation after each form
  prologue and stops accepted script/file processing before a later form.
- `runtime/tests/session_requests/api.rs` contains direct-order tests for
  pre-entry cancel, idempotence, stale/completed handles, current-vs-stale CAS,
  ticket drop, wrong-session validation, exhaustion rotation, normal/timeout
  usability, script/file form stopping, and suspension/resuspension.
- I did not touch either optional private interrupt test file. No `unsafe`, new
  dependency, `Do` change, host/CLI/REPL/Jupyter/stdlib change, or out-of-scope
  source change was made. The five pre-existing modified files and untracked
  `scripts/peak-rss.py` were preserved.

Organization assessment: the new 5.7k model module keeps the state machine out
of already-large files. `session_api.rs`, `session_helpers.rs`,
`session_requests.rs`, and `session_state.rs` remain in the split-plan range;
this scoped migration cuts across their existing coherent API/helper/request/
state responsibilities, so an unrelated split would obscure the cancellation
lifetime review. `tests/session_requests/api.rs` is 31k, below the 45k test-debt
threshold but in its warning range; a future split boundary is accepted-request
lifecycle versus the older transport/error mapping tests.

### 2. Red-first proof and post-change pass

The first test was written before the state machine. It used the only
pre-change capability (the raw handle) to force cancel before the request
prologue. Verbatim failure:

```text
running 1 test

thread 'runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed' panicked at crates/rum-runtime/src/runtime/tests/session_requests/api.rs:445:5:
assertion `left == right` failed
  left: None
 right: Some(Cancelled)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... FAILED

failures:
    runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 905 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

After changing the same named regression to the typed accepted lane, its exact
run passed:

```text
running 1 test
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 905 filtered out; finished in 0.00s
```

### 3. Deterministic ordering evidence reached before the stop

The accepted-test filter used no sleeps and passed direct ordering, suspension,
nested resuspension, session validation, script/file channel handshakes, normal
completion, and timeout:

```text
running 6 tests
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... ok
test runtime::tests::session_requests::api::accepted_cancel_survives_input_suspension_without_consuming_input ... ok
test runtime::tests::session_requests::api::accepted_nested_resuspension_keeps_request_identity ... ok
test runtime::tests::session_requests::api::accepted_ticket_is_validated_against_its_session ... ok
test runtime::tests::session_requests::api::accepted_script_and_file_stop_before_later_forms_after_cancellation ... ok
test runtime::tests::session_requests::api::accepted_request_normal_completion_and_timeout_leave_session_usable ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 909 filtered out; finished in 0.01s
```

Cancel-before-entry followed by the completion CAS proves the cancel-wins side;
completion followed by stale cancellation proves the completion-wins side. A
current cancel followed by a stale cancel proves the stale generation cannot
overwrite the current cancelled state. A zero-capacity start/ack channel forces
script/file cancellation in the first form before the second form can run.
The resume input uses a conversion probe; the cancellation branch returns
before `Into<String>` is invoked. Nested resuspension reuses the original
handle.

### 4. Test output and the blocking red test

Before strengthening the suspension test, the crate suite was green:

```text
test result: ok. 913 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.17s

Doc-tests rum_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

That output is superseded by the current deterministic test, which checks that
abandoning a cancelled suspended `Block` restores its dynamic scope. Verbatim:

```text
running 1 test

thread 'runtime::tests::session_requests::api::accepted_cancel_survives_input_suspension_without_consuming_input' panicked at crates/rum-runtime/src/runtime/tests/session_requests/api.rs:600:5:
assertion `left == right` failed
  left: "List[8, inside]"
 right: "List[8, outside]"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::tests::session_requests::api::accepted_cancel_survives_input_suspension_without_consuming_input ... FAILED

failures:
    runtime::tests::session_requests::api::accepted_cancel_survives_input_suspension_without_consuming_input

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 915 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
```

Therefore the mandatory focused set, current crate gate, workspace gate, and
final `cargo fmt --check` did not reach green and were not claimed. The
workspace gate was not run after discovering the scope blocker.

### 5. Base noise floor; candidate comparison not run

Profiling environment variables were absent. The release base build completed:

```text
Finished `release` profile [optimized] target(s) in 23.34s
```

All three controls used the same saved binary and every invocation ran through
`taskset -c 0,2,4,6` under `scripts/run-bounded.py`. The machine was otherwise
idle; the pre-run process sample showed only `codex`, `bash`, `ps`, and `head`,
all at `0.0` CPU. Verbatim benchmark completion lines were:

```text
core microbench output: /tmp/brief-400-base-1
core microbench output: /tmp/brief-400-base-2
core microbench output: /tmp/brief-400-base-3
```

Named rows and their same-binary spread:

| label | base-1 | base-2 | base-3 | spread |
| --- | ---: | ---: | ---: | ---: |
| literal-integer | 0.000075342 | 0.000080311 | 0.000072944 | 10.10% |
| literal-string | 0.000078971 | 0.000083687 | 0.000078646 | 6.41% |
| symbol-ownvalue-read | 0.000218013 | 0.000202424 | 0.000200309 | 8.84% |
| list-literal-small | 0.000080809 | 0.000081216 | 0.000077611 | 4.64% |

Across all labels, measured min-to-max spreads ranged from 0.05% to 18.75%.
The rows at or above 10% were `table-iterator-symbol` 12.69%,
`literal-integer` 10.10%, `stringtrim-literal-list-32000` 12.10%,
`stringtrim-single-literal-32000` 11.23%, `stringtrim-whitespace-32000` 13.59%,
`appendto-list-incremental-small` 10.15%, `set-part-fixed100-len800` 18.75%,
`set-part-fill-large` 15.84%, `sort-default-wide` 18.62%,
`sort-symbol-default-width16000` 11.65%, and `reversesort-default-wide` 15.98%.
The complete raw per-label controls remain in the three `/tmp` directories.

No candidate build or candidate timing run was made after the blocker, so
there are no base/new rows and no all-row candidate verdict to claim.

### 6. Stop reason, step 2 boundary, and resume-prologue answer

The simple cancellation branch correctly avoids converting the supplied host
input, but clearing `pending_suspension` drops saved frames without driving
their error cleanup. The deterministic `Block` test proves that this leaves
dynamic runtime state installed. Correct cancellation must resume the saved
frame driver with `Err(EvalError::Cancelled)`, so every continuation frame
unwinds its scope while no host input is converted or delivered.

That driver and the private frame queue are in
`crates/rum-runtime/src/runtime/suspension/core.rs`, which is outside this
brief's MAY-modify list. There is no safe API in the allowed files that can
inject an error outcome at the front of the saved continuation. Reimplementing
the private queue, discarding frames, feeding dummy input, or using layout
access would respectively violate ownership, leak state, run continuation
effects, or require forbidden `unsafe`. The needed scope amendment is narrowly
to allow `suspension/core.rs` to expose a runtime-private cancellation resume
entrypoint that drives the existing frames with `Err(Cancelled)`.

Step 2 remains deliberately untouched: no host contract or consumer was
migrated, and the raw boolean lane remains. The partial qualified shape itself
would remain safe if resume later gained a prologue: the ticket stays installed
across suspension, arming derives from that active scope, and the prologue
clears only the legacy flag, never the qualified control word. The unresolved
issue is continuation cleanup, not prologue erasure.
