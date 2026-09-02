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

---

# Report — 2026-09-02

## 1. Per-caller migration

- Queued input: [builtins_input.rs:233](crates/rum-runtime/src/runtime/builtins_input.rs:233) calls `abandon_pending_suspension(SupersededOrQueuedInputDiscard)` before [removing the queued value](crates/rum-runtime/src/runtime/builtins_input.rs:238). It neither consumes first nor abandons twice.
- Monitor callback: [monitor.rs:116](crates/rum-runtime/src/runtime/debug_tools/monitor.rs:116) uses `HelperBestEffortDiscard`.
- Monitor display: [monitor.rs:149](crates/rum-runtime/src/runtime/debug_tools/monitor.rs:149) uses `HelperBestEffortDiscard`. Neither path calls `finish_monitor`.
- Checkpoint rollback: [session_helpers.rs:135](crates/rum-runtime/src/runtime/session/session_helpers.rs:135) now captures `Option<SuspensionGeneration>`. A trial-created generation is abandoned with `CheckpointRollbackDiscard`, the empty slot is asserted, and only then is the loader snapshot restored at [line 161](crates/rum-runtime/src/runtime/session/session_helpers.rs:161). A matching pre-existing generation is preserved.
- Ownership: [SuspensionSlot](crates/rum-runtime/src/runtime/suspension/core.rs:24) owns the continuation and generation counter. Install/take/restore mutations are private to `suspension/core.rs`; installing over a live owner asserts. `clear_pending_suspension_state` is deleted.

## 2. Public entry-point inventory

| Public entry point | Runtime seam | Disposition |
|---|---|---|
| [HostSession::eval](crates/rum-host/src/session.rs:325) | `eval_request`, guarded at [session_requests.rs:129](crates/rum-runtime/src/runtime/session/session_requests.rs:129) | Rejected before interrupt setup, parsing, lowering, or checkpoint creation. |
| [HostSession::call](crates/rum-host/src/session.rs:347) | Host preflight at [session.rs:415](crates/rum-host/src/session.rs:415), then guarded `eval_expr_request` at [session_requests.rs:148](crates/rum-runtime/src/runtime/session/session_requests.rs:148) | Rejected even when the supplied head or typed arguments are invalid; no conversion precedes rejection. |
| [HostSession::eval_script](crates/rum-host/src/session.rs:372) | Guarded at [session_requests.rs:274](crates/rum-runtime/src/runtime/session/session_requests.rs:274) | Rejected before script evaluation. |
| [HostSession::eval_file](crates/rum-host/src/session.rs:393) | Guarded at [session_requests.rs:293](crates/rum-runtime/src/runtime/session/session_requests.rs:293) | Rejected before file-provider access. |
| [HostSession::eval_with_timeout](crates/rum-host/src/session.rs:502) | Routes through guarded `eval_request` | Rejected; the temporary timeout wrapper is cleared afterward. |
| [HostSession::restore_from_bytes](crates/rum-host/src/session.rs:203) | Resets, then uses `eval_snapshot_restore_request` → guarded script request | Available because valid snapshot restoration destroys the old session before beginning evaluation. Invalid UTF-8 leaves the old session untouched. |

Safe non-new-evaluation operations:

- [reset](crates/rum-host/src/session.rs:174) replaces the whole session, so destruction of the old owner and its contaminated state is safe.
- [resume_with_input](crates/rum-host/src/session.rs:540) consumes the exact pending request checkpoint and owner rather than beginning unrelated evaluation.
- [queue_input_response](crates/rum-host/src/session.rs:531) only queues future input; it does not resume, supersede, or unlock a new request.
- Snapshot, parse/lower, definitions, message catalog, contexts, inspection, completion, completeness, and pending-input inspection do not reach an evaluation request seam.
- A script-shaped suspension remains non-resumable. Evaluation is rejected until [reset or whole-session restoration](docs/guide/embedding.md:345); inspection remains available.

The rejection is `HostFailureCode::Other`, does not require reset for an ordinary resumable continuation, and preserves that continuation.

## 3. Red first, then green

Initial red run, before the repair:

```text
   Compiling rum-host v0.2.0 (crates/rum-host)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.43s
     Running unittests src/lib.rs (target/debug/deps/rum_host-6b9bfeb214af5253)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s

     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-61fe524a67fb26d2)

running 1 test

thread 'public_contract_rejects_overlapping_eval_and_preserves_block_suspension' panicked at crates/rum-host/tests/adapter_contract.rs:96:18:
expected overlapping-evaluation rejection, got Success(HostEvalSuccess { value: HostValue { text: "\"replacement\"", typed: Str("replacement") }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: "\"replacement\"" }] })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test public_contract_rejects_overlapping_eval_and_preserves_block_suspension ... FAILED

failures:

failures:
    public_contract_rejects_overlapping_eval_and_preserves_block_suspension

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-host --test adapter_contract`
```

Final green target:

```text
running 1 test
test public_contract_rejects_overlapping_eval_and_preserves_block_suspension ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

The test verifies the rejection, unchanged pending prompt, successful original continuation resume, and restoration of the outer `Block` binding.

## 4. Public regression matrix

Before-picture rows reused from brief-401:

```text
queue.block.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
queue.block.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=0)
queue.block.after=Success(value="inside", termination=None, messages=0, prints=[], trace=0)
queue.module.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
queue.module.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=0)
queue.module.names_after=Success(value=List["Global`queueModule401$1"], termination=None, messages=0, prints=[], trace=0)
queue.with.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
queue.with.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=0)
queue.with.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)

valueq.block.eval=Success(value=True, termination=None, messages=0, prints=[], trace=0)
valueq.block.after=Success(value=List["outside", 7], termination=None, messages=0, prints=[], trace=0)
valueq.block.after_clear=Success(value=List[valueQBlock401, 7], termination=None, messages=0, prints=[], trace=0)
valueq.module.eval=Success(value=True, termination=None, messages=0, prints=[], trace=0)
valueq.module.names_after=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
valueq.with.eval=Success(value=True, termination=None, messages=0, prints=[], trace=0)
valueq.with.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)

inventory.attributes.after=Success(value=List[Protected], termination=None, messages=0, prints=[], trace=0)
inventory.reap.memory_before=Success(value=67655, termination=None, messages=0, prints=[], trace=0)
inventory.reap.memory_after=Success(value=99773, termination=None, messages=0, prints=[], trace=0)
inventory.monitor.after=Success(value=1, termination=None, messages=0, prints=["1"], trace=0)
inventory.options.after=Success(value="explicit", termination=None, messages=0, prints=[], trace=0)
inventory.trace.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=1)
inventory.trace.after=Success(value=2, termination=None, messages=0, prints=[], trace=1)
inventory.time.after=Success(value=TerminatedEvaluation["TimeConstraint"], termination=Some(TimeConstraint), messages=1, prints=[], trace=0)
inventory.random.control=Success(value=488421, termination=None, messages=0, prints=[], trace=0)
inventory.random.after=Success(value=913882, termination=None, messages=0, prints=[], trace=0)
inventory.abort_protect.after=Success(value="continued", termination=None, messages=0, prints=[], trace=0)
inventory.cleanup.after=Success(value="not-run", termination=None, messages=0, prints=[], trace=0)
```

Current public-host matrix, verbatim:

```text
running 3 tests
test public_monitor_matrix_restores_capture_package_and_cleanup_owners ... matrix.capture.evaluation_data=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.capture.check=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
checkFallback403=Success(value=0, termination=None, messages=0, prints=[], trace=0)
matrix.capture.quiet=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.capture.test_report=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.package.monitor=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
Names["AbandonPublic403`*"]=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
matrix.with_cleanup.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
matrix.monitor.contract=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
ok
test public_monitor_matrix_restores_reap_monitor_trace_constraints_random_and_abort_state ... matrix.reap.monitor=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.reap.memory=before:67384,after:68216,growth:832
matrix.monitor.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
matrix.monitor.refreshes=abandoned:0,completed:4
matrix.trace.monitor=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.trace.after=Success(value=2, termination=None, messages=0, prints=[], trace=0)
matrix.time.monitor=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.time.after=Success(value="done", termination=None, messages=0, prints=[], trace=0)
matrix.memory.monitor=Success(value=20000, termination=None, messages=0, prints=[], trace=0)
matrix.random.monitor=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.random.after=Success(value=934555, termination=None, messages=0, prints=[], trace=0)
matrix.abort_protect.monitor=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
matrix.abort_protect.abort=Success(value=$Aborted, termination=None, messages=0, prints=[], trace=0)
abortAfter403=Success(value=0, termination=None, messages=0, prints=[], trace=0)
ok
test public_rollback_matrix_restores_lexical_iterator_attribute_and_option_state ... matrix.block.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
block403=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
matrix.module.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
Names["Global`moduleLocal403$*"]=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
matrix.with.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
with403=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
matrix.attributes.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
Attributes[attribute403]=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
matrix.iterator.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
iterator403=Success(value=iterator403, termination=None, messages=0, prints=[], trace=0)
matrix.option.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
matrix.option.after=Success(value=OptionValue[choice403], termination=None, messages=1, prints=[], trace=0)
matrix.valueq.contract=Success(value=True, termination=None, messages=0, prints=[], trace=0)
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

This covers Block, Module, With, attributes, iterator/product scope, options, Reap, Monitor, trace, time/memory constraints, PRNG, AbortProtect, capture families, package rollback, and WithCleanup. `ValueQ` remains `True`; `Monitor` remains `Null`.

## 5. Verification

Final mandatory task-owned gates:

```text
scripts/run-bounded.py --timeout 300 -- cargo --offline fmt --check
exit 0; no output

scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
test result: ok. 922 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out

scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-host
lib:                    87 passed, 0 failed
adapter_contract:        7 passed, 0 failed
installer:               4 passed, 0 failed
lifecycle_buildout:      7 passed, 0 failed
lifecycle_contract:     13 passed, 0 failed
random_password_policy:  2 passed, 0 failed
stress:                   4 passed, 0 failed, 2 ignored
abandonment matrix:       3 passed, 0 failed
doctest:                  1 passed, 0 failed
```

The final workspace command:

```text
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast
error: 1 target failed:
    `-p rum-jupyter --lib`
```

Both failures are exactly the brief’s declared known failures:

```text
thread 'tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle' panicked at crates/rum-jupyter/src/tests/pump.rs:250:9:
interrupted execute did not produce a shell reply within 2s

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 102 filtered out; finished in 2.00s
```

```text
thread 'tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately' panicked at crates/rum-jupyter/src/tests/pump.rs:37:5:
assertion failed: pump.pump_execute_completion(&mut transport).expect("completion poll should succeed")

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 102 filtered out; finished in 0.00s
```

No other workspace target failed. `rum-jupyter` was not modified.

Repository invariant rows:

```text
named_suspension_frames=545
named_pure_finalizers=60
pure_frame_abandon_derives=516
handwritten_stateful_frame_abandons=29
evaluate_cleanup_allowlist=WithCleanup owner frames (2)
raw_live_owner_clears=0
raw_live_owner_replacements=0
legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 923 filtered out
```

Flipped probe:

```text
queue.block.setup=Success(value="outside")
queue.block.suspend=InputRequired(prompt="old")
queue.block.drain=Failure(code=Other, message="cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first")
queue.block.after=Success(value="outside")
```

The brief’s literal probe has an internal contradiction: after the mandated rejection, its immediately following `eval("x")` must also be rejected because the owner must remain installed and resumable. To obtain the required `after="outside"` row without violating that contract, the probe performed one unprinted `resume_with_input("answer")` between `drain` and `after`. The temporary `/tmp` crate was removed.

An intermediate runtime run exposed two old white-box GC tests that deliberately began a second public request; they were updated to use internal `eval_text` for their sweep trigger. An intermediate format check also caught two rustfmt-only wraps. Both final focused gates are green.

## 6. Cost statement

All production changes are confined to request and suspension cold paths. No ordinary evaluator, dispatch, matcher, expression construction, or default-path instrumentation was changed. No timing A/B was run, as required. No `unsafe`, dependency, or `Cargo.lock` changes were introduced.

## 7. Stop-loss, risks, open questions

- The only unresolved issue is the contradictory literal flipped-probe sequence described above; the implementation follows the higher-priority rejection-and-preserve contract.
- The two workspace failures are the predeclared briefs 404/405 `rum-jupyter` failures.
- No `FrameAbandon` classification was changed.
- No out-of-scope evaluator, cancellation, Jupyter, generated-reference, or dependency work was performed.
- Changes remain uncommitted in the working tree, and the brief-400 stash was untouched.
