# Brief 735 — g15-c2: the request report/ticket ownership install

**The identity repair plus the four changes, on the minimal live-tree base**

Written by the director after brief 734's continuation-5 gate
(`docs/dev/codex/brief-734-g15c2-request-report-suspension-pairing.md`,
the "Continuation-5 gate" section). The design, the mechanism and the
calibration are receipted; this brief installs them.

## Goal

Install, on the live working tree (which carries the authorized
uncommitted brief-731 stage-2 install), the combined request
report/suspension/ticket repair exactly as designed, receipted and
calibrated by brief 734's continuations 1 through 5, so that:

- every pending report is stored, retired and resumed beside the
  suspension it names (change 1, the C1 pairing);
- a raw consumption that retires a published report ends that report's
  request ticket at the consume authority (change 2);
- a deferred reservation is promoted only onto a request-owned owner:
  by the report-gated cancellation hand-off, or by an admitting request
  entry after its overlap rejection (change 3);
- the abandonment authority ends the request whose stored report it
  retires, after the cleanup drain and never before it (change 4);
- and the pending report carries the **identity of the accepted-request
  ticket it belongs to**, minted by the entry that installs that ticket;
  a report published by a plain entry is **ticketless**; every finish
  authority takes the stored identity and finishes only a matching
  active ticket, so a ticketless retirement leaves an active foreign
  ticket installed (the continuation-5 ownership identity).

Ship both pairing regressions and the two re-entrant class witnesses as
permanent tests, and name the report/ticket **lifecycle contract** in
the host doc and `CHANGELOG.md`. No public surface change, no new
dependency, no `unsafe`. STOP for the director's gate; the working tree
stays uncommitted.

The single-owner principle this install protects: a pending report can
never retire a foreign accepted-request ticket, and an accepted-request
ticket ends at its own driver tail or at a retirement authority acting
on a report that names it — never on a borrowed close.

## Context pointers

- Brief 734's document and gates: the design STOP gate, continuations
  1-4 gates, the continuation-5 order and its gate. Read the
  continuation-5 gate first: it carries the three amendments this
  brief enforces.
- **The arm of record:** `.temp/brief-734/continuation-5/source-identity-c5/`
  — the install source. Its final receipt is
  `source-identity-c5-after-replay-hashes.json` (2,736 files, my walk
  reproduces it). `identity-source-delta.attempt4.json` is the
  source-path before/after manifest; `identity-source-delta.json` is
  stale and is **not** a manifest. The derived inventory TSV in the arm
  is regenerated canonically on install, never installed by hash.
- **The four-change base:** `.temp/brief-734/continuation-4/source-minimal-c4/`
  (the three production files and both pairing regressions at their C4
  receipt bytes). `.temp/brief-734/continuation-4/` also holds the
  receipted red arms and binaries — read-only.
- The continuation-5 design: `.temp/brief-734/continuation-5/design.md`
  (§2 mechanism, §3 class inventory with `identity-audit.json`, §4a-4c
  the token/mint/call sites, §5 owed rows, §6 alternatives) and
  `draft-implementation-brief.md`.
- The measured calibration records: `c5-ladder-reproduction.json`,
  `runtime-failures-c5.json`, `host-results-c5.json`,
  `fmt-results-c5.json`, `cost-summary.json`, `cost-increases.json`,
  `identity-derived-replay.json` and the logs under
  `.temp/brief-734/continuation-5/logs/`.
- The three capture-repair files belong to brief 733 / brief 731
  continuation 6 and stay at live bytes:
  `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs`,
  `suspension/core/queue.rs`, `suspension/core/storage.rs`.
- Read before installing, under `crates/rum-runtime/src/runtime/`:
  `model/requests.rs`, `model/request_cancellation.rs`,
  `session/session_requests.rs`, `session/session_helpers.rs`,
  `session/session_api.rs`, `suspension/core.rs`,
  `suspension/core/abandon.rs`, `suspension/core/driver.rs`,
  `suspension/core/fault.rs`, plus
  `crates/rum-runtime/src/runtime/tests/session_memory_structures.rs`.

## Scope

Install on the live working tree. The complete change set, one
disposition each:

Production (nine paths, all inside the crates; no public surface —
`PendingReport` and `ReportOwner` are `pub(in crate::runtime)`):

1. `crates/rum-runtime/src/runtime/model/requests.rs` — the
   `ReportOwner` token and the `PendingReport.owner` field (design §4a).
2. `crates/rum-runtime/src/runtime/model/request_cancellation.rs` — the
   `AcceptedRequest::generation()` accessor.
3. `crates/rum-runtime/src/runtime/model/mod.rs` — the export.
4. `crates/rum-runtime/src/runtime/session/session_requests.rs` — the
   four `_admitted` threading bodies, the accepted-entry mints, the
   owner-gated tails, `take_pending_report_owner` (design §4c).
5. `crates/rum-runtime/src/runtime/session/session_helpers.rs` — the
   owner gates (`accepted_request_owned_by`,
   `finish_report_owner_ticket`, `cancels_owned_request`), the
   activation capture, the hand-off's promoted-adoption qualification,
   **and the amendment: shrink `finish_active_accepted_request` from
   `pub(in crate::runtime) fn` to `fn`** so the compiler enforces that
   `finish_report_owner_ticket` is the only finish reachable outside
   `session_helpers.rs` (its only caller is in the same file; prove
   that with a grep and report it — any other caller is a STOP).
6. `crates/rum-runtime/src/runtime/session/session_api.rs` — drop the
   bare `finish_active_accepted_request()` at the snapshot-restore
   failure.
7. `crates/rum-runtime/src/runtime/suspension/core.rs` — change 2's
   consume authority takes the owner and finishes only a matching
   ticket.
8. `crates/rum-runtime/src/runtime/suspension/core/abandon.rs` — change
   4's abandonment authority does the same on both exits, its finish
   after `drain_abandonment_worklist`, never before it.
9. `crates/rum-runtime/src/runtime/tests/session_memory_structures.rs` —
   the compile-forced owner field on the parked-report construction.

Tests (four paths added, public APIs only):

10. `crates/rum-stdlib/tests/request_report_pairing.rs` — the C1
    pairing regression (C4 receipt bytes).
11. `crates/rum-stdlib/tests/request_ticket_pairing.rs` — the nine-case
    panel plus three appended cases (C4 receipt bytes, 12 cases).
12. `crates/rum-stdlib/tests/request_reentrant_retirement_pairing.rs` —
    the drain ladder as a permanent class witness (byte-equal to the
    C4 probe source).
13. `crates/rum-stdlib/tests/request_reentrant_raw_takeover_pairing.rs`
    — the consume ladder as a permanent class witness (byte-equal to
    the C4 probe source).

Documentation:

14. `docs/host-embedding-and-hardening.md` — the report/ticket
    lifecycle contract subsection.
15. `CHANGELOG.md` — the lifecycle contract entry naming the
    queued-input retirement (change 4), the deferred-reservation
    admission (change 3), the raw takeover (change 2), and the
    borrowed-composition behavior change: a nested plain publication
    under a foreign active ticket can no longer retire that ticket; the
    cancelled foreign ticket leaves the pair installed and the entry
    returns its existing overlap refusal; the outer ticket survives
    through the abandonment drain to its own after-drain finish.

Derived (regenerated, never hand-edited): the canonical inventory
`crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`
— 68 cells, fingerprint-only, through the canonical generator.

MUST NOT: change any public surface — no public type, signature,
`EvalError` variant or wire change; the three capture-repair files
(named above — install them at live bytes, untouched); the seven
brief-731 refusal production files; the five C4 owner records; any
frozen witness, assertion or inventory row (fingerprint-only replay
through the generator); `Cargo.lock` or any manifest; `.git` in any
form (no commit, no branch, no push — the director owns git; leave the
tree uncommitted for the gate); the permanent
`crates/rum-stdlib/tests/nested_public_evaluation.rs` bytes; the
receipted pre-repair arms and binaries under
`.temp/brief-734/continuation-4/`. No network; no new dependency; no
`unsafe`; no hand-edited inventory, no weakened assertion, no
regenerated expectation.

## The measured state — what the install must reproduce

The red/green calibration is receipted; the install must reproduce
exactly these numbers on the live tree, and any drift is a STOP:

- **Both ladders**, red on the four-change arm and green on the plain
  entry and the repaired arm, at eight and one (exit codes and tuples
  verbatim from `c5-ladder-reproduction.json`):
  - drain (`recursive_abandonment_must_not_finish_outer_retirement_ticket`):
    red 0/1 on the change-4 arm — `inside=(ticket_free_after_recursive_discard=true,
    retirement_cancel_observed=false, pending=None)`, outer
    `Ok(("outer-answer", None, ["outer-cleanup"], []))`, observations 3;
    green 1/1 on the plain entry and the repaired arm — `inside=(false,
    true, None)`, outer `Ok(("outer-answer", None, [],
    ["WithCleanup::skipped"]))`, observations 5, cleanup skipped.
  - consume (`raw_takeover_during_cleanup_must_not_finish_outer_retirement_ticket`):
    red 0/1 on the change-2 arm; green 1/1 on the plain entry and the
    repaired arm, same tuple shapes.
- **The gate set on the repaired live tree**, all at eight and one
  unless noted: the 12-case ticket panel plus the C1 regression 13/13;
  the frozen at-entry test `cancelled_request_retires_a_non_request_suspension`
  1/1 with its pinned bytes untouched; both C4 controls inside the
  panel; the six named runtime filters nonzero and green
  (`resume_evaluation_context` 11, `session_requests` 119,
  `suspension_abandonment` 26, `session_memory::request_owners` 2,
  `frame_root_trace` 15, `queue_boundaries` 9); the full rum-runtime
  lib suite 1432 passed / exactly lp655 and lp656 failed / 5 ignored
  (the ledger aggregate may read +2 from the two 1/0/0 nested child
  receipts — harness accounting, never a semantic red); the four host
  targets 50/50 ordinary and profiled (switch on where gated) and the
  lifecycle cancel 6/6; `nested_public_evaluation` **14/15** with
  exactly the known pre-existing failing set
  (`nested_reentrant_resume_preserves_the_real_preexisting_owner`,
  `evaluation_context.rs:33`) on live and on the installed tree — a
  fifteenth red or a moved failure is a STOP; `cargo check
  --all-targets` green; `cargo fmt --check` reporting exactly the three
  ruled brief-731 arms (`outcome.rs:696`, the `model/requests.rs`
  entry, `request_errors.rs:133`; the fourth diff is the same
  `outcome.rs` through the `suspension_oracle` alias).
- **The changed-function cost rows against the four-change base**
  (deterministic in every draw/thread/N cell): pause store
  `finish_input_required_request` 501.0→504.0 per publish (+3.0);
  consume site `take_accounted_pending_suspension_for_resume` −3.5 per
  take; `abandon_pending_suspension` +8.0 (126.3→134.3);
  `eval_accepted_request` +4.0; `eval_expr_request_from_checkpoint`
  +2.5; `resume_eval_request_from_checkpoint` +22.0;
  `finish_cancelled_pending_evaluation_request` +2.4;
  `finish_accepted_value_request` +0.25; reservation mint 0. No
  deterministic per-call increase in any non-changed function; the
  fixed one-time startup rows and the draw-varying interner/address
  rows are recorded and never normalized.

## What to do

1. **Scene and pre-state.** Pin the live tree (full-tree sha256 walk;
   the expected state is the authorized eleven-file stage-2 install
   plus the one untracked witness); record every path. Verify the
   three capture files are at live bytes. Announce the spend plan
   against the ratified build budget before the first canonical build.
2. **Install the two permanent class witnesses first** (the two
   re-entrant probe files at their receipted bytes). Run them at eight
   and one: both must be **green on the unchanged live tree** — this
   is the install's own calibration floor.
3. **Install the four-change topology** (the three production files at
   the C4 receipt bytes, plus both pairing regressions; regenerate the
   canonical inventory). Run both witnesses at eight and one: both
   must now be **red 0/1** with exactly the receipted tuples — the
   defect is introduced on the install tree. A red that differs from
   the receipted tuple, or a red at only one thread, is a STOP
   (reproduce a red at `--test-threads=1` before it is a defect).
4. **Install the identity repair** (the nine-path arm of record, plus
   the `finish_active_accepted_request` visibility amendment; the two
   witnesses are already in place). Run the canonical fingerprint
   replay. Run both witnesses at eight and one: both **green 1/1**
   with the receipted green tuples.
5. **Verify the installed source** equals the arm of record on the nine
   production paths and the two added witnesses (the visibility
   amendment is the sole permitted difference; quote the arm hash and
   the installed hash per file), and that the derived inventory delta
   is fingerprint-only (68 cells, 0 added/removed/reordered).
6. **The full gate set at eight and one** exactly as the measured-state
   section names it: the 13 regressions, the frozen test, the six
   filters, the runtime lib suite, the four host targets, the
   `nested_public_evaluation` 14/15 known-red set, `check
   --all-targets`, and `fmt --check` with exactly the three ruled
   arms.
7. **Release and cost.** Canonical release builds through
   `scripts/probe-build.py` at the stable active path, one release
   codegen unit: a base arm (the four-change topology) and the
   candidate (the full install), plus one forced same-source SHA
   spot-check. Reproduce the five C4 owner comparisons byte-exact at
   eight and one (as named in brief 733's and brief 731's gates; if the
   exact set cannot be located, report and STOP rather than invent
   one). Measure the changed-function cost rows at eight and one, two
   draws: every row must land within the C5 deltas quoted above. Run
   the rum-vs-rum microbench A/B (`scripts/core-language-microbench.sh`)
   on base and candidate: no hot-label regression, and no equal-call
   `rum_runtime` increase outside the named mechanism rows. Record
   fixed one-time startup layout rows, never normalize them.
8. **Docs.** Add the report/ticket lifecycle contract subsection to
   `docs/host-embedding-and-hardening.md` and the `CHANGELOG.md` entry
   as scoped above; the reference drift test must pass in the same
   change.
9. **Report and STOP.** Write the `rum-report/1` record, make
   `scripts/report.py check 735` pass with no PROBLEM, close with
   `scripts/report.py summary 735` output, leave the tree uncommitted,
   and STOP for the director's gate.

## Hard rules

- Worker sandbox `workspace-write`; one bounded Cargo at a time inside
  the 16-GiB memory scope; every invocation recorded through a bounded
  wrapper; derive bounds from the recorded receipts (live debug about
  25 s, release about 55 s; 400 s for a full cargo, 300 s for a
  canonical build).
- **Canonical build budget (ratified):** at most **8** canonical
  release builds through `scripts/probe-build.py` with one release
  codegen unit and the ledger record — one base arm, one candidate
  arm, one forced SHA spot-check, and up to five for repair
  iterations. Standing entirely separate from brief 733's held 4 and
  brief 731's remaining 8; no transfer, no borrowing. A step that
  cannot fit is a STOP.
- The standing inventory rule: replay the canonical 68-cell
  fingerprint-only inventory after every source edit and before the
  first acceptance suite; a suite red caused only by a stale
  fingerprint, certificate or position cell is an ordering artifact,
  not a regression; never weaken or hand-edit an inventory.
- `cargo fmt --check` reports exactly the three ruled brief-731 arms;
  do not touch them.
- Any ownership counterexample, any unproven class row, any need to
  change an assertion, the frozen test bytes, a C4 record, a witness,
  a protected file, the documented host contract, or the public
  surface is a STOP, never an exclusion; never relabel an accepted C2
  ledger row.
- No fabricated state: every attempt retained and reported, including
  failed ones; every claim carries its log path and a verbatim verdict
  line; no credit from unrun gates.
- A red is reproduced at `--test-threads=1` before it is called a
  defect.

## Deliverable

- The pinned pre-state and the completed install — every path with its
  before/after sha256 equal to the arm of record (the visibility
  amendment named separately), the C4 receipt bytes for the
  four-change files, and the fingerprint-only inventory delta.
- The red/green calibration receipts on the install tree: the two
  witnesses green on live, red on the four-change arm, green on the
  repaired arm — at eight and one, with the receipted tuples.
- The full gate-set receipts at eight and one (regressions, frozen,
  filters, runtime lib, host suites, `nested_public_evaluation`'s
  known-red set, check, fmt).
- The release receipts: base and candidate canonical builds, the
  forced SHA spot-check, the five C4 owner comparisons, the cost rows
  within the C5 deltas, the microbench A/B verdicts.
- The doc receipts: the host doc subsection and the `CHANGELOG.md`
  entry, with the reference drift test green.
- The `rum-report/1` record.

## Verification (paste verbatim lines)

```
[pre-state] live pinned; capture files at live bytes; witnesses green 1/1 at 8,1 on live
[red calibration] both witnesses 0/1 at 8,1 on the four-change install arm, receipted tuples
[install] nine production paths + two regression paths = arm of record; visibility amendment quoted; witnesses green 1/1 at 8,1
[gates] 13/13 regressions, 1/1 frozen, filters 11/119/26/2/15/9, runtime lib 1432/2/5, host 50/50 x4 + lifecycle 6/6, nested_public_evaluation 14/15 known-red set, check green, fmt three ruled arms
[cost] changed rows within the C5 deltas; microbench A/B no hot-label regression; no equal-call increase outside the named mechanism
[docs] host doc + CHANGELOG.md name the lifecycle contract; reference drift test green
```

## Stop-loss

- Any ownership counterexample, any unproven class row, or any route
  outcome that differs from the settled tuples is a STOP.
- Any need to change an assertion, the frozen test bytes, a C4 record,
  a witness, a protected file, the three capture files, the documented
  host contract, or the public surface is a STOP.
- Two consecutive compiling candidates that move neither a witness nor
  a named measured counter are a STOP.
- A cost row outside the C5 measured deltas on any drawn count — or
  any deterministic per-call increase in a non-changed function — is a
  STOP for the director's cost ruling; the fixed startup and
  draw-varying rows are recorded, never normalized.
- Any need to exceed the ratified canonical build budget is a STOP.

## Checkpointing — protect the scene

Keep every log, receipt and driver under `.temp/brief-735/`; never
overwrite brief 733, 731, the C4 evidence, or any frozen G15 scene.
After each milestone update `.temp/brief-735/PROGRESS.md` with the
completed evidence, the exact next action and the spent/remaining
build budget. Preserve user changes in a dirty worktree; never revert
unrelated edits.

## Report format

`rum-report/1` JSON at `.temp/brief-735/report.json`; every command and
attempt with verbatim verdict lines, a test total for every executed
suite, scope hashes, the red/green calibration tuples, the cost rows,
the doc receipts, and any open decision. Close with
`python3 scripts/report.py summary 735` output and nothing else. Do not
commit; STOP for the director's gate.

## Director gate (2026-11-13)

**Accepted in full.** Run: `mst` (GLM-5.3, session
`dc09d88f-80aa-3927-660d-a27c19c94383`; log `.temp/logs/mst_brief_735.log`,
34,207 B; $14.94, 49,686,627 in / 192,937 out; memory-scope peak
3,595,362,304 B, oom 0, oom_kill 0, rc 0; 220 run records, 9 checks PASS,
3 measurements PASS and one FLAGGED (microbench, settled below);
`report.py check 735` OK with 712 citations by my own run; 5 of the 8
canonical release builds used, 3 unused with no transfer; git-attempts 0,
no network; every write under `.temp/brief-735/`; the tree left
uncommitted as ordered).

**My scope walk.** The post-install walk reads 2,737 files and equals
`final-tree-hashes.json` exactly (0 missing / 0 extra / 0 differing); the
change set against the pinned pre-state is exactly the 16 scoped paths
(12 modified, 4 added, 0 deleted) on top of the authorized eleven-file
brief-731 stage-2 base. Of the nine production paths, eight are
byte-equal to the continuation-5 arm of record and to its after-replay
receipt; the ninth (`session_helpers.rs`) differs from the arm by exactly
line 121 (`pub(in crate::runtime) fn` -> `fn`), and the grep finds two
matches, both in that file, so `finish_report_owner_ticket` is the only
finish reachable outside it. The two witnesses are byte-equal to the arm
and to the C4 probe sources; the four-change files carry their C4
receipt bytes (`session_requests.rs c1257a27`, `suspension/core.rs
0e971f88`, `abandon.rs 0900f344`, both pairing regressions). The three
capture-repair files are at live bytes (`bd67758`, `de397f`, `cbc436`);
the nine protected pins equal both the entry and the arm;
`Cargo.lock`/`Cargo.toml` are untouched; there is no `unsafe`; every
added `pub` is `pub(in crate::runtime)` or `pub(super)`, so the public
surface is unchanged. The inventory delta against the entry TSV is
exactly 68 fingerprint cells with 0 added/removed/reordered, and my own
canonical regeneration reproduces the installed TSV byte-exactly
(`9e7175cc…`).

**My reproductions.** The three-point ladder at eight and one, on my own
builds: the plain-entry copy green with the receipted entry tail
(`ticket_free=false`, `observer=true`); the four-change topology red 0/1
with the receipted tuples (`ticket_free_after_*=true`,
`retirement_cancel_observed=false`, outer prints `["outer-cleanup"]`,
observations 3); the installed tree green 1/1 with the identity tail
(`ticket_free=true`, `observer=false`). The gate set: 13/13 regressions,
the frozen at-entry test 1/1, both C4 controls 1/1, the six filters
11/119/26/2/15/9, the runtime lib 1432/2/5 with exactly lp655/lp656
(aggregate 1434 from the two 1/0/0 child receipts), the four host
targets 50/50 ordinary and profiled, lifecycle cancel 6/6,
`nested_public_evaluation` 14/15 with exactly the known pre-existing red
(`evaluation_context.rs:33`), `check --all-targets` green, `fmt --check`
exactly the three ruled brief-731 arms plus the aliased `outcome.rs`
entry, and the docs reference test green. From my own parse of all
sixteen callgrind captures, every named row is deterministic in every
cell at the C5 deltas (pause +3.0, consume -3.5, abandon +8.0, accepted
entry +4.0, value-entry shard +2.5, resume driver +22.0, hand-off +2.4,
value tail +0.25, reservation mint 0); my equal-call scan finds exactly
those seven rows scaling per call, all on changed functions, while the
remaining increases are constant offsets in libc/std/address rows and
the draw-varying interner rows, recorded and never normalized. I re-ran
the twenty C4 owner comparisons on the receipted release binaries:
20/20 byte-exact against the frozen brief-731 records, no normalization.
The forced canonical rebuild reproduces the candidate binary byte-exact
(`26ce86b9…`).

**Q1 accepted.** The microbench wall-time flags are release-layout
effects, not added work: the instruction lane is the stronger instrument
and reads +0.0113% whole-process (+1,274 of 11,246,081) on the exact
persistent-label workload, with the timed label interiors net -1,472 Ir
and the positive rows being the authorized driver-protocol reshape at
the accepted C5 values; the four remaining wall-time flags are at most
1.65%, inside the documented release-layout envelope, and the canonical
one-codegen-unit panel lane is deterministic at the accepted deltas.

**Deviations accepted:** the plain-entry/identity tail difference is the
two receipted shapes; the empty third inventory replay is a determinism
receipt that my own regeneration confirms; the extra draws and the
instruction control are a stronger instrument for the same bar.
**Disclosures recorded:** the generator's fixed `target/brief-544/`
review copy, and the microbench attempt-1 exec-bit failure redone on the
chmod'ed binary with no source effect.

**State:** the combined install (the eleven-file brief-731 stage-2 base
plus this brief's 16 paths) is committed with this ruling and pushed.
Next: brief 733's continuation on the repaired tree (its old dispatch
log archived first), then brief 731 continuation 6, then continuation 38
of brief 623; then C2's settlement round, CM, `CHANGELOG.md` before S10,
G13 S10-S54/F1-F23/R, E2.
