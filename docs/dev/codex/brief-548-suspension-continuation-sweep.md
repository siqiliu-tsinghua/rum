# brief-548 — design: find every suspension path that drops its continuation

`Scope: NONE — no production change.` Design-only, with one declared scratch
exception for probes (see Scope). Census the defect class, probe it, and return
a repair plan, an enforcement design and drafted implementation briefs. **Do
not run `git` in any form**, including while first loading this repository.

## Goal

When guest code requests input (`InputString`, `Input`, `Dialog`), evaluation
suspends with `EvalError::InputRequired`, and `resume_eval_request_with_input`
later continues it. A builtin that suspends in the middle of its work must
resume **into** that work. Two builtins do not: they resume *past* it, and
complete with the value of whatever guest call happened to suspend.

Find every place this happens, classify each, and design the repair together
with a structure that keeps the class closed. This is a correctness branch
inserted into the retention line; **it builds no leases**. Correctness comes
first, as in brief-530: converting a path whose continuation is broken would
build accounting on a defect.

## The mechanism, confirmed by the director from source

- As `InputRequired` propagates outward, each Rust function that wants to be
  resumed pushes a frame: `resume_or_suspend!`
  (`crates/rum-runtime/src/runtime/suspension/core.rs:784-801`) calls
  `insert_suspension_frame` (`:515`), which appends to the pending queue.
- On resume, `drive_resuming_frames` (`core.rs:709`) drives the queue from
  the innermost frame outward. Each frame receives the inner outcome and
  returns a `FrameAction`; `Complete(outcome)` hands that outcome to the next
  frame out, as if it were the result of the function that pushed the next
  frame.
- So a Rust function with work remaining after a may-suspend call must push a
  frame that performs that work on resume. If it pushes none — it propagates
  with a plain `?` — or pushes one whose `resume` just completes with the inner
  value, **the remaining work is skipped and the inner value flows outward in
  its place.** When the next frame out is the generic
  `RunCallDispatchStageSuspensionFrame`
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:466-492`), the whole
  builtin call completes with the inner value.
- brief-530 found this once already, on the option loop, through a plain `?`
  (`docs/dev/codex/brief-530-option-suspension-continuation.md`). It was
  repaired there and nowhere else, because nobody swept for the class.

## Two confirmed instances, and four controls

The director ran each source below twice — suspended then resumed with input
`"two"`, and again with the same input queued up front
(`Session::queue_input_response`,
`crates/rum-runtime/src/runtime/session/session_api.rs:286`). The callback is
`f = Function[x, (AppendTo[log, x]; If[SameQ[x, 2], InputString["p: "], x])]`
with `log = {}` beforehand. The probe source is
`.temp/director/suspend-probe.rs`, its raw output
`.temp/director/suspend-probe.txt`.

| Probe | Resumed value | Resumed `log` | Queued value | Queued `log` |
| --- | --- | --- | --- | --- |
| `Scan[f, {1, 2, 3}]` | `"two"` | `{1, 2}` | `Null` | `{1, 2, 3}` |
| `Normal[LazyTake[LazyMap[f, LazyRange[1, 1]], 3]]` | `"two"` | `{1, 2}` | `{1, "two", 3}` | `{1, 2, 3}` |
| `Do[f[x], {x, {1, 2, 3}}]` | `Null` | `{1, 2, 3}` | `Null` | `{1, 2, 3}` |
| `Select` with a suspending predicate | `{1, 2, 3}` | `{1, 2, 3}` | `{1, 2, 3}` | `{1, 2, 3}` |
| `NestList` with a suspending function | `{1, 2, 3, 4}` | `{1, 2, 3}` | `{1, 2, 3, 4}` | `{1, 2, 3}` |
| `Table` with a suspending body | `{1, "two", 3}` | `{1, 2, 3}` | `{1, "two", 3}` | `{1, 2, 3}` |

- **Scan**: `evaluate_scan_parts`
  (`crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:446-479`)
  suspends inside its loop with `EvaluateScanPartsSuspensionFrame`, a frame
  with no fields whose `resume` completes with the inner value. The loop, the
  remaining parts and the final `Null` are lost.
- **LazyMap**: the pull stepper propagates with a plain `?`
  (`crates/rum-stdlib/src/lazy.rs:163-179`, the `NestList`, `Map` and `Select`
  arms), so the realization loop above it is lost.

Those are the two shapes this brief is about: **a frame that does not carry the
remaining work, and no frame at all.** They are not the only possible shapes;
see pitfalls 5-7.

## The oracle

For any source, the suspended-then-resumed path must produce **the same value
and the same side effects** as the queued-input path. That is rum compared with
rum — the property the resume API promises — and is not the Testing Policy's
banned shape. A probe proves nothing unless it also asserts that the suspension
actually happened (`SessionRequestError::InputRequired`); a source that never
suspends passes trivially.

## The population

Director counts from `grep`, over `crates/rum-runtime/src` and
`crates/rum-stdlib/src` excluding tests — recount them, they are regex-derived:

- 324 direct `.evaluate(` call sites;
- 570 `resume_or_suspend!` uses and 370 `SuspensionFrame` impls;
- 85 frame structs with no fields (`.temp/director/unit-frames.txt`); a
  fieldless frame is correct only where no work remains after the suspension
  point. `EvalMeanSuspensionFrame` (`rum-stdlib/src/arithmetic.rs:183-189`) and
  `AveragePairSuspensionFrame` (`arithmetic/stats.rs:239-245`) are at tail
  positions, as far as the director read;
- 9 single-line `.evaluate(...)?` sites (`.temp/director/evaluate-question.txt`),
  which undercounts: a multi-line `?`, a helper that evaluates on the caller's
  behalf, and `match … { Err(e) => return Err(e) }` all propagate the same way.

The unit of the census is a **(function, may-suspend call) pair**, where a
may-suspend call is `Session::evaluate` or any function that can return
`InputRequired` because it transitively evaluates. For each pair: is there work
after the call in this function — loop continuation, post-processing, wrapping
the result, cleanup, restoring state — and does the frame pushed there (or its
absence) carry it? Then the same question one level up, for every caller that
receives the propagated `InputRequired`.

Also check `rum-host`, `rum-jupyter` and any other crate that calls into
evaluation, and say whether any of them can hold remaining work across a
suspension.

## What I want back

### 1. The census

Every pair, TSV under `.temp/brief-548/`, with one verdict:

- **`TailCorrect`** — no work remains after the call, in this function or in a
  caller that pushes no frame. Cite the lines that show it.
- **`FrameCarries`** — the frame pushed here performs exactly the remaining
  work on resume. Cite the `resume` and the state it carries.
- **`Lost`** — remaining work is skipped. Say exactly what is lost (the loop,
  the post-processing, a restore, a message), and which user-visible builtins
  reach this path.
- **`Transformed`** — `InputRequired` is converted into another error or
  swallowed (for example `map_err(SchemaAbort::Eval)`,
  `crates/rum-stdlib/src/schema.rs:246`). Say what the user sees.
- **`Unsettled`** — with what you tried and what evidence is missing.

Aggregate by verdict, by crate and by builtin.

### 2. Probes

A probe, through the oracle, for **every `Lost` verdict** that source code can
reach, and for a sample of at least twenty `TailCorrect`/`FrameCarries` pairs
chosen to include every fieldless frame you judged correct at a non-obvious
position. A `Lost` verdict you cannot reach from source says so, with the
evidence. Paste the probe table verbatim, in the same shape as the one above.

### 3. Root causes

Group the `Lost` sites by shared mechanism — a helper, a driver, a macro, a
frame type reused out of place — so the repairs can be cut along those
boundaries rather than one site at a time.

### 4. Enforcement — keeping the class closed

This defect has now been repaired once and has recurred, because nothing stops
the next `?`. Design what the compiler or a required test can enforce. At least
consider:

- a result type for may-suspend calls whose suspension cannot be propagated by
  `?` without an explicit disposition (push a frame, or declare the position
  tail), so each of the call sites above must choose;
- a required test that drives the oracle over the installed catalog —
  suspending at a middle callback position of every builtin that takes a
  callback, and comparing with the queued path — as the falsifier for a
  disposition that is declared but false;
- anything better you find.

For each: its reach, what it cannot catch (a tail declaration that is false is
the obvious gap), and its default-path risk — `Session::evaluate` is the
evaluator's most-consulted function, and this project has measured 2-7%
regressions from changing its return shape (the `CLAUDE.md` note on the hot
recursive frame). Say what would have to be measured, with which labels; do
not measure it here. A mechanism that compiles but can be trivially bypassed is
worse than an honest list, because it looks enforced. Size is not an argument
against a mechanism.

### 5. The repair plan

Ordered steps along the root-cause groups, each with the `Lost` count predicted
after it. The frames must derive `FrameRootTrace` (a missing traversal is a
compile error by design) and preserve the existing suspend, resume and abandon
contract. State how each repair meets the retention line: R01 converts Scan,
R02 converts the callback-history builtins, and their frames will later carry
leases — this branch builds frames that carry state, not leases.

### 6. Drafted implementation briefs

In `TEMPLATE.md` shape. Each Scope states **an objective, a fence and a
disclosure duty**, not a list of permitted files. Every repair brief begins
with its defects as red tests through the oracle, with the pre-fix value
recorded verbatim. Scratch and checkpoints under `.temp/brief-NNN/`. The
stop-loss is keyed on progress (`TEMPLATE.md` pitfall 7).

### 7. Open questions for the director

## Settled — do not reopen

- Overlapping public requests are rejected, not superseded (brief-411). Out of
  scope.
- This branch restores correctness and builds no leases.
- The oracle is rum against rum, and is admissible.
- `FrameRootTrace` on every frame is compiler-enforced and stays so.
- The retention census (brief-547) is not reopened here.

## Context pointers

- `AGENTS.md` first.
- `crates/rum-runtime/src/runtime/suspension/core.rs` — the queue, the
  driver, the macro. `suspension/*.rs` — the shared frame machinery.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:400-492` — the generic
  dispatch frame.
- `docs/dev/codex/brief-530-option-suspension-continuation.md` and
  `brief-531-option-consumer-reentry.md` — the previous instance and its
  repair, including the typed re-entry descriptor that made the wrong shape
  fail to compile.
- Existing resume tests, as models for the probe shape:
  `crates/rum-stdlib/src/tests/incidental_session_requests_input_resume.rs`,
  `incidental_session_requests_transforms_resume.rs`,
  `incidental_session_requests_matcher_resume.rs`,
  `crates/rum-runtime/src/runtime/tests/session_requests/`.
- `.temp/director/suspend-probe.rs` — the director's probe, a working harness
  to start from.

## Scope

- Files you MAY modify: **NONE**, with three exceptions:
  - append your report to **this** file;
  - write scratch under `.temp/brief-548/`;
  - create **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief548_probe.rs`, for the probes. The
    `crates/rum-stdlib/tests/` directory does not exist today. **Before you
    append the report, copy the file's final content to
    `.temp/brief-548/probe.rs`, then delete the file and the directory**, and
    say in the report that you did.
- Everything else is read-only. No production, test, doc or `Cargo.toml`
  change.

## rum / Rust pitfalls (read twice)

1. **A probe that did not suspend proves nothing.** Assert `InputRequired`, then
   compare with the queued path.
2. **Suspend in the middle.** Loss usually shows only when the suspending call
   is not the last one and work follows it. Suspend at a middle element, not
   the first or last.
3. **Side effects are part of the answer.** Record every callback invocation
   (the `AppendTo[log, …]` pattern) and compare the logs as well as the values.
4. **Suspend twice.** The resumed path can suspend again
   (`drive_resuming_frames`'s nested branch, `core.rs:748`). For every
   loop you repair, a probe that suspends at two positions belongs in the red
   tests.
5. **A frame with fields can still lose work.** Carrying the cursor but not
   the accumulated prefix, or not the post-processing step, is the same
   defect.
6. **The loss can sit in a caller.** A correct frame at the call site does not
   help if a caller between it and the dispatch boundary has remaining work and
   pushes nothing.
7. **Error transformation is its own verdict.** Matching `Err(_)`, `map_err`,
   or `ok()` can turn a suspension into a different error or silently drop it.
8. **A count derived by regex is suspect.** Cross-check against source reads.

## Verification (MANDATORY — the evidence IS the deliverable)

- State every command; persist raw output under `.temp/brief-548/` before
  parsing it.
- Build and run probes only with
  `python3 scripts/run-bounded.py --timeout <seconds> -- cargo test -p rum-stdlib --test zz_brief548_probe -- --nocapture`,
  with the timeout derived from your first build's measured duration.
- Paste: the census counts by verdict; every `Lost` row; the probe table; the
  root-cause groups.
- State that the scratch test file and directory were deleted, and that
  `.temp/brief-548/probe.rs` holds their final content.

## Checkpointing — protect the scene

After each milestone — population recounted, each crate censused, probes run,
root causes grouped, enforcement designed, each draft — write
`.temp/brief-548/PROGRESS.md` with what is done and verified, its saved
evidence, the exact next action, and whether the scratch test file currently
exists.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the three exceptions in Scope. If you believe
  you must, STOP and write that in the report instead.
- Touch nothing outside this repository. No network. Commands are limited to
  file reads, `rg`/`grep`, `python3`, edits to the one scratch test, and the
  bounded cargo command above.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** Editing a file under `src/` means you have left the
  brief.
- No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", the scratch artifacts, and the deletion of the
   scratch test.
2. Verification — commands, counts, the probe table.
3. The census by verdict, every `Lost` row, and the unsettled list.
4. Root-cause groups.
5. The enforcement design, with reach, gaps and default-path risk.
6. The repair plan with predicted `Lost` counts per step.
7. The drafted implementation briefs.
8. Open questions for the director.

---
