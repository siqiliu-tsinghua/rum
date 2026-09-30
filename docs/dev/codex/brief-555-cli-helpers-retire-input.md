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
