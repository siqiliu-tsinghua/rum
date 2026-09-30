# brief-557 — Jupyter: a waiting execute owns its lifecycle through interrupt and overlap

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 3E of the suspension-continuation branch (brief-552, draft E, rewritten).
brief-556 (`1e598aa3`) made every Jupyter surface that cannot take input retire
its request. What remains is the asynchronous execute that **is** waiting for
input, where brief-552 executed three defects on `26f5d434`:

1. **Interrupt while waiting.** `interrupt_request` reports ok, but
   `awaiting_stdin` stays true and the next execute is rejected; a later
   `input_reply` still resumes the old work; the lifecycle stays
   `InterruptRequested`. The idle `ExecuteState` has no worker to cancel
   (`crates/rum-jupyter/src/adapter/mod.rs:183-190`, `adapter/worker.rs:67-74`)
   — the suspension lives in the host session, not in a running thread.
2. **Overlap while waiting.** A second execute is correctly rejected, but the
   rejection marks the adapter Idle while the first is still waiting
   (`finish_execute_request`, `adapter/mod.rs:528-529`).
3. **Overlap through the event pump.** The first execute's eventual reply
   keeps its parent header but loses its routing ids: the second dispatch
   overwrites the pending routing and its rejected completion clears it
   (`transport/event_loop.rs:54-68,94-108`).

When you are done, an admitted execute owns its context, cancel handle, parent
metadata and routing from admission to its one terminal reply, through every
input wait; an interrupt retires that same request; and a rejected overlapping
execute cannot touch any of it.

## User ruling, 2026-09-30 — implement exactly this

**U8** — an interrupt while an execute is waiting for input finishes the
**original** execute: its `input_request` is withdrawn (a late `input_reply` is
rejected and resumes nothing), the request is cancelled and re-entered once
through the documented host route so guest cleanup runs once and the abandoned
remainder never runs, and the original execute then gets its terminal reply
with a Cancelled error on its original parent and route, followed by one idle
status. The session then accepts a new execute. An overlapping execute stays
rejected (brief-411's rule is settled); its rejection must not publish idle for
the still-pending original, must not replace its context, and must not erase
its route. Shutdown is out of scope (U10: disposal, documented, no cleanup
promise).

## Design

- **One owned lifecycle** for an admitted asynchronous execute — running,
  waiting for input, retiring — holding the cancel handle taken before
  admission, the original parent header and execution count, and its routing
  identity. brief-556 kept `ExecuteContext` as the sole pending owner; build on
  it rather than adding a second one.
- **Interrupt while waiting** does its retirement where the evaluation runs —
  on the worker, not the transport loop — because cleanup is guest code.
- **Routing belongs to the admitted execute's identity**, not to "whatever was
  dispatched last". A rejected request gets its own error route.
- Reject an overlap **before** moving a session or touching context.
- Use the existing deterministic worker gates in the test harness for races;
  no sleeps, no long guest computations.

## Red tests first

`.temp/brief-552/tests/zz_brief552_jupyter.rs` holds
`red_interrupt_while_waiting_finishes_request`,
`red_overlap_rejection_preserves_busy_state` and
`red_overlap_reply_keeps_original_routing`. Move them into the crate's
socket-free harness (`crates/rum-jupyter/src/tests/`) and strengthen the first
to assert: exactly one execute reply, with a Cancelled error, on the original
parent and route; one final idle; no pending input request or context; cleanup
once; no tail; a late or stale `input_reply` rejected; a following execute of
`1+1` returning `2`. Add, red first where they fail today: an interrupt at the
second of two input prompts; an overlap while the first execute is running (not
waiting) and while it is waiting; a mismatched-parent reply still rejected
(already correct — a control). Keep brief-556's tests and the resuming
`allow_stdin` controls green.

Run them before changing production code; save the failing output under
`.temp/brief-557/red/`.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-552's report §2 "Additional executed Jupyter findings", §3 (U8) and
  draft E; brief-556's report (what it built, `input_retirement.rs`).
- `crates/rum-jupyter/src/adapter/mod.rs` (`:174-190` lifecycle and
  interrupt, `:302-347` input replies, `:415-541` dispatch and completion),
  `adapter/worker.rs`, `transport/event_loop.rs` (`:54-108`, `:140-172`), and
  the harnesses in `src/tests/`.

## Scope

- **Objective:** the owned execute lifecycle, interrupt-while-waiting
  retirement, overlap handling that preserves the original's state and route,
  the permanent tests, and the adapter rustdoc for these transitions.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventory positions; only fingerprints, hashes and positions may change, and
  any identity, status, disposition or exclusion change is a STOP.
- **Fence:** `rum-jupyter` only. No change to `rum-host`, the kernel,
  `rum-stdlib` source, the wire schema, other front ends, the overlap-rejection
  rule, shutdown semantics, or output emission (brief-558 makes each output
  appear once — do not fix duplication here, and do not make it worse); no new
  dependency; no `unsafe`. Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  state transition added or changed; every context field; every notebook-
  visible change, before and after. STOP and report anything that needs a
  change outside the fence.

## rum / Rust pitfalls (read twice)

1. **An idle worker is not an idle evaluation**: the suspension lives in the
   host session. Clearing the pending input request does not retire it.
2. **A successful `interrupt_reply` is not proof of cancellation**; prove it
   with the original's Cancelled reply and a following `1+1`.
3. **Never cancel on an overlap rejection**, and never reset the notebook's
   definitions to recover.
4. **Parent headers alone do not identify a route**; two executes can share a
   parent shape.
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

Scratch under `.temp/brief-557/`. After each milestone write
`.temp/brief-557/PROGRESS.md`: done and verified with evidence, the exact next
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

1. What changed — per file with line ranges; state transitions and context
   fields; notebook-visible changes.
2. Verification — every command's verbatim result; red before and green after
   for every permanent test.
3. Progress counters, disclosures, and anything unresolved.

---

## Execution report — brief-557

### 1. What changed

An admitted asynchronous execute now retains one ExecuteContext from admission
through every input wait and retirement to its terminal reply. Interrupting a
wait withdraws stdin immediately, cancels that context's request generation,
and re-enters the host once on a worker. Guest cleanup runs once, the abandoned
tail never runs, and the original parent/count/route receive Cancelled plus
one final idle. Late and stale replies cannot resume the retired continuation;
the next admitted 1+1 returns 2 without resetting notebook definitions.

| File | Final lines | Why |
| --- | --- | --- |
| crates/rum-jupyter/src/adapter/mod.rs | 22; 42–49; 178–186; 303–363; 429–435; moved execute methods | Register execute module, document lifecycle/disposal, withdraw stdin on interrupt, carry routing and preserve Busy during other replies. |
| crates/rum-jupyter/src/adapter/execute.rs | 1–279 | Own admission, overlap refusal, suspension/completion, cancellation races and existing output assembly. |
| crates/rum-jupyter/src/adapter/worker.rs | 17–41; 61–65; 76–137; 174–193; 222–227; 369–384 | Keep cancel/route/interruption in context, unify segment spawning, cancel/re-enter live input once on a worker. |
| crates/rum-jupyter/src/transport/event_loop.rs | 1–3; 12–34; 49–93 | Replace independent pending route with the admitted context route; rejected work uses its own route. |
| crates/rum-jupyter/src/tests/mod.rs | 91 | Register the permanent lifecycle harness. |
| crates/rum-jupyter/src/tests/execute_lifecycle/mod.rs | 1–146 | Socket-free helpers, mismatched-parent control and Busy preservation. |
| crates/rum-jupyter/src/tests/execute_lifecycle/interrupt.rs | 1–276 | First/second prompts, publication/retirement races, cleanup/route/count/idle/reuse/stale-answer proofs, and denied-input race. |
| crates/rum-jupyter/src/tests/execute_lifecycle/overlap.rs | 1–148 | Direct/dispatch overlap while running/waiting; routes with different or identical parents. |
| crates/rum-jupyter/src/tests/input_abandonment/mod.rs | 20–28 | Accept the now-immediate refusal in the existing helper; retain every assertion. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | 2–69, final field only | Install 68 replayed fingerprints. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | 2–3, columns 5–6 only | Install four hash-only C01/C02 cells from both builds. |
| docs/dev/codex/brief-557-jupyter-execute-lifecycle.md | 169–610 | Append this report; preserve the original brief. |

ExecuteContext fields (all disclosed):

| Field | Ownership and purpose |
| --- | --- |
| parent_header | Existing original execute parent; moved unchanged through worker/wait/retirement. |
| request | Existing original request/code/stdin/history policy. |
| execution_count | Existing admitted count; overlapping requests allocate no count. |
| cancel | Added to context, moved from worker-only ownership; captured before admission and cloned only into that request's worker segments. |
| routing_ids | Added original wire route; event pump's independent pending-route slot is removed. |
| interrupted | Moved from worker into context, preserving the admitted interruption through retirement. |

ExecuteContext is no longer Clone. ExecuteWorker retains its thread handle,
context and existing test signals; it no longer separately owns cancel or
interrupted. ExecuteState still describes worker Idle/Running, while
pending_execute owns the same context during an input wait. There is no second
pending context and no output cursor.

State transitions and notebook-visible changes:

- Idle -> running: refuse overlap before moving the session, reserving a
  cancellation handle, incrementing history, or emitting an execute prologue.
  Accepted work alone owns Busy and its context.
- Running -> waiting: preserve the same context, handle and route while
  advertising the current input_request; execution stays Busy.
- Waiting -> running on valid input: move that context into the resume worker.
  Mismatched parents leave the current prompt and context untouched.
- Waiting -> retiring on interrupt: withdraw the prompt, set InterruptRequested
  and cancel the admitted handle; start CancelInput on a worker. Repeated
  interrupts and overlaps during retirement cannot start another retirement.
- Running -> retiring when interrupt races with a finished suspension before
  completion polling: suppress the never-published prompt and retire on a
  worker. InputRequired from stdin-denied work is already retired by brief-556;
  it must retain that terminal reason instead of being retired again.
- Terminal completion releases the context and publishes one idle. The admitted
  interrupted lifecycle returns to Running unless shutdown has taken ownership.
  This does not add a shutdown cleanup promise.
- Overlap previously panicked while a worker owned the session, or ran a
  rejected worker while waiting and falsely published idle. Immediate and
  dispatched overlap now return Other without changing the original. A
  dispatch rejection is immediate (including the reply-only shell helper),
  retains its own error IOPub and shell route, emits no execute prologue/idle,
  has no execution count, and uses the message
  "cannot start a new evaluation while an execute request is pending".
- Other replies while an admitted execute waits no longer cycle Busy/Idle.
  Their reply still reports the request response. Idle-session status cycles
  and shutdown's disposal semantics remain unchanged.
- Routing now travels with the admitted context. Rejected work uses its own
  local route, even with an identical parent header; it cannot overwrite or
  clear the original's route.

Output rendering, wire schemas, public return shapes, dependencies and
host/kernel implementations are unchanged. Cumulative output replay remains
assigned to brief-558; no output deduplication is claimed here. The cancellation
tests prove cleanup output once and absence of tail output, and separately
prove worker-thread cleanup with HostSession's streaming sink.

Organization: execute transitions now occupy adapter/execute.rs, a real
boundary extracted from adapter/mod.rs. The moved outcome/output assembly is
unchanged. Final sizes: adapter/mod.rs 19,316 characters (down from 23,782), execute.rs 9,692, worker.rs 12,360. The facade and worker remain in the warning range; non-execute services and worker outcome normalization are future boundaries. This change isolates execute lifecycle without unrelated churn. New test files remain below 15k characters. Moved finish_execute_outcome and finish_execute_request are byte-identical to baseline; rendering files are unchanged. Red-run/final-run test names match after adding the one review regression (139 names).

### 2. Verification

All verification commands below used run-bounded, four Cargo jobs, offline mode
and repository-local TMPDIR. The recorder was bounded ten seconds longer.
Bounds derive from measured runs: previous Jupyter 5.561s and this red 6.771s
(110s initially, then 40s); fmt 1.909s (10s); fingerprints 16.755s (80s);
certificates 34.322/35.388s (120s); no-default retention previously about 315s
and this task's first replay 274.222s (650s); workspace previously about 275s
(650s). These calibrate timeouts, not performance.

The initial red run compiled: all three promoted reds and eight added reds
failed; the mismatched-parent control and 126 pre-existing tests passed.
The first implementation turned nine reds green. Two fixture failures mixed
streaming print sinks with buffered output expectations, contrary to HostSession
rustdoc (session.rs:266). The fixture now runs both modes independently,
preserving the IOPub-once and worker-thread assertions. All initial reds passed.

Review exposed an additional double-retirement race: stdin-denied work retains
InputRequired after host retirement. A new deterministic test ran red before
correcting that branch; it caught Panic instead of InputRequired. The final
correction preserves cleanup, definitions and the original input error. This
source change required another fingerprint/certificate replay; both rounds are
recorded below.

The four exit-101 certificate-writer runs each earned fresh zero-rejection marks
and then rejected the old recorded cell. Replayed hash cells were installed
afterward. Final no-default and workspace gates pass reconciliation; these
intermediate stale-certificate failures are resolved.

Every recorded verification command, result, exit status and count is copied
verbatim below. Each RESULT log path contains full stdout/stderr.

~~~text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.30s
test result: FAILED. 127 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1", "bound_seconds": 110.0, "exit_status": 101, "elapsed_seconds": 6.771, "log": ".temp/brief-557/red/jupyter.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.67s
test result: FAILED. 136 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 6.122, "log": ".temp/brief-557/green/jupyter-attempt-1.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-jupyter execute_lifecycle -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.51s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 126 filtered out; finished in 0.05s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-jupyter execute_lifecycle -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 0, "elapsed_seconds": 4.668, "log": ".temp/brief-557/green/lifecycle-2.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt -p rum-jupyter
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt -p rum-jupyter", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 0.199, "log": ".temp/brief-557/green/format.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt --check
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt --check", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 1.75, "log": ".temp/brief-557/green/fmt.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.95s
test result: ok. 138 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 0, "elapsed_seconds": 5.466, "log": ".temp/brief-557/green/jupyter.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.10s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture", "bound_seconds": 80.0, "exit_status": 0, "elapsed_seconds": 16.305, "log": ".temp/brief-557/retention/fingerprints.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.62s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 19.01s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 33.769, "log": ".temp/brief-557/retention/default-certificate.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.91s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 19.29s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 35.326, "log": ".temp/brief-557/retention/no-default-certificate.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.38s
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
expired-C01: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=false
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
expired-C02: RED stale or omitted instance certificate C02 hash=false
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 259.61s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture", "bound_seconds": 650.0, "exit_status": 0, "elapsed_seconds": 274.222, "log": ".temp/brief-557/retention/no-default.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-jupyter interrupt_after_denied_input_retirement_does_not_retire_again -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.41s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 138 filtered out; finished in 0.01s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-jupyter interrupt_after_denied_input_retirement_does_not_retire_again -- --nocapture", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 6.519, "log": ".temp/brief-557/red/denied-retirement.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt -p rum-jupyter
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt -p rum-jupyter", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 0.202, "log": ".temp/brief-557/green/format-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.49s
test result: ok. 139 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 0, "elapsed_seconds": 5.059, "log": ".temp/brief-557/green/jupyter-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt --check
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo fmt --check", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 1.703, "log": ".temp/brief-557/green/fmt-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.39s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.12s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture", "bound_seconds": 80.0, "exit_status": 0, "elapsed_seconds": 15.608, "log": ".temp/brief-557/retention/fingerprints-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.69s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 18.35s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 33.166, "log": ".temp/brief-557/retention/default-certificate-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.59s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.86s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 33.573, "log": ".temp/brief-557/retention/no-default-certificate-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.74s
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
expired-C01: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=false
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=false
expired-C02: RED stale or omitted instance certificate C02 hash=false
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 260.82s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture", "bound_seconds": 650.0, "exit_status": 0, "elapsed_seconds": 274.832, "log": ".temp/brief-557/retention/no-default-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.77s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.96s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 139 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.07s
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
expired-C01: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-0: RED stale or omitted instance certificate C01 hash=true
omitted-C01-instance-1: RED stale or omitted instance certificate C01 hash=true
expired-C02: RED stale or omitted instance certificate C02 hash=true
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 264.66s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.33s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.17s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.79s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-557/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture", "bound_seconds": 650.0, "exit_status": 0, "elapsed_seconds": 305.032, "log": ".temp/brief-557/green/workspace.log"}
~~~

Permanent tests, red before and green after:

| Test | Before | Final |
| --- | --- | --- |
| tests::execute_lifecycle::interrupt::interrupt_after_denied_input_retirement_does_not_retire_again | FAILED (review red) | ok |
| tests::execute_lifecycle::interrupt::interrupt_after_suspension_before_completion_poll_retires_on_worker | FAILED | ok |
| tests::execute_lifecycle::interrupt::interrupt_at_second_prompt_finishes_original_request | FAILED | ok |
| tests::execute_lifecycle::interrupt::red_interrupt_while_waiting_finishes_request | FAILED | ok |
| tests::execute_lifecycle::mismatched_parent_preserves_waiting_execute | ok (control) | ok |
| tests::execute_lifecycle::non_execute_reply_during_input_wait_preserves_busy | FAILED | ok |
| tests::execute_lifecycle::overlap::immediate_overlap_while_running_is_rejected | FAILED | ok |
| tests::execute_lifecycle::overlap::immediate_overlap_while_waiting_preserves_count_and_context | FAILED | ok |
| tests::execute_lifecycle::overlap::overlap_while_running_is_rejected_before_session_move | FAILED | ok |
| tests::execute_lifecycle::overlap::overlap_with_identical_parent_keeps_original_routing | FAILED | ok |
| tests::execute_lifecycle::overlap::red_overlap_rejection_preserves_busy_state | FAILED | ok |
| tests::execute_lifecycle::overlap::red_overlap_reply_keeps_original_routing | FAILED | ok |
| tests::execute_lifecycle::overlap::wire_overlap_while_running_keeps_original_routing | FAILED | ok |

Final test lines, verbatim:

~~~text
test tests::execute_lifecycle::interrupt::interrupt_after_denied_input_retirement_does_not_retire_again ... ok
test tests::execute_lifecycle::interrupt::interrupt_after_suspension_before_completion_poll_retires_on_worker ... ok
test tests::execute_lifecycle::interrupt::interrupt_at_second_prompt_finishes_original_request ... ok
test tests::execute_lifecycle::interrupt::red_interrupt_while_waiting_finishes_request ... ok
test tests::execute_lifecycle::mismatched_parent_preserves_waiting_execute ... ok
test tests::execute_lifecycle::non_execute_reply_during_input_wait_preserves_busy ... ok
test tests::execute_lifecycle::overlap::immediate_overlap_while_running_is_rejected ... ok
test tests::execute_lifecycle::overlap::immediate_overlap_while_waiting_preserves_count_and_context ... ok
test tests::execute_lifecycle::overlap::overlap_while_running_is_rejected_before_session_move ... ok
test tests::execute_lifecycle::overlap::overlap_with_identical_parent_keeps_original_routing ... ok
test tests::execute_lifecycle::overlap::red_overlap_rejection_preserves_busy_state ... ok
test tests::execute_lifecycle::overlap::red_overlap_reply_keeps_original_routing ... ok
test tests::execute_lifecycle::overlap::wire_overlap_while_running_keeps_original_routing ... ok
~~~

Native ZeroMQ skips from the final sequential Jupyter gate, verbatim:

~~~text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
~~~

Workspace aggregate across binaries/doctests: {'passed': 2844, 'failed': 0, 'ignored': 58, 'measured': 0, 'filtered': 0}. Cargo's pass count includes the nine native skips; socket-free cases executed.

The workspace reports 58 ignored tests. The fingerprint writer ran explicitly; the other 57 ignored tests did not run. Exact recorded names/reasons are in .temp/brief-557/ignored-tests.txt.

Ownership discovery reconciliation, verbatim:

~~~text
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
~~~

Standing maintenance changed exactly 68 contract fingerprints and four
certificate hash cells. Normalizing only fnv1a64 values leaves both inventories
byte-identical to baseline. Identities, statuses, dispositions, preconditions,
strengths and exclusions did not change. C01/C02 retain two body rows and three
instances per build; all four LeaseProbe538 marks remain clean. The 62
Discharged/six Pending contracts and four UnresolvedConsumer rows retain their
statuses. No ownership positions changed: discovery roots exclude Jupyter.
Seven ownership TSVs, six other retention TSVs and the prelude-method policy
text are byte-identical. Every changed value and baseline comparisons are in
retention/contract-changes.json, retention/certificate-baseline-changes.json
and retention/unchanged-inventories.json.

Changed contract row IDs, in file order:

~~~text
B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, I01, I02, I03, I04, P01, P02, P03, P04, P05, P06, P070, P080, P090, P100, P110, P120, P130, P140, P150, P071, P081, P091, P101, P111, P121, P131, P141, P151, B13, R01, R02, R03, R040, R050, R060, R070, R080, R090, R100, R110, R041, R051, R061, R071, R081, R091, R101, R111, R12, R13, R14, R15, O010, O0200, O011, O0210
~~~

### 3. Progress counters, disclosures, and unresolved work

- Original reds: 3/3 green; additional initial reds: 8/8 green; review race:
  1/1 green; mismatched-parent control: 1/1 green.
- All 126 pre-existing Jupyter cases remain green (nine explicitly skip).
  Final Jupyter: 139. Candidate compiler errors: zero. Expected compiler
  falsifier diagnostics inside retention tests are not build failures.
- No two consecutive stagnant compiling repair attempts. All required final
  gates passed. Source hashes were frozen after final Jupyter verification.
- No unresolved in-scope work, half-edited source, changed inventory status or
  out-of-fence API change. Two rejected JavaScript tool invocations executed no
  command and changed no file; corrected before successful edits.

Not run: microbenchmarks (excluded by the brief), actual native loopback
exchanges (nine explicit skips), remaining ignored tests, or other-platform
tests. No git command, network fetch, configuration write, dependency change,
new unsafe, or host/kernel/other-front-end implementation edit occurred.
Only the two maintenance inventories changed in stdlib. The first action read
the brief; all subsequent shell commands were bounded. No external-vendor
testing or outputs were used.

Scratch, full logs, checkpoints, baseline copies, hashes and review diffs are
under .temp/brief-557/. Required maintenance tools also wrote their prescribed
artifacts under target/ and .temp/brief-546/certificates/. The final scope audit
is at .temp/brief-557/scope-audit.json. The original brief is preserved
byte-for-byte before this appended report.

Exact next action: the director reviews the completed tree and runs the
authoritative gate. All git operations remain with the director.
