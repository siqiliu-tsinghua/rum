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


---

## Execution report — resumed brief-558

1. **What changed**

The first run stopped before implementation because the original complete cross-family ordering requirement could not be met by the grouped host outcome. Its saved control showed that reversed Print/Message source order produces an identical queued HostEvalSuccess. Both ordering reds and the original report remain intact. The director revised the requirement to per-family equality; this resumed run implements that requirement.

Each admitted asynchronous execute now owns three usize positions (prints, messages, trace) in ExecuteContext.emitted, retained through suspension, resume and interrupt retirement and discarded with terminal completion. Rendering slices each cumulative family at its saved position and advances only that family's position. Equal strings remain distinct occurrences. No content comparison or combined-output cursor is used.

| File | Lines | Change |
| --- | --- | --- |
| crates/rum-jupyter/src/adapter/worker.rs | 25 | Put emission progress in the existing execute owner. |
| crates/rum-jupyter/src/adapter/execute.rs | 129; 149–177; 205–214 | Initialize positions once, advance at suspension, consume the same context at terminal completion. |
| crates/rum-jupyter/src/adapter/render.rs | 24–135 | Render family suffixes; keep immediate/rejected requests at zero positions; emit one result/error; handle empty synthesized panic checkpoints. |
| crates/rum-jupyter/src/adapter/mod.rs | 29–31; 50–63 | Import rendering entry point and document once-only delivery, family grouping and trace segmentation. |
| crates/rum-jupyter/src/tests/mod.rs | 93 | Register the socket-free harness. |
| crates/rum-jupyter/src/tests/output_once/mod.rs | 1–188 | Promote the archived exact stream comparison and add the full parity matrix. |
| crates/rum-jupyter/src/tests/output_once/support.rs | 1–395 | Record actual wire sends, compare full output families and MIME contents, enforce output-before-prompt, segment grouping, parent/count/route, one terminal reply and busy/idle. |
| crates/rum-jupyter/src/tests/output_once/host_buffers.rs | 1–140 | Measure six host-only boundaries and pin synthesized panic's absent-buffer contract. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | 2–69, final field only | Copy all 68 replayed fingerprints. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | 2–3, columns 5–6 only | Copy four replayed C01/C02 certificate cells; only hashes change. |
| docs/dev/codex/brief-558-jupyter-output-once.md | 242–499 | Append this report, preserving the brief and both earlier sections. |

The measured print, message and trace collections are append-only across input-to-input and input-to-terminal success, Exit failure and cancellation boundaries; their final buffers match queued execution. The combined outputs list is rebuilt by family and is not append-only. The terminal value is rendered only at success; errors are emitted only at terminal failure. Trace entries retain their order and multiplicity across multiple display bundles. The additional host-only panic probe found a real exception: synthesized panic failures contain empty print/message/trace/output buffers. Published prefixes survive; the terminal segment emits the error without applying positions to an absent checkpoint. This limitation is documented and tested, without changing the host or claiming that missing panic buffers can be reconstructed.

Notebook-visible changes: two prompts formerly displayed prefix three times and middle twice; now each appears once before its corresponding prompt. Four equal prints formerly produced six occurrences in the one-prompt case; now all four intended occurrences survive. Message and trace prefixes no longer replay on later prompts, success, Exit, or interrupt retirement. Cleanup remains visible exactly once, followed by one terminal error on cancellation. Within every segment the order stays stdout, stderr, trace, result/error. Cross-family chronology still follows the host's grouping, as the revised ruling requires. The final response's rendered outputs now describe its terminal segment, while its underlying host outcome retains complete cumulative buffers. Immediate and stdin-denied retirement still assemble prefix plus cleanup once.

Organization assessment: execute.rs is 9,504 characters; worker.rs is 12,417; adapter/mod.rs is 20,295; render.rs is 23,949 including 5,554 characters of unchanged inline tests (18,395 implementation characters). The new test files are 7,484, 14,867 and 5,276 characters. Lifecycle ownership and rendering remain at their existing boundaries; this small rendering change does not split unrelated inspection/service code or move existing tests. Future boundaries are non-execute services versus adapter dispatch, worker outcome normalization versus worker execution, and execution-output versus inspection rendering with tests in a sibling module. This is the explicit assess-on-touch exception for the existing larger modules.

2. **Verification**

All Cargo commands below used four jobs, offline mode, repository-local TMPDIR and run-bounded. Recorder invocations had an outer bound ten seconds greater than the recorded command. The first run measured 22.334s/23.497s with 110s bounds. This run used 110s initially from the prior Jupyter 26.353s envelope, then 40s after its 6.565s red measurement; formatting used 10s from the prior 1.909s measurement; fingerprints 80s from 16.305s; certificates 120s from 33.769/35.326s; no-default retention and workspace 650s from prior 274.222s and approximately 275s runs. These are timeout calibrations, not performance measurements.

Red evidence was saved before any production edit. All ten parity tests failed for replay; two host-buffer controls passed. The new harness initially compared decoded errors with unencoded errors; it was corrected before production work to run both through the same wire codec. The first codec correction used an invalid placeholder message type; that setup failure is also saved. The final red run has valid comparisons throughout. No pre-existing test or helper was loosened. The first production implementation passed all twelve focused tests and all 139 pre-existing Jupyter test entries.

The full retention gates also emit compiler diagnostics from deliberately invalid fixtures; these are negative-control evidence, not source compilation failures.

The first certificate runs below intentionally failed on expired recorded certificate hashes after generating zero-rejection receipts. After hash-only refresh, the full default workspace and no-default retention gates validate those certificates.

~~~text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo run --manifest-path .temp/brief-558/host-probe/Cargo.toml --target-dir target
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.24s
BOUNDARY 42/suspend-suspend: prints 1->2 prefix=true messages 1->2 prefix=true trace 5->8 prefix=true outputs 7->12 prefix=false
BOUNDARY 42/suspend-terminal: prints 2->4 prefix=true messages 2->3 prefix=true trace 8->15 prefix=true outputs 12->23 prefix=false
QUEUED_EQ 42: prints=true messages=true trace=true outputs=true
BOUNDARY Exit[7]/suspend-suspend: prints 1->2 prefix=true messages 1->2 prefix=true trace 5->8 prefix=true outputs 7->12 prefix=false
BOUNDARY Exit[7]/suspend-terminal: prints 2->4 prefix=true messages 2->3 prefix=true trace 8->12 prefix=true outputs 12->19 prefix=false
QUEUED_EQ Exit[7]: prints=true messages=true trace=true outputs=true
BOUNDARY cancel/suspend-suspend: prints 1->2 prefix=true messages 1->2 prefix=true trace 5->8 prefix=true outputs 7->12 prefix=false
BOUNDARY cancel/suspend-terminal: prints 2->3 prefix=true messages 2->3 prefix=true trace 8->9 prefix=true outputs 12->16 prefix=false
QUEUED_EQ cancel: prints=true messages=true trace=true outputs=true
RESULT {"command": "python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo run --manifest-path .temp/brief-558/host-probe/Cargo.toml --target-dir target", "bound_seconds": 110.0, "exit_status": 0, "elapsed_seconds": 22.334, "log": ".temp/brief-558/measure/host-buffers.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test --manifest-path .temp/brief-558/host-probe/Cargo.toml --target-dir target --test output-order -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.37s
test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test --manifest-path .temp/brief-558/host-probe/Cargo.toml --target-dir target --test output-order -- --nocapture --test-threads=1", "bound_seconds": 110.0, "exit_status": 101, "elapsed_seconds": 23.497, "log": ".temp/brief-558/red/output-order.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.44s
test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 139 filtered out; finished in 0.05s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 110 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1", "bound_seconds": 110.0, "exit_status": 101, "elapsed_seconds": 6.565, "log": ".temp/brief-558/red/permanent-matrix.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.72s
test result: FAILED. 2 passed; 10 failed; 0 ignored; 0 measured; 139 filtered out; finished in 0.07s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 4.864, "log": ".temp/brief-558/red/permanent-matrix-wire.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.38s
test result: FAILED. 2 passed; 10 failed; 0 ignored; 0 measured; 139 filtered out; finished in 0.07s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 101, "elapsed_seconds": 4.509, "log": ".temp/brief-558/red/permanent-final.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 139 filtered out; finished in 0.08s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-jupyter output_once -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 0, "elapsed_seconds": 5.324, "log": ".temp/brief-558/green/permanent-first.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo fmt -p rum-jupyter
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo fmt -p rum-jupyter", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 0.193, "log": ".temp/brief-558/green/format.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.61s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 40 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter -- --nocapture --test-threads=1", "bound_seconds": 40.0, "exit_status": 0, "elapsed_seconds": 5.262, "log": ".temp/brief-558/green/jupyter.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.82s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.11s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib regenerate_local_contract_fingerprints -- --ignored --nocapture", "bound_seconds": 80.0, "exit_status": 0, "elapsed_seconds": 15.004, "log": ".temp/brief-558/retention/fingerprints.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo fmt --check
RESULT {"command": "python3 scripts/run-bounded.py --timeout 10 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo fmt --check", "bound_seconds": 10.0, "exit_status": 0, "elapsed_seconds": 1.75, "log": ".temp/brief-558/green/fmt-check.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.68s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 18.01s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 31.809, "log": ".temp/brief-558/retention/default-certificate.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.31s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 16.95s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib --no-default-features local_production_acceptance -- --nocapture", "bound_seconds": 120.0, "exit_status": 101, "elapsed_seconds": 31.369, "log": ".temp/brief-558/retention/no-default-certificate.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.18s
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 225.46s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp cargo test -p rum-stdlib --no-default-features retention_closure -- --nocapture", "bound_seconds": 650.0, "exit_status": 0, "elapsed_seconds": 238.849, "log": ".temp/brief-558/retention/no-default.log"}
~~~

~~~text
$ python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.76s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
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
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.96s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.35s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 227.39s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.21s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.39s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
RESULT {"command": "python3 scripts/run-bounded.py --timeout 650 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-558/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture", "bound_seconds": 650.0, "exit_status": 0, "elapsed_seconds": 263.726, "log": ".temp/brief-558/green/workspace.log"}
~~~

The permanent matrix, before/after:

| Permanent test | Red before production | Green after |
| --- | --- | --- |
| red_worker_streams_match_queued_input | FAILED | passed |
| one_prompt_outputs_match_queued_input | FAILED | passed |
| two_prompts_outputs_match_queued_input | FAILED | passed |
| nested_get_outputs_match_queued_input | FAILED | passed |
| equal_prints_keep_every_occurrence | FAILED | passed |
| messages_before_and_after_prompt_match_queued_input | FAILED | passed |
| trace_events_match_queued_input | FAILED | passed |
| failure_after_resume_matches_queued_input | FAILED | passed |
| interrupt_after_resume_matches_queued_input | FAILED | passed |
| cross_family_order_is_grouped_per_segment | FAILED | passed |
| host_families_append_across_success_exit_and_cancellation | passed control | passed |
| synthesized_panic_has_no_buffers_and_preserves_published_prefix | passed control | passed |


Native ZeroMQ bodies did not run because loopback binding is denied. The explicit existing skip opt-in was used; Cargo counts these early-return tests as passed. The nine skip lines from the Jupyter gate are verbatim:

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

Ownership discovery and exact reconciliation from the workspace gate:

~~~text
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7546 rows
~~~

3. **Counters, disclosures and unresolved work**

Production implementation attempts: 1. Production/test-harness compiler errors: 0 (intentional compiler-negative retention fixtures are separate). Consecutive compiling implementation attempts without progress: 0. Permanent parity tests: 10/10 green; additional host-buffer controls: 2/2 green. All nine full-family comparisons report stdout=true, stderr=true, trace=true, terminal=true; the promoted archived test additionally preserves its exact aggregate stream/queued-host-print comparison. Existing Jupyter entries: 139/139 green (nine native bodies skipped); total Jupyter entries: 151. Workspace totals across 40 result blocks: 2856 passed, 0 failed, 58 ignored. No-default retention: 49 passed, 0 failed, 1 ignored. The ignored test names/reasons are saved in green/ignored-tests.txt; the fingerprint regeneration test was run explicitly. No-progress stop-loss was not reached.

Standing-duty changes are limited to the final fingerprint fields on every row (lines 2–69) of local-contracts.tsv and the C01/C02 hash=true/hash=false certificate cells. The fingerprint identities are:

B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, I01, I02, I03, I04, P01, P02, P03, P04, P05, P06, P070, P080, P090, P100, P110, P120, P130, P140, P150, P071, P081, P091, P101, P111, P121, P131, P141, P151, B13, R01, R02, R03, R040, R050, R060, R070, R080, R090, R100, R110, R041, R051, R061, R071, R081, R091, R101, R111, R12, R13, R14, R15, O010, O0200, O011, O0210

The replay installers verified equality of all non-hash fields, including identities, statuses, dispositions, preconditions, strengths and exclusions. No ownership inventory positions needed changes. Every before/after cell is recorded in retention/fingerprint-changes.json and retention/certificate-changes.json; certificate receipts and generated tables are copied under .temp/brief-558/retention/. Prescribed replay artifacts also exist under target/brief-544/ and .temp/brief-546/certificates/.

No git command, network access, new dependency, unsafe code, host/kernel/stdlib implementation change or other frontend change. No evaluator-path instrumentation or cost was added. No microbench ran, as instructed. Nine native ZeroMQ bodies and the gate's ignored tests did not run; the fingerprint writer was explicitly invoked separately. The first run did not run its implementation gates because it stopped at the original scope conflict; this resumed run supplies the required implementation, retention and workspace gates.

All scratch, raw logs, command/exit records, before-images, the progress checkpoint and the final scope audit are under .temp/brief-558/. Source hashes match the source tested by the Jupyter/fmt gates. The original cross-family ordering falsifiers remain report evidence only, not permanent acceptance tests. No unresolved scope change is requested.
