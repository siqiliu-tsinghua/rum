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
