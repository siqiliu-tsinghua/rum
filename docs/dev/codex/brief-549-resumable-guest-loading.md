# brief-549 — design: resumable guest package loading, and the leaked suspension owner

`Scope: NONE — no production change.` Design-only, with one declared scratch
exception for probes (see Scope). Diagnose the leak, design resumable guest
loading, and return drafted implementation briefs. **Do not run `git` in any
form**, including while first loading this repository.

## Goal

This is step 1 of the suspension-continuation branch (`TODO.md`, "ACTIVE
BRANCH"; brief-548's report, §2 transformed diagnostics and §8 question 2).

**User ruling, 2026-09-28: guest package loading becomes resumable.** When a
file loaded by guest `Get`, `Needs`, `BeginPackage`'s dependency list, or a
`DeclarePackage` stub's autoload requests input, evaluation suspends like any
other guest evaluation, and `resume_eval_request_with_input` continues the load
from where it stopped and then completes the builtin that asked for it. Today
the loader turns that suspension into `$Failed` with `Get::load` "input
required", and **leaks the pending suspension while doing so**. The public host
batch APIs (`HostSession::eval_script`, `HostSession::eval_file`,
`crates/rum-host/src/session.rs:378-412`) document that script suspension is
not resumable. **That contract is not part of this ruling and is not changed
implicitly** — see §4.

Design the change, fix the leak as part of it, and draft the implementation
briefs.

## What the director measured

The director's probe is `.temp/director/leak-probe.rs`, its raw output
`.temp/director/leak-probe.txt`. It uses brief-548's read-only in-memory
provider, whose `Dep548.rum` is:

```text
AppendTo[Global`log,1]
InputString["p: "]
AppendTo[Global`log,3]
Dep548`foo=42
77
```

Each source ran in a fresh session. After it, the probe sent a second request,
`1 + 1`.

| Source | Result | Second request |
| --- | --- | --- |
| `Get["Dep548`"]` | `$Failed` | **rejected**: "cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first" |
| `Get["Dep548`"]; InputString["q: "]` | **panic** at `suspension/core.rs:228`, "a new suspension must not replace a live continuation owner" | — |
| `Needs["Dep548`"]; InputString["q: "]` | **panic**, same assertion | — |
| `ValueQ[InputString["a: "]]; InputString["q: "]` | suspends on `"q: "`, correctly | waiting for `"q: "`, correctly |
| `Monitor[77, InputString["a: "]]; InputString["q: "]` | suspends on `"q: "`, correctly | waiting for `"q: "`, correctly |

brief-548's `DeclarePackage["Dep548`", {"foo"}]; Dep548`foo` panics on the
same assertion.

So the loader returns a completed value while the input request it swallowed
is still installed as the session's pending continuation owner. The session is
then wedged: every later request is refused, and any later input request in
the same evaluation fails the kernel's own assertion. `ValueQ` and `Monitor`,
which also swallow a nested suspension, leave no owner behind.
`HostSession`'s guard catches the panic and poisons the session
(`rum-host/src/session.rs:695`); a direct `Session` user has no guard.

## The code, as the director read it

- `Session::eval_get` (`crates/rum-runtime/src/runtime/builtins_context_loader.rs:248-302`)
  calls the public `Session::eval_file` and turns any error except `Exit` into
  `Get::load` plus `$Failed`.
- `Session::eval_file` (`crates/rum-runtime/src/runtime/session/session_eval.rs:389-412`)
  reads the file, captures a loader snapshot, pushes `load_stack`, runs
  `eval_script_for_loader`, pops `load_stack`, and restores the snapshot on
  error. It is **public** and shared by the host batch path and guest `Get`.
- `eval_script_with_terminated_handling` (`session_eval.rs:303-387`) splits the
  source into forms and runs each through `eval_input_with_terminated_handling`
  — a top-level entry, nested inside a running evaluation — collecting value,
  termination, messages and prints, with message and print checkpoints.
- `Session::eval_needs` (`builtins_context_loader.rs:304`) and
  `eval_begin_package` (`:422`) load through the same route;
  `evaluate_symbol` (`eval_core/symbol.rs:56-68`) calls `eval_needs` for a
  declared stub whose context is not yet loaded.
- Pending suspension installation is `suspension_access().install(...)`
  (`suspension/core.rs:227-240`, the assertion at `:228`), and the retained
  evaluation stack is taken at `:491-497`.

These are the director's readings, to be confirmed or corrected from the
source, not premises.

## What I want back

### 1. The leak's exact mechanism

Which path installs the pending owner, which conversion drops the error while
leaving the owner installed, and why `ValueQ` and `Monitor` do not leak — cite
the lines that abandon or restore in their case. Say whether the fix for the
leak is only a consequence of making loading resumable, or whether some route
will still convert a suspension into a value after this change (a
non-resumable host boundary, a loader error path, an abandonment) and
therefore needs its own abandon-exactly-once discipline.

### 2. The owner-leak class

brief-548 recorded 38 `Transformed` pairs (`.temp/brief-548/census.tsv`,
`verdict == Transformed`). For every one that converts a suspension into a
value or another error, determine whether the pending owner is abandoned. Probe
each reachable one with the shape above — the construct, then a second input
request in the same source, then a second request — and tabulate the results.
The front-end rows (`rum-cli`, `rum-repl`, `rum-jupyter`, snapshot restore)
are boundaries, not guest constructs; say whether each leaves the session
usable.

### 3. The resumable loading design

State what a suspended load must carry and how it resumes, for every entry:

- **`Get`**: the remaining parsed forms and the form index, the value and
  termination so far, accumulated messages and prints and their checkpoints,
  the `load_stack` entry, the loader snapshot and its restore-on-error rule, the
  terminated-evaluation handling, and `Get`'s own completion (merging messages
  and prints, returning the last value).
- **`Needs`**: the context and load spec, then its post-load checks, the
  `$Packages` update and its messages.
- **`BeginPackage`**: the dependency cursor and the remaining dependencies,
  then the context changes it performs after them.
- **The `DeclarePackage` stub**: resuming into `evaluate_symbol` after the load
  completes, so the symbol's value is read as if the load had never suspended.
- **Nesting**: a loaded file that itself calls `Get`; `$InputFileName` read
  after a resume inside a nested load; two suspensions inside one file.

And for each: what happens on an ordinary error after a resume (the snapshot
must still restore), on abandonment and cancellation (restore the snapshot and
pop `load_stack` exactly once — no double pop, no leaked entry), on `reset`,
and on a fresh request afterwards. The form-level entry
`eval_input_with_terminated_handling` is a top-level entry nested in a running
evaluation; say precisely what request-level state it checkpoints, and whether
any of it is disturbed by suspending inside it (the evaluation stack retained
at `core.rs:491-497`, message and print checkpoints, `$Line`, handled-error
checkpoints, the active accepted request).

Frames derive `FrameRootTrace` and keep the existing suspend, resume and
abandon contract. This branch builds **no leases** — the retention line's R11
and R12 cover the loader later, and brief-547 recorded the loader's snapshot
and handoff as N; say what state your frames carry so that conversion can
build on it.

### 4. The shared `eval_file`

Guest `Get` and the host batch API share `Session::eval_file`. The ruling makes
guest loading resumable and leaves the host batch contract alone. Design the
split — or show that one resumable implementation can keep the host contract
exactly as documented. Either way, state what the host batch path does after
the change when a form requests input, and confirm it no longer leaks an owner
either. If you believe the host contract should change, put it in §7 as a
question; do not design it in.

### 5. Red tests

Every behaviour above as a failing test first, through brief-548's oracle: the
suspended-then-resumed path equals the queued-input path in value, side
effects (`log`), messages and prints, and asserts that the suspension actually
happened. Include the leak regressions: after each case completes, a second
request must be accepted. Name each test and its expected pre-fix result,
recorded from your probes.

### 6. Drafted implementation briefs

In `TEMPLATE.md` shape. Scope states **an objective, a fence and a disclosure
duty**, not a list of permitted files. Scratch and checkpoints go under
`.temp/brief-NNN/`. The stop-loss is keyed on progress (`TEMPLATE.md` pitfall
7). Every draft starts from its red tests. Any draft that touches an
evaluation hot path names the microbench labels it exposes; the measurement
discipline is per-iteration instructions (`perf stat -e instructions:u`,
pinned with `taskset -c 2`), allocation counts first, timing secondary.

### 7. Open questions for the director

## Settled — do not reopen

- Guest loading is resumable (user ruling, 2026-09-28).
- The host batch APIs' documented non-resumability is unchanged by this work.
- This branch builds no leases.
- Overlapping public requests are rejected, not superseded (brief-411).
- The oracle is rum against rum, and is admissible.

## Context pointers

- `AGENTS.md` first.
- `docs/dev/codex/brief-548-suspension-continuation-sweep.md` — the report
  from line 304: the mechanism, the oracle, the transformed diagnostics and the
  census. `.temp/brief-548/probe.rs` — the harness and the in-memory package
  provider; `.temp/brief-548/census.tsv`.
- `.temp/director/leak-probe.rs` and `.temp/director/leak-probe.txt`.
- `crates/rum-runtime/src/runtime/suspension/core.rs` — install, the queue, the
  driver, abandonment.
- `crates/rum-runtime/src/runtime/builtins_context_loader.rs`,
  `session/session_eval.rs`, `session/session_helpers.rs:97-160` (loader
  snapshot), `session/session_state.rs:290` (`LoaderSnapshot`),
  `package_tools.rs`, `eval_core/symbol.rs`.
- `crates/rum-host/src/session.rs:370-420,695-715` — the host batch contract
  and the panic guard.
- `docs/guide/` — whatever documents `Get`, `Needs`, `BeginPackage`,
  `DeclarePackage` and input requests; say which passages the change makes
  false.

## Scope

- Files you MAY modify: **NONE**, with three exceptions:
  - append your report to **this** file;
  - write scratch under `.temp/brief-549/`;
  - create **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief549_probe.rs`. The
    `crates/rum-stdlib/tests/` directory does not exist today. **Before you
    append the report, copy the file's final content to
    `.temp/brief-549/probe.rs`, then delete the file and the directory**, and
    say in the report that you did.
- Everything else is read-only.

## rum / Rust pitfalls (read twice)

1. **A probe that did not suspend proves nothing**, and a leak shows only on
   the *next* suspension or the *next* request. Probe both.
2. **The loader restores a snapshot on error.** After a resume, an error must
   still restore it; after completion it must not.
3. **`load_stack` is pushed and popped around the load.** A suspended load has
   popped nothing yet; an abandoned one must pop exactly once.
4. **Messages and prints are part of the answer.** Compare them as well as
   values and `log`.
5. **Suspend in the middle and suspend twice**, including once in a nested
   `Get`.
6. **The form entry is a top-level entry.** Suspending inside it is not the
   same as suspending inside an ordinary builtin call; read what it
   checkpoints.

## Verification (MANDATORY — the evidence IS the deliverable)

- State every command; persist raw output under `.temp/brief-549/` before
  parsing it.
- Build and run probes only with
  `python3 scripts/run-bounded.py --timeout <seconds> -- cargo test -p rum-stdlib --test zz_brief549_probe -- --nocapture`,
  the timeout derived from your first build's measured duration.
- Paste the owner-leak table (§2), the probe results behind every red test
  (§5), and the state each frame carries (§3).
- State that the scratch test file and directory were deleted.

## Checkpointing — protect the scene

After each milestone write `.temp/brief-549/PROGRESS.md`: what is done and
verified, its saved evidence, the exact next action, and whether the scratch
test file currently exists.

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
2. Verification — commands and the tables named above.
3. The leak's mechanism, and the owner-leak class table.
4. The resumable loading design, per entry, with abandonment and errors.
5. The `eval_file` split and the host batch path after the change.
6. The red tests.
7. The drafted implementation briefs.
8. Open questions for the director.

---

# brief-549 report — 2026-09-28

## 1. What changed

**Design only; no production change.** This appended report contains the
diagnosis, 38-pair owner audit, resumable-loading design, host split,
red-test evidence and two full implementation drafts. Get and its three
package callers are deliberately one implementation unit: shipping only
the lower driver would leave the callers' work lost.

Evidence and scratch are under [.temp/brief-549/](../../../.temp/brief-549/).
The final authorized integration test was copied byte-for-byte to
[probe.rs](../../../.temp/brief-549/probe.rs), **then**
`crates/rum-stdlib/tests/zz_brief549_probe.rs` and its newly created
`tests/` directory were deleted **before this report was appended**.
[scope-audit-before-report.json](../../../.temp/brief-549/scope-audit-before-report.json)
records no changes to any of the 1,679 baseline-hashed existing repository
files and no new files under crates/docs/scripts after that deletion.
The final scope audit is `scope-audit-final.json`.

No git command, network operation, outside-repository write, production edit,
new unsafe or lease was used. One literal command-allowlist deviation must
be disclosed: initial discovery ran `pwd` before I had read this brief,
alongside the permitted file listing. Initial discovery and the first brief
read were viewed in tool output before a scratch evidence recorder existed;
they are recorded retrospectively in `initial-commands.txt` /
`initial-outputs.txt`. Subsequent source evidence and **all probe raw output**
were persisted before reduction. Three orchestration snippets failed JavaScript
parsing before executing any shell command; none mutated a file.
I therefore do not claim perfect compliance with the bootstrap
command/raw-capture requirements.

## 2. Verification

All commands ran from `/home/remi/dev/rum`. The complete exact command
inventory, including every Python inline edit/analysis body, wrapper
invocation, rg query and nested bounded cargo command, is
[commands.jsonl](../../../.temp/brief-549/commands.jsonl);
[commands.md](../../../.temp/brief-549/commands.md) is its rendered form.
Reads/searches used the recorded
`python3 .temp/brief-549/evidence.py read ...` and `... rg ...` invocations,
saving `read-*.txt` before displaying results. Missing-path search/read
attempts are preserved; no result was inferred from a missing file.
Hashing, scratch edits, reduction, archiving, deletion and report assembly
were Python commands recorded in the same ledger.

Every probe was built and run **only** with:
```text
python3 scripts/run-bounded.py --timeout 30 -- cargo test -p rum-stdlib --test zz_brief549_probe -- --nocapture
python3 scripts/run-bounded.py --timeout 27 -- cargo test -p rum-stdlib --test zz_brief549_probe -- --nocapture
```

The evidence wrapper redirects stdout/stderr to the named raw file before
reading it and records monotonic elapsed duration separately. Run 01's
bootstrap bound 30s came from the director's measured 6.22s build and
brief-548's sub-8s runs of the same shape. This brief's first actual build
took 6.60s; bounded command duration was 6.720204812s. Every later bound was
`ceil(4 × 6.720204812) = 27s`. Five runs were used, below the declared cap of
six; no timing/performance conclusion is drawn from build duration.

| Run / raw output | Bound (s) | Elapsed (s) | Exit | Recorded result |
| --- | ---: | ---: | ---: | --- |
| [probe-01.txt](../../../.temp/brief-549/probe-01.txt) | 30 | 6.720204812 | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s |
| [probe-02.txt](../../../.temp/brief-549/probe-02.txt) | 27 | 6.468036008 | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s |
| [probe-03.txt](../../../.temp/brief-549/probe-03.txt) | 27 | 27.079936873 | 124 | run-bounded: timed out after 27s |
| [probe-04-red.txt](../../../.temp/brief-549/probe-04-red.txt) | 27 | 6.819854265 | 101 | test result: FAILED. 4 passed; 40 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s |
| [probe-05-final.txt](../../../.temp/brief-549/probe-05-final.txt) | 27 | 6.673000863 | 101 | test result: FAILED. 6 passed; 44 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s |


Run 03 timed out in an added nested-monitor/cleanup stress fixture; the
bounded runner killed its process group. It supplied no completion evidence
for that fixture. It was removed, not given a larger timeout. Simple direct
cleanup, cancellation-cleanup and loader-cleanup cases remain covered.
Run 02's incomplete-tail syntax case and under-supplied stub-error/monitor
controls were diagnostic fixture errors; corrected controls are in runs
04/05. A stable Monitor body also failed to invoke its callback; the final
callback fixture uses 1+2 and records provider reads proving it reached Get.

Verbatim final gate lines:
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.40s
running 50 tests
test result: FAILED. 6 passed; 44 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
```

Exit 101 is **expected red evidence**, not a repaired or green workspace.
There are 45 diagnostic source comparisons plus 33 duplicate comparisons
inside named red oracle tests, for 78 paired executions. The remaining
red tests exercise cancellation/cleanup/batch boundaries directly. No cargo
fmt, workspace test or microbenchmark command was run: those are outside
this design brief's command authorization.

[case-sources.json](../../../.temp/brief-549/case-sources.json) retains exact
source, in-memory file contents and response sequences.
[case-summary.json](../../../.temp/brief-549/case-summary.json) retains every
response's complete messages/prints and state.
[streams-and-state.md](../../../.temp/brief-549/streams-and-state.md) pastes
both sides' streams and state for all 45 diagnostic cases. All providers
are in-memory, read-only, and use .rum names; no package file was written.

The `state` field is
``{Global`log,$Context,$ContextPath,$Packages,$InputFileName,Global`marker,Global`$Line}``.
After probing a fresh public request, diagnostics read that state through
low-level `eval_text` even when request admission is wedged; **that bypass
is not counted as session usability**. “Fresh request” below is always the
guarded `eval_request("1+1")`. For every valid queued control it succeeds.

| Probe / test suffix | Pre-fix result | Actual suspensions | Queued result | Fresh request (pre-fix / queued) |
| --- | --- | ---: | --- | --- |
| begin_dependency_second | $Failed | 0 | "Main549`" | False / True |
| begin_error_after_resume | $Failed | 0 | $Failed | False / True |
| begin_package | $Failed | 0 | "Main549`" | False / True |
| begin_package_then_q | PANIC | 0 | "two" | False / True |
| begin_two_dependencies_suspend | $Failed | 0 | "Main549`" | False / True |
| cleanup_loader | PANIC | 0 | "one" | False / True |
| cleanup_nested_input | "one" | 1 | "one" | True / True |
| get | $Failed | 0 | 77 | False / True |
| get_abort_form | $Failed | 0 | 77 | False / True |
| get_cycle_form | $Failed | 0 | TerminatedEvaluation["IterationLimit"] | False / True |
| get_dynamic_context | 0 | 0 | 0 | False / True |
| get_error_after_resume | $Failed | 0 | $Failed | False / True |
| get_exit_form | $Failed | 0 | View { status: "Exit { code: 7 }", value: None, termination: "None", messages: [], prints: [] } | False / True |
| get_input_expression | $Failed | 0 | 77 | False / True |
| get_input_syntax_error | $Failed | 0 | $Failed | False / True |
| get_late_syntax_error | $Failed | 0 | $Failed | False / True |
| get_message_checkpoint | List[HoldForm[probe::tag], HoldForm[Get::load]] | 0 | List[HoldForm[probe::tag], HoldForm[probe::tag], HoldForm[probe::tag]] | False / True |
| get_outer_check | 999 | 0 | 999 | False / True |
| get_outer_quiet | $Failed | 0 | 77 | False / True |
| get_return_form | $Failed | 0 | 77 | False / True |
| get_streams_line | $Failed | 0 | 77 | False / True |
| get_then_q | PANIC | 0 | "two" | False / True |
| get_twice | $Failed | 0 | 77 | False / True |
| lower_later_forms | $Failed | 0 | 42 | False / True |
| monitor_display | 77 | 0 | 77 | True / True |
| monitor_display_then_q | "one" | 1 | "five" | True / True |
| monitor_eval_callback | 3 | 0 | 3 | True / True |
| monitor_eval_callback_then_q | "one" | 1 | "two" | True / True |
| monitor_loader | PANIC | 0 | "two" | False / True |
| monitor_step_callback | 3 | 0 | 3 | True / True |
| monitor_step_callback_then_q | "one" | 1 | "two" | True / True |
| needs | $Failed | 0 | Null | False / True |
| needs_error_after_resume | $Failed | 0 | $Failed | False / True |
| needs_explicit | List[Dep549`foo, List[]] | 0 | List[42, List["Other549`"]] | False / True |
| needs_then_q | PANIC | 0 | "two" | False / True |
| nested_resume_then_error | 88 | 0 | 88 | False / True |
| nested_twice_filename | PANIC | 0 | 77 | False / True |
| repeat_form_stack | $Failed | 0 | 77 | False / True |
| same_form_message_scope | $Failed | 0 | 77 | False / True |
| stub | PANIC | 0 | 42 | False / True |
| stub_error_after_resume | PANIC | 0 | Bad549`foo | False / True |
| stub_then_q | PANIC | 0 | "two" | False / True |
| valueq | True | 0 | True | True / True |
| valueq_loader | "one" | 1 | "one" | True / True |
| valueq_then_q | "one" | 1 | "one" | True / True |


Zero suspensions in these rows means **zero public InputRequired outcomes**.
For the leaking loaders the pending prompt proves the inner input did
install an owner, but the caller swallowed it. No loader equality claim is
admitted without an actual public suspension. Values that happen to agree
(e.g. $Failed, 999 or 88) still have differing diagnostics/effects or a
rejected next request. Monitor/ValueQ intentionally discarded work is
classified separately in ``3 and 6.


## 3. The leak's mechanism and the owner-leak class

Source citations below are repository-relative, at the unchanged source revision
hashed in `baseline-hashes.json`. The census row IDs are the original IDs,
not a renumbered list.

### Exact mechanism

1. `InputString` reaches `drive_input_request`
   (`crates/rum-runtime/src/runtime/builtins_input.rs:231-248`). An empty
   queue calls `begin_input_suspension`. That copies the active evaluation
   stack except its last two entries (`suspension/core.rs:484-499`) and calls
   `SuspensionAccess::install` (`:255-262`), which installs the sole
   `SuspendedEvaluation` owner and prompt. `SuspensionSlot::install` asserts
   that the previous owner is absent (`:227-240`). The failing assertion is
   correct; relaxing it would hide lost cleanup obligations.
2. The loaded form's ordinary evaluator frames attach to this owner.
   `eval_input_with_terminated_handling` returns the error without a frame for
   its own remaining work (`session/session_eval.rs:210-286`).
   `eval_script_with_terminated_handling` splices earlier detached output back
   into session buffers and returns (`:352-372`); its form cursor and local
   accumulators disappear. No script completion frame exists.
3. `Session::eval_file` pops `load_stack` and restores `LoaderSnapshot` for
   **every** error, including `InputRequired` (`:389-411`). This snapshot
   includes definitions, package/context state and messages, but **not**
   the pending suspension, input prompt, prints, evaluation stack, or request
   checkpoint (`session_state.rs:290-319`;
   `session_helpers.rs:97-207`). Restoring it cannot remove the owner.
4. `eval_get` then emits `Get::load` and returns `$Failed` for every error
   except `Exit` (`builtins_context_loader.rs:282-300`). This is the decisive
   swallowed-error conversion. `Needs` adds `Needs::load`
   (`:343-351`). **Correction to brief-548's census prose:** this tree's
   `BeginPackage` does not ignore a failed dependency and continue: it rolls
   its package frame back and returns `$Failed` (`:444-455`).
5. The outer public request sees a value, clears `pending_eval_request`
   (`session_requests.rs:263-266`), and finishes any accepted cancellation
   ticket (`:533-565`), while the pending suspension remains. A new request
   rejects on its generation (`:70-85`); a second input in the same source
   attempts another install and panics. An already-finished cancellation
   handle cannot repair this; `get_cancel`, `needs_cancel`, `begin_cancel`
   and `nested_cancel` measured that additional consequence.
6. The declared stub calls `eval_needs` at `eval_core/symbol.rs:56-68`.
   Its evaluation continues after the swallowed failure. The fixed-point
   evaluator can retry the still-declared, unloaded symbol; the second load
   installs another owner and panics. The lone-stub probe records a remaining
   `"./Dep549.rum"` load-stack entry after unwinding, because the panic skips
   `eval_file`'s manual pop. Direct `Session` has no panic boundary.
   `HostSession::guard` catches the panic and poisons the host session
   (`crates/rum-host/src/session.rs:695-714`).

Why the controls are different:

- `ValueQ` catches its trial and always restores its evaluation rollback
  snapshot (`messages/builtins_messages_inspect.rs:90-118`). That broader
  snapshot records the pending generation; restoring a newly created trial
  generation explicitly abandons it **before** restoring state
  (`session_helpers.rs:237-265`), then restores prompt/request checkpoints
  (`:301-307`). Its conversion to `True` for input is at
  `builtins_messages_inspect.rs:81-85`.
- Monitor callbacks explicitly abandon `InputRequired` at
  `debug_tools/monitor.rs:112-118`, and display refresh does so at
  `:131-154`. Neither merely clears the prompt. The positive controls reach
  a distinct `"q: "` suspension, resume it, and accept `1+1`.
- Abandonment takes the owner out of the slot before executing any cleanup
  (`suspension/core.rs:628-645`). A cleanup that itself returns
  `InputRequired` is taken into the abandonment worklist
  (`:676-685`). However, **a cleanup that calls today's Get** returns a
  value while hiding that new input error; the condition at `:677` is
  false. `red_cancel_cleanup_get_releases_owner` actually suspends on the
  outer input, cancels, and then fails the no-pending-owner assertion.
  This is the same loader defect reached during abandonment, not a defect
  in the worklist's direct `InputRequired` branch.

Making guest loading propagate suspension fixes the loader's normal and
cleanup variants together, but it does **not** eliminate the need for
abandon-exactly-once boundaries. ValueQ, Monitor, nested WithCleanup
abandonment, non-resumable batch requests and restore failure still need
explicit dispositions. A terminal loader error must close its loader
transaction after child continuations have resolved; an abandoned loader
must restore through its frame's structural abandonment method. A prompt
field cleared alone is never evidence that the owner was retired.

### All 38 transformed pairs

Table codes: **G/N/B/S** are the guest probes `get/needs/begin_package/stub`,
each also run with `;InputString["q: "]`. **V** is `valueq`; **MD/MC** are
Monitor display and both callback options. **A** is `cleanup_nested_input`
plus the cancellation probes. For G/N/B, the construct alone returns
`$Failed` and the same-source second input panics; S alone already panics.
Every such subsequent public request is rejected. V/MD/MC/A direct-input
controls expose q, resume q and accept the next request.

**H** means the runtime boundary equivalents were executed; the named host
or frontend wrapper was inspected, not linked or executed. The scratch
crate has no rum-host/repl/jupyter dependencies, and the brief forbids
changing its manifest or running those binaries/tests. Source-only results
are deliberately not presented as adapter execution results.

| Census ID | Call site | Owner disposition | Probe / usability result |
| --- | --- | --- | --- |
| P0110 | builtins_context_loader.rs:282, eval_get → eval_file | No abandonment; converts InputRequired to $Failed | G: alone $Failed; then q PANIC; next rejected |
| P0111 | builtins_context_loader.rs:343, eval_needs → eval_get | Inherits leaked owner; adds Needs::load | N: alone $Failed; then q PANIC; next rejected |
| P0112 | builtins_context_loader.rs:449, eval_begin_package → eval_needs | Context/package rollback, no suspension abandonment | B: alone $Failed; then q PANIC; next rejected |
| P0123 | debug_tools/monitor.rs:113, callback evaluate | Explicit abandon at :115-116 for direct input; loader-hidden input bypasses this arm | MC direct: q resumes, next=2. monitor_loader: PANIC, next rejected |
| P0124 | debug_tools/monitor.rs:131, display evaluate | Explicit abandon at :152-153 for direct input | MD direct: q resumes, next=2; loader-hidden errors remain unsafe |
| P0417 | eval_core/symbol.rs:62, stub → eval_needs | No abandonment; may retry the leaked autoload | S: alone and then q PANIC; next rejected |
| P0958 | messages/builtins_messages_inspect.rs:79, trial result → truth | Restoration inside trial has already abandoned new generation | V: True; q resumes; next=2 |
| P0959 | messages/builtins_messages_inspect.rs:107, trial evaluate | restore_evaluation_rollback_snapshot at :114; helper :247 abandons | V and valueq_loader: q resumes; next=2 |
| P1365 | suspension/core.rs:676, abandonment cleanup evaluate | Direct InputRequired taken into worklist :677-685; hidden Get input is missed | A direct: q resumes, next=2. cleanup_get_cancel: cancellation completes, next rejected |
| P2356 | rum-host/src/session.rs:221, restore → runtime restore | Reset is BEFORE load; conversion :231-232 has no abandon | H snapshot: owner remains, next rejected, ordinary resume Other. Proposed restore cleanup needed |
| P2367 | rum-jupyter/src/adapter/mod.rs:139, synchronous execute → request | No abandon; renders error but retains InputRequired outcome | H/source: next execute rejected; no synchronous PendingExecute/stdin continuation |
| P2368 | rum-jupyter/src/adapter/mod.rs:636, request → HostSession::eval | Passes live owner into synchronous response path | H/source: same blocked session; asynchronous worker is a separate path |
| P2374 | rum-cli/src/main.rs:40, main → run | No explicit abandon; local session ultimately dropped | Source: no reusable session after CLI exit |
| P2375 | rum-cli/src/main.rs:64, run → run_repl | No abandon while REPL retains borrowed session | Source: REPL blocked after input; local session dropped on exit |
| P2376 | rum-cli/src/main.rs:89, run → run_non_interactive | No abandon; noninteractive local session dropped | Source: process path exits; helper is not a reusable idle session |
| P2377 | rum-cli/src/main.rs:104, run_non_interactive → argument binding | No suspension with normal queued-literal setup :298-305 | Source: setup remains usable; script phase checked separately |
| P2378 | rum-cli/src/main.rs:106, run_non_interactive → eval_script_source | InputRequired rendered, no abandonment | H script/source: blocked until local session drops |
| P2379 | rum-cli/src/main.rs:110, run_non_interactive → stdin binding | One queued literal for one InputString :286-287 | Source: stock setup cannot exhaust queue; remains usable |
| P2380 | rum-cli/src/main.rs:112, run_non_interactive → argument binding | Exact prequeue, no reachable ordinary suspension | Source: setup usable |
| P2381 | rum-cli/src/main.rs:113, run_non_interactive → eval_source | Live owner rendered as error; local session dropped on return | H single eval/source: borrowed helper would remain blocked |
| P2382 | rum-cli/src/main.rs:117, run_non_interactive → stdin binding | Exact prequeue for literal input | Source: setup usable |
| P2383 | rum-cli/src/main.rs:119, run_non_interactive → argument binding | Exact prequeue for all generated InputStrings | Source: setup usable |
| P2384 | rum-cli/src/main.rs:120, run_non_interactive → eval_file | InputRequired rendered, no abandonment | H file/source: blocked until local session drops |
| P2385 | rum-cli/src/main.rs:218, run_repl → Repl::run | No abandon; borrowed session retained | Source: REPL loop blocked. Census target cli::run is an imprecise name match, corrected here |
| P2386 | rum-cli/src/main.rs:225, eval_source → eval_request helper | Helper renders input error without cleanup | H single eval/source: next rejected if caller retains session |
| P2387 | rum-cli/src/main.rs:225, closure → HostSession::eval | Returns resumable owner, then helper renders it | H single eval/source: next rejected; host resume could recover |
| P2388 | rum-cli/src/main.rs:232, eval_script_source → helper | No abandon at rendering boundary | H script/source: new request rejected; ordinary resume Other |
| P2389 | rum-cli/src/main.rs:233, closure → HostSession::eval_script | Batch owner retained without public resume checkpoint | H script/source: blocked; accepted cancellation recovers |
| P2390 | rum-cli/src/main.rs:241, eval_file → helper | No abandon at rendering boundary | H file/source: new request rejected; ordinary resume Other |
| P2391 | rum-cli/src/main.rs:242, closure → HostSession::eval_file | Batch owner retained without public resume checkpoint | H file/source: blocked; accepted cancellation recovers |
| P2392 | rum-cli/src/main.rs:253, eval_request → closure | InputRequired converted to CLI error at :272-277 | H/source: borrowed session stays blocked; normal CLI drops it |
| P2393 | rum-cli/src/main.rs:287, bind_script_input → setup helper | No reachable input failure with exact prequeue | Source: setup usable; generic helper's failure arm has no abandon |
| P2394 | rum-cli/src/main.rs:305, bind_script_args → setup helper | No reachable input failure with exact prequeue | Source: setup usable; strings are data, not evaluated source |
| P2395 | rum-cli/src/main.rs:312, setup helper → HostSession::eval | No abandonment in unexpected InputRequired arm :315 | Source: stock setup cannot trigger it; an unexpected suspension would block session |
| P2396 | rum-repl/src/commands.rs:354, trace → HostSession::eval | InputRequired rendered at :372, no abandon/resume | H/source: subsequent REPL evaluation rejected; enclosing host must resume/reset/cancel |
| P2397 | rum-repl/src/lib.rs:91, EOF pending source → evaluate_input | No abandon; run returns with borrowed session | Source: caller receives still-blocked HostSession |
| P2398 | rum-repl/src/lib.rs:134, loop → evaluate_input | No abandon/resume; loop continues | Source: later guest submissions rejected |
| P2399 | rum-repl/src/lib.rs:142, evaluate_input → HostSession::eval | Renders InputRequired at :147-149, no abandon | H/source: session blocked; no usable fresh request without host recovery |


The table covers 9 runtime pairs, 1 host pair, 2 Jupyter pairs, 22 CLI pairs,
and 4 REPL pairs: **38/38**. Setup bindings in the CLI have a stricter
reachability result than the old census: the stock code queues exactly the
literal strings it consumes, so its InputRequired arm is not reached by
ordinary command-line arguments. Deliberately installing different host
definitions would be a different setup, not evidence against this result.

The synchronous Jupyter path does retain the InputRequired outcome in the
response, but does not install a `PendingExecute`/stdin context; it is not
the asynchronous worker's resumable path. REPL rendering similarly does not
resume or abandon. CLI normally exits and drops its local session: no live
session remains to use, although its borrowed helper would leave one blocked
if reused. These are boundary disposition issues, not additional guest
continuation frames.

## 4. The resumable loading design

### One guest driver, separate batch policy

Introduce an internal guest file driver, conceptually
`start_guest_file_load(path) -> Result<Expr, EvalError>`. It reads once,
captures the existing loader snapshot, pushes the path, parses the script
plan, and lowers/evaluates **one form at a time**. Parsed syntax can be
retained; it must not be lowered ahead of context changes. The
`lower_later_forms` control returns 42 only because a later unqualified
binding is lowered after `Begin`. Text-only forms retain original source
spans and the original full source for diagnostics.

A private state machine has `StartForm(index)`, `AwaitForm(index)`,
`FinishForm` and `FinishFile` phases. Normal execution and resumption call
the same finish helpers. A form index advances only after that form has
completed; responding to input never reopens the provider or replays a
prefix. When another input occurs, the newly installed child frames precede
the existing outer suffix, as `core.rs:748-774` already requires.

Preserve the existing `SuspensionFrame::resume` / `FrameAction` /
`FrameAbandon` contracts. No continuation is run from Rust `Drop`. All new
frames derive `rum_derive::FrameRootTrace`; only actually pure frames derive
`PureFrameAbandon`. The loader and form-scope frames require explicit
structural abandonment.

### Concrete frame/state inventory

| Owner/frame | State carried across suspension | Completion / resuspension | Abandonment |
| --- | --- | --- | --- |
| `GuestScriptFormFrame` | `Arc<GuestScriptPlan>` containing original source and parsed/unparsed forms; current form index and next cursor; last completed `Expr` and `Option<EvalTerminationReason>`; loader-level message/print start indices and completed-form end indices; this form's message/print/handled-error checkpoints; previous `current_message_list_checkpoint` and dirty flag; prior last-message-list state where needed; shared snapshot of the caller evaluation-stack prefix, its length and logical depth; terminated-handling mode; previous guest-form scope marker | Finalize the resumed child under the loader's form policy, close this form scope, then drive the next form. On another suspension install only the new current-form frame; retain the existing file-close suffix | Restore the form's ambient message/error/stack scope, without finalizing a value or running another form |
| `GuestFileCloseFrame` | Resolved `PathBuf` and original source identity; `Arc<LoaderSnapshot>`; pre-push load-stack depth and expected path; invariant that this frame owns one active file transaction | Success: pop/check this entry once and discard snapshot. Terminal error: restore snapshot and pop/check once, then forward error. InputRequired never closes it | Structural helper restores this snapshot and removes exactly this load-stack entry, after child scopes have unwound |
| `GetCompleteFrame` | Resolved path/source for error diagnostics, stream range checkpoints, explicit guest completion policy | Return last form's value; streams are committed to the surrounding evaluation once. Ordinary loader errors emit existing Get message and return $Failed; Exit remains an error. InputRequired and cancellation propagate | Pure discard; transaction belongs to file-close frame |
| `NeedsCompleteFrame` | Normalized context, explicit load spec, no repeated argument evaluation | On $Failed emit Needs::load and return $Failed. Otherwise mark context loaded, prepend context path, return Null. Propagate input/errors | Pure discard: no post-load $Packages update |
| `BeginPackageDependencyFrame` | Target context, immutable normalized dependency list, current dependency index, saved begin result; original context/path and context-stack depth; loaded-package length checkpoint; identity of its active BeginPackage context frame | Complete the suspended dependency, check $Failed, then advance remaining dependencies. Only after the final dependency install final package context path and return saved result | Restore its own BeginPackage frame/context/path and loaded-package tail once, after any active dependency file has rolled back |
| `DeclaredStubLoadFrame` | Original resolved `Symbol` and the continuation phase “after Needs”; required capability-result check | Read that symbol's current own/delayed/dynamic value using a helper factored from `symbol.rs:63-83`. Do not restart the autoload preamble from this frame | Pure discard |
| Pending owner's guest-form environment | Cold-path optional snapshot of the **innermost live form**'s message-list checkpoint/dirty/last-list state, handled-error suffix/checkpoint, active interrupt/deadline/memory budget state needed during the paused form, and form scope identity | Reinstall before any inner frame executes on resume, including nested Input parsing; preserve fresh inner environment when appending an outer suffix | Restore before draining abandonment actions that need the dynamic form environment; enclosing scope frames then restore outward |

No untyped encoding of an EvalOutcome into a guest-visible Expr is needed:
the guest driver's result is the last value; the form state separately
retains termination while the script is active. Today's Get returns only
`outcome.value`, so the queued cycle probe produces
`TerminatedEvaluation["IterationLimit"]` with outer response
`termination == None`. Preserve that distinction.

For streams, prefer **session-owned buffers with stable ranges** during a
guest load. The accumulated messages and prints are the buffer ranges from
the file's checkpoints to the completed-form end indices; the unfinished
form has its own subsequent range. They are not cloned into a second vector
on every form. Form completion still updates `last_message_list` from the
current form's range. Get completion commits the already-present full
ranges once; this is the existing logical merge, with no duplicate append.
Public batch code may retain its existing detached EvalOutcome aggregation.
If implementation instead retains detached vectors, it must publish every
prefix before returning InputRequired, record published cursors, and never
re-splice them on later resumes; a mixture of both ownership models is
unacceptable.

On a terminal loader error, existing snapshot semantics win: loader messages
are rolled back to the captured message state before Get emits its failure
diagnostic; prints are not transactional and remain. The observable-error
control is deliberately `Print["before"]`, input, `Print["after"]`,
`Message[probe::tag]`, `Throw[9]`: queued result is $Failed, log/marker
restored, prints are both lines, and the surviving message is Get::load.
Nested inner failure restores only the inner snapshot, then the outer file
may continue, as the queued `nested_resume_then_error` result 88 demonstrates.

### The nested top-level entry: what is and is not checkpointed

The current `eval_input_with_terminated_handling` is not an ordinary builtin
helper. At `session_eval.rs:215-223` it:

- starts another interrupt budget;
- saves message, print and handled-error lengths;
- replaces current message-list checkpoint and dirty flag.

On success it splits messages/prints, updates last-message-list and truncates
handled errors (`:252-270`). On **all** errors, including suspension, it
restores outer message-list scope, updates last-message-list and truncates
handled errors (`:249-250,272-284`). Its caller also owns earlier detached
output (`:308-370`). None of that Rust-local work is carried today.

`begin_top_level_interrupt_budget` resets the poll counter, renews
`eval_deadline` from `eval_timeout`, sets `current_memory_limit` from
resource limits, potentially arms accounting, and derives `interrupt_armed`
from deadline or accepted request (`session_helpers.rs:30-38`). It does not
create a new accepted request and does not clear the evaluation stack.
`eval_request`, not the form helper, installs the resumable request checkpoint
(`session_requests.rs:216-247`); batch request methods do not install one.
The accepted ticket remains the outer request's ticket until outer completion.

There is **no special $Line counter** in this implementation. The probe binds
``Global`$Line=100``; each loaded-form read remains 100. Neither form entry nor
public request entry increments it. Do not import an undocumented
notebook-style line-counter behavior into this change.

At the actual input boundary, `core.rs:491-497` retains a copy of the active
stack, minus two entries, **including active outer guest/loader expressions**.
Ordinary evaluator unwinding pops the live stack
(`eval_core/mod.rs:74-88`). Resumption restores the captured copy
(`core.rs:585-610`). The form helper never establishes an independent stack.
Its continuation must restore the saved caller prefix and logical depth
before evaluating a new form, so previous form entries cannot falsely report
a cycle. Save the prefix entries themselves, not only a length: the initial input
capture intentionally removes two entries and may have removed an enclosing
entry needed at the next form boundary. Reinstall the original caller prefix
when advancing to the next form; do not simply truncate the shortened retained
stack or clear the whole stack. The prefix is shared across a file's form
frames and traced as real EvaluationStackEntry roots. `repeat_form_stack` is the regression for repeating the same
function in successive forms after two inputs.

Merely adding a **post-child** form frame does not restore the correct message
scope for child work that runs first. The pending owner's optional guest-form
environment in the table addresses that: capture it at actual input install,
before form/request unwinding overwrites the fields, and restore it in the
resume driver **after** the public resume wrapper has installed its root
checkpoint, **before** dispatching the first child frame. Capture the
innermost environment only; outer unwinding must not overwrite it.
`last_message_list` must be included because `$MessageList` falls back to it
when the current scope is not dirty (`eval_core/symbol.rs:255-271`).

Likewise, public failure packaging currently truncates handled-error state
even for InputRequired (`session_requests.rs:477-495`). Preserve the live
guest form's handled-error tail on suspension (or restore its saved exact
tail before resumption); do not retain an index into a shorter vector.
Rebase only through an explicit stream operation, never by clamping an
invalid index and silently dropping messages. `same_form_message_scope`,
`get_message_checkpoint`, `get_outer_check` and `get_outer_quiet` cover the
visible consequences; add private checkpoint/root assertions as specified
in the implementation draft.

Do not call the top-level initializer again when resuming the **same**
form: preserve its absolute deadline and enclosing limit state. New-form
budget renewal is a pre-existing issue, discussed in `8; the drafts do not
silently redefine it. The same accepted request remains active across all
file forms and nested loads. Cancellation targets that ticket, and file
completion never finishes it independently.

### Per-entry behavior and nesting

**Get.** Normalize/resolve once, read source once on the successful path,
start its file transaction, and evaluate forms with
`ConvertCyclesToValue`. Return/Abort/ProtectedAbort retain the current
per-form conversion; later forms still execute. Cycle detection becomes a
termination value at the loader form boundary. Ordinary non-cycle errors
roll the file snapshot back and become the existing Get diagnostics/$Failed.
Exit rolls the file back and propagates. Cancellation must roll back and
propagate through the loader instead of being converted to $Failed and
running a suffix; the outer request owns its Cancelled response.

**Needs.** The already-loaded short circuit remains. The continuation starts
after the pending Get; no context registration occurs just because input
was requested. Its current post-load test is only “was the value $Failed?”:
it does **not** validate that the source defined the requested context.
`needs_explicit` deliberately requests Other549 while the file defines
Dep549 and proves this. Do not invent a new context-validation rule.
After success update $Packages/path once; after ordinary error preserve
Get::load followed by Needs::load.

**BeginPackage.** Preserve the real ordering: it pushes its BeginPackage
frame, switches current context/path, and marks the target package loaded
**before** loading dependencies (`session_api.rs:870-889`). Its frame must
therefore carry rollback state from before this opening. On successful
dependency completion advance exactly once; on the final success set
`begin_package_context_path(target, dependencies)`. A failed dependency
rolls back the BeginPackage context/path and package-list tail, but does not
undo definitions from earlier successfully completed dependencies. The
`begin_dependency_second` / `begin_error_after_resume` controls leave the
earlier dependency's log effect 5, matching existing tests
(`tests/context_and_loader.rs:471-513`). An active dependency's loader
snapshot rolls back before the outer BeginPackage frame.

**Declared stub.** Suspend with the original resolved Symbol; after Needs,
resume into a factored **after-load** symbol-read helper. Do not return Null
(the Needs result) as the symbol's value and do not force a fresh
`evaluate_symbol` call through its autoload branch. The *enclosing
fixed-point evaluator* still controls legitimate retries. The queued
stub-error control performs two failed loads because the load/restore bumps
state; its two input responses, two diagnostic pairs and four prints are
intentional observed baseline behavior. Avoid changing that policy as an
incidental “retry fix.”

**Nested loads/two inputs.** Keep both paths on load_stack while the inner
file is suspended. Inner completion removes only its entry; a resumed read
of $InputFileName inside the inner form returns Inner549, and a following
outer read returns Outer549. Relative lookup after a resume must use that
same stack (`resolution.rs:258-270`). A second suspension constructs a
fresh pending owner and appends the still-owned suffix, not duplicate
file-close/BeginPackage frames. On final completion $InputFileName is empty
and the next public request is accepted.

**Abandonment/cancellation.** Add a narrow
`AbandonmentContext::rollback_guest_file(...)` structural operation and a
corresponding BeginPackage restore operation. They do not evaluate guest
code, emit loader failures or drive remaining forms. Child scopes unwind
before form scope, before file snapshot/load-stack closure, before the
package wrapper. WithCleanup's existing exceptional cleanup evaluation may
suspend again; that new load propagates InputRequired so the worklist drains
its loader frames before returning to the outer suffix. Check expected depth
and path; do not use a blind `pop` that can remove an outer file twice.

A frame is attached exactly once for a live transaction and is consumed by
exactly one driver transition: normal completion/error or abandonment.
Never place the same rollback obligation in both a Drop guard and a frame.
An Arc-held snapshot can be cloned on terminal rollback if required by the
driver's immutable Arc protocol; do not deep-clone the full snapshot on
every resume. When a form resuspends, transfer only its updated driver state;
the already-queued file-close frame remains the sole transaction owner.

**Reset and later requests.** Host reset currently rebuilds the entire
Session (`rum-host/src/session.rs:165-178`); Session's Drop is not an
abandonment driver (`session_state.rs:696-699`). Preserve that rule:
retiring the entire old session discards its stack, snapshots, owners and
accepted generations together, with no WithCleanup execution against the
new session. There is no surviving stack entry to pop twice. Partial
in-session abandonment uses the rollback actions above. Successful/error
guest completion and cancellation leave no pending owner or file entry and
permit a fresh request. Ordinary pending guest input rejects overlapping
requests, including a file request **before** provider access; the recorded
overlap control has `reads=[]`. Queueing input is not supersession.

### Retention and organization

This branch builds **no leases** and earns no new retention certificates.
FrameRootTrace is root tracing, not a capacity receipt. GuestScriptPlan's
syntax/source payload has no runtime Expr/Symbol roots; use an explicitly
documented root-free wrapper for syntax, without claiming its allocation
is accounted. LoaderSnapshot must acquire transitive root tracing for all
its real Expr/Symbol-bearing fields; do not mark the whole snapshot root-free.

Snapshot inventory: attributes, options, syntax metadata, declared stubs,
removed symbols/generations, Once values, gensym candidates, current
context/path/stack, search paths, packages, definitions and installed-system
value mirrors, upvalue flag, message definitions/selectors, exception
registry, symbol/sealed/source-unit counters and sealed units, trace
events/depths, messages and last-message-list. This is exactly
`session_state.rs:290-319`, not an approximate “definitions rollback.”
Prints, pending owner/request, dynamic evaluator scopes and active ticket
are separately owned.

Keep the original source/parsed-form plan/path in named payloads for R11,
and the snapshot/context rollback metadata/outcome handoff in named payloads
for R12. brief-547's N findings and deferred R11/R12 work remain open
(`brief-547-n-class-census-and-ranking.md:704-705,721-722`). Transfer/restore
boundaries, capacities, unique cleanup ownership and source lifetime should
be explicit so that later receipts can replace these raw retained payloads.

Suggested responsibility split: a guest script driver/form-scope module,
a file transaction/rollback module, and loader caller frames beside context
loading. Do not add another large frame inventory to session_eval.rs.
Assess actual implementation sizes against the organization guide when
implementing; the existing session/loader/core modules are already large.
The state-machine organization, not arbitrary line cuts, defines the split.


### Documentation delta

The reviewed normative passages are
`docs/guide/scoping-and-contexts.md:94-120` (Get/Needs and packages),
`docs/guide/evaluation.md:282-289` (resume the same input request),
`docs/guide/embedding.md:350-381` (queued input and non-resumable batch),
`docs/reference/builtins.md:446-455,1168-1177,2117-2129,2364-2373,3366-3375`
(BeginPackage, DeclarePackage, Get, InputString and Needs), and host rustdoc
`rum-host/src/session.rs:378-412`.

**None of those passages promises that guest input becomes Get::load/$Failed.**
The change makes no existing guest guide/reference sentence false; it removes
an undocumented exception to the general same-request suspension model.
The package guide and the four loader/autoload catalog entries need an
explicit note that source reached from a resumable eval request can suspend,
continue at the current form/dependency/symbol read, and return the normal
builtin result. Get::load remains for actual load/evaluation failures;
InputRequired must no longer be presented as one. Update the catalog source
and regenerate the reference together; do not hand-edit generated prose alone.
The host batch passages quoted above remain true, including rejection of new
requests; do not broaden the embedding guide's single-source resume examples
to promise batch resumption. Snapshot failure cleanup can be clarified without
changing its existing partial-state guarantee.

## 5. The eval_file split and host batch path after the change

Do **not** make public `Session::eval_file` the guest continuation entry.
Use the internal guest driver in `4 from Get. Keep the public
`eval_script/eval_file` and request wrappers as non-resumable batch entries.
Factor source parsing and ordinary form-finalization helpers where useful;
do not route guest loading through public request admission or install a
second `PendingEvalRequest`.

The host contract includes more than “resume fails”:
`rum-host/src/session.rs:378-412` and `docs/guide/embedding.md:364-380`
require InputRequired, no evaluation of remaining forms, Other on ordinary
resume, rejection of new requests, and reset/restore/replacement or accepted
cancellation as escape routes. **Abandoning and making the session idle would
violate that contract.**

Recommended ownership design: at a batch boundary, transfer the pending
owner into an explicit private **BlockedBatch** state that owns an
**abandonment-only** work item, the input request, stream/trace checkpoints,
and the accepted request identity/deadline when present. The resumable slot
and `pending_eval_request` are empty. The retained work item has no public
resume route; its purpose is to execute structural cleanup and existing
WithCleanup obligations on cancellation. Keeping that reachable, named
cleanup obligation is deliberate ownership, not an unclassified continuation
left behind by a completed request.

- Initial outcome remains InputRequired with buffered output. A direct
  input, or a guest Get anywhere in a batch form, stops that batch.
- `resume_with_input` reports Other without converting its argument or
  evaluating the rest of the file. Ordinary new requests reject before
  parsing, lowering, provider calls or deadline replacement. Queuing input
  does not change the blocked state.
- Accepted cancellation takes the blocked work item exactly once, drains
  abandonment inner-to-outer, finishes the old ticket, returns the current
  documented cancellation response, then permits the following request.
  Cancellation is checked before input conversion/new source. No guest
  completion frames are resumed.
- Reset/replacement drops the whole blocked session and all its payloads,
  preserving the existing no-guest-cleanup-on-reset rule.
- Prompt observation, HostSession timeout ownership, configuration/read-only
  inspection, request admission, cancellation and generation checks must
  recognize **both** resumable and blocked-batch states. The guard at
  `session_requests.rs:72` cannot continue to equate “no resumable generation”
  with “idle.” `queue_input_response` currently clears the prompt
  (`session_api.rs:286-294`); it must not clear the blocked disposition.
- An ordinary error without a pending input follows the existing failure
  path and does not create a blocker.

For a public file batch that suspends, attach its file transaction's
rollback obligation to the abandonment-only work item, **after** any nested
guest-load cleanup and **before** outer request retirement. This prevents
restoring an outer snapshot and then resurrecting intermediate state when a
nested frame later restores its own snapshot. On normal success/error use
the existing immediate commit/rollback. This explicitly changes the
*internal timing* of public eval_file's current eager rollback on
InputRequired: the rollback now happens when the blocked batch is abandoned.
The documented host non-resumability/rejection/cancellation contract is
preserved. Whether that eager rollback is itself a required additional
contract for direct Session users is an open question in `8; do not conceal
it in an implementation brief.

If eager rollback on the initial batch failure must be preserved, the
alternative is to abandon the entire child continuation immediately,
inner-to-outer, then restore the outer snapshot, and keep a lightweight
BlockedBatch marker without frames. That also preserves the documented
blocking/resume contract and leaves no live continuation owner, but runs
WithCleanup earlier. It must not be substituted silently: the two choices
differ observably in cleanup timing. There is no honest design that discards
all owners, preserves delayed cleanup in scoped environments, and stores no
cleanup ownership.

**Snapshot restore is different.** `HostSession::restore_from_bytes` resets
before evaluating, then converts InputRequired to a restore failure
(`session.rs:213-237`); it currently does **not** reset or abandon afterward.
Its source is no longer a pending public input operation. Before returning
that failure, explicitly drain its owned pending/BlockedBatch work, clear
the prompt/checkpoints, and retire the accepted identity if any. Preserve
already installed snapshot definitions (the documented nontransactional
partial state); do not reset a second time. Do not abandon a pre-existing
unrelated request if restore admission failed. The runtime restore helper
can own this policy so the host wrapper only translates the failure.

Thus guest completion owns real resumable continuations; batch suspension
owns a reachable cancellation-only work item; restore failure retires its
work. No route may report a completed value or terminal failure while an
unclassified input owner remains installed.

## 6. The red tests

The archive `.temp/brief-549/probe.rs` contains executable red tests, not just
proposed names. Final run: **44 deliberately failing tests, 6 passing
diagnostic/positive tests**. No production repair was attempted.

For the 33 oracle tests, the function name is `red_` plus the matching
suffix in the table below. Every test first requires the measured number of
actual public InputRequired results, then compares typed Expr values,
termination, full RuntimeMessage values, buffered prints, side-effect state
and acceptance/result of a fresh `1+1` request against a fresh queued-input
session. The diagnostic path catches the known kernel panic so the other
cases still run; a panic remains a failing observation. No suspended path
was invented for a loader that returned $Failed.

| Named red test | Public suspensions: actual / required | Unqueued result; log | Queued result; log | Queued messages / prints | Fresh request: unqueued / queued |
| --- | ---: | --- | --- | --- | --- |
| ``red_get`` | 0 / 1 | ``$Failed; List[]`` | ``77; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_get_then_q`` | 0 / 2 | ``PANIC; List[]`` | ``"two"; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_needs`` | 0 / 1 | ``$Failed; List[]`` | ``Null; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_needs_then_q`` | 0 / 2 | ``PANIC; List[]`` | ``"two"; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_begin_package`` | 0 / 1 | ``$Failed; List[]`` | ``"Main549`"; List[1, 3, 5]`` | ``[] / []`` | False / True |
| ``red_begin_package_then_q`` | 0 / 2 | ``PANIC; List[]`` | ``"two"; List[1, 3, 5]`` | ``[] / []`` | False / True |
| ``red_stub`` | 0 / 1 | ``PANIC; List[1]`` | ``42; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_stub_then_q`` | 0 / 2 | ``PANIC; List[]`` | ``"two"; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_get_streams_line`` | 0 / 1 | ``$Failed; List[]`` | ``77; List[List[1, 100, "./Obs549.rum"], List[3, 100, "./Obs549.rum"]]`` | ``[RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }] / ["before", "after"]`` | False / True |
| ``red_get_twice`` | 0 / 2 | ``$Failed; List[]`` | ``77; List[1, 3, 5]`` | ``[] / []`` | False / True |
| ``red_needs_explicit`` | 0 / 1 | ``List[Dep549`foo, List[]]; List[]`` | ``List[42, List["Other549`"]]; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_begin_dependency_second`` | 0 / 1 | ``$Failed; List[5]`` | ``"Main549`"; List[5, 1, 3]`` | ``[] / []`` | False / True |
| ``red_begin_two_dependencies_suspend`` | 0 / 2 | ``$Failed; List[]`` | ``"Main549`"; List[1, 3, 5, 7]`` | ``[] / []`` | False / True |
| ``red_nested_twice_filename`` | 0 / 2 | ``PANIC; List[List[1, "./Outer549.rum"], List[5, "./Outer549.rum"]]`` | ``77; List[List[1, "./Outer549.rum"], List[2, "./Inner549.rum"], List[4, "./Inner549.rum"], List[5, "./Outer549.rum"], List[7, "./Outer549.rum"]]`` | ``[] / []`` | False / True |
| ``red_nested_resume_then_error`` | 0 / 1 | ``88; List[0, 5]`` | ``88; List[0, 5]`` | ``[RuntimeMessage { symbol: "Get", tag: "load", text: "uncaught throw: 9 with tag None" }] / ["before", "after"]`` | False / True |
| ``red_get_error_after_resume`` | 0 / 1 | ``$Failed; List[]`` | ``$Failed; List[]`` | ``[RuntimeMessage { symbol: "Get", tag: "load", text: "uncaught throw: 9 with tag None" }] / ["before", "after"]`` | False / True |
| ``red_needs_error_after_resume`` | 0 / 1 | ``$Failed; List[]`` | ``$Failed; List[]`` | ``[RuntimeMessage { symbol: "Get", tag: "load", text: "uncaught throw: 9 with tag None" }, RuntimeMessage { symbol: "Needs", tag: "load", text: "context `Bad549`` could not be loaded" }] / ["before", "after"]`` | False / True |
| ``red_begin_error_after_resume`` | 0 / 1 | ``$Failed; List[5]`` | ``$Failed; List[5]`` | ``[RuntimeMessage { symbol: "Get", tag: "load", text: "uncaught throw: 9 with tag None" }, RuntimeMessage { symbol: "Needs", tag: "load", text: "context `Bad549`` could not be loaded" }] / ["before", "after"]`` | False / True |
| ``red_stub_error_after_resume`` | 0 / 2 | ``PANIC; List[1]`` | ``Bad549`foo; List[]`` | ``[RuntimeMessage { symbol: "Get", tag: "load", text: "uncaught throw: 9 with tag None" }, RuntimeMessage { symbol: "Needs", tag: "load", text: "context `Bad549`` could not be loaded" }, RuntimeMessage { symbol: "Get", tag: "load", text: "uncaught throw: 9 with tag None" }, RuntimeMessage { symbol: "Needs", tag: "load", text: "context `Bad549`` could not be loaded" }] / ["before", "after", "before", "after"]`` | False / True |
| ``red_get_input_expression`` | 0 / 1 | ``$Failed; List[]`` | ``77; List[1, 3]`` | ``[] / []`` | False / True |
| ``red_get_input_syntax_error`` | 0 / 1 | ``$Failed; List[]`` | ``$Failed; List[]`` | ``[RuntimeMessage { symbol: "Get", tag: "syntax", text: "expected expression, found ] at line 1, column 1 (bytes 0..1)" }] / []`` | False / True |
| ``red_get_late_syntax_error`` | 0 / 1 | ``$Failed; List[]`` | ``$Failed; List[]`` | ``[RuntimeMessage { symbol: "Get", tag: "syntax", text: "expected expression, found * at line 3, column 3 (bytes 38..39)" }] / []`` | False / True |
| ``red_get_return_form`` | 0 / 1 | ``$Failed; List[]`` | ``77; List[3]`` | ``[] / []`` | False / True |
| ``red_get_abort_form`` | 0 / 1 | ``$Failed; List[]`` | ``77; List[3]`` | ``[] / []`` | False / True |
| ``red_get_exit_form`` | 0 / 1 | ``$Failed; List[]`` | ``View { status: "Exit { code: 7 }", value: None, termination: "None", messages: [], prints: [] }; List[]`` | ``[] / []`` | False / True |
| ``red_get_cycle_form`` | 0 / 1 | ``$Failed; List[]`` | ``TerminatedEvaluation["IterationLimit"]; List[]`` | ``[RuntimeMessage { symbol: "General", tag: "passlim", text: "evaluation cycle detected while re-entering `f549[]`" }] / []`` | False / True |
| ``red_get_dynamic_context`` | 0 / 1 | ``0; List[]`` | ``0; List[20, 20]`` | ``[] / []`` | False / True |
| ``red_get_message_checkpoint`` | 0 / 1 | ``List[HoldForm[probe::tag], HoldForm[Get::load]]; List[]`` | ``List[HoldForm[probe::tag], HoldForm[probe::tag], HoldForm[probe::tag]]; List[List[1, 100, "./Obs549.rum"], List[3, 100, "./Obs549.rum"]]`` | ``[RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }] / ["before", "after"]`` | False / True |
| ``red_get_outer_check`` | 0 / 1 | ``999; List[]`` | ``999; List[List[1, 100, "./Obs549.rum"], List[3, 100, "./Obs549.rum"]]`` | ``[RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }] / ["before", "after"]`` | False / True |
| ``red_get_outer_quiet`` | 0 / 1 | ``$Failed; List[]`` | ``77; List[List[1, 100, "./Obs549.rum"], List[3, 100, "./Obs549.rum"]]`` | ``[] / ["before", "after"]`` | False / True |
| ``red_same_form_message_scope`` | 0 / 1 | ``$Failed; List[]`` | ``77; List[List[HoldForm[probe::tag]], List[HoldForm[probe::tag]]]`` | ``[RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "probe message" }] / []`` | False / True |
| ``red_lower_later_forms`` | 0 / 1 | ``$Failed; List[]`` | ``42; List[]`` | ``[] / []`` | False / True |
| ``red_repeat_form_stack`` | 0 / 2 | ``$Failed; List[]`` | ``77; List[1, 3, 1, 3]`` | ``[] / []`` | False / True |


All currently fail at the actual-suspension count (0), after recording both
paths, or encounter the captured owner assertion. The two-input requirements
include `get_twice`, `begin_two_dependencies_suspend`,
`nested_twice_filename`, `repeat_form_stack`, `stub_error_after_resume` and
all four `*_then_q` tests. In the unfixed implementation none reaches its
first *public* loader suspension, so later phase assertions are red but
blocked by that earlier defect. The queued path supplies concrete expected
values/effects; this report does not claim it observed an after-resume
loader rollback.

The remaining eleven named red tests and their measured pre-fix failures:

| Test | Actual pre-fix failure | Required completion behavior |
| --- | --- | --- |
| `red_get_cancel_restores` | Initial result $Failed, no public suspension; old cancel handle no longer recovers the leaked owner | InputRequired, cancel without using supplied input, restore file snapshot/pop, accept next request |
| `red_needs_cancel_restores` | Same, plus Needs::load | Same; no post-load package registration |
| `red_begin_cancel_restores` | Same; initial BeginPackage rollback already ran | Cancel active dependency and restore BeginPackage structural state once |
| `red_nested_cancel_restores` | Outer load returns 99 with pending inner input; cancellation cannot recover | Cancel both file transactions inner-to-outer |
| `red_stub_cancel_restores` | Owner-install assertion during initial autoload | Cancel before symbol value read; leave no file entry |
| `red_get_cancel_at_both_inputs` | First expected suspension missing ($Failed) | Independently cancel at first and second file input; log=[], marker=0, filename empty, next request=2 |
| `red_needs_cancel_at_both_inputs` | First expected suspension missing ($Failed) | Same plus no Needs completion |
| `red_begin_cancel_at_both_inputs` | First expected suspension missing ($Failed) | Same plus context/package-frame restoration |
| `red_stub_cancel_at_both_inputs` | Owner-install assertion at first load | Same plus no autoload replay/value-read suffix |
| `red_cancel_cleanup_get_releases_owner` | **Actually suspends**, cancellation returns TerminatedEvaluation["Cancelled"], but pending input is still present | Nested cleanup load abandoned exactly once; a fresh q suspension/resume and then 1+1 both work |
| `red_batch_guest_get_reports_input_without_panic` | Batch Get followed by input panics at owner install | Batch reports first input, ordinary resume remains Other and new requests remain rejected |

The 45-case diagnostic table in `2 includes exact current values; the
separate full stream/state table is linked there. Particular comparisons:

- `get_streams_line`: queued log
  `List[List[1, 100, "./Obs549.rum"], List[3, 100, "./Obs549.rum"]]`,
  messages are two `probe::tag` records, prints `["before","after"]`.
  Unqueued gives $Failed, empty log, one Get::load and only `["before"]`.
- `nested_twice_filename`: queued log has Outer at 1, Inner at 2 and 4,
  Outer at 5 and 7; output 77 and final filename empty. Unqueued panics
  before completing either resumed load.
- `get_error_after_resume` and `needs_error_after_resume`: queued log/marker
  roll back to []/0, both prints survive; the ordinary-error diagnostic
  replaces the spurious input-required diagnostic. BeginPackage's successful
  earlier dependency leaves log [5].
- `stub_error_after_resume`: the valid control needs **two** inputs; final
  value is ``Bad549`foo``, with two Get/Needs diagnostic pairs and
  `["before","after","before","after"]`, then a fresh request works.
- `get_late_syntax_error` uses the **hard** error `1+*2` and records line 3,
  column 3 (bytes 38..39). The earlier incomplete-tail fixture in run 02
  failed during splitting, before input; it is explicitly excluded as an
  after-resume test.
- Return and Abort are converted per form, so their controls finish at 77
  and log [3]. Exit yields `Exit { code: 7 }` with the file rolled back.
  The cycle value is preserved without upgrading Get's outer termination
  field.

Already-green behavior must remain green: direct ValueQ/Monitor discards,
nested cleanup input discard, q's overlap rejection, normal batch
non-resumability, batch cancellation recovery, and Session replacement.
Best-effort Monitor intentionally consumes queued callbacks and prints
differently from discarded callbacks; raw equality with the queued path is
**not** its existing contract. Its oracle is the expected q prompt followed
by accepted completion, not forcing discarded work to run. Only resumable
guest loading receives the queued-versus-resumed equality requirement.

Additional permanent white-box/host tests are specified in the drafts:
frame roots, exact load-stack depths, one rollback/pop per transaction,
no pending/resuming/worklist owner after terminal completion, canceled
request identity, deadline preservation within a paused form, reset of a
suspended loader, and snapshot restore failure cleanup. Those tests could
not access private runtime fields or the HostSession type from this
authorized integration target. Their pre-fix foundations are the saved
public probes and cited source; they are **proposed tests, not claimed
executed gates**. Reset itself is an already-green control, not an invented
pre-existing failure.


## 7. Drafted implementation briefs

The following are raw material for the director's rewrite, in TEMPLATE.md
shape. They are not dispatched here. Each begins with red tests, states an
objective/fence/disclosure duty, uses .temp/brief-NNN checkpoints, has a
progress-based stop-loss, and names allocation/instruction/timing gates.
The first draft keeps the guest driver and all callers in one atomic unit;
the second owns the unchanged non-resumable host boundary. The standalone
copies are [draft-guest-loading.md](../../../.temp/brief-549/draft-guest-loading.md)
and [draft-batch-ownership.md](../../../.temp/brief-549/draft-batch-ownership.md).



---

# Draft brief-NNN — resumable guest file loading through all package callers

## Goal

Start from the red tests in brief-549: Get, Needs, BeginPackage dependency
loading and DeclarePackage autoload must actually suspend, resume without
replaying forms, and equal their queued-input controls. Implement the whole
guest continuation chain atomically, including structural rollback on
failure/cancellation and no live owner after completion. Do not land a Get
change while Needs/BeginPackage/stub callers still discard their completion
work. Public batch non-resumability remains a separate boundary.

## Context pointers

- Read AGENTS.md, docs/testing-strategy.md and
  docs/code-organization-guidelines.md first.
- brief-549 ``3–6 contain source citations, full frame inventory, current
  host contract and pre-fix evidence. Archived runnable sources:
  .temp/brief-549/probe.rs; exact cases: case-sources.json; all raw streams:
  probe-05-final.txt and streams-and-state.md.
- Runtime session/session_eval.rs owns form/file evaluation;
  session_helpers.rs/session_state.rs own loader snapshots;
  builtins_context_loader.rs owns caller completion;
  eval_core/symbol.rs owns the stub's value-read continuation;
  suspension/core.rs owns queue/driver/abandonment.
- Normative docs: guide/scoping-and-contexts.md, guide/evaluation.md,
  guide/embedding.md, generated builtin reference and host session rustdoc.

## Scope

**Objective:** implement the guest-only loader driver and every caller
continuation/structural cleanup required to complete that driver.

**Fence:** no batch resumability, overlapping-request supersession, leases,
retention-certification promotion, Expr/Symbol/serde redesign, new builtin
semantics, adapter UI changes, unrelated continuation-sweep repairs,
dependency/lockfile changes or default-path diagnostics. Changes to tests,
catalog documentation and guide prose are authorized only to state/prove
guest resumable loading and its existing surrounding semantics. Do not edit
AGENTS.md, root PROGRESS.md, TODO.md, unrelated docs or earlier briefs.

**Disclosure duty:** record every changed module and why it is necessary,
every new frame field/root edge, any changed existing synchronous behavior,
performance exposure, and every uncovered callback/error path. Stop and
report a needed change outside this objective/fence. Preserve unrelated
worktree edits. Append the report to the dispatched brief; scratch belongs
only under .temp/brief-NNN/.

## Deliverable

1. **Red first:** install permanent versions of all 33 brief-549 oracle
   cases, the five caller-cancellation regressions, four two-position
   cancellation tests, and red_cancel_cleanup_get_releases_owner. Run and
   save their failures before implementation. Retain the explicit
   actual-suspension assertions, full streams and next-request checks.
   Keep already-green best-effort/overlap/batch controls as controls.
2. A guest script plan/cursor, per-form completion/scope handling, one
   file transaction owner, and Get/Needs/BeginPackage/stub continuations
   carrying precisely the `4 state. Lower forms only when reached.
3. A cold-path pending guest-form environment restored before inner
   continuation dispatch. Carry message-list and handled-error state,
   evaluation-stack boundaries and the paused form's budget/identity.
4. Shared finish/error helpers for initial and resumed execution;
   FrameRootTrace derives and exact structural FrameAbandon implementations.
   No full-snapshot clone on each input. No second rollback owner in Drop.
5. White-box tests for inner-to-outer restoration, two nested files, both
   suspension positions, exact pop/restore counts, pending/resuming/worklist
   emptiness, FrameRootTrace liveness of definitions and buffered values,
   cancellation while awaiting input and during resumed work, request
   identity, $MessageList scopes and paused-form deadline preservation.
   A test print sink must not receive replayed prefix output.
6. Host tests through HostSession::eval for resumed loading, no poisoning,
   reset/replacement from either suspended position, canceled old handles,
   and accepted fresh requests. Reset stays a rebuild, without guest cleanup.
7. Documentation of the four guest entries' suspension semantics in the
   guide and catalog/reference, without changing the host batch contract.
   Name the module-size/split assessment and save all measured gates.

## rum / Rust pitfalls (read twice)

1. Start with the named reds; a source containing InputString is not proof
   it suspended. Assert prompt order/count before comparing results.
2. A post-child frame alone cannot restore the inner message scope before
   its child executes. Preserve/reinstall the cold pending environment.
3. Preserve current ConvertCyclesToValue form semantics, Return/Abort,
   Exit, ordinary-error rollback and Get's outer termination field.
4. Context changes occur before BeginPackage dependencies; successful earlier
   dependencies are not an all-or-nothing transaction. Stub resumption starts
   after Needs; the fixed-point evaluator retains its existing retry policy.
5. InputRequired is propagation, not failure closure. Cancellation propagates
   through all loader callers after rollback; it must not run a success suffix.
6. FrameRootTrace traces roots; it proves no lease or capacity receipt.
   No new unsafe, instrumentation cost or eagerly lowered/cloned source tree.
7. Progress counters: named red tests now passing with actual suspension;
   completed caller continuation paths; proved rollback/root invariants;
   instruction/allocation regression rows eliminated. Report after each
   attempt. Stop after **two consecutive compiling attempts improve none**.
   Reducing compiler errors is progress; do not apply the stop-loss to
   converging compiler repairs. Preserve evidence and report unresolved reds.

## Verification (MANDATORY)

First run the focused reds and save raw output. Iterate until the guest
regressions and existing package/suspension tests pass. Then run bounded:
```text
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test -p rum-host
cargo test --workspace
```
Wrap every command in scripts/run-bounded.py with bounds derived from measured
runs of its shape; paste commands, bounds, real exit statuses and test counts.
Do not mark a permanently incomplete suite green.

Exposed existing microbench labels:
symbol-ownvalue-read, symbol-ownvalue-guarded-read, set-ownvalue,
literal-integer, literal-string, downvalue-exact-hit, downvalue-exact-miss,
downvalue-pattern-hit, downvalue-pattern-miss, if-true, block-local,
module-local, do-count-null, do-iterator-null, table-iterator-null,
valueq-undefined-symbol-definitions-2000 and
valueq-undefined-symbol-definitions-8000. The symbol read/guard/set and
exact/pattern hit/miss panels are same-tier neighbors, not optional extras.
No dedicated existing loader label exists; add a scratch loader operation
panel (queued normal completion, input/resume, error/abandon) as a supplement,
never as a replacement for the existing suite.


Measurement order for **each compiling implementation iteration**: allocation
counts first, per-iteration retired user instructions second, timing third.
Before editing, preserve base default and alloc-count release binaries under
`.temp/brief-NNN/base/`; do not use git/checkouts to reconstruct a baseline.
Keep all generated source under that scratch directory with a `.rum` suffix.

Use the existing optional `rum-cli/alloc-count` build with
`RUM_PROFILE_ALLOC=1` only for the allocation lane. Save count, bytes_total and
peak_live_bytes from its raw stderr; do not add counters or unsafe code.
Build a default binary without alloc-count for instruction and timing lanes.
For each exposed label use its exact statement from
`scripts/core-language-microbench.sh`, with identical setup and loop counts
in base/candidate. Do not add a sink assignment or Print to the statement.
Save counts at N and 2N and report both total/N and the amortized
`(count(2N)-count(N))/N`, with the same harness on both builds.

Instruction command shape, run separately per label and build:
```bash
python3 scripts/run-bounded.py --timeout "$measured_bound" -- \
  taskset -c 2 perf stat -x, -e instructions:u \
  -o ".temp/brief-NNN/instructions/label-build-N.csv" -- \
  ".temp/brief-NNN/build/rum-cli" --file ".temp/brief-NNN/label-N.rum"
```
Do not aggregate the whole suite into one instruction count and call it a
per-label result. Report per-iteration base/new deltas. If CPU 2 or perf
counters are unavailable in the sandbox, save that failure and report the
gate unavailable; no elevation, alternative unpinned claim, or substitution
of timing for instructions.

With profiling env vars off, run the required complete
`scripts/core-language-microbench.sh` A/B with explicit in-repository
`--output-dir`, `--rum-cli`, and measured `--timeout/--build-timeout` values,
pinned to CPU 2. Compare per-label `timings.tsv`, including every named
neighbor and all other hot labels. Timing is secondary evidence and must
come from the script's in-kernel monotonic AbsoluteTiming. Never difference
wall-clock timestamps or decide timings under another CPU-heavy workload.
At most two complete timing A/B pairs per brief, at most two instruction and
allocation samples per (label, N, build) per compiling iteration. An anomalous
or unavailable measurement is reported, not chased with unlimited reruns.
Any hot/default-path or same-tier-neighbor regression is a defect to fix or
report. After a meaningful edit rerun the same focused lanes; do not rerun
unrelated gates after they pass without a new reason.


## Checkpointing — protect the scene

After each milestone write .temp/brief-NNN/PROGRESS.md with verified work,
saved raw evidence, the exact next action, compile/test state, counters,
half-edited files and whether any scratch test still exists. Use only .rum
for generated guest sources. Preserve base binaries before implementation.

## Hard rules

- No git in any form; no commit/branch/push/config changes.
- Modify only what the objective/fence authorizes; preserve user edits.
- Touch nothing outside the repository; no network.
- Commands: repository file reads/edits, rg/grep/python3, bounded cargo,
  the repository microbench scripts and bounded taskset/perf/release CLI
  commands explicitly required above. No unbounded child process.
- Testing Policy is permanent: rum against rum only; no third-party engines.
- No new unsafe. No leases or retention claims. No unconditional diagnostics
  or default-path allocations introduced to simplify suspended-state handling.
- Reject/regress/report according to the named measurement and stop-loss gates.

## Report format

1. Changes and complete caller/frame/root/rollback inventory, with path:line.
2. Red-before/green-after output and exact resumed/queued values, log,
   termination, messages, prints, prompt counts and fresh-request acceptance.
3. Allocation counts, per-iteration instructions and per-label timings;
   all neighbor deltas and any unavailable gate.
4. Scope/size disclosure, progress counters, unresolved risks and questions.


---

# Draft brief-NNN — explicit ownership at non-resumable batch/restore boundaries

## Goal

Start from brief-549's batch-nested-Get panic and snapshot-owner evidence.
Preserve the documented non-resumable HostSession batch API while making
its retained cleanup ownership explicit. Snapshot restore must abandon its
failed input work before converting it into a terminal restore failure.
This draft follows the completed guest loader chain.

## Context pointers

- AGENTS.md; docs/testing-strategy.md; code organization guide.
- brief-549 ``3,5,6,8, .temp/brief-549/probe-05-final.txt, the BOUNDARY,
  BATCH_GUEST and CANCEL rows, and archived red tests.
- session/session_requests.rs: admission, pending checkpoints, batch wrappers,
  cancellation and response packaging.
- session/session_eval.rs: public batch file snapshot lifecycle.
- session/session_api.rs: restore mode and queued input.
- suspension/core.rs: ownership transfer and abandonment worklist.
- rum-host/src/session.rs: batch rustdoc, reset, restore conversion and timeout.
- guide/embedding.md:364-380 is normative for batch blocking/cancellation.

## Scope

**Objective:** make every batch/restore input outcome own either an explicit
non-resumable cleanup operation or no operation after abandonment, preserving
the documented public batch behavior.

**Fence:** no public batch resume, accepting overlaps, running remaining forms,
leases, unrelated adapter changes, snapshot-format/Expr/public serde changes,
dependencies, lockfile edits, general continuation-result redesign or changes
to reset's no-guest-cleanup behavior. Documentation edits may describe this
boundary ownership and snapshot failure cleanup only; no other roadmap/docs
changes. Guest loading itself is the preceding brief's completed dependency.

**Disclosure duty:** enumerate every admission/cancellation/prompt/timeout
call site affected and all retained root payloads. Explicitly resolve `5/`8's
public-file rollback-timing choice in the director's rewritten brief **before
implementation**, preserving non-resumability in either case. State any
observable lifecycle change; do not hide it as a refactor. Keep scratch and
checkpoints under .temp/brief-NNN/, report appended only to the dispatched
brief, and preserve unrelated worktree changes.

## Deliverable

1. **Red first:** reproduce red_batch_guest_get_reports_input_without_panic
   with script, file and restore sources, plus a host test
   restore_input_failure_abandons_owner_preserves_partial_state: restore a
   valid prefix followed by InputString, observe Failure, inspect prefix
   state, then require a fresh input/resume and 1+1. The current source leaves
   the owner installed. Record real failure before implementation.
2. White-box red tests for explicit batch ownership: no resumable owner or
   public resume checkpoint after transition; exactly one blocked cleanup
   owner (or marker under the selected immediate-abandon policy); nested
   guest/file rollback order, no unreachable owner after cancellation or
   restore failure. Do not substitute an inaccessible owner for the marker.
3. A named private blocked-batch disposition. It retains prompt, checkpoints,
   accepted identity/deadline and precisely the cleanup obligation selected
   by the director. All normal new requests reject, before any parsing or I/O.
4. Cancellation takes/drains owned cleanup exactly once and returns the old
   request's cancellation result before accepting a following request.
   Queued input cannot supersede or resume the blocked batch.
5. Snapshot failure explicitly abandons only its own operation; prior valid
   snapshot definitions remain installed, invalid UTF-8 leaves the original
   session untouched, and no second reset is introduced.
6. Host tests for both direct input and guest Get in batch, queue-after-block,
   overlapping eval/call/file/timeout entries, cancellation before argument
   conversion, no provider read on rejected file request, stale handles,
   reset/replacement and preserved no-guest-cleanup-on-reset behavior.
7. Keep CLI/REPL/synchronous Jupyter disposition issues documented as findings
   unless the director commissions their separate policy change. Do not
   implicitly add interactive stdin behavior to those adapters.

## rum / Rust pitfalls (read twice)

1. Non-resumable does not mean idle: the host contract explicitly rejects new
   requests until reset/replacement or accepted cancellation.
2. A prompt field is not an owner. Moving to an abandonment-only owner must be
   a real typed transition, with exhaustive rooting and eventual disposal.
3. Never restore an outer file snapshot before still-live inner rollback
   actions that can later reintroduce intermediate state.
4. Preserve active accepted identity and owned deadline through blocked state.
   Cancellation wins before input conversion, parsing, provider I/O or new work.
5. Do not run WithCleanup on reset; whole-session retirement is different from
   in-session cancellation/abandonment. Restore failure does abandon.
6. Partial snapshot state is documented; a second reset “to fix the leak”
   would weaken that contract.
7. Progress counters: failed boundary regressions made green; unclassified
   owner conversions eliminated; admission/cancellation call sites covered;
   allocation/instruction regressions eliminated. Stop after two consecutive
   compiling attempts improve none. Compiler-error reduction is progress.
   Save evidence rather than guessing that another owner flag solved it.

## Verification (MANDATORY)

Capture the red failures first. Then bounded focused runtime and host tests,
followed by:
```text
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-host
cargo test --workspace
```
Use scripts/run-bounded.py for every command with measured bounds. Native
ZeroMQ skips in a restricted sandbox are permitted; report actual skips.
Do not run another vendor's engine.

This touches request/evaluation entry and suspension ownership; measure the
existing literal-integer, literal-string, loop-empty, do-count-null,
symbol-ownvalue-read, symbol-ownvalue-guarded-read, set-ownvalue,
downvalue-exact-hit, downvalue-exact-miss, downvalue-pattern-hit,
downvalue-pattern-miss, block-local, do-iterator-null and table-iterator-null
labels and their same-tier neighbors. Add scratch request/cancel/blocked-batch
operation counts only as supplemental coverage.


Measurement order for **each compiling implementation iteration**: allocation
counts first, per-iteration retired user instructions second, timing third.
Before editing, preserve base default and alloc-count release binaries under
`.temp/brief-NNN/base/`; do not use git/checkouts to reconstruct a baseline.
Keep all generated source under that scratch directory with a `.rum` suffix.

Use the existing optional `rum-cli/alloc-count` build with
`RUM_PROFILE_ALLOC=1` only for the allocation lane. Save count, bytes_total and
peak_live_bytes from its raw stderr; do not add counters or unsafe code.
Build a default binary without alloc-count for instruction and timing lanes.
For each exposed label use its exact statement from
`scripts/core-language-microbench.sh`, with identical setup and loop counts
in base/candidate. Do not add a sink assignment or Print to the statement.
Save counts at N and 2N and report both total/N and the amortized
`(count(2N)-count(N))/N`, with the same harness on both builds.

Instruction command shape, run separately per label and build:
```bash
python3 scripts/run-bounded.py --timeout "$measured_bound" -- \
  taskset -c 2 perf stat -x, -e instructions:u \
  -o ".temp/brief-NNN/instructions/label-build-N.csv" -- \
  ".temp/brief-NNN/build/rum-cli" --file ".temp/brief-NNN/label-N.rum"
```
Do not aggregate the whole suite into one instruction count and call it a
per-label result. Report per-iteration base/new deltas. If CPU 2 or perf
counters are unavailable in the sandbox, save that failure and report the
gate unavailable; no elevation, alternative unpinned claim, or substitution
of timing for instructions.

With profiling env vars off, run the required complete
`scripts/core-language-microbench.sh` A/B with explicit in-repository
`--output-dir`, `--rum-cli`, and measured `--timeout/--build-timeout` values,
pinned to CPU 2. Compare per-label `timings.tsv`, including every named
neighbor and all other hot labels. Timing is secondary evidence and must
come from the script's in-kernel monotonic AbsoluteTiming. Never difference
wall-clock timestamps or decide timings under another CPU-heavy workload.
At most two complete timing A/B pairs per brief, at most two instruction and
allocation samples per (label, N, build) per compiling iteration. An anomalous
or unavailable measurement is reported, not chased with unlimited reruns.
Any hot/default-path or same-tier-neighbor regression is a defect to fix or
report. After a meaningful edit rerun the same focused lanes; do not rerun
unrelated gates after they pass without a new reason.


## Checkpointing — protect the scene

After each milestone write .temp/brief-NNN/PROGRESS.md: completed/verified
facts, exact raw evidence paths, exact next action, compile/test state,
counters and all incomplete files. Preserve base binaries and use only .rum
guest sources. Record which rollback-timing choice the director selected.

## Hard rules

- No git in any form; leave work uncommitted.
- Objective/fence only, preserve unrelated changes; no dependency/lockfile edits.
- Nothing outside repository; no network; no new unsafe; no leases.
- Commands: file reads/edits, rg/grep/python3, bounded cargo, named microbench
  scripts, and bounded taskset/perf/release CLI measurement commands.
- Rum-vs-rum testing only. No default-path instrumentation allocation/mutex.
- Do not weaken host rejection, partial-restore, reset or cancellation contracts
  to turn a red test green. Report a contract conflict.
- All named measured gates and progress stop-loss apply.

## Report format

1. Boundary transition/call-site/root inventory and observable lifecycle changes.
2. Exact red-before/green-after outputs, ownership assertions, fresh requests
   and unchanged blocked-batch/cancellation behavior.
3. Allocation counts, per-iteration instructions and complete per-label timing
   comparison, with neighbor failures/unavailable lanes disclosed.
4. Scope/organization assessment, progress counters and unresolved questions.


---


## 8. Open questions for the director

1. **Public file batch rollback timing, not resumability.** The recommended
   explicit abandonment-only owner preserves delayed scoped cleanup but
   defers the public file snapshot rollback until cancellation/retirement.
   Today's public Session::eval_file eagerly restores it on InputRequired,
   while the host rustdoc specifies the same multi-form behavior as script
   and does not specify this internal rollback timing. Is eager rollback a
   required additional direct-Session contract? If yes, choose immediate
   inner-to-outer abandonment plus a lightweight blocked marker, and
   explicitly accept the resulting earlier WithCleanup execution. Settle
   this in the boundary draft; neither option allows batch resumption or a
   fresh request while blocked.
2. **Existing budget renewal at each form.** The nested top-level helper
   renews the timeout and resets the active memory limit for each new form.
   Resumption of a paused form must not renew either budget. Should the
   existing *new-form* behavior be repaired in the same vertical slice or
   in a separate resource-budget brief? I recommend a separate explicit
   contract/test decision, not a hidden change to satisfy a continuation
   test. HostSession's no-new-deadline-on-resume contract remains mandatory.
3. **BeginPackage's early propagated error.** Ordinary dependency failure
   ($Failed) rolls back, but its current bare ? propagates Exit without
   running the BeginPackage rollback helper. The active file snapshot still
   restores. Should propagated Exit also restore the package frame?
   Preserve the existing queued behavior unless the director explicitly
   includes and documents this correction; cancellation/abandonment must
   restore the package frame regardless.
4. **Frontend terminal-input disposition.** REPL rendering and synchronous
   Jupyter execute leave their borrowed/owned session blocked after reporting
   input; CLI usually drops its session. Should those reusable frontends
   gain explicit input handling or abandon on their terminal error path in
   a separate brief? This audit proves their present ownership disposition
   from source, but the authorized scratch crate cannot execute their actual
   wrappers. Commission adapter-specific red tests before changing policy.

No question reopens resumable guest loading, unchanged host batch
non-resumability, no leases, rejection of overlaps, or the rum-vs-rum oracle.

Completed milestones and evidence are in
[PROGRESS.md](../../../.temp/brief-549/PROGRESS.md). There are no remaining
scratch tests in crates/rum-stdlib and no production edits to gate.
