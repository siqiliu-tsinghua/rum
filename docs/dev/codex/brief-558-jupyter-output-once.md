# brief-558 — Jupyter: each output of a resumed execute is emitted exactly once

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 3F, the last front-end step of the suspension-continuation branch
(brief-552, draft F, rewritten). When an asynchronous execute suspends for
input and resumes, `rum-jupyter` re-renders the host's whole buffered output
at every segment. brief-552 executed it on `26f5d434`: a source that prints
`prefix`, asks for input, prints `middle`, asks again and finishes, shows
`prefix`, `prefix`, `middle`, `prefix`, `middle` on the notebook's streams,
while the same request with queued answers shows `prefix`, `middle`. The host's
final print list is correct; the adapter replays it (`adapter/render.rs:53-111`,
`adapter/mod.rs:444-469,506-529`).

When you are done, the IOPub effects of a resumed execute — streams, messages,
and anything else the adapter renders — equal the effects of the same request
with its answers queued, **in order and in multiplicity**. Output produced
before an input request is still visible before the prompt; each later segment
emits only what is new.

## User ruling, 2026-09-30

**U9** — each output appears exactly once. Track what has already been emitted
**by position**, not by content: a source that prints the same string twice has
two outputs, and both must appear.

## Design

- brief-557 (`7c4d90b0`) gave the admitted asynchronous execute one owned
  lifecycle. Keep the emission progress **in that owner**, for the life of the
  request, and discard it only when the request is retired.
- **Measure before choosing cursors.** Through `rum-host` only, establish which
  collections are cumulative across suspension, success, failure and
  cancellation — prints, messages, trace, anything else rendered — and whether
  each is append-only across every boundary. A collection that is not
  append-only across some boundary gets its actual contract documented and
  tested, not a cursor that silently drops entries.
- Keep one final result, the original parent and the execution count.
- brief-556's retirement already assembles cleanup output once for the
  terminal-error paths; do not regress it.

## Red tests first

`.temp/brief-552/tests/zz_brief552_jupyter.rs` holds
`red_worker_streams_match_queued_input`. Move it into the crate's socket-free
harness (`crates/rum-jupyter/src/tests/`) keeping its exact comparison with the
queued path, and add, red first where they fail today: one prompt, two prompts,
an input inside a nested guest `Get`; the same string printed twice before and
after a prompt (both occurrences survive); messages emitted before and after a
prompt; a failure after a resume; a cancellation by interrupt after a resume
(brief-557's path). Each compares the complete, ordered IOPub stream and output
effects with the queued-input execution of the same source.

Run them before changing production code; save the failing output under
`.temp/brief-558/red/`.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-552's report §2 (the duplicated-stream finding), §3 (U9) and draft F;
  brief-556's and brief-557's reports (retirement, the owned lifecycle).
- `crates/rum-jupyter/src/adapter/mod.rs`, `adapter/execute.rs` (brief-557's
  lifecycle), `adapter/render.rs`, `adapter/worker.rs`, and the harnesses in
  `src/tests/`.
- `crates/rum-host/src/session.rs` — the outcome types and what they carry.

## Scope

- **Objective:** per-request emission progress in the owned execute; rendering
  only unseen outputs at each segment; the permanent tests; the adapter rustdoc
  stating the once-only delivery.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventory positions; only fingerprints, hashes and positions may change, and
  any identity, status, disposition or exclusion change is a STOP.
- **Fence:** `rum-jupyter` only. No change to `rum-host`'s buffering contract,
  the kernel, `rum-stdlib` source, the wire schema, other front ends or
  evaluation behaviour; no new dependency; no `unsafe`; no default-path cost.
  Do not loosen an existing test or helper to make a new test pass — if one
  conflicts, STOP and report. Do not edit `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, earlier briefs or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; which
  output collections you found cumulative or not; the emission state you
  added; every notebook-visible change, before and after. STOP and report
  anything that needs a change outside the fence.

## rum / Rust pitfalls (read twice)

1. **A correct final value is not a correct notebook.** Compare the streams.
2. **Content deduplication is wrong**: equal prints are distinct outputs.
3. **Do not change the host's buffering** to make the adapter simpler.
4. **Stop-loss is keyed on progress**: parity cases passing per output family,
   controls still green, compile errors falling. Stop when two consecutive
   compiling attempts move none.

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

Scratch under `.temp/brief-558/`. After each milestone write
`.temp/brief-558/PROGRESS.md`: done and verified with evidence, the exact next
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

1. What changed — per file with line ranges; the output-collection findings;
   the emission state; notebook-visible changes.
2. Verification — every command's verbatim result; red before and green after
   for every permanent test.
3. Progress counters, disclosures, and anything unresolved.
