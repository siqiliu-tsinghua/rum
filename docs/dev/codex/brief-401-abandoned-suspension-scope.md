# brief-401 — diagnose: does abandoning a suspension leave dynamic scope installed?

**Scope: NONE — diagnosis and plan only, no source change.** Reading, grepping
and throwaway probes are in scope; revert any temporary instrumentation before
reporting and name it.

**Write your findings before running any verification commands.**

## Where this came from

brief-400 stopped on a blocker rather than exceeding its scope, which was the
right call. While writing a deterministic test for cancellation across input
suspension, it found that cancelling a suspended evaluation cleared
`pending_suspension` and dropped the saved continuation frames **without
driving their error cleanup**, so a `Block`'s dynamic binding stayed installed
after the request ended:

```
left:  "List[8, inside]"
right: "List[8, outside]"
```

Its diagnosis: correct abandonment must resume the saved frame driver with
`Err(EvalError::Cancelled)` so every continuation frame unwinds its own scope,
while no host input is converted or delivered. The driver and the private frame
queue live in `crates/rum-runtime/src/runtime/suspension/core.rs`, outside that
brief's scope, so it stopped and asked for an amendment.

brief-400's work is stashed; you are working against the committed tree.

## The question this brief must answer first

I told the user this looks like a **pre-existing** defect rather than one
brief-400 introduced, and split it out on that basis. **That is a hypothesis,
not an established fact, and your first job is to settle it.**

`clear_pending_suspension_state` is committed code
(`crates/rum-runtime/src/runtime/suspension/core.rs:159-161`), and committed
callers drop a pending suspension on paths that do not obviously unwind frames:

- `crates/rum-runtime/src/runtime/builtins_input.rs:233`
- `crates/rum-runtime/src/runtime/suspension/core.rs:165`
- `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:115` and `:146`, both
  on `Err(EvalError::InputRequired)`
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:160-161`, in the
  request checkpoint rollback (`if !snapshot.had_pending_suspension { … = None }`)

So:

1. **Is the leak reachable on the committed tree, with no brief-400 code?**
   Build a probe from the ordinary public surface — a `Block` (and separately a
   `Module` and a `With`) whose body suspends on `InputString`, then reach each
   of the abandonment paths above that you can actually reach, and observe
   whether the dynamic binding survives the request. Report each path as
   reachable-and-leaking, reachable-and-clean, or unreachable-from-here, with
   `path:line` and the probe.
2. If it is **not** reachable today, say so plainly. That makes it a defect in
   brief-400's new cancellation branch, the split was wrong, and the repair
   belongs back inside brief-400. I would rather learn that now than build a
   line on a bad premise.
3. Note the standing-debt entry this brushes against: `TODO.md` records that
   "`Monitor` refresh and `ValueQ` rollback remain best-effort unless their
   documented contract is deliberately promoted." Say whether what you find in
   `monitor.rs` is that documented best-effort limitation or something worse,
   and do not silently promote a contract.

## What else to establish

4. **What exactly leaks?** Dynamic scope (`Block`) is the observed case. Check
   the other state a dropped frame stack can own: `Module`/`With` gensym
   bindings and their reclamation, `$Context` and context-path changes, open
   `Reap` buckets, `Monitor` refresh state, attribute or `Protected` changes
   made inside the abandoned body, and anything else the frame inventory says a
   frame can hold. The suspension-roots line built
   `SuspensionFrame: FrameRootTrace` and a repository invariant test listing
   every handwritten traversal — use that inventory as the checklist rather
   than guessing, and say which frames own unwind-relevant state.
5. **Is the same shape reachable through cancellation as it exists today**,
   without the accepted-request work? The host has `cancel_handle()` and the
   runtime has an interrupt flag. If a cancel that lands during a suspension can
   reach any of the abandonment paths, that is a live defect on the current
   published surface, and it should be described as one.
6. **What is the right repair, and where does it live?** brief-400 proposed a
   runtime-private entrypoint in `suspension/core.rs` that drives the saved
   frames with `Err(EvalError::Cancelled)`. Confirm or improve that. State what
   the entrypoint's contract is, what it must guarantee about not running
   ordinary continuation effects while unwinding, how re-entrancy and nested
   suspension are handled, and how it interacts with the checkpoint rollback in
   `session_helpers.rs`. **Prefer a shape the type system enforces** — a frame
   that can be abandoned only through the unwinding entrypoint beats a rule that
   callers must remember.

## Deliverable

- The reachability verdict for each committed abandonment path, with probes.
- The leaked-state inventory from question 4.
- The repair design, with the entrypoint contract.
- A step-by-step plan, each step independently gateable, naming files, gate
  tests, and what must not break. Say which step brief-400 needs in order to
  resume, since its scope amendment depends on this.
- **Drafts of the follow-up implementation briefs** in `TEMPLATE.md` shape.
- Risks, open questions, and corrections to the background above.

## Verification and cost

No source change, so no test gate. If your plan's repair would touch the
evaluation hot path, say which part and why; the abandonment path itself is a
cold error path and a repair confined to it needs **no timing A/B** — say so
explicitly if that is the case, rather than performing the ceremony.

Where you do need to show a cost claim, prefer the env-gated allocation
counters over timings: they are load-immune and answer "did the ordinary path
gain work" deterministically. This host's same-binary microbench spread was
measured at 0.05%-18.75% per label even pinned to P-cores, so a timing A/B here
could not resolve anything a counter would not answer better.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Do not `git stash`,
  `git checkout`, or otherwise disturb the working tree — **there is a stash
  holding brief-400's work in progress; leave it alone.**
- Preserve the pre-existing worktree changes (five modified files under
  `scripts/` and `.claude/`, plus untracked `scripts/peak-rss.py`).
- **No source change.** Temporary instrumentation only if reverted before
  reporting and named; verify with `git status` and paste it.
- Do not modify `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` outside this
  file, or anything under `docs/dev/codex/` except appending your report.
- Leave no probe files in the repository; rum sources you generate are `*.rum`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Use `scripts/run-bounded.py --timeout SECONDS --` for long or child-spawning
  commands.
- **Stop-loss**: if you cannot settle question 1 after two serious attempts,
  stop and report what you ruled out. A precise negative is a good result.
- Paste real verbatim output for every claim.

## Report format

1. The question-1 verdict: reachable on the committed tree, or not.
2. Per-path reachability table with probes.
3. The leaked-state inventory.
4. The repair design and entrypoint contract.
5. The plan and the drafted implementation briefs.
6. Stop-loss, risks, open questions, corrections to my background.

---

# Report — 2026-09-02

## 1. The question-1 verdict

**Verdict: the pre-existing-defect hypothesis is established on the committed
tree.** Ordinary public `rum-host` calls can cause a live input continuation to
be discarded by committed code. `Block` state and `Module` activation/gensym
state then survive into later requests. `With` reaches the same discard sites
but is clean because it installs no session-local binding: it carries lexical
substitution data only (`crates/rum-runtime/src/runtime/scoping/lexical/with.rs:96-160`).

The public reachability hinge is not a private test hook. `HostSession::eval`
reports input suspension and says it should be resumed
(`crates/rum-host/src/session.rs:318-332`), but neither that method nor
`Session::eval_request` rejects a new evaluation while one is pending
(`crates/rum-runtime/src/runtime/session/session_requests.rs:108-134`). Public
`queue_input_response` also clears only `pending_input_request`, not the saved
continuation (`crates/rum-runtime/src/runtime/session/session_api.rs:213-216`).
Consequently, a new public `eval("InputString[...]")` can consume the queued
response and execute the raw clear at
`crates/rum-runtime/src/runtime/builtins_input.rs:231-239`.

All behavioral probes used only `rum_host::HostSession`,
`queue_input_response`, `resume_with_input`, and `cancel_handle`. The throwaway
Cargo harness lived at `/tmp/rum-brief401-probe`; no probe or instrumentation
was placed in the repository. The evidence command was:

```text
scripts/run-bounded.py --timeout 60 -- cargo run --offline --quiet --manifest-path /tmp/rum-brief401-probe/Cargo.toml
```

The shortest committed-tree reproducer and its verbatim result were:

```rust
let mut s = HostSession::new();
s.eval("queueBlock401 = \"outside\"");
s.eval("Block[{queueBlock401 = \"inside\"}, InputString[\"old\"]]");
s.queue_input_response("drain");
s.eval("InputString[\"new\"]");
s.eval("queueBlock401");
```

```text
queue.block.setup=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
queue.block.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
queue.block.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=0)
queue.block.after=Success(value="inside", termination=None, messages=0, prints=[], trace=0)
```

That contradicts the documented `Block` duration/restoration contract:
`docs/guide/scoping-and-contexts.md:23-42` says the rebinding lasts for the
body and the outer ownvalue remains afterward.

## 2. Per-path reachability table with probes

| Committed path | Public route | `Block` | `Module` | `With` | Verdict |
|---|---|---|---|---|---|
| `crates/rum-runtime/src/runtime/builtins_input.rs:231-234` | Suspend one `HostSession::eval`, queue a response, then call a new `eval` containing `InputString` | outer state not restored | activation and unescaped gensym not reclaimed | no installed state | **reachable-and-leaking** for `Block`/`Module`; reachable-and-clean for `With` |
| `crates/rum-runtime/src/runtime/suspension/core.rs:164-166`, called at `:216` | Resume with input, then make the continuation fail with an uncaught `Throw` | restored | reclaimed | clean | **reachable-and-clean**; this is completion after the frame driver has emptied the queue, not abandonment |
| `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:112-117` | A public `Monitor` `StepMonitor :>` callback evaluates a suspending scoped expression | outer state not restored | activation/gensym remains | no installed state | **reachable-and-leaking** for `Block`/`Module`; reachable-and-clean for `With` |
| `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:120-148` | A public `Monitor` display expression evaluates a suspending scoped expression | outer state not restored | activations/gensyms remain | no installed state | **reachable-and-leaking** for `Block`/`Module`; reachable-and-clean for `With` |
| `crates/rum-runtime/src/runtime/session/session_helpers.rs:135-162` | Public `ValueQ` trial evaluation suspends, then rollback sees that the snapshot had no pending continuation | loader snapshot masks the definition leak, but the dynamic stack is not in the snapshot | snapshot restores activations/loader state and no gensym remains | no installed state | **reachable-and-leaking** for dynamic iterator/`Block` bookkeeping; reachable-and-clean for observable `Module`/`With` state |

### Queued-input raw clear (`builtins_input.rs:233`)

The three probe bodies were respectively:

```text
Block[{queueBlock401 = "inside"}, InputString["old"]]
Module[{queueModule401 = "local"}, InputString["old"]]
With[{queueWith401 = "inside"}, {queueWith401, InputString["old"]}]
```

After each suspension the harness queued `"drain"`, evaluated a new
`InputString["new"]`, and then inspected state. The `Module` probe deliberately
did **not** call `Names` while the module was suspended, because namespace
inspection can mark active modules escaped
(`crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:36`). Verbatim:

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
```

The mechanism is direct: the queued branch calls
`clear_pending_suspension_state`; that method assigns both pending fields to
`None` (`suspension/core.rs:159-162`) without visiting the frames.

### Completed-error clear (`suspension/core.rs:164-166`, caller `:216`)

Each scoped body was
`CompoundExpression[InputString["core"], Throw["boom"]]`; the harness used
public `resume_with_input("answer")`. Verbatim:

```text
core.block.suspend=InputRequired(kind=InputString, prompt="core", messages=0, prints=[])
core.block.resume_error=Failure(code=Other, message="uncaught throw: \"boom\" with tag None", messages=1, prints=[])
core.block.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
core.module.suspend=InputRequired(kind=InputString, prompt="core", messages=0, prints=[])
core.module.resume_error=Failure(code=Other, message="uncaught throw: \"boom\" with tag None", messages=1, prints=[])
core.module.names_after=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
core.with.suspend=InputRequired(kind=InputString, prompt="core", messages=0, prints=[])
core.with.resume_error=Failure(code=Other, message="uncaught throw: \"boom\" with tag None", messages=1, prints=[])
core.with.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
```

`drive_resuming_frames` returns only when its front frame is absent
(`suspension/core.rs:222-253`); the non-input error branch clears storage only
after that return (`:200-216`). The `core.rs:165` item in the brief is therefore
not itself a committed abandonment path today. It should be renamed or guarded
to make its completed-only precondition obvious, but it must not be replaced by
a second unwind.

### `Monitor` display raw clear (`monitor.rs:146`)

The display expressions were a suspending `Block`, `Module`, and `With`; the
main body was `Null`. Verbatim:

```text
monitor_display.block.eval=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
monitor_display.block.after=Success(value="inside", termination=None, messages=0, prints=[], trace=0)
monitor_display.module.eval=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
monitor_display.module.names_after=Success(value=List["Global`monitorDisplayModule401$1", "Global`monitorDisplayModule401$2"], termination=None, messages=0, prints=[], trace=0)
monitor_display.with.eval=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
monitor_display.with.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
```

The two module names agree with the two display refresh sites: before the body
at `monitor.rs:43-52` and in `finish_monitor` at `:80-84`.

### `Monitor` callback raw clear (`monitor.rs:115`)

The calibration proves the delayed `StepMonitor` callback ran; the scoped
versions replace its increment with a suspending `Block`, `Module`, or `With`.
Verbatim:

```text
monitor_callback.calibration=Success(value=1, termination=None, messages=0, prints=["0"], trace=0)
monitor_callback.block.eval=Success(value=2, termination=None, messages=0, prints=["0"], trace=0)
monitor_callback.block.after=Success(value="inside", termination=None, messages=0, prints=[], trace=0)
monitor_callback.module.eval=Success(value=2, termination=None, messages=0, prints=["0"], trace=0)
monitor_callback.module.names_after=Success(value=List["Global`monitorCallbackModule401$1"], termination=None, messages=0, prints=[], trace=0)
monitor_callback.with.eval=Success(value=2, termination=None, messages=0, prints=["0"], trace=0)
monitor_callback.with.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
```

The standing debt at `TODO.md:823-825` and
`docs/runtime-evaluation-audit.md:163-164` permits `Monitor` refresh/callback
results to remain best-effort. Ignoring a helper's value, error, or input
request is that limitation. Leaving unrelated `Block`, `Module`, `Reap`,
option, or resource state installed after the helper returns is worse: it
violates those mechanisms' own documented duration and contaminates later
evaluation. The repair must keep `Monitor` best-effort—do not promote its
result/input contract—but must structurally abandon the helper continuation.
In particular, abandoning a monitor frame must pop the scope without calling
`finish_monitor`, because that method performs another ordinary display refresh
(`monitor.rs:80-84`).

### Checkpoint rollback (`session_helpers.rs:160-161`) through `ValueQ`

`ValueQ` captures a rollback snapshot, evaluates a trial, treats
`InputRequired` as evidence of a value, then restores the snapshot
(`messages/builtins_messages_inspect.rs:77-117`). The snapshot includes loader,
option, module, reap, monitor, resource, and abort state
(`session_helpers.rs:107-132`), but `EvaluationRollbackSnapshot` has no
`dynamic_own_values` field (`session_helpers.rs:6-28`). The probe nested a
suspending input under `Block` and a one-element `Table`, whose iterator frame
pushes a dynamic value (`branching_loops/iteration/driver.rs:403-420`).
Verbatim:

```text
valueq.block.eval=Success(value=True, termination=None, messages=0, prints=[], trace=0)
valueq.block.after=Success(value=List["outside", 7], termination=None, messages=0, prints=[], trace=0)
valueq.block.after_clear=Success(value=List[valueQBlock401, 7], termination=None, messages=0, prints=[], trace=0)
valueq.module.eval=Success(value=True, termination=None, messages=0, prints=[], trace=0)
valueq.module.names_after=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
valueq.with.eval=Success(value=True, termination=None, messages=0, prints=[], trace=0)
valueq.with.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
```

The restored loader snapshot makes the outer `Block` definition look clean,
and its leaked `None` shadow is semantically latent. The iterator's `7` makes
the missing dynamic-stack rollback observable. This violates `ValueQ`'s
documented “without committing side effects” result contract
(`docs/reference/builtins.md:6002-6005`), even though the broader trial remains
documented as best-effort.

### Current cancellation surface

`HostCancelHandle::cancel` only stores an atomic flag
(`crates/rum-host/src/session.rs:113-125`). A new top-level evaluation resets
that flag (`session_helpers.rs:31-39`); ordinary evaluation polls it only at
amortized evaluator checks (`eval_core/mod.rs:13-24,82-97`). Resume currently
converts the input and enters the saved driver without an up-front cancel check
(`session_requests.rs:185-212`; `suspension/core.rs:169-200`).

The direct continuation did not poll and accepted the input. A continuation
with a long `Do` polled later, after an ordinary post-input assignment, and its
`Cancelled` error then traversed the normal frame driver cleanly. Verbatim:

```text
cancel.direct.suspend=InputRequired(kind=InputString, prompt="cancel", messages=0, prints=[])
cancel.direct.resume=Success(value="delivered", termination=None, messages=0, prints=[], trace=0)
cancel.direct.after=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
cancel.polled.suspend=InputRequired(kind=InputString, prompt="cancel", messages=0, prints=[])
cancel.polled.resume=Success(value=TerminatedEvaluation["Cancelled"], termination=Some(Cancelled), messages=1, prints=[], trace=0)
cancel.polled.after=Success(value=List["outside", "yes"], termination=None, messages=0, prints=[], trace=0)
```

Thus the current published cancellation surface does **not by itself** reach a
drop-without-unwind path. It either is not observed by an input-only
continuation, or is observed later and unwinds normally. It is still deficient
for brief-400's accepted-request requirement: input was converted/delivered and
ordinary continuation effects (`"yes"`) ran before cancellation won. The new
brief-400 branch will need the abandonment entrypoint; current cancellation is
not an additional committed raw-drop caller.

## 3. The leaked-state inventory

The checklist source is the suspension-roots repository gate at
`crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:282-527`. It scans all
Rust sources in `rum-runtime` and `rum-stdlib`, requires every
`SuspensionFrame` to be a named `FrameRootTrace` type, confines queue mutation
to `suspension/core.rs`, and prints the total frame/finalizer inventory. The
formal gate was intentionally deferred until after this report, as required by
the brief; its verbatim count is appended below in the verification addendum.

The inventory divides into three classes.

### Frames that own unwind-relevant session state or an unwind obligation

| State/obligation | Owning frame family and normal cleanup | Required abandonment behavior |
|---|---|---|
| Saved continuation queue and active evaluation stack | `SuspendedEvaluation` owns `active_stack` and the private frame deque (`suspension/core.rs:10-13,74-121`) | empty the frame queue through the abandonment protocol; clear the paired input/request owner; never merely assign the option to `None` in a live session |
| `Block` dynamic frames and the complete localized symbol slice (definitions, attributes including `Protected`, options, syntax information, messages) | `SuspendedBlockInitializers`, `SuspendedBlockScope`, `SuspendedSingleBlockScope` (`scoping/lexical/block.rs:9-30`); cleanup truncates `dynamic_own_values` and restores saved symbol state (`:401-480,503-511`) | invoke the restoration primitive directly, without evaluating a continuation expression |
| Loop iterator dynamic/full symbol scopes | iterator state frames at `branching_loops/iteration/driver/resume_frames.rs:124-197`; cleanup at `driver.rs:424-432,483-505` and following | restore/truncate exactly once; do not continue the loop |
| `Module` activation plus gensym reclamation/candidate bookkeeping | activation installed at `scoping/lexical/module.rs:54-64`; frames at `:395-469`; cleanup/reclamation at `:306-350` | finish the activation with an error/no-result root set, so unescaped locals are reclaimed and activation depth is truncated; do not run remaining initializers/body |
| `OptionValue` dynamic scope | `EvaluateWithOptionValueScopeSuspensionFrame` (`options/patterns.rs:169-183`) | pop the option scope only |
| Open `Reap` scope and buckets | push at `debug_tools/reap.rs:56-80`, `EvalReapSuspensionFrame` at `:384-395`, pop at `:84-108` | pop/discard the scope; do not render buckets or evaluate a handler |
| Open `Monitor` scope | push at `debug_tools/monitor.rs:43-48`, frame at `:152-163`, normal pop at `:80-84` | pop without refresh/callback evaluation |
| Time and memory constraint nesting | installed at `debug_tools/resources/constraints.rs:55-72,119-135`; saved-prior-state frames at `:335-409`; normal restoration at `:225-275` | restore prior deadline/limit; never select/evaluate a fallback |
| Trace and debug capture state | `Trace` sets/restores `tracing_enabled` and trace checkpoints (`debug_tools/trace.rs:116-143,146-218`); `EvaluationData` does the same plus message/print/handled-error checkpoints (`debug_tools/evaluation_data.rs:15-52,56-125,204-240`) | restore tracing and truncate only the capture-owned suffix; do not render a trace or turn the abandonment error into an association value |
| `Check`, `Quiet`, `EvaluationData`, and test-report capture checkpoints | `kernel_data/control_data/check_quiet.rs:23-47,67-88,162-225`; `debug_tools/test_report/evaluation.rs:253-320,494-528` | restore/truncate capture bookkeeping according to the abandoned request; never evaluate `Check` fallback or synthesize a successful captured result |
| `AbortProtect` depth and delayed-abort checkpoint | installed at `kernel_data/control_data/abort_cleanup.rs:181-202`; restored at `:301-334`; frame at `abort_cleanup/resume_frames.rs:59-74` | decrement/restore the exact checkpoint without processing the abandoned body |
| Iteration product budget | saved in `WithIterationProductScopeSuspensionFrame` (`branching_loops/iteration/mod.rs:11-29,151-169`) | restore the parent budget |
| PRNG restoration (`BlockRandom`) | saved frame at `session/session_api.rs:575-591` and `session/session_api/resume_frames.rs:5-24` | restore the saved PRNG state |
| Partial package-install ownership | `ResumePackageInstallSuspensionFrame` owns the plan (`scoping/packages/resume_frames.rs:25-46`); failure cleanup clears minted sealed symbols/unit state (`scoping/packages.rs:476-497`) | run a non-reporting rollback variant; do not continue forms or emit an ordinary package-failure message solely because the host discarded work |
| Deferred `WithCleanup` cleanup | frames at `kernel_data/control_data/cleanup.rs:149-205`; normal abnormal-exit path evaluates cleanup at `:80-94` | preserve the documented cleanup obligation, but expose it as a uniquely allowlisted cleanup action rather than ordinary frame resumption |

The package, capture, and `WithCleanup` entries are important even though the
small public probe panel did not force every one: the frame inventory shows
that a “just fix `Block`” patch would leave other owners broken.

### Frames with only immutable computation state

`With` binding states, matcher snapshots/bindings, rewrite and tree-walker
worklists, call/definition argument accumulators, timing starts, and similar
frames retain roots needed to resume but install no corresponding session
scope. On abandonment they are discarded. Control frames such as catches,
confirmation handlers, `Check`, and `EvaluationData` can nevertheless run
ordinary work or transform errors during their normal `resume`; “no installed
state” does **not** mean it is safe to call their normal resume method while
abandoning.

### State changed by the body but not owned by a suspension frame

`Begin`/`End` mutate `current_context`, `context_path`, and `context_stack`
directly (`builtins_context_loader.rs:353-382`; `session/session_api.rs:705-759`).
There is no `Begin` suspension frame. An explicit `Begin` before an input
therefore persists after raw abandonment, but it also persists after an
ordinary uncaught error. Verbatim:

```text
inventory.context.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.context.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=0)
inventory.context.after=Success(value=List["InventoryLeak401`", List["InventoryLeak401`", "System`", "Global`"]], termination=None, messages=0, prints=[], trace=0)
inventory.context.normal_error=Failure(code=Other, message="uncaught throw: \"boom\" with tag None", messages=1, prints=[])
inventory.context.normal_error_after=Success(value=List["InventoryError401`", List["InventoryError401`", "System`", "Global`"]], termination=None, messages=0, prints=[], trace=0)
```

That is an ordinary committed effect requiring `End[]`, not structural state
the abandonment entrypoint may transactionally undo. The same distinction
applies to arbitrary definitions, `SetAttributes`, `Protected`, messages, and
other effects performed before suspension. When the changed symbol is a
`Block` or iterator-localized symbol, however, its saved frame explicitly owns
restoration, so the effect must be undone. The `Protected` probe is in that
owned category:

```text
inventory.attributes.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.attributes.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=0)
inventory.attributes.after=Success(value=List[Protected], termination=None, messages=0, prints=[], trace=0)
```

### Public evidence for the additional installed-state owners

The harness used the queued-input discard route for each case. It executed
`Sow[Range[1000]]` between the two `Reap` memory readings. Verbatim key rows:

```text
inventory.reap.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.reap.memory_before=Success(value=67655, termination=None, messages=0, prints=[], trace=0)
inventory.reap.memory_after=Success(value=99773, termination=None, messages=0, prints=[], trace=0)
inventory.monitor.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=["0"])
inventory.monitor.after=Success(value=1, termination=None, messages=0, prints=["1"], trace=0)
inventory.options.suspend=InputRequired(kind=InputString, prompt="", messages=0, prints=[])
inventory.options.after=Success(value="explicit", termination=None, messages=0, prints=[], trace=0)
inventory.trace.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.trace.drain=Success(value="drain", termination=None, messages=0, prints=[], trace=1)
inventory.trace.after=Success(value=2, termination=None, messages=0, prints=[], trace=1)
inventory.time.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.time.after=Success(value=TerminatedEvaluation["TimeConstraint"], termination=Some(TimeConstraint), messages=1, prints=[], trace=0)
inventory.random.control=Success(value=488421, termination=None, messages=0, prints=[], trace=0)
inventory.random.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.random.after=Success(value=913882, termination=None, messages=0, prints=[], trace=0)
inventory.abort_protect.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.abort_protect.after=Success(value="continued", termination=None, messages=0, prints=[], trace=0)
inventory.cleanup.suspend=InputRequired(kind=InputString, prompt="old", messages=0, prints=[])
inventory.cleanup.after=Success(value="not-run", termination=None, messages=0, prints=[], trace=0)
inventory.cleanup.normal_error=Failure(code=Other, message="uncaught throw: \"boom\" with tag None", messages=1, prints=[])
inventory.cleanup.normal_error_after=Success(value="ran", termination=None, messages=0, prints=[], trace=0)
```

Interpretation tied to source:

- `Sow` records into every matching open reap scope
  (`debug_tools/reap.rs:21-53`), so the retained bucket accounts for the
  post-`Sow` memory increase.
- The later print `"1"` comes from the leaked monitor scope refreshing in a
  later request (`monitor.rs:87-109`).
- `OptionValue` sees the leaked explicit scope (`options/patterns.rs:20-35`).
- Trace remains enabled because the frame that restores `prior_enabled` was
  dropped (`debug_tools/trace.rs:192-213`).
- The prior time deadline remains active because its saved-prior frame was
  dropped (`resources/constraints.rs:225-248`). A per-request memory limit is
  also frame-owned, although `begin_top_level_interrupt_budget` overwrites it
  at the next top-level request (`session_helpers.rs:31-36`), so its leak is
  most relevant inside `Monitor`/helper continuation.
- The seeded control says the next PRNG value should be `488421`; `913882`
  shows that `BlockRandom`'s saved state was not restored.
- The leaked `AbortProtect` depth lets evaluation continue past `Abort[]`.
- `WithCleanup`'s cleanup ran on a normal uncaught error but not when its frame
  was discarded. Its generated reference contract explicitly says cleanup is
  evaluated on abnormal exit (`docs/reference/builtins.md:6079-6082`).

## 4. Repair design and entrypoint contract

### Do not reuse the ordinary resume driver unchanged

The background's proposed `drive_resuming_frames(Err(EvalError::Cancelled))`
is directionally right about needing an unwind, but it is not a safe final
contract. The ordinary driver permits both `FrameAction::Evaluate`, which
calls `self.evaluate(expr)`, and arbitrary `resume` implementations
(`suspension/core.rs:222-253`). Several normal frames deliberately consume an
error or run work:

- `EvaluationData` converts most errors, including cancellation, into a
  successful association (`debug_tools/evaluation_data.rs:67-125`). An outer
  compound-expression frame could then execute statements after the suspended
  expression.
- `Check` emits a message and may evaluate its fallback on an error
  (`kernel_data/control_data/check_quiet.rs:91-129`).
- `Monitor`'s normal finish refreshes the display before popping
  (`debug_tools/monitor.rs:80-84`).
- `WithCleanup` intentionally evaluates cleanup
  (`kernel_data/control_data/cleanup.rs:80-94`). That is a designated unwind
  effect, not permission to resume every ordinary frame.

Therefore abandonment needs its own closed transition protocol.

### Proposed private shape

1. Add a runtime-private
   `Session::abandon_pending_suspension(AbandonReason)` in
   `crates/rum-runtime/src/runtime/suspension/core.rs`. `AbandonReason` should be
   a closed enum at least distinguishing cancelled request, helper-best-effort
   discard, rollback discard, and superseded/queued-input discard. The reason
   is diagnostic/policy data; no caller supplies host input.
2. Add a **mandatory, no-default** object-safe abandonment method to
   `SuspensionFrame`. It receives a restricted `AbandonmentContext`, not
   `&mut Session`, and returns a closed `FrameAbandonAction`. The context's
   `Session` field remains private to the suspension module and exposes only
   structural restore/pop/truncate operations. It exposes neither `evaluate`
   nor input conversion/delivery.
3. Every named frame in the existing `FrameRootTrace` inventory must explicitly
   classify itself. Pure computation frames return `Discard`. Stateful frames
   invoke their exact structural restoration. This makes a newly added frame a
   compile error until its abandonment behavior is chosen.
4. Put `pending_input_request`, the pending frame owner, the resuming owner, and
   a monotonically increasing suspension generation behind a non-constructible
   internal `SuspensionSlot`/owner API. Other runtime modules may request
   `take_for_resume` or `take_for_abandon`; they cannot obtain a raw `Option` or
   assign `None`. Extend the repository invariant test to reject raw owner
   replacement and any live-session drop outside `suspension/core.rs`.
   Whole-`Session` destruction/reset remains allowed because all contaminated
   state is destroyed with it.
5. Keep ordinary completion separate. Rename
   `clear_suspension_state` to a completed-resumption operation and assert that
   its frame queue is already empty. It must not invoke abandonment after
   `drive_resuming_frames` has already unwound the queue.

The type boundary matters more than a convention: the current four raw callers
all compile because `pending_suspension` is a broadly visible `Option`
(`runtime/model/session.rs:108-111`). A private owner plus a mandatory frame
abandon method makes the invalid operation unavailable.

### Entrypoint guarantees

For a live `Session`, successful return from the entrypoint must guarantee:

- the selected suspension generation has no pending/resuming frames and no
  paired input request or pending top-level request checkpoint;
- frame cleanup ran in the same inner-to-outer order as normal resumption;
- every frame-owned scope/deadline/limit/capture/checkpoint is restored exactly
  once, and unescaped module locals are reclaimed or registered for ordinary
  GC exactly as on an error;
- no host input is converted, wrapped as an `Expr`, consumed from a queue, or
  delivered;
- no ordinary continuation, catch/confirmation handler, `Check` fallback,
  resource fallback, `Reap` handler, trace renderer, monitor refresh/callback,
  or later body expression is evaluated;
- ordinary effects already committed before suspension (`Begin`, an
  unscoped assignment, etc.) are not transactionally rolled back unless the
  caller is separately restoring a checkpoint such as `ValueQ`;
- the original abandonment result/reason cannot be swallowed or transformed
  into a normal value by an ordinary frame.

`WithCleanup` needs the one explicit exception. Its documented cleanup
expression should be represented by an allowlisted
`FrameAbandonAction::EvaluateCleanup`, constructible only from the
`WithCleanup` frame family (backed by a repository allowlist gate). The driver
may evaluate that expression, then resumes **abandonment**, never the ordinary
continuation. Its result does not replace the latched abandonment reason.
This preserves the documented abnormal-exit obligation while making every
other evaluation request a gate failure.

### Re-entrancy and nested suspension

The abandoner should be an iterative worklist, not a recursive call while a
frame queue is mutably borrowed:

1. Take the selected owner into an `abandoning` slot and process its front frame.
2. If the frame requests an allowlisted cleanup evaluation, release all borrows,
   evaluate that cleanup under an `abandonment_active` guard, and then continue
   the outer queue.
3. If cleanup itself returns `InputRequired`, take that newly created pending
   owner immediately and push it onto a LIFO abandonment worklist. Never expose
   its request and never source an input value. Its own nested `WithCleanup`
   obligations are still processed.
4. If a best-effort monitor helper calls the entrypoint re-entrantly while the
   guard is active, it transfers the new pending owner to that worklist rather
   than clearing or recursively driving the current owner.
5. Latch cleanup errors for diagnostics if desired, but continue structural
   unwind and retain the original cancellation/discard reason.

This ordering preserves lexical cleanup placement: an inner cleanup runs
before an outer `Block` is restored; a cleanup outside that `Block` runs after
restoration. It also closes the nested-input case without inventing input.

### Checkpoint rollback interaction

The `had_pending_suspension: bool` snapshot field is insufficient
(`session_helpers.rs:27,131,160-161`): it cannot distinguish the same owner
from a replacement and cannot recover an owner that `begin_input_suspension`
silently overwrote (`suspension/core.rs:125-142`). Replace it with a suspension
generation/token.

For a trial that began with no owner and created one, rollback order is:

1. abandon the trial-created generation structurally;
2. verify the slot is empty;
3. restore the evaluation snapshot.

This order lets `Block`, iterator, module, capture, and other frame-specific
cleanup run while their saved state is coherent, then lets the snapshot provide
the transactional final word. If the snapshot began with an existing owner,
the runtime must preserve that exact generation; a nested trial may not
overwrite it. The request boundary should reject overlapping public `eval`
while a top-level input request is pending (the documented route is
`resume_with_input`), or an explicitly documented supersede operation must
first abandon the old generation. I recommend rejection: public queued input
is documented for pre-supplying batch/script input, not for replacing a
suspended single-source request (`crates/rum-host/src/session.rs:509-523`).

### Callers after the entrypoint exists

- `builtins_input.rs:233`: remove the raw clear. With the request-boundary
  overlap guard, the old/new public request exploit is rejected; if an internal
  queued path nevertheless finds a different live generation, explicitly
  abandon it before consuming the queued value.
- `monitor.rs:115,146`: structurally abandon the helper-created generation,
  ignore its input/result as before, and continue best-effort behavior.
- `session_helpers.rs:160-161`: generation-aware abandonment before snapshot
  restore.
- `suspension/core.rs:216`: completed-only clear/assert, not abandonment.
- brief-400 cancellation: after a cancel request has been accepted but before
  `input.into()` at `session_requests.rs:189`, abandon with reason `Cancelled`
  and return the normal cancelled termination without cloning, converting, or
  delivering input.

### Cost boundary

The abandonment driver, frame abandonment methods, and migrated callers are
cold suspension/error/helper paths. A new vtable method and a private slot type
add no executed work to ordinary expression evaluation. This repair, if kept
to those paths, needs **no timing A/B**, exactly as this brief states. A single
non-allocating pending-owner check at the host request boundary is request
lifecycle policy, not an evaluator-step hot path. If implementation changes
`evaluate_once`, dispatch/matcher ordinary branches, `resume_or_suspend!`'s
non-suspending arm, or `Expr` construction/drop, then it has escaped this design
and the normal hot-path A/B rule applies. If reviewers need proof that the
ordinary request gained no allocation, use the existing env-gated allocation
counter rather than noisy timing.

## 5. Stepwise plan and drafted implementation briefs

### Gateable plan

1. **Introduce and prove the closed abandonment protocol, without migrating
   callers yet.** Modify `runtime/suspension/core.rs`, `runtime/suspension.rs`,
   `runtime/kernel_api.rs`, `runtime/model/session.rs`,
   `runtime/session/session_state.rs`, every existing `SuspensionFrame` impl
   (only to add its mandatory abandonment classification), and
   `runtime/tests/frame_root_trace.rs`. Add direct private tests for all
   stateful categories and for forbidden ordinary effects. Existing raw clear
   methods may remain temporarily but are marked legacy and not used by the new
   tests. Gate: focused suspension/inventory tests, `cargo test -p rum-runtime`,
   then workspace. Must not change normal resume behavior or run cleanup except
   the explicitly designated `WithCleanup` action.
2. **Migrate every committed raw-discard caller and close ownership.** Modify
   `runtime/builtins_input.rs`, `runtime/debug_tools/monitor.rs`,
   `runtime/session/session_helpers.rs`, and the request boundary in
   `runtime/session/session_requests.rs`; remove raw clear APIs/field access and
   strengthen the repository invariant. Add public runtime/host regressions for
   every row in section 2 and the additional state panel. Gate runtime, host,
   workspace. Preserve `Monitor`'s best-effort result contract and `ValueQ`'s
   rollback contract.
3. **Document and test the overlapping-request rule.** Update
   `crates/rum-host/src/session.rs` rustdoc and `docs/guide/embedding.md` to say
   that a new eval while a single-source continuation is pending is rejected;
   queued input remains pre-supply only. Add host contract tests proving the old
   continuation remains resumable after a rejected request and that reset/drop
   remains legal. This can be included in step 2 if the director accepts the
   recommendation; otherwise choose and document explicit supersede semantics
   before dispatch.
4. **Amend/resume brief-400.** Its accepted-cancellation branch calls the
   private entrypoint before input conversion/delivery and returns `Cancelled`.
   Add a custom `Into<String>` counter test, post-input side-effect sentinel,
   `Block`/`Module`/`With` scope panel, nested suspension, and `WithCleanup`.
   Gate runtime + host + workspace. **This is the step brief-400 needs; it
   depends on steps 1-2 (and on the overlap policy being settled).**

No step needs a timing A/B if it stays on the cold paths named above.

### Draft follow-up implementation brief A — closed abandonment protocol

#### Goal

Add a runtime-private, type-directed suspension-abandonment protocol that
restores all frame-owned kernel state without resuming ordinary continuation
work or delivering input. Every named suspension frame must explicitly declare
its abandonment behavior, and direct private tests must prove the complete
stateful inventory, including the narrowly allowlisted `WithCleanup` behavior.

#### Context pointers

- Read `AGENTS.md`, `ARCHITECTURE.md`, `docs/testing-strategy.md`,
  `docs/runtime-evaluation-audit.md`,
  `docs/host-embedding-and-hardening.md`, and brief-401's report.
- Core protocol: `crates/rum-runtime/src/runtime/suspension/core.rs`.
- Session ownership: `runtime/model/session.rs`,
  `runtime/session/session_state.rs`, `runtime/suspension.rs`,
  `runtime/kernel_api.rs`.
- Inventory gate: `runtime/tests/frame_root_trace.rs` and
  `runtime/tests/frame_root_trace/derived_inventory.rs`.
- Stateful families: lexical `block.rs`/`module.rs`, iteration `driver.rs` and
  `iteration/mod.rs`, `options/patterns.rs`, `debug_tools/{reap,monitor,trace,
  evaluation_data}.rs`, `debug_tools/resources/constraints.rs`,
  `kernel_data/control_data/{check_quiet,cleanup}.rs`, abort cleanup frames,
  test-report capture, session random-state frames, and package-install frames.
- Acceptance is mechanical: the named tests and repository invariant decide.

#### Scope

- Files you MAY modify:
  - the core/session/inventory files named above;
  - existing Rust files under `crates/rum-runtime/src/runtime/` and
    `crates/rum-stdlib/src/` **only where they already implement
    `SuspensionFrame`**, and only for the mandatory abandonment method or a
    directly used structural restore helper;
  - focused runtime tests under `crates/rum-runtime/src/runtime/tests/`.
- Files/areas you MUST NOT touch: public `rum-core` representation, `Cargo.lock`,
  unrelated docs, public host semantics, or ordinary evaluator/dispatch logic.
- Do not migrate the four production raw-clear callers in this brief; that is
  brief B, so this change remains independently reviewable.

#### Deliverable

- Closed `AbandonReason`, restricted `AbandonmentContext`, and closed
  abandonment action protocol.
- A mandatory no-default abandonment implementation/classification for every
  inventory frame.
- Structural restoration for every stateful family in section 3.
- `WithCleanup` as the sole repository-gated cleanup-evaluation action;
  nested input is immediately abandoned without input.
- Direct tests proving no later continuation, monitor refresh, handler,
  fallback, or trace/reap rendering runs, and proving every saved session field
  is restored.
- Updated repository invariant that fails when a frame is unclassified or a
  non-allowlisted frame can request evaluation.

#### rum / Rust pitfalls (read twice)

1. Do not call normal `SuspensionFrame::resume` with a synthetic error; frames
   can consume it and run ordinary work.
2. Preserve front-to-back inner-to-outer order, including two-frame payload
   protocols used by `Block` and loop iterators.
3. The restricted abandonment context must not expose `Session::evaluate`,
   input queues, or a general closure callback.
4. Do not render `Reap`, refresh `Monitor`, choose constraint fallbacks, or run
   `Check` fallback while abandoning.
5. Whole-session drop/reset is legal; only live-session owner discard is
   forbidden.
6. No new `unsafe`; do not add ordinary-path allocation, locking, or evaluator
   branches.

#### Verification (MANDATORY)

```text
scripts/run-bounded.py --timeout 300 -- cargo fmt --check
scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-runtime suspension_protocol_repository_invariants_hold -- --nocapture
scripts/run-bounded.py --timeout 1200 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
```

Paste real test counts and invariant inventory rows. No timing A/B is required
for cold-only methods; if ordinary eval/dispatch code changes, stop and report
that the brief's cost premise was exceeded.

#### Hard rules

- No commit, branch, push, stash, checkout, or git-config changes; preserve the
  dirty worktree and the brief-400 stash.
- Modify only Scope; no generated artifacts or probe files in the repository.
- Testing Policy is absolute: rum-native contract tests only, no external
  oracle/differential lane.
- No new `unsafe`; no drive-by reformatting or renames.
- Use `scripts/run-bounded.py` for long/child-spawning commands and paste real
  verbatim output.

#### Report format

1. Per-file change and frame classification counts.
2. Structural owner inventory and exact abandonment action for each family.
3. Verification output.
4. Ordinary-path cost statement and any allocation-counter evidence.
5. Stop-loss, risks, open questions.

### Draft follow-up implementation brief B — migrate committed discard sites

#### Goal

Route every committed live-session suspension discard through the new
abandonment entrypoint, remove raw-clear capability, close overlapping public
request behavior, and add public regressions proving `Block`, `Module`, `With`,
and every additional stateful owner remain clean while `Monitor`/`ValueQ`
retain their existing result contracts.

#### Context pointers

- Read brief-401's sections 1-4 and the completed report from brief A.
- Raw callers: `crates/rum-runtime/src/runtime/builtins_input.rs:231-239`,
  `runtime/debug_tools/monitor.rs:112-148`, and
  `runtime/session/session_helpers.rs:135-162`.
- Completed-only path: `runtime/suspension/core.rs:200-216`.
- Request lifecycle: `runtime/session/session_requests.rs:108-249`,
  `crates/rum-host/src/session.rs:318-332,509-536`, and
  `docs/guide/embedding.md:272-289`.
- Runtime/host tests own the behavior; docs/rustdoc own the public overlap rule.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/builtins_input.rs`;
  - `crates/rum-runtime/src/runtime/debug_tools/monitor.rs`;
  - `crates/rum-runtime/src/runtime/session/session_helpers.rs`;
  - `crates/rum-runtime/src/runtime/session/session_requests.rs`;
  - `crates/rum-runtime/src/runtime/suspension/core.rs` and
    `runtime/model/session.rs` only to remove the temporary legacy raw-clear
    surface and strengthen ownership;
  - `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and focused
    runtime session/scoping tests;
  - `crates/rum-host/src/session.rs`, focused `crates/rum-host/src/tests/`, and
    `docs/guide/embedding.md` for the accepted overlap contract.
- Do not modify builtin result semantics, generated builtin reference, or
  unrelated evaluator/adapter code.

#### Deliverable

- All three true abandonment callers use the private entrypoint; the completed
  error path asserts empty completion and does not double-unwind.
- The checkpoint stores/compares suspension generations and abandons a
  trial-created owner before restoring its snapshot.
- New public eval/call while a single-source continuation is pending returns a
  documented host failure without changing or abandoning the old owner; the
  old request remains resumable. Queued input remains pre-supply behavior.
- Repository gate reports zero raw live-owner clears/replacements outside core.
- Public regression matrix for every path: separate `Block`, `Module`, `With`,
  plus attributes, iterator dynamic state, option scope, Reap, Monitor, trace,
  time/memory constraints, PRNG, `AbortProtect`, capture checkpoints, package
  rollback, and `WithCleanup`.
- `Monitor` still ignores helper values/errors/input and `ValueQ` still returns
  its documented boolean; only state contamination changes.

#### rum / Rust pitfalls (read twice)

1. Abandon helper-created state before restoring a rollback snapshot.
2. Do not abandon a pre-existing generation captured by the snapshot.
3. Do not call `finish_monitor`; abandonment pop must not refresh.
4. Do not consume a queued response while discarding an older owner.
5. Preserve single-source `resume_with_input`, script pre-queue behavior,
   reset, panic poison, and structured outputs.
6. Keep all work on request/suspension cold paths; no ordinary evaluator-step
   branch or allocation.

#### Verification (MANDATORY)

```text
scripts/run-bounded.py --timeout 300 -- cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
```

Paste the exact public matrix rows/test counts and repository-invariant rows.
No timing A/B for the confined cold path. If an ordinary eval/dispatch path is
changed, stop and report scope/cost expansion.

#### Hard rules

- No commit, branch, push, stash, checkout, or git-config changes; preserve all
  pre-existing changes and the brief-400 stash.
- Modify only Scope. Update public docs in the same change as the overlap
  contract; do not edit generated reference text.
- Testing Policy is absolute; no vendor engine, oracle, compatibility matrix,
  or third-party output.
- No new `unsafe`; no drive-by changes; bounded commands only; paste genuine
  verbatim output.

#### Report format

1. Per-caller migration and overlap contract.
2. Public regression matrix with verbatim outcomes.
3. Verification output.
4. Cost boundary.
5. Stop-loss, risks, open questions.

### Draft brief-400 amendment — cancellation before input delivery

#### Goal

Complete brief-400's accepted-request cancellation behavior using the now
landed private abandonment entrypoint: a cancellation accepted while a
single-source input continuation is pending must win before host input is
converted or delivered, unwind all frame-owned state, preserve documented
cleanup behavior, return the structured `Cancelled` termination, and leave the
session reusable.

#### Context pointers

- Read the original brief-400, its restored work, brief-401, and completed
  briefs A/B before editing.
- Cancellation flag: `crates/rum-host/src/session.rs:113-125,440-447`.
- Resume request: `crates/rum-runtime/src/runtime/session/session_requests.rs:185-249`.
- Runtime resumer: `runtime/suspension/core.rs:169-220` plus the new private
  abandonment entrypoint.
- Public contract: `docs/guide/embedding.md:267-289,410-418` and host rustdoc.
- The director must merge this amendment into brief-400's exact original scope;
  do not inspect or manipulate its stash from Codex.

#### Scope

- Files you MAY modify: brief-400's already authorized files, plus
  `runtime/session/session_requests.rs`, the minimal call to the private
  abandonment API in `runtime/suspension/core.rs` if needed, focused
  `rum-runtime` tests, focused `rum-host` tests, and cancellation/input rustdoc
  explicitly authorized by the director.
- Do not redesign abandonment, change general evaluator polling cadence, or
  alter unrelated cancellation/timeout/resource semantics.

#### Deliverable

- Accepted pending-request cancellation is checked before
  `input.into()`, string `Expr` construction, queue consumption, or frame
  resumption.
- It invokes `abandon_pending_suspension(Cancelled)` exactly once and returns
  `HostTerminationReason::Cancelled` through the established structured path.
- A custom `Into<String>` test proves conversion was not called; a rum-side
  sentinel proves statements after `InputString` did not run.
- Separate public `Block`, `Module`, and `With` cases plus attributes,
  iterator/option/monitor/reap/resource/trace/PRNG/abort state are clean.
- Nested cleanup input is abandoned without a second host request;
  `WithCleanup` follows the policy landed in brief A.
- Cancellation arriving too late follows the already documented cooperative
  semantics; reset and subsequent eval remain usable.

#### rum / Rust pitfalls (read twice)

1. Do not clone/convert the resume input before checking the accepted cancel.
2. Do not call the ordinary resume driver with synthetic `Cancelled`.
3. Do not let a frame transform cancellation into success or run later
   continuation/fallback/handler work.
4. Preserve the cancellation flag's request-generation/reset rules and avoid
   leaking a stale flag into the next request.
5. Cleanup evaluation must not publish nested `InputRequired`.
6. Do not change the amortized polling hot path in this amendment.

#### Verification (MANDATORY)

```text
scripts/run-bounded.py --timeout 300 -- cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 900 -- cargo test --offline -p rum-host
scripts/run-bounded.py --timeout 1800 -- cargo test --offline --workspace
```

Paste the exact cancellation outcome, conversion counter, side-effect
sentinel, scope-cleanliness rows, and test counts. No timing A/B is required
unless the polling/evaluator ordinary path is changed.

#### Hard rules

- Do not manipulate the stash. No commit, branch, push, checkout, or git-config
  change; preserve unrelated worktree edits.
- Stay inside the amended scope. No new `unsafe`, no external oracle, no
  generated probe artifacts, and no drive-by cleanup.
- Use bounded commands and paste real verbatim output; stop if the amendment
  needs a broader cancellation model than authorized.

#### Report format

1. Accepted-cancel branch and exact point before conversion.
2. Scope/cleanup regression matrix.
3. Verification output.
4. Cost statement.
5. Stop-loss, risks, open questions.

## 6. Stop-loss, risks, open questions, and background corrections

### Stop-loss and temporary artifacts

The stop-loss did not trigger: the first serious public-surface harness settled
question 1. The first Cargo invocation omitted `--offline` and the workspace's
configured USTC mirror was unavailable; it produced no semantic evidence:

```text
error: failed to get `thiserror` as a dependency of package `rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)`

Caused by:
  failed to download from `https://mirrors.ustc.edu.cn/crates.io-index/config.json`

Caused by:
  [6] Could not resolve hostname (Could not resolve host: mirrors.ustc.edu.cn)
```

All evidence runs after that used `--offline`. There was no source
instrumentation. The only throwaway artifact was
`/tmp/rum-brief401-probe` (Cargo manifest and `src/main.rs`); it is removed
after this report is written and before final `git status`.

### Risks and open decisions

1. **`WithCleanup` policy under host discard.** The generated contract says it
   runs on abnormal exit, so the recommendation is to run only this explicitly
   designated cleanup. The implementation must decide how an already-latched
   cancel/deadline constrains a long cleanup (attempt under current cooperative
   limits, or a separate bounded cleanup budget). Silently skipping it would be
   a contract regression; masking cancellation without a bound can hang.
2. **Overlapping public requests.** Rejection is the recommended contract and
   matches existing “resume with `resume_with_input`” guidance, but it is a
   public lifecycle decision. If supersede is desired instead, it must be an
   explicit documented operation that abandons before starting new work—not an
   accidental side effect of consuming queued input.
3. **Snapshot owner identity.** A boolean cannot preserve an old pending owner
   across a nested trial. A generation/token and a no-overwrite invariant are
   required before claiming checkpoint safety.
4. **Module cleanup roots.** Abandonment has no successful result root. Reuse
   the existing error/no-result reclamation branch and test escaped locals,
   side-table stamps, GC candidates, and nested modules; do not simply delete
   every gensym.
5. **Capture buffer ownership.** Messages/prints already returned in an
   `InputRequired` outcome and those created by a later request must not be
   double-reported or truncated. Tests need exact checkpoints around overlap,
   monitor helper discard, `ValueQ`, and cancellation.
6. **Frame-audit breadth.** The repository has hundreds of named frames. A
   mandatory method plus the inventory gate is intentionally a large mechanical
   compile-time audit; a default no-op would make the patch smaller but would
   recreate the “future frame forgot cleanup” class.

### Corrections to the background

1. The split from brief-400 was justified: committed paths already leak state.
2. `suspension/core.rs:164-166`/`:216` is not a raw abandonment path on the
   committed tree; it clears only after the ordinary driver has drained all
   frames.
3. Driving every normal frame with `Err(Cancelled)` is not sufficient to meet
   the “no ordinary continuation effects” contract. A distinct abandonment
   protocol is required because normal frames can consume errors and evaluate
   work.
4. Current cancellation alone does not raw-drop the queue. It either goes
   unnoticed until input-only continuation completion or is polled after input
   delivery and normally unwinds. Brief-400's new accepted-request branch is
   where cancellation must call the new entrypoint.
5. `Monitor`'s best-effort standing debt covers ignored helper outcomes, not
   leaked kernel scopes. Fixing structural cleanup does not promote the monitor
   result contract.
6. `$Context`/`$ContextPath` changes from explicit `Begin` and arbitrary
   unscoped mutations are ordinary effects, not frame-owned state. A structural
   abandoner must not become an implicit full transaction; checkpoint callers
   such as `ValueQ` remain responsible for their broader rollback.

### Verification addendum (run only after sections 1-6 were written)

Temporary harness cleanup was verified with:

```text
temporary_probe_removed
```

Focused inventory verification:

```text
$ scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-runtime suspension_protocol_repository_invariants_hold -- --nocapture
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:296:27
    |
296 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:305:27
    |
305 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.86s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 1 test
named_suspension_frames=545
named_pure_finalizers=60
legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 904 filtered out; finished in 0.40s
```

The three warnings are pre-existing dead-code warnings in test-builtin helpers;
the focused invariant is green. Per this diagnostic brief, no broader source
test gate or performance A/B was run.

Report whitespace check:

```text
$ git diff --check -- docs/dev/codex/brief-401-abandoned-suspension-scope.md
```

The command exited `0` with no output.

Final worktree status at that check:

```text
$ git status --short
 M .claude/hooks/guard-dispatch.py
 M docs/dev/codex/brief-401-abandoned-suspension-scope.md
 M scripts/README.md
 M scripts/core-memory-microbench.sh
 M scripts/dispatch-brief.sh
 M scripts/guarded-test-run.sh
?? scripts/peak-rss.py
```

The brief report is the only change made for brief-401. The other five modified
files and untracked `scripts/peak-rss.py` are the pre-existing worktree state
named in the brief; they were preserved. No repository probe file remains.
