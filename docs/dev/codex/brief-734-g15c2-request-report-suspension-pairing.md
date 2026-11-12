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
