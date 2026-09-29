# brief-552 — design: what each front end does when guest code asks for input

`Scope: NONE — no production change.` Design-only, with one declared scratch
exception for probes (see Scope). Establish what every in-tree front end does
today when evaluation suspends for input, propose a disposition for each, and
draft the implementation briefs. **Do not run `git` in any form**, including
while first loading this repository — not even `git status`.

## Goal

Step 3 of the suspension-continuation branch. The kernel now suspends and
resumes correctly through guest loading (brief-550, `c6e95dd1`) and retires
suspensions at the batch boundary (brief-551, `34e57a8d`), with a
request-boundary check that no suspension owner survives a completed request.
What remains at this layer is the **front ends** — the in-tree hosts that sit
on `rum-host`: `rum-repl`, `rum-cli` and `rum-jupyter`. brief-549's audit of
the 38 `Transformed` pairs (report §3, rows P2367-P2399) found, from source
only, that several of them report `InputRequired` and then leave their session
blocked: every later evaluation is refused until something resets or cancels
it. Those findings were never executed, because brief-549's scratch crate had
no front-end dependencies.

Settle, per front end and per entry point, what happens now, what should
happen, and how to get there — and hand the director the policy decisions
that are the user's.

## What is known, from source

- `rum-repl`: `Repl::evaluate_input` (`crates/rum-repl/src/lib.rs:141-151`)
  renders `HostEvalOutcome::InputRequired` and returns to the read loop. The
  `HostSession` it borrows keeps the pending suspension, so the next line the
  user types is rejected. `:trace` has the same shape
  (`crates/rum-repl/src/commands.rs:343-373`).
- `rum-jupyter`: the asynchronous worker path handles input — when the
  execute request allows stdin it emits `input_request` and resumes on the
  reply (`crates/rum-jupyter/src/adapter/mod.rs:280-291,444-501`). The
  synchronous `KernelAdapter::execute` (`:137-140`, `run_execute_request`
  around `:632-637`) converts the outcome into a completed response and keeps
  no pending execute context.
- `rum-cli`: non-interactive modes bind stdin to `$ScriptInput` and command
  arguments to `$ScriptArgs` by queuing exactly the responses the bindings
  consume (`crates/rum-cli/src/main.rs:281-320`); a script that then calls
  `InputString` gets `input required` and exits 1. The director confirmed it
  end to end on `34e57a8d` with a `--file` script whose nested `Get` asks for
  input: `! input required: p:`, exit 1, no panic — with and without piped
  stdin, because stdin went to `$ScriptInput`. The interactive REPL mode goes
  through `rum-repl`.

These are readings and one probe, not premises. Confirm or correct each.

## What I want back

### 1. The current behaviour, executed

For every front-end entry point that can evaluate guest code — REPL line
evaluation and `:trace` (and any other command that evaluates), CLI `--eval`,
`--file`, stdin-program and REPL-or-filter modes, Jupyter synchronous
`execute` and the asynchronous worker with `allow_stdin` true and false, and
any other adapter surface you find — **run** a guest source that asks for
input, and record: what the user sees, the exit or reply status, whether the
session accepts a following evaluation, and whether a pending suspension
remains. Do it through each crate's own public API or test harness, in a
scratch integration test (Scope). Include one case where the input is asked
for inside a nested guest `Get`, and one where it is asked for twice.

### 2. A disposition for each entry point

Propose, with reasons, what each should do. The shapes available are at least:

- **resume interactively** — the front end obtains a line from its user and
  resumes the same request (the natural behaviour for an interactive REPL, and
  what the Jupyter worker already does);
- **abandon and report** — the front end reports that input was required,
  cancels and re-enters (or otherwise abandons through the documented route)
  so the session is usable again, without running the rest of the request;
- **leave blocked, as a documented contract** — only where the entry point's
  contract says so and something visible tells the user how to recover.

For the REPL, say what happens on EOF and on interrupt while it is waiting for
the answer, what prompt it shows, whether `Input` (which parses its answer) and
`InputString` differ, and how password requests (`RuntimeInputRequest.password`)
are handled on a terminal. For Jupyter, whether the synchronous path should
gain the worker's behaviour or abandon, and why the two paths exist. For the
CLI, whether an interactive terminal on stdin changes anything, and whether
any helper leaves a borrowed session blocked for a caller that reuses it.

**Mark which choices are the user's to make** — anything that changes what a
person sees at a prompt or in a notebook — and state your recommendation for
each separately from the facts.

### 3. The tests that pin it

For each entry point, the red tests that fail today and pass after its
disposition, written against the crate's own harness so they run in this
host's workspace gate. The oracle where it applies is rum against rum: the
resumed-interactively path must equal the queued-input path.

### 4. Drafted implementation briefs

In `TEMPLATE.md` shape, one per coherent change, each starting from its red
tests. Scope states **an objective, a fence and a disclosure duty**, not a
list of permitted files. Scratch and checkpoints under `.temp/brief-NNN/`.
Stop-loss keyed on progress (`TEMPLATE.md` pitfall 7). These crates are
`rum-host` consumers only — `rum-cli` and `rum-repl` have no `rum-runtime` or
`rum-stdlib` dependency by user ruling (dogfood stance A); keep it that way.

### 5. Open questions for the director

## Settled — do not reopen

- The kernel and host contracts from briefs 550 and 551 stand: guest loading
  resumes; batch entry points stay non-resumable; cancellation then re-entry is
  the documented escape; `reset` runs no guest cleanup.
- Front ends consume `rum-host` only; no front end gains a kernel dependency.
- Overlapping requests are rejected, not superseded (brief-411).
- The oracle is rum against rum.

## Context pointers

- `AGENTS.md` first.
- brief-549's report §3 (the owner-leak audit, front-end rows) and §8
  question 4; brief-551's report §1 (the batch contract as it now stands).
- `docs/guide/embedding.md` — the host input, batch and cancellation contract.
- `crates/rum-repl/src/lib.rs`, `crates/rum-repl/src/commands.rs`,
  `crates/rum-cli/src/main.rs`, `crates/rum-jupyter/src/adapter/`, and each
  crate's existing tests as the model for its harness.
- `crates/rum-host/src/session.rs` — `eval`, `resume_with_input`,
  `cancel_handle`, `queue_input_response`, the batch entry points.

## Scope

- Files you MAY modify: **NONE**, with these exceptions:
  - append your report to **this** file;
  - write scratch under `.temp/brief-552/`;
  - create scratch integration tests named `zz_brief552_*.rs` in the `tests/`
    directory of `rum-repl`, `rum-cli` or `rum-jupyter` as needed, and **delete
    every one of them before appending the report**, copying their final
    content to `.temp/brief-552/`. Say in the report that you did.
- Everything else is read-only. No manifest change.

## rum / Rust pitfalls (read twice)

1. **A probe that did not suspend proves nothing.** Assert the input request
   happened before recording what follows.
2. **"Blocked" is measured by the next evaluation**, not by the prompt field.
3. **A front end's own stdin binding consumes queued input.** The CLI queues
   exactly what `$ScriptInput` and `$ScriptArgs` consume; do not mistake that
   for a resume path.
4. **Native ZeroMQ tests may not bind loopback in your sandbox.** Use the
   socket-free harnesses `rum-jupyter` already has for the adapter paths.

## Verification (MANDATORY — the evidence IS the deliverable)

- State every command; persist raw output under `.temp/brief-552/` before
  parsing it. Run probes with
  `python3 scripts/run-bounded.py --timeout <seconds> -- env CARGO_BUILD_JOBS=4 cargo test -p <crate> --test zz_brief552_<name> -- --nocapture`,
  bounds derived from a measured run.
- Paste the per-entry-point behaviour table and the proposed dispositions.
- State that every scratch test was deleted.

## Checkpointing — protect the scene

After each milestone write `.temp/brief-552/PROGRESS.md`: what is done and
verified, its evidence, the exact next action, and which scratch tests
currently exist.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions in Scope. If you believe you
  must, STOP and write that in the report.
- Touch nothing outside this repository, including user configuration. No
  network. Commands: file reads, `rg`/`grep`, `python3`, edits to the scratch
  tests, and the bounded cargo command above.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch artifacts, deletion of the scratch
   tests.
2. Verification — commands and the behaviour table.
3. Proposed dispositions, with the user's decisions marked and your
   recommendation for each.
4. The red tests.
5. The drafted implementation briefs.
6. Open questions for the director.

---
