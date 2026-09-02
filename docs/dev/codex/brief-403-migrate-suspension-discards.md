# brief-403 — implement: migrate every suspension discard, and reject overlapping requests

Shape: **implementation**. This is the brief that actually fixes the public
leak brief-401 demonstrated. brief-402 landed the protocol and deliberately
migrated nothing; this one migrates every caller, removes the raw-discard
capability, and closes the public lifecycle hole that let the leak be reached.

## Where this came from

brief-401 proved, through public `rum-host` API alone, that discarding a live
suspension drops its frames without unwinding them, so `Block` bindings,
`Module` activations, iterator scopes, `Reap`/`Monitor` scopes, constraint
deadlines, trace and capture checkpoints, `AbortProtect` depth, `BlockRandom`
state, and partial package installs all survive into the next request. The
leaked `AbortProtect` depth lets evaluation continue past `Abort[]`; the leaked
deadline terminates an unrelated later evaluation.

brief-402 (`fc959898`) landed the closed abandonment protocol:
`Session::abandon_pending_suspension` (`runtime/suspension/core.rs:484`), a
mandatory `FrameAbandon` classification on all 545 named frames, and
`finish_completed_resumption` (`:418`) for the completed path. It left the
legacy raw clear in place on purpose so the callers below still compile and
still leak. My gate confirmed the leak reproduces unchanged.

## The user's ruling this brief implements

**A new public evaluation request, arriving while a single-source input
continuation is pending, is rejected.** Supersede stays unimplemented rather
than remaining what it is today — an accident of consuming queued input. If an
explicit supersede is ever wanted it will be its own documented operation.

## Goal

Every live-session suspension discard travels through the abandonment
entrypoint; the raw-discard capability is gone and the repository gate proves
it; the public overlap hole is closed by a documented rejection; and a public
regression matrix proves each leaked owner in brief-401 §3 is clean while
`Monitor` and `ValueQ` keep their documented result contracts.

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md`,
  `docs/guide/embedding.md`, **brief-401's report sections 2, 3 and 4**, and
  **brief-402's committed report** before touching anything.
- The three raw callers, all still on the legacy API
  `clear_pending_suspension_state` (`runtime/suspension/core.rs:413`):
  - `runtime/builtins_input.rs:233` — discards an older owner before consuming
    a queued response;
  - `runtime/debug_tools/monitor.rs:115` and `:146` — helper best-effort
    discards.
- Checkpoint rollback: `runtime/session/session_helpers.rs:27,131,160`. The
  `had_pending_suspension: bool` is insufficient — brief-401 §4 shows it cannot
  distinguish the same owner from a replacement, nor recover an owner that
  `begin_input_suspension` overwrote. It must become a generation or token.
- Request lifecycle: `runtime/session/session_requests.rs`,
  `crates/rum-host/src/session.rs`, and `docs/guide/embedding.md:272-345`.
- **Read `docs/guide/embedding.md:335-338` carefully.** A script-shaped request
  that suspends is *already* documented as not resumable through
  `resume_with_input`. Under the new rejection, such a session accepts only
  `reset()`, read-only inspection, and whole-session replacement. That is the
  intended contract, not an oversight, and the guide must say so.
- Enforcement to extend: `runtime/tests/frame_root_trace.rs`, the
  `suspension_protocol_repository_invariants_hold` gate, and
  `runtime/tests/suspension_abandonment.rs` from brief-402.
- **Your sandbox has no network.** Pass `--offline` to every `cargo` command.

## Do the failing test first

Before any repair, write the public `rum-host` regression for the `Block` case
and **run it red**. Paste the failing output in your report. Only then repair.
A green test written after the fix proves less, and this line has already been
caught once closing the instances that were seen rather than the class.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/builtins_input.rs`,
    `runtime/debug_tools/monitor.rs`,
    `runtime/session/session_helpers.rs`,
    `runtime/session/session_requests.rs`;
  - `runtime/suspension/core.rs`, `runtime/model/session.rs`,
    `runtime/session/session_state.rs`, `runtime/kernel_api.rs` — to remove the
    legacy raw-clear surface, tighten ownership, and export what the request
    boundary needs;
  - `runtime/tests/frame_root_trace.rs`,
    `runtime/tests/suspension_abandonment.rs`, and focused new or existing
    tests under `crates/rum-runtime/src/runtime/tests/`;
  - `crates/rum-host/src/session.rs`, the `HostFailure`/failure-code types it
    needs, `crates/rum-host/src/tests/`, `crates/rum-host/tests/`;
  - `docs/guide/embedding.md`.
- Files/areas you MUST NOT touch:
  - `crates/rum-core` representation, `Cargo.lock`, dependency edges;
  - `rum-jupyter` — its two pump failures are briefs 404 and 405, not this one;
  - `HostSession::cancel_handle` and the cancellation lane — that is brief-404;
  - the generated builtin reference; ordinary evaluator, dispatch, or matcher
    logic;
  - any `FrameAbandon` classification landed by brief-402. If one is wrong, do
    not fix it here — **STOP and report it**, because a wrong classification is
    a brief-402 gate miss and I want to know that, not have it quietly patched.
- If the work needs a file outside this list, **STOP and report** rather than
  working around it. My MAY lists have been short six times on this project.

## The accept-vs-stop criterion

Making a discard structural is in scope. **Changing what a builtin returns is a
stop.** `Monitor` still ignores its helper's values, errors, and input;
`ValueQ` still returns its documented boolean; `eval_script`'s existing
non-resumable behavior is unchanged. Only state contamination changes. If
removing a leak appears to require changing a documented result, that is a
finding, not a licence — stop and report it.

## Deliverable

1. All three raw callers use `abandon_pending_suspension` with the right
   `AbandonReason`. `builtins_input.rs` must abandon an older live generation
   **before** consuming the queued value, never after and never both.
2. The legacy `clear_pending_suspension_state` is **deleted**, not deprecated,
   and the broadly visible owner field is gone. The repository invariant
   reports zero raw live-owner clears or replacements outside
   `suspension/core.rs`, and fails if one returns.
3. The checkpoint carries a suspension generation instead of a bool. Rollback
   order per brief-401 §4: abandon the trial-created generation, verify the
   slot is empty, then restore the snapshot. A pre-existing generation captured
   by the snapshot is preserved, never abandoned.
4. **The rejection.** A new public evaluation request arriving while a
   single-source input continuation is pending returns a documented
   `HostFailure` — not a panic, not a poison — and leaves the pending owner
   **untouched and still resumable**. Prove the old continuation still resumes
   after a rejection; that is the test that distinguishes rejection from a
   quiet discard.
5. **An entry-point inventory, as a deliverable in the report.** Enumerate
   every public `rum-host` entry point that can begin an evaluation, with
   `path:line`, and state for each whether it is guarded and why. Derive the
   list by tracing what reaches the runtime request seam — do not recall it and
   do not guess from names. `reset`, `restore_from_bytes` (it resets first),
   `resume_with_input`, `queue_input_response`, and read-only inspection must
   stay available; say for each why that is safe.
6. `docs/guide/embedding.md` documents the rejection, and documents that a
   script-shaped suspension leaves `reset()` as the escape.
7. A public regression matrix covering **every** owner in brief-401 §3:
   separate `Block`, `Module`, and `With` cases, plus attributes, iterator
   dynamic state, option scope, `Reap`, `Monitor`, trace, time and memory
   constraints, PRNG, `AbortProtect`, capture checkpoints, package rollback,
   and `WithCleanup`. Reuse brief-401's probe shapes and its verbatim rows as
   the before-picture.

## rum / Rust pitfalls (read twice)

1. Abandon helper-created state **before** restoring a rollback snapshot, or
   the snapshot restores over state the frames still own.
2. Never abandon a generation the snapshot captured from before the trial.
3. Do not call `finish_monitor`; abandonment pops without refreshing.
4. Do not consume a queued response while discarding an older owner — abandon
   first, then consume.
5. Preserve single-source `resume_with_input`, script pre-queue semantics,
   `reset`, panic poisoning, and structured outputs exactly.
6. Everything here is request and suspension cold path. No ordinary
   evaluator-step branch, allocation, or lock.
7. No new `unsafe`.

## Verification (MANDATORY — you run it; iterate to green)

Pass `--offline` to every `cargo` command.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
```

Use `--no-fail-fast` on the workspace run: `cargo test` stops after a failing
binary and silently under-reports the rest. Paste real counts.

**Known-failing, not yours:** `rum-jupyter`'s
`tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle`
and `tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately`
fail on the committed tree — brief-397 findings B and A, scheduled as briefs
404 and 405. Report them; do not fix them. Any *other* new failure is yours.

Then the **flipped leak probe**. brief-402's gate item was that this probe
still leaks. Now it must not. Build a throwaway `/tmp` crate on `rum-host`:

```rust
let mut s = HostSession::new();
s.eval("x = \"outside\"");
s.eval("Block[{x = \"inside\"}, InputString[\"old\"]]");   // suspends
s.queue_input_response("drain");
s.eval("\"drain\"");                                        // discards the owner
s.eval("x");                                                // must now be "outside"
```

Paste the four verbatim rows. `after` must read `"outside"`. Remove the crate
afterwards.

No timing A/B: request and suspension paths only. If you change ordinary eval,
dispatch, or matcher code, the cost premise is exceeded — stop and report.

## Hard rules

- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  **Do not touch the brief-400 stash.** Leave changes in the working tree.
- Modify only Scope; if you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except the `/tmp` probe crate this brief
  grants; remove it when done. No network, no probe artifacts in the repo.
- **Testing Policy is a hard line**: rum's own documented behavior only. No
  vendor engine, oracle harness, compatibility matrix, or third-party output.
- **No new `unsafe`.**
- Minimal diff; no drive-by reformatting or renames.
- Paste genuine verbatim output. If a run did not reach green, say so.

## Report format

1. Per-caller migration: the reason passed, and where the abandonment sits
   relative to queue consumption and snapshot restore.
2. The entry-point inventory from Deliverable 5.
3. The red test, verbatim, before the repair; then the same test green.
4. The public regression matrix, verbatim.
5. Verification output including the flipped leak probe and the repository
   invariant rows.
6. Cost statement.
7. Stop-loss, risks, open questions.
