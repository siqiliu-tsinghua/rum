# brief-556 — Jupyter: an execute that cannot take input retires its request

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 3D of the suspension-continuation branch (brief-552, draft D, rewritten).
`rum-jupyter` services guest input only on its asynchronous path with
`allow_stdin` true, where it sends `input_request` and resumes on the reply.
Everywhere else it returns a terminal `InputRequired` error — and leaves the
suspension installed in its `HostSession`, so the next execute is refused.
brief-552 executed five such surfaces as red on `26f5d434`: the immediate
`KernelAdapter::execute` (`crates/rum-jupyter/src/adapter/mod.rs:137-140`,
`run_execute_request` around `:632-637`), the worker path with `allow_stdin`
false, the reply-only `handle_shell_request(Execute)` wrapper, the event pump
and the local socket driver.

When you are done, every Jupyter response that reports input as unavailable
also leaves its own request retired, and the notebook session accepts the next
execute. The asynchronous `allow_stdin == true` path keeps resuming exactly as
now.

## User rulings, 2026-09-30 — implement exactly these

- **U6** — keep both APIs. The immediate `execute` stays immediate: on an input
  request it retires the request and returns the existing terminal error; it
  never blocks for an answer and never grows an implicit continuation. The
  asynchronous dispatch path owns stdin. Document that `allow_stdin` cannot
  supply answers to the immediate API, and document `handle_shell_request`'s
  reply-only contract for `Execute` (it returns `None` for a pending request,
  its caller must poll and service stdin and completion, and it drops the
  initial IOPub prologue); recommend the full dispatch or event-pump APIs for
  real notebook transport. Do not change either return shape.
- **U7** — retire only a request this adapter admitted: take its cancel handle
  before starting it, cancel on `InputRequired`, re-enter once through the
  documented host route. Keep the original input-required reason in the error
  and its metadata; add no synthetic "cancelled" error. A request refused at
  admission owns nothing and must not cancel another request.

Retirement runs guest cleanup. On the worker path, do that retirement through
the worker's continuation mechanism, not by blocking the transport loop.

## The shared design with the next two briefs

brief-557 (draft E) will make the asynchronous execute own its whole lifecycle —
cancel handle, parent metadata and routing — through input waits, interrupt
and overlap; brief-558 (draft F) will make each execute emit every output
exactly once across resumes. Build this brief's retirement so both can build on
it: **one owner for a pending execute's context, and one record of what output
has already been emitted for it.** If this brief needs either, introduce it
minimally and disclose it; do not build E's or F's behaviour here.

## Red tests first

`.temp/brief-552/tests/zz_brief552_jupyter.rs` holds the five reds
(`red_sync_execute_abandons`, `red_worker_without_stdin_abandons`,
`red_shell_wrapper_without_stdin_abandons`, `red_event_pump_without_stdin_abandons`,
`red_local_driver_without_stdin_abandons`). Move them into the crate's own
socket-free test harness (`crates/rum-jupyter/src/tests/`), and add, red first
where they fail today: the immediate API with `allow_stdin` both true and
false; an input asked for inside a nested guest `Get`; cleanup `Print` output
delivered once and the abandoned remainder not run; output produced before the
request delivered once; a request refused at admission leaving the other
request intact. Each asserts reuse by evaluating `1+1` afterwards — the
adapter's `awaiting_stdin` flag is not evidence that no host suspension
remains. Keep the working `allow_stdin == true` worker cases (nested, twice,
password flag) as green controls.

Run them before changing production code; save the failing output under
`.temp/brief-556/red/`.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-552's report §2 (the Jupyter rows and the additional findings), §3 (U6,
  U7) and draft D; `.temp/brief-552/tests/zz_brief552_jupyter.rs`.
- `crates/rum-jupyter/src/adapter/mod.rs` (`execute`, `finish_execute_outcome`
  at `:444-501`, `run_execute_request`), `adapter/worker.rs`, `adapter/render.rs`,
  `transport/`, and the existing harnesses in `src/tests/` (`stdin.rs`,
  `pump.rs`, `adapter/`).
- `crates/rum-host/src/session.rs` (`cancel_handle`, `resume_with_input`, the
  batch rustdoc); `docs/guide/embedding.md:364-395`.
- brief-555's `crates/rum-cli/src/input_retirement.rs` — the same retirement
  done for the CLI; reuse the shape, not the code (different crate).

## Scope

- **Objective:** retirement on every Jupyter surface that returns a terminal
  input-required error; the permanent tests; the rustdoc for `execute` and
  `handle_shell_request`.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventory positions; only fingerprints, hashes and positions may change, and
  any identity, status, disposition or exclusion change is a STOP.
- **Fence:** `rum-jupyter` only. No change to `rum-host`, the kernel,
  `rum-stdlib` source, the wire schema or other front ends; no change to the
  `allow_stdin == true` resume behaviour; no interrupt, overlap or routing
  change (brief-557) and no output-deduplication change (brief-558) beyond what
  retirement itself requires; no new dependency; no `unsafe`. Do not edit
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier briefs or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  notebook-visible change, before and after; any context or output-progress
  state you introduced for E and F to build on. STOP and report anything that
  needs a change outside the fence.

## rum / Rust pitfalls (read twice)

1. **Prove reuse with the next evaluation**, not with an adapter flag.
2. **Take the cancel handle before admission.**
3. **Never answer, reset or replay** to get rid of a suspension.
4. **Native ZeroMQ tests may not bind loopback in your sandbox**; the
   socket-free harnesses are the mandatory coverage.
5. **Stop-loss is keyed on progress**: reds turned green, controls still
   green, compile errors falling. Stop when two consecutive compiling attempts
   move none.

## Verification (MANDATORY — you run it; iterate to green)

Every command through `python3 scripts/run-bounded.py --timeout <seconds> --`,
bounds from measured runs, cargo as `env CARGO_BUILD_JOBS=4
CARGO_NET_OFFLINE=true cargo …`.

```
cargo fmt --check
cargo test -p rum-jupyter
cargo test --workspace
```

Plus the retention replay in the `--no-default-features` build as the template
describes. Paste commands, bounds, exit statuses and counts verbatim; list
native ZeroMQ skips. No microbench.

## Checkpointing — protect the scene

Scratch under `.temp/brief-556/`. After each milestone write
`.temp/brief-556/PROGRESS.md`: done and verified with evidence, the exact next
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

1. What changed — per file with line ranges; notebook-visible changes; any
   shared context or output-progress state introduced.
2. Verification — every command's verbatim result; red before and green after
   for every permanent test.
3. Progress counters, disclosures, and anything unresolved.


---

## Execution report — brief-556

### 1. What changed

Every terminal Jupyter input error now retires its admitted request before
returning. Immediate execution remains immediate for both stdin flags. When
stdin is denied, the asynchronous worker continues into `RetireInput` on the
same thread: cleanup completes before error/idle/reply without blocking transport.
The original input request, prompt, outcome variant, error name and metadata
survive. Refused admission leaves the original request resumable.

| File | Final lines | Why |
| --- | --- | --- |
| `crates/rum-jupyter/src/adapter/mod.rs` | 22; 138–146; 418–424; 653–660 | Register retirement, document both API contracts, capture cancellation before admission and retire only InputRequired. |
| `crates/rum-jupyter/src/adapter/worker.rs` | 12; 54–56; 158–162; 189; 321; 341–343; 353–378 | Carry a clone of the pre-admission handle into the worker and continue denied input into retirement there. |
| `crates/rum-jupyter/src/adapter/input_retirement.rs` | 1–72 | Cancel/re-enter once, verify typed cancellation, preserve prefix and cleanup once, suppress only the synthesized cancellation notice/result. |
| `crates/rum-jupyter/src/tests/mod.rs` | 91 | Register permanent socket-free tests. |
| `crates/rum-jupyter/src/tests/input_abandonment/mod.rs` | 1–351 | Reuse, both immediate flags, nested Get rollback, cleanup/prefix output, original errors/metadata, guest cancel messages, and worker-thread cleanup. |
| `crates/rum-jupyter/src/tests/input_abandonment/controls.rs` | 1–105 | Nested/twice/password stdin, two refused-admission controls, reply-only shell polling. |
| `crates/rum-jupyter/src/tests/input_abandonment/transport.rs` | 1–94 | Event-pump/local-driver reds; reuse through socket-free wire transports. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 2–69, last field only | Copy all 68 replayed fingerprints. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 2–3, columns 5–6 only | Copy four replayed C01/C02 certificate cells; only hashes changed. |
| `docs/dev/codex/brief-556-jupyter-input-abandonment.md` | 160–473 | Append this report; preserve the brief verbatim. |

Notebook-visible before/after: immediate execute, stdin-denied worker, reply-only
shell wrapper, event pump and local driver previously returned InputRequired
while leaving the host blocked; each now returns the same input error and its
next `1+1` evaluates to `2`. Cleanup prints now arrive once before the terminal
error; prefix prints remain once and the abandoned remainder never runs. Guest
cleanup messages remain visible, including an explicit guest `General::cancel`;
no extra synthesized cancellation error/result is delivered. Nested Get cleanup
runs inner to outer (`inner:3`, `outer:2`), rolls state back to marker 1/tail 0,
and clears `$InputFileName`. Definitions outside rollback remain intact.

The `allow_stdin=true` worker still requests and resumes nested, repeated and
password input. The shell wrapper still returns `None` while pending and drops
the initial IOPub prologue; its rustdoc now tells callers to poll and service
stdin/completion and recommends full dispatch/event-pump transport. No return
shape or wire schema changed. Unexpected non-cancellation retirement outcomes
are surfaced as an explicit panic, following the CLI retirement shape, instead
of claiming successful retirement; none occurred in verification.

Shared state: existing `ExecuteContext` remains the sole pending execute context
owner. The worker segment temporarily owns the original suspension and uses a
clone of the existing pre-admission cancellation handle. No persistent output
cursor was introduced: retirement finishes before any suspension output is
emitted, and its cumulative output is assembled once before completion.
Waiting-interrupt, overlap/routing and interactive output replay behavior remain
assigned to briefs 557/558.

Organization assessment: adapter/mod.rs is 23,782 characters (split-plan range).
Retirement has its own 2,864-character module. Splitting lifecycle state here
would broaden this bounded change ahead of brief-557; the future boundary is
execute lifecycle versus non-execute dispatch. Worker.rs is 11,743 characters
(warning range); its lifecycle, segments and test gates remain cohesive during
that transition. A later split can separate worker execution from lifecycle
ownership. New test files are 11,620, 3,543 and 3,640 characters, below the
preferred test threshold.

### 2. Verification

Every verification command below ran through run-bounded with four Cargo jobs,
offline mode and repository-local TMPDIR. Recorder invocations themselves had
an outer bound ten seconds larger. Bounds derive from measured runs: Jupyter
26.353s in brief-552 (110s); this task's red run 7.366s (40s focused candidate);
formatting 1.648s previously (10s); brief-555 fingerprint regeneration 19.648s
(80s), certificates 33.790/35.317s (120s), no-default retention 233.122s (600s),
and workspace 274.778s (650s). These calibrate timeouts, not performance claims.

The following command, result, exit-status and count lines are copied verbatim
from every recorded verification invocation. Full stdout/stderr remains at each
RESULT log path. Both certificate refresh failures are expected: fresh
zero-rejection evidence was written before rejecting the old recorded cell.
After copying the hash-only cells, the full gates passed their replays.

```text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-jupyter input_abandonment -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.22s
test result: FAILED. 6 passed; 14 failed; 0 ignored; 0 measured; 106 filtered out; finished in 0.03s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-jupyter input_abandonment -- --nocapture", "bound_seconds": 110.0, "exit_status": 101, "elapsed_seconds": 7.366, "log": ".temp/brief-556/red/input-abandonment.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-jupyter input_abandonment -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.48s
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 106 filtered out; finished in 0.05s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-jupyter input_abandonment -- --nocapture", "bound_seconds": 40.0, "exit_status": 0, "elapsed_seconds": 5.62, "log": ".temp/brief-556/green/input-abandonment-1.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo fmt -p rum-jupyter
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo fmt -p rum-jupyter", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 0.197, "log": ".temp/brief-556/green/format.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo fmt --check
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo fmt --check", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 1.909, "log": ".temp/brief-556/green/fmt.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.02s
test result: ok. 126 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1", "bound_seconds": 110.0, "exit_status": 0, "elapsed_seconds": 5.561, "log": ".temp/brief-556/green/jupyter.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.52s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture", "bound_seconds": 80.0, "exit_status": 0, "elapsed_seconds": 16.755, "log": ".temp/brief-556/retention/fingerprints.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.43s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 18.76s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 34.322, "log": ".temp/brief-556/retention/default-certificate.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.89s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 19.39s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 35.388, "log": ".temp/brief-556/retention/no-default-certificate.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.74s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 300.43s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 600 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture", "bound_seconds": 600.0, "exit_status": 0, "elapsed_seconds": 315.498, "log": ".temp/brief-556/green/no-default-retention.log"}
```

```text
$ python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.44s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.86s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 126 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.65s
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 263.90s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.31s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.83s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.74s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-556/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture", "bound_seconds": 650.0, "exit_status": 0, "elapsed_seconds": 302.989, "log": ".temp/brief-556/green/workspace.log"}
```

Permanent tests, exact saved result tokens (`tests::input_abandonment::` prefix):

| Test | Before production edits | After |
| --- | --- | --- |
| `cleanup_messages_keep_guest_cancel_and_original_input_reason` | `FAILED` | `ok` |
| `controls::refused_immediate_admission_keeps_original_request` | `ok` | `ok` |
| `controls::refused_worker_admission_keeps_original_request` | `ok` | `ok` |
| `controls::shell_wrapper_is_reply_only_and_caller_services_stdin` | `ok` | `ok` |
| `controls::worker_nested_get_still_resumes` | `ok` | `ok` |
| `controls::worker_password_flag_still_resumes` | `ok` | `ok` |
| `controls::worker_twice_still_resumes` | `ok` | `ok` |
| `immediate_cleanup_and_prefix_are_delivered_once` | `FAILED` | `ok` |
| `immediate_nested_get_abandons_and_rolls_back` | `FAILED` | `ok` |
| `immediate_prefix_output_is_delivered_once` | `FAILED` | `ok` |
| `immediate_without_stdin_abandons` | `FAILED` | `ok` |
| `red_shell_wrapper_without_stdin_abandons` | `FAILED` | `ok` |
| `red_sync_execute_abandons` | `FAILED` | `ok` |
| `red_worker_without_stdin_abandons` | `FAILED` | `ok` |
| `transport::red_event_pump_without_stdin_abandons` | `FAILED` | `ok` |
| `transport::red_local_driver_without_stdin_abandons` | `FAILED` | `ok` |
| `worker_cleanup_and_prefix_are_delivered_once` | `FAILED` | `ok` |
| `worker_nested_get_abandons_and_rolls_back` | `FAILED` | `ok` |
| `worker_prefix_output_is_delivered_once` | `FAILED` | `ok` |
| `worker_retirement_runs_cleanup_off_transport_thread` | `FAILED` | `ok` |

All twenty ran before production edits; no test-contract corrections followed
the red baseline. The five original reds were promoted from the preserved
brief-552 scratch tests. Every retirement test proves reuse with `1+1`; the
admission controls first resume the original continuation with its real answer.

Native ZeroMQ skips from the sequential Jupyter gate, verbatim:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Workspace aggregate across reported binaries and doctests: {'passed': 2831, 'failed': 0, 'ignored': 58, 'measured': 0, 'filtered': 0}. The nine native cases return `ok` after explicitly skipping loopback and are included in Cargo pass counts. Socket-free tests executed. The ignored fingerprint writer was run explicitly above.

The workspace left 58 existing tests ignored by their declared attributes: release-only profiling/stack/depth probes, opt-in native-stack guards, heavier stress tests, two stability tests, and the fingerprint writer. The writer ran explicitly above; the remaining 57 did not run. Exact names/reasons are saved in `.temp/brief-556/ignored-tests.txt`.

Standing maintenance: all 68 local-contract rows changed only their fingerprint
field (`retention/contract-changes.json` records old/new values). C01 and C02
each changed both build cells (`retention/certificate-changes.json`, four cells).
Normalizing only fnv1a64 values proves other certificate text byte-identical.
Identities, 62 Discharged/six Pending statuses, dispositions, preconditions,
strengths, exclusions and four UnresolvedConsumer rows remain unchanged. Each
build retains two body rows, three production instance marks, four probe marks,
zero fixture marks and zero helper-body marks.

Changed contract row IDs in file order:

```text
B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, I01, I02, I03, I04, P01, P02, P03, P04, P05, P06, P070, P080, P090, P100, P110, P120, P130, P140, P150, P071, P081, P091, P101, P111, P121, P131, P141, P151, B13, R01, R02, R03, R040, R050, R060, R070, R080, R090, R100, R110, R041, R051, R061, R071, R081, R091, R101, R111, R12, R13, R14, R15, O010, O0200, O011, O0210
```

Ownership inventory reconciliation, verbatim:

```text
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
```

No ownership positions changed: discovery roots are core, runtime, stdlib and
syntax; Jupyter is downstream. All seven ownership and six other retention
inventories are byte-identical (`retention/unchanged-inventories.json`).

### 3. Progress counters, disclosures, and unresolved work

- Original reds: **5/5 green**; additional reds: **9/9 green**.
- Controls: **6/6 green**, including all three requested interactive controls.
- Candidate compiler errors: **0**. The retention falsifiers intentionally
  compile rejected mutations (including 84 located E0308 diagnostics). One compiling production attempt turned all fourteen
  reds green; stagnant implementation attempts: **0**.
- Formatting, Jupyter, workspace and no-default retention gates: **green**.
  Both certificate builds replay successfully in the full gates.
- No remaining work within this brief, half-written source or out-of-fence API
  change. Git remains entirely with the director.

Not run: microbenchmarks (explicitly excluded), actual native loopback exchanges
(the nine skips listed above), or other-platform tests. No git command, network
fetch, dependency change, new unsafe, configuration write or edit to another
front end/host/kernel/stdlib implementation occurred. The only stdlib edits are
the two standing-maintenance inventories. Baseline hashes preserve the existing
worktree independently of git; the final source hash audit confirms the Jupyter
source verified by the gates remained unchanged afterward.

Scratch, full logs, red/green evidence, source hashes, row-by-row maintenance
changes and checkpoints are under `.temp/brief-556/`. Existing maintenance tools
also wrote their prescribed review artifacts under `target/` and
`.temp/brief-546/certificates/`. One scratch report-generator construction had a
Python quoting error and was corrected before any report append; it made no
source change. The final scope audit has six baseline files changed, four new
Jupyter files, no deletions and 1,710 unchanged baseline files.

Exact next action: the director reviews the completed tree and runs the
authoritative gate.
