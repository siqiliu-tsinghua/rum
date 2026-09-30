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

1. **What changed — stopped at the scope fence.**

The mandatory host-buffer measurement exposed an ordering conflict before production implementation. The brief requires both complete ordered IOPub parity with queued input and visibility of all prefix output before the prompt. Current host outcomes do not carry the cross-family chronology needed to satisfy both generally.

Concrete source:

```rum
notice::before="before"; Message[notice::before];
InputString["prompt: "]; Print["tail"]; 42
```

Queued execution renders stdout `tail` followed by stderr `notice::before`. Resumed execution has already published that stderr message before the prompt, so subsequent stdout necessarily follows it. Removing the replayed stderr by its exact position still fails the complete IOPub comparison. A separate passing control shows that reversing the source's print/message order produces an identical queued `HostEvalSuccess`, including `outputs`.

A chronological host-output contract would cross the fence. Weakening parity to comparison within each output family would change the brief's requirement. Neither was done.

Changed repository file: `docs/dev/codex/brief-558-jupyter-output-once.md`, lines 142–202, appends this report only. All 77 saved source/inventory/brief files were unchanged before this append; no new Jupyter source files exist. No emission state was added, and notebook behavior is unchanged.

Scratch evidence under `.temp/brief-558/`: `host-probe/main.rs:1–37` measures host buffers; `host-probe/output_order.rs:1–85` contains the socket-free order falsifiers; `host-probe/Cargo.toml:1–15` builds these probes against existing repository crates; `run.py:1–24` records bounded commands; `PROGRESS.md:1–17` records the STOP and next action. Raw logs, command records, before-images and `scope-audit.json` remain there. No production dependency was added.

Collection findings are limited to the measured single-source requests: prints, messages and trace preserve their prefixes across both input waits and success, `Exit[7]`, and cancellation after resume. The final buffers equal the queued counterparts. The combined `outputs` collection is rebuilt in print/message/trace/result order and is **not append-only** across these boundaries. No cursor was selected from an unverified assumption.

2. **Verification — verbatim commands and results.**

Both recorder invocations had an outer 120-second bound. Their inner 110-second bounds used the prior measured Jupyter build/run envelope (26.353 seconds). Cargo ran offline with four jobs and repository-local temporary storage.

```text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo run --manifest-path .temp/brief-558/host-probe/Cargo.toml --target-dir target
BOUNDARY 42/suspend-suspend: prints 1->2 prefix=true messages 1->2 prefix=true trace 5->8 prefix=true outputs 7->12 prefix=false
BOUNDARY 42/suspend-terminal: prints 2->4 prefix=true messages 2->3 prefix=true trace 8->15 prefix=true outputs 12->23 prefix=false
QUEUED_EQ 42: prints=true messages=true trace=true outputs=true
BOUNDARY Exit[7]/suspend-suspend: prints 1->2 prefix=true messages 1->2 prefix=true trace 5->8 prefix=true outputs 7->12 prefix=false
BOUNDARY Exit[7]/suspend-terminal: prints 2->4 prefix=true messages 2->3 prefix=true trace 8->12 prefix=true outputs 12->19 prefix=false
QUEUED_EQ Exit[7]: prints=true messages=true trace=true outputs=true
BOUNDARY cancel/suspend-suspend: prints 1->2 prefix=true messages 1->2 prefix=true trace 5->8 prefix=true outputs 7->12 prefix=false
BOUNDARY cancel/suspend-terminal: prints 2->3 prefix=true messages 2->3 prefix=true trace 8->9 prefix=true outputs 12->16 prefix=false
QUEUED_EQ cancel: prints=true messages=true trace=true outputs=true
```

Exit **0**, elapsed **22.334s**; full output: `.temp/brief-558/measure/host-buffers.log`.

```text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test --manifest-path .temp/brief-558/host-probe/Cargo.toml --target-dir target --test output-order -- --nocapture --test-threads=1
test red_complete_iopub_order_matches_queued_input ... BEFORE_PROMPT [Stream(KernelStreamOutput { name: Stderr, text: "notice::before: before\n" })]
failures:
    red_complete_iopub_order_matches_queued_input
    red_order_still_differs_after_positional_replay_removal

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Exit **101**, elapsed **23.497s**; full ordered payloads and assertion output: `.temp/brief-558/red/output-order.log`. The second red removes only the known replay position in the probe; production rendering is untouched.

**Did not run:** `cargo fmt --check`, `cargo test -p rum-jupyter`, `cargo test --workspace`, retention/certificate replay in either feature build, ownership discovery, or native ZeroMQ tests. No native skip was observed. The hard scope STOP was reached during measurement, before moving the archived red into the permanent harness or adding the requested permanent matrix. There is no green-after result and no claim of completed implementation. No microbench ran.

3. **Counters, disclosures and unresolved work.**

Production implementation attempts: **0**. Probe compiler errors: **0**. Measured host boundaries: **6**. Passing host information-loss control: **1**. Full IOPub parity reds: **2**. Lifecycle controls: **not run**. No-progress implementation attempts: **0**. This is the explicit scope STOP, not stop-loss exhaustion.

No git commands, network access, production-source edits, inventory changes, test weakening, or new unsafe code. Retention fingerprints were not expired by these scratch-only probes; their replay was not performed.

The director must settle the contract/fence conflict before implementation: authorize upstream chronological output delivery, or revise parity to specify ordering within each family and the treatment of trace bundles. The exact next action and unchanged tree state are recorded in `.temp/brief-558/PROGRESS.md`.

---

## Resume — the director revises the parity requirement

You stopped correctly, and the conflict is in the brief, not in the code: the
host's outcomes carry **no cross-family chronology** at all — your control
showed that swapping a `Print` and a `Message` in the source yields an identical
queued `HostEvalSuccess`. Demanding cross-family order equal to the queued path
would demand that the resumed path be made *less* faithful than it is. This
brief's problem is duplication; chronology across families is a separate,
pre-existing property of the host contract that exists without any suspension.
It is not in this brief's scope and must not be changed here.

**Revised requirement — implement exactly this:**

1. **Per family, exactly once.** For each output family the adapter renders —
   prints (stdout stream), messages (stderr stream), trace, and the single
   result — the sequence emitted across all segments of a resumed execute
   equals the queued execution's sequence for that family: same items, same
   multiplicity, same order. Track progress **by position per family** (you
   measured prints, messages and trace as append-only across all six
   boundaries); do not use the combined `outputs` list, which you measured as
   rebuilt and not append-only.
2. **Before the prompt.** Output the host reports at a suspension is emitted
   before that suspension's `input_request`.
3. **Across families within one segment**, keep the renderer's existing order
   and document it in the adapter rustdoc as a limitation of the host
   contract, which groups output by family.
4. Equal strings printed twice remain two outputs.

Rewrite the parity tests to assert (1)-(4); keep your
`red_complete_iopub_order_matches_queued_input` evidence in the report, not as
a permanent test. Then do the rest of the brief as written: the permanent
matrix (one prompt, two prompts, nested `Get`, repeated equal prints, messages
before and after a prompt, failure after resume, interrupt after resume), red
first; the emission state in brief-557's owned lifecycle; the standing
retention replay; every gate in Verification. Append your report covering both
runs.
