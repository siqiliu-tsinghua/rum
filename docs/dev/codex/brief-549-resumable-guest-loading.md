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
