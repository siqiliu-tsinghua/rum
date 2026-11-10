# Brief 720 — definition-resume logical boundaries: publish the deferred RHS record, sample the declining stage before its consumer, and order the Return recovery (design)

**Shape: design-only. `Scope: NONE` — no source change.** The director's
gate of brief 623's continuation 7 accepted three `Lost` rows and the C7
repair draft. This round turns that draft into a reviewed, executable
plan. You write no code outside `.temp/brief-720/`.

## Goal

Make this true, with evidence, on the current tree: for every
definition-application resume path — down-values, sub-values, up-values —
a valid keyed native observation made on the resumed path equals the
queued path's observation, including

1. **the definition RHS record.** A `down_value_rhs_eval`,
   `sub_value_rhs_eval` or `up_value_rhs_eval` definition record must not
   stay terminal-open when the resumed RHS work completes, errors, returns
   or is abandoned; its ten counters must match the queued path's record
   for the same key.
2. **the declining stage's Option.** A sub-/up-value guard that declines
   must let its own stage sample the actual `None` before the dispatch
   consumer reconstructs a later `Expr`; the stage's `Some`/`None`
   counters must match the queued path.
3. **the repeated-RHS Return.** A second inner RHS suspension that returns
   through the carried recovery owner must close the definition record
   with the post-recovery `Some`, not with `Return`.

The guest language result, evaluation count and order, effects, trace
ordering, request freshness, cancellation/abandonment ownership, public
carriers and the disabled path's cost do not change.

## The measured defects (accepted evidence, 2026-11-10)

Continuation 7 of brief 623 found 147 valid, source-reached bound native
differences, 49 per row on P0267 (`down_values.rs:521`), P0312
(`sub_values.rs:365`) and P0323 (`up_values.rs:451`), every one with
`semantic_same=true`:

- 144 leave the definition record terminal-open: resumed
  `[0,0,0,0,0,0,0,0,0,1]` against queued `[1,1,0,1,0,0,0,0,0,0]`; the
  two-pause case (guard then RHS, or RHS then a second RHS pause) is the
  trigger — one pause alone is SAME.
- 3 repeated-RHS Return cases close with `[1,0,1,0,0,1,0,0,0,0]` against
  the queued `[1,1,0,1,0,0,0,0,0,0]` (Twice ×
  `return_boundary.last-pause-return`).
- 2 minimized declining-stage cases (P0312/P0323 `false_guard`) sample
  `[2,2,0,1,1,0,0,0,0,0]` where queued samples `[2,2,0,0,2,0,0,0,0,0]`;
  the down-value decline control passes, consistent with its explicit
  before-consumer exception.

The minimized set is 21 cases (8 DIFF, 13 SAME); every case keeps
`semantic_same=true` and real == disabled. The director reran the
`optional-real` campaign and reproduced all 470 records, including the
147 native differences. The evidence is frozen at `.temp/g15-c2/c7/`:
`port/loss-category-audit.json`, `port/loss-minimized-witnesses.json`,
`port/loss-return-witnesses.json`, `port/loss-evidence.json`,
`port/fixtures/`, and the draft at
`.temp/g15-c2/c7/draft-repair-definition-resume-logical-boundaries-complete.md`.
Never overwrite that scene.

## Context pointers

- `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`:
  `finish_suspended_down_value_application` (:418), the adopted RHS rule
  record open (:473), the RHS evaluation (:478) and its recording (:479),
  the resume owner (:511-524). The brief-719 repaired neighbour:
  `down_values/default_fallback.rs:29-45` (flush on a second
  `InputRequired`).
- `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs`:
  the analogous adopted RHS start (:322), evaluation (:327), recording
  (:328), typed resume (:356-367).
- `crates/rum-runtime/src/runtime/definitions/application/up_values.rs`:
  the analogous adopted RHS start (:315), evaluation (:320), recording
  (:321), typed resume (:442-453).
- `crates/rum-runtime/src/runtime/definitions/application/rhs.rs:109-121`
  (Return consumption) and `:146-158` (the typed RHS owner mapping the
  recovered result to `Some`).
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs:509-521`
  (`complete_definition_rule`: `InputRequired` defers the record),
  `logical/completion.rs:12-18` (`defer_definition`) and `:22-45`
  (`flush_definition_boundaries`), `logical/mod.rs:1272-1282`
  (`publish_stashed_phases_slow` flushes deferred definitions).
- `crates/rum-runtime/src/runtime/eval_core/call.rs:66` (dispatch
  appends an Option-consuming frame) and `:489` (its `None` mapping);
  `profiling/logical/mod.rs:1526` (down-values inserted before that
  consumer), `profiling/logical/queue_api.rs:8` (the validated
  before-consumer insertion API), and the generic after-consumer
  placement for sub-/up-values.
- The repaired neighboring path's gate is brief 719's; do not regress it.

## Scope

`Scope: NONE — no source change.` Your work lives in
`.temp/brief-720/`: a byte copy of the live tree and probe-only
diagnostic additions made to that copy, never to the live tree.

## What the plan must contain

1. **A closed inventory** of every resumed definition-application typed
   frontier (down-, sub-, up-values, the default fallback, and the shared
   prepared-definition/rule paths), each with `path:line`, the owning
   frame, the actual result carrier, the queue frontier at that point,
   and whether a deferred definition record can exist there. Include
   every caller of every function whose duty would move.
2. **Per defect, a precise insertion point and signature**: the copied
   experiment's exact edit, why it is the actual suspension frontier (not
   merely a convenient return), what it does for `Ok(Some)`,
   `Ok(None)`, `InputRequired`, `Error`, `Return`, cancellation and
   abandonment, and its ordering relative to the recovery owner. A blind
   flush of every typed return is not a design; name what publishes what,
   when, and to whose queue.
3. **The class duty.** Every site that publishes deferred definition
   boundaries or samples an Option before a dispatch consumer: classify
   each as covered by this repair, already correct with the proof, or a
   STOP with its reason. No new call edge is left unreachable.
4. **The falsifiers.** The eight minimized native differences as base-red
   / candidate-green on the copied experiment; the thirteen controls stay
   SAME; all 147 bound witnesses retained; the declining-stage cases and
   the three Return witnesses; and a fresh focused witness per mechanism
   that the implementation must keep. The gate must also catch the
   omission or misplacement of each new publication/sampling tail
   (independently omit it and watch the observer go red).
5. **The disabled-path cost bar.** The three typed frontiers and the
   phase helpers are hot. Measure instructions per call with Callgrind on
   the disabled path before and after each copied edit at identical call
   counts (at least three draws, ranges as the floor); pin the bar and
   name the env-gated cold path that keeps the disabled caller unchanged.
   A measurable default-path increase is a STOP, not a recorded
   deviation.
6. **Risks and open questions**, with the exact question when the
   director must decide something.
7. **The drafted implementation brief(s)** in `docs/dev/codex/TEMPLATE.md`
   shape: staged so each mechanism has its own base-red/candidate-green
   gate; the copied experiment first, the live edit only after the
   director's review.

## Reproduce first

On a fresh byte copy of the live tree with a probe-only diagnostic
package (the C7 `c2-probe` pattern), reproduce the three witnesses from
`.temp/g15-c2/c7/port/loss-minimized-witnesses.json` and
`loss-return-witnesses.json` with the exact ten-counter records, the
declining-stage records and the language observations. A rerun that does
not reproduce them is a STOP with the raw log; never redefine the
witness.

## Hard rules

- No git in any form, no network, no writes outside `.temp/brief-720/`.
- Never edit `crates/`, `docs/`, `scripts/`, tests or inventories on the
  live tree; scratch copies only. Preserve `.temp/g15-c2/c7/` untouched.
- Rust 1.85.0, offline, locked; one cargo process at a time at eight
  build jobs; native diagnostics serial; every command bounded.
- Every claim carries `path:line` or a raw receipt. Copied log lines for
  every verdict.
- Two consecutive executed bounded experiments with no new evidence
  stop the round; failed launches and pure source reviews do not count.
- No `unsafe`, no dependency change, no public API change, no foreign
  engine comparison.

## Deliverable

A `rum-report/1` JSON at `.temp/brief-720/report.json` (written with
`scripts/report.py`, checked before finish) containing: the plan with
the closed inventory table, the three mechanism designs with exact
signatures, the falsifier matrix, the cost rows with raw captures, the
risks and open questions, and the drafted implementation brief(s). Also
leave every raw log, fixture and copied-experiment diff under
`.temp/brief-720/` and name them in the report.

## Verification of this design round

- The witnesses reproduce on the copy with exact raw records; a negative
  control (a deliberately wrong read of one field) must fail the same
  check.
- Each proposed insertion point is exercised in the copy: red before,
  green after (or the measured reason it cannot be green yet), quoted
  from raw logs.
- The disabled-path rows read equal at identical call counts on the
  copied base, with the candidate's range pinned.
- The report cites every claim; `report.py check` passes; every drafted
  implementation brief is TEMPLATE-shaped and names its own gate.

Stop and report; the director reviews the plan, settles the open
questions and writes the implementation brief.

## Director review (2026-11-10) — the design is accepted in substance, and mechanism A's shared hook stops on its own disabled-cost bar

Run 1 (log `.temp/logs/cdx_brief_720.log`, 25,828 lines; memory-scope peak
5.50 GB, rc=0) reproduced the C7 scene on a byte copy, repaired all eight
minimized differences and 147 bound native differences in the copy,
retained every control, armed four independent omission/placement
mutants, and replayed the standing inventories. **Accepted in
substance; one STOP.**

**My own reproductions.** The live tree against the continuation-7 entry
manifest (`cfdd9ec3…`) has 0 missing and exactly my own two ruling
commits differing; no file outside `.temp/brief-720/` and `.temp/logs/`
was written during the run. The accepted C7 scene replays 25,257/25,257
hashes, 0 changed, 0 missing, and its supplement binds the base manifest
hash. The copied base reproduces the 21 minimized records and the 470
frozen records with whole-record equality (21/21, 470/470): 8 DIFF / 13
SAME minimized, 147 valid native differences, two pre-existing invalid
records. Candidate publication leaves exactly the two declining-stage
records (`P0312/P0323.false_guard`) and closes all 147; publication plus
sampling leaves 0 DIFF over all 21 and all 470, with every valid record
five-part SAME and the two invalid records retained. The four mutants
reproduce independently: publication omission restores exactly 147;
sub-only omission restores exactly `P0312.false_guard`; up-only omission
exactly `P0323.false_guard`; before-recovery placement produces 125
native differences and reddens all three Twice/Return witnesses. The
nine valid fresh witnesses are 8 DIFF on base and 0 on the candidate;
the indexed supplement is base-red/candidate-green; 18 lifecycle cells
pass with exact vectors. Inventory replays read the standing values I
hold (fingerprints 68; ownership 61/79/8,825; both certificates GREEN;
census 1,220 = 14/1,168/38; LP01 five modes; typed slots 10/0/1;
frame-root 15/0). The focused runtime-profiling suite is 219/2/2 on the
untouched byte base and 220/2/2 on the candidate with the identical two
failures (`lp655_root_context_falsifier`, `lp656_public_root_context_falsifier`);
I confirmed `lp655` fails on the live tree too, so these are the known
standing root reds, not a copy artifact. The wrong-terminal-open-field
negative control is rejected.

**The STOP — mechanism A's publication hook costs the disabled path.**
The report's cost lane selects 48 physical rows and finds them equal; I
re-parsed all 18 raw captures and confirm every selected row is constant.
But the selection omits the one rum function that contains the changed
call site: `session_requests::<impl Session>::resume_eval_request_from_checkpoint`.
My own captures of the shipped `bin/base` and `bin/publish` at the
campaign's disabled configuration (all fourteen flags zero; the C2
records are empty, so the profiled mode is not selected) read

```text
base     resume_eval_request_from_checkpoint 81,588 instructions / 30 calls
publish  resume_eval_request_from_checkpoint 82,545 instructions / 30 calls  (+957)
```

and the difference is deterministic across every draw of both arms and
across both cohorts; `sample` reads 82,545 as well, so the sampling edit
adds none of it. Two controls attribute it to the publication edit and
not to build noise: (1) a forced rebuild of the untouched `source-base`
produces a different binary sha256 (`0b7cad54…` against `b15fca87…` —
this test target is not byte-reproducible) yet executes exactly
81,588/30 again; (2) a signature-only variant that applies only
`M::pause(self, &profile_context)` and the trait signature, with no
flush, compiles to `0b7cad54…` and executes 81,588/30. Only when
`Profiled::pause` carries `session.flush_definition_boundaries()` does
the disabled caller execute 957 more. In the disabled captures neither
`flush_definition_boundaries` nor `Profiled::pause` executes at all, so
the increase is the enabled-only body shifting the shared inlined
function's register allocation, not new work — but it is executed work
on the disabled path at identical calls, and the plan's own bar says
that is a STOP. In the enabled control the same two arms read 105 flush
calls / 15,633 instructions and 117 / 17,349 — the new flush work
itself, which is the intended enabled cost and not at issue. The 48-row
selection could not have seen this: it selects by helper-name substrings
and the driver's generic hook has no standalone symbol.

**Settled questions.**
- **Q1 — not approved as measured.** The shared `Mode::pause` route is
  refused on the disabled-cost bar, not on shape. Use the published
  `const PROFILED: bool` frame pattern that the brief-719 repair already
  proved (its ordinary instantiation drops the flush call entirely), or
  any other route the continuation measures instruction-identical. The
  revised cost table must include
  `resume_eval_request_from_checkpoint` explicitly, the three typed
  frontiers, the phase helpers, and a same-source rebuild control (this
  target's builds are not byte-reproducible, so equality of executed
  instructions across two builds of one source is the control).
- **Q2 — approved.** The private rename and the two extra phase keys are
  the intended sampling repair; the sampling edit measures zero
  additional disabled cost in my scan.
- **Q3 — approved.** Lifecycle tests stay beside the runtime profiling
  tests; keyed-native witnesses stay in the installed stdlib oracle; the
  C7 projection harness is not transplanted into stock fixtures.
- **Q4 — a standing obligation, not a prerequisite.** `lp655`/`lp656`
  are the known root-context reds, red on the live tree before this
  line. Keep both assertions unchanged, name them as the only failures
  of the focused suite, and keep their repair separate from this
  definition-resume repair.
- The five deviations are recorded, not gated: the unbounded first
  `cat`, the relocated scratch adapters, the three invalid first fresh
  recipes, the C7-projection fingerprint ordering artifact fixed by
  restoring the stock harness, and the two failed launches.

**Continuation 1 order.** Re-measure the STOP row first and confirm my
numbers. Then revise mechanism A to a route whose disabled caller is
instruction-identical at identical calls, with the `const PROFILED`
frontier pattern as the proven reference; re-run the whole publication
and Return falsifier matrix on the revised route (147 → 0, the three
`[1,0,1,0,0,1,0,0,0,0]` witnesses to `[1,1,0,1,0,0,0,0,0,0]`, the 13
controls, the two remaining decline records still red until mechanism
B, the four independent mutants); keep mechanisms B and C, the closed
inventory, the drafts and every replay unchanged. Cost lanes carry the
caller row and the same-source rebuild control; eight rows of the
repaired shape are not a design if the caller still moves. Stop for my
gate; no question is open.

## Director review (2026-11-10) — continuation 1: the STOP is a
measurement, the round is not a route, and the driver hook is refuted
as a shape

**What I reproduced myself.** The continuation's one executed family
reproduces exactly from its twelve raw captures with the canonical
reader: shipped base 81,588 instructions / 30 calls in all three draws,
the refused publication 82,545 / 30 in all three, and the forced
same-source rebuild 81,588 / 30 with all 60 required rows equal. The
live tree matches the continuation's entry manifest (2,710 files, 0
changed, 0 missing, 0 extra), the C7 scene is untouched (25,251
entries), no git or network ran, and `report.py check` verifies 899
citations. Scope is clean.

**The attempted revision was not one.** The cold/noinline variant is
codegen-identical to the refused publication: `publish` inlines
`Profiled::pause`, `cold` outlines it at `0x144d30` (0x26 B), and both
execute the identical caller row, 82,545 / 30, with an identical
executed-opcode histogram. LLVM had already declined the `#[inline]`
hint, so the attribute pair changed nothing and could not have.

**The obvious structural alternative is worse, measured by me.**
`.temp/director/probe-720/src-outline` is the publication tree with the
profiled driver instantiation moved into an `#[inline(never)] fn
drive_frames_profiled<O>`; its binary is `7f99edee…` and its three
serial captures read 82,563 / 30 in every draw — +975 against base.
The caller's call multiset is unchanged (279 direct calls in base, 280
in publish, one extra indirect), so the +957 is not new work: it is
code the disabled path executes, laid out differently, and it moves
with any change to the profiled arm.

**Ruled.** The driver's own code is shared by both instantiations. The
const-PROFILED literal shape is what the `M` type parameter already
provides; it is the refused publication under another spelling and is
not to be retried. No wrapper, attribute or const in `drive_frames`
can restore the row, and three measurements now say so. The next route
must invoke the publication from enabled-only code that is not part of
the disabled path — riding an existing call edge at the frontier with a
body-neutral call-site change, or a frontier whose machinery already
owns the session — and it must prove the caller row back at exactly
81,588 / 30 at 30 calls, three draws, with the same-source rebuild
control. The bar stands: a measurable default-path increase is a STOP,
not a recorded deviation. The goal is fixed; the route is the worker's
choice, and a route that silently changes when the flush runs is a
STOP.

The full publication/Return matrix was not re-run, which is correct
under the STOP, and it must be re-run on the route that passes.
Mechanisms B and C, the closed inventory, the drafts and the seven
replay receipts are unchanged and were not touched. The continuation's
two deviations are recorded, not gated: the route departed from the
order's reference pattern, and its first whole-runtime reducer was
stricter than the three-draw range floor; the corrected reduction
keeps every row.

**Continuation 2 order.** Do not retry the two measured dead ends
(attributes on `Profiled::pause`; outlining the profiled driver). Find
a route whose disabled caller is instruction-identical at identical
calls, measure it with the caller row, three draws per arm and the
same-source rebuild control, and only then re-run the whole publication
and Return falsifier matrix (147 → 0, the three
`[1,0,1,0,0,1,0,0,0,0]` witnesses to `[1,1,0,1,0,0,0,0,0,0]`, the 13
controls, the two remaining decline records still red until mechanism
B, the four independent mutants), keeping mechanisms B and C, the
closed inventory, the drafts and every replay unchanged. Stop for my
gate; no question is open.

## Director review (2026-11-10) — continuation 2: the route is accepted;
publication rides the enabled-only profiled queue join

**What I reproduced myself.** Every required number re-derives from the
raw files with the canonical reader. **Cost:** the explicit caller
`resume_eval_request_from_checkpoint` is 81,588 instructions / 30 calls
in all three draws on base, the forced same-source rebuild, publication
and sampling; the indexed cohort is 86,950 / 32 on its four arms,
point-equal by draw. All 60 required rows are constant across arms and
draws, and my own parse matches the submitted tables on every cell (0
mismatches). My sweep of every `rum_runtime` row outside the 60 finds no
deterministic equal-call difference against base in any arm.
Whole-process totals vary as before and are recorded, not gated.
**Records:** base 21 minimized 8 DIFF / 13 SAME, publication 2 DIFF
(exactly `C7.loss.P0312.false_guard` and `C7.loss.P0323.false_guard`),
sampling 0; base 470 frozen 147 valid DIFF / 468 valid / 2 invalid,
publication 0, sampling 0. The 470-record outputs are byte-identical to
the accepted run-1 sampling records, and the base outputs to run-1's
base. Mutants: omit-publication 147, before-recovery 125, omit-sub 1,
omit-up 1, and in each the changed-id set equals its DIFF set exactly.
The three Return witnesses read `[1,0,1,0,0,1,0,0,0,0]` under
misplacement and `[1,1,0,1,0,0,0,0,0,0]` repaired. **Actual edge:** in
the raw probe log all 147 bound ids have at least one queue-join
publication, the typed frontier target and a nested pause; the saved
reach file is exactly that id set. **Builds:** the four arm receipts
hash-match their binaries and assembled trees; base and the forced
rebuild share tree, toolchain, command and path, the rebuild is
`cache_hit=false` with `cargo_exit=0`; the rebuilt binary is
byte-identical to the old one — a deterministic recompilation with fresh
executed counts, not a cached artifact. **Scope:** the live tree matches
the entry manifest (2,710 files, 0 changed, 0 missing, 0 extra); the C7
scene is 25,251 files with equal size+mtime and the five frozen
witnesses byte-equal; the 62 distinct commands contain no git and no
network; `report.py check` verifies 1,552 citations. Source deltas:
base→publication exactly two files, base→sampling four, base→maintenance
those four plus the two tests and six generated inventories; driver, the
mode trait, storage dispatch, the three typed frontiers and Return
recovery are byte-identical.

**Why the route satisfies the ruling.** The publication lives in
`ProfileQueue::append`, reached only from the cold, non-inlined
`append_profiled` and only when the storage is profiled, so the disabled
path executes nothing new: the 24 disabled captures show zero
instructions in the flush, the publisher, the shared constructor and
`Profiled::pause`, and the caller row is exactly where it was. On the
enabled side it is the queue's own insertion shape — `Context::capture`,
a charged `Kind::Boundary` cell, `deferred=false`, at the nested
frontier after the RHS recovery and before the carried suffix join. The
owner lookup matches root and epoch in the existing registry, drains
only that control and generation, manufactures nothing when ownership is
missing, and leaves foreign records deferred; the dedicated foreign test
proves it across all three families and both joins. The shared
one-record constructor has exactly two callers, the existing Session
publisher and the new join, and the extraction is behaviorally identical
to the loop it replaces.

**Recorded, not gated.** (1) The forced rebuild happens to reproduce the
old binary byte-for-byte; the control is the fresh compile and the fresh
executed counts. (2) The owner lookup is a linear scan under the
existing registry mutex on the enabled-only append; no enabled
performance claim is made, and the default path carries none of it. (3)
The campaign arms carry the scratch probe module and native fixture
while the maintenance arm does not; all cost arms share one scaffold, so
the A/B is fair, and the installed tree is the four production files.
(4) The native fixture and the lifecycle test are scratch in this round;
the implementation installs the foreign and lifecycle regressions and
re-runs the campaign from a copy.

**Ruled.** The route is accepted: it meets the hard line (enabled-only,
caller 81,588/30 with three draws and a genuine same-source control), it
repairs the whole matrix, and it is the queue API's own insertion path
rather than a new mechanism. Mechanism B remains the accepted private
sampling change, byte-identical to its accepted records. No question is
open. Next: my implementation brief 721 installs publication and Return
first and sampling second, with the permanent foreign and lifecycle
regressions, the canonical inventory refreshes and the full acceptance
(the caller-inclusive 60-row cost lane, the 470-case rum-vs-rum matrix,
the focused and workspace suites, the microbench lane). The two dead
ends stay dead, and the historical STOP stays visible.
