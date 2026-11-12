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

## Class-duty STOP gate (2026-11-13) — the STOP is well taken: a published request report and its suspension are not one owner, the raw inspection resume leaves the report behind, and the same state panics a public request resume; brief 733 is held and brief 734 designs the pairing

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; dispatched forked from
the brief-732 thread `01a1d11e-ea46-7b13-a770-1b0de2b23281`, run thread
`01a1d151-2841-73b1-930e-3276e1c10ae2`); log
`.temp/logs/cdx_brief_733.log` (7,681 lines, 433,043 B, sha256
`30f9449a…`); memory-scope peak 3,080,945,664 B with oom 0, oom_kill 0,
rc 0; 16 run records (12 exit 0; the 4 exit-101 designed reds — entry
14/15 at eight and one, control 0/1 at eight and one); `report.py
check 733` OK with 162 citations by my own run; 0 of the 4 canonical
builds used; no executed git or network; every write under
`.temp/brief-733/`; no live source or inventory change.

**My scope gate.** The entry copy is the live working tree: my recursive
comparison under `crates/` finds 0 differences, and the only five paths
that differ overall are the non-source dot directories and
`scripts/__pycache__` the copy left out. The nine pinned files (the seven
refusal production files, the shared entry, the permanent witness) hash
match live. The candidate arm differs from entry in exactly the four named
files; its three production files are byte-equal to brief 732's
`source-candidate-a`, its test file is byte-identical to brief 732's probes
arm (504 lines, the probe function verbatim, the pre-existing 444 lines and
11 tests unchanged). The control arm differs from entry by the four C4
harness files and exactly my confirmed removal of the one
`refuse_nested_top_level_input` call line (`session_eval.rs:298`). The live
tree is untouched: `git status` is still exactly the authorized stage-2
install and no live file is newer than the dispatch; the brief-732,
brief-731, `g15-c2` and brief-730 scenes have no file newer than the
dispatch. The class-route binary links only the entry arm's rlibs and uses
only public APIs.

**My reproductions.** I ran the worker's route myself at eight and one and
read the same tuples it recorded: the old owner's report prints
`["old-prefix"]`; the raw inspection resume completes the owner and leaves
`pending_input_request` None; a fresh raw `InputString` capture yields the
typed refusal with the new owner pending; the request resume returns
`"new-answer"` with `["old-prefix"]`. I then extended the route: the same
stale-report state panics a public request resume — `eval_request` →
`resume_inspect_rewrite_rules_with_input` → `resume_eval_request_with_input`
panics at `suspension/core.rs:646:18`, `resume requires a pending suspended
evaluation`, an unconditional `expect` in both build profiles
(`.temp/director/gate-733/panic-route.rs`, compiled against the entry arm's
rlibs).

**The mechanism, read from source.** The report is stored by
`finish_input_required_request` (`session_requests.rs:506` →
`store_pending_request:70`) and taken only by the request driver's resume
(`:293`) and by cancellation (`take_pending_report:53`). The raw resume
`resume_suspended_slot` (`core.rs:635-714`) takes the slot, clears the
host-facing prompt (`:664`, `finish_completed_resumption`
`:607-624`), and never touches `pending_request`. A nested entry defers to
a stored report (`refuse_nested_top_level_input`, `session_eval.rs:40-51`).
So after a raw inspection resume a published report survives its owner;
the worker's route attaches it to a different owner's response, and the
panic route resumes a vacant slot through it. The class is the pairing
itself: the report (the request driver's checkpoint and reported output)
and the suspension are created together and must be destroyed together,
and today a non-driver consumer can destroy one without the other. Brief
733's step-6 invariant — whenever `pending_request.is_some()` a foreign
capture sees that report's own suspension — is false on the live source,
so the capture repair's guard cannot be certified from it.

**Ruling.** Q1: the STOP is well taken, and the counterexample and the
panic are genuine pre-existing lifecycle defects, not defects of the
capture repair; the accepted three-file shape, the C4 evidence and the
brief-732 design stand, and the report/slot pairing is a premise the design
took for granted and the brief asked to prove. Brief 733 is held, not
failed: its entry/control reds, its prepared candidate (unformatted,
unbuilt, untested) and its scope receipts are preserved, none of the
unrun gates is credited, and its 4 canonical builds stay untouched. Brief
734 (design-only, `Scope: NONE`) designs the pairing in a byte copy of this
live tree: it must reproduce both routes at eight and one itself,
inventory every store/take/clear of `pending_request` and every install/
consume/abandon/fault/reset/restore of the suspension with one disposition
each and name every separation path, settle the public contract of
`resume_inspect_rewrite_rules_with_input` on a request-created owner
(typed refusal or report retirement, with every public surface and any API
break named for `CHANGELOG.md`), recommend exactly one mechanism with exact
signatures and evaluate at least two alternatives, prove from source that
after the repair `pending_request.is_some()` is an exact ownership
predicate so brief 733's guard is sound, keep the documented host workflow
(`eval_request`, `resume_eval_request_with_input`, cancel, reset, script
and file batches, snapshot), the permanent tests and the five C4 owner
records byte-identical, measure the changed functions' instruction rows
with no new work on the ordinary entry path, check the C2 ledger for an
existing row or name the new obligation, and draft the implementation
brief with the combined install and cost plan for brief 731 continuation 6
(the seven-file stage-2 install + this pairing repair + the three-file
capture repair + the accounting regression). Any change to 733's accepted
shape, the witness, an assertion, a C4 record, the documented host contract
or the budget is a STOP for my ruling. Recorded, not gated: the worker ran
the class duty before steps 4-5 (candidate green, release equality) rather
than after; I accept it because those receipts would be superseded by the
repaired source and brief 732 already holds the candidate's 15/15 and the
ten byte-exact C4 comparisons on the same patches. The entry-manifest
FLAGGED is exactly the two director documents; the control's numeric tuple
is brief-732 historical evidence, not re-instrumented; the class survey is
51 anchored sites and six capture callers, not an ownership proof; the two
`audit-stop` attempts are retained.

**Next.** Write and dispatch brief 734; gate its design; write and gate its
implementation brief; then resume brief 733 (a continuation forked from
`01a1d151-2841-73b1-930e-3276e1c10ae2`) on the repaired source to
re-establish its entry/control red, candidate green at eight and one
ordinary and profiled, the ten C4 comparisons and the named surfaces; then
brief 731 continuation 6 installs the combined delta and runs the standing
replay, acceptance and cost lane on that final source; then continuation 38
of brief 623; then C2's settlement round, CM, `CHANGELOG.md` before S10
(including this lifecycle contract and the earlier enum break and
`EvalError::Syntax` break), G13 S10-S54/F1-F23/R, E2.
