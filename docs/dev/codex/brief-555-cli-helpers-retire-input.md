# brief-555 — the CLI's evaluation helpers retire the input request they started

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 3C of the suspension-continuation branch (brief-552, draft C, rewritten).
`rum-cli`'s non-interactive modes (`--eval`, `--file`, a positional file, `-`
for a stdin program, and the non-terminal filter) report an input request and
exit 1, and that stays so (user ruling U5: a script is never interactive, even
with a terminal on stdin). But the helpers that do it —
`eval_source`, `eval_script_source`, `eval_file`, their shared `eval_request`
(`crates/rum-cli/src/main.rs:221-280`), and `eval_setup_binding` (`:308-320`)
— return the error while their borrowed `HostSession` still holds the
suspension they started. The process usually exits right after, so nobody
notices; a caller that reused the session would find every later request
refused. brief-552 executed all four as red on `26f5d434`.

When you are done, every one of these helpers, on every path that returns —
including the early return when stdout is a broken pipe — leaves its borrowed
session with no suspension it started, and the command-line behaviour a user
sees is unchanged.

## User rulings, 2026-09-30 — implement exactly these

- **U5** — no CLI script mode becomes interactive; a terminal on stdin changes
  only whether stdin is bound to `$ScriptInput`, as today; piped stdin and `--`
  arguments are data consumed by the bindings, not answers to later requests.
  Exit status 1 on a required input, as today.
- **U7** — retire only a request this helper admitted: take the request's
  cancel handle before starting it, cancel it on `InputRequired`, and re-enter
  once through the documented host route (`resume_with_input` or a
  guest-evaluation entry point — `docs/guide/embedding.md:364-380`). Report the
  **original** input-required reason as the error; do not add a second
  synthetic "cancelled" error. A request refused at admission owns nothing and
  must not cancel someone else's pending request. Never resume a batch with an
  answer, real or invented, and never use `reset` — it would erase definitions
  and skip the cleanup that cancellation runs.

## What retirement does, which the tests must show

Cancelling a batch runs its abandonment: `WithCleanup` cleanup runs once, a
suspended public file rolls back inner to outer (brief-551), a script keeps its
earlier forms (scripts are not transactional). Output that cleanup produces
must reach the user once — decide where it goes (stdout, like other guest
prints, is the natural answer) and disclose it. The re-entry after
cancellation can come back as `HostEvalOutcome::Success` carrying
`HostTerminationReason::Cancelled`; match the typed termination.

## Red tests first

`.temp/brief-552/tests/zz_brief552_cli.rs` holds the four reds
(`red_eval_helper_abandons`, `red_script_helper_abandons`,
`red_file_helper_abandons`, `red_unexpected_setup_input_abandons`), but it
reaches the private helpers by `include!`-ing a copy of `main.rs` from
`.temp/`. **The permanent tests must not do that**: put them in a
`#[cfg(test)]` module inside `rum-cli` beside the helpers, calling them
directly with a `HostSession`. Each asserts that the helper returns the
input-required error and that `1+1` then evaluates to `2` on the same session.

Add, red first where they fail today: the broken-pipe early return retires its
request; cleanup output appears exactly once and the abandoned remainder never
runs; a public file suspended inside a nested guest `Get` is rolled back after
retirement; a script keeps its earlier forms; a request refused at admission
leaves the pre-existing pending request intact. Keep the binary-level controls
that already pass: every non-interactive mode still prints the prefix output,
reports the input requirement, exits 1 and runs no tail, and the stdin and
argument bindings still consume exactly their queued responses.

Run the permanent tests before changing production code and save the failing
output under `.temp/brief-555/red/`.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-552's report §2 (the CLI rows) and draft C;
  `.temp/brief-552/tests/zz_brief552_cli.rs`.
- `crates/rum-cli/src/main.rs:95-125` (mode dispatch), `:221-320` (the
  helpers and bindings), `crates/rum-cli/tests/cli.rs` (binary tests).
- `crates/rum-host/src/session.rs` (`cancel_handle`, `resume_with_input`, the
  batch rustdoc), `docs/guide/embedding.md:364-395`.
- `docs/guide/getting-started.md` — the CLI and REPL sections.

## Scope

- **Objective:** retirement in every `rum-cli` evaluation helper on every
  returning path; the permanent tests; CLI documentation of exit status,
  `$ScriptInput`/`$ScriptArgs` consumption, stdin-as-program versus stdin-as-
  data, and what a script that asks for input does.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventory positions; only fingerprints, hashes and positions may change, and
  any identity, status, disposition or exclusion change is a STOP.
- **Fence:** `rum-cli` only. No change to `rum-host`, the kernel, `rum-stdlib`
  source, `rum-repl`, `rum-terminal` or `rum-jupyter`; no batch resumability;
  no change to what a user sees from the command line beyond cleanup output
  that retirement now produces; no new dependency; no `unsafe`. Do not edit
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier briefs or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  command-line-visible change, before and after; where cleanup output goes.
  STOP and report anything that needs a change outside the fence, including a
  missing host API.

## rum / Rust pitfalls (read twice)

1. **Take the cancel handle before starting the request**; a handle taken
   after admission failed would belong to someone else's request.
2. **Process exit is not retirement.** A test that only checks the exit code
   proves nothing about the borrowed session; call the helper and then use the
   session.
3. **Queued binding responses are not a resume route.**
4. **Do not replay output** already printed before the input request.
5. **Stop-loss is keyed on progress**: reds turned green, controls still green,
   compile errors falling. Stop when two consecutive compiling attempts move
   none.

## Verification (MANDATORY — you run it; iterate to green)

Every command through `python3 scripts/run-bounded.py --timeout <seconds> --`,
bounds from measured runs, cargo as `env CARGO_BUILD_JOBS=4
CARGO_NET_OFFLINE=true cargo …`.

```
cargo fmt --check
cargo test -p rum-cli
cargo test -p rum-repl
cargo test --workspace
```

Plus the retention replay in the `--no-default-features` build as the template
describes. Paste commands, bounds, exit statuses and counts verbatim; list
native ZeroMQ skips. Record stdout, stderr and exit status for `--eval`,
`--file`, a positional file, `-` and the filter, each with a nested `Get` that
asks for input and with an input asked for twice. No microbench.

## Checkpointing — protect the scene

Scratch under `.temp/brief-555/`. After each milestone write
`.temp/brief-555/PROGRESS.md`: done and verified with evidence, the exact next
action, the tree's state, the counters.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository, user configuration included. No
  network. Commands: file reads and edits within scope, `rg`/`grep`, `python3`,
  bounded `cargo`, the built `rum-cli`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges; command-line-visible changes.
2. Verification — every command's verbatim result; red before and green after
   for every permanent test; the mode table.
3. Progress counters, disclosures, and anything unresolved.

---

## Execution report — brief-555 (stopped at the scope fence)

### 1. What changed

**Incomplete.** The four required retirement reds now pass, but I stopped at
this brief's instruction: “STOP and report anything that needs a change outside
the fence, including a missing host API.” No host/kernel change was made.

The blocking test interprets “Output that cleanup produces must reach the user
once” to include `Message[...]` diagnostics as well as `Print[...]`. Public-file
rollback restores the message buffer and erases cleanup diagnostics before the
host packages cancellation. The CLI has a print sink, but no message sink or
pre-rollback message output route. `Print` cleanup works; file cleanup messages
do not. Treating the requirement as Print-only would need the director's ruling;
I have not silently narrowed it or removed the failing assertion.

Source evidence: `crates/rum-runtime/src/runtime/session/session_helpers.rs:125`
snapshots messages; `:203` restores them. `session/batch_file.rs:64–65` performs
that restoration during abandonment, before `session/session_requests.rs:67`
packages the cancellation. The exact excerpts are saved in
`.temp/brief-555/boundary-evidence.txt`.

Changed files (all source edits are within rum-cli):

| File | Current lines | Change |
| --- | --- | --- |
| `crates/rum-cli/src/main.rs` | 8–10, 227–300, 330–350, 488–490 | Capture handles before admission, retire InputRequired before the broken-pipe return, retire unexpected setup input, select cancellation output history, attach the private test module. |
| `crates/rum-cli/src/input_retirement.rs` | 1–59 | New shared cancellation/one-reentry helper; match typed Cancelled outcomes, preserve the original input reason, avoid replay, deliver buffered cleanup prints. |
| `crates/rum-cli/src/eval_tests.rs` | 1–444 | Fifteen permanent tests call private helpers directly with HostSession; no include or copied main. Covers reuse, state, cleanup, output, broken pipe, bindings, terminal branch, admission and queued future input. |
| `crates/rum-cli/tests/input_requests.rs` | 1–170 | Six permanent binary tests cover all five modes, nested Get, two inputs, cleanup and binding consumption. |
| `docs/dev/codex/brief-555-cli-helpers-retire-input.md` | 165–393 | This report; original brief preserved byte-for-byte. |

Scratch files under `.temp/brief-555/` contain original copies/hashes, the bounded
recorder, command records, logs, `.rum` fixtures, before-mode stdout/stderr/status,
the source-boundary evidence, scope audit and checkpoint. No existing source file
other than main.rs changed before appending this report: 1,710 of 1,711 baseline
files remained byte-identical, with three new CLI files and no deletions.

Observable behavior in the candidate helper tests: the original input-required
reason remains the error; cleanup Print lines now go to **stdout once**, before
the error returns. Cleanup messages that the host retains join the existing
stderr diagnostics once, with the final synthetic cancellation diagnostic
suppressed. Public-file cleanup messages remain missing: this is the unresolved
boundary. A closed stdout during cleanup retires the session and preserves the
existing successful broken-pipe exit policy. Ordinary prefix output is not
replayed, and the abandoned tail does not run. No script mode reads an answer;
no reset or batch answer is used. Binary after-mode captures were not run after
the scope stop, so complete command-line equivalence is not claimed.

Organization: main.rs is 15,512 characters (warning range); new retirement logic
has a separate 2,352-character module and tests are in a sibling test module
(14,221 characters). A larger split of invocation parsing versus evaluation
would be unrelated churn in this brief and remains deferred. No dependencies,
unsafe, git operations, network operations or configuration changes were added.

### 2. Verification

All cargo commands used four jobs, offline mode and repository-local TMPDIR.
Every evidence command was bounded. Initial bounds use brief-554's measured
CLI 6.48s; later focused bounds use this run's measured 8.574s/5.16s. The mode
capture took 0.646s overall, with each of fifteen child CLI invocations bounded
at 10s. Full raw stdout/stderr is in each linked scratch log. Results below are
verbatim excerpts, not reconstructed test counts.

`.temp/brief-555/red/cli.log`

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture
test result: FAILED. 4 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 8.574, "log": ".temp/brief-555/red/cli.log"}
```

`.temp/brief-555/red/binary.log`

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test cli --test input_requests --test terminal --no-fail-fast -- --nocapture
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test cli --test input_requests --test terminal --no-fail-fast -- --nocapture", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 0.3, "log": ".temp/brief-555/red/binary.log"}
```

`.temp/brief-555/red/modes.log`

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp python3 .temp/brief-555/modes.py before
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp python3 .temp/brief-555/modes.py before", "bound_seconds": 30.0, "exit_status": 0, "elapsed_seconds": 0.646, "log": ".temp/brief-555/red/modes.log"}
```

`.temp/brief-555/red/binary-corrected.log`

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test input_requests -- --nocapture
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test input_requests -- --nocapture", "bound_seconds": 10.0, "exit_status": 101, "elapsed_seconds": 0.548, "log": ".temp/brief-555/red/binary-corrected.log"}
```

`.temp/brief-555/red/helpers-final.log`

```text
$ python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --bin rum-cli -- --nocapture
test result: FAILED. 4 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --bin rum-cli -- --nocapture", "bound_seconds": 35.0, "exit_status": 101, "elapsed_seconds": 5.16, "log": ".temp/brief-555/red/helpers-final.log"}
```

`.temp/brief-555/green/format.log`

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo fmt -p rum-cli
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo fmt -p rum-cli", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 0.095, "log": ".temp/brief-555/green/format.log"}
```

`.temp/brief-555/green/cli.log`

```text
$ python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture", "bound_seconds": 35.0, "exit_status": 101, "elapsed_seconds": 7.017, "log": ".temp/brief-555/green/cli.log"}
```

The first binary run had two incorrect test expectations: Print renders lists
with braces, while final result values use canonical List syntax. Those two
expectations were corrected **before production changes**, and the corrected
binary baseline shows five passing controls and only the cleanup-output red.

Permanent-test before/after evidence:

| Test | Before production edits | Candidate |
| --- | --- | --- |
| eval_helper_abandons | FAIL | PASS |
| script_helper_abandons | FAIL | PASS |
| file_helper_abandons | FAIL | PASS |
| unexpected_setup_input_abandons | FAIL | PASS |
| shared_request_helper_abandons | FAIL | PASS |
| cleanup_output_appears_once_and_tail_never_runs | FAIL | PASS, eval/script/file/setup |
| public_file_nested_get_rolls_back_inner_to_outer | FAIL | PASS; inner:3 then outer:2; final marker 1 |
| script_keeps_earlier_forms_after_nested_get_retirement | FAIL | PASS; before 7, cleanup count 1, tail 0 |
| broken_pipe_during_cleanup_still_retires_input | FAIL | PASS; same session evaluates 2 |
| cleanup_messages_appear_once_with_original_input_reason | FAIL | FAIL on file; eval/script child cases pass, setup child not reached |
| retirement_does_not_consume_a_queued_future_answer | FAIL | PASS |
| refused_admission_preserves_someone_elses_pending_request | PASS | PASS for all five helpers |
| bindings_consume_exactly_their_queued_responses | PASS | PASS |
| terminal_stdin_does_not_make_scripts_interactive | PASS | PASS; forced terminal branch, no physical TTY script probe |
| already_broken_pipe_returns_with_reusable_session | PASS | PASS |
| all_script_modes_report_input_and_skip_tail | PASS | Not reached after unit failure |
| all_script_modes_stop_at_first_of_two_inputs | PASS | Not reached after unit failure |
| all_script_modes_report_nested_get_input | PASS | Not reached after unit failure |
| all_script_modes_deliver_cleanup_once_without_cancel_diagnostic | FAIL | Not reached after unit failure |
| program_modes_consume_stdin_and_arguments_as_binding_data | PASS after preimplementation fixture correction | Not reached after unit failure |
| stdin_program_modes_bind_only_arguments | PASS after preimplementation fixture correction | Not reached after unit failure |

All tests were run before production changes. The four required reds fail at
the following request, with this verbatim diagnostic:

```text
expected 2, got Failure(HostFailure { code: Other, exit_status: None, message: "cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })
```

The candidate's remaining failure is verbatim:

```text
assertion `left == right` failed: file: ! notice::before: before-message
! input required: p: 

  left: 0
 right: 1
```

Before-mode captures below preserve stdout/stderr exactly using JSON escapes.
Each command and its stdin are recorded verbatim in
`.temp/brief-555/modes/before/rows.json`; separate `.stdout`, `.stderr` and `.exit`
files are also saved. All program modes were supplied piped data, and every mode
received arguments after `--`. For `-` and filter, stdin carried source.

| Case | Mode | stdout | stderr | Exit | After |
| --- | --- | --- | --- | --- | --- |
| nested | --eval | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1 | Not run: scope stop |
| nested | --file | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1 | Not run: scope stop |
| nested | positional | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1 | Not run: scope stop |
| nested | - | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1 | Not run: scope stop |
| nested | filter | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1 | Not run: scope stop |
| twice | --eval | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1 | Not run: scope stop |
| twice | --file | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1 | Not run: scope stop |
| twice | positional | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1 | Not run: scope stop |
| twice | - | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1 | Not run: scope stop |
| twice | filter | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1 | Not run: scope stop |
| cleanup | --eval | `"prefix\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1 | Not run: scope stop |
| cleanup | --file | `"prefix\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1 | Not run: scope stop |
| cleanup | positional | `"prefix\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1 | Not run: scope stop |
| cleanup | - | `"prefix\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1 | Not run: scope stop |
| cleanup | filter | `"prefix\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1 | Not run: scope stop |

The existing binary controls passed before implementation: 28 ordinary CLI
and nine real-PTY tests. The first candidate CLI run stopped at its one failing
unit test, so its integration/PTY targets did not execute.

**Not run after the scope stop:** `cargo fmt --check`, `cargo test -p rum-repl`,
`cargo test --workspace`, contract fingerprint regeneration, both certificate
replays, the no-default-features retention gate, ownership-position
reconciliation, and the after-mode capture. `cargo fmt -p rum-cli` did run.
The required `cargo test -p rum-cli` ran and failed (14 unit tests passed, one
failed); there is no green full CLI gate. Native ZeroMQ tests did not run, so
there are no observed native skips to list. No microbenchmark was run, as
instructed. CLI documentation remains unedited and unfinished.

### 3. Progress counters, disclosures and unresolved work

- Original reds green: **4/4**.
- Added unit retirement reds green: **6/7**; cleanup Message loss in a public
  file remains red. The added binary cleanup red has no candidate result yet.
- Existing/new unit controls green: **4/4**.
- Compiler errors: **0** in every test attempt.
- Compiling implementation attempts: **1**, with ten unit reds turned green;
  stagnant compiling attempts: **0**. This is a scope stop, not stop-loss.
- Retention fingerprints/certificates and ownership inventory positions were
  **not replayed**. They remain untouched and may now be stale. No identity,
  status, disposition, exclusion or ownership-inventory row was changed.
- Production helper bodies compile, and the complete candidate is left in the
  tree. No files are half-written. The preserved failing test is intentional
  evidence, so the tree is **not ready for a green authoritative gate**.

The precise decision needed is whether cleanup output in this brief includes
Message diagnostics discarded by existing file rollback. If it does, the host
or kernel must preserve/expose those diagnostics, requiring work outside this
fence. If the ruling is Print-only with existing transactional Message behavior,
record that contract explicitly before adjusting the additional test. Then
finish CLI documentation, standing retention/inventory maintenance, all mandatory
gates and after-mode evidence. No permission for an outside-fence change was
assumed, and no git command ran.

---

## Resume — the director's decision on cleanup messages

You stopped correctly: the question was mine to answer. **Decision: "cleanup
output reaches the user once" means `Print` output only. Messages keep the
existing transactional semantics of a file load.** A public file's rollback
restores its loader snapshot, messages included, and brief-549's design
recorded that as the file-load rule (§4: loader messages roll back, prints are
not transactional and remain). Cancelling a suspended public file is that
rollback, so a message emitted by cleanup during its abandonment is rolled back
with the rest of the file's messages. Messages emitted before the input
request already reached the caller with the input-required report. Do not
change the host or the kernel.

So:

1. Change the cleanup-message test to assert the contract as it is: for a
   **public file**, a `Message` emitted by cleanup during retirement does not
   reach the user; for a **script** (not transactional), the same cleanup
   `Message` reaches the user exactly once. Keep the `Print` assertions: cleanup
   `Print` output reaches stdout once in both. Run the script case first — if
   it does not behave as stated, STOP and report instead of adjusting it.
2. Document it in the CLI section of `docs/guide/getting-started.md`: when a
   script or file needs input, the CLI reports it, cancels, runs cleanup once,
   and exits 1; a file's messages from that point are rolled back with it.
3. Finish what remains: the standing retention replay and ownership-inventory
   reconciliation, the binary cleanup red you added, the after-change mode
   captures, and every gate in Verification.

Continue from `.temp/brief-555/PROGRESS.md`; do not redo what is done. Append
your report covering both runs.


---

## Execution report — brief-555 completed after the director's resume ruling

### 1. What changed across both runs

The helpers now retire every input suspension they admitted before returning,
including the broken-pipe path, and preserve the original input-required reason.
The first run stopped at the public-file cleanup-message question. The director
settled it as Print-only output with existing transactional messages. The resumed
run changed that test as directed, ran its script case first, and completed all
remaining work. No additional production edit was needed on resume.

| File | Final lines | Why |
| --- | --- | --- |
| `crates/rum-cli/src/main.rs` | 8–10, 227–302, 330–350, 489–490 | Capture the request handle before admission, retire InputRequired before returning, select host output-history semantics, and attach the private helper tests. |
| `crates/rum-cli/src/input_retirement.rs` | 1–59 | Shared cancel/one guest-evaluation reentry; match typed Cancelled, preserve the input reason, suppress only the synthesized cancellation notice, and deliver cleanup prints without replay. |
| `crates/rum-cli/src/eval_tests.rs` | 1–452; resumed change 383–432 | Fifteen crate-local helper tests use HostSession directly. The resumed cleanup-message case checks script first, expects file cleanup messages to roll back, and checks one cleanup Print in every helper case. |
| `crates/rum-cli/tests/input_requests.rs` | 1–170 | Six binary tests cover all five modes, nested Get, two requests, cleanup once, and exact stdin/argument binding consumption. |
| `docs/guide/getting-started.md` | 35–36, 140–153, 159–187 | Document terminal dispatch, program versus data stdin, literal binding consumption, non-interactive input, cleanup, file-message rollback, and exit policy. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 2–69, column 19 only | Replay all 68 source-dependent fingerprints under the standing maintenance duty. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 2–3, columns 5–6 only | Replay C01/C02 certificate cells for both feature builds; only hash values changed. |
| `docs/dev/codex/brief-555-cli-helpers-retire-input.md` | 165–393 (first report), 429–918 (completion) | Append this combined completion report; preserve the initial brief, first report, and director ruling verbatim. |

Command-line-visible behavior: all five modes still print their prefix, report
the original input requirement on stderr, exit 1, and run no abandoned tail.
The added observable output is cleanup `Print` on **stdout exactly once**.
Nontransactional cleanup `Message` diagnostics now reach stderr once; cleanup
messages inside a rolled-back public file remain absent. Prefix messages remain
visible, and retirement adds no synthetic cancellation error. A closed stdout
still returns the existing successful exit status after retirement. A terminal
on stdin does not make a script interactive. No answers are invented, queued
bindings are consumed exactly, and no reset or batch resume-with-answer is used.

The same borrowed session evaluates `1+1` to `2` after each helper's input error.
The nested-file test observes cleanup `inner:3`, then `outer:2`, with the final
marker restored to 1. The script test retains its earlier value 7, one cleanup,
and no tail. Rejected admission preserves and can still resume its original
owner's request; retirement also leaves a separately queued future answer intact.

Organization: main.rs remains 15,512 characters in the warning range. Retirement
has its own 2,352-character module; helper tests are 14,636 characters in a sibling
test module, below the test-file preferred threshold. A further split of main's
invocation parsing and evaluation glue would be unrelated churn and is deferred.
No dependencies or unsafe were added. There are no host, kernel, stdlib-source,
REPL, terminal, Jupyter, configuration, or unrelated documentation edits.

### 2. Verification across both runs

All cargo invocations use four jobs, offline mode, and repository-local TMPDIR.
The recorder delegates each verification command to `scripts/run-bounded.py`;
resumed recorder invocations themselves have an outer bound ten seconds larger.
Bounds follow the measured checkpoint: reads below 0.1s (10s cap), CLI 8.574s
(35–40s), REPL 8.307s (45s), formatting 1.648s (10s), fingerprint regeneration
19.648s (80s), certificate replay 33.79/35.317s (120s), no-default retention
233.122s (600s), workspace 274.778s (650s). Each captured CLI invocation uses a
10s bound; the whole mode capture uses 30s, following the measured 0.646s run.
No microbenchmark ran, as required.

The blocks below copy actual command, result and status lines from every recorded
verification invocation in both runs. Full stdout/stderr remains at each path;
compiler/test durations and counts are verbatim, not reconstructed. Expected
certificate refresh failures are included: both builds wrote zero-rejection
certificates before rejecting the previously stored stale cell; the full gates
then replayed the updated cells successfully.

`.temp/brief-555/red/cli.log`

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
test result: FAILED. 4 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 8.574, "log": ".temp/brief-555/red/cli.log"}
```

`.temp/brief-555/red/binary.log`

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test cli --test input_requests --test terminal --no-fail-fast -- --nocapture
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test cli --test input_requests --test terminal --no-fail-fast -- --nocapture", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 0.3, "log": ".temp/brief-555/red/binary.log"}
```

`.temp/brief-555/red/modes.log`

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp python3 .temp/brief-555/modes.py before
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp python3 .temp/brief-555/modes.py before", "bound_seconds": 30.0, "exit_status": 0, "elapsed_seconds": 0.646, "log": ".temp/brief-555/red/modes.log"}
```

`.temp/brief-555/red/binary-corrected.log`

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test input_requests -- --nocapture
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --test input_requests -- --nocapture", "bound_seconds": 10.0, "exit_status": 101, "elapsed_seconds": 0.548, "log": ".temp/brief-555/red/binary-corrected.log"}
```

`.temp/brief-555/red/helpers-final.log`

```text
$ python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --bin rum-cli -- --nocapture
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
test result: FAILED. 4 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --bin rum-cli -- --nocapture", "bound_seconds": 35.0, "exit_status": 101, "elapsed_seconds": 5.16, "log": ".temp/brief-555/red/helpers-final.log"}
```

`.temp/brief-555/green/format.log`

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo fmt -p rum-cli
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo fmt -p rum-cli", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 0.095, "log": ".temp/brief-555/green/format.log"}
```

`.temp/brief-555/green/cli.log`

```text
$ python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli -- --nocapture", "bound_seconds": 35.0, "exit_status": 101, "elapsed_seconds": 7.017, "log": ".temp/brief-555/green/cli.log"}
```

`.temp/brief-555/resume/cleanup-contract.log`

```text
$ python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --bin rum-cli eval_tests::cleanup_messages_appear_once_with_original_input_reason -- --exact --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 35 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli --bin rum-cli eval_tests::cleanup_messages_appear_once_with_original_input_reason -- --exact --nocapture", "bound_seconds": 35.0, "exit_status": 0, "elapsed_seconds": 5.114, "log": ".temp/brief-555/resume/cleanup-contract.log"}
```

`.temp/brief-555/resume/fmt.log`

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo fmt --check
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo fmt --check", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 1.656, "log": ".temp/brief-555/resume/fmt.log"}
```

`.temp/brief-555/resume/cli.log`

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-cli", "bound_seconds": 40.0, "exit_status": 0, "elapsed_seconds": 0.351, "log": ".temp/brief-555/resume/cli.log"}
```

`.temp/brief-555/resume/repl.log`

```text
$ python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-repl
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-repl", "bound_seconds": 45.0, "exit_status": 0, "elapsed_seconds": 0.495, "log": ".temp/brief-555/resume/repl.log"}
```

`.temp/brief-555/resume/modes.log`

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp python3 .temp/brief-555/modes.py after
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp python3 .temp/brief-555/modes.py after", "bound_seconds": 30.0, "exit_status": 0, "elapsed_seconds": 0.748, "log": ".temp/brief-555/resume/modes.log"}
```

`.temp/brief-555/resume/retention/regenerate.log`

```text
$ python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.13s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture", "bound_seconds": 80.0, "exit_status": 0, "elapsed_seconds": 15.806, "log": ".temp/brief-555/resume/retention/regenerate.log"}
```

`.temp/brief-555/resume/retention/earn-default.log`

```text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test tests::retention_closure::local::production::local_production_acceptance ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 18.33s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 32.127, "log": ".temp/brief-555/resume/retention/earn-default.log"}
```

`.temp/brief-555/resume/retention/earn-no-default.log`

```text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test tests::retention_closure::local::production::local_production_acceptance ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 18.62s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 33.668, "log": ".temp/brief-555/resume/retention/earn-no-default.log"}
```

`.temp/brief-555/resume/retention/no-default.log`

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test tests::retention_closure::local::production::local_production_acceptance ... ok
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 305.69s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture", "bound_seconds": 600.0, "exit_status": 0, "elapsed_seconds": 319.91, "log": ".temp/brief-555/resume/retention/no-default.log"}
```

`.temp/brief-555/resume/workspace.log`

```text
$ python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.96s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.72s
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test tests::retention_closure::local::production::local_production_acceptance ... ok
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 267.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.44s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.89s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.74s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-555/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture", "bound_seconds": 650.0, "exit_status": 0, "elapsed_seconds": 307.053, "log": ".temp/brief-555/resume/workspace.log"}
```

The first run's binary expectations for printed lists were corrected before
production changes (surface braces, not canonical List syntax). The final red
baseline is `red/helpers-final.log` plus `red/binary-corrected.log`. The final
cleanup-message assertion was adjusted only under the explicit director ruling:
script first, file rollback expected, cleanup Print asserted once. It was not
rerun against preimplementation source on resume; the preserved red evidence
belongs to its earlier version. No completed baseline work was redone.

Permanent-test result mapping (`FAIL`/`PASS` below maps the saved FAILED/ok results):

| Permanent test | Before production edits | Final |
| --- | --- | --- |
| `eval_helper_abandons` | FAIL | PASS |
| `script_helper_abandons` | FAIL | PASS |
| `file_helper_abandons` | FAIL | PASS |
| `unexpected_setup_input_abandons` | FAIL | PASS |
| `shared_request_helper_abandons` | FAIL | PASS |
| `cleanup_output_appears_once_and_tail_never_runs` | FAIL | PASS |
| `public_file_nested_get_rolls_back_inner_to_outer` | FAIL | PASS |
| `script_keeps_earlier_forms_after_nested_get_retirement` | FAIL | PASS |
| `refused_admission_preserves_someone_elses_pending_request` | PASS | PASS |
| `bindings_consume_exactly_their_queued_responses` | PASS | PASS |
| `terminal_stdin_does_not_make_scripts_interactive` | PASS | PASS |
| `broken_pipe_during_cleanup_still_retires_input` | FAIL | PASS |
| `already_broken_pipe_returns_with_reusable_session` | PASS | PASS |
| `cleanup_messages_appear_once_with_original_input_reason` | FAIL; director clarified file expectation on resume | PASS |
| `retirement_does_not_consume_a_queued_future_answer` | FAIL | PASS |
| `all_script_modes_report_input_and_skip_tail` | PASS | PASS |
| `all_script_modes_stop_at_first_of_two_inputs` | PASS | PASS |
| `all_script_modes_report_nested_get_input` | PASS | PASS |
| `all_script_modes_deliver_cleanup_once_without_cancel_diagnostic` | FAIL | PASS |
| `program_modes_consume_stdin_and_arguments_as_binding_data` | PASS | PASS |
| `stdin_program_modes_bind_only_arguments` | PASS | PASS |

Mode table: stdout and stderr use JSON escapes, preserving trailing spaces and
newlines. Each before/after pair has identical arguments, stdin, stderr and exit
status. Only cleanup stdout gains one line. Exact command lines and stdin for
all thirty invocations are in `.temp/brief-555/modes/{before,after}/rows.json`;
separate `.stdout`, `.stderr`, and `.exit` files preserve each result. Raw MODE
records also appear in `red/modes.log` and `resume/modes.log`.

| Case | Mode | stdout before | stdout after | stderr both | Exit before/after |
| --- | --- | --- | --- | --- | --- |
| nested | --eval | `"prefix\n"` | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1/1 |
| nested | --file | `"prefix\n"` | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1/1 |
| nested | positional | `"prefix\n"` | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1/1 |
| nested | - | `"prefix\n"` | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1/1 |
| nested | filter | `"prefix\n"` | `"prefix\n"` | `"! input required: nested: \nerror: input required: nested: \n"` | 1/1 |
| twice | --eval | `"prefix\n"` | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1/1 |
| twice | --file | `"prefix\n"` | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1/1 |
| twice | positional | `"prefix\n"` | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1/1 |
| twice | - | `"prefix\n"` | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1/1 |
| twice | filter | `"prefix\n"` | `"prefix\n"` | `"! input required: first: \nerror: input required: first: \n"` | 1/1 |
| cleanup | --eval | `"prefix\n"` | `"prefix\ncleanup\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1/1 |
| cleanup | --file | `"prefix\n"` | `"prefix\ncleanup\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1/1 |
| cleanup | positional | `"prefix\n"` | `"prefix\ncleanup\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1/1 |
| cleanup | - | `"prefix\n"` | `"prefix\ncleanup\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1/1 |
| cleanup | filter | `"prefix\n"` | `"prefix\ncleanup\n"` | `"! input required: p: \nerror: input required: p: \n"` | 1/1 |

Final aggregate counts (summed from the verbatim test-result lines above):

- `resume/cli`: 58 passed, 0 failed, 0 ignored.
- `resume/repl`: 70 passed, 0 failed, 0 ignored.
- `resume/retention/no-default`: 49 passed, 0 failed, 1 ignored.
- `resume/workspace`: 2811 passed, 0 failed, 58 ignored.

Native ZeroMQ output, verbatim; these permitted early returns count as harness passes, not exercised native transport tests:

```text
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
skipping native ZeroMQ test: loopback socket binding is not permitted; test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... RUM_ALLOW_NATIVE_ZEROMQ_SKIPok explicitly allows this skip
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; okRUM_ALLOW_NATIVE_ZEROMQ_SKIP
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIPtest zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ...  explicitly allows this skip
skipping native ZeroMQ test: loopback socket binding is not permitted; test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... RUM_ALLOW_NATIVE_ZEROMQ_SKIPok explicitly allows this skip
skipping native ZeroMQ test: loopback socket binding is not permitted; test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... RUM_ALLOW_NATIVE_ZEROMQ_SKIPok explicitly allows this skip
skipping native ZeroMQ test: loopback socket binding is not permitted; test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... RUM_ALLOW_NATIVE_ZEROMQ_SKIPok explicitly allows this skip
```

Affected native tests:

```text
zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets
zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket
zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip
zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages
zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata
zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown
zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply
zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages
zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end
```

Standing maintenance: all 68 local-contract rows below changed only their last
fingerprint field; `resume/retention/contract-changes.json` preserves every
old/new value. C01 and C02 each changed both build cells (four cells total);
`certificate-changes.json` preserves them, and comparison after normalizing only
`fnv1a64` values proved the remaining certificate text byte-identical. Each build
still has two production body rows, three instance marks, four separate probe
body marks, zero fixture marks and zero helper-body marks. The 62 Discharged and
six Pending contracts retain their statuses; the four unresolved consumers and
all exclusions are unchanged. Other retention inventory files are byte-identical.

Changed local-contract row IDs, in file order:

```text
2	B01	fnv1a64:c6bd57c335a29f17 -> fnv1a64:746e7619509a483f
3	B02	fnv1a64:e4e072cee099a80e -> fnv1a64:062bd5887ac78dc6
4	B03	fnv1a64:b8bc29c5b53efb19 -> fnv1a64:2965822374afc2a1
5	B04	fnv1a64:6d3b8cfbdb3764df -> fnv1a64:4d415de07e4ddec7
6	B05	fnv1a64:bc5318dcd3f9e2ea -> fnv1a64:3909a19cf61c901a
7	B06	fnv1a64:992e702c69f6ed8b -> fnv1a64:19d30fc03700769b
8	B07	fnv1a64:3d8d91bac96d4f7a -> fnv1a64:34cbdecfb5d3c12a
9	B08	fnv1a64:cc0a66f341f6cf35 -> fnv1a64:09809b50b8259885
10	B09	fnv1a64:8985ecb417fa5453 -> fnv1a64:388d30f11737db63
11	B10	fnv1a64:69ddd58bbdb4cf80 -> fnv1a64:64e9fb040c440ae0
12	B11	fnv1a64:2e933e58f52b8adf -> fnv1a64:0e990f3d984204c7
13	B12	fnv1a64:48926caff402e953 -> fnv1a64:f799b0ecf3407063
14	I01	fnv1a64:a49f42c0a25a9597 -> fnv1a64:52506116bd523ebf
15	I02	fnv1a64:13949670cc306aea -> fnv1a64:904b1f30ee53181a
16	I03	fnv1a64:764679a1cdec4a13 -> fnv1a64:db8ba7ca13686e23
17	I04	fnv1a64:aee6ecdfcdbd2dae -> fnv1a64:e6dfd11f5b2273e6
18	P01	fnv1a64:074eb82a253052ab -> fnv1a64:da1198b36cac3fbb
19	P02	fnv1a64:a2986655ebb053f0 -> fnv1a64:5ac94c8d07c68450
20	P03	fnv1a64:5c2164e80f99f6b1 -> fnv1a64:26b5ffd92d081bd9
21	P04	fnv1a64:d9bb90e46542c40e -> fnv1a64:fb06f39dff70a9c6
22	P05	fnv1a64:0e854047bb64b4c7 -> fnv1a64:55114e0738676baf
23	P06	fnv1a64:99119180803282d3 -> fnv1a64:4818d5bd7f7009e3
24	P070	fnv1a64:d35f2dc25ea7fcd3 -> fnv1a64:826671ff5de583e3
25	P080	fnv1a64:ea130752d4115b40 -> fnv1a64:374358be6e73cea0
26	P090	fnv1a64:39b8673feeb8f3ce -> fnv1a64:64adb661dc4ffd86
27	P100	fnv1a64:778981d470e5ebef -> fnv1a64:f74d7aa1252fee57
28	P110	fnv1a64:93f4c3534e906538 -> fnv1a64:b8712b087891dfb8
29	P120	fnv1a64:910e92d2f417ca55 -> fnv1a64:e1b319e1621b3ca5
30	P130	fnv1a64:e15aeb17f821297a -> fnv1a64:d899382ce4879b2a
31	P140	fnv1a64:749f978dac41b186 -> fnv1a64:0abe6a84ad8d0b1e
32	P150	fnv1a64:7645f8e9e3c8526e -> fnv1a64:780b084fd4b371a6
33	P071	fnv1a64:621feea8b237e99a -> fnv1a64:5d5910bd05ab5bca
34	P081	fnv1a64:f454dff6a9d1c873 -> fnv1a64:aeec6d15da9e5e83
35	P091	fnv1a64:b5455b33405d1b6b -> fnv1a64:e33e9697bfb6627b
36	P101	fnv1a64:8bf7b42083f40238 -> fnv1a64:b0741bd5adf57cb8
37	P111	fnv1a64:5c2b85d14f748823 -> fnv1a64:df53400e91243cb3
38	P121	fnv1a64:e95d4b705ca2fce4 -> fnv1a64:18c4c165e0f1bc3c
39	P131	fnv1a64:0ae5af62118b865f -> fnv1a64:eaeb8046b4a20047
40	P141	fnv1a64:44149e34332e754b -> fnv1a64:0643225b840c335b
41	P151	fnv1a64:f86dbe4a85170b95 -> fnv1a64:bdf1aaaf23753ee5
42	B13	fnv1a64:273f03b59a93a98b -> fnv1a64:a7e3a349679d329b
43	R01	fnv1a64:df0e494293bb8a1e -> fnv1a64:737680e06d3afd56
44	R02	fnv1a64:63d594e9b1cf4671 -> fnv1a64:1e496580d4abe499
45	R03	fnv1a64:89b91ee1181f4082 -> fnv1a64:79f7ae01a70243d2
46	R040	fnv1a64:7ef00754b6cec23e -> fnv1a64:c56df7832f655676
47	R050	fnv1a64:f34365291cad6da7 -> fnv1a64:3896178046890b8f
48	R060	fnv1a64:9585c696a265bf56 -> fnv1a64:4915ae8daaffabae
49	R070	fnv1a64:5cc2a8108b738773 -> fnv1a64:175a352fbc401d83
50	R080	fnv1a64:75eb55d29d41f6ae -> fnv1a64:ade43a122aa73ce6
51	R090	fnv1a64:dfbe7c8b9b8bcfaf -> fnv1a64:ac6caa568af17817
52	R100	fnv1a64:7de1ce185f14a599 -> fnv1a64:ee8b26761e856d21
53	R110	fnv1a64:6ef760d1f1550888 -> fnv1a64:901c10e751ae0f08
54	R041	fnv1a64:0d7866dfb1bb688b -> fnv1a64:8e1d06737ec4f19b
55	R051	fnv1a64:c7da95557be7004e -> fnv1a64:f2cfe477697e0a06
56	R061	fnv1a64:aac266ebd8651aa5 -> fnv1a64:f11e43a6d9da1435
57	R071	fnv1a64:b0e7ed9f1eefa1ac -> fnv1a64:f62605a5e35f96c4
58	R081	fnv1a64:3373eb290c91c0ef -> fnv1a64:b337e3f5c0dbc357
59	R091	fnv1a64:7996358c404665c6 -> fnv1a64:0d727237cea8aa5e
60	R101	fnv1a64:3b5b8ceb0075ed16 -> fnv1a64:d9dfe6e1295a116e
61	R111	fnv1a64:18baf36e967a60dd -> fnv1a64:9b3918358f0778ad
62	R12	fnv1a64:787b4f2d566df41d -> fnv1a64:9df3e1e0e5c382ed
63	R13	fnv1a64:3af875b39aa13c49 -> fnv1a64:83e62ceeb0959591
64	R14	fnv1a64:91e1904a4d96abef -> fnv1a64:11a5891701e0ae57
65	R15	fnv1a64:0a46b3fcd3a86f7b -> fnv1a64:6952bd00e0e67dcb
66	O010	fnv1a64:fa1dab67c30fd085 -> fnv1a64:597112b273bbbc95
67	O0200	fnv1a64:6f673961faf8d786 -> fnv1a64:05860c58fc44311e
68	O011	fnv1a64:d80e831eaff783d6 -> fnv1a64:8b9e6b15b891702e
69	O0210	fnv1a64:f16daba6940791fd -> fnv1a64:0af3c36a7567b2cd
```

All seven ownership inventory files remain byte-identical. The workspace discovery reconciled them without position changes:

```text
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
```

### 3. Progress counters, disclosures, and unresolved work

- Original retirement reds green: **4/4**.
- Added helper retirement reds green: **7/7**, including the message contract
  clarified by the director. Added binary cleanup red green: **1/1**.
- Helper controls green: **4/4**; new binary controls **5/5**; existing CLI
  controls **28/28** and real-PTY controls **9/9**.
- Candidate compiler errors: **0**; intentional mutated-source compiler probes
  produce the expected negative diagnostics. One production implementation attempt in the first
  run; the resumed test-contract correction turns the last red green without
  another production change. Stagnant compiling implementation attempts: **0**.
- Both mandatory certificate builds and the full no-default retention gate
  completed. The two stale-cell failures during refresh are preserved above;
  no identity, status, disposition or exclusion changed.
- All mandatory gates are green. No source is half-written. The earlier scope
  stop is resolved by the director's ruling, and no outside-fence API change is
  needed. No unresolved implementation or verification work remains.

Not run: microbenchmarks (explicitly excluded), other-platform execution, or a
new physical-TTY script-mode probe. The script terminal branch is covered directly
and the nine existing real-PTY controls pass. Native ZeroMQ transport cases skip
only where loopback binding is forbidden, with the explicit existing test switch;
no network access or elevated execution was requested. The manual fingerprint
writer remains ignored in ordinary gates and was run explicitly as recorded.

The source/docs/scripts audit across both runs has eight changed or added files:
five baseline files changed, three CLI files added, no deletions, and 1,706
baseline files unchanged. The resumed run preserves the existing production
candidate. Its edits are the directed test correction, CLI guide, two standing
maintenance inventories, and this append-only report. Scratch, command records,
full logs, mode captures, hash-only checks, and checkpoints remain under
`.temp/brief-555/`; existing test tools write their prescribed review artifacts
under `target/` and `.temp/brief-546/certificates/`. No git command, dependency
change, new unsafe, configuration write, or unrelated source change occurred.

Exact next action: the director reviews this completed tree and runs the
authoritative gate. Git remains entirely with the director.
