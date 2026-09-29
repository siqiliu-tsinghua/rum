# brief-554 — the REPL's terminal reader: masked answers and Ctrl-C while waiting

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 3B of the suspension-continuation branch (brief-552, draft B, rewritten).
brief-553 (`230e79bb`) made the REPL answer input requests from its next line
and resume; it reads through a plain `BufRead`, so it refuses every password
request ("password input unavailable") and treats only an `Interrupted` read
error as an interrupt. When you are done, the interactive REPL that `rum-cli`
starts on a terminal reads answers through a terminal-aware reader: a password
request is answered with echo off, and Ctrl-C while the REPL is waiting for an
answer cancels that request and returns to the primary prompt.

## User rulings, 2026-09-30 — implement exactly these

- **U3 — Ctrl-C while waiting** cancels the pending request through the
  documented host route (cancel its handle, re-enter once) and returns to the
  primary prompt, with one concise notice and any guest cleanup output once.
  EOF while waiting cancels and leaves the REPL (already implemented; keep it).
- **U4 — password requests** (`HostInputRequest.password == true`, from
  `FieldMasked -> True`): no echo of the typed characters, no placeholder
  characters, never recorded in history or diagnostics; the terminal's state is
  restored on every exit — normal return, EOF, interrupt, I/O error and panic
  unwinding. A reader that cannot guarantee this keeps refusing with "password
  input unavailable"; **never fall back to visible entry.** Masking protects
  entry only; what guest code prints afterwards is its own business.
- **Dependencies authorized by the user: `rustix` and `signal-hook`.** The
  director verified that both resolve and build offline from this host's cargo
  cache: `rustix 1.1.4` (for example `default-features = false, features =
  ["std", "termios", "stdio"]`, plus whatever safe feature your tests need) and
  `signal-hook 0.3.18`. Build with `CARGO_NET_OFFLINE=true` (or `--offline`);
  you have no network. Add them only to the crate that needs them, and let
  `Cargo.lock` change for exactly these packages and their existing cached
  dependencies. **No other new dependency.**
- **Scope of the signal policy**: this brief handles Ctrl-C **only while the
  REPL is waiting for an answer**. What Ctrl-C does at the primary prompt or
  during an ordinary evaluation stays exactly as it is today.

## What exists now

- `crates/rum-repl/src/request.rs` (brief-553): `RequestInput` with
  `password_available()` and `read_answer(password)`, returning
  `InputEvent::{Answer, Eof, Interrupted}`; `PlainInput` over a `BufRead`
  reports `password_available() == false`. The driver `drive` abandons through
  the documented route on EOF, interrupt, password-unavailable and I/O error.
- `crates/rum-cli/src/main.rs:214-218` starts the REPL with
  `Repl::new(stdin.lock(), stdout.lock()).with_default_history()` when stdin
  is a terminal.
- `crates/rum-terminal` is a renderer only.

## Design constraints — these are the parts that are easy to get wrong

1. **One reader owns the buffered input.** The REPL reads lines through a
   buffered stdin lock. The terminal reader must read the answer **through
   that same buffered reader** — switching echo is a terminal-mode change on
   the file descriptor, not a reason to open a second reader that would lose
   bytes already buffered.
2. **Restore by construction.** Hold the terminal-mode change in a guard whose
   `Drop` restores the saved attributes, so every exit path — including a
   panic — restores them. Do not rely on each branch remembering.
3. **Ctrl-C must actually interrupt the wait.** Check what your signal
   registration does to a blocking read: a handler installed with `SA_RESTART`
   makes the read restart, so the REPL would not notice until Enter. Choose a
   mechanism that ends the wait promptly — and prove it with a test that sends
   the signal (or the terminal's interrupt character) while the read is
   blocked, not with a fake `Interrupted` error.
4. **Only while waiting.** Whatever you install for the wait is removed or
   inert outside it; the primary prompt and ordinary evaluation keep today's
   behaviour.
5. **No `unsafe` in our code.** Use the safe APIs of `rustix` and
   `signal-hook`. Their internals are theirs.

## Red tests first

Before changing production code, write and save failing under
`.temp/brief-554/red/`:

1. **Real terminal tests** using a pseudo-terminal (for example `rustix`'s pty
   support, in the test build only): a password request answered with a
   synthetic secret — the secret does not appear in what the terminal echoes
   back, the guest receives it exactly, the history file does not contain it,
   and the terminal's attributes after the answer equal those before; the same
   restoration on EOF and on an I/O error during the masked read; an ordinary
   (non-password) answer still echoes.
2. **Ctrl-C while waiting**, on the pseudo-terminal: the pending request is
   cancelled, cleanup runs once, the abandoned remainder does not run, the REPL
   is back at the primary prompt, and `1+1` then evaluates to `2`.
3. **Ctrl-C not waiting** keeps today's behaviour — a control, not a change.

If pseudo-terminals are unavailable in your sandbox, **say so, save the
failure, and stop before claiming any terminal behaviour**; do not substitute a
`Cursor` for a terminal and call it proof.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-552's report §3 (U3, U4) and draft B; brief-553's report and
  `crates/rum-repl/src/request.rs`, `lib.rs`, `tests/`.
- `crates/rum-cli/src/main.rs` (the REPL branch), `docs/guide/getting-started.md`
  (the REPL section brief-553 wrote).

## Scope

- **Objective:** the terminal answer reader (echo control and the wait-time
  interrupt), its wiring into the CLI's interactive REPL, the pseudo-terminal
  tests, the two dependencies, and the REPL documentation that describes
  masked input and Ctrl-C while waiting.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  any source change expires the retention contract fingerprints and
  certificates and moves ownership-inventory positions. Refresh them by
  replaying as the template describes; only fingerprints, hashes and positions
  may change, and any identity, status, disposition or exclusion change is a
  STOP.
- **Fence:** no change to `rum-host`, the kernel or `rum-stdlib` source; no
  change to what non-interactive CLI modes do with stdin; no Jupyter change; no
  signal policy outside the answer wait; no dependency beyond the two named;
  no `unsafe`. Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier
  briefs or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; the exact
  dependency entries and the `Cargo.lock` delta; the signal mechanism and why
  it ends a blocked read; every user-visible change at the prompt. STOP and
  report anything that needs a change outside the fence.

## rum / Rust pitfalls (read twice)

1. **A flag is not proof.** `password == true` reaching the reader does not
   show that echo was off; only the pseudo-terminal's output shows that.
2. **An `Interrupted` fixture is not a signal.** Prove the real path.
3. **Never inject an answer** on EOF or interrupt.
4. **Terminal state is global to the process.** A test that fails mid-read must
   still restore it; run pty tests on their own pty, never on the test
   runner's terminal.
5. **Stop-loss is keyed on progress**: terminal cases passing, restoration
   cases passing, compile errors falling. Stop when two consecutive compiling
   attempts move none.

## Verification (MANDATORY — you run it; iterate to green)

Every command through `python3 scripts/run-bounded.py --timeout <seconds> --`,
bounds from measured runs, cargo as `env CARGO_BUILD_JOBS=4
CARGO_NET_OFFLINE=true cargo …`.

```
cargo fmt --check
cargo test -p rum-repl
cargo test -p rum-cli
cargo test --workspace
```

Plus the retention replay in the `--no-default-features` build as the template
describes. Paste commands, bounds, exit statuses and counts verbatim; list any
native ZeroMQ skips. No microbench: nothing here touches evaluation.

## Checkpointing — protect the scene

Scratch under `.temp/brief-554/`. After each milestone write
`.temp/brief-554/PROGRESS.md`: done and verified with evidence, the exact next
action, the tree's state, the counters.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository, user configuration included. No
  network. Commands: file reads and edits within scope, `rg`/`grep`, `python3`,
  bounded `cargo`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges; dependencies and `Cargo.lock`
   delta; the signal mechanism.
2. Verification — every command's verbatim result; red before and green after
   for every pty test; the retention replay.
3. Progress counters, disclosures, and anything unresolved.
