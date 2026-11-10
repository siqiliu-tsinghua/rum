# Brief 723 — install the fixed-point pass publication frontier

**Shape: implementation, two staged gates.** Stage A proves the accepted
copied repair and the unexecuted failure frontier on a fresh byte copy;
stage B installs the exact accepted shape on the live tree and runs the
standing replays, the cost gate and the acceptance suites. A stage-A red
STOPs before any live edit. This is the implementation of brief 722's
accepted design; its evidence is `.temp/brief-722/` and the frozen C8
scene `.temp/g15-c2/c8/`. Neither may be modified.

## Goal

For the two fixed-point producer rows P1470 and P1471, the paused
`evaluate.once_unstable` logical record must sample the actual completed
pass — the typed `Expr` the fixed-point consumer receives, after the
evaluation-return resource check — before that consumer runs another
pass. For the same key and program the resumed record's ten counters must
equal the queued record's, in every counter, including the two-pause and
three-pause Throw/Return shapes.

The guest language result, messages, effects, evaluation count and order,
trace ordering, request freshness, evaluation-return resource checks,
cancellation and abandonment ownership, public carriers, both fixed-point
frame bodies and the disabled path's cost do not change. The brief-721
definition-resume repair keeps its 470 whole records exact.

## The measured defect (accepted evidence, 2026-11-11)

Ten bound native differences, five per row, all valid and semantic-SAME,
in the single phase `Runtime { phase: "evaluate.once_unstable", source:
None }`; twelve minimized cases (8 DIFF / 4 SAME, all semantic-SAME). The
two-pause witness `C8FixedPointStep[Input["c8step1"],1,2]` (replies 73
then `Throw[83,c8error]`) reads queued `[5,2,3,0,0,0,3,0,0,0]` and
resumed `[5,1,4,0,0,0,4,0,0,0]`; the three-pause witness reads
`[7,4,3,0,0,0,3,0,0,0]` against `[7,2,5,0,0,0,5,0,0,0]`. The frozen
records are `port/fixedpoint-losses.json` and
`logs/fixedpoint-{real,min-real}-1.log` in `.temp/g15-c2/c8/`; brief 722's
`base-all-records.json` (90 records) and `base-min-records.json` (12) are
the director-verified whole-record copies of them, and
`base-control-records.json` (470) equals the brief-721 accepted panel
`sample-all-records.json` whole-record (468 valid, two inherited invalid).

## The accepted mechanism

The exact copied experiment is `candidate-edits.json` and
`diffs/{collector.rs,mod.rs,profile_queue.rs}.patch` in
`.temp/brief-722/`. Install exactly:

```rust
// profiling/logical/collector.rs — the one new cold transfer.
#[cold]
#[inline(never)]
pub(in crate::runtime) fn take_evaluation_pass_boundary(
    epoch: &Arc<super::Epoch>,
    control: &Arc<crate::runtime::RequestInterruptState>,
) -> Option<Arc<ProfileBoundary>>;
// exactly one member of the process stash matching the exact epoch Arc,
// belongs_to_admitted (control identity AND logical generation), and the
// runtime key "evaluate.once_unstable"; zero matches leave the stash
// alone; more than one is a visible diagnostic fault that transfers
// nothing; it does not loop, choose newest, or drain a queue.

// profiling/logical/mod.rs — a runtime-private re-export only.

// suspension/core/profile_queue.rs — the enabled-only publisher and the
// existing Profiled EvaluationReturn push tail:
if consumer {
    true
} else if !self.return_inserted() {
    false
} else {
    self.publish_evaluation_pass_after_return()
}
```

The publisher finds the owner for its own root and epoch under the
existing registry lock, releases it, takes the selected boundary, uses
the existing charged `Context::capture` and `add` insertion at
`self.frontier()` with a non-deferred `Cell`, and returns `false` only on
capture/insertion failure (the caller's existing depofile path). The
consumer and no-arena early returns, the four other `ProfileQueue::push`
callers, both frame bodies, `resolution.rs`, the storage/driver/typed
carriers, `complete_evaluation_return` and the definition completion
constructor stay byte-identical. A change from this shape, even to fix
cost, is STOP.

Why it is the actual frontier: the queue order becomes
`EvaluationReturn, Boundary, consumer`; `Profiled::resume` observes the
boundary with the `Result<Slot, EvalError>` that `complete_evaluation_
return` produced (the resource check is applied) and only then dispatches
the fixed-point consumer frame. The selector is strictly narrower than
the existing enabled `take_belonging`/`drain_attached_for` identity
logic, and the closed inventory holds: `stash_boundary` has one
production caller, `pause_phase_at_finish` and `suspend_evaluation_
return` have exactly the two resolution helpers, and `register_owner`
keeps one registered queue per control.

## Scope

Live production edits, and nothing else:

- `crates/rum-runtime/src/runtime/profiling/logical/collector.rs`
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` (the one
  re-export line)
- `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs`
- one new crate-owned regression test module under
  `crates/rum-runtime/src/runtime/profiling/tests/` plus its one-line
  declaration in that directory's `mod.rs` (name and wiring disclosed in
  the report), using only existing test-cfg accessors such as
  `logical_profile_test_rows()`; if the test cannot be expressed without
  new diagnostic surface, STOP and report — do not widen it.
- canonical replay-generated cells in the affected standing inventories
  (retention fingerprints and both feature certificate cells, ownership
  graph, LP01 positions/hashes/bindings/losses, typed slots, rejection
  census, frame closed lists). Canonical generators only; no hand edit.

Everything else is STOP: `resolution.rs`, `resolution/resume_frames.rs`,
storage/driver/typed carriers, `evaluation_return` checks, the definition
completion constructor and its join helper, `Cargo.toml`/`Cargo.lock`,
docs, scripts, the `c2_probe`/attribution packages, and any other test
file. The three-file organization exception is accepted for this
insertion; do not stage a module split here.

## Stage A — copied failure-frontier proof (no live edit)

Work in a fresh byte copy of the live tree under `.temp/brief-723/`
(separate copy per arm and per mutant, private target dirs). Nothing
outside the scene is written.

1. **Reproduce first.** On the copied base, reproduce the 90 fixed-point
   records and the 12 minimized records whole-record (ten counters and
   language) against the frozen C8 references; reproduce the 470
   definition records against brief-722's `base-control-records.json`;
   run the wrong-field negative control. A rerun that does not reproduce
   them is a STOP with the raw log; never redefine a witness.
2. **Apply the accepted diff** to a candidate copy. Require 90/90 and
   12/12 SAME, the 84 non-DIFF records whole-record exact, the queued
   arms and language identical, both fresh witnesses SAME, the explicit
   nested witness SAME, the surrounding-Return records exact between arms
   (their two pre-existing `call.stage.prepare_arguments` differences
   are recorded, not repaired), and the 470 panel exact with its two
   invalid controls.
3. **Own-row omission mutants.** In separate copies, omit the publication
   for exactly one row's dynamic suspension scope. Require the
   probe-disabled arm's 104 records byte-identical to the clean
   candidate; P1470 to change exactly 20 records and P1471 exactly 14;
   every changed record to carry exactly one one-pass signature (Throw
   `[0,-1,1,0,0,0,1,0,0,0]`, Return `[0,-1,1,0,0,1,0,0,0,0]`) with the
   queued arm and language untouched; P1471 to leave the two-pause Throw
   green; and the own-row frontier-hit receipts to match the changed
   records. At most one new attempt per mutant.
4. **The unexecuted failure frontier.** Each item is a real executed
   probe with an actual producer and request — a source review is not
   evidence; a fabricated carrier, forged suspension or synthetic
   `InputRequired` is STOP.

   - **Resource-return rejection.** A real producer returns a value that
     the evaluation-return resource check rejects. Correct placement
     records the post-check rejection in the moved boundary, and its ten
     counters equal the queued arm's record for the same program. A
     deliberately misplaced publication (before the EvaluationReturn
     event) records the earlier success and must fail the same equality
     check at the same key. Language identical on every arm.
   - **Cancel / abandon / restart at both pauses.** Pause at the first
     and at the second fixed-point pause; cancel before the reply, drop
     the session, and abandon the queue. The relocated boundary is
     retired with the existing reason, charged bytes return to zero, no
     boundary or guest frame is lost, and cancellation still wins at the
     request boundary. A completed-resume/drop control stays green.
   - **Foreign / interleaved generation.** A foreign control's stashed
     boundary is never taken; a constructed duplicate match faults
     visibly and transfers nothing (both boundaries remain and are later
     published by their owners); interleaved requests on one epoch keep
     their own boundary.
   - **Missing owner and zero match.** Force a route with no registered
     owner for the queue, and a route with no matching stashed boundary.
     In both, nothing is transferred, the stash is intact, no new fault
     is raised, and the existing `publish_stashed_after_consumer` still
     publishes the boundary later — no boundary loss. (The known defect
     may remain on such a route; it must not become a new loss.)
   - **Allocation cap failure.** Exhaust the diagnostic arena at the
     publication. The existing `add` failure retires diagnostics, the
     guest result is identical to the no-diagnostics arm, and nothing
     panics.

5. **The permanent regression test.** Design and calibrate the test this
   brief installs. It must drive the actual two-pause and three-pause
   fixed-point producers in-process on both arms (queued and resumed),
   capture the `evaluate.once_unstable` ten counters through existing
   test accessors, assert whole-record and language equality, be red on
   the base copy and green on the candidate copy, fail under the
   stage-3 omission mutation, and pass at 8 and 1 threads. If the crate's
   in-tree facilities cannot express it, STOP and report; do not weaken
   the assertions or add surface.

6. Copied sanity only: `cargo fmt --check` and `cargo check
   --all-targets` on the candidate copy. The full suites run once in
   stage B on the live tree; do not run a workspace here.

## Stage B — live insertion and acceptance

1. **Live base snapshot** without git: hash the live files this brief
   touches and record the pre-edit tree for the red arms. On a copied
   base arm of the live source, reproduce the ten bound differences, the
   twelve minimized records and the 470 controls (stage-A receipts may be
   reused only if the copied source and fixtures are byte-identical).
2. **Install exactly the accepted shape** and the calibrated regression
   test. Then, before any acceptance suite, replay the canonical standing
   inventories and quote every verdict: retention fingerprints and both
   feature certificate cells, ownership graph, LP01 producer/hash/
   binding/loss positions, typed slots, rejection census, frame closed
   lists as applicable. A red solely from a stale derived cell is an
   ordering artifact and replay is the fix; changed established
   identities, dispositions or exclusions, a new `UnknownHostPayload`,
   unrelated new unknowns, or a changed rejection census are STOPs for
   the director.
3. **Acceptance.** `cargo fmt --check`; `cargo check --all-targets`;
   focused fixed-point / `queue_boundaries` / `retained_pause` /
   `profile_queue` / `resume_evaluation_context` / frame-root tests at 8
   and 1 threads, the new test red on the base copy and green on the
   live tree at both counts; `cargo test -p rum-runtime`; the new test's
   base-red receipt; and `cargo test --workspace` once (if the sandbox
   denies the native ZeroMQ loopback binds, STOP and report it as the
   standing sandbox boundary, not a product signal; the director runs the
   host workspace). Reproduce any behavioral red at 1 thread before
   calling it a defect.
4. **Copied native matrix on the final source.** Re-run the 90/12/2 and
   470 panels and both own-row mutants on a copy of the final live tree,
   with the accepted bars from stage A. Reused receipts must be
   byte-identical stages; never redefine a witness.
5. **The disabled cost gate.** Three serial Callgrind draws each of
   base, candidate and the forced same-source control, on both the
   fixed-point twelve-case workload and the accepted definition
   twenty-one-case workload, at one absolute path with one codegen unit,
   profiling off, offline and locked, one cargo process at a time.
   Required rows, exact: fixed-point panel
   `resume_eval_request_from_checkpoint` 58,208/26, the aliased F0/F1
   `TypedFrame::resume` 5,893/53, `resume_fixed_point_from` 5,158/33,
   initial suspend 5,733/63, resumed suspend 1,204/14, `freeze` 1,386/63,
   `complete_evaluation_return` 2,546/53; definition panel
   `resume_eval_request_from_checkpoint` 81,588/30, `resume` 6,663/63,
   `resume_fixed_point_from` 5,400/57, initial suspend 9,555/105,
   `freeze` 2,310/105, `complete_evaluation_return` 3,684/63. Every
   deterministic rum/rum-core row equal across the three arms and every
   named function's call count equal; both new helpers absent from every
   disabled capture; the forced rebuild sha equal to the candidate's; a
   `--force` spot check. A measurable default-path increase is STOP. The
   standing exclusions are restated in every table:
   `default_builtin_dispatch_spec_for_symbol` is draw noise and never
   cost evidence; the brief-620 association key-read
   `width4000` and its sibling are outside the timing lane; instruction
   rows decide. Symbol-less libc/ld address rows and shared std helpers
   are the instrument's layout/address noise and are recorded, not gated.
6. **The timing lane (secondary).** `scripts/core-language-microbench.sh`
   rum-vs-rum base/candidate with diagnostics OFF, all hot and same-tier
   labels, excluding the ruled association key-read labels, four passes
   with the layout control; per-label `timings.tsv` preserved. A
   separated hot or same-tier regression is STOP; instruction rows decide
   the cost verdict.
7. **The definition neighbour.** The 470-record panel stays whole-record
   equal to the accepted panel, including its two invalid controls; the
   brief-719 `down_values/default_fallback.rs` neighbour and the brief-721
   completion constructor with exactly two callers remain byte-identical.

## Hard rules

- No git in any form, no network; no writes outside `.temp/brief-723/`
  (and, in stage B, the approved live files plus the canonical replay
  outputs).
- Never modify `.temp/brief-722/`, `.temp/g15-c2/c7/`,
  `.temp/g15-c2/c8/` or `.temp/brief-623/`; never redefine a frozen
  witness.
- Rust 1.85.0, offline, locked; one cargo process at a time at eight
  build jobs; native diagnostics serial; Callgrind serial and alone;
  every command bounded with `scripts/run-bounded.py` (native twelve-case
  120 s, complete definition panel 600 s, builds 880 s with a 940 s
  outer bound, derived from brief-722's 65-68 s builds).
- Probe builds through `scripts/probe-build.py` with the complete
  assembled tree, one CGU, one absolute path, the content-addressed
  cache and one `--force` sha256 spot check; relocate its output paths
  inside the scene as brief-722 did; do not change optimization, the
  lockfile or the target shape independently.
- No `unsafe`, dependency change, public API change, foreign-engine
  comparison, fabricated carrier or suspension, generic stash drain,
  newest-wins selection, multi-boundary drain, weakened inventory or
  diagnostic, duplicated evaluator/frame body, or skipped/reordered
  resource check. No instrumentation on the disabled path.
- Every claim carries `path:line` or a raw receipt; quote the raw verdict
  lines. Two consecutive completed bounded experiments with no new
  evidence stop the round; failed launches and pure source reviews do not
  count.
- Preserve every completed receipt, including failures; no rerun of a
  consumed reference to reconstruct a lost log.

## Deliverable

A `rum-report/1` JSON at `.temp/brief-723/report.json` (written with
`scripts/report.py`, `check` run before finish) containing: the exact
installed diff and closed inventory; the stage-A reproduction, repair,
mutant and failure-frontier receipts; the permanent regression test and
its red/green calibration; the staged live acceptance with every replay
verdict; the disabled cost tables with three-draw ranges, the same-source
control and the exclusions; the timing-lane rows; the 470 definition
panel; the hash delta (touched files + canonical replay outputs); and
every raw log, fixture and capture left under `.temp/brief-723/` and
named.

## Verification of this round

- Stage A's 90/12/470 records reproduce whole-record from the frozen
  references; the candidate repairs all 18 named differences and keeps
  the 84 controls exact; both own-row omissions are caught with their
  predicted single-pass deltas; every failure-frontier probe is executed
  with raw evidence and, where negative, watched red first.
- The permanent regression test is red on base and green on the repaired
  copy, fails under the omission mutation, and passes at 8 and 1 threads.
- Stage B installs exactly the accepted shape; the canonical replays are
  quoted before acceptance; the focused and runtime suites are green at 8
  and 1; the workspace is run once (or stopped at the sandbox boundary
  with the raw log).
- The disabled cost rows and the 671-row comparison are exact at matched
  calls, both new helpers are absent, the forced control is sha-equal,
  and no deterministic rum row rises.
- The 470 definition panel stays exact; the timing lane shows no
  separated hot/same-tier regression.
- `report.py check` passes and every claim is cited.

Stop and report for the director's authoritative gate; do not commit,
tag or close anything.

## Director gate (2026-11-11) — the stage-A STOP is accepted; the restart probe is the worker's and the continuation completes stage A

Run 1 (session `01a1c674-3261-7a91-986d-91d2a0eab5a4`; log
`.temp/logs/cdx_brief_723.log`, 17,988 lines, sha256 `2436ed74…`,
memory-scope peak 3.05 GB, rc=0) reproduced the frozen panels on the
copies, applied the accepted diff, ran both own-row omission mutants and
the resource-return frontier, then STOPed at the lifecycle restart route
with no live edit. **The STOP is well taken:** stage A is incomplete, so
the stage-B prohibition holds; the tree is entry-identical, no inventory
moved, and the full suites never started.

**My own reproductions (74/74 checks, `.temp/director/gate-723/verify.py`):**
live tree 2,713 files exact (digest `8dd26c55…`), protected C7 25,251 /
C8 36,460 / brief-623 151 / brief-722 46,086 hash-equal, `git` clean,
`live-hash-delta.json` empty. All 90 fixed-point and all 12 minimized
records read from the frozen C8 `C2_NATIVE` raw logs are whole-record
equal to the round's base records (10 and 8 DIFF, each only
`evaluate.once_unstable`, language SAME; the frozen witnesses and the 12
minimized verdicts agree with my derivation, and the wrong-field negative
is rejected). The copied candidate is byte-identical to brief 722's
accepted candidate (three file hashes and all three patches), reads 0
DIFF over 90+12, keeps all 84 non-DIFF records and the 102 queued arms
and language exact, passes both fresh witnesses and the explicit nested
witness (base red → candidate green), preserves the two surrounding-Return
records and the invalid exploratory fixture, and keeps the 470 definition
records whole-record equal to brief-721's accepted panel (468 valid). The
two probe-disabled arms are byte-identical to the candidate; P1470 changes
exactly 20 records and P1471 exactly 14, each with one single-entry
one-pass Throw/Return delta and an untouched queued arm, and P1471 leaves
the two-pause Throw green. The resource probe uses a real producer whose
`TimeConstrained` deadline expires at the return check; both rows read
equal ten counters and language, and the separate before-event arm fails
the same-key equality in the predicted direction (the first, malformed
negative is disclosed and not used). First-pause cancel, drop and abandon
pass with zero charged bytes and the existing reasons. Artifact index 144
files hash-exact; `report.py check` 299 citations.

**The red is the probe, not the product.** `lifecycle_frontier.rs:42`
calls `eval_request("91")` while the session awaits host input; the
settled request-freshness contract at `session_requests.rs:105-108`
refuses it and says the host must resume or reset first (it has its own
tests, e.g. `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs:17`).
The brief's lifecycle item authorises cancel, drop and abandon; a
"restart" is the documented sequence, not a direct supersession, and
`SupersededOrQueuedInputDiscard` belongs to queued input driving
(`builtins_input.rs:234`), not to a new host request. No guard is
weakened and no production file moves.

**Ruled for the continuation (brief 723, forked from
`01a1c674-3261-7a91-986d-91d2a0eab5a4`):**

1. Keep every executed receipt; none is rerun to reconstruct a log.
   Re-run only what the corrected probes and the missing stage-A gates
   need.
2. Correct the restart arm to the documented lifecycle: first settle the
   pending request (cancel and drive the cancellation, or explicitly
   abandon the pending suspension), then start the fresh evaluation and
   let it complete. A direct `eval_request` while input is pending is the
   documented refusal, never a positive arm. Observe that the retired
   records settle under the chosen route's existing reason and that
   charged bytes return to zero.
3. Complete the failure frontier at both pauses: cancel, drop, abandon,
   restart and the completed-resume/drop control; foreign and interleaved
   generation; the duplicate-match visible fault that transfers nothing;
   missing owner and zero match (stash intact, existing
   `publish_stashed_after_consumer` still publishes later); and the
   diagnostic-arena allocation-cap failure with an unchanged guest
   result.
4. Complete the permanent regression test: drive the two-pause and
   three-pause producers in-process on both arms, assert the whole ten-
   counter records and language, red on the base copy, green on the
   candidate copy, red under the stage-3 omission mutation, and pass at
   8 and 1 threads. Do not weaken assertions; if the in-tree facilities
   cannot express it, STOP and report. Install it only after the copied
   gates pass.
5. Close the two executed-layout deviations: rebuild the base and the
   candidate with per-source private target directories and run the
   required one `--force` sha256 spot check, then re-read the panels on
   the fresh binaries and require the same record files. Copied
   `cargo fmt --check` and `cargo check --all-targets` follow.
6. Then stage B exactly as written: live base snapshot, the exact
   accepted shape plus the calibrated test, canonical replays quoted
   before any acceptance suite, `fmt`/`check --all-targets`, focused and
   runtime suites at 8/1, the workspace once (or the sandbox boundary
   with the raw log), the copied final native matrix, the 18-capture
   disabled cost gate with the exact rows, the timing lane, the 470
   definition panel, and the report.

Nothing else changes: the accepted mechanism, the three-file scope, the
hard rules and the stop-for-the-gate discipline stand. Next: append this
ruling, commit and push, archive the run-1 log as `cdx_brief_723.run1`,
and dispatch the continuation forked from the run-1 session.
