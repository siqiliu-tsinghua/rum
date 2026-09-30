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

## Implementation report — 2026-09-30

### 1. What changed

The CLI's Unix interactive REPL now accepts masked answers and cancels a pending
input request on SIGINT. The original source reader supplies every answer.

The file table below uses current line ranges. Only the brief's objective and
standing fingerprint maintenance were changed; no git command was run.

| File | Lines | Change |
|---|---|---|
| [crates/rum-repl/Cargo.toml](../../../crates/rum-repl/Cargo.toml) | 21–24, 27–29 | Unix-only authorized dependencies and test-only PTY feature. |
| [crates/rum-repl/src/request.rs](../../../crates/rum-repl/src/request.rs) | 24–24, 28–61, 66–66, 167–176 | Choose the answer adapter; prepare modes before prompt output; masked newline inside retirement boundary. |
| [crates/rum-repl/src/terminal.rs](../../../crates/rum-repl/src/terminal.rs) | 1–233 | Safe terminal mode guard, self-pipe SIGINT wait and buffered answer reading. |
| [crates/rum-repl/src/lib.rs](../../../crates/rum-repl/src/lib.rs) | 23–28, 31–32, 47–49, 60–61, 77–78, 80–95, 197–201, 289–293 | Terminal opt-in, facade documentation, and shared input wiring for source and trace. |
| [crates/rum-repl/src/terminal/tests.rs](../../../crates/rum-repl/src/terminal/tests.rs) | 1–122 | Three extra real-PTY restoration and read-ahead tests. |
| [crates/rum-cli/src/main.rs](../../../crates/rum-cli/src/main.rs) | 217–220 | Enable the terminal reader only in the interactive REPL. |
| [crates/rum-cli/tests/terminal.rs](../../../crates/rum-cli/tests/terminal.rs) | 1–69 | Run nine separate Python PTY scenarios in the cargo gate. |
| [crates/rum-cli/tests/terminal/driver.py](../../../crates/rum-cli/tests/terminal/driver.py) | 1–185 | Own isolated PTYs/processes; verify echo, history, attributes, errors, cancellation and outside-wait controls. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 2–69 | Replay 68 fingerprints; other columns unchanged. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/leaves.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/leaves.tsv) | 2–6 | Replay five fixture fingerprints expired by Cargo.lock; other columns unchanged. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | 2–3 | Refresh C01/C02 certificate hashes in both builds; all other bytes unchanged. |
| [docs/guide/getting-started.md](../../../docs/guide/getting-started.md) | 76–80, 85–91 | Document hidden entry, wait-only Ctrl-C and unsupported-reader refusal. |
| [Cargo.lock](../../../Cargo.lock) | 384–389, 682–683, 728–740, 853–862 | Add only the two authorized packages and their cached linux-raw-sys dependency. |
| This brief | 184–676 | Append this report; original brief preserved byte for byte. |

Exact additions to `crates/rum-repl/Cargo.toml`:

```toml
[target.'cfg(unix)'.dependencies]
rustix = { version = "1.1.4", default-features = false, features = ["std", "termios", "fs", "event", "pipe"] }
signal-hook = "0.3.18"

[target.'cfg(unix)'.dev-dependencies]
rustix = { version = "1.1.4", default-features = false, features = ["pty"] }
```

`Cargo.lock` adds exactly `rustix 1.1.4`, `signal-hook 0.3.18`, and the cached
transitive package `linux-raw-sys 0.12.1`, plus the two edges from `rum-repl`.
The existing `bitflags`, `errno`, `libc`, `windows-sys`, and
`signal-hook-registry` packages retain their versions. Its complete delta is:

```diff
--- before/Cargo.lock
+++ Cargo.lock
@@ -381,6 +381,12 @@
 ]
 
 [[package]]
+name = "linux-raw-sys"
+version = "0.12.1"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53"
+
+[[package]]
 name = "lock_api"
 version = "0.4.14"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -673,6 +679,8 @@
  "rum-runtime",
  "rum-syntax",
  "rum-terminal",
+ "rustix",
+ "signal-hook",
 ]
 
 [[package]]
@@ -720,6 +728,19 @@
 ]
 
 [[package]]
+name = "rustix"
+version = "1.1.4"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190"
+dependencies = [
+ "bitflags",
+ "errno",
+ "libc",
+ "linux-raw-sys",
+ "windows-sys",
+]
+
+[[package]]
 name = "rustversion"
 version = "1.0.22"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -829,6 +850,16 @@
 version = "2.0.1"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "f8fadd59c855ef2080decdef8ff161eb6661b86933c9d82e5ba29dc602a55aba"
+
+[[package]]
+name = "signal-hook"
+version = "0.3.18"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "d881a16cf4426aa584979d30bd82cb33429027e42122b169753d6ef1085ed6e2"
+dependencies = [
+ "libc",
+ "signal-hook-registry",
+]
 
 [[package]]
 name = "signal-hook-registry"
```

The reader guard captures terminal attributes and file flags. It clears `ECHO`
and `ECHONL` before showing a masked prompt, then enables `NONBLOCK` after
flushing that prompt: stdin and stdout may share an open file description.
All actual reads go through the existing reader's `fill_buf`/`consume`;
the duplicate descriptor controls modes only. Buffered bytes are consumed before
polling for more input. Restoration runs on normal return, EOF, interrupt,
read/prompt errors and panic unwinding, with interrupted restoration calls
retried. UTF-8 errors contain no answer bytes.

SIGINT writes to a scoped self-pipe registered through safe `signal-hook` APIs.
The wait polls that pipe and stdin. This wakes without Enter despite
`SA_RESTART`, and the input descriptor's nonblocking mode prevents a restarted
read from trapping the wait. Partial terminal input is discarded on interrupt.
The existing driver then cancels its host handle and re-enters exactly once;
no EOF or interrupt answer is delivered to guest code.

`signal-hook::unregister` removes an action without restoring the OS default
handler. Therefore one conditional-default action remains after the first wait:
it reproduces default SIGINT termination outside waits. The pipe action is
removed by `Drop`. `with_terminal_input` is an explicit opt-in for a process
owning this SIGINT policy and exclusive terminal input; `Repl::new` remains
plain. Controls verify default termination at the initial primary prompt,
after an answer, and during ordinary evaluation after an answer. No behavior
for a host's custom SIGINT handlers is claimed.

At the prompt, masked requests now accept hidden entry, with no placeholders,
history entry or reader diagnostics containing the answer. A successful masked
answer adds one newline before guest output because Enter was not echoed.
SIGINT during either kind of answer produces one `! input cancelled` notice,
delivers cleanup once, skips the abandoned remainder, and returns to `rum> `;
`1+1` then produces `2`. EOF still cancels and exits. Ordinary answers still
echo. Readers without terminal support retain the password refusal. Guest code
can print its answer after entry, as explicitly allowed by U4.

Organization assessment: the new terminal implementation is under 8k bytes,
with separate tests. The REPL facade (11.7k), request driver (11.2k), and existing
CLI entry point (14.6k) are in the warning range. This narrow change preserves
their current responsibilities; future useful boundaries are history
persistence versus the REPL loop, answer adapters versus request lifecycle,
and CLI parsing versus execution. No touched implementation reaches 20k.

### 2. Verification

Real PTY access succeeded in the sandbox:

```text
COMMAND: python3 scripts/run-bounded.py --timeout 10 -- python3 .temp/brief-554/red/pty_availability.py
PASS: opened a real pseudo-terminal and read its terminal attributes
EXIT STATUS: 0
```

The required baseline ran before production edits. Its corrected result was
four passing controls and five expected failures. The first baseline also had
an evaluation-control synchronization defect: guest `Print` output is retained
until evaluation ends. That test was corrected before production edits; its
initial failure is not counted as a product defect. The final PTY tests also
check descriptor restoration, partially typed interrupted answers, and absence
of the synthetic secret from read-error diagnostics.

| CLI PTY case | Corrected original-source baseline | Final CLI gate |
|---|---|---|
| `masked_answer` | FAILED: password refused | ok |
| `masked_eof` | FAILED: password refused | ok |
| `masked_io_error` | FAILED: password refused | ok |
| `masked_interrupt` | FAILED: password refused | ok |
| `interrupt` | FAILED: process terminated by SIGINT (-2) | ok |
| `ordinary_answer` | ok | ok |
| `primary_sigint` | ok | ok |
| `primary_sigint_after_answer` | ok | ok |
| `evaluation_sigint_after_answer` | ok | ok |

Three additional Rust tests use separate real PTYs: panic restoration,
prompt-write-error restoration, and consuming bytes already buffered by the
source reader. These were added after the first implementation, so no
preimplementation behavioral-red result is claimed for them. Their first run
found three compile errors (`Termios` has no `PartialEq`); the tests now compare
all exposed attributes through its complete Debug representation. All three
pass, as do the original REPL tests. The CLI checks compare the complete
`tcgetattr` attribute lists directly.

Timeouts use the prior brief's measured 8.14s REPL, 5.695s CLI and 268.873s
workspace runs, the earlier 17.28s certificate replay, and this brief's recorded
runs. Bounds include build headroom; later 95s certificate bounds derive from
this brief's 32.419s replay, and the final 60s CLI bound includes possible build
lock contention. Every cargo command used four build jobs and offline mode,
and all verification commands ran through `run-bounded.py`. Raw stdout/stderr
and command records are under `.temp/brief-554/`; excerpts below are verbatim.
Successful formatting commands emitted no stdout/stderr.

[Complete output: terminal.log](../../../.temp/brief-554/red/terminal.log)

```text
$ python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture
test result: FAILED. 3 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture", "bound_seconds": 60, "exit_status": 101, "elapsed_seconds": 3.506, "log": ".temp/brief-554/red/terminal.log"}
```

[Complete output: terminal-corrected.log](../../../.temp/brief-554/red/terminal-corrected.log)

```text
$ python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture
test result: FAILED. 4 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture", "bound_seconds": 20, "exit_status": 101, "elapsed_seconds": 0.244, "log": ".temp/brief-554/red/terminal-corrected.log"}
```

[Complete output: terminal-01.log](../../../.temp/brief-554/green/terminal-01.log)

```text
$ python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture
error[E0432]: unresolved imports `rustix::fs::FdFlags`, `rustix::fs::fcntl_setfd`
error[E0599]: no variant or associated item named `Input` found for enum `QueueSelector` in the current scope
RESULT {"command": "python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture", "bound_seconds": 60, "exit_status": 101, "elapsed_seconds": 1.499, "log": ".temp/brief-554/green/terminal-01.log"}
```

[Complete output: terminal-02.log](../../../.temp/brief-554/green/terminal-02.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli --test terminal -- --nocapture", "bound_seconds": 30, "exit_status": 0, "elapsed_seconds": 7.92, "log": ".temp/brief-554/green/terminal-02.log"}
```

[Complete output: format.log](../../../.temp/brief-554/green/format.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt -p rum-repl -p rum-cli
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt -p rum-repl -p rum-cli", "bound_seconds": 30, "exit_status": 0, "elapsed_seconds": 0.235, "log": ".temp/brief-554/green/format.log"}
```

[Complete output: repl-01.log](../../../.temp/brief-554/green/repl-01.log)

```text
$ python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-repl
error[E0369]: binary operation `==` cannot be applied to type `Termios`
error[E0369]: binary operation `==` cannot be applied to type `Termios`
error[E0369]: binary operation `==` cannot be applied to type `Termios`
RESULT {"command": "python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-repl", "bound_seconds": 45, "exit_status": 101, "elapsed_seconds": 9.966, "log": ".temp/brief-554/green/repl-01.log"}
```

[Complete output: repl-02.log](../../../.temp/brief-554/green/repl-02.log)

```text
$ python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-repl
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-repl", "bound_seconds": 45, "exit_status": 0, "elapsed_seconds": 8.811, "log": ".temp/brief-554/green/repl-02.log"}
```

[Complete output: format-final.log](../../../.temp/brief-554/green/format-final.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt -p rum-repl -p rum-cli
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt -p rum-repl -p rum-cli", "bound_seconds": 30, "exit_status": 0, "elapsed_seconds": 0.176, "log": ".temp/brief-554/green/format-final.log"}
```

[Complete output: cli.log](../../../.temp/brief-554/gate/cli.log)

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli", "bound_seconds": 40, "exit_status": 0, "elapsed_seconds": 7.25, "log": ".temp/brief-554/gate/cli.log"}
```

[Complete output: regenerate.log](../../../.temp/brief-554/retention/regenerate.log)

```text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.28s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture", "bound_seconds": 120, "exit_status": 0, "elapsed_seconds": 19.648, "log": ".temp/brief-554/retention/regenerate.log"}
```

[Complete output: earn-default.log](../../../.temp/brief-554/retention/earn-default.log)

```text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.16s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture", "bound_seconds": 120, "exit_status": 101, "elapsed_seconds": 32.419, "log": ".temp/brief-554/retention/earn-default.log"}
```

[Complete output: format-reviewed.log](../../../.temp/brief-554/green/format-reviewed.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt -p rum-repl -p rum-cli
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt -p rum-repl -p rum-cli", "bound_seconds": 30, "exit_status": 0, "elapsed_seconds": 0.228, "log": ".temp/brief-554/green/format-reviewed.log"}
```

[Complete output: repl.log](../../../.temp/brief-554/gate/repl.log)

```text
$ python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-repl
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 45 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-repl", "bound_seconds": 45, "exit_status": 0, "elapsed_seconds": 8.307, "log": ".temp/brief-554/gate/repl.log"}
```

[Complete output: regenerate-final.log](../../../.temp/brief-554/retention/regenerate-final.log)

```text
$ python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.18s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture", "bound_seconds": 60, "exit_status": 0, "elapsed_seconds": 0.33, "log": ".temp/brief-554/retention/regenerate-final.log"}
```

[Complete output: fmt.log](../../../.temp/brief-554/gate/fmt.log)

```text
$ python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt --check
RESULT {"command": "python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo fmt --check", "bound_seconds": 30, "exit_status": 0, "elapsed_seconds": 1.648, "log": ".temp/brief-554/gate/fmt.log"}
```

[Complete output: earn-default-final.log](../../../.temp/brief-554/retention/earn-default-final.log)

```text
$ python3 scripts/run-bounded.py --timeout 95 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.43s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 95 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture", "bound_seconds": 95, "exit_status": 101, "elapsed_seconds": 33.79, "log": ".temp/brief-554/retention/earn-default-final.log"}
```

[Complete output: earn-no-default.log](../../../.temp/brief-554/retention/earn-no-default.log)

```text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.49s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture", "bound_seconds": 120, "exit_status": 101, "elapsed_seconds": 35.317, "log": ".temp/brief-554/retention/earn-no-default.log"}
```

[Complete output: retention-no-default.log](../../../.temp/brief-554/gate/retention-no-default.log)

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture
expired-C01: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=false
test result: FAILED. 40 passed; 9 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 274.49s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture", "bound_seconds": 600, "exit_status": 101, "elapsed_seconds": 290.31, "log": ".temp/brief-554/gate/retention-no-default.log"}
```

[Complete output: refresh-leaves.log](../../../.temp/brief-554/retention/refresh-leaves.log)

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp python3 .temp/brief-554/refresh-leaves.py
Replayed five fixture-only fingerprints; all other fields byte-identical.
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp python3 .temp/brief-554/refresh-leaves.py", "bound_seconds": 10, "exit_status": 0, "elapsed_seconds": 0.132, "log": ".temp/brief-554/retention/refresh-leaves.log"}
```

[Complete output: cli-final.log](../../../.temp/brief-554/gate/cli-final.log)

```text
$ python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-cli", "bound_seconds": 60, "exit_status": 0, "elapsed_seconds": 6.48, "log": ".temp/brief-554/gate/cli-final.log"}
```

[Complete output: retention-no-default-final.log](../../../.temp/brief-554/gate/retention-no-default-final.log)

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 213.38s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture", "bound_seconds": 600, "exit_status": 0, "elapsed_seconds": 233.122, "log": ".temp/brief-554/gate/retention-no-default-final.log"}
```

[Complete output: workspace.log](../../../.temp/brief-554/gate/workspace.log)

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.76s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.62s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 236.65s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.54s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.57s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-554/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture", "bound_seconds": 600, "exit_status": 0, "elapsed_seconds": 274.778, "log": ".temp/brief-554/gate/workspace.log"}
```


Retention maintenance replayed 68 local contract fingerprints, both builds'
C01/C02 cells, and five older trusted-fixture fingerprints. The initial
certificate runs earned their marks with zero rejections, then rejected the
old stored cells, as expected. The first full no-default replay found nine
failures caused by the older leaf fingerprints' dependency on `Cargo.lock`.
The leaf refresh replays the owner's FNV-1a algorithm and first verifies that
the original lockfile reproduces the original stored hash. No test assertion
or admitted contract changed to fix these failures.

Changed maintenance rows are `local-contracts.tsv` rows 2–69 (all 68 IDs),
`promoted.tsv` rows 2–3 (C01/C02, hash=true and hash=false cells), and
`leaves.tsv` rows 2–6 (`start`, `callback`, `adopt`, `append`, `send`).
All non-fingerprint contract columns were compared byte for byte. Normalizing
only `fnv1a64` hashes leaves the old and new promoted rows byte-identical:
identities, statuses, dispositions, source anchors, preconditions, guarantees,
strengths and exclusions are unchanged. Ownership inventories require no edits.

Both feature replays earn the same mark counts. The default workspace gate also reconciles ownership exactly:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
```


Nine native ZeroMQ tests returned early under the explicit sandbox skip allowance. They count as passed in Rust's totals, but no socket execution is claimed for them. The log has nine skip notices (some interleaved):

```text
skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

- `native_zeromq_backend_round_trips_execute_over_loopback_sockets`
- `native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket`
- `native_zeromq_backend_frontend_style_control_interrupt_round_trip`
- `native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`
- `native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata`
- `native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown`
- `native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply`
- `native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages`
- `native_zeromq_backend_rejects_tampered_signed_requests_end_to_end`

### 3. Counters, disclosures and remaining limits

All mandatory gates passed. Workspace: **2790 passed, 0 failed, 58 ignored**, including the 9 explicitly disclosed native skips. REPL: 70 passed. CLI: 37 passed. No-default retention: 49 passed, 0 failed, 1 ignored (the manual fingerprint regenerator). No failing gate remains.

Implementation counters: 12 real-PTY cases passing (9 CLI, 3 reader), six
masked-restoration paths passing (answer, EOF, SIGINT, read error, prompt error,
panic), and zero compiler errors. No unsuccessful implementation repair reached
two consecutive compiling attempts without progress. Maintenance failures and
their repairs are preserved above rather than hidden.

Linux terminal behavior was exercised. Other Unix targets and non-Unix builds
were not executed. No microbenchmark ran, as the brief explicitly excludes it.
The three supplemental reader tests have no preimplementation behavioral-red
baseline; that evidence limitation is stated above. No new unsafe, network,
git, user-configuration changes, host/kernel executable changes, or
non-interactive CLI policy changes were introduced. Changes remain for the
director's gate; no commit was made. Checkpoint and evidence index:
[.temp/brief-554/PROGRESS.md](../../../.temp/brief-554/PROGRESS.md).
