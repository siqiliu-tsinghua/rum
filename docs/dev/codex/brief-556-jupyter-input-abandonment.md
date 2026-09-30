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
