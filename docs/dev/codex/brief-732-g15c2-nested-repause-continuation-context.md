# Brief 732 — G15-C2: design the nested re-pause continuation-context repair (`Scope: NONE`)

## Goal

The permanent rule-5 witness of brief 731 is red in the debug acceptance
build: a pre-existing resumable owner, resumed from a nested raw entry, pauses
again; the driver merges the fresh nested suspension with the resuming
suffix, and the merged suspension's active stack no longer covers the suffix
cells' saved `ResumeContext`s; the next top-level resume violates
`restore_resume_context`'s debug invariant
(`suspension/core/evaluation_context.rs:33`, measured `context.stack_len=1,
depth=1` against an installed evaluation stack of length 0). This brief
designs the repair. It changes no live source: the deliverable is a reviewable
design, the measured red and base control, the calibrated candidate in a byte
copy, and the drafted implementation brief.

## Context pointers

- The live tree is dirty with the authorized brief-731 stage-2 install: 11
  modified files plus one new test over `HEAD d385a233`. The exact live
  manifest is `.temp/brief-731/continuation-5/installed-tree-hashes.json`; the
  twelve changed paths and their hashes are `live-changes.json`; the seven
  pinned production hashes are `production-hashes.json`; the three canonical
  rustfmt expansions are ruled but not yet applied
  (`required-rustfmt.patch`). The brief-731 continuation-5 gate section (last
  section of `docs/dev/codex/brief-731-g15c2-nested-eval-text-typed-refusal.md`)
  states the finding, values and ruling.
- The red: `crates/rum-stdlib/tests/nested_public_evaluation.rs:353-381`,
  `nested_reentrant_resume_preserves_the_real_preexisting_owner`.
- Mechanism files (read them all before designing): `runtime/suspension/core/driver.rs`
  (`drive_frames`, its `InputRequired` merge at `:160-185`), `runtime/suspension/core.rs`
  (`begin_input_suspension` `:500-511`, `resume_suspended_slot` `:630-700`,
  `take_accounted_pending_suspension_for_resume`, `restore_nested`),
  `runtime/suspension/core/evaluation_context.rs` (`ResumeContext`,
  `restore_resume_context` `:32-40`), `runtime/suspension/core/abandon.rs`,
  `runtime/session/session_requests.rs`
  (`resume_eval_request_from_checkpoint` `:283-379`,
  `finish_input_required_request` `:506`), and `runtime/model/evaluation_depth.rs`.
- The C4 release evidence: `.temp/brief-731/continuation-4/source-candidate-a/crates/rum-stdlib/tests/b731_owner_probe.rs`,
  `owner-proof.json`, and the five `owner4-*` logs. It is behavior evidence
  only: the release build compiles the debug assertion out.

## Scope

- **Scope: NONE — design only. No live edit.**
- Work only in a fresh byte copy of the live working tree under
  `.temp/brief-732/source-entry/`. Record its complete entry manifest before
  any scratch edit; every scratch arm gets its own name. Copies build into
  their own target dir (`Cargo's unit hash does not include the package
  path`).
- MUST NOT touch: the live tree, `.git`, any `.temp/g15-c2` scene, any other
  brief's evidence, `Cargo.lock`, dependencies, `Expr`/`Symbol`
  representation, public surfaces; the seven brief-731 production files and
  the witness file stay byte-identical. (A scratch arm may remove only the
  refusal call to prove pre-existence; that arm is disclosed and never
  installed.)
- No git in any form; no network.

## Hard rules

- Worker sandbox `workspace-write`; all writes under `.temp/brief-732/`; one
  Cargo at a time in the 16G scope; no `unsafe`; no new dependency; no
  `#[cfg(not(debug_assertions))]` in the witness; no weakened, skipped or
  shape-dodged assertion; no fabricated state.
- The debug assertion at `evaluation_context.rs:33` is the observer: the
  candidate must simply not fire it. Never remove, narrow or cfg-gate it.
- Release behavior is the contract: the five C4 owner probes must stay
  byte-identical on the candidate.
- Canonical probe builds go through `scripts/probe-build.py`, at most **12**
  for this design; debug/test builds are host cargo in the copy's own target
  dir and every invocation is recorded. A measurement or step that cannot
  fit is a STOP, not a self-authorized cap.

## What to do

1. **Reproduce the red** in the copy's debug build at eight and one threads;
   quote the panic, the test and the measured values (my gdb trace of
   `restore_resume_context` on the installed test binary reads the failing
   restore as `stack_len=1, depth=1` against `len=0, cur_depth=0`, after a
   `0/0` restore on the same empty stack; the backtrace is the top-level
   resume at test line 370 → `resume_eval_request_from_checkpoint:329` →
   `resume_suspended_slot:682` → `drive_frames:36` → `restore_resume_context:33`).
2. **Prove pre-existence** with a base control: in a scratch arm remove only
   the refusal call in the shared entry's outer error arm (the brief-731
   falsifier shape), so the nested entry returns the raw `InputRequired`; the
   same nested-resume-then-top-level-resume route must hit the same assert
   with the same values. If it does not, STOP and report the difference: the
   refusal shape would be causal.
3. **Survey the class**: every `ResumeContext` capture (`begin_input_suspension`,
   the two frame pushes at `core.rs:548/583`), storage (`Queued`,
   `FrameStorage`), merge (`nested.append(&mut suffix)`), restore (the driver,
   the report install) and abandon site; the accounting model
   (`EvaluationStackEntry`, `evaluation_depth`, memory owners); which sites can
   see a nested re-pause and what each assumes. One table, one disposition
   each.
4. **Explain the mechanism** from source with values: which cells' contexts
   exceed the merged active stack, why the fresh stack is what it is, what the
   accounting means for growth limits and memory owners while resumed, and
   why release completes correctly anyway.
5. **Propose repair candidates** with exact signatures and file lists; at
   least (a) rebase the suffix cells' contexts against the merged stack at
   merge time, (b) build the merged suspension's active stack so it covers
   every suffix context (keep the first suspension's needed prefix), (c) any
   route the source supports. For each: the invariant restored, the
   accounting effect, the release behavior, the risks, and the effect on
   `depth` and memory owners. Recommend one by evidence, not cost.
6. **Calibrate the recommended candidate** in the copy: the 15-test permanent
   file green at eight and one threads (debug); the five C4 owner probes
   byte-identical (release, using the C4 probe file and its recorded outputs);
   the affected suspension/resume surfaces green at eight and one (name every
   suite); the debug assert never firing. Any change to a C4 release record is
   a STOP.
7. **Draft the implementation brief** in TEMPLATE shape: exact scope and file
   list; the red and green calibration; the class duty; the release-record
   equality bar; the build budget; the stop-loss; and the combined-tree
   standing replay, acceptance and cost duty that follows the install.
8. Report and STOP for my gate.

## Deliverable

- The measured red and base control with copied verdict lines and the gdb
  values.
- The site inventory table.
- The mechanism explanation with values.
- The candidates and the recommendation.
- The calibrated candidate's green/red receipts.
- The drafted implementation brief.

## Verification (paste verbatim lines)

```
cargo test -p rum-stdlib --test nested_public_evaluation -- --test-threads=1
cargo test -p rum-stdlib --test nested_public_evaluation -- --test-threads=8
[base control] the same route without the refusal call
[candidate] the same two at 8 and 1
[candidate, release] the five C4 owner probes
[named suspension/resume suites] at 8 and 1
```

A skipped or zero-test run is not evidence. Any red is reproduced at
`--test-threads=1` before it is called a defect.

## Stop-loss

Two consecutive compiling candidate attempts that move neither the witness
(the same assert still fires) nor a named progress counter (the offending
context values, the cells' contexts, the merge site) are a STOP. Any need to
change an assertion, the seven refusal files, a public surface, the witness
file or a C4 release record is a STOP.

## Report format

`rum-report/1` JSON at `.temp/brief-732/report.json`; every claim carries a
log path and a copied verdict line.

## Design gate (2026-11-13) — brief 732's design is accepted: the defect is a post-publication foreign capture, not the merge, and candidate C (capture-time bound by the pending owner's remaining prefix, depth unchanged) is the repair; brief 733 implements it in a copied arm

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1d11e-ea46-7b13-a770-1b0de2b23281`, fork of
`01a1d103-6d6e-71c3-8736-610803d677e5`); log
`.temp/logs/cdx_brief_732.log` (18,494 lines, 1,211,326 B); memory-scope
peak 8,563,159,040 B with oom 0, oom_kill 0, rc 0; 58 run records (45
exit 0; 6 exit 101 — the four red reproductions and the two instrumented
entry/control diagnostics; 5 exit 1 and 2 exit 2 retained instrument and
script-preparation failures, all disclosed in the report); `report.py
check 732` OK with 943 citations by my own run; 2 of the 12 canonical
design builds used, fresh and forced with the same SHA `ed2b630a…` at one
absolute path and one codegen unit; no executed git or network; the
21,955 reported additions are all under `.temp/brief-732/`; no live
source change.

**My scope gate.** My own hash walk of the live tree against
`.temp/brief-732/live-entry-hashes.json` reads 2,730 manifest paths and
2,730 live files: 0 missing, 0 extra, 0 differing. HEAD is still
`db841b34`; the seven brief-731 refusal files, the permanent witness,
`Cargo.lock` and the dependency manifests are untouched. My own recursive
diffs of the copied arms: the candidate arm differs from the entry copy in
exactly the three named production files; the release arm adds exactly the
four C4 harness test files; the probes arm differs from the candidate arm
by the regression addition only; the control arm differs from the entry
copy by the four harness files and my confirmed removal of the single
`refuse_nested_top_level_input` call in `session_eval.rs`. The debug
assertion in the candidate arm is byte-identical to live
(`evaluation_context.rs:32-40`).

**The design corrects my continuation-5 mechanism sentence, and I accept
the correction.** My gate said the fresh `active_stack` captured at the
second pause does not cover the suffix cells' contexts. The measurement
says otherwise: at the merge (`driver.rs:171`, `nested.append(&mut
suffix)`) the queue is 14 valid cells on a one-entry saved stack
(`InputString["owner-second"]`, non-increasing from `(1,3)` to `(0,0)`).
The two invalid cells are created *after* the driver republished the
owner: `finish_input_required_request` stores the report
(`session_requests.rs:506` → `store_pending_request:70`), the displaced
native caller stack/depth are restored (`core.rs:688-689`), and the
native evaluator's `InputRequired` unwind appends four frames through the
typed/legacy pending-push captures (`core.rs:583/548`). The first two
carry absolute contexts `(1,1)` measured against the caller's
`ResumeExisting[]` stack, not against the owner's remaining prefix, which
after its first 14 cells is length 0. Zero-based cells 14 and 15 of the
18-cell queue are therefore the offenders. The entry, control and
candidate instrumented copies show the merged 14 cells identical in all
arms, the offending captures with the report present, and the candidate's
tail changed from `[(1,1),(1,1),(0,0),(0,0)]` to
`[(0,1),(0,1),(0,0),(0,0)]` with every earlier cell, payload, epoch and
depth equal (`mechanism-proof.json`, `logs/observe-*-t1-1.log`). This is
an erratum to my continuation-5 gate: a merge-only rebasing cannot repair
cells that do not exist at the merge, and the recorded sentence is
withdrawn. The design's correction is measured, not argued.

**My own reproductions.** In the byte copies: the entry arm runs the
unchanged permanent file at one thread as 14 passed / 1 failed with
`continuation contexts unwind a saved prefix` at
`suspension/core/evaluation_context.rs:33:9`; the control arm — my own
diff removes only the one refusal call — reaches the same assertion at
the same line through C4's unchanged real-owner route at one thread; the
candidate arm runs 15 passed / 0 failed at eight and at one thread; the
candidate release binary (sha `ed2b630a…`, matching its receipt) yields
`B731 C*` records byte-identical to the frozen C4 originals in all ten
comparisons (five cases × eight and one threads, 0 normalized tokens). I
enumerated the six production `resume_context()` callers
(`core.rs:548`, `core.rs:583`, `evaluation_context.rs:59`,
`driver.rs:89`, `driver.rs:136`, `queue_api.rs:59`) and spot-checked the
site table's anchors (`core.rs:505` capture, `core.rs:269` slot install,
`driver.rs:36` restore, `abandon.rs:82` strict observer,
`model/session.rs:114-115` the two fields, `SuspensionSlot::as_ref` at
`core.rs:248`). I did not rerun the 180/38/436 surface suites, the
profiled/accounting arms, or the design's observer scripts; I read their
retained logs, and their acceptance-grade reruns belong to continuation 6
on the installed source.

**The accepted shape.** When the request driver has published a pending
request and the slot holds its suspension, `Session::resume_context`
bounds the captured length by the pending owner's remaining prefix — its
last semantic cell, or the saved active-stack length for an empty queue —
while capturing the current logical depth unchanged. `FrameStorage` and
`FrameQueue` gain private read-only tail accessors; the Guest and
Profiled forms both read the semantic guest queue; no flag, frame, root,
allocation, epoch or public signature is added. The corrected value is
exactly the length release actually uses: `truncate_accounted_evaluation_
stack` never grows the vector, so a recorded length above the remaining
prefix was always a no-op in release; the capture now records the
effective value instead of an ineffective request. That is why this is
the repair and not a test accommodation. Depth stays independent
(`evaluation_depth`), the prefix assertion stays byte-identical, and the
release records confirm behavior is unchanged.

**Ruling.** Q1: accepted — candidate C is the repair. Q2: accepted — the
implementation scope is the three production files plus the
actual-request armed/unarmed × resume/cancel accounting regression
appended to `runtime/tests/resume_evaluation_context.rs`; the combined
standing replay, acceptance and cost stay in brief 731 continuation 6.
Brief 733 (implementation, staged) is written: a fresh byte copy of this
live tree proves the entry red, the one-call control red, the four-file
candidate green at eight and one (ordinary and profiled), the ten frozen
C4 release comparisons and the named surfaces, and STOPs; its canonical
budget is 4 fresh builds, and brief 731's 8 remaining builds stay
separate — no transfer, no borrowing, and a step that cannot fit is a
STOP. The rule-5 witness, the seven refusal files and the assertion are
never edited. After 733's gate, brief 731 continuation 6 applies the
ruled formatting and re-pins, installs the accepted repair, re-runs
`fmt`/`check` and the witness green, then runs stage 3 exactly as ruled
(standing replay first, acceptance once, the cost lane on the combined
ten-file delta, two draws, equal calls, one codegen unit, every positive
row retained, one forced SHA; the entry row's 599/4 bar and the
no-increase scan unchanged). Until then the stage-2 install stays
uncommitted and stage 2 does not close.

**Recorded, not gated.** The design corrects my continued-5 sentence,
which is why the gate records it as an erratum rather than a STOP. The
worker's ptrace denial is the sandbox boundary, and its isolated source
observers are calibrated against my gdb tuple (1/1 on 0/0 after a 0/0
restore). The retained exit-1 and exit-2 attempts are instrument and
script-preparation failures with no candidate source edit; the first
release parser's empty extraction and the first calibration audit's
missing `should panic` names are disclosed and corrected without rerunning
accepted reference stages. The candidate's double blank line before the
regression must be canonicalised by the implementation's formatter, and
the three ruled brief-731 formatter arms stay red on `cargo fmt --check`
until continuation 6 applies them. No live commit.

**Next.** Write and dispatch brief 733 (implementation, staged, copied
proof); gate it; then dispatch brief 731 continuation 6 (formatting and
re-pin, repair install, `fmt`/`check` and witness green, stage 3); then
continuation 38 of brief 623 on the installed source; then C2's
settlement round, CM, `CHANGELOG.md` before S10 (including this enum break
and the earlier `EvalError::Syntax` break), G13 S10-S54/F1-F23/R, E2.
