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
