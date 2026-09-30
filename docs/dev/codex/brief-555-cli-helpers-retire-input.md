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
