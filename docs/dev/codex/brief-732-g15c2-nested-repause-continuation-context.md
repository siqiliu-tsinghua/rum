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
