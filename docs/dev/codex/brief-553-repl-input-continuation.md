# brief-553 — the REPL answers input requests and resumes

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 3A of the suspension-continuation branch (brief-552, draft A, rewritten).
Today, when a line typed at the `rum-repl` prompt asks for input
(`InputString`, `Input`), the REPL prints `! input required: …`, returns to the
prompt, and leaves the evaluation suspended in its `HostSession`, so every line
the user types afterwards is refused with "cannot start a new evaluation while
the session is waiting for host input". `:trace` behaves the same. The crate's
own rustdoc (`crates/rum-repl/src/lib.rs:5`) already claims "resumable input".

When you are done, a REPL line or `:trace` command owns its request until the
request completes or is explicitly abandoned: the REPL reads the answer from
its input and resumes the same request — through nested `Get`s and any number
of input requests — and a `HostSession` handed back to the caller never holds a
suspension the REPL started.

## User rulings, 2026-09-30 — implement exactly these

- **U1 — interactive answers.** The next line read while a request is waiting
  is the answer. It resumes the pending request; it is never replayed as guest
  source, never treated as a shell command (a literal `:quit` is an answer to
  `InputString`), and never recorded in history. `:trace` keeps its renderer
  and every option (`--rules`, `--form`, `--depth`, `--full`) across the wait.
- **U2 — prompt and answer text.** Show a nonempty `HostInputRequest.prompt`
  verbatim and flush before reading; show `input> ` for an empty prompt; do not
  show `rum> ` or `...> ` during the wait. Show any output produced before the
  request first. Strip only the line terminator (CR/LF); keep other whitespace
  and an empty answer. `InputString` receives the literal text; `Input`'s
  parsing belongs to `rum-host` — do not add a second parser or a reprompt rule.
- **U3 — end of input and interruption.** EOF while waiting cancels the pending
  request (cancel its handle, re-enter once through the documented host route)
  and then leaves the REPL cleanly. An interrupted read cancels the same way and
  returns to the primary prompt. A read or write error retires the request the
  same way before the error is returned. Show one concise cancellation notice
  and any guest cleanup output once; definitions made before the request stay;
  the abandoned remainder never runs. A successful cancellation can arrive as
  `HostEvalOutcome::Success` with `HostTerminationReason::Cancelled` — match the
  typed termination, do not assume a `Failure`.
- **U4 — password requests.** A request with `password == true` is answered
  without echo, without placeholder characters and without history — **and a
  reader that cannot guarantee that must report "password input unavailable"
  and abandon the request without reading an answer. Never fall back to visible
  entry.** The terminal reader that can suppress echo is the next brief's work;
  in this brief every reader is a plain `BufRead`, so every password request
  takes the report-and-abandon path, through a reader capability the next brief
  can switch on.
- **U7 — abandonment reporting.** Automatic abandonment reports the original
  input-required reason and does not add a second synthetic "cancelled" error;
  only a user interrupt or EOF reports cancellation. The REPL cancels only a
  request it admitted — a rejected overlapping request must not cancel someone
  else's.

## Red tests first

`.temp/brief-552/tests/zz_brief552_repl.rs` holds brief-552's executable REPL
reds; the director re-ran them on `26f5d434` (2 passed, 9 failed, identical to
the worker). Install as **permanent tests in `crates/rum-repl/tests/`** — the
directory does not exist yet; create it — using only `rum-host` and `rum-repl`:

- `red_line_resume_matches_queued`, `red_two_inputs_resume_matches_queued`,
  `red_nested_get_resume_matches_queued`, `red_plain_trace_resume_matches_queued`,
  `red_trace_resume_matches_queued` — the resumed result, rendering and state
  equal the queued-input path, and a following `1+1` is `2`;
- `red_eof_abandons_and_leaves_borrowed_session_usable`,
  `red_interrupt_abandons_and_leaves_borrowed_session_usable`;
- `red_answer_is_literal_and_not_saved_in_history`;
- `red_password_without_secure_reader_abandons`.

Add, red first where they fail today: guest cleanup (`WithCleanup`) runs exactly
once and the abandoned remainder does not run, on EOF and on interrupt; a read
error and a write error each retire the request; an empty answer and a
whitespace-only answer reach the guest unchanged; output printed before the
request appears exactly once; `:trace` with `--form` and `--depth` resumes; an
empty prompt shows `input> `. **The nested-`Get` fixture must not depend on
`.temp/`**: use a fixture under `crates/rum-repl/tests/` or an in-memory
`FileIo` provider through `HostConfig`. Do not promote brief-552's `current_*`
tests: they assert today's defects.

Run the permanent tests before changing production code and save the failing
output under `.temp/brief-553/red/`.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-552's report (appended to its brief) §2 behaviour table, §3 dispositions
  and draft A; `.temp/brief-552/tests/zz_brief552_repl.rs`.
- `crates/rum-repl/src/lib.rs` (the read loop, `evaluate_input` at `:141-151`,
  history at `:153-229`), `crates/rum-repl/src/commands.rs` (`:trace`,
  `:343-398`), `crates/rum-host/src/session.rs` (`eval`, `resume_with_input`,
  `cancel_handle`, the input request type), `docs/guide/embedding.md` (the
  input and cancellation contract).

## Scope

- **Objective:** the REPL's request-driving lifecycle — start once, answer and
  resume until completion, abandon on EOF, interrupt and I/O failure — as a
  cohesive module rather than growth in `commands.rs`; the reader capability
  for masked input; the permanent tests; the crate rustdoc and any guide text
  that describes the REPL at a prompt.
- **Fence:** `rum-repl` only. No change to `rum-host`, the kernel, `rum-stdlib`,
  `rum-cli` (the next brief wires the terminal), `rum-jupyter` or
  `rum-terminal`; no new dependency; no direct `rum-runtime` or `rum-stdlib`
  use in production or in the new tests; no language, batch or snapshot
  contract change. Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier
  briefs or unrelated docs.
- **Disclosure duty:** list every changed file with line ranges and why; every
  public API change in `rum-repl`; every change a user sees at the prompt or in
  history. If the objective needs a change outside the fence, STOP and report.

## rum / Rust pitfalls (read twice)

1. **A probe that did not suspend proves nothing.** Assert the input request
   happened before comparing results.
2. **Take the cancel handle for the request you admitted**, before driving it;
   a rejected overlapping request owns nothing.
3. **`reset` is not cancellation** and runs no guest cleanup; do not use it to
   recover.
4. **Do not replay output.** Output delivered before a prompt must not be
   printed again when the request completes.
5. **A read-ahead buffer belongs to one owner.** Do not read the answer through
   a second reader that can lose bytes the line reader already buffered.
6. **Stop-loss is keyed on progress**: permanent reds turned green, lifecycle
   assertions added, existing tests still green. Reducing compiler errors is
   progress; stop when two consecutive compiling attempts move nothing.

## Verification (MANDATORY — you run it; iterate to green)

Wrap every command in `python3 scripts/run-bounded.py --timeout <seconds> --`,
bounds derived from a measured run of the same shape (brief-552's REPL probe ran
in about 4 s), and run cargo as `env CARGO_BUILD_JOBS=4 cargo …`.

```
cargo fmt --check
cargo test -p rum-repl
cargo test -p rum-cli
cargo test --workspace
```

Paste commands, bounds, exit statuses and counts verbatim. Native ZeroMQ tests
that cannot bind loopback in the sandbox need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`;
if you set it, say so and list them. No microbench: nothing here is on the
evaluator's path — if you find it is, STOP and report.

## Checkpointing — protect the scene

Scratch under `.temp/brief-553/`. After each milestone — reds saved, driver
compiling, each red green, the extra lifecycle tests, docs, the gate — write
`.temp/brief-553/PROGRESS.md`: what is done and verified with its evidence, the
exact next action, the tree's state, and the counters.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git` at
  all. Leave all changes in the working tree.
- Do NOT modify files outside the objective and fence; STOP and report instead.
- Touch nothing outside this repository, user configuration included. No
  network. Commands: file reads and edits within scope, `rg`/`grep`, `python3`,
  bounded `cargo`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No new `unsafe`.**
- Do not weaken a red assertion or a host contract to make a test pass; report
  the conflict.
- You MUST paste real verbatim output, and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges and why; the prompt-visible and
   history-visible changes, before and after.
2. Verification — every command's verbatim result; red before and green after
   for every permanent test.
3. The progress counter sequence, disclosures, and anything unresolved.
