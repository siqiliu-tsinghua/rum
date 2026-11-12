# Brief 733 — G15-C2: implement the accepted published-owner continuation-context capture (staged, copied proof)

## Goal

Install the capture-time shape accepted from brief 732 into a fresh byte copy
of the current live tree, together with the permanent actual-request
accounting regression, and prove on that four-file shape: the entry red, the
one-call control red at the same assertion, the candidate green in debug at
eight and one threads (ordinary and profiled), the ten frozen C4 release
record comparisons byte-exact, and the named suspension/resume surfaces
green. The deliverable is the installable patch and its receipts. No live
edit; STOP for the director's implementation gate.

## Context pointers

- Brief 732's design, gate section (the last section of
  `docs/dev/codex/brief-732-g15c2-nested-repause-continuation-context.md`),
  `design.md`, the 51-row `context-sites.tsv`, `mechanism-proof.json`,
  `calibration-proof.json`, `release-owner-proof.json`, `scope-proof.json`,
  and the three `*.candidate-a.patch` files under `.temp/brief-732/`.
- The regression addition is the diff from
  `.temp/brief-732/source-candidate-a/crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs`
  to the probes arm's same file (`.temp/brief-732/source-probes-candidate-a/...`):
  one test, `b732_reentrant_published_owner_keeps_accounting_and_cleanup`.
- The live entry bytes are pinned by `.temp/brief-732/live-entry-hashes.json`
  (2,730 files, my own walk verified it file for file). The live tree is
  dirty with the authorized brief-731 stage-2 install; the copy is taken from
  the live working tree, not from a git revision.
- Read before editing: `suspension/core.rs`, `core/driver.rs`,
  `core/evaluation_context.rs`, `core/queue.rs`, `core/storage.rs`,
  `core/abandon.rs`, `session/session_requests.rs`, `model/session.rs`,
  `model/evaluation_depth.rs`.

## Scope

- Work only under `.temp/brief-733/`, in a fresh complete byte copy of the
  live working tree under `.temp/brief-733/source-entry/`. Record the
  complete entry manifest before any edit; every scratch arm gets its own
  name and its own target directory.
- Four arms, and nothing else:
  1. **Entry arm** — the unmodified copy.
  2. **Control arm** — entry plus the four unchanged C4 harness test files
     (`b730_nested.rs`, `b731_owner_probe.rs`, `b731_legacy_probe.rs`,
     `b731_legacy_probe_old.rs` from
     `.temp/brief-732/source-release-candidate-a/crates/rum-stdlib/tests/`),
     with exactly one line removed from `session_eval.rs`: the call
     `let err = self.refuse_nested_top_level_input(err);`. This arm is
     disclosed and never installed.
  3. **Candidate arm** — entry plus the three accepted production files from
     the three `*.candidate-a.patch` files, plus the accounting regression
     appended to
     `crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs`.
  4. **Release arm** — the candidate arm plus the same four harness files.
- The three patched production files must be byte-identical to brief 732's
  `source-candidate-a` files. The regression addition must carry the probe
  function's exact tokens, assertions and four cases; formatting may reflow
  it, and the reflow and a whitespace-normalized token proof must be
  reported. The pre-existing bytes of that test file (its 444 lines, 11
  tests) stay unchanged.
- MUST NOT touch: the live tree; `.git`; any other brief's evidence;
  `Cargo.lock` or dependency manifests; the seven brief-731 refusal
  production files (`outcome.rs`, `host/src/session.rs`,
  `model/requests.rs`, `request_errors.rs`, `session_eval.rs` (except the
  control arm's one disclosed line), `session_state.rs`,
  `suspension/error.rs`); the permanent
  `crates/rum-stdlib/tests/nested_public_evaluation.rs`; public types,
  methods, serde/wire data; the driver algorithm, request lifecycle, owner
  topology, epochs, native-stack polls, depth logic, memory formulas, frame
  payloads, profile topology and cleanup ordering.
- No git in any form; no network; no new dependency; no `unsafe`.

## Hard rules

- Worker sandbox `workspace-write`; one Cargo at a time inside the 16-GiB
  scope; every invocation recorded; bounded wrappers (400 s copied debug,
  300 s canonical build, outer wrapper allowing cleanup).
- Canonical release builds go through `scripts/probe-build.py` at the stable
  active path with one release codegen unit and a forced `--force`
  spot-check; at most **4** fresh builds for this brief (the release arm's
  fresh build plus its forced rebuild normally uses two). Cache hits and
  reused receipted binaries do not count. Brief 731 continuation 6 keeps its
  separately ruled 8 remaining builds; no transfer or borrowing. A step that
  cannot fit is a STOP, not a self-authorized cap.
- The debug assertion at `evaluation_context.rs:33` is the observer. It is
  never removed, narrowed, moved or cfg-gated; the candidate must simply
  not fire it.
- Any change to a C4 release record, the seven refusal files, the witness,
  a public surface, depth semantics or the memory-owner model is a STOP.
- `cargo fmt --check` on the entry and candidate arms is expected to report
  exactly the three ruled brief-731 arms (`outcome.rs:696`,
  `model/requests.rs:53`, `request_errors.rs:133`; the fourth diff entry is
  the same `outcome.rs` through the `suspension_oracle` path alias). The
  implementation must not touch them — continuation 6 applies that ruled
  formatting. A fourth file or any diff inside the touched files is a STOP.

## What to do

1. **Entry copy and manifest.** Copy the live working tree; verify all 2,730
   files byte-identical to `.temp/brief-732/live-entry-hashes.json`. Record
   each arm's full manifest before its edit; disclose every scratch
   addition.
2. **Entry red.** Run the permanent file in debug at eight and one threads:
   expect 14 passed / 1 failed, the panic
   `continuation contexts unwind a saved prefix` at
   `suspension/core/evaluation_context.rs:33:9`. Paste both logs.
3. **Control red.** In the control arm run C4's unchanged
   `b731_owner_probe::b731_reentrant_resume_preserves_preexisting_owner` at
   eight and one threads. It must reach the same assertion with the same
   observed tuple; its helper name and epoch (`B731ResumePending[]`, 857)
   stay recorded literally. A different failure is a STOP.
4. **Candidate green.** In the candidate arm prove the three patched files
   byte-equal brief 732's and the regression added; run `cargo fmt` on the
   addition only; then run at eight and one threads: the permanent file
   (15/15), the permanent file with
   `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` (15/15), the accounting regression in
   ordinary and profiled modes (16 cases), the runtime lib surface filters
   (`resume_evaluation_context`, `session_requests`,
   `suspension_abandonment`, `session_memory_request_owners`,
   `frame_root_trace`, `queue_boundaries`), the seven named host suites, the
   nine named stdlib suites, `cargo check --all-targets`, and the fmt check
   above. A skipped or zero-test run is not evidence.
5. **Release equality.** Build the release arm through `scripts/probe-build.py`,
   run the five C4 owner probes at eight and one threads, and compare the
   `B731 C*` records byte-for-byte with the frozen originals at
   `.temp/brief-731/continuation-4/logs/owner4-<case>-t<threads>-1.log`.
   Slice libtest framing by C4's existing marker rule only; no payload
   normalization. One forced SHA. Any changed record is a STOP; release
   equality is behavior evidence, never the debug invariant.
6. **Class duty.** Re-survey the 51-row `context-sites.tsv` against the
   installed shape with one disposition per row; enumerate the six
   production `resume_context()` callers and state each one's capture
   condition under the guard; prove from source that whenever
   `pending_request.is_some()` and a foreign capture occurs, the slot holds
   that request's own suspension (or is vacant with the documented
   fallback), and that front pushes and abandonment cannot see the bound.
   A route that breaks that invariant is a STOP.
7. **Report and STOP.**

## Deliverable

- The four-file patch (three production files and the regression addition),
  complete entry and per-arm manifests, and the formatted addition with its
  token proof against the probe function.
- The red, control and green receipts; the ten release comparisons; the
  named surface totals; the build-budget and forced-SHA receipts.
- The re-surveyed site table and the ownership-invariant source argument.
- `rum-report/1` JSON at `.temp/brief-733/report.json`, every claim with a
  log path and a copied verdict line, including every failed attempt.

## Verification (paste verbatim lines)

```
[copied debug, entry arm] the permanent file at 8 and 1: 14/1, the assertion
[control arm] C4's real-owner probe at 8 and 1: same assertion and tuple
[candidate arm] the permanent file at 8 and 1: 15/15
[candidate arm, profiled] the permanent file at 8 and 1: 15/15
[candidate arm] the accounting regression at 8 and 1, ordinary and profiled
[candidate arm] check --all-targets; fmt check with exactly the three ruled arms
[candidate arm] runtime filters / seven host suites / nine stdlib suites at 8 and 1
[release arm] the five C4 probes at 8 and 1, byte-exact records
[release arm] one forced SHA receipt
```

A red is reproduced at `--test-threads=1` before it is called a defect.

## Stop-loss

Two consecutive compiling candidate attempts that move neither the witness
(the same assertion still fires) nor a named measured counter (the tail
cells, the callers' captures, the release records) are a STOP; the accepted
shape is fixed, and a search for a new shape needs my ruling. Any need to
change the assertion, the seven files, the witness, a public surface, a C4
record, the depth or memory-owner semantics, or the build budget is a STOP.
Every attempt and every consumed artifact is preserved and reported.

## Checkpointing — protect the scene

After each milestone update `.temp/brief-733/PROGRESS.md` with the completed
evidence, the exact next action, the copy state and the spent/remaining
budget. Keep every log, manifest and patch under `.temp/brief-733/`; never
overwrite brief 732, the C4 evidence or any frozen G15 scene.

## Report format

`rum-report/1` JSON at `.temp/brief-733/report.json`; every command and
attempt, verbatim verdicts, test totals, scope hashes, the release comparison
rows, and any open decision. Close with
`python3 scripts/report.py summary 733` output and nothing else. Do not
commit; STOP for the director.
