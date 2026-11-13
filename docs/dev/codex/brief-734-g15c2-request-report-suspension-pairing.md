# Brief 734 — G15-C2: design the published request report / suspension pairing (design only)

## Goal

Design, in a byte copy of the current live tree, the one-owner pairing of the
request driver's published report (`Session::pending_request`) and the
suspension it describes (`Session::pending_suspension`), so that a report can
never outlive its owner or be attached to a different suspension, and settle
the public contract of `resume_inspect_rewrite_rules_with_input` on a
request-created owner. The design must explain today's two measured defects
from source, recommend exactly one repair with exact types and signatures,
prove from source that after the repair brief 733's guard
(`pending_request.is_some()`) is an exact ownership predicate, and draft the
implementation brief. No live edit; STOP for the director's design gate.

## Context pointers

- Brief 733's class-duty STOP gate, the last section of
  `docs/dev/codex/brief-733-g15c2-published-owner-continuation-contexts.md`,
  and its `.temp/brief-733/report.json`, `class-route.rs`,
  `logs/class-route-t{8,1}-1.log`.
- `.temp/director/gate-733/panic-route.rs` — the director's extension of
  that route: request → raw inspection resume → request resume panics at
  `suspension/core.rs:646:18`, `resume requires a pending suspended
  evaluation`. Re-create and re-run it in your own copy; it links only the
  entry arm's rlibs and public APIs.
- Read before designing, under `crates/rum-runtime/src/runtime/`:
  `session/session_requests.rs`, `session/session_eval.rs`,
  `session/session_api.rs`, `session/session_helpers.rs`,
  `model/session.rs`, `suspension/core.rs`, `suspension/core/driver.rs`,
  `suspension/core/evaluation_context.rs`, `suspension/core/fault.rs`,
  `suspension/core/abandon.rs`, and the request-driver tests
  (`tests/session_requests/`, `tests/resume_evaluation_context.rs`,
  `tests/suspension_abandonment.rs`).
- Brief 731 continuation-1's ownership ruling: a request's own captured
  continuation is never stolen or abandoned by a nested entry; ownership is
  decidable from `Session::pending_request` without a new field. Brief 733
  and the accepted brief-732 design: the capture repairs the post-
  publication foreign capture and its guard is exactly
  `pending_request.is_some()`.
- The C2 campaign scene. Check `settled.tsv` and the residual lists for an
  existing row covering "a published report outliving its owner" or the
  inspection-resume composition. Never relabel an accepted row.

## The measured state — what the design must explain

Two public compositions of the current live tree (the working tree, which
carries the authorized uncommitted brief-731 stage-2 install):

1. `eval_request("Print[\"old-prefix\"];InputString[\"old-owner\"]")` →
   `resume_inspect_rewrite_rules_with_input("old-answer")` →
   `eval_text("InputString[\"new-owner\"]")` →
   `resume_eval_request_with_input("new-answer")` returns `"new-answer"`
   with `prints == ["old-prefix"]` at eight and one threads.
2. The same first two calls, then `resume_eval_request_with_input("late")`,
   panic at `suspension/core.rs:646:18`.

The report is stored by `finish_input_required_request`
(`session_requests.rs:506` → `store_pending_request:70`) and taken only by
the request-driver resume (`:293`) and cancellation (`take_pending_report:53`).
The raw resume `resume_suspended_slot` (`core.rs:635-714`) takes the slot,
clears the host-facing prompt (`:664`; `finish_completed_resumption`
`:607-624`), and never touches `pending_request`. A nested entry defers to
a stored report (`refuse_nested_top_level_input`, `session_eval.rs:40-51`).
The design must confirm or correct this account with its own measurements.

## Scope

Design only (`Scope: NONE`). Work only under `.temp/brief-734/`, in a fresh
complete byte copy of the live working tree at `.temp/brief-734/source-entry/`.
Record the complete entry manifest before any edit; every candidate arm gets
its own name and its own target directory. The live tree is dirty with the
authorized stage-2 install; copy the working tree, not a git revision.

MUST NOT change the live tree; `.git`; any other brief's evidence;
`Cargo.lock` or dependency manifests; the seven brief-731 refusal production
files; the permanent `crates/rum-stdlib/tests/nested_public_evaluation.rs`;
the five C4 owner records; any assertion, witness or inventory. No git in any
form; no network; no new dependency; no `unsafe`.

## What to do

1. **Entry copy and manifest.** Copy the live working tree; record every
   path's hash; verify by your own walk that the copy equals the live tree.
   Disclose every scratch addition.
2. **Reproduce both defects yourself** at eight and one threads, with
   drivers that link only the entry arm's rlibs and use only public APIs.
   Quote the exact tuples and the panic line; a red is reproduced at one
   thread before it is called a defect. Do not reuse brief 733's receipts as
   evidence; re-establish them in your own copy.
3. **Class inventory.** One table, one disposition per row, over:
   every write/read/take/clear of `Session::pending_request`,
   `Session::pending_suspension`, `Session::resuming_suspension` and
   `pending_input_request`; every suspension install, consume, abandonment,
   fault, reset, cancellation and snapshot restore; every request-driver
   entry (text, expression, script, file, resume, cancel, batch, restore).
   State for each whether the report and the slot are created/destroyed
   together, and name every path that can separate them. Include the
   `SuspensionGeneration`/`pending_suspension_generation` facts and what the
   report does and does not carry today.
4. **C2 ledger disposition.** Determine whether the defect already has a
   ledger row; if not, name the new obligation for the settlement round.
   Do not relabel any accepted row.
5. **Design the pairing.** Recommend exactly one mechanism with exact types,
   signatures and call sites, and evaluate at least two alternatives,
   explicitly including:
   (i) the report carries its owner's suspension generation and every
   consumption checks it (state what retires a stale report);
   (ii) any non-driver consumption of the slot retires the report (state
   every consumption route and the accounting/cleanup);
   (iii) the public inspection resume refuses a request-created owner with a
   typed error (name the error and the public surfaces).
   For each candidate, state its effect on: the documented host workflow
   (`eval_request`, `resume_eval_request_with_input`, cancel, reset, script
   and file batches, snapshot restore); silence versus a typed outcome;
   every public surface and any API break to name in `CHANGELOG.md`; the
   continuation-1 ownership rule; the entry's deferral rule; brief 733's
   guard; the permanent tests; the five C4 owner records; and the changed
   functions' instruction rows (the ordinary entry path must gain no work;
   measure the resume path with the standing identity lane).
6. **Calibrate in the byte copy.** For the recommended shape and for each
   discarded alternative that compiles: both routes' outcomes at eight and
   one; the permanent tests; the five C4 owner probes at eight and one with
   their `B731 C*` records byte-identical to
   `.temp/brief-731/continuation-4/logs/owner4-<case>-t<threads>-1.log`;
   the class table; the changed functions' disabled-path rows. Build one new
   permanent regression that is red on the entry copy and green on the
   candidate at eight and one, and keep every attempt, including failed
   ones. Release equality is behavior evidence, never the invariant proof.
7. **Prove the post-repair invariant from source.** After the recommended
   repair, whenever `pending_request.is_some()` a capture can only append to
   that report's own suspension (or the slot is vacant and no capture can
   occur), so brief 733's guard is exact. Enumerate the store/take/clear
   paths and show none can separate the pair. Any counterexample is a STOP
   or a design change, not a disposition by argument.
8. **Draft the implementation brief** in `TEMPLATE.md` shape: the install
   delta next to the uncommitted stage-2 install; the combined install plan
   for brief 731 continuation 6 (the seven-file stage-2 install + this
   pairing repair + brief 733's three-file capture repair + its accounting
   regression); the acceptance bar and the cost lane on that combined
   delta; the class duty; the stop-loss; its own canonical build cap. State
   which of brief 733's held receipts its continuation must re-establish on
   the repaired source.
9. **Report and STOP.**

## Hard rules

- Worker sandbox `workspace-write`; one Cargo at a time inside the 16-GiB
  memory scope; every invocation recorded; bounded wrappers (derive bounds
  from the recorded receipts of briefs 732/733: copied debug about 25 s,
  release about 55 s; 400 s for copied Cargo, 300 s for a canonical build).
- Canonical release builds go through `scripts/probe-build.py` at the stable
  active path with one release codegen unit and one forced `--force` SHA
  spot-check; at most **8** fresh builds for this design. Brief 733's 4 and
  brief 731's 8 remaining builds stay separate: no transfer, no borrowing.
  A step that cannot fit is a STOP.
- Any change to the five C4 records, the seven refusal files, the witness,
  a public surface (unnamed), depth or memory semantics, or the documented
  host contract is a STOP. Any need to change brief 733's accepted
  three-file shape is a Q, not a default.
- `cargo fmt --check` on the entry and candidate arms is expected to report
  exactly the three ruled brief-731 arms (`outcome.rs:696`,
  `model/requests.rs:53`, `request_errors.rs:133`; the fourth entry is the
  same `outcome.rs` through the `suspension_oracle` alias). Do not touch
  them; a fourth file or a diff inside your own touched files is a STOP.
- Every attempt is retained and reported; no fabricated state; no weakened
  assertion; no hand-edited inventory; no regenerated expectation.

## Deliverable

- The entry copy and manifest; both reproduced routes with their logs and
  tuples at eight and one.
- The complete class table with one disposition per row and every
  separation path named; the C2 ledger disposition.
- One recommended mechanism with exact types/signatures and at least two
  alternatives evaluated against the stated bars; the public contract of
  `resume_inspect_rewrite_rules_with_input` on a request-created owner.
- Calibration receipts for the recommended shape and compiled alternatives:
  both routes, permanent tests, the five C4 record comparisons, the
  changed-function instruction rows, and the new permanent regression's
  red/green calibration at eight and one.
- The post-repair invariant proof from source, and the TEMPLATE-shaped
  implementation brief draft with the combined install and cost plan.
- `rum-report/1` JSON at `.temp/brief-734/report.json`, every claim with a
  log path and a copied verdict line, including every failed attempt.

## Verification (paste verbatim lines)

```
[entry copy] both routes at eight and one: the panic line, the tuples
[class table] one disposition per row; every separation path named
[recommended shape] both routes at eight and one: the documented outcome
[calibration] permanent tests green at eight and one; the new regression red on entry, green on candidate
[release] the five C4 owner probes at eight and one, byte-exact records
[invariant] the post-repair guard premise proved from source, no counterexample
[draft] TEMPLATE-shaped implementation brief with the combined install and cost plan
```

A red is reproduced at `--test-threads=1` before it is called a defect.

## Stop-loss

Two consecutive compiling candidates that move neither route nor a named
measured counter (the panic, the tuple, a class-table disposition, a C4
record) are a STOP; a search for a new shape needs my ruling. Any need to
change an assertion, a C4 record, the documented host contract, the
accepted capture shape or the build budget is a STOP. Every attempt and
every consumed artifact is preserved and reported.

## Checkpointing — protect the scene

After each milestone update `.temp/brief-734/PROGRESS.md` with the completed
evidence, the exact next action, the copy state and the spent/remaining
budget. Keep every log, manifest, patch and driver under `.temp/brief-734/`;
never overwrite brief 733, brief 732, the C4 evidence or any frozen G15
scene.

## Report format

`rum-report/1` JSON at `.temp/brief-734/report.json`; every command and
attempt, verbatim verdicts, a test total for every executed suite, scope
hashes, the route tuples, the C4 comparison rows, the invariant proof
sketch, and any open decision. Close with
`python3 scripts/report.py summary 734` output and nothing else. Do not
commit; STOP for the director's design gate.

## Design STOP gate (2026-11-13) — the inventory scope is resolved as a copied-arm derived replay, the measured account is confirmed, the retirement-at-consume contract is selected, and continuation 1 finishes the proof and calibration

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1d16a-eb72-7070-9813-8602a36a498d`; log
`.temp/logs/cdx_brief_734.log`; 22 run records — 17 exit 0, the two
designed entry-regression reds at eight and one (exit 101), and three
retained class-audit preparation failures (exit 1); memory-scope peak
4,073,869,312 B with oom 0, oom_kill 0, rc 0; `report.py check 734` OK
with 177 citations by my own run; 0 of the 8 canonical design builds
used; no executed git or network and no `.git-attempts` file; 5,536
scratch additions all under `.temp/brief-734/`; no live source, inventory
or frozen-scene change).

**My scope gate.** My own full-tree SHA-256 walk (excluding `.git`,
`.temp`, `target`, the dot directories and `__pycache__`) reads 2,732
live files and 2,732 entry-copy files: 0 missing, 0 extra, 0 differing.
`git status` is still exactly the authorized brief-731 stage-2 install
(eleven modified paths and one untracked permanent test) and no live file
is newer than the dispatch. The candidate arm differs from entry in
exactly four changed files plus the new regression: the retirement
insertion in `suspension/core.rs` and the accepted capture-repair files
`core/evaluation_context.rs`, `core/queue.rs`, `core/storage.rs`, each
byte-equal to brief 732's `source-candidate-a` (three matching hashes).
The fingerprint review records 68 changed rows, `non_fingerprint_changes:
0`, `installed: false`. The live tree, the frozen scenes and every
earlier brief's evidence are untouched.

**My reproductions.** I ran the entry arm's public-route binary myself at
eight and one and read the same tuples:

```
B734 ROUTE replacement tuple=(value="new-answer",prints=["old-prefix"],fresh_error=NestedEvaluationInputBoundary,pending=None) report_owner_association=false
B734 ROUTE vacant tuple=(late=PANIC,message="resume requires a pending suspended evaluation",pending=None)
```

with the panic at `crates/rum-runtime/src/runtime/suspension/core.rs:646:18`.
The entry regression is red at both counts (0 passed / 1 failed) with its
three named violations: the vacant-slot panic, the changed raw-entry
outcome, and the cross-owner attachment. I also read the class table's
244 rows (217 source accesses, 27 lifecycle rows) and the ledger audit.
The supplied account of the mechanism is correct and now complete enough
to name every report mutator: `take_pending_report` (`session_requests.rs:53`)
and `store_pending_request` (`:70`), the driver's take-before-resume and
non-`Resumable` restore (`:283-311`), the successful/cancelled release
clear (`:595`), the abandonment clear after the abandon take
(`abandon.rs:9-20`, `fault.rs:116`), the rollback snapshot's captured
report with its generation match at `session_helpers.rs:239-260`
(`(None, Some)` abandons the trial owner, `(Some, Some)` asserts equal
generations, `(Some, None)` panics) and restore at `:299-306`, the driver
merge that takes the fresh nested owner and reinstalls the same
generation (`driver.rs:156-182`), `SuspensionSlot::install`'s
unconditional vacant-slot assert with checked generation
(`core.rs:269-285`), `take_for_resume` clearing only the host prompt
(`core.rs:307-310`), and the accounted resume take
(`core.rs:483-492`) that today never touches the report.

**Ruling.** Q1: the STOP is well taken as a question and is resolved as
follows. The brief's freeze covers authored receipts — assertions,
witnesses, expectations, the five C4 records and every frozen input
inventory. A canonical generator's own output is not an authored receipt:
the standing maintenance rule requires the replay before the first
acceptance suite, and brief 729's gate already ruled a derived refresh
the canonical replay consequence. Continuation 1 may install the 68
generated fingerprint cells **in the copied candidate arm only**, and
must prove the delta is fingerprint-cell only with no row added, removed,
reordered, or any identity, status, disposition or exclusion changed;
every generated row is recorded before and after; the entry copy's
inventory stays byte-identical to live; the live tree and every frozen
scene are never touched. A stale-cell red remains an ordering artifact
and is never a regression.

Q2 stands as the worker framed it: continuation 1 must measure the mixed
accepted-cancellation-ticket composition with public APIs and report the
exact outcome; a retirement that lets a ticket abandon an owner it does
not own, or leaves any report/owner mismatch, is a STOP for my ruling; a
clean outcome names the contract in the host doc and `CHANGELOG.md`. The
provisional retirement contract itself is selected now, so the
continuation calibrates it rather than re-opening the choice:

- **Mechanism.** The seven-line insertion in
  `take_accounted_pending_suspension_for_resume` (`core.rs:483`): after
  the slot take and before the helper returns, take `pending_request` and
  refresh `MemoryOwner::PendingRequest` when it was present. That helper
  is the single accounted consume authority for resume; the request
  driver has already taken its report before reaching it; abandonment,
  nested-cleanup abandonment and structural-fault unwind already clear
  the report before any guest code; the drive-merge moves the same fresh
  owner without a report. No new field, type, signature, enum variant,
  frame, generation counter or memory formula.
- **Why this mechanism and not the alternatives.** (i) A report-carried
  owner generation is insufficient alone: a consumption-time check leaves
  `pending_request.is_some()` true between the owner's take and the
  report's retirement, so brief 733's guard stays inexact and the rule
  reduces to retirement plus extra state, while needing the protected
  `model/requests.rs`. (iii) A typed refusal at the public inspection
  resume preserves the pair but closes only that entry, not the consume
  authority, and it does not fix the measured replacement composition:
  with the request owner still installed, the subsequent raw capture hits
  `SuspensionSlot::install`'s unconditional assert — a public panic —
  unless a second public refusal (raw-entry overlap) is added, which is a
  different class and an over-generalized scope. Retirement closes the
  class at the take, leaves the driver untouched and changes no public
  type.
- **The settled contract.** An explicit raw resume
  (`Session::resume_inspect_rewrite_rules_with_input`) consumes the
  current suspension, including a request-created one, and retires that
  request's report at the consume, before any callback. The resumed value
  decodes exactly as today; the request's already-published prefix output
  stands; the completion output is not returned to the request protocol,
  which is the existing raw-API behavior and must be measured, not
  assumed, with armed accounting. A later
  `resume_eval_request_with_input` takes no `Resumable` report and returns
  the existing `SessionRequestError::Other`
  (`no suspended evaluation is waiting for host input`) with empty
  messages, prints and trace; a raw owner created afterwards keeps its
  prompt and stays resumable by the raw API; a fresh request then
  completes. A `Batch` report is retired by the same explicit raw consume;
  the request driver still refuses to resume a batch. No signature, wire
  or `EvalError` variant changes; the observable mixed-driver change is
  named in `docs/host-embedding-and-hardening.md` and `CHANGELOG.md`
  before S10.

**Continuation 1 order.** Design only (`Scope: NONE`) in the same scene
`.temp/brief-734/`; round-1 arms and receipts are preserved, and new arms
get new names. The remaining bar:

1. **Derived replay.** Install the generator's 68 fingerprint cells into
   the candidate copy's `local-contracts.tsv` under the Q1 authorization;
   prove 68 fingerprint-cell changes, 0 non-fingerprint changes, no row
   added/removed/reordered, no identity/status/disposition/exclusion
   change; record every changed row before and after and the generated
   file's hash; the entry copy's inventory stays byte-identical to live.
2. **Candidate calibration at eight and one.** Both public routes with
   the exact candidate tuples: the vacant route returns the settled
   `Other` failure with empty output and no panic; the replacement route
   returns raw `EvalError::InputRequired { prompt: "new-owner" }`, then
   the same `Other` failure, preserves the new owner and prompt, resumes
   it through the raw API, and a fresh request returns 42. The new
   permanent regression green on the candidate and red on a freshly
   re-established entry arm. The unchanged permanent
   `nested_public_evaluation` 15/15 at eight and one ordinary and
   profiled on the combined candidate (pairing plus the accepted
   capture repair). The runtime filters `resume_evaluation_context`,
   `session_requests`, `suspension_abandonment`,
   `session_memory_request_owners`, `frame_root_trace`, `queue_boundaries`,
   the inspection-resume control, the batch-report case, and brief 733's
   seven named host and nine named stdlib suites at eight and one; a
   skipped or zero-test run is not evidence. Armed accounting for the
   retired report (its `PendingRequest` charge released; the completion
   output's retention is exactly the measured raw-API behavior).
   `cargo check --all-targets` and `cargo fmt --check` with exactly the
   three ruled brief-731 arms red.
3. **Release equality.** Build the release candidate arm through
   `scripts/probe-build.py` at the stable active path with one release
   codegen unit; run the five C4 owner probes at eight and one and
   compare the `B731 C*` records byte-for-byte with
   `.temp/brief-731/continuation-4/logs/owner4-<case>-t<threads>-1.log`;
   slice only C4's existing libtest marker framing; one forced same-source
   SHA. Any changed record is a STOP.
4. **Cost rows.** The standing identity lane on the combined candidate
   against the live entry: the ordinary entry row at most 599 self Ir at
   four calls in both draws, no `rum_runtime` equal-call increase anywhere
   in either draw, the changed functions' rows recorded, two draws at
   N=10/20, one codegen unit, serial, profiling off. A paired resume panel
   with fixed guest statements and fresh sessions per iteration — request
   pause then request resume, and request pause then inspection resume —
   with no guest sink, print, assignment or wrapper added to the targets.
   Every positive row is recorded, never normalized or offset.
5. **The post-repair invariant proof from source.** Enumerate every store,
   take, clear and restore of `pending_request` and every slot install,
   take, merge, restore, abandonment and fault, with one disposition
   each, and prove the induction: a published report implies the slot
   holds that report's own suspension (same generation), including the
   transient windows (the take-to-clear gap, the install-to-store gap),
   which must contain no guest code or capture. State precisely when
   `refuse_nested_top_level_input`'s report branch is reachable and prove
   that in each such case the report owns the slot; prove the capture
   repair's `pending_request.is_some()` guard is exact whenever a foreign
   capture appends to a published owner's suspension; confirm the driver
   merge, the rollback generation checks, the abandonment clears and
   `release_request_session_output`'s clear cannot separate the pair. A
   counterexample is a STOP or a design change, never a prose disposition.
6. **Class-table completion.** Give one disposition to every composition
   round 1 left open: the overlapping raw entry that captures while a
   request is published (`eval_text` / `eval_script` / `eval_file` →
   `SuspensionSlot::install`'s unconditional assert), the re-entrant
   native-callback compositions during a resume, the rollback
   `(Some, None)` panic, the `resume requires a pending suspended
   evaluation` expect, and the `Batch` report. In-class repaired /
   out-of-class named obligation with the exact public composition and
   assert line, registered for the C2 settlement round / unreachable with
   proof. The pairing repair is not expanded to repair an out-of-class
   row without my ruling.
7. **Q2 measurement.** The accepted-ticket mixed route with public APIs
   only: install an accepted ticket, publish a request, consume it with
   the raw inspection resume, then exercise cancellation; report the
   exact behavior of the ticket, the retired report, the later raw owner
   and `finish_cancelled_pending_evaluation_request`. Any owner/report
   mismatch or a ticket abandoning an owner it does not own is a STOP.
8. **Draft completion.** Rewrite the implementation-brief draft in
   `TEMPLATE.md` shape with the measured gates and the combined topology
   this line now takes: the pairing install is its own staged live brief
   (one production file, the permanent regression, the host doc and
   `CHANGELOG.md`) on the working tree that already carries the
   authorized stage-2 install; brief 733's continuation then re-establishes
   its receipts on that repaired tree; brief 731 continuation 6 then
   installs the three-file capture repair and the accounting regression
   on top and runs the standing replay first, acceptance once, and the
   combined cost lane on the final source with the unchanged 599/4 entry
   bar and the no-increase scan. Propose the implementation brief's own
   canonical cap; I ratify it at that gate. State the class duty and the
   stop-loss.

**Hard rules for continuation 1.** Same as this brief, plus: the Q1
authorization covers only the canonical generator's own fingerprint cells
in the copied candidate arm — never a hand edit, never an authored
inventory, never the live tree; the live tree stays exactly the
authorized stage-2 install; no git, network, dependency or `unsafe`; the
seven refusal files, the permanent witness, the five C4 records and every
assertion stay byte-identical; at most **4** of the remaining **8**
canonical design builds (brief 733's held 4 and brief 731's remaining 8
stay separate), and a step that cannot fit is a STOP.

**Verification (paste verbatim lines).**

```
[derived] 68 fingerprint cells installed in the copy; 0 non-fingerprint changes; 0 added/removed
[candidate] both routes at 8 and 1: the settled tuples, no panic
[candidate] the new regression green at 8 and 1; the entry red re-established
[candidate] nested_public_evaluation 15/15 at 8 and 1, ordinary and profiled
[candidate] runtime filters / host suites / stdlib suites at 8 and 1; armed accounting
[release] the five C4 probes at 8 and 1, byte-exact records; one forced SHA
[cost] the entry row at most 599/4 both draws; no equal-call rum_runtime increase; paired resume rows
[invariant] the report-slot pairing proved from source; both guards exact; no counterexample
[draft] TEMPLATE-shaped implementation brief with the combined install and cost plan
```

**Stop-loss.** Any invariant counterexample, any candidate route outcome
that differs from the settled contract, any C4 record change, any fmt
diff outside the three ruled arms, or any need to change an assertion, a
witness, a protected file, the documented host contract or the budget is
a STOP for my ruling. Two consecutive compiling candidates that move
neither route nor a named measured counter are a STOP. Every attempt is
preserved and reported.

**Report format.** `rum-report/1` JSON at `.temp/brief-734/report.json`
(a continuation report under the same brief; round-1 evidence is
preserved, not overwritten); every command and attempt, verbatim
verdicts, test totals, the replay rows, the route tuples, the C4
comparison rows, the cost rows, the invariant proof and any open
decision. Close with `python3 scripts/report.py summary 734` output and
nothing else. Do not commit; STOP for my design gate.

**Next.** Continuation 1 of this brief; then brief 735 (the pairing
install, staged live on the working tree); then brief 733's continuation
on that repaired tree; then brief 731 continuation 6; then continuation
38 of brief 623; then C2's settlement round, CM, `CHANGELOG.md` before
S10 (including this lifecycle contract), G13 S10-S54/F1-F23/R, E2.

## Continuation-1 gate (2026-11-13)

**Run.** `cdx` (gpt-6-astra, max; session
`01a1d18e-8d82-74f1-ae2b-4774a0dccf8f`, fork of
`01a1d16a-eb72-7070-9813-8602a36a498d`; log
`.temp/logs/cdx_brief_734.log`, 25,173 lines, 1,644,824 B, sha256
`8e728cbd…`; 18 continuation run records — 13 exit 0, the two designed
entry-regression reds and the two mandatory Q2 reds at eight and one
(exit 101), one designed wrapper exit 1 (the Q2 panel's STOP); memory-scope
peak 3,275,902,976 B with oom 0, oom_kill 0, rc 0; `report.py check 734`
OK with 298 citations by my own run; 0 of 4 continuation canonical builds,
0 of 8 overall; no executed git or network, no `.git-attempts`; 37,303
scratch files all under `.temp/brief-734/continuation-1`; the round-1
`report.json` and `PROGRESS.md` preserved byte-exact under
`continuation-1/round1/`; no live source, protected file or earlier scene
changed).

**My scope gate.** My own full-tree SHA-256 walk reads 2,732 live files,
2,732 entry-copy files and 2,733 candidate files: live equals the
continuation's `live-entry-hashes.json` exactly (0 missing, 0 extra, 0
differing), the entry copy equals live exactly, and the candidate differs
from entry in exactly five changed paths plus one added file —
`suspension/core.rs`, the three capture-repair files byte-equal to round
1's candidate, the canonical inventory, and the new regression. The
inventory delta is exactly 68 of 69 lines, every one differing only in its
final fingerprint field, with no added, removed or reordered row, and the
installed file is byte-equal to the canonical generator's output (sha256
`c5ceb5eb…`); the entry inventory is byte-equal to live. The nine
immutable pins match live; `git status` is still exactly the authorized
eleven-file stage-2 install plus the one untracked witness; HEAD is
`b7e13f27`; every candidate source file predates the receipted builds. The
report's 5,518 added paths are the copies and the continuation's own
records, and its two modified paths are the scene's `report.json` and
`PROGRESS.md`, whose round-1 originals are preserved; no live path
appears in the change set.

**My reproductions.** I ran the receipted binaries myself. The entry
regression is red 0/1 at eight and one with the same three violations and
the panic at `suspension/core.rs:646:18`. The Q2 route is red 0/1 at both
counts: report retired, late request resume returns the existing `Other`
with empty output, the later raw `InputString["new-owner"]` is pending
with the old ticket still reserved, and the old handle's cancellation
returns `TerminatedEvaluation["Cancelled"]`, `termination=Cancelled`,
`prints=["new-cleanup"]`, `General::cancel`, `pending=None`,
`observer_cancelled=false`, then 42 and a free ticket. The candidate
`request_report_pairing` is green 1/1 and the unchanged permanent
`nested_public_evaluation` 15/15, at eight and one, on the receipted
test binaries; the ordinary and profiled runs are the round's logs.

**The STOP, confirmed.** The accepted ticket outlives its request on the
raw consume path. The consume helper takes the report but never touches
`active_accepted_request`; the ticket is retired only by the driver's
terminal paths (`session_requests.rs:352`, `:629`, `:650`, `:690`, the
cancellation hand-off, and `session_api.rs:103`).
`finish_cancelled_pending_evaluation_request` (`session_requests.rs:78-98`)
qualifies a cancellation on `active_accepted_request_is_cancelled()` plus
any `pending_suspension_generation()` and never on an ownership
association, so once the old owner is consumed and its report retired the
old ticket can abandon any later raw owner, and
`RequestCancelHandle::cancel`'s own documentation
(`model/request_cancellation.rs:118-121`: calls after that request
completes are inert) is false. The ticket machinery is byte-identical on
the entry arm — the candidate's four production files are the consume
helper and the three capture files, and `session_requests.rs`,
`session_helpers.rs`, `session_api.rs` and `request_cancellation.rs`
are unchanged — so the abandonment is pre-existing, not introduced by the
retirement; the pairing only removes the stale prints it would have
carried. The worker's refusal to attribute it is correct, and the round
correctly stopped rather than editing a candidate past this bar.

**Ruling.** The accepted-ticket lifetime is in the same class and at the
same consume authority as the report. Two changes, both minimal and both
required before brief 735 can be written:

1. **Retire the ticket at the consume.** Inside the same
   `if self.pending_request.take().is_some()` block in
   `take_accounted_pending_suspension_for_resume`, before any callback,
   call `self.finish_active_accepted_request();` beside the
   `PendingRequest` refresh. No new field, type, signature, enum variant,
   frame, counter or memory formula. The request driver has already taken
   its report before the helper, so the driver path never executes the
   call and its terminal ticket handling is unchanged; the take, the
   refresh and the finish are the only statements added. The contract it
   creates: an explicit raw resume of a request-owned suspension ends that
   request — the handle becomes inert, the cancellation observer clears,
   any unhonoured cancel is closed by the consume, the raw resume's
   evaluation runs without request cancellation (the eval deadline still
   applies), the request protocol no longer resumes it (the existing
   `Other` with empty output), and a new cancellable request can be
   accepted. A cancelled ticket whose own request is completed by a raw
   resume therefore yields the raw result, not a `Cancelled` response,
   with the observer cleared. This is the raw-API "host took over"
   behaviour, to be named in the host doc and `CHANGELOG.md`.

2. **A reservation acts only on request-owned work.** The deferred
   reservation (`pending_accepted_request`) may be activated by the
   cancellation hand-off only when the session has no report-less pending
   owner: activation is permitted when a request-owned suspension is
   pending (`pending_request.is_some()`, exact after the pairing) or when
   nothing is pending, and refused while a report-less owner (a raw
   `eval_text`/`eval_script`/`eval_file` suspension) is installed. The
   frozen `cancelled_request_retires_a_non_request_suspension` semantics —
   an installed accepted request whose entry is admitting it retires the
   session's pending work, including a report-less owner — is unchanged,
   and that test's bytes stay identical. The measured consequence to be
   proven: a reservation can never abandon a raw owner.

The single-owner principle is now exact: the report is the ownership
certificate, the ticket is the request's identity, and both are consumed
with the suspension they belong to. The remaining alternatives stay
rejected for the reasons already ruled; a generation field on the ticket
is not required because the consume authority and the report predicate
already identify the association.

**Continuation 2 order.** Design only (`Scope: NONE`), in the same scene;
round-1 and continuation-1 arms, receipts and reds are preserved, and new
arms get new names.

1. Implement both changes in the copied candidate and rebuild. Prove from
   source that the driver path executes no new instruction (the branch is
   taken only by the raw consume) and that the reservation gate is not
   reached by any request entry that has a report.
2. Measure at eight and one, pre-repair and post-repair: the
   completed-ticket route above (inert handle, cleared observer, later
   raw owner intact and resumable through the raw API, fresh request 42,
   new ticket accepted); the vacant route (`Ok(None)`, ticket free,
   observer false); the cancelled-own-request raw-resume route (raw
   result, observer cleared, the next plain request not spuriously
   `Cancelled`); the deferred-reservation route (raw owner intact — the
   pre-repair red is the abandonment); the deferred-reservation route
   with a request-owned suspension (cancellation unchanged: `Cancelled`
   with the published prefix); the frozen at-entry retirement, unchanged
   and green; the normal request pause/resume and cancellation controls
   unchanged. Pre-repair reds are the calibrated falsifiers; no assertion
   is weakened and no existing test is edited.
3. Prove the ticket induction from source: every store, take, finish and
   activation site with one disposition; the two activation callers; the
   active/deferred split; the report predicate exact at every hand-off;
   every transient window covered with no guest code between take and
   retirement; no counterexample. An unproven row is a STOP, never an
   assertion.
4. Complete the report/suspension induction including the ticket, with
   both guards exact, and dispose of the class-table rows owed from
   continuation 1: the overlapping raw-entry install assert, re-entrant
   callbacks, the rollback `(Some, None)` panic, the resume `expect`, and
   the `Batch` report.
5. Then run the still-held calibration gates of continuation 1: the two
   routes at eight and one with the settled tuples, the new regressions at
   eight and one, `nested_public_evaluation` 15/15 ordinary and profiled,
   the named runtime filters, the seven host and nine stdlib suites,
   armed report accounting, `cargo check --all-targets` and `cargo fmt
   --check` with only the three ruled brief-731 arms red, the five C4
   release comparisons with one forced SHA, and the cost rows (entry at
   most 599/4 both draws, no equal-call `rum_runtime` increase, the paired
   resume panel). No credit from unrun gates.
6. Rewrite the TEMPLATE-shaped implementation brief with the settled
   topology — brief 735 installs the pairing and both ticket changes on
   the working tree that carries the stage-2 install (production files
   `suspension/core.rs` and, if the design requires it,
   `session_requests.rs`; the permanent regressions; the host doc;
   `CHANGELOG.md`), then brief 733's continuation on that repaired tree,
   then brief 731 continuation 6. State the class duty and the stop-loss.

**Hard rules for continuation 2.** Same as this brief. The Q1
authorization covers only the canonical generator's own fingerprint cells
in the copied candidate arm. At most **4** of the remaining **8**
canonical design builds; brief 733's held 4 and brief 731's remaining 8
stay separate, and a step that cannot fit is a STOP. No git, network,
dependency or `unsafe`; the live tree stays exactly the authorized
stage-2 install; the seven refusal files, the permanent witness, the five
C4 records and every assertion stay byte-identical.

**Stop-loss.** Any ownership counterexample, any change to a frozen test,
a documented host flow, a C4 record or a protected file, any route outcome
that differs from the contract above, and any need to weaken an
assertion is a STOP for my ruling. Two consecutive compiling candidates
that move neither the Q2 panel nor a named measured counter are a STOP.

**Next.** Continuation 2 of this brief; then my design gate and brief 735;
then brief 733's continuation; then brief 731 continuation 6; then
continuation 38 of brief 623; then C2's settlement round, CM,
`CHANGELOG.md` before S10 (including this lifecycle contract), G13
S10-S54/F1-F23/R, E2.

## Continuation-2 gate (2026-11-13)

**Run.** `cdx` (gpt-6-astra, max; session
`01a1d1a6-d16e-75a1-bfc5-ce949aa0eb4e`, fork of
`01a1d18e-8d82-74f1-ae2b-4774a0dccf8f`; log `.temp/logs/cdx_brief_734.log`,
10,959 lines, 741,005 B, sha256 `5c94a12f…`; 27 continuation-2 run
records — 20 exit 0, the six designed exit-101 reds (both ticket-panel
pre runs; both idle-activation runs on both arms), one designed wrapper
exit 1; memory-scope peak 5,139,734,528 B with oom 0, oom_kill 0, rc 0;
`report.py check 734` OK with 449 citations by my own run; 0 of 4
continuation canonical builds, 0 of 8 overall; no executed git or
network, no `.git-attempts`; every write under
`.temp/brief-734/continuation-2`; live source, the seven refusal files,
the permanent witness, the five C4 records and every earlier scene
unchanged).

**My scope gate.** My own full-tree walk reads 2,732 live files = the
scene's `live-entry-hashes.json` exactly (0/0/0), and the entry copy =
live exactly; the pre arm = entry + five changed files + the C1 pairing
regression; the candidate = entry + six changed + two added; candidate
minus pre = exactly `session/session_requests.rs`,
`suspension/core.rs`, the canonical inventory and the added
`crates/rum-stdlib/tests/request_ticket_pairing.rs`. The C1 pairing
regression and the three capture files are byte-equal to the C1
candidate; the inventory changes 68 of 69 rows, every one
final-fingerprint-field only, no row added/removed/reordered, and the
installed file is the generator's own output (sha256 `415059c2…`); the
nine immutable pins match live; the frozen
`cancelled_request_retires_a_non_request_suspension` file is
byte-identical on live and on all three arms; the ticket regression
equals the scene copy (280 lines); every change is under `.temp`, no live
path changed. Gate script: `.temp/director/gate-734c2/verify.py`.

**My reproductions.** The pre-repair arm is 4/4 failed at eight and one
(the four calibrated reds: completed-ticket/later-raw,
completed-ticket/vacant, unhonoured cancel before raw takeover, deferred
reservation after a raw owner); the candidate is 8/8 at eight and one;
the frozen test is 1/1 on both arms at both counts; and the
idle-activation route is 0/1 on both arms at both counts with the exact
tuple — the idle hand-off returns `Ok(None)` after promoting the
reservation, the later raw `eval_text` pauses at `later-raw-owner`, the
reserved handle's cancel is observed, and the second hand-off returns
`TerminatedEvaluation["Cancelled"]`, `prints=["later-raw-cleanup"]`, the
`General::cancel` message, `pending=None`, `raw_intact=false`.

**The STOP is well taken.** The continuation-2 gate admitted activation
when the session had no *report-less* pending owner; that is vacuously
true at an idle call, so the hand-off still promoted a reservation with
no request entry, and the promoted ticket later owned and abandoned raw
work. Reading the source: `session_requests.rs:81` is the only activation
outside `install_accepted_request`'s four explicit install sites, and
every public caller of the hand-off reaches it (`eval_request`,
`eval_expr_request`, `eval_script_request` and `eval_file_request` call
it first; the host's `call_unprotected` calls it directly). The unproved
implication the worker named is exact: an active ticket at a hand-off
either owns the current slot or was installed by the request entry
admitting it. The defect is pre-existing relative to both ticket changes
(both arms red).

**Ruling — change 3: the deferred reservation is an entry-owned
admission.** Two edits in one production file; no new field, type,
signature, variant, frame or counter:

1. In `finish_cancelled_pending_evaluation_request`
   (`session_requests.rs:78-98`), replace the continuation-2 two-branch
   gate with the single report-gated activation:

   ```rust
   if self.pending_request.is_some() {
       self.activate_pending_accepted_request()?;
   }
   ```

   The stored report is the ownership certificate: only a request-owned
   owner may be retired by a reservation. This is the documented host
   flow — `cancel_handle` targets an input-suspended request, and the
   script contract's "cancel the batch's request handle and re-enter to
   report cancellation and abandon it" — and an idle or report-less call
   never promotes. The cancelled check, the abandonment and the finish
   after it are byte-unchanged.

2. In each of the four admitting entries, immediately after
   `self.reject_overlapping_evaluation_request()?;`
   (`eval_request:187`, `eval_expr_request:216`, `eval_script_request:390`,
   `eval_file_request:428`), insert
   `self.activate_pending_accepted_request()?;`.

   The order is load-bearing: the overlap rejection has already refused
   every suspended session, so a promoted ticket can only own the request
   this entry is about to run. A cancelled reservation is then reported
   by the existing `finish_cancelled_active_request`
   (text/expression) or finished by the same completion the entry's own
   ticket gets (script/file); an uncancelled one is that request's
   ticket and is finished by the existing terminal paths. Activation must
   not move to the top of an entry or into the hand-off's idle path —
   either ordering recreates this counterexample.

Do not change the frozen test's bytes, the eight existing panel cases,
the hand-off's cancelled/abandon path, or the entry order (hand-off,
overlap, activation). **Contract:** a deferred reservation can never
touch a report-less owner; it is consumed by the next admitted request
(`cancel_handle`'s "reserves the next evaluation request") or by a
cancelled hand-off on a request-owned suspension; after that request
completes, calls on the handle are inert. Name it in the host doc and
`CHANGELOG.md` together with the earlier two changes.

**My calibration (measured, not gated).** A byte copy of the C2 candidate
with exactly this shape (`.temp/director/probe-734c3/`; one scope, 8/1):
the 8-case panel 8/8 at eight and one; the idle route 1/1 at eight and
one; four added compositions green at eight and one (an idle reservation
cancels the next request and then goes inert; an uncancelled idle
reservation runs the next request; a refused entry leaves the reservation
inert with the raw owner intact and the next admitted request cancelled;
a deferred reservation blocks a second accepted identity); the frozen
at-entry test 1/1 at eight and one; the whole `rum-host` suite green at
eight, with the three switch-gated targets green again under
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` at eight and one — including
`suspension_abandonment_contract` 12/12, `suspended_timeout_contract`
13/13, `retirement_interrupt` 9/9 and `lifecycle_contract`'s six cancel
tests. This is evidence for the ruling, not a receipt for the
continuation.

**Continuation 3 order** (design only, same scene, at most 4 of the
remaining 8 design builds):

1. In a new copied candidate (new arm name; the C1 and C2 arms, reds,
   binaries and receipts preserved), implement change 3 exactly as ruled
   on top of the C2 candidate. No other production edit; the three
   capture files, the C1 pairing regression and the frozen test stay
   byte-identical.
2. Promote the idle-activation diagnostic into the permanent panel as its
   ninth case, using the panel's own helpers, asserting in order: the
   first hand-off is `Ok(None)` and leaves the reservation unpromoted
   (control non-idle, no pending input, observer false); the later raw
   `WithCleanup[InputString["later-raw-owner"],Print["later-raw-cleanup"]]`
   pauses with that prompt; cancelling the reserved handle sets the
   observer; the second hand-off is `Ok(None)` with the cancel message
   absent, the raw owner intact and raw-resumable; the next admitted
   `eval_request` returns `Cancelled` (the reservation is honoured, not
   lost); a fresh request returns 42 with a free ticket and a clear
   observer. The case must be red on the entry arm and on the C2 arm
   (0/1 at eight and one) and green on the new candidate; the eight
   existing cases stay green on the new candidate.
3. Re-run the frozen at-entry test on the new candidate at eight and one
   (byte-identical, 1/1), the named host cancel-contract suites, and the
   three-arm ladder at eight and one: entry arm 5 failed of 9, C2 arm 1
   failed of 9, new arm 0 failed of 9.
4. Complete the ticket induction from source with activation as a
   two-kind, five-site duty: the report-gated hand-off and the four
   post-reject entries. Prove each activation installs the ticket into a
   session that immediately consumes it (the hand-off only with a
   request-owned owner; the entry only into an empty session), and that
   no active ticket can coexist with a report-less owner outside the
   frozen at-entry installation and the pre-existing out-of-class
   overlapping raw-entry assert; give every store/take/finish/restore/
   reset site one disposition; state every transient window; no
   counterexample. Then complete the report/suspension induction and
   dispose the owed class rows (overlapping raw-entry install assert,
   re-entrant callbacks, rollback `(Some, None)`, the resume `expect`,
   the `Batch` report). An unproven row is a STOP, never an assertion.
5. Then the still-held continuation-1 gates, unchanged: the two routes at
   eight and one with their settled tuples, both regressions, permanent
   `nested_public_evaluation` 15/15 ordinary and profiled, the named
   runtime filters, the seven host and nine stdlib suites, armed report
   accounting, `cargo check --all-targets` and `cargo fmt --check` with
   only the three ruled brief-731 arms red, the five C4 release
   comparisons with one forced SHA, and the cost rows (entry at most
   599/4 both draws, no equal-call `rum_runtime` increase, the paired
   resume panel). No credit from unrun gates.
6. Rewrite the TEMPLATE-shaped implementation brief for 735 with the
   settled three-change topology — the report retirement and the ticket
   finish at the consume authority (`suspension/core.rs`), the
   reservation activation ownership (`session/session_requests.rs`),
   both permanent regressions, the host doc and `CHANGELOG.md`, on the
   working tree that carries the stage-2 install — then brief 733's
   continuation on that repaired tree, then brief 731 continuation 6.
   State the class duty and the stop-loss.

**Hard rules for continuation 3.** Same as this brief. At most **4** of
the remaining **8** canonical design builds; brief 733's held 4 and brief
731's remaining 8 stay separate, and a step that cannot fit is a STOP.
**Stop-loss.** Any ownership counterexample, any change to the frozen
test, a documented host flow, a C4 record or a protected file, any route
outcome differing from the contract above, or any need to weaken an
assertion is a STOP for my ruling. Two consecutive compiling candidates
that move neither the panel nor a named measured counter are a STOP.

**Q3, answered.** A deferred reservation may be promoted only by the
report-gated cancellation hand-off (a request-owned owner) or by an
admitting request entry after its overlap rejection; no idle call and no
report-less owner may ever promote it.

**Next.** Continuation 3 of this brief; then my design gate and brief
735; then brief 733's continuation; then brief 731 continuation 6; then
continuation 38 of brief 623; then C2's settlement round, CM,
`CHANGELOG.md` before S10 (including this lifecycle contract), G13
S10-S54/F1-F23/R, E2.

## Continuation-3 gate (2026-11-13)

**Run.** `cdx` (gpt-6-astra, max; session forked from
`01a1d1a6-d16e-75a1-bfc5-ce949aa0eb4e`; log `.temp/logs/cdx_brief_734.log`,
14,047 lines, 999,955 B, sha256 `1c1266f7…`; 50 run records — 32 exit 0
and the 15 designed exit-101 reds (entry/C2 panels at 8 and 1, the four
ninth-case baselines, the six queued-input probes, the initial profiled
host run), plus three retained exit-1 wrapper/preparation refusals;
memory-scope peak 4,370,878,464 B with oom 0, oom_kill 0, rc 0;
`report.py check 734` OK with 810 citations by my own run; 0 of 4
continuation canonical builds, 0 of 8 overall; no executed git or
network; every write under `.temp/brief-734/continuation-3`; live source,
protected files and earlier scenes unchanged).

**My scope gate** (`gate-734c3/verify.py`): my own full-tree walk reads
2,732 live files equal to the scene's `live-entry-hashes.json` (0 missing,
0 extra, 0 differing); the entry copy equals live exactly; the C2 control
equals the C2 arm on every path except the appended brief, and equals
entry plus exactly the C2 candidate's eight paths; the change-3 delta
against the C2 control is exactly `session/session_requests.rs`
(`bf0a90…` → `c1257a…`) and the panel (`592f67…` → `97ad28…`); the replay
delta is exactly the canonical `local-contracts.tsv` with 68
fingerprint-field-only rows; the panel is the C2 panel's 10,016 bytes plus
the ninth case verbatim; the eleven pins, the three capture files, the C1
pairing regression, the frozen at-entry file and the ten C4 reference
logs hash-match; the live tree is still exactly the authorized eleven-file
stage-2 install plus the one untracked witness at `29f17698`.

**My reproductions.** Entry 4 passed / 5 failed, C2 control 8 / 1 and the
ninth case 0/1 on both at eight and one; the new candidate's panel 9/9 at
eight and one on the cargo-built binary; the frozen at-entry test 1/1 at
eight and one; the host cancel suites `suspension_abandonment_contract`
12, `suspended_timeout_contract` 13 and `retirement_interrupt` 9 ordinary
and profiled at eight and one, and `lifecycle_contract`'s six cancel cases
at eight and one; the queued-input STOP 0/1 on entry, C2 and the new
candidate at eight and one with the exact tuple (after the discard
`ticket_free=false`, observer false, no prompt; the request resume returns
the existing `Other`; the old handle's cancel yields
`TerminatedEvaluation["Cancelled"]` with `General::cancel`,
`prints=["new-cleanup"]` and `raw_intact=false`).

**The profiled-t8 red, attributed.** I reproduced the worker's initial
red — `expired_deadline_before_retirement_skips_every_cleanup` failing on
the profiled `profile_queue.rs:888` policy assertion — on the untouched
entry arm 6 of 20 runs, on the C2 arm 1 of 10 and on my change-4 probe 1
of 20, and 0 of 5 at one thread on both the entry arm and the probe. It
is a pre-existing, t8-only observation under the switch on the retirement
deadline path, not a candidate regression and not a defect under the
standing rule; it is recorded for settlement and does not block.

**The STOP, confirmed.** The queued-input discard is a genuine,
pre-existing ownership counterexample: `drive_input_request`'s
queued-response branch (`builtins_input.rs:231-244`) calls
`abandon_pending_suspension(SupersededOrQueuedInputDiscard)`; the
abandonment authority takes the owner and clears `pending_request` without
touching `active_accepted_request`; the raw entry returns with a live
ticket and no report; the next raw pause installs a report-less owner the
old ticket never owned; the handoff's unchanged `cancelled ticket && any
pending generation` condition then lets that ticket retire it. The five
activation sites are not the cause and cannot close it: the ticket was
installed by the earlier request entry and was never finished. The
counterexample is red on the entry arm, so it is pre-existing relative to
all three ticket changes. Change 3 is accepted as measured.

**Ruling — change 4: the abandonment authority ends the request whose
report it retires.** One production file, one function; no new field,
type, signature, variant, frame or counter. In
`Session::abandon_pending_suspension` (`suspension/core/abandon.rs`),
replace the plain report clear with a take that records whether a stored
report was retired, keep the unconditional `PendingRequest` refresh, and
finish the active accepted request on both exits — immediately on the
no-suspension early return, and after `drain_abandonment_worklist` returns:

```rust
let pending = self.take_accounted_pending_suspension_for_abandon();
let retired_report = self.pending_request.take().is_some();
self.refresh_memory_owner(
    crate::runtime::debug_tools::resources::MemoryOwner::PendingRequest,
);
let Some(pending) = pending else {
    if retired_report {
        self.finish_active_accepted_request();
    }
    return reason;
};

self.abandonment_worklist.push(pending);
self.refresh_memory_owner(
    crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist,
);
let reason = self.drain_abandonment_worklist(reason);
if retired_report {
    self.finish_active_accepted_request();
}
reason
```

The order is load-bearing: the retirement ticket stays installed through
the drain, so a later cancel of the same generation and the request's
absolute deadline still stop the remaining cleanups exactly as today
(`begin_ordered_abandonment`'s consumed-cancel window and
`drain_host_interrupt_pending`); only the finish moves after the drain.
The driver's own cancellation takes its report first, so the authority
skips the finish there and the driver's terminal handling is unchanged.
The queued-input discard is the measured instance; every other
`abandon_pending_suspension` caller that can retire a stored report — the
accounting-mode `HelperBestEffortDiscard` sites, the trace/monitor sites,
the checkpoint-rollback discard and the snapshot-restore failure — is
covered by the same authority change when it does, and the report-less
abandonments (the frozen at-entry test) are untouched because
`pending_request` is `None` there. Rejected alternatives: finishing at
the take (ends the retirement ticket before the drain and loses the
escalation/deadline window); finishing at the queued-input call site
(closes the instance, not the class, and re-derives the report predicate
at each caller); requiring a stored report in the handoff's cancellation
qualification (breaks the frozen
`cancelled_request_retires_a_non_request_suspension`, whose whole point is
that a cancelled ticket retires a report-less owner).

The contract: the stored report is the ownership certificate of the
active accepted request. It is stored only by a request driver whose
ticket is active, and every route that retires it ends that request —
the driver's own take (the driver finishes), the raw resume take (the
consume finish), and the abandonment authority (change 4, after the
drain). After the queued-input discard: the handle is inert, the observer
is clear, the request protocol returns the existing `Other` with empty
output, a later raw owner stays installed and raw-resumable, and a fresh
request runs with a free ticket.

**My calibration** (`gate-734c4/patch_c4.py`, two copied trees). The
change-3 candidate plus change 4 (`.temp/director/probe-734c4/tree`)
passes the nine-case panel 9/9 at eight and one; my three-composition
driver (`probe-734c4/queued_positive.rs`, whose central case is the
worker's probe's assertion sequence) 3/3 at eight and one; the C1 pairing
regression 1/1 at eight and one; the full runtime lib suite 1432 passed /
2 failed / 5 ignored at eight, exactly the two standing root reds and the
same totals as the pre-change arm; the named filters (frozen at-entry
1/1, `session_requests` 119/119, `suspension_abandonment` 26/26) at eight
and one; the four host targets 50/50 ordinary and profiled at eight and
one with `lifecycle_contract`'s six cancel cases. The **minimal 735
base** (`.temp/director/probe-734c4/min`: live plus `suspension/core.rs`,
`session/session_requests.rs` and `suspension/core/abandon.rs`, the three
capture files reverted to live bytes) passes the ticket panel 9/9 and the
pairing regression 1/1 at eight and one and my driver 3/3, so the
pairing/ticket design does not depend on brief 733's capture repair and
735's install base is the minimal one.

**Scope correction to the continuation-3 draft.** Brief 735 installs
three production files, not two: `suspension/core.rs`,
`session/session_requests.rs` and `suspension/core/abandon.rs`, plus both
permanent regressions, the host doc and `CHANGELOG.md`, on the working
tree that carries the stage-2 install. The three capture files are brief
733's shape and stay out of 735; the copied-arm install stage must use
the minimal live-tree base, not the C1/C2 arms that carry them.

**Continuation 4 order.** Design only (`Scope: NONE`), same scene; every
earlier arm, red, binary and receipt is preserved and new arms get new
names.

1. Implement change 4 exactly as ruled in a new copied candidate on top
   of the admission-c3 arm, and prove the delta against that arm is
   exactly `suspension/core/abandon.rs` plus the appended panel cases and
   the canonical fingerprint replay; no other production, capture, test,
   frozen-file or inventory identity/status/disposition/exclusion change.
2. Promote the queued-input route to the permanent panel as its tenth
   case with the ordered assertions: after the discard the ticket is free
   and the observer clear; the old handle is inert (a later cancel changes
   nothing and the handoff returns `None`); the later raw owner pauses,
   stays installed and completes through the public raw API; a fresh
   request returns 42 with a free ticket. Keep the report-less raw-discard
   control as an eleventh case and the request-owned cancellation handoff
   control (cleanup runs, `TerminatedEvaluation["Cancelled"]`,
   `General::cancel`) in the same panel if its helpers allow, otherwise in
   a small companion permanent test. The admission panel's existing bytes
   stay untouched (a prefix assertion proves it).
3. Measure the four-arm ladder at eight and one: entry (the original
   4 passed / 5 failed plus the tenth case red), C2 control (8/1 plus the
   tenth case red), change-3 candidate (9/0 plus the tenth case red), new
   candidate (every case green); the ninth and tenth baselines are 0/1 on
   the arms that lack their repairs. Frozen at-entry 1/1 at eight and
   one. Both C4 route controls.
4. Build the **minimal-base arm** (the exact 735 install shape: live plus
   the three production files and both regressions, the capture files at
   live bytes, the canonical inventory regenerated on that base) and run
   the full panel, the pairing regression, the frozen at-entry test, the
   named runtime filters, the runtime lib suite with exactly the two
   standing root reds, and the host cancel suites ordinary/profiled at
   eight and one. This is the arm 735 installs from; a red here is a STOP.
5. Complete the report/suspension/ticket induction from source with
   change 4: every report store/take/clear, every abandonment caller with
   its certificate disposition, every ticket store/take/finish/
   activation, the two activation kinds and five sites, the three
   retirement authorities, every transient window, the owed class rows
   (overlapping raw-entry install assert, re-entrant callbacks, rollback
   `(Some, None)`, the resume `expect`, the `Batch` report) and both
   capture/deferral guards. Every counterexample or unproven row is a
   STOP, never an exclusion.
6. Then the still-held continuation-1 gates, unchanged: both pairing
   routes and both permanent regressions at eight and one,
   `nested_public_evaluation` 15/15 ordinary and profiled, the named
   runtime/host/stdlib suites, armed report accounting, `cargo check
   --all-targets` and `cargo fmt --check` on copies with only the three
   ruled brief-731 arms red, the five C4 comparisons with one forced SHA,
   the cost rows (entry at most 599/4 both draws, no equal-call
   `rum_runtime` increase, the paired resume panel) and the rum-vs-rum
   microbench A/B.
7. Rewrite the TEMPLATE-shaped implementation brief for 735 with the
   settled four-change topology and the minimal-base scope correction
   above; the host doc and `CHANGELOG.md` name the report/ticket contract,
   including the queued-input retirement, the deferred-reservation
   admission and the raw takeover. Then brief 733's continuation on that
   repaired tree, then brief 731 continuation 6.

**Hard rules for continuation 4.** Same as this brief. At most **4** of
the remaining **8** canonical design builds; brief 733's held 4 and brief
731's remaining 8 stay separate, and a step that cannot fit is a STOP.
**Stop-loss.** Any ownership counterexample, any change to the frozen
test, a documented host flow, a C4 record or a protected file, any route
outcome differing from the contract above, or any need to weaken an
assertion is a STOP for my ruling. Two consecutive compiling candidates
that move neither a panel nor a named measured counter are a STOP.

**Q4, answered.** The queued-input abandonment leak is repaired at the
single abandonment authority — a stored report retires its active
accepted ticket after the cleanup drain, never before it — with the
escalation, deadline and frozen at-entry behaviour unchanged; the scope
is `suspension/core/abandon.rs` only.

**Next.** Continuation 4 of this brief; then my design gate and brief
735; then brief 733's continuation; then brief 731 continuation 6; then
continuation 38 of brief 623; then C2's settlement round, CM,
`CHANGELOG.md` before S10 (including this lifecycle contract), G13
S10-S54/F1-F23/R, E2.

## Continuation-4 checkpoint (2026-11-13)

**The round stopped on Codex's own safety filter, not on a product
signal.** Run: `cdx` (gpt-6-astra, max; session
`01a1d1ef-84e7-7350-83e5-e1242c4097e7`, fork of
`01a1d1c5-97af-7b50-894a-d5a0cf47491e`; log archived as
`.temp/logs/archive/cdx_brief_734.continuation-4.cdx-filter`, 6,584
lines, 461,145 B, sha256 `d8527a58…`; 68 run records — 48 exit 0, 18
exit 101 (the designed panel/baseline reds at eight and one on the
entry, C2 and C3 arms, and the minimal base's two standing runtime-lib
reds), two retained exit-1 harness refusals (`minimal_gates.py`'s
aggregate-versus-primary wrapper and `minimal_resume.py`'s filename
selector, both resolved in PROGRESS and neither counted as a gate);
memory-scope peak 4,396,109,824 B with oom 0, oom_kill 0, rc 1; 0 of 4
continuation canonical builds, 0 of 8 overall; no executed git, no
`.git-attempts`, no network; every write under
`.temp/brief-734/continuation-4`). After the step-4 minimal-base gates
passed, the worker was constructing the step-5 re-entrant class probe
when its next turn was refused: "This content was flagged for possible
cybersecurity risk. … apply for Daybreak access". The drafted probe
(`reentrant_drain_probe.rs`, `reentrant_drain_probe.py`) is preserved
and was never executed; no counterexample or class row was settled or
excluded.

**Steps 1-4, preserved and not yet gated.** My own walk of the scene at
the stop: the live tree equals the round's `live-entry-hashes.json`
exactly (2,732 files, 0 missing / 0 extra / 0 differing) and `git
status` is still the authorized eleven-file stage-2 install plus the one
untracked witness; both changed arms equal their after-replay receipts
byte-for-byte (2,734 files each, 0 / 0 / 0); the change-4 delta against
the admission-c3 control is exactly `suspension/core/abandon.rs`
(`09ac19ec…`→`0900f344…`) plus the panel
(`97ad286a…`→`31909d29…`), and the panel is the unchanged nine-case
13,101-byte prefix plus the three appended cases (`a0a3d2af…`, one blank
separator line); the recorded ladder at eight and one is entry 6/6 (the
old 4/5 plus the tenth case red plus both controls), C2 10/2, C3 11/1
and the new candidate 12/0, with the ninth baseline 0/1 on entry/C2, the
tenth 0/1 on entry/C2/C3, the frozen at-entry test 1/1 and both C4 route
controls 1/1; the minimal-base arm — live plus `suspension/core.rs`,
`session/session_requests.rs` and `suspension/core/abandon.rs`, both
regressions, the capture files at live bytes and the canonical inventory
regenerated on that base — passes the 12-case panel plus the pairing
regression, the frozen test 1/1, the six named runtime filters, the
primary runtime lib 1432 passed / 2 failed / 5 ignored at eight and one
with exactly the two standing root reds (1434/2/5 counting the two
nested child receipts, no unexpected failure), four host targets 50/50
ordinary and profiled at eight and one and `lifecycle_contract` cancel
6/6. These are the record; acceptance waits for the report and my gate.

**Ruling.** The STOP is the provider's safety boundary, not a defect and
not a worker failure. Per the user's 2026-11-12 routing — a Codex quota
death asks about the second Codex account, `mst` only when Codex itself
refuses on its safety guardrail — continuation 4 resumes on `mst`
(GLM-5.3). A second refusal is a STOP for my ruling; there is no third
worker, no rephrased retry on `cdx` and no scope reduction. Completed
steps are not rerun and their evidence is not re-credited: the resume
re-verifies the preserved scene against the receipts above and continues
from step 5.

**Resume order.** (1) Verify the preserved scene: the live tree against
`live-entry-hashes.json`, both changed arms against their after-replay
receipts, the panel prefix against `panel-provenance.json`, the change-4
delta against `change4-source-delta.json`; a mismatch is a STOP. Do not
rerun the completed ladder, the minimal gates or any earlier build. (2)
Step 5 exactly as written, including the re-entrant callback row the
draft probe was built for: every report store/take/clear, every
abandonment caller with its certificate disposition, every ticket
store/take/finish/activation, the two activation kinds and five sites,
the three retirement authorities, every transient window, the owed class
rows and both capture/deferral guards. Every counterexample or unproven
row is a STOP, never an exclusion. (3) Step 6, the held continuation-1
gates, unchanged: both pairing routes and both permanent regressions at
eight and one, `nested_public_evaluation` 15/15 ordinary and profiled,
the named runtime/host/stdlib suites, armed report accounting, `cargo
check --all-targets` and `cargo fmt --check` on copies with only the
three ruled brief-731 arms red, the five C4 comparisons with one forced
SHA, the cost rows (entry at most 599/4 in both draws, no equal-call
`rum_runtime` increase, the paired resume panel) and the rum-vs-rum
microbench A/B. (4) Step 7, the rewritten TEMPLATE implementation brief
for 735 on the minimal base, with the four-change topology, both
regressions, the host doc and `CHANGELOG.md`. (5) Write the round's
`rum-report/1` record at `.temp/brief-734/report.json` (replacing the
continuation-3 record preserved in `continuation-4/previous/`), make
`report.py check 734` pass, and STOP for my gate with every verdict line
quoted.

**Hard rules for the resume.** Identical to this brief; additionally:
no live edit, no git, no network, one bounded Cargo at a time in the
16 GiB scope, all scratch under `.temp/brief-734/continuation-4`, the
4-build canonical cap unchanged, the frozen witnesses, assertions, C4
records and protected files untouched, and no unrun gate credited. If
the `mst` backend itself refuses, stop and report; I rule the next step.

## Continuation-4 gate (mst, 2026-11-13) — the re-entrant row is a real counterexample; the round is accepted as a well-taken STOP

**Run.** `mst` (GLM-5.3, session `3adf560a-9640-b570-7845-429ce39acc8d`;
log `.temp/logs/mst_brief_734.log`, 20,371 B; $7.11, 22,833,122 in /
107,333 out; memory-scope peak 2,430,087,168 B, oom 0, oom_kill 0, rc 0;
103 run records; 0 git attempts, no network; every write under
`.temp/brief-734/continuation-4`; `report.py check 734` OK with 694
citations by my own run; 0 of this continuation's 4 canonical design
builds, 0 of 8 overall). One disclosure: `/tmp/c4-inventory.txt`, a
transient scene-path listing written outside the fence during report
assembly and removed immediately; I verified it is gone. The never-`/tmp`
rule stands and the deviation is recorded, not a product signal.

**My walk of the scene.** Live 2,732 files = `live-entry-hashes.json`
except this brief's appended checkpoint (5,809 B; 0 missing / 0 extra / 0
differing elsewhere). `source-retire-c4` and `source-minimal-c4` equal
their after-replay receipts (2,734 files each, 0/0/0); the change-4 delta
against `source-c3-control-c4` is exactly `suspension/core/abandon.rs`,
the three appended panel cases and the canonical 68-cell
fingerprint-only replay; the minimal arm is entry plus
`suspension/core.rs`, `session/session_requests.rs`,
`suspension/core/abandon.rs`, both pairing regressions and the replay.
The panel is the unchanged 13,101-byte nine-case prefix plus the three
appended cases (`31909d29…` full, `97ad286a…` prefix). `git status` is
still exactly the authorized eleven-file stage-2 install plus the one
untracked witness; no live file is newer than the dispatch.

**My reproduction — 12 runs of the six receipted probe binaries.** All
six at eight and one threads reproduce the report's table exactly:
- drain ladder: `c3-control-c4` 1/1 at both (`inside=
  (ticket_free_after_recursive_discard=false,retirement_cancel_observed=
  true,pending=None)`, outer `Ok(("outer-answer",None,[],
  ["WithCleanup::skipped"])),observations=5,ticket_free=false,
  observer=true`); `retire-c4` and `minimal-c4` 0/1 at both
  (`inside=(true,false,None)`, outer `Ok(("outer-answer",None,
  ["outer-cleanup"],[])),observations=3,ticket_free=true,observer=false`).
- consume ladder: `entry-c4` 1/1 at both (the green tuple);
  `c2-control-c4` and `minimal-c4` 0/1 at both (the red tuple).
The probes are calibrated: `flags & 1` proves the real nested sequence
executed, `flags & 2` fails exactly on the ticket window, `flags & 4` on
the cancel observation; no assertion was weakened and both failures are
reproduced at one thread.

**The mechanism, read from source (minimal arm).**
`abandon_pending_suspension` takes the outer report at `abandon.rs:10`,
pushes the owner and drains at `:25`; the drain runs guest cleanups
inside the window, and the outer finish sits at `:27` after it, so the
outer ticket is installed through the drain by design. In that window
`begin_input_suspension` (`core.rs:513-519`) installs unconditionally; a
nested plain driver entry (`eval_request`) has no ticket of its own
(only the four `eval_accepted_*` entries install one), and
`finish_input_required_request` (`session_requests.rs:512-528`) stores
the nested report in the single slot while the outer ticket is active.
Both finish authorities — the change-2 consume helper
(`core.rs:487-492`) and the change-4 abandon finish (`abandon.rs:16`,
`:27`) — call `finish_active_accepted_request()`
(`session_helpers.rs:93-96`), which closes whichever ticket is active;
nothing binds the report to a ticket. The C2-gate premise "a stored
report implies the active ticket is its own" is false in this
composition. The counterexample is introduced at change 2 (entry green,
`c2-control-c4` red) and inherited by change 4; change 3 is not the
introducer (the drain ladder's `c3-control-c4` is green), and the
pairing (change 1) is not falsified: every report is stored beside its
own suspension and every retirement consumes the pair together.

**Ruling (Q5).** The STOP is well taken. The change-2 and change-4
finish authorities as ruled cannot be installed; there is no rephrased
retry and no scope cut. The single-owner principle is saved by making
the report's ticket identity explicit and enforced: the report carries
the identity of the accepted-request ticket it belongs to, minted by
the entry that installs the ticket; a report stored by a plain entry is
ticketless; each finish authority takes the identity and finishes only
a matching active ticket; a ticketless report's retirement leaves the
active ticket untouched, so the outer ticket survives to the drain's own
after-drain finish. The change-2 ruling's "no new field/type/signature"
constraint is reopened for this identity only: prefer an existing
identity (the interrupt generation/phase word already carried by
`RequestInterruptState`), justify and cost-measure any new state, and
treat any new public surface as a STOP. The drain-active public refusal
is not selected as the primary: it adds a typed refusal to the
documented host flow and guards only one window, while the identity
repairs the authority at every window. Leaving the measured red for
settlement is rejected: it would ship a known cancel/deadline loss
inside the 735 shape. **Q6:** yes — both probes become the permanent
re-entrant class witnesses once the repair is ruled and calibrated;
they use only public APIs and enter the install fleet with the
implementation.

**The other owed rows.** Dispositions accepted as reported: the
overlapping raw-entry install assert stays registered out of class with
its exact public-panic composition for the C2 settlement round; the
resume expect is unchanged outside the C1 route contracts; the Batch
report put-back/refusal is unchanged and green on every arm; the
rollback `(Some,None)` panic stays registered, its reachability covered
by the same re-entrant class and to be disposed in the design round.

**Held.** Steps 6-7 and the conditional 735 draft stay held and take no
credit; brief 733's 4 builds and brief 731's remaining 8 stay
untouched. The round's 103 runs are recorded; no gate beyond the STOP
evidence is credited.

## Continuation 5 — design-only, same scene, on `mst`

The line is the one Codex's safety filter refused; per the user's
2026-11-13 ruling this task is unsuitable for Codex and the `mst` switch
is necessary, not optional. A second `mst` refusal is a STOP for my
ruling. **Scope: NONE** — no live edit; work in a byte copy under
`.temp/brief-734/continuation-5/`; no git, no network; one bounded Cargo
at a time in the 16 GiB scope; all scratch under that root; at most 4 of
the remaining 8 canonical design builds; frozen witnesses, assertions,
C4 records and protected files untouched; no unrun gate credited.

1. Reproduce both ladders on the receipted binaries before any design;
   the counterexample must reproduce first.
2. Inventory every report store/take/finish/clear, every ticket
   install/finish/activation, and every window that runs guest code
   (the abandonment drain, the fault unwind, the cleanup-pause clear,
   the rollback, the capture), with one disposition each; prove which
   entries can store a report while a foreign ticket is active, and
   prove the nested plain entry is the only ticketless publisher — any
   other is a STOP.
3. Design the ownership identity with exact signatures and site
   changes: the token (prefer the existing interrupt generation/phase
   word over a new type), the field on the pending report, the minting
   entry, the threading through the four accepted entries and the
   plain entries, the finish predicate, the ticketless-retirement
   behavior, and the effect on the batch put-back, the rollback
   `(Some,None)`, the fault unwind, the cleanup-pause clear, and both
   guards' proof premises. Evaluate at least two alternatives (this
   identity, the drain-active refusal, an entry-instanced finish) and
   recommend one strictly by evidence. The compiler must enforce the
   finish precondition.
4. Calibrate in the copy: both re-entrant ladders green at eight and
   one on the repaired shape; the same probes red on the change-2 and
   change-4 arms; the 12-case ticket panel, the frozen at-entry test,
   both pairing regressions and the C4 controls unchanged; the minimal
   735 base green through the same gate set; the changed-function cost
   rows at eight and one (the pause store and both finish sites), with
   any default-path increase a STOP; the four-change topology intact.
5. Draft the TEMPLATE implementation brief for the combined 735
   install: the identity repair plus the four changes, both
   regressions, the two probes as permanent tests (public APIs only;
   exact names, placement, calibrated red on the pre-repair arms and
   green on the repaired arm), the host doc and `CHANGELOG.md` naming
   the lifecycle contract; the three capture files stay 733's/731 c6's.
6. Write the round's `rum-report/1` record (preserving this round's
   under `continuation-5/previous/`), make `report.py check 734` pass,
   and STOP for my gate with every verdict line quoted.

**Hard rules.** Identical to this brief; no live, protected or frozen
edit; no git; no network; the probe binaries link only public APIs; no
assertion, witness, inventory or C4 record changes; a counterexample,
an unproven row or a public-surface change is a STOP, never an
exclusion.

## Continuation-5 gate (2026-11-13)

**The design round is accepted. The ownership identity is ruled sound
as implemented, with three director amendments carried into brief 735.**
Run: `mst` (GLM-5.3, session `2d49cb8d-44a8-3a2c-2ca9-7a2f92899341`;
log `.temp/logs/mst_brief_734.log`, 27,577 B; $15.90, 53,352,300 in /
241,759 out; memory-scope peak 4,453,531,648 B, oom 0, oom_kill 0,
rc 0; checks 13 PASS, measurements 5 PASS; `report.py check 734` OK
with 1,209 citations by my own run; 3 of the continuation's 4 canonical
design builds used, 5 of the overall 8 remain; git-attempts 0, no
network; every write under `.temp/brief-734/continuation-5/` or the
scene's own `report.json`/`PROGRESS.md`).

**My scope walk.** Live 2,732 files = the round's entry pin exactly
(0 missing / 0 extra / 0 differing); `git status` is still the
authorized eleven-file stage-2 install plus the one untracked witness
at `5a7a351e`; no live file is newer than the dispatch except the
brief itself (07:33:21, my own pre-dispatch append); no other scene
was touched. The entry copy equals live except the three untracked
`scripts/__pycache__` artifacts it left out. The identity arm equals
its `source-identity-c5-after-replay-hashes.json` receipt on 2,736
files (0/0/0); against the continuation-4 minimal arm it differs in
exactly the nine runtime paths, the two added witnesses and the
canonical inventory (68 fingerprint-only cells, 0 added/removed/
reordered, 0 non-fingerprint changes, `installed:false`); the two
witnesses are byte-equal to the C4 probe sources and both pairing
regressions byte-equal to the C4 minimal arm. Bookkeeping: the named
`identity-source-delta.json` is stale for three source files and omits
the inventory cell; `identity-source-delta.attempt4.json` reproduces
the arm's hashes on the nine source paths and the two added tests
(only the derived TSV's hash is superseded by the final replay), so
**the arm plus its after-replay receipt is the install manifest, never
the stale file.**

**My reproduction.** Both witnesses green 1/1 at eight and one on the
identity arm with the receipted green tuple (`inside=(false,true,None)`,
outer `Ok(("outer-answer", None, [], ["WithCleanup::skipped"]))`,
observations 5, cleanup skipped); the 12-case ticket panel plus the C1
pairing regression 13/13 at eight and one; the frozen at-entry test 1/1
at both counts; the six named filters 11/119/26/2/15/9 green at both;
the runtime lib suite 1432 passed / the two standing root reds lp655
and lp656 / 5 ignored at both; the four host targets 50/50 ordinary and
profiled at both (switch on where gated); `check --all-targets` green
at rc 0; `fmt --check` exactly the three ruled brief-731 arms (plus
the aliased `outcome.rs` entry). The C4 red arms stand as receipted by
the continuation-4 gate and this round's 12-run reproduction, which I
re-verified then; this gate reproduces the green side.

**One false bar, corrected.** The draft's measured-state list claims
`nested_public_evaluation` 15/15 ordinary and profiled. The suite is
**14/15** on the identity arm and on the live tree, at eight and one;
the single failure is the known pre-existing
`nested_reentrant_resume_preserves_the_real_preexisting_owner`
(`evaluation_context.rs:33`, `continuation contexts unwind a saved
prefix`), which brief 733 / brief 731 continuation 6 own. The 735 bar
is 14/15 with exactly that failing set on live and on the installed
tree; a fifteenth red or a moved failure is a STOP.

**The cost, accepted (Q1).** The seven per-call rows — resume driver
+22.0, abandon +8.0, accepted entry +4.0, pause store +3.0,
entry-from-checkpoint +2.5, hand-off +2.4, value tail +0.25 — are the
identity's own functions, deterministic in every draw/thread/N cell;
the consume site is a measured decrease (`take_accounted_pending...`
-3.5 per take); no non-changed function shows a deterministic per-call
increase (the four fixed one-time startup rows and the draw-varying
interner/address rows are recorded, never normalized). The
continuation-1 strict letter is clarified: the ordinary path is the
guest evaluation hot interior, which shows no delta; the request-driver
protocol rows are the authorized cost of the new state, and a
mint-at-store reshape would only move the same instructions. The
analyzer's `deterministic_across_all_4_runs=false` flag on
`abandon_finish` is an absolute-Ir artifact — the per-call cells are
exact +8.0 in all eight.

**Rulings.** Q2 ratified: brief 735 gets at most **8** canonical
release builds through `scripts/probe-build.py`, one release codegen
unit, one forced SHA spot-check, standing separate from brief 733's
held 4 and brief 731's remaining 8. The design is accepted: the
private-field `ReportOwner` riding the existing interrupt generation,
the entry-local mint (installed-or-promoted, never reading the active
field), the owner-gated finishes and the ticketless-retirement no-op
are sound; I read the nine-file diff and the induction's six points
hold against the source. The continuation-4 Q5/Q6 rulings are
satisfied (the two probes enter the install fleet as permanent
witnesses). The four deviations are accepted: the hand-off gate
refinement is the change-3 ruling's own promotion leg with the failed
panel case retained; the resolved fourth filter selector and the
runtime-lib aggregate distinction; the two arm-local formatting fixes
(formatting-only, replayed); and the four-change base choice for the
identity A/B, with the combined-install A/B owed at 735.

**Amendments carried into brief 735.**
1. `finish_active_accepted_request` shrinks to module-private (`fn`).
   The design's tooth 1 claims the two authorities cannot name a bare
   finish, but the helper is still `pub(in crate::runtime)` and
   therefore reachable from both. Its only caller is
   `finish_report_owner_ticket` in the same file, so the shrink is
   behavior-free and makes the compiler enforce the claim; a grep
   proof of no other caller is required at install.
2. The `nested_public_evaluation` bar is 14/15 with the named
   pre-existing red, never 15/15.
3. The install manifest is the arm `source-identity-c5` plus its
   after-replay receipt; the stale `identity-source-delta.json` is not
   a manifest, and the derived inventory TSV is regenerated canonically,
   never installed by hash.

**Next:** brief 735 (implementation, the combined install) is written
and dispatched on `mst` under the user's 2026-11-13 ruling that this
task line is unsuitable for Codex; after its gate, brief 733's
continuation on the repaired tree, brief 731 continuation 6,
continuation 38 of brief 623; then C2's settlement round, CM,
`CHANGELOG.md` before S10, G13 S10-S54/F1-F23/R, E2.
