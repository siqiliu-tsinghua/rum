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

## Implementation report — 2026-09-30

The REPL continuation implementation and focused gates pass. The mandatory
workspace gate is **not green**: it reports 17 retention-certificate failures in
`rum-stdlib`. Its fingerprint function hashes every Rust source under `crates/`,
so the authorized REPL edits invalidate stored evidence. Refreshing that
evidence requires changes outside this brief's fence. Work stopped at that
boundary; no certificates or assertions were changed.

### 1. What changed

Every changed or added review file is listed below. Line ranges refer to the
final files; the removed legacy suspension/failure helpers were formerly at
`lib.rs:303–319`.

| File | Lines | Reason |
| --- | --- | --- |
| [crates/rum-repl/src/lib.rs](../../../crates/rum-repl/src/lib.rs) | 8–26, 33–41, 122–173, 227–267; removal before 340 | Document input ownership and plain-reader limits; route ordinary evaluation and trace through the driver; propagate EOF without reopening a primary prompt; remove the old suspension-only renderer. |
| [crates/rum-repl/src/commands.rs](../../../crates/rum-repl/src/commands.rs) | 1–6, 47–55, 144–158, 341–352 | Prepare trace source and retained rendering options without starting an evaluation; render the completed typed trace through the existing tree renderer. |
| [crates/rum-repl/src/request.rs](../../../crates/rum-repl/src/request.rs) | 1–277, new | Cohesive admitted-request driver, plain-reader capability, input events, cumulative-output cursors and cancellation/retirement path. |
| [crates/rum-repl/tests/input_continuation.rs](../../../crates/rum-repl/tests/input_continuation.rs) | 1–291, new | Six promoted reds and six additional continuation cases: host queued/resumed equality, exact REPL rendering, all trace options, history, whitespace, empty prompts, parser failure, prompt flushing and output once. |
| [crates/rum-repl/tests/input_abandonment.rs](../../../crates/rum-repl/tests/input_abandonment.rs) | 1–308, new | Three promoted reds, four additional lifecycle cases and three supplementary controls: cleanup once, no remainder, ownership, I/O errors, masked input and reuse. |
| [crates/rum-repl/tests/support/mod.rs](../../../crates/rum-repl/tests/support/mod.rs) | 1–128, new | Host-only setup, actual-suspension preflights, scripted reader errors and repository-local disposable history files. |
| [crates/rum-repl/tests/fixtures/input-outer.rum](../../../crates/rum-repl/tests/fixtures/input-outer.rum) | 1, new | Permanent outer nested-Get fixture. |
| [crates/rum-repl/tests/fixtures/input-inner.rum](../../../crates/rum-repl/tests/fixtures/input-inner.rum) | 1–4, new | Permanent suspended inner load with before/after effects; found through HostConfig search paths, with no .temp dependency. |
| [docs/guide/getting-started.md](../../guide/getting-started.md) | 55–87 | User-facing REPL input, history, cancellation, trace and password behavior. |
| This brief | 179–519, appended | Append this report only; the original brief is byte-preserved. |

**Public API:** none changed. `Repl<R, W>`, its constructors/builders and
`run(&mut HostSession) -> io::Result<()>` retain their signatures and bounds.
The reader capability is internal: `RequestInput::password_available` and
`read_answer`, implemented by a `PlainInput` borrow of the same buffered reader.
All current readers advertise no password capability. No dependency or manifest
was changed in the production workspace.

**Visible behavior:** a waiting ordinary line or `:trace` formerly printed an
input-required diagnostic, returned to `rum> ` and stranded the session. It now
prints unseen output, displays the exact guest prompt (or `input> `), flushes,
and reads answers until completion. Answers preserve empty text and whitespace
apart from one CR/LF line ending; `Input` parsing remains in the host. Literal
`:quit` answers `InputString`. Trace flags and the filter-rewrite notice survive
the wait. Buffered prints now appear for ordinary/trace completion and failure
as well as before input; messages and prints already delivered do not replay.

History still records submitted source and shell commands. Answer lines bypass
history, shell command dispatch and completeness checks, including answers that
look like commands. EOF cancels and exits; an interrupted answer read cancels
and returns to the primary prompt. Cancellation re-enters the host once, runs
cleanup once, preserves prior definitions and does not run the abandoned tail.
I/O errors retire before returning the original error; output is best effort
when the writer itself fails. Explicit EOF/interrupt gets one cancellation
notice. Automatic abandonment retains the input-required reason and omits the
synthetic cancellation diagnostic. Earlier guest messages, including guest
`General::cancel` messages, remain visible.

Masked input formerly left the session blocked. It now reports
`password input unavailable` and retires without reading an answer or falling
back to visible entry. A following line remains ordinary REPL source. Rejected
overlaps retain the caller's pending request and never acquire its cancel handle.

**Organization assessment:** the new request module is 10,105 characters and
keeps one lifecycle and its reader seam together; its slight excess over the
preferred 10k size does not justify an arbitrary cut. `lib.rs` is 10,219
characters including expanded rustdoc. `commands.rs` shrank to 22,467 characters
including its existing inline tests. This patch removes evaluation ownership
from it; a future separation of command parsing/dispatch from trace rendering
is the remaining natural boundary. No unrelated split or forbidden TODO edit
was made.

### 2. Verification

All nine promoted reds and the ten initial additional/control cases were
installed and run before changing production. Their raw failures are preserved
under [.temp/brief-553/red/](../../../.temp/brief-553/red/). No `current_*` defect
tests were promoted. Each input fixture first asserts an actual host suspension;
parity fixtures assert every input request and complete host outcome equality
against prequeued rum input, then exact REPL rendering and state/reuse.

The first cargo invocation stopped after the failing abandonment target, so the
continuation target was run separately before production edits. The first driver
run turned every continuation test green, while abandonment tests exposed the
extra `General::cancel` diagnostic. The second run passed all 19 initial cases
without weakening an assertion.

Three supplementary controls were added afterward and first executed green in
the REPL gate. An isolated original-source crate was prepared under
`.temp/brief-553/baseline/crates/rum-repl/` for additional before/after evidence,
but was **not run** after discovering the scope blocker. Those three cases have
no original-code red result; the table states that explicitly.

Permanent cases (statuses copied from test output):

| Test | Original code | Final REPL gate |
| --- | --- | --- |
| `red_line_resume_matches_queued` | FAILED | ok |
| `red_two_inputs_resume_matches_queued` | FAILED | ok |
| `red_nested_get_resume_matches_queued` | FAILED | ok |
| `red_plain_trace_resume_matches_queued` | FAILED | ok |
| `red_trace_resume_matches_queued` | FAILED | ok |
| `trace_form_and_depth_resume_matches_queued` | FAILED | ok |
| `trace_full_and_default_limit_survive_input` | FAILED | ok |
| `red_answer_is_literal_and_not_saved_in_history` | FAILED | ok |
| `empty_and_whitespace_answers_are_preserved` | FAILED | ok |
| `empty_prompt_uses_input_prompt` | FAILED | ok |
| `malformed_input_is_left_to_host_without_reprompt` | FAILED | ok |
| `prefix_output_is_flushed_before_each_prompt_and_never_replayed` | FAILED | ok |
| `red_eof_abandons_and_leaves_borrowed_session_usable` | FAILED | ok |
| `red_interrupt_abandons_and_leaves_borrowed_session_usable` | FAILED | ok |
| `read_error_retires_request_before_returning_original_error` | FAILED | ok |
| `write_error_retires_request_including_prompt_flush_and_prefix` | FAILED | ok |
| `red_password_without_secure_reader_abandons` | FAILED | ok |
| `trace_abandonment_runs_cleanup_once_and_keeps_session_usable` | FAILED | ok |
| `rejected_overlaps_preserve_the_callers_pending_request` | ok | ok |
| `eof_at_a_later_prompt_cancels_the_original_request` | not run; supplementary | ok |
| `cancellation_keeps_prefix_output_and_guest_cleanup_messages_once` | not run; supplementary | ok |
| `permanently_broken_output_cannot_prevent_guest_cleanup` | not run; supplementary | ok |


The following are the exact bounded cargo commands, result lines and runner
records. Complete unedited stdout/stderr is linked for each invocation; the
workspace log also contains expected compile-fail controls, so its incidental
compiler errors are not all gate failures.

Bounds: the initial 30 s REPL bound used brief-552's 3.844 s measurement with
headroom. Its first build/run measured 12.036 s, giving the next 55 s bound;
subsequent runs used 50 s after a 10.634 s measurement. CLI used 40 s against
the earlier 6.967 s crate probe and measured 5.695 s here. Workspace used 600 s
against the earlier 351.789 s gate and measured 268.873 s here. Package
formatting calibrated at 0.254 s; workspace formatting used 30 s and measured
2.423 s. The logging wrapper itself also ran under run-bounded, with 10 s more
than each recorded child bound.

`CARGO_BUILD_JOBS=4` was set for every cargo invocation, and
`CARGO_NET_OFFLINE=true` prevented dependency network access. `TMPDIR` kept
test temporary files inside this repository.

[Complete output](../../../.temp/brief-553/red/permanent-01.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation --test input_abandonment -- --nocapture
test result: FAILED. 1 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation --test input_abandonment -- --nocapture", "bound_seconds": 30, "exit_status": 101, "elapsed_seconds": 12.036, "log": ".temp/brief-553/red/permanent-01.log"}
```

[Complete output](../../../.temp/brief-553/red/continuation-01.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation -- --nocapture
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation -- --nocapture", "bound_seconds": 30, "exit_status": 101, "elapsed_seconds": 0.146, "log": ".temp/brief-553/red/continuation-01.log"}
```

[Complete output](../../../.temp/brief-553/green/permanent-01.log)

```text
$ python3 scripts/run-bounded.py --timeout 55 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation --test input_abandonment --no-fail-fast -- --nocapture
test result: FAILED. 1 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 55 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation --test input_abandonment --no-fail-fast -- --nocapture", "bound_seconds": 55, "exit_status": 101, "elapsed_seconds": 10.634, "log": ".temp/brief-553/green/permanent-01.log"}
```

[Complete output](../../../.temp/brief-553/green/permanent-02.log)

```text
$ python3 scripts/run-bounded.py --timeout 50 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation --test input_abandonment --no-fail-fast -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 50 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl --test input_continuation --test input_abandonment --no-fail-fast -- --nocapture", "bound_seconds": 50, "exit_status": 0, "elapsed_seconds": 7.859, "log": ".temp/brief-553/green/permanent-02.log"}
```

[Complete output](../../../.temp/brief-553/green/format-package.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo fmt -p rum-repl
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo fmt -p rum-repl", "bound_seconds": 30, "exit_status": 0, "elapsed_seconds": 0.254, "log": ".temp/brief-553/green/format-package.log"}
```

Cargo emitted no stdout/stderr for this successful formatting command.

[Complete output](../../../.temp/brief-553/gate/fmt.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo fmt --check
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo fmt --check", "bound_seconds": 30, "exit_status": 0, "elapsed_seconds": 2.423, "log": ".temp/brief-553/gate/fmt.log"}
```

Cargo emitted no stdout/stderr for this successful formatting command.

[Complete output](../../../.temp/brief-553/gate/repl.log)

```text
$ python3 scripts/run-bounded.py --timeout 50 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 50 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-repl", "bound_seconds": 50, "exit_status": 0, "elapsed_seconds": 8.14, "log": ".temp/brief-553/gate/repl.log"}
```

[Complete output](../../../.temp/brief-553/gate/cli.log)

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-cli
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp cargo test -p rum-cli", "bound_seconds": 40, "exit_status": 0, "elapsed_seconds": 5.695, "log": ".temp/brief-553/gate/cli.log"}
```

[Complete output](../../../.temp/brief-553/gate/workspace.log)

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.75s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.22s
test result: FAILED. 1008 passed; 17 failed; 53 ignored; 0 measured; 0 filtered out; finished in 225.56s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-553/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture", "bound_seconds": 600, "exit_status": 101, "elapsed_seconds": 268.873, "log": ".temp/brief-553/gate/workspace.log"}
```

Workspace failure list and terminal result, verbatim:

```text
failures:
    tests::retention_closure::census::tests::production_rejection_census_is_complete
    tests::retention_closure::local::admission_tests::local_pending_argument_and_wrapped_payload_controls
    tests::retention_closure::local::contract_tests::local_concrete_contract_controls
    tests::retention_closure::local::option_boundaries::tests::local_option_contract_controls
    tests::retention_closure::local::oracle::local_probe_claims_match_stable_rustc
    tests::retention_closure::local::production::local_production_acceptance
    tests::retention_closure::local::promotion_tests::production_body_domain_and_build_expiry_and_raw_local
    tests::retention_closure::local::promotion_tests::production_inventory_rejects_removed_redirected_stale_and_omitted_evidence
    tests::retention_closure::local::promotion_tests::production_marks_require_current_local_lint_and_direct_contracts
    tests::retention_closure::local::promotion_tests::production_promotion_cannot_launder_consumers_or_runtime_domain
    tests::retention_closure::local::reads::oracle::local_option_read_claims_match_stable_rustc
    tests::retention_closure::local::reads::tests::local_option_borrowed_pattern_controls
    tests::retention_closure::local::reads::tests::local_option_read_contract_controls
    tests::retention_closure::local::reads::tests::local_option_short_circuit_ownership_controls
    tests::retention_closure::local::rules::tests::local_rule_contract_controls
    tests::retention_closure::local::tests::local_contract_falsifiers
    tests::retention_closure::production_obligations_are_exact

test result: FAILED. 1008 passed; 17 failed; 53 ignored; 0 measured; 0 filtered out; finished in 225.56s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```


Reached workspace targets total **2,486 passed, 17 failed, 58 ignored**. The
passing count includes nine native ZeroMQ cases that returned early under the
explicit skip policy; it does not claim successful socket execution.
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` was set only for the workspace command. The log
contains nine instances of this notice (some interleaved by parallel test output):

```text
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

- `native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket`
- `native_zeromq_backend_frontend_style_control_interrupt_round_trip`
- `native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata`
- `native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown`
- `native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply`
- `native_zeromq_backend_rejects_tampered_signed_requests_end_to_end`
- `native_zeromq_backend_round_trips_execute_over_loopback_sockets`
- `native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages`
- `native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`


Did not run: the workspace's later stdlib integration tests, syntax/terminal
targets and workspace doctests after cargo stopped at the failed stdlib unit
target; the isolated baseline supplement; physical terminal masking or OS
Ctrl-C tests (next brief); any microbenchmark (no evaluator hot-path change);
certificate regeneration, promotion replay or an out-of-fence repair.
The focused REPL gate did run its zero-doctest target.

### 3. Progress counters, disclosures and unresolved work

Counter order is compiler errors / named reds green / initial additional cases
green / existing REPL+CLI regressions. The sequence is:

- Original-code runs: `0 / 0 of 9 / 1 of 10 / not measured`.
- Driver compiling: `0 / 6 of 9 / 7 of 10 / not measured`; cleanup/reuse worked,
  with remaining failures at the duplicate cancellation diagnostic.
- Cancellation reporting fixed: `0 / 9 of 9 / 10 of 10 / not measured`.
- REPL gate: `0 / 9 of 9 / 10 of 10 / 0 of 45`, plus 3/3 supplementary cases.
- CLI gate: `0 / 9 of 9 / 10 of 10 / 0 of 73`.
- Workspace gate: same focused counters; **17 unresolved certificate failures**.

No two consecutive compiling attempts stalled. Every required lifecycle
assertion is installed; no red assertion or host contract was weakened.
Checkpoints, exact next actions and tree state are recorded in
[PROGRESS.md](../../../.temp/brief-553/PROGRESS.md).

**Scope blocker:** the fingerprint implementation at
[contracts.rs:88–134](../../../crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs)
recursively hashes all `.rs`, `.py` and `.toml` files under `crates/`
(collection at 99–125). Thus even REPL-only source or test changes expire the
stored contract fingerprints. The stale-contract failures propagate to the
promotion and census controls. Repair would require maintenance of
`rum-stdlib`'s `inventory/local-contracts.tsv` and dependent promotion evidence,
followed by the required controls and workspace gate. Those files are explicitly
outside this brief's fence; they remain untouched. This is not a completed
workspace gate and not a claim that the certificate failures can be ignored.

No git command was run, no network or elevated execution was used, and no
unsafe, direct kernel/stdlib use or production dependency was added. No host,
kernel, stdlib, CLI, terminal, Jupyter, AGENTS, TODO, unrelated documentation or
earlier brief was edited. There is no language, batch or snapshot contract
change. All implementation changes remain in the working tree for the director.

Scratch is confined to `.temp/brief-553/`: original-file copies and hashes,
the bounded logger and its command ledger, complete red/green/gate output,
the review patch, fingerprint diagnosis, the unexecuted isolated baseline
copy, this report generator/copy, scope audits and checkpoints. Test-generated
build artifacts are not review files. The final scope audit compares all
1,699 initial source/document/script files without using git and verifies that
only the listed review files changed and this brief's prefix is unchanged.

The next required action is a separately scoped certificate refresh and full
workspace re-verification by the director; this brief's hard stop is honored.
