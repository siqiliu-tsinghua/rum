# brief-623 — G15-C2: settle the census rows no later step owns

Scope: **NONE — no source change.** All work is scratch: copied trees,
probe crates and mutant builds under `.temp/brief-623/`.

## Goal

brief-621 settled 198 of its 2 186 census rows by execution. It gave every
other row an owner: 964 rows belong to G13 steps that will convert their
code, and **1 024 rows belong to G15-C2, because no later step will touch
them.** This brief settles those 1 024, on the current tree (`f205e535`,
after G15-C1).

When you are done, every C2 row carries exactly one of the outcomes below.
No row is left without one; a row you cannot settle is reported as a
residual, with its evidence.

- **`Equal`**: every required tuple is SAME on the real build, **and** the
  row's own tail-skip mutant is caught by at least one of them. This is
  brief-621's measured-observer rule, unchanged.
- **`Lost`**: a valid real-build DIFF, minimized, with its mechanism, the
  skipped tail at `path:line`, and a repair design drafted as a separate
  brief. A new repair group must land **before** C2 can close.
- **`Unreachable`**, **`Boundary`**, **`Retired`** or **`Duplicate`**, with
  the evidence each needs (see *Settlement rules*).
- **`NoObservableTail` (proposal)**: the mutant was armed in every required
  tuple and survived in all of them, phase observation included, *and* a
  source argument explains why skipping the tail cannot be observed. This
  is **not** `Equal`. Group these proposals by mechanism; I rule on each
  group.
- **Residual**: still owned by G15-C2, with what was tried and what is
  missing. C2 does not close while any residual remains.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`.
- **brief-621** is the governing design. Read its director continuations
  1-3 and its director review. Its report defines the population, the
  25-policy enum, the context recipes, the mutant schema, the ledger and
  `OwnedObligation`. Do not restate them; build on them.
- **The scene** (read-only, copy what you adapt):
  - `.temp/brief-621/continuation3/remaining-owned-rows/G15-C2.tsv` lists
    the 1 024 rows;
  - `continuation3/c2-remaining-composition.json` gives their composition;
  - `owned-obligations.tsv` holds the initial partition evidence;
  - `continuation3/{mutant-schema,policy-mutant-shapes,required-tuples,tuple-gaps,mutation-baseline-receipts}.tsv`,
    `mutations-*.tsv` and `row-mutants/` are the prototype's evidence;
  - `build_schema.py`, `semantic_schema.py` and `row_mutants.py` are its
    generators;
  - `settled.tsv` holds every row's current state;
  - `.temp/brief-621/draft-G15-C2-settlement.md` is the raw draft. This
    brief governs where they differ.
- **The composition you start from** (brief-621 continuation 3):

  | Class | Rows |
  | --- | ---: |
  | unsupported schema | 364 |
  | survived, or no mutation hit | 450 |
  | compiled, but no valid binding reached | 190 |
  | caught mutant, incomplete real tuples | 20 |

  Of the 1 024 rows, 742 have no partition link, 310 lie in code whose
  G13 step is already done (S1-S9), and one is a shared helper with no
  unique owner. The 29 C2 rows already settled stay settled.
- **What changed since brief-621.** G15-C1 (`b4f81ebe`) changed
  `kernel_data/rewrite_data/replace/all.rs` and added
  `replace/all_application.rs`. Cross-walk P0941/P0946 and any row in those
  files to their new sites.

## Scope

- **Files you MAY modify**: none under `crates/`, `docs/` (except appending
  your report to this brief), `scripts/`, or anywhere tracked.
- **Scratch.**
  - All scratch goes under `.temp/brief-623/`: copied trees, mutant
    builds, probe crates outside the workspace (depending on `rum-host` by
    path), generators, logs and tables.
  - Do not add a scratch test under `crates/`.
- Do not edit `.temp/brief-548/`, `.temp/brief-583/`, `.temp/brief-618/` or
  `.temp/brief-621/`.

## Steps

1. **Rebase and calibrate.**
   - Copy brief-621's census tooling and mutant generators. Re-run them on
     `f205e535` and cross-walk all 1 024 C2 rows to current sites. A row
     that moved is paired; a row whose site is gone is `Retired`, with
     G15-C1 cited.
   - **A new edge is a new row.** If C1 introduced a new producer edge, it
     enters the population. C1's own frames are already settled by C1's
     fixtures and mutants; cite them.
   - **Calibrate before settling.**
     - The real build must report DIFF on the G15-A, G15-B and G15-C1
       witnesses against `.temp/brief-619/base-source/` (pre-G15) or
       `.temp/brief-622/base-source/` (pre-C1), and SAME on the current
       tree.
     - Each mutant shape must be caught on a row with a known tail. The
       four semantic mutants (`Some`, G15-A's Return finish, G15-B's
       recovery, G15-C1's finish) are those controls.
2. **Freeze the requirements.** Before any probe factory runs, record each
   C2 row's policies, variants, phases and consumer routes from source.
   Correct a classifier error with evidence. Never remove a context because
   it blocks promotion.
3. **Settle by class, ordered by leverage.** Work family by family, the
   family that settles the most rows first. Park a resistant row after one
   bounded attempt and move on.
   - **Caught mutant, incomplete tuples (20).** Complete the missing
     contexts.
   - **Compiled, no valid binding reached (190).** Build the native or
     adapter driver the row needs: a host call or timeout, a script, a
     CLI/REPL/Jupyter request lifecycle. If a bounded search cannot reach
     it, give a call-site `Unreachable` proof.
   - **Survived, or no mutation hit (450). Split this class first.**
     - *Never armed*: no probe paused in this producer while the mutant was
       selected. That is missing coverage. Reach the producer, or give an
       `Unreachable` proof.
     - *Armed and survived*: a stronger context may observe the tail; try
       every context the row's policy allows, phase observation included.
       If one catches it, the row can become `Equal`. If none does, propose
       `NoObservableTail` with the source argument, and group it.
   - **Unsupported schema (364).** Write row-specific mutants:
     - at an attached frame, forward its actual input before the tail,
       armed by this producer's pause;
     - at a frameless tail, skip the ordinary finish after the actual
       producer, respecting its enclosing closure and return boundary.

     Where carriers differ, omit a concrete phase rather than fabricate a
     default. If no mutant shape exists for a row, say why, and the row
     stays a residual.
4. **Any valid real-build DIFF.** Minimize it, find its mechanism and
   group it. Draft the repair as a separate brief in `TEMPLATE.md` shape,
   under `.temp/brief-623/`. Keep settling the other rows. The repair's
   production change is a later brief of mine, and it precedes C2's
   closure.
5. **Revisit parked rows** within the stop-loss. Then report the exact
   residual list and its composition.
6. **Ready the port.** For every settled row, record its tuples, its
   fixture programs, its mutant and the receipt, in the shape brief-621's
   manifest design expects. The manifest brief ports them.

## Settlement rules

- **`Equal` needs both halves.**
  - Every required real-build tuple SAME.
  - The row's own mutant caught by at least one of them, with final values
    allowed to differ or not.
  - Another row's mutant, an unrelated value corruption, or a generated
    `caught=True` field without raw evidence does not count. Verify each
    proposed `Equal` against the raw JSON and logs.
- **`Unreachable`** pins a call site whose callee chain cannot evaluate
  guest code or produce `InputRequired`. It carries the strongest probe
  that tried, with its observed pause count. This is the class brief-621's
  review accepted. A zero counter or an unregistered handler alone is not a
  proof.
- **`Boundary`** cites a documented non-resumable boundary or ruling.
- **`NoObservableTail`** needs three things:
  - the mutant armed in every required tuple;
  - every required tuple SAME under it, phase observation included;
  - a source argument naming what the tail does and why no context can
    see it (for example, an `Arc`-sharing optimization that rebuilds an
    equal value).

  It is a proposal, grouped by mechanism, for my ruling.
- **Internal-encoding contexts.** Reuse brief-621's eight saved
  `e1-guest_heads.*` batches and their script-only classification. Run no
  new probes in that family. A required tuple that only a new
  internal-encoding run could cover is listed as **blocked**, not dropped
  and not treated as SAME.

## Verification (MANDATORY — the evidence is the deliverable)

Paste verbatim:

- the rebase cross-walk counts and the calibration checks;
- the frozen requirement counts per policy;
- per batch, the probe totals (SAME/DIFF/invalid), the mutant totals
  (armed/caught/survived) and the outcomes gained;
- the final outcome totals, overall and per class and source population;
- every `Lost` witness, every new `Unreachable` proof and every
  `NoObservableTail` group in full;
- the residual list.

Run the ledger model's negative controls on the final state:

- C2 marked done while residuals remain;
- deletion and duplication of a row;
- a wrong-row mutant;
- a surviving identity mutant promoted;
- an incomplete matrix promoted.

Each must be red.

`settled.tsv` (your copy) holds one line per C2 row.

## rum / Rust pitfalls (read twice)

- **Calibrate before believing.** A comparator or mutant shape that has
  never been seen to fail is not known to work. Step 1's controls come
  first.
- **A surviving mutant is information, not a pass.** It means either the
  tail cannot be observed or our contexts are blind. Decide which, by a
  stronger context or a stated argument, and never by silence.
- **Unsupported is not `Equal`.** Neither is an unarmed mutant.
- **Diagnostic guards and switches live only in copies.** The real build's
  default path is untouched.
- **Content filter.** These are semantic equality probes. If an upstream
  filter interrupts the run, checkpoint and stop; do not reword probes.

## Checkpointing — protect the scene

- Checkpoint `.temp/brief-623/PROGRESS.md` and `state.json` after every
  batch and every 100 settlements. Record the counts, the next family or
  row, the build state and source integrity.
- Append report sections to `.temp/brief-623/REPORT.md` as they become
  final, and copy the whole file into this brief at the end.

## Stop-loss

- **Batches.** A batch is one family, or at least 200 executed probes.
- **Progress.** Progress means rows that gain a frozen requirement, a
  coverage cell, an armed mutant or an outcome.
- **When to stop.** STOP only after two consecutive batches that gain
  none, or on a calibration failure you cannot explain from source.
- **Hard rows.** A hard row is parked, never a reason to stop.
- **Compile errors.** Reducing compile errors in a mutant build is
  progress.

## Hard rules

- **No `git`, in any form.** No network, no new `unsafe`, nothing outside
  this repository.
- **No tracked file is modified** except appending the report to this
  brief. Verify the tree's hashes at the end.
- Rum against rum only (`AGENTS.md` Testing Policy). Never weaken a
  contract, fake a receipt, or record a verdict the evidence does not
  carry.
- Paste real output. Say plainly what did not run.

## Report format (append to this brief)

1. **Rebase and calibration.**
2. **The frozen requirements.**
3. **Outcomes**: totals by class and source population; `Lost`,
   `Unreachable` and `NoObservableTail` in full.
4. **New loss groups**, with their drafted repair briefs, and whether S10
   can wait for them (it cannot, if C2 must close first).
5. **The residual list.**
6. **The port tables.**
7. **Open questions**, each with a recommendation.

## Director continuation 1 (2026-10-19) — resume from the checkpoint

Run 1 ran for about 72 minutes, then ended on an upstream service
interruption. Its log is kept as `cdx_brief_623.filter-1.log`. Its scene is
intact and accepted as checkpointed in `.temp/brief-623/PROGRESS.md`,
`state.json` and `batches.jsonl`:

- 1 032 rows (the 1 024 plus eight new graph edges) with 75 120 frozen
  tuples;
- calibration passing: 32/32 parity, and the four controls plus C1's shape
  red;
- 20 `Boundary`, 3 `Unreachable` and 3 `NoObservableTail` proposals;
- every batch through `revisit-adapters`.

This is a fresh session. Read this brief, then the checkpoint, and continue
from it. Do not regenerate completed batches.

**Two rulings from the director's look at the scene.**

1. **The 21 real-build "DIFF" rows in `batches/survived/real.jsonl` are
   not losses.**
   - All of them are `phase_observation.trace` probes on timing
     descriptors (`AbsoluteTiming.body`, `TestReport.*`, report capture).
     They differ only in elapsed durations inside the trace (for example
     `60521/500000000` against `21779/250000000`).
   - brief-618's observation model, which this line accepted, excludes
     elapsed durations from comparison.
   - Mask them in the trace comparison as the plans' `duration_paths` intend,
     rerun those 21, and record the result. Only a difference that survives
     the mask is a loss candidate.
2. **Report the armed-and-survived class by mechanism.**
   - In the `survived` batch, 16 734 of 23 145 armed tuples survived, and
     only 2 rows were caught. That is this brief's central finding so far.
   - Group the proposals by the tail's mechanism, for example:
     - an `Arc`-sharing rebuild of an equal value;
     - an `evaluate` the fixed point redoes with an identical trace;
     - an identity wrapper;
     - a profiling label.
   - Each group gets one source argument, the rows it covers, and its
     strongest context attempt.
   - Do not write per-row narratives. I rule by group.

Everything else in the brief stands, including the stop-loss and the
prohibition on new internal-encoding probes.



## Director continuation 2 (2026-10-19) — classify before probing

The stop-loss fired correctly, and the report is accepted:

- the rebase (1 024 paired and 8 new C1 edges);
- calibration;
- 75 175 frozen requirements;
- 20 `Boundary` and 3 `Unreachable` rows;
- the duration-masked timing reruns (21/21 SAME);
- the survivor mechanism groups.

**Your grouping found the reason this brief stalled, and the reason is my
rules.** 409 of the 450 survivor rows are *strict whole-Result tails*: the
producer's `Result` is the function's (or the arm's) return value, with no
`?`, map, match or other work after it. Such a row has no local tail. Its
identity-shaped mutant cannot differ from the source, so the `Equal` rule's
"own mutant caught" half can never hold for it.

brief-621's review already accepted the rule for this shape: a strict tail
**transfers** its obligation to its caller. That is why 513 such rows left
the population. These 409 stayed only because the census found no
*indexed* caller. They are called dynamically, through builtin registration
tables, trait objects and function pointers. The rows are not unsettled;
their transfer target is unresolved.

**Rulings.**

1. **A new disposition: `Transferred { to }`.** A row is `Transferred` when
   all three of these hold, each checked mechanically and recorded:
   - **(a) Strict tail.** The AST shows the producer's call as the whole
     return value of its function or match arm. There is no `?`, no
     `map`/`and_then`/`map_err`, no match on it, and no statement after it in
     that arm.
   - **(b) No observable drop.**
     - List every local still live at the tail whose type has a `Drop` impl.
     - Plain data, `Expr`, `Arc` and collections of them are neutral.
     - A guard or scope type whose `Drop` touches `Session` state (a scope
       restore, a profiling record, a lease) makes the row a `ScopeCleanup`
       obligation instead. It is then not `Transferred`; it stays a
       residual with that policy.
   - **(c) Named callers.**
     - For indexed callers, name their rows.
     - For dynamic callers, resolve the dispatch: the registration table
       entry (`BuiltinSpec` and similar), the trait object's call site, or
       the function-pointer call. Name the census row of that dispatch site.

   The transferred row inherits its targets' outcomes. It counts as settled
   only when every target is settled (or is an `OwnedObligation` of an open
   owner, which it then shares).
2. **Calibrate (a) on known answers before using it.**
   - `try_apply_function`'s status branch before G15-A, and
     `resume_definition_lhs_preparation` before G15-B, must **not** classify
     as strict tails.
   - A known plain forwarding wrapper must classify as one.
   - Use `.temp/brief-619/base-source/` for the pre-G15 shapes.
3. **Apply the classification to all 1 032 rows, not only the survivor
   class.** The unsupported (348), unbound (185) and incomplete (19) classes
   likely contain strict tails too. Report the counts per class after the
   pass.
4. **The identity proposals are accepted as a disposition, `Identity`.**
   - **The two groups.** The whole-Result mode dispatcher (P0506, P0507) and
     the identity exception-RHS frame (P0777): their resumes or branches are
     syntactically `ok(value) => Ok(value)` or a whole-Result return. Their
     90 receipts are kept.
   - **Extend the rule.** Apply it mechanically to any other frame whose
     continuation is exactly that identity, with the same evidence.
   - **P0107 and P0547.** These need their internal-encoding cells only as
     G13-transport's (owned there). Under `Identity` they are otherwise
     settled.
5. **Profiling-result rows (18) need the enabled-count observer**, as you
   recommend.
   - Run their probes with profiling enabled and compare the non-duration
     phase and outcome counts.
   - A difference is a loss in a diagnostic-accuracy class: minimize it and
     draft its repair.
   - A row whose counts agree and whose own mutant is caught by the counts
     is `Equal`.
6. **What remains after 1-5 is the real work.** It is the rows with a
   genuine local tail: adoption and value finishes, closure owners, the
   Return finish, macro routes, fixed-point passes, request packaging, and
   anything else the classification leaves. Probe these with row-specific
   mutants at their true boundary (the closure's outer owner, the concrete
   carrier), as your recommendations say.

**Stop-loss.** As before, keyed on progress. Classification passes count as
batches: a pass that settles or reclassifies rows is progress.

**Report.** Add a section *Continuation 2* at the end. It holds:

- the calibration of (a);
- the dynamic-caller resolution table;
- the drop inventory;
- counts per class and disposition before and after;
- the remaining residual list.

---

## Execution report — brief-623

Evidence is scratch-only. C2 remains open until every residual is resolved and proposals receive a ruling.
**Stopped under the required two-batch stop-loss; C2 is OPEN.** The 1,032-row working population has 20 Boundary, 3 Unreachable, 3 NoObservableTail proposals and 1,006 residuals. No new Equal or Lost outcome is claimed. The separate 29 previously accepted C2 Equal rows remain accepted and are retained unchanged. There are 75,175 frozen requirements. All work resumed from the accepted checkpoint; completed semantic batches were not regenerated.

### 1. Rebase and calibration.

The brief names the target revision `f205e535`; no version-control command was used to inspect it. The production files match the checkpoint's entry hashes. The ordinary call graph has 2,657 old and 2,665 current edges; adding the unchanged 25 supplemental edges gives 2,682 and 2,690. Every one of the original 1,024 C2 rows pairs to its current source site, with no moved or retired C2 row. Eight new C1 edges enter separately. The C1 frames retain their accepted fixtures and mutants from `.temp/brief-622/c1-fixtures.json` and `mutants-final/{all,unheld}/{default,no-default}.log`; these do not fabricate new edge-specific receipts.

```text
GRAPH CROSSWALK {"current": 2665, "missing": 0, "new": 8, "old": 2657, "paired": 2657}
NEW P0942 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:65 self . finish_replace_all_application (replaced)
NEW P0943 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:67 self . suspend_replace_all_application (witness , request)
NEW P0949 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:148 self . finish_replace_all_unheld_application (replaced)
NEW P0950 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:150 self . suspend_replace_all_unheld_application (witness , request)
NEW P0952 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:180 witness . suspend (self , request , ReplaceAllApplicationFrame)
NEW P0953 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:190 witness . suspend (self , request , ReplaceAllUnheldApplicationFrame)
NEW P0956 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all_application.rs:23 session . finish_replace_all_application (replaced)
NEW P0957 crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all_application.rs:46 session . finish_replace_all_unheld_application (replaced)
C2 CROSSWALK {"moved": 0, "paired": 1024, "retired": 0, "rows": 1024}
FROZEN REQUIREMENTS {"encoding_tuples": 17940, "policies": 25, "rows": 1024, "tuples": 74185}
INITIAL CLASSES {"caught mutant; incomplete real tuples": 20, "compiled but no valid binding reached": 190, "survived or no mutation hit": 450, "unsupported schema": 364}
```

```text
C1 CROSSWALK {"current_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "current_line": "62", "current_row": "P0941", "expression": "self . apply_replace_all (target , & rules , RuleApplicationSource :: ReplaceAll)", "old_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "old_line": "59", "old_row": "P0941", "status": "Paired"}
```

```text
C1 CROSSWALK {"current_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "current_line": "81", "current_row": "P0944", "expression": "self . evaluate (replaced)", "old_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "old_line": "63", "old_row": "P0942", "status": "Paired"}
```

```text
C1 CROSSWALK {"current_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "current_line": "145", "current_row": "P0948", "expression": "self . apply_replace_all (target , & rules , RuleApplicationSource :: ReplaceAllUnheld)", "old_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "old_line": "127", "old_row": "P0946", "status": "Paired"}
```

```text
C1 CROSSWALK {"current_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "current_line": "164", "current_row": "P0951", "expression": "self . evaluate (replaced)", "old_file": "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs", "old_line": "131", "old_row": "P0947", "status": "Paired"}
```

In particular, old P0941 is now `replace/all.rs:62`, old P0946 is current P0948 at `:145`; their evaluation-finish edges moved from `:63/:131` to `:81/:164`. New C1 edge IDs use the `C1.` prefix throughout the working ledger, avoiding collision with historical P IDs.

```text
CALIBRATION CURRENT {"DIFF": 0, "SAME": 32, "invalid": 0, "total": 32}
CALIBRATION PRE-G15 {"DIFF": 15, "SAME": 11, "invalid": 4, "total": 30}
CALIBRATION PRE-C1 {"DIFF": 2, "SAME": 0, "invalid": 0, "total": 2}
CALIBRATION DISABLED exact public parity=32/32
SEMANTIC Some: baseline=1/1 SAME; actual omission hit; oracle RED with final values excluded
SEMANTIC G15A: baseline=1/1 SAME; actual omission hit; oracle RED with final values excluded
SEMANTIC G15B: baseline=1/1 SAME; actual omission hit; oracle RED with final values excluded
SEMANTIC C1: baseline=2/2 SAME; actual omission hit; oracle RED with final values excluded
```

```text
KNOWN TAIL SHAPE P0031 baseline=1 SAME; armed own omission caught=1
KNOWN TAIL SHAPE P0343 baseline=1 SAME; armed own omission caught=1
```

The v2 copied Result-closure shape also has its own C1Closure baseline/mutant evidence in `calibration-v2/`. The four semantic controls are independent controls, not borrowed row settlements. The initial pre-G15 panel has four invalid plans, explicitly shown above; only valid DIFF witnesses calibrate A/B.

Director continuation 1 was applied to the scratch E1 comparator only. `duration_mask.rs` preserves trace event order and all non-duration data. It follows declared timing output subtrees and structurally tagged TestReport Timing rules instead of comparing elapsed durations. The 21 identified real-build DIFF programs were rerun after this change; their disabled-copy observations are identical, and 28 frozen cells gained valid current coverage. The unsupported batch's one timing DIFF is the same already-rerun program.

```json
{
  "controls": [
    {
      "control": "Some",
      "baseline_SAME": 1,
      "mutant_red": true,
      "actual_hit": true
    },
    {
      "control": "G15A",
      "baseline_SAME": 1,
      "mutant_red": true,
      "actual_hit": true
    },
    {
      "control": "G15B",
      "baseline_SAME": 1,
      "mutant_red": true,
      "actual_hit": true
    },
    {
      "control": "C1",
      "baseline_SAME": 2,
      "mutant_red": true,
      "actual_hit": true
    }
  ],
  "rerun": {
    "total": 21,
    "SAME": 21,
    "DIFF": 0,
    "invalid": 0
  },
  "exact_copy_parity": 21,
  "coverage_cells_gained": 28,
  "original_raw": "batches/survived/real.jsonl",
  "new_raw": "resume/timing/real.jsonl",
  "production_changes": 0
}
```

No valid non-boundary real-build difference remains established. The two raw adapter-revisit differences are `primary.valueq` boundary fixtures with an added outer sentinel. The complete semantic observations are equal; only the test-only `OracleBoundaryEvidence` projection differs (attempted retired trial input versus prequeued trial input). `oracle/fixtures/boundaries.rs:12-20,37-54,58-103` records InputRequired attempts outside rollback, while `messages/builtins_messages_inspect.rs:77-120` retires the trial through rollback. The adapter is on that retired trial's stack; the external pause is the added sentinel. These are not valid pausing settlements for P0096 and are not new Lost witnesses. Both remain visible in `real-diff-adjudications.tsv`; no tuple was blessed from them.

### 2. The frozen requirements.

The original 1,024 rows/74,185 tuples were frozen before the first factory batch. The eight new C1 edges added 670 tuples. Source review added 260 concrete primary-outcome/schema-mode route tuples; the adapter revisit added five phase tuples and the string revisit added 55. No requirement was removed. The final count is 1,032 rows and 75,175 unique source-route/variant/context cells. `frozen-policies.tsv`, `required-tuples.tsv`, `route-corrections.tsv` and `source-route-gaps.tsv` retain source reasons and pending work. Policy totals below overlap because a row can carry several policies.

```tsv
policy	rows	tuples	encoding_tuples	real_SAME	saved_SAME	missing	contexts	variants
DefinitionLhsPreparation	12	945	240	253	56	636	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;same_type_error_finish;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
MatchingSymbolCycleRecovery	0	0	0	0	0	0	value;error_before_tail;prefix_and_tail;same_type_error_finish;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
OptionalStage	264	20215	5280	6861	1948	11406	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
FunctionResult	10	805	200	60	20	725	value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;phase_observation;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
ReturnBoundary	46	2985	620	962	196	1827	value;return_boundary;error_before_tail;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
ErrorTransform	43	3150	720	539	120	2491	value;return_boundary;error_before_tail;prefix_and_tail;same_type_error_finish	Middle;Twice;Error;AlreadyResumed;Nested
PrefixLoop	55	4470	1100	1996	356	2118	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
DefinitionStore	52	3710	1040	1411	536	1763	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
PrepareArgumentsRestore	33	2420	660	1068	340	1012	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
OptionReader	45	3275	900	1015	320	1940	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
ScopeCleanup	37	2590	340	781	164	1645	value;return_boundary;error_before_tail;held_and_splice;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
CaptureOutput	13	965	260	240	100	625	value;return_boundary;error_before_tail;guest_heads;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
PhaseOwnership	51	3000	560	1146	220	1634	value;return_boundary;error_before_tail;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
MatchDecision	174	13470	3520	3618	1070	8782	value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
RebuildTraversal	273	21525	5500	10325	2918	8282	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
ShapeDecode	141	11420	2820	4382	1372	5666	value;guest_heads;held_and_splice;prefix_and_tail;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
ScalarFinish	31	2280	620	1176	404	700	value;return_boundary;guest_heads;held_and_splice;prefix_and_tail	Middle;Twice;Error;AlreadyResumed;Nested
ConditionalBranch	40	3400	800	1500	496	1404	value;return_boundary;error_before_tail;guest_heads;prefix_and_tail	Middle;Twice;Error;AlreadyResumed;Nested
NativeCallbackBridge	32	2560	640	360	120	2080	value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;phase_observation;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
GuestLoading	3	255	60	138	12	105	value;return_boundary;error_before_tail;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
HostRequestDriver	95	4145	500	838	32	3275	value;return_boundary;error_before_tail;prefix_and_tail;phase_observation	Middle;Twice;Error;AlreadyResumed;Nested
ForwardResult	331	28135	6620	15026	4070	9039	value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;same_type_error_finish;phase_observation;consumer_routes	Middle;Twice;Error;AlreadyResumed;Nested
NonSuspending	0	0	0	0	0	0		
HostBatchBoundary	0	0	0	0	0	0		
RetiredBoundary	0	0	0	0	0	0
```

The seven concrete-route gaps are retained in full. They remain blockers even when the current site matrix is SAME and its own mutant is caught.

```tsv
row	generic_parameter	missing_source_routes	evidence
P0403	Route: DefinitionResumeRoute	ordinary witness versus resumed continuation-token route	definitions/definitions/destructuring.rs:98; prototype keyed the source site rather than both consuming error/continuation routes
P0508	M: AccountingMode	concrete armed and unarmed accounting modes	eval_core/mod.rs:28; site-only guard does not identify the concrete accounting mode
P1147	C: OptionReadContinuation<Option<Expr>>	each concrete next.finish continuation type	options/core.rs:175; source-site route cannot substitute for the continuation-type inventory
P1155	C: OptionReadContinuation<Expr>	each concrete next.finish continuation type	options/read_phase.rs:87; source-site route cannot substitute for the continuation-type inventory
P0709	source finite finish routes	primary-Ok;primary-Err	cleanup.rs:96-106 finish_with_cleanup_outcomes consumes the saved primary Result separately from cleanup Result; original site fragments do not establish both saved-outcome routes
P2299	source finite finish routes	SchemaMode-Errors;SchemaMode-Conform	schema/format.rs::finish_validation branches on state.mode; source entries SchemaErrors and ConformSchema have distinct result finishes
P2604	T and native body closure in HostSession::guard	explicit inventory of guest-call, timed-call, resumed-call and non-resumable restore/script/file body consumers	session.rs:703-724; the ordinary timeout panel was initially blind because projections clear timeout; guard-fresh control catches actual cleanup omission; source-site matrix does not close all body consumers
```

Internal encoding was classified only from the eight saved brief-621 batches. No new probe in that family ran. Their original saved summary follows; the final blocked count is 9,466 after new C1/route requirements were added, with 8,234 saved SAME and 440 saved DIFF cells. The latter remain transport classifications, not independently established skipped-tail losses.

```json
{
  "batches": {
    "e1-guest_heads.nested-current.jsonl": {
      "total": 2994,
      "same": 2955,
      "invalid": 24
    },
    "e1-guest_heads.none-current.jsonl": {
      "total": 2994,
      "same": 2954,
      "invalid": 39
    },
    "e1-guest_heads.some-empty-current.jsonl": {
      "total": 2994,
      "same": 2954,
      "invalid": 39
    },
    "e1-guest_heads.some5-current.jsonl": {
      "total": 2994,
      "same": 2955,
      "invalid": 24
    },
    "e1-guest_heads.tail-nested-current.jsonl": {
      "total": 2994,
      "same": 2781,
      "invalid": 75
    },
    "e1-guest_heads.tail-none-current.jsonl": {
      "total": 2994,
      "same": 2735,
      "invalid": 130
    },
    "e1-guest_heads.tail-some-empty-current.jsonl": {
      "total": 2994,
      "same": 2735,
      "invalid": 130
    },
    "e1-guest_heads.tail-some5-current.jsonl": {
      "total": 2994,
      "same": 2773,
      "invalid": 75
    }
  },
  "new_probes": 0,
  "tuples": {
    "BLOCKED_ENCODING": 9266,
    "SAME_SAVED": 8234,
    "DIFF_SAVED": 440
  }
}
```

### 3. Outcomes.

All totals are execution/evidence totals, not coverage claims. Current working-population totals, disjoint initial classes and disjoint source-membership combinations follow. `source-membership-totals.tsv` additionally counts each source population independently, so those overlapping memberships are not added together.

```tsv
dimension	population	total	Equal	Lost	Unreachable	Boundary	Retired	Duplicate	NoObservableTail_proposal	Residual
ALL	ALL	1032	0	0	3	20	0	0	3	1006
initial_class	caught mutant; incomplete real tuples	20	0	0	0	1	0	0	0	19
initial_class	compiled but no valid binding reached	190	0	0	0	5	0	0	0	185
initial_class	new C1 producer edge	8	0	0	0	0	0	0	0	8
initial_class	survived or no mutation hit	450	0	0	0	1	0	0	3	446
initial_class	unsupported schema	364	0	0	3	13	0	0	0	348
source_population	frame_carries	81	0	0	0	0	0	0	1	80
source_population	frame_carries;review_tail_correct	1	0	0	0	0	0	0	0	1
source_population	frame_carries;screen_optional-source-helpers	3	0	0	0	0	0	0	0	3
source_population	new-C1-edge	8	0	0	0	0	0	0	0	8
source_population	screen_effect-questions	2	0	0	0	0	0	0	0	2
source_population	screen_frameless-direct-evaluate	16	0	0	0	0	0	0	0	16
source_population	screen_frameless-direct-evaluate;screen_frameless-transforms	3	0	0	0	0	0	0	0	3
source_population	screen_frameless-transforms	53	0	0	0	0	0	0	0	53
source_population	screen_optional-source-helpers	12	0	0	3	0	0	0	0	9
source_population	strict_tail_without_indexed_caller	249	0	0	0	2	0	0	0	247
source_population	tool_unsettled	211	0	0	0	9	0	0	2	200
source_population	tool_unsettled;frame_carries	61	0	0	0	0	0	0	0	61
source_population	tool_unsettled;frame_carries;review_tail_correct	1	0	0	0	0	0	0	0	1
source_population	tool_unsettled;frame_carries;screen_frameless-direct-evaluate	6	0	0	0	0	0	0	0	6
source_population	tool_unsettled;frame_carries;screen_frameless-direct-evaluate;screen_frameless-transforms	4	0	0	0	0	0	0	0	4
source_population	tool_unsettled;frame_carries;screen_frameless-direct-evaluate;screen_frameless-transforms;screen_optional-source-helpers	1	0	0	0	0	0	0	0	1
source_population	tool_unsettled;frame_carries;screen_frameless-transforms	16	0	0	0	0	0	0	0	16
source_population	tool_unsettled;frame_carries;screen_frameless-transforms;screen_effect-questions	3	0	0	0	0	0	0	0	3
source_population	tool_unsettled;frame_carries;screen_optional-source-helpers	5	0	0	0	0	0	0	0	5
source_population	tool_unsettled;review_tail_correct	173	0	0	0	6	0	0	0	167
source_population	tool_unsettled;review_tail_correct;screen_frameless-direct-evaluate	9	0	0	0	0	0	0	0	9
source_population	tool_unsettled;review_tail_correct;screen_frameless-transforms	53	0	0	0	0	0	0	0	53
source_population	tool_unsettled;review_tail_correct;screen_frameless-transforms;screen_effect-questions	10	0	0	0	0	0	0	0	10
source_population	tool_unsettled;review_tail_correct;screen_frameless-transforms;screen_optional-source-helpers;screen_effect-questions	1	0	0	0	0	0	0	0	1
source_population	tool_unsettled;review_tail_correct;screen_optional-source-helpers	13	0	0	0	0	0	0	0	13
source_population	tool_unsettled;screen_frameless-direct-evaluate	1	0	0	0	0	0	0	0	1
source_population	tool_unsettled;screen_frameless-direct-evaluate;screen_frameless-transforms	2	0	0	0	0	0	0	0	2
source_population	tool_unsettled;screen_frameless-transforms	8	0	0	0	3	0	0	0	5
source_population	tool_unsettled;screen_frameless-transforms;screen_optional-source-helpers	4	0	0	0	0	0	0	0	4
source_population	tool_unsettled;screen_optional-source-helpers	22	0	0	0	0	0	0	0	22
```

```tsv
source	total	Equal	Lost	Unreachable	Boundary	NoObservableTail (proposal)	Residual
frame_carries	182	0	0	0	0	1	181
new-C1-edge	8	0	0	0	0	0	8
review_tail_correct	261	0	0	0	6	0	255
screen_effect-questions	16	0	0	0	0	0	16
screen_frameless-direct-evaluate	42	0	0	0	0	0	42
screen_frameless-transforms	158	0	0	0	3	0	155
screen_optional-source-helpers	61	0	0	3	0	0	58
strict_tail_without_indexed_caller	249	0	0	0	2	0	247
tool_unsettled	604	0	0	0	18	2	584
```

Including the separately retained 29 accepted Equal rows gives 1,061 known C2 rows: 29 Equal, 20 Boundary, 3 Unreachable, 3 pending proposals and 1,006 residuals. `port/retained-29-equal.tsv` preserves those earlier settlements; they were not rerun or promoted again.

Per execution batch, these are the actual probe and mutant totals. SAME, valid DIFF and invalid are disjoint for real probes. A mutant can correctly fail the planned-pause oracle; `invalid_under_mutation` is reported separately. `armed_no_hit` is neither a caught omission nor a hit-and-survive receipt.

```text
RAW MUTATION AUDIT {"armed": 885, "armed_no_hit": 39, "batch": "mutations-caught", "caught": 739, "hit": 846, "invalid_under_mutation": 353, "rows": 20, "survived": 107, "tuples": 885}
RAW MUTATION AUDIT {"armed": 274, "armed_no_hit": 47, "batch": "mutations-revisit-adapters", "caught": 0, "hit": 227, "invalid_under_mutation": 0, "rows": 56, "survived": 227, "tuples": 274}
RAW MUTATION AUDIT {"armed": 120, "armed_no_hit": 28, "batch": "mutations-revisit-control-flow", "caught": 0, "hit": 92, "invalid_under_mutation": 0, "rows": 24, "survived": 92, "tuples": 120}
RAW MUTATION AUDIT {"armed": 45, "armed_no_hit": 12, "batch": "mutations-revisit-nonlocal", "caught": 8, "hit": 33, "invalid_under_mutation": 0, "rows": 9, "survived": 25, "tuples": 45}
RAW MUTATION AUDIT {"armed": 120, "armed_no_hit": 24, "batch": "mutations-revisit-strings", "caught": 0, "hit": 96, "invalid_under_mutation": 0, "rows": 24, "survived": 96, "tuples": 120}
RAW MUTATION AUDIT {"armed": 23145, "armed_no_hit": 6398, "batch": "mutations-survived", "caught": 13, "hit": 16747, "invalid_under_mutation": 0, "rows": 450, "survived": 16734, "tuples": 23145}
RAW MUTATION AUDIT {"batch": "mutations-unbound", "rows": 0}
RAW MUTATION AUDIT {"armed": 201, "armed_no_hit": 95, "batch": "mutations-unsupported", "caught": 0, "hit": 106, "invalid_under_mutation": 0, "rows": 7, "survived": 106, "tuples": 201}
RAW MUTATION AUDIT {"armed": 180, "armed_no_hit": 0, "batch": "native-guard-mutant", "caught": 0, "hit": 180, "invalid_under_mutation": 0, "rows": 1, "survived": 180, "tuples": 180}
RAW MUTATION AUDIT {"armed": 295, "armed_no_hit": 0, "batch": "repl-mutants", "caught": 240, "hit": 295, "invalid_under_mutation": 0, "rows": 3, "survived": 55, "tuples": 360}
RAW REAL AUDIT {"DIFF": 0, "SAME": 880, "batch": "batches/caught", "invalid": 27, "total": 907}
RAW REAL AUDIT {"DIFF": 0, "SAME": 0, "batch": "batches/unbound", "invalid": 2, "total": 2}
RAW REAL AUDIT {"DIFF": 21, "SAME": 14920, "batch": "batches/survived", "invalid": 101, "total": 15042}
RAW REAL AUDIT {"DIFF": 1, "SAME": 2590, "batch": "batches/unsupported", "invalid": 45, "total": 2636}
RAW REAL AUDIT {"DIFF": 0, "SAME": 180, "batch": "native", "invalid": 0, "total": 180}
RAW REAL AUDIT {"DIFF": 0, "SAME": 3, "batch": "pure", "invalid": 0, "total": 3}
RAW REAL AUDIT {"DIFF": 0, "SAME": 120, "batch": "repl", "invalid": 0, "total": 120}
RAW REAL AUDIT {"DIFF": 2, "SAME": 274, "batch": "revisit/adapters", "invalid": 3, "total": 279}
RAW REAL AUDIT {"DIFF": 0, "SAME": 21, "batch": "resume/timing", "invalid": 0, "total": 21}
RAW REAL AUDIT {"DIFF": 0, "SAME": 120, "batch": "revisit/strings", "invalid": 0, "total": 120}
RAW REAL AUDIT {"DIFF": 0, "SAME": 120, "batch": "revisit/control-flow", "invalid": 0, "total": 120}
RAW REAL AUDIT {"DIFF": 0, "SAME": 55, "batch": "revisit/nonlocal", "invalid": 0, "total": 55}
PROPOSAL RAW VERIFIED P0506 required=30 real_SAME=30 exact_disabled_parity=30 armed=30 actual_hit=30 mutant_SAME=30 phase_observation=present
PROPOSAL RAW VERIFIED P0507 required=30 real_SAME=30 exact_disabled_parity=30 armed=30 actual_hit=30 mutant_SAME=30 phase_observation=present
PROPOSAL RAW VERIFIED P0777 required=30 real_SAME=30 exact_disabled_parity=30 armed=30 actual_hit=30 mutant_SAME=30 phase_observation=present
KNOWN TAIL SHAPE P0031 baseline=1 SAME; armed own omission caught=1
KNOWN TAIL SHAPE P0343 baseline=1 SAME; armed own omission caught=1
EVIDENCE AUDIT PASS: every mutation table claim checked against its own raw JSON and case-scoped arm/hit log; no proposed Equal rows.
```

Batch outcomes gained: caught=0; unbound=0; survived=2 provisional Equal then both withdrawn; identity classification=3 pending proposals; unsupported=0; native=0; source proofs=20 Boundary + 3 Unreachable; REPL=0; adapter revisit=0; timing correction=0 outcomes/28 coverage cells; string revisit=0 outcomes/55 new phase requirements and coverage cells; control-flow=0; nonlocal=0. The two withdrawn candidates are P0709 (saved primary Ok/Err routes) and P2299 (SchemaMode Errors/Conform routes). No raw caught mutant was discarded when their promotion was withdrawn.

The native fresh-request budget observer does catch P2604's actual cleanup omission before a projection can clear its timeout. The large existing raw files were streamed, their real verdict flags and own arm/hit checked, and their hashes saved in `native-guard-fresh-raw-verified.json`. This is a caught mutant with incomplete guard body routes, not Equal.

```text
NATIVE GUARD FRESH RAW verified: real SAME; disabled SAME; mutant DIFF; all valid; P2604 own arm/hit; no promotion because routes remain incomplete.
PORT TABLES {"active_new_equal_rows": 0, "frozen_requirements": 75175, "owned_rows": 1009, "production_manifest_implemented": false, "proposal_actual_receipts": 90, "proposal_fixture_programs": 90, "proposal_rows": 3, "proposal_tuple_bindings": 90, "retained_prior_equal_rows": 29, "settled_proof_rows": 23, "source_entries": 1032}
```

Native boundary checks exercised script/file/restore contracts (three passed); the CLI process was not run for the CLI source proofs. REPL EOF/interrupt/I/O lifecycle checks exercised three cleanup/retirement/fresh-request contracts with real/disabled parity; they are lifecycle receipts, not completed semantic matrices.

```json
{
  "contracts": 3,
  "cleanup_once": 3,
  "retired": 3,
  "fresh": 3,
  "parity": 3,
  "completion_tuples": false,
  "raw_real": "repl/lifecycle-real.jsonl",
  "raw_copy": "repl/lifecycle-disabled.jsonl"
}
```

#### Armed survivors grouped by tail mechanism.

```json
{
  "rows": 450,
  "never_armed_rows": 0,
  "armed_no_hit_rows": 24,
  "own_mutant_caught_rows": 2,
  "fully_hit_surviving_bound_rows": 53,
  "other_partial_hit_rows": 371
}
```

All 450 initial survivor-class rows were armed somewhere. The 24 all-no-hit rows and 371 partial-hit rows must not be described as tails proved unobservable. In the original survivor batch, 23,145 tuples were armed, 16,747 hit, 13 were caught, 16,734 actually hit and survived, and 6,398 armed tuples never entered the omission body. The two caught rows still lack their concrete route matrices. The groups below classify source mechanisms, not settlements; all 450 row assignments and exact source suffixes are in `survivor-mechanisms.tsv`. No separate Arc-rebuild equivalence group was established.

**strict-result — 409 rows.** The source AST marks the selected call as a function/return tail, including selected match/if arms. For a genuine whole-Result tail the ordinary mutant returns its actual Ok payload in Ok, and the Err path remains the original Result. Work before the producer, including symbol resolution, store preparation, retained-prefix adoption and dispatch selection, still runs. There is no new local success value to manufacture. This explains the large identity population, but source caller transfers, concrete routes and the unchanged destructor scope still need their own inventory. A syntactic tail flag alone is not an Equal or a complete NoObservableTail receipt.

Rows: P0008, P0015, P0046, P0050, P0051, P0052, P0053, P0054, P0055, P0056, P0057, P0058, P0059, P0060, P0061, P0062, P0063, P0064, P0065, P0066, P0067, P0068, P0069, P0070, P0071, P0072, P0073, P0074, P0075, P0076, P0077, P0078, P0079, P0080, P0081, P0082, P0083, P0084, P0085, P0086, P0087, P0088, P0089, P0090, P0091, P0093, P0094, P0095, P0096, P0097, P0098, P0099, P0100, P0101, P0102, P0103, P0104, P0105, P0106, P0108, P0109, P0110, P0114, P0126, P0128, P0144, P0186, P0187, P0188, P0193, P0203, P0219, P0232, P0233, P0234, P0236, P0252, P0254, P0255, P0262, P0277, P0283, P0313, P0330, P0331, P0332, P0333, P0334, P0335, P0355, P0358, P0367, P0376, P0377, P0388, P0395, P0399, P0402, P0407, P0423, P0447, P0448, P0465, P0468, P0478, P0506, P0507, P0524, P0526, P0530, P0536, P0538, P0539, P0541, P0542, P0545, P0560, P0575, P0584, P0587, P0599, P0600, P0622, P0624, P0625, P0626, P0708, P0763, P0770, P0774, P0776, P0821, P0822, P0826, P0828, P0904, P0975, P0983, P1005, P1085, P1101, P1102, P1107, P1116, P1124, P1136, P1144, P1146, P1149, P1165, P1187, P1231, P1232, P1239, P1267, P1275, P1283, P1284, P1323, P1324, P1326, P1354, P1496, P1497, P1526, P1535, P1544, P1547, P1556, P1602, P1603, P1605, P1606, P1608, P1609, P1611, P1617, P1618, P1619, P1622, P1634, P1635, P1639, P1642, P1661, P1679, P1680, P1681, P1704, P1729, P1758, P1759, P1760, P1768, P1770, P1771, P1772, P1773, P1774, P1779, P1780, P1812, P1813, P1815, P1816, P1819, P1823, P1833, P1834, P1841, P1847, P1848, P1859, P1860, P1873, P1877, P1879, P1880, P1881, P1882, P1883, P1884, P1885, P1886, P1887, P1888, P1889, P1890, P1893, P1894, P1895, P1897, P1898, P1899, P1900, P1901, P1902, P1903, P1908, P1909, P1917, P1918, P1919, P1923, P1928, P1929, P1940, P1941, P1950, P1955, P1962, P1963, P1973, P1974, P1977, P1978, P1982, P1984, P1996, P2009, P2010, P2011, P2013, P2015, P2017, P2018, P2020, P2022, P2023, P2025, P2070, P2080, P2082, P2083, P2096, P2098, P2099, P2100, P2117, P2118, P2136, P2144, P2145, P2155, P2156, P2157, P2158, P2182, P2183, P2185, P2187, P2196, P2201, P2202, P2206, P2207, P2223, P2224, P2225, P2226, P2227, P2228, P2236, P2245, P2250, P2263, P2264, P2265, P2266, P2275, P2277, P2278, P2279, P2280, P2281, P2282, P2283, P2284, P2302, P2312, P2329, P2347, P2364, P2369, P2377, P2380, P2388, P2406, P2407, P2410, P2413, P2419, P2423, P2429, P2430, P2431, P2432, P2433, P2434, P2435, P2436, P2437, P2438, P2439, P2440, P2446, P2452, P2453, P2454, P2455, P2456, P2457, P2458, P2459, P2460, P2461, P2462, P2463, P2464, P2465, P2466, P2467, P2468, P2469, P2470, P2471, P2472, P2473, P2474, P2475, P2476, P2477, P2478, P2479, P2480, P2481, P2482, P2483, P2501, P2505, P2506, P2507, P2508, P2509, P2510, P2511, P2512, P2513, P2514, P2524, P2525, P2526, P2537, P2551, P2552, P2565, P2566, P2567, P2568, P2569, P2570, P2571, P2572, P2573, P2574, P2575, P2576, P2590, P2592.

```text
{"complete_real_rows": 91, "count": 409, "group": "strict-result", "missing_real_cells": 4865, "phase_armed": 1964, "phase_caught": 0, "phase_receipts": 1964, "phase_survived": 1570, "required_cells": 32590, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**adoption-or-value-finish — 8 rows.** These are actual success-dependent tails: option-state adoption/cursor advancement (accounting_mode/options/driver.rs:31-80), default selection/fallback, abort fallback normalization, saved-primary/cleanup result selection (cleanup.rs:84-139), failure-template fallback, matcher normalization, sequence-choice consumption and schema-mode finishing. Different carriers and saved state are part of the tail. A preserved Err arm can arm the row without executing a success omission. P0580 is the clearest all-no-hit case. P0709 and P2299 additionally lack their concrete saved-outcome/mode routes. The existing SAME runs do not establish source equivalence for this group.

Rows: P0012, P0249, P0580, P0709, P0840, P0934, P1238, P2299.

```text
{"complete_real_rows": 4, "count": 8, "group": "adoption-or-value-finish", "missing_real_cells": 342, "phase_armed": 15, "phase_caught": 0, "phase_receipts": 15, "phase_survived": 9, "required_cells": 815, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**identity-frame — 3 rows.** builtin_adapters.rs:933-940, hold_thread.rs:25-32 and control_data/exceptions.rs:485-492 attach identity Expr-to-Expr resume frames. Their successful continuation is ok(value)=>Ok(value); forwarding the actual incoming Result preserves its value and error without evaluating another guest expression. P0777 has all 30 required real/mutant tuples, including phase observation. P0107 and P0547 still have saved internal-encoding requirements without the required own-mutant receipts; source equivalence cannot fill those cells.

Rows: P0107, P0547, P0777.

```text
{"complete_real_rows": 3, "count": 3, "group": "identity-frame", "missing_real_cells": 0, "phase_armed": 15, "phase_caught": 0, "phase_receipts": 15, "phase_survived": 15, "required_cells": 165, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**profiling-result — 18 rows.** The successful producer result is ultimately returned unchanged; the suffix records runtime/definition/module timing and outcome labels. Examples are eval_core/mod.rs:145-174, resolution.rs:326-331, eval_core/call.rs:42-65 and :80-90. profiling/timing.rs:375-449 shows that the enabled path changes call/Ok/Err/Some/None/Return/Throw/InputRequired counts as well as elapsed durations. Ordered semantic trace and value parity do not observe those enabled counters. Therefore these rows are candidates for a stronger enabled-count observer, not a proposal that the tail is unobservable. P0487 also lies in a Result closure, so its outer profiling tail is outside an early return inside that closure.

Rows: P0256, P0259, P0266, P0314, P0315, P0413, P0431, P0484, P0487, P0489, P0509, P0511, P0515, P0814, P0817, P0818, P1472, P1507.

```text
{"complete_real_rows": 11, "count": 18, "group": "profiling-result", "missing_real_cells": 118, "phase_armed": 69, "phase_caught": 0, "phase_receipts": 69, "phase_survived": 55, "required_cells": 1115, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**return-finish — 1 rows.** definitions/application/rhs.rs:19-47 attaches the RHS frame and then calls finish_definition_rhs_evaluation; :137-153 absorbs EvalError::Return while preserving other errors and records profiling outcomes. A success-only pass-through preserves the real error branch and therefore cannot establish whether this Return finish is observable on an error route. Retain the source Return/projection and enabled-phase obligations; no equivalence proposal.

Rows: P0287.

```text
{"complete_real_rows": 0, "count": 1, "group": "return-finish", "missing_real_cells": 20, "phase_armed": 0, "phase_caught": 0, "phase_receipts": 0, "phase_survived": 0, "required_cells": 75, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**macro-completion-route — 1 rows.** definitions/definitions.rs:95 invokes finish_rule_lhs as the success arm of resume_typed_or_suspend. The actual producing phase, the macro Result boundary and the typed suspension branch must be distinguished. A successful local return does not test every continuation/error route. Keep the generated shape as partial evidence and the unfilled phases as residuals.

Rows: P0417.

```text
{"complete_real_rows": 1, "count": 1, "group": "macro-completion-route", "missing_real_cells": 0, "phase_armed": 0, "phase_caught": 0, "phase_receipts": 0, "phase_survived": 0, "required_cells": 70, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**closure-owner — 8 rows.** The producer is inside a concrete closure. Examples: control_data/mod.rs:86,97,105,109 are callbacks owned by with_iteration_product_scope; orderless collect/match callbacks have their own owner; session_requests.rs:170 is inside the parse/lower/eval closure whose result is later passed to finish_accepted_value_request. A return inside that closure does not skip its outer owner finish. The current local-success mutation must not be counted as an omission of that outer tail. A row-specific outer-boundary mutation is missing. No identity or unreachability outcome follows.

Rows: P0790, P0795, P0798, P0801, P1310, P1333, P1335, P1562.

```text
{"complete_real_rows": 5, "count": 8, "group": "closure-owner", "missing_real_cells": 180, "phase_armed": 25, "phase_caught": 0, "phase_receipts": 25, "phase_survived": 20, "required_cells": 555, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**fixed-point-pass — 1 rows.** resolution.rs:434 onward consumes an actual evaluate-once result, compares next with state.current, records an EvaluationPass when it changes, and loops. A surrounding fixed point can recover the final value even when this finish is skipped. Trace is the stronger existing observer; a survivor is not proof that every next==current and changed-value route is equivalent. The row has only two currently bound tuples. No NoObservableTail proposal is made.

Rows: P1476.

```text
{"complete_real_rows": 0, "count": 1, "group": "fixed-point-pass", "missing_real_cells": 28, "phase_armed": 1, "phase_caught": 0, "phase_receipts": 1, "phase_survived": 1, "required_cells": 30, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

**request-packaging — 1 rows.** session_requests.rs:293 passes resume_eval_request_from_checkpoint to package_eval_request_outcome. A producer with remaining input returns Err(InputRequired), and the success-only ordinary mutation leaves that path unchanged. The 29 saved mutation tuples are armed without any body hit; it has not omitted the request packaging tail. A genuine native request-result mutation, respecting the request lifetime, is still missing.

Rows: P1567.

```text
{"complete_real_rows": 1, "count": 1, "group": "request-packaging", "missing_real_cells": 0, "phase_armed": 5, "phase_caught": 0, "phase_receipts": 5, "phase_survived": 0, "required_cells": 30, "status": "Mechanism classification; only separately verified identity subgroups are proposals"}
```

#### Every Unreachable proof, in full.

**618.optional-source-helpers.038 — crates/rum-runtime/src/runtime/lowering.rs:168.** Screen identity lower_decimal_atom is crates/rum-runtime/src/runtime/lowering.rs:168-205. It takes only &str, splits a decimal/exponent, validates ASCII digits, parses BigInt values, scales integer numerator/denominator, and constructs Rational/Expr data. Its only fallible helper split_decimal_exponent (:207-220) maps parse failure to EvalError::InvalidInteger; multiply_by_power_of_ten (:222-226) only multiplies a BigInt. Neither helper receives a Session, callback, provider, continuation, or guest expression evaluator. Every explicit error constructor in this closed helper chain is InvalidInteger; no callee constructs or forwards InputRequired. Guest code in a parsed string is evaluated only by a later ToExpression consumer after this lowering helper has returned. The probe supplies a numeric string through InputString while also lowering a decimal literal; its 1 pause is outside this helper. The copied helper entry guard was entered 4 times and was absent at every pause.

```text
{"file": "crates/rum-runtime/src/runtime/lowering.rs", "helper_entries": "4", "line": "168", "probe": "623.pure.decimal", "producer_pauses": "0", "raw_copy": "pure/disabled.jsonl", "raw_log": "pure/disabled.log", "raw_real": "pure/real.jsonl", "row": "618.optional-source-helpers.038", "total_pauses": "1"}
```

**618.optional-source-helpers.054 — crates/rum-stdlib/src/predicates/collections.rs:462.** Screen identity effective_same_test is crates/rum-stdlib/src/predicates/collections.rs:462-474. For each borrowed RetainedOptions item it calls same_test_option_rule_value (:439-449), whose entire chain is an Expr::Call head/arity test and same_test_option_name (:452-460), a string/symbol-name read. It clones the selected RHS as data and filters the literal Automatic symbol. RetainedOptions IntoIterator is the concrete slice iterator in runtime/accounting_mode/options.rs:51-57; its Deref is a slice borrow (:44-48), with no guest dispatch. The helper has no Session or callback parameter and every return is Ok(None/Some(data)); there is no Err constructor. A SameTest Function, including an InputString inside it, remains inert here and is invoked only in the later resume_duplicate_free_same_test consumer. The strongest probe installs a SameTest function that actually pauses in that later comparison. It records 1 external pause, 1 helper entry, and 0 helper pauses.

```text
{"file": "crates/rum-stdlib/src/predicates/collections.rs", "helper_entries": "1", "line": "462", "probe": "623.pure.same-predicate", "producer_pauses": "0", "raw_copy": "pure/disabled.jsonl", "raw_log": "pure/disabled.log", "raw_real": "pure/real.jsonl", "row": "618.optional-source-helpers.054", "total_pauses": "1"}
```

**618.optional-source-helpers.055 — crates/rum-stdlib/src/structure_ops/ordering_sets.rs:584.** Screen identity effective_same_test is crates/rum-stdlib/src/structure_ops/ordering_sets.rs:584-596. It iterates the same concrete borrowed RetainedOptions slice, tests Rule/RuleDelayed head and arity with same_test_option_rule_value (:561-571), reads a string/symbol name with same_test_option_name (:574-582), clones the selected RHS, and filters Automatic. All returns are Ok; there is no Session, callback invocation, provider, error constructor or continuation decode. RetainedOptions iteration and Expr cloning only borrow/share data. Guest SameTest evaluation happens later in resume_set_same_test (:598 onward), after this helper has returned. The strongest Union probe actually invokes a pausing SameTest function there, recording 1 external pause, 1 helper entry, and 0 helper pauses.

```text
{"file": "crates/rum-stdlib/src/structure_ops/ordering_sets.rs", "helper_entries": "1", "line": "584", "probe": "623.pure.same-set", "producer_pauses": "0", "raw_copy": "pure/disabled.jsonl", "raw_log": "pure/disabled.log", "raw_real": "pure/real.jsonl", "row": "618.optional-source-helpers.055", "total_pauses": "1"}
```

These are closed callee-chain proofs, supplemented by actual helper entries and strongest attempts; no zero counter or unregistered handler alone was used. Each strongest attempt had one external pause and zero producer/helper pauses.

#### Boundary settlements.

The 20 rows are native non-resumable script/file or restore routes. HostSession::eval_script rustdoc (`rum-host/src/session.rs:378-394`) and `docs/guide/embedding.md:385-395` require prequeued batch input; eval_file (`:408-417`) delegates to it. Runtime `session_eval.rs:310-369,396` has no remaining-forms continuation. Restore rustdoc (`session.rs:199-217`) abandons restore input, returns Other, retires the prompt and retains partial state. `boundary-proofs.tsv` records each exact call site and whether that native probe actually paused on it. CLI source-chain proofs do not claim direct CLI execution.

```tsv
row	site	route	native_pause_at_site
P1556	crates/rum-runtime/src/runtime/session/session_eval.rs:297	script	True
P1557	crates/rum-runtime/src/runtime/session/session_eval.rs:304	file	True
P1558	crates/rum-runtime/src/runtime/session/session_eval.rs:328	script	True
P1559	crates/rum-runtime/src/runtime/session/session_eval.rs:336	script	False
P1560	crates/rum-runtime/src/runtime/session/session_eval.rs:414	file	True
P1569	crates/rum-runtime/src/runtime/session/session_requests.rs:389	script	True
P1570	crates/rum-runtime/src/runtime/session/session_requests.rs:409	script	False
P1571	crates/rum-runtime/src/runtime/session/session_requests.rs:423	file	True
P1572	crates/rum-runtime/src/runtime/session/session_requests.rs:452	file	False
P2594	crates/rum-host/src/session.rs:227	restore	True
P2597	crates/rum-host/src/session.rs:400	script	True
P2598	crates/rum-host/src/session.rs:427	file	True
P2617	crates/rum-cli/src/eval_tests.rs:59	script	False
P2618	crates/rum-cli/src/eval_tests.rs:60	file	False
P2628	crates/rum-cli/src/main.rs:109	script	False
P2634	crates/rum-cli/src/main.rs:123	file	False
P2638	crates/rum-cli/src/main.rs:243	script	False
P2639	crates/rum-cli/src/main.rs:247	script	False
P2640	crates/rum-cli/src/main.rs:255	file	False
P2641	crates/rum-cli/src/main.rs:259	file	False
```

```jsonl
{"contract_checks_pass":true,"fresh":"Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] })","id":"623.boundary.script","overlap_rejected":true,"owner_retained":true,"pause_count":1,"queued":"Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] })","resumed_attempt":"Failure(HostFailure { code: Other, exit_status: None, message: \"no suspended evaluation is waiting for host input\", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })"}
{"contract_checks_pass":true,"fresh":"Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] })","id":"623.boundary.file","overlap_rejected":true,"owner_retained":true,"pause_count":1,"queued":"Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] })","resumed_attempt":"Failure(HostFailure { code: Other, exit_status: None, message: \"no suspended evaluation is waiting for host input\", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })"}
{"contract_checks_pass":true,"fresh":"Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] })","id":"623.boundary.restore","owner_retired":true,"partial":"Success(HostEvalSuccess { value: HostValue { text: \"1\", typed: Integer(1) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"1\" }] })","restore":"Err(HostFailure { code: Other, exit_status: None, message: \"snapshot restore unexpectedly requested input\", syntax_rendered: None, messages: [], prints: [], trace: [], outputs: [] })"}
```

#### Every NoObservableTail proposal group, in full.

Only the following three rows satisfy the complete proposal evidence rule. They are two mechanism groups, remain OwnedObligations pending the director's ruling, and are not Equal. The corresponding 90 real/disabled/mutant observations, exact producer arm/hit logs and phase variants were checked individually. Their source/setup/answer programs and receipts are materialized in `port/`.

```json
{
  "group": "Identity whole-Result mode dispatcher",
  "rows": [
    "P0506",
    "P0507"
  ],
  "status": "NoObservableTail (proposal); director ruling required; not Equal",
  "source_argument": "crates/rum-runtime/src/runtime/eval_core/mod.rs:20-25 selects the already concrete AccountingShadow mode before the producer, moves that mode and expr into evaluate_once_in_accounting_mode, then returns its whole Result. The selected branch has no success suffix, error mapping, allocation, state update, or caller-owned guard after the producer. Returning the actual Ok payload wrapped in Ok is the same Result. The unchanged Err branch remains the same. The suspended branch does not lose any continuation because the caller contributes none. crates/rum-runtime/src/runtime/eval_core/mod.rs:23-25 is the other whole-Result branch of that same dispatcher. Unarmed is the concrete unit mode moved into the producer. The branch and function end immediately after the producer; there is no owner restoration, guest evaluation, Return absorption, or other success work to skip. The mutation returns the actual Ok value and leaves Err untouched.",
  "required": 60,
  "real_SAME": 60,
  "armed": 60,
  "actual_hit": 60,
  "survived": 60,
  "strongest_context": {
    "context": "phase_observation.trace",
    "variants": [
      "Middle",
      "Twice",
      "Error",
      "AlreadyResumed",
      "Nested"
    ],
    "tuples": 10,
    "probes": [
      "E1.2091.phase_observation.trace",
      "E1.2092.phase_observation.trace",
      "E1.2093.phase_observation.trace",
      "E1.2094.phase_observation.trace",
      "E1.2095.phase_observation.trace",
      "E1.0935.phase_observation.trace",
      "E1.0936.phase_observation.trace",
      "E1.0937.phase_observation.trace",
      "E1.0938.phase_observation.trace",
      "E1.0939.phase_observation.trace"
    ]
  },
  "receipts": "verified-proposal-tuples.tsv",
  "mutants": "mutant-schema.tsv; copied mutant-source and corresponding immutable raw logs"
}
```

```json
{
  "group": "Identity exception-RHS frame",
  "rows": [
    "P0777"
  ],
  "status": "NoObservableTail (proposal); director ruling required; not Equal",
  "source_argument": "crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:485-490 forwards the evaluated RHS through an identity ok(value)=>Ok(value) continuation. Its actual attached EvaluateExceptionHandlerRhsSuspensionFrame, exceptions/resume_frames.rs:177-189, has no fields, ignores session/token, maps Ok(value) to Ok(value) and Err(error) to Err(error), then calls TypedAction::complete. Its row mutant returns TypedAction::complete(outcome) directly. These are identical for both variants of the actual Result, without running, removing, or constructing any guest expression. Neither phase count, Return, Throw, retained state nor a later request can distinguish the two expressions.",
  "required": 30,
  "real_SAME": 30,
  "armed": 30,
  "actual_hit": 30,
  "survived": 30,
  "strongest_context": {
    "context": "phase_observation.trace",
    "variants": [
      "Middle",
      "Twice",
      "Error",
      "AlreadyResumed",
      "Nested"
    ],
    "tuples": 5,
    "probes": [
      "E1.0975.phase_observation.trace",
      "E1.0976.phase_observation.trace",
      "E1.0977.phase_observation.trace",
      "E1.0978.phase_observation.trace",
      "E1.0979.phase_observation.trace"
    ]
  },
  "receipts": "verified-proposal-tuples.tsv",
  "mutants": "mutant-schema.tsv; copied mutant-source and corresponding immutable raw logs"
}
```

Every Lost witness: **none established in this run.** No unsupported, unarmed or surviving row was promoted to Equal.

### 4. New loss groups and repair briefs.

No new valid skipped-tail loss was established, so no new repair brief was drafted. The 21 timing cases are excluded-duration noise under the director's ruling; the two boundary-telemetry cases do not qualify as producer-resumption tuples. This does not establish that the 1,006 residuals contain no further loss. No production repair was attempted. S10 cannot advance while C2 remains open; the standing order remains completed C2, the manifest gate, CHANGELOG, then S10. Any later genuine C2 loss needs its own repair brief and repair before C2 can close.

### 5. The residual list.

The stop-loss fired after the control-flow and nonlocal families. Both ran real probes and selected own mutants but gained no new requirement, coverage cell, armed row, caught row or outcome. Their caught omissions were already caught earlier; they added no settlement.

```json
{
  "family": "control-flow",
  "targets": 38,
  "probes": 120,
  "real": {
    "total": 120,
    "SAME": 120,
    "DIFF": 0,
    "invalid": 0
  },
  "mutants": {
    "rows": 24,
    "tuples": 120,
    "armed": 120,
    "caught": 0,
    "survived": 92
  },
  "gains": {
    "requirements": 0,
    "coverage_cells": 0,
    "armed_rows": 0,
    "caught_rows": 0,
    "outcomes": 0
  },
  "nonprogress_streak": 1
}
```

```json
{
  "family": "nonlocal",
  "targets": 20,
  "probes": 55,
  "real": {
    "total": 55,
    "SAME": 55,
    "DIFF": 0,
    "invalid": 0
  },
  "mutants": {
    "rows": 9,
    "tuples": 45,
    "armed": 45,
    "caught": 8,
    "survived": 25
  },
  "gains": {
    "requirements": 0,
    "coverage_cells": 0,
    "armed_rows": 0,
    "caught_rows": 0,
    "outcomes": 0
  },
  "nonprogress_streak": 2
}
```

The 1,006 residuals comprise 348 initially unsupported, 446 initially survived/no-hit, 185 initially unbound, 19 initially caught/incomplete and eight new C1 edges. There are 36,676 missing real cells among residuals; 528 residual rows have blocked encoding cells, 43 have saved transport DIFF cells, and seven have explicit concrete-route gaps. These overlapping blocker counts are not a partition.

The exact list below records every row, current site, original class, real passing/required cells, and attempted mutant tuple/armed/hit/caught counts. Class codes: U=initially unsupported; S=initially survived/no-hit; B=initially compiled/unbound; C=initially caught/incomplete; N=new C1 edge. Missing codes: T=no compiled type-correct own omission; R=concrete route gap; X=closure-boundary review; A=no own mutant arm; H=armed but no actual omission hit; M<n>=missing real cells; E<n>=blocked internal-encoding cells; D<n>=saved encoding DIFF cells; K=no own caught mutant; P<n>=no armed SAME mutant receipt for that many required tuples (relevant to an unobservable-tail proposal, not a demand that Equal catch every tuple). Full programs, exact pending keys, prior attempts and carrier/source arguments remain in `settled.tsv`, `final-row-evidence.tsv`, `residuals.tsv`, `tuple-gaps-final.tsv` and `survivor-mechanisms.tsv`. Zero attempted mutant tuples means no valid bound current mutant program ran for that row; it is not a bounded-search proof.

```tsv
row	site	initial	real_same_required	attempted_armed_hit_caught	shape	missing
P0001	crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:93	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0002	crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:110	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0004	crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:245	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0008	crates/rum-runtime/src/runtime/accounting_mode/mod.rs:129	S	80/80	60/60/60/0	OrdinarySuccessPassThrough	K,P20
P0010	crates/rum-runtime/src/runtime/accounting_mode/mod.rs:228	U	80/80	0/0/0/0	Unsupported	T,A,K,P80
P0011	crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:26	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0012	crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:31	S	80/80	60/60/40/0	OrdinarySuccessPassThrough	X,K,P20
P0014	crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:59	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0015	crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:80	S	80/80	60/60/44/0	OrdinarySuccessPassThrough	K,P20
P0016	crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:119	U	80/80	0/0/0/0	Unsupported	T,A,K,P80
P0017	crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:164	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0018	crates/rum-runtime/src/runtime/accounting_mode/options.rs:191	U	80/80	0/0/0/0	Unsupported	T,A,K,P80
P0019	crates/rum-runtime/src/runtime/accounting_mode/options.rs:230	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0020	crates/rum-runtime/src/runtime/accounting_mode/retained.rs:88	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0021	crates/rum-runtime/src/runtime/accounting_mode/retained.rs:120	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0022	crates/rum-runtime/src/runtime/accounting_mode/retained.rs:134	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0025	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:104	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0026	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:106	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0027	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:111	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0028	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:111	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0030	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:113	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0032	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:141	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0033	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:142	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0034	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:149	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0035	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:151	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0036	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:153	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0037	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:165	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0038	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:167	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0039	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:176	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0043	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:202	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0044	crates/rum-runtime/src/runtime/accounting_mode/rule.rs:204	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0046	crates/rum-runtime/src/runtime/arithmetic/parts.rs:22	S	60/85	52/52/41/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P0050	crates/rum-runtime/src/runtime/builtin_adapters.rs:25	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0051	crates/rum-runtime/src/runtime/builtin_adapters.rs:34	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0052	crates/rum-runtime/src/runtime/builtin_adapters.rs:43	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0053	crates/rum-runtime/src/runtime/builtin_adapters.rs:52	S	80/85	56/56/36/0	OrdinarySuccessPassThrough	M5,K,P29
P0054	crates/rum-runtime/src/runtime/builtin_adapters.rs:61	S	80/85	56/56/36/0	OrdinarySuccessPassThrough	M5,K,P29
P0055	crates/rum-runtime/src/runtime/builtin_adapters.rs:70	S	80/85	56/56/36/0	OrdinarySuccessPassThrough	M5,K,P29
P0056	crates/rum-runtime/src/runtime/builtin_adapters.rs:97	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0057	crates/rum-runtime/src/runtime/builtin_adapters.rs:105	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0058	crates/rum-runtime/src/runtime/builtin_adapters.rs:113	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0059	crates/rum-runtime/src/runtime/builtin_adapters.rs:149	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0060	crates/rum-runtime/src/runtime/builtin_adapters.rs:158	S	79/85	63/63/54/0	OrdinarySuccessPassThrough	M6,K,P26
P0061	crates/rum-runtime/src/runtime/builtin_adapters.rs:194	S	80/85	65/65/65/0	OrdinarySuccessPassThrough	M5,K,P25
P0062	crates/rum-runtime/src/runtime/builtin_adapters.rs:203	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0063	crates/rum-runtime/src/runtime/builtin_adapters.rs:212	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0064	crates/rum-runtime/src/runtime/builtin_adapters.rs:221	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0065	crates/rum-runtime/src/runtime/builtin_adapters.rs:230	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0066	crates/rum-runtime/src/runtime/builtin_adapters.rs:239	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0067	crates/rum-runtime/src/runtime/builtin_adapters.rs:248	S	80/85	65/65/65/0	OrdinarySuccessPassThrough	M5,K,P25
P0068	crates/rum-runtime/src/runtime/builtin_adapters.rs:257	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0069	crates/rum-runtime/src/runtime/builtin_adapters.rs:266	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0070	crates/rum-runtime/src/runtime/builtin_adapters.rs:275	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0071	crates/rum-runtime/src/runtime/builtin_adapters.rs:284	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0072	crates/rum-runtime/src/runtime/builtin_adapters.rs:293	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0073	crates/rum-runtime/src/runtime/builtin_adapters.rs:302	S	80/85	65/65/54/0	OrdinarySuccessPassThrough	M5,K,P25
P0074	crates/rum-runtime/src/runtime/builtin_adapters.rs:310	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0075	crates/rum-runtime/src/runtime/builtin_adapters.rs:318	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0076	crates/rum-runtime/src/runtime/builtin_adapters.rs:326	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0077	crates/rum-runtime/src/runtime/builtin_adapters.rs:334	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0078	crates/rum-runtime/src/runtime/builtin_adapters.rs:342	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0079	crates/rum-runtime/src/runtime/builtin_adapters.rs:351	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0080	crates/rum-runtime/src/runtime/builtin_adapters.rs:360	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0081	crates/rum-runtime/src/runtime/builtin_adapters.rs:368	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0082	crates/rum-runtime/src/runtime/builtin_adapters.rs:376	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0083	crates/rum-runtime/src/runtime/builtin_adapters.rs:384	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0084	crates/rum-runtime/src/runtime/builtin_adapters.rs:392	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0085	crates/rum-runtime/src/runtime/builtin_adapters.rs:401	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0086	crates/rum-runtime/src/runtime/builtin_adapters.rs:410	S	80/85	65/65/65/0	OrdinarySuccessPassThrough	M5,K,P25
P0087	crates/rum-runtime/src/runtime/builtin_adapters.rs:419	S	4/85	4/4/0/0	OrdinarySuccessPassThrough	H,M81,E20,K,P81
P0088	crates/rum-runtime/src/runtime/builtin_adapters.rs:455	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0089	crates/rum-runtime/src/runtime/builtin_adapters.rs:509	S	53/85	53/53/53/0	OrdinarySuccessPassThrough	M32,E16,K,P36
P0090	crates/rum-runtime/src/runtime/builtin_adapters.rs:518	S	49/85	49/49/49/0	OrdinarySuccessPassThrough	M36,E16,K,P40
P0091	crates/rum-runtime/src/runtime/builtin_adapters.rs:572	S	50/85	51/51/51/0	OrdinarySuccessPassThrough	M35,E16,K,P39
P0092	crates/rum-runtime/src/runtime/builtin_adapters.rs:599	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0093	crates/rum-runtime/src/runtime/builtin_adapters.rs:608	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0094	crates/rum-runtime/src/runtime/builtin_adapters.rs:617	S	68/85	53/53/40/0	OrdinarySuccessPassThrough	M17,K,P37
P0095	crates/rum-runtime/src/runtime/builtin_adapters.rs:626	S	54/85	55/55/44/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P0096	crates/rum-runtime/src/runtime/builtin_adapters.rs:680	S	2/85	2/2/2/0	OrdinarySuccessPassThrough	M83,E20,K,P83
P0097	crates/rum-runtime/src/runtime/builtin_adapters.rs:760	S	60/85	57/57/44/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P0098	crates/rum-runtime/src/runtime/builtin_adapters.rs:786	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0099	crates/rum-runtime/src/runtime/builtin_adapters.rs:794	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0100	crates/rum-runtime/src/runtime/builtin_adapters.rs:802	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0101	crates/rum-runtime/src/runtime/builtin_adapters.rs:829	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0102	crates/rum-runtime/src/runtime/builtin_adapters.rs:865	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0103	crates/rum-runtime/src/runtime/builtin_adapters.rs:874	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0104	crates/rum-runtime/src/runtime/builtin_adapters.rs:883	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0105	crates/rum-runtime/src/runtime/builtin_adapters.rs:892	S	80/85	65/65/45/0	OrdinarySuccessPassThrough	M5,K,P25
P0106	crates/rum-runtime/src/runtime/builtin_adapters.rs:923	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0107	crates/rum-runtime/src/runtime/builtin_adapters.rs:933	S	65/65	45/45/45/0	FrameResumePassThrough	K,P20
P0108	crates/rum-runtime/src/runtime/builtin_adapters.rs:960	S	80/85	65/65/49/0	OrdinarySuccessPassThrough	M5,K,P25
P0109	crates/rum-runtime/src/runtime/builtin_adapters.rs:969	S	80/85	65/65/65/0	OrdinarySuccessPassThrough	M5,K,P25
P0110	crates/rum-runtime/src/runtime/builtin_adapters.rs:978	S	80/85	65/65/65/0	OrdinarySuccessPassThrough	M5,K,P25
P0114	crates/rum-runtime/src/runtime/builtins_context_loader.rs:287	S	50/85	46/46/46/0	OrdinarySuccessPassThrough	M35,E16,K,P39
P0126	crates/rum-runtime/src/runtime/debug_tools/monitor/options.rs:52	S	2/85	2/2/2/0	OrdinarySuccessPassThrough	M83,E20,K,P83
P0127	crates/rum-runtime/src/runtime/debug_tools/monitor/options.rs:61	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0128	crates/rum-runtime/src/runtime/debug_tools/monitor/options.rs:83	S	2/85	2/2/2/0	OrdinarySuccessPassThrough	M83,E20,K,P83
P0144	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:37	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P0145	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:55	U	50/50	0/0/0/0	Unsupported	T,A,K,P50
P0146	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:67	B	0/50	0/0/0/0	OrdinarySuccessPassThrough	A,M50,E20,K,P50
P0186	crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs:77	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P0187	crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs:82	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P0188	crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs:86	S	80/85	55/55/45/0	OrdinarySuccessPassThrough	M5,K,P30
P0193	crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:209	S	79/85	55/55/36/0	OrdinarySuccessPassThrough	M6,K,P30
P0203	crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:460	S	64/85	44/44/0/0	OrdinarySuccessPassThrough	H,M21,E4,K,P41
P0214	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	U	50/50	0/0/0/0	Unsupported	T,A,K,P50
P0216	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	U	80/80	0/0/0/0	Unsupported	T,A,K,P80
P0217	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:79	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0219	crates/rum-runtime/src/runtime/debug_tools/trace/options.rs:19	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P0220	crates/rum-runtime/src/runtime/debug_tools/trace/options.rs:32	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0232	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:46	S	55/75	47/47/36/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0233	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:65	S	3/85	3/3/3/0	OrdinarySuccessPassThrough	M82,E20,K,P82
P0234	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:86	S	55/75	47/47/37/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0235	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:96	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0236	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:99	S	49/75	45/45/36/0	OrdinarySuccessPassThrough	M26,E16,K,P30
P0237	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:123	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0238	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:141	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0239	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:144	U	49/75	0/0/0/0	Unsupported	T,A,M26,E16,K,P75
P0240	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:154	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0241	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:181	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0242	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:202	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0243	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:219	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0244	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:239	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0245	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:243	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0246	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:256	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0247	crates/rum-runtime/src/runtime/definitions/application/default_values.rs:305	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0248	crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs:29	U	44/75	0/0/0/0	Unsupported	T,A,M31,E12,K,P75
P0249	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:71	S	55/75	47/47/36/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0251	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:85	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0252	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:98	S	71/75	55/55/41/0	OrdinarySuccessPassThrough	M4,E4,K,P20
P0253	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:156	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0254	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:164	S	71/75	55/55/41/0	OrdinarySuccessPassThrough	M4,E4,K,P20
P0255	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:202	S	44/80	36/36/36/0	OrdinarySuccessPassThrough	M36,E12,K,P44
P0256	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:231	S	71/80	55/55/41/0	OrdinarySuccessPassThrough	M9,E4,K,P25
P0257	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:258	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0258	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:291	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0259	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334	S	55/80	47/47/38/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P0260	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:362	U	2/75	0/0/0/0	Unsupported	T,A,M73,E20,K,P75
P0262	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:381	S	55/75	47/47/37/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0263	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:384	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0264	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:403	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0265	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:427	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0266	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467	S	55/80	47/47/37/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P0267	crates/rum-runtime/src/runtime/definitions/application/down_values.rs:509	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0270	crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:66	U	57/75	0/0/0/0	Unsupported	T,A,M18,E4,K,P75
P0271	crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:99	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0272	crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:130	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0273	crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:149	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0274	crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs:179	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0275	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:90	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0276	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:105	U	51/75	0/0/0/0	Unsupported	T,A,M24,E12,K,P75
P0277	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:136	S	80/85	60/60/60/0	OrdinarySuccessPassThrough	M5,K,P25
P0278	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:147	U	51/75	0/0/0/0	Unsupported	T,A,M24,E12,K,P75
P0281	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:188	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0282	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:199	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0283	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:272	S	57/75	41/41/0/0	OrdinarySuccessPassThrough	H,M18,E4,K,P34
P0284	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:391	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0285	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:400	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0286	crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:447	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0287	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:19	S	55/75	47/47/36/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0289	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:28	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0290	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:37	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0292	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:64	U	71/75	0/0/0/0	Unsupported	T,A,M4,E4,K,P75
P0294	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:73	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0296	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:82	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0298	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:101	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0299	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:111	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0301	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:147	U	0/75	0/0/0/0	ResumeClosureSuccessPassThrough	A,M75,E20,K,P75
P0302	crates/rum-runtime/src/runtime/definitions/application/rhs.rs:166	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0303	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:58	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0304	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:179	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0305	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:208	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0307	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:227	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0308	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:234	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0309	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:257	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0310	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:279	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0311	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:319	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0312	crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:356	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0313	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:52	S	55/75	47/47/47/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0314	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106	S	55/75	47/47/46/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0315	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186	S	49/75	45/45/36/0	OrdinarySuccessPassThrough	M26,E16,K,P30
P0316	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:204	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0317	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:220	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0318	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:245	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0319	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:265	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0320	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:311	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0321	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:384	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0322	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:403	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0323	crates/rum-runtime/src/runtime/definitions/application/up_values.rs:441	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0325	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs:91	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0326	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs:121	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0327	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs:152	U	42/70	0/0/0/0	Unsupported	T,A,M28,E8,K,P70
P0328	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs:162	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0329	crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs:205	U	42/70	0/0/0/0	Unsupported	T,A,M28,E8,K,P70
P0330	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:49	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	K,P20
P0331	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:60	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	K,P20
P0332	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:66	S	51/75	43/43/28/0	OrdinarySuccessPassThrough	M24,E12,K,P32
P0333	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:77	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	K,P20
P0334	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:133	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	K,P20
P0335	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:137	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	K,P20
P0336	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:150	B	0/75	0/0/0/0	FrameResumePassThrough	A,M75,E20,K,P75
P0338	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:185	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0339	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:201	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0340	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:206	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0341	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:217	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0342	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:222	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0344	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:237	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0345	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:401	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0346	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:423	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0347	crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:448	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0348	crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:312	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0349	crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:335	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0350	crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:357	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0351	crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs:380	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0353	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs:174	U	45/80	33/33/30/0	ResumeTypedSuccessPassThrough	M35,E8,K,P47
P0354	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:166	U	49/75	0/0/0/0	Unsupported	T,A,M26,E16,K,P75
P0355	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:188	S	70/75	50/50/42/0	OrdinarySuccessPassThrough	M5,K,P25
P0357	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:222	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0358	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:230	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P0359	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:251	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0361	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:292	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0363	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:358	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0365	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:426	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0366	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:463	U	49/80	0/0/0/0	Unsupported	T,A,M31,E16,K,P80
P0367	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:539	S	42/75	30/30/0/0	OrdinarySuccessPassThrough	H,M33,E8,K,P45
P0369	crates/rum-runtime/src/runtime/definitions/definition_tags.rs:14	B	0/70	0/0/0/0	FrameResumePassThrough	A,M70,E20,K,P70
P0372	crates/rum-runtime/src/runtime/definitions/definition_tags.rs:57	B	0/70	0/0/0/0	FrameResumePassThrough	A,M70,E20,K,P70
P0376	crates/rum-runtime/src/runtime/definitions/definition_tags.rs:133	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0377	crates/rum-runtime/src/runtime/definitions/definition_tags.rs:150	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0378	crates/rum-runtime/src/runtime/definitions/definition_tags.rs:269	U	0/70	0/0/0/0	ResumeTypedSuccessPassThrough	A,M70,E20,K,P70
P0379	crates/rum-runtime/src/runtime/definitions/definition_tags.rs:293	U	0/70	0/0/0/0	ResumeTypedSuccessPassThrough	A,M70,E20,K,P70
P0380	crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs:26	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0381	crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs:78	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0382	crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs:101	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0388	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:178	S	80/85	60/60/60/0	OrdinarySuccessPassThrough	M5,K,P25
P0394	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:234	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0395	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:238	S	80/85	60/60/60/0	OrdinarySuccessPassThrough	M5,K,P25
P0396	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:253	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0397	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:263	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0398	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:397	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0399	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:488	S	70/70	50/50/50/0	OrdinarySuccessPassThrough	K,P20
P0401	crates/rum-runtime/src/runtime/definitions/definition_updates.rs:516	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0402	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs:95	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0403	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs:106	C	70/70	50/50/50/50	OrdinarySuccessPassThrough	R,P70
P0404	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs:111	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0405	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs:146	U	56/70	40/40/36/0	ResumeTypedSuccessPassThrough	M14,E4,K,P30
P0406	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs:163	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0407	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs:184	S	56/70	40/40/0/0	OrdinarySuccessPassThrough	H,M14,E4,K,P30
P0408	crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs:22	U	0/70	0/0/0/0	ResumeClosureSuccessPassThrough	A,M70,E20,K,P70
P0410	crates/rum-runtime/src/runtime/definitions/definitions.rs:36	C	69/70	49/49/49/35	FrameResumePassThrough	M1,P56
P0413	crates/rum-runtime/src/runtime/definitions/definitions.rs:65	S	70/70	50/50/50/0	OrdinarySuccessPassThrough	K,P20
P0416	crates/rum-runtime/src/runtime/definitions/definitions.rs:91	C	62/70	50/50/50/36	FrameResumePassThrough	M8,E8,P56
P0417	crates/rum-runtime/src/runtime/definitions/definitions.rs:95	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0419	crates/rum-runtime/src/runtime/definitions/definitions.rs:112	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0423	crates/rum-runtime/src/runtime/definitions/definitions.rs:181	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0425	crates/rum-runtime/src/runtime/definitions/definitions.rs:291	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0426	crates/rum-runtime/src/runtime/definitions/definitions.rs:319	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0428	crates/rum-runtime/src/runtime/definitions/definitions.rs:329	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0430	crates/rum-runtime/src/runtime/definitions/definitions.rs:338	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0431	crates/rum-runtime/src/runtime/definitions/definitions.rs:344	S	70/70	50/50/41/0	OrdinarySuccessPassThrough	K,P20
P0433	crates/rum-runtime/src/runtime/definitions/definitions.rs:474	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0434	crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs:39	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0435	crates/rum-runtime/src/runtime/definitions/part_assignment.rs:41	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0436	crates/rum-runtime/src/runtime/definitions/part_assignment.rs:46	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0437	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:28	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0438	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:56	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0439	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:79	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0440	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:98	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0441	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:109	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0442	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:109	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0445	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:132	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0447	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:163	S	75/75	55/55/55/0	OrdinarySuccessPassThrough	K,P20
P0448	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:179	S	75/75	55/55/55/0	OrdinarySuccessPassThrough	K,P20
P0450	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:218	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0451	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:223	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0452	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:234	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0453	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:266	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0454	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:285	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0455	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:305	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0456	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:308	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0457	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:310	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0458	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:329	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0459	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:354	U	39/75	0/0/0/0	Unsupported	T,A,M36,E8,K,P75
P0460	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:359	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0461	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:363	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0462	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:381	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0463	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:462	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0464	crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs:21	U	60/75	44/44/36/0	ResumeTypedSuccessPassThrough	M15,E4,K,P31
P0465	crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs:37	S	60/75	44/44/0/0	OrdinarySuccessPassThrough	H,M15,E4,K,P31
P0466	crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs:62	U	0/75	0/0/0/0	ResumeTypedSuccessPassThrough	A,M75,E20,K,P75
P0467	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:34	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0468	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:47	S	60/85	52/52/50/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P0470	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:76	C	67/75	55/55/55/55	OrdinarySuccessPassThrough	M8,E8,P75
P0471	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:81	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0472	crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:146	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0473	crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs:164	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0474	crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs:55	U	47/70	0/0/0/0	Unsupported	T,A,M23,E4,D8,K,P70
P0475	crates/rum-runtime/src/runtime/eval_call_prepare.rs:259	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0478	crates/rum-runtime/src/runtime/eval_call_prepare.rs:400	S	47/70	39/39/0/0	OrdinarySuccessPassThrough	H,M23,E4,D8,K,P31
P0479	crates/rum-runtime/src/runtime/eval_call_prepare.rs:430	U	47/70	0/0/0/0	Unsupported	T,A,M23,E4,D8,K,P70
P0481	crates/rum-runtime/src/runtime/eval_call_prepare.rs:443	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0484	crates/rum-runtime/src/runtime/eval_core/call.rs:42	S	75/75	55/55/55/0	OrdinarySuccessPassThrough	K,P20
P0485	crates/rum-runtime/src/runtime/eval_core/call.rs:46	C	58/70	50/50/50/49	OrdinarySuccessPassThrough	M12,E12,P69
P0487	crates/rum-runtime/src/runtime/eval_core/call.rs:54	S	70/70	50/50/50/0	OrdinarySuccessPassThrough	X,K,P20
P0489	crates/rum-runtime/src/runtime/eval_core/call.rs:80	S	70/70	50/50/50/0	OrdinarySuccessPassThrough	K,P20
P0492	crates/rum-runtime/src/runtime/eval_core/call.rs:130	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0494	crates/rum-runtime/src/runtime/eval_core/call.rs:163	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0496	crates/rum-runtime/src/runtime/eval_core/call.rs:228	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0498	crates/rum-runtime/src/runtime/eval_core/call.rs:262	U	54/75	0/0/0/0	Unsupported	T,A,M21,E16,K,P75
P0500	crates/rum-runtime/src/runtime/eval_core/call.rs:322	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0501	crates/rum-runtime/src/runtime/eval_core/call.rs:342	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0502	crates/rum-runtime/src/runtime/eval_core/call.rs:390	U	0/75	0/0/0/0	ResumeClosureSuccessPassThrough	A,M75,E20,K,P75
P0503	crates/rum-runtime/src/runtime/eval_core/call.rs:450	U	0/75	0/0/0/0	ResumeTypedSuccessPassThrough	A,M75,E20,K,P75
P0504	crates/rum-runtime/src/runtime/eval_core/call.rs:514	U	0/75	0/0/0/0	ResumeTypedSuccessPassThrough	A,M75,E20,K,P75
P0505	crates/rum-runtime/src/runtime/eval_core/call.rs:563	U	0/75	0/0/0/0	ResumeTypedSuccessPassThrough	A,M75,E20,K,P75
P0508	crates/rum-runtime/src/runtime/eval_core/mod.rs:76	C	30/30	30/30/30/4	OrdinarySuccessPassThrough	R,P4
P0509	crates/rum-runtime/src/runtime/eval_core/mod.rs:146	S	22/30	22/22/17/0	OrdinarySuccessPassThrough	M8,K,P8
P0511	crates/rum-runtime/src/runtime/eval_core/mod.rs:155	S	30/30	30/30/30/0	OrdinarySuccessPassThrough	K
P0515	crates/rum-runtime/src/runtime/eval_core/mod.rs:186	S	30/30	29/29/26/0	OrdinarySuccessPassThrough	K,P1
P0523	crates/rum-runtime/src/runtime/eval_core/symbol.rs:92	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0524	crates/rum-runtime/src/runtime/eval_core/symbol.rs:98	S	22/30	22/22/17/0	OrdinarySuccessPassThrough	M8,K,P8
P0525	crates/rum-runtime/src/runtime/eval_core/symbol.rs:101	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0526	crates/rum-runtime/src/runtime/eval_core/symbol.rs:112	S	22/30	22/22/17/0	OrdinarySuccessPassThrough	M8,K,P8
P0527	crates/rum-runtime/src/runtime/eval_core/symbol.rs:118	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P0528	crates/rum-runtime/src/runtime/eval_core/symbol.rs:129	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0529	crates/rum-runtime/src/runtime/eval_core/symbol.rs:131	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0530	crates/rum-runtime/src/runtime/eval_core/symbol.rs:150	S	22/30	22/22/17/0	OrdinarySuccessPassThrough	M8,K,P8
P0531	crates/rum-runtime/src/runtime/eval_core/symbol.rs:152	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0532	crates/rum-runtime/src/runtime/eval_core/symbol.rs:162	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0533	crates/rum-runtime/src/runtime/eval_core/symbol.rs:175	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0534	crates/rum-runtime/src/runtime/eval_core/symbol.rs:329	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0535	crates/rum-runtime/src/runtime/eval_core/symbol.rs:331	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0536	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:115	S	80/80	60/60/60/0	OrdinarySuccessPassThrough	K,P20
P0537	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:124	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0538	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:139	S	59/75	51/51/51/0	OrdinarySuccessPassThrough	M16,E12,K,P24
P0539	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:142	S	71/75	55/55/45/0	OrdinarySuccessPassThrough	M4,E4,K,P20
P0540	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:145	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P0541	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:148	S	75/75	55/55/55/0	OrdinarySuccessPassThrough	K,P20
P0542	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:150	S	75/75	55/55/55/0	OrdinarySuccessPassThrough	K,P20
P0543	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:187	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0544	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:197	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
P0545	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:317	S	70/70	50/50/50/0	OrdinarySuccessPassThrough	K,P20
P0546	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:319	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0547	crates/rum-runtime/src/runtime/hold_thread.rs:25	S	70/70	50/50/50/0	FrameResumePassThrough	K,P20
P0548	crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs:56	U	60/80	44/44/0/0	ResumeClosureSuccessPassThrough	H,M20,E4,K,P36
P0549	crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs:80	U	0/70	0/0/0/0	ResumeClosureSuccessPassThrough	A,M70,E20,K,P70
P0550	crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs:99	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0551	crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs:118	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0552	crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs:136	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0553	crates/rum-runtime/src/runtime/kernel_data/association_data.rs:736	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P0554	crates/rum-runtime/src/runtime/kernel_data/association_data.rs:748	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0555	crates/rum-runtime/src/runtime/kernel_data/association_data.rs:753	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P0557	crates/rum-runtime/src/runtime/kernel_data/association_data.rs:834	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0558	crates/rum-runtime/src/runtime/kernel_data/association_data.rs:874	C	56/70	40/40/40/36	FrameResumePassThrough	M14,E4,P66
P0559	crates/rum-runtime/src/runtime/kernel_data/association_data.rs:882	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P0560	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs:16	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P0561	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs:27	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0566	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:24	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0567	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:60	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0568	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:202	U	0/55	0/0/0/0	Unsupported	T,A,M55,K,P55
P0569	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:221	U	0/55	0/0/0/0	Unsupported	T,A,M55,K,P55
P0570	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:237	U	0/55	0/0/0/0	Unsupported	T,A,M55,K,P55
P0571	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:257	U	0/55	0/0/0/0	Unsupported	T,A,M55,K,P55
P0574	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:131	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0575	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:141	S	55/55	55/55/36/0	OrdinarySuccessPassThrough	K
P0579	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:180	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0580	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:199	S	55/55	55/55/0/0	FrameResumePassThrough	H,K
P0581	crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:203	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0584	crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:41	S	60/80	52/52/41/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0586	crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:90	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0587	crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:99	S	60/80	52/52/41/0	OrdinarySuccessPassThrough	M20,E12,K,P28
P0588	crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:102	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0590	crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:181	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0591	crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:212	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0592	crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:248	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0599	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:50	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0600	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:63	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0622	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:22	S	80/85	60/60/60/0	OrdinarySuccessPassThrough	M5,K,P25
P0623	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:24	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0624	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:52	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0625	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:56	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0626	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:73	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0639	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:244	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0693	crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:125	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0694	crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:139	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0695	crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:205	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P0696	crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:276	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0697	crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:298	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0703	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:52	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0704	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:53	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0705	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:65	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0707	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:73	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0708	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:75	S	55/55	55/55/41/0	OrdinarySuccessPassThrough	K
P0709	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:84	S	55/165	55/55/55/5	FrameResumePassThrough	R,M110,P115
P0710	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:89	B	0/55	0/0/0/0	OrdinarySuccessPassThrough	A,M55,K,P55
P0711	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:167	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0712	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:199	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0713	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:260	U	0/55	0/0/0/0	Unsupported	T,A,M55,K,P55
P0714	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:275	U	0/55	0/0/0/0	Unsupported	T,A,M55,K,P55
P0715	crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:291	U	0/55	0/0/0/0	Unsupported	T,A,M55,K,P55
P0756	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:61	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0757	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:86	U	48/80	36/36/0/0	ResumeClosureSuccessPassThrough	H,M32,E8,K,P44
P0758	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:112	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0759	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:138	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0760	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs:164	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0763	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:207	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P0764	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:239	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0765	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:286	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P0767	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:299	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P0768	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:308	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0769	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:374	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P0770	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:382	S	18/30	18/18/0/0	OrdinarySuccessPassThrough	H,M12,K,P12
P0771	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:416	B	0/35	0/0/0/0	OrdinarySuccessPassThrough	A,M35,K,P35
P0772	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:422	B	0/35	0/0/0/0	OrdinarySuccessPassThrough	A,M35,K,P35
P0773	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:427	U	18/30	0/0/0/0	Unsupported	T,A,M12,K,P30
P0774	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:435	S	18/30	18/18/0/0	OrdinarySuccessPassThrough	H,M12,K,P12
P0775	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:467	B	0/35	0/0/0/0	OrdinarySuccessPassThrough	A,M35,K,P35
P0776	crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:471	S	18/30	18/18/0/0	OrdinarySuccessPassThrough	H,M12,K,P12
P0790	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:86	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	X,K,P20
P0795	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:97	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	X,K,P20
P0798	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:105	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	X,K,P20
P0801	crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs:109	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	X,K,P20
P0812	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:102	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0813	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:104	B	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
P0814	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:105	S	30/30	35/35/31/0	OrdinarySuccessPassThrough	K
P0817	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:131	S	80/85	65/65/0/0	OrdinarySuccessPassThrough	H,M5,K,P25
P0818	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:144	S	80/80	65/65/55/0	OrdinarySuccessPassThrough	K,P20
P0819	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:160	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0820	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:163	C	64/80	65/65/65/49	FrameResumePassThrough	M16,E16,P65
P0821	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:176	S	80/80	65/65/55/0	OrdinarySuccessPassThrough	K,P20
P0822	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:189	S	80/85	65/65/54/0	OrdinarySuccessPassThrough	M5,K,P25
P0823	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:199	U	80/85	0/0/0/0	Unsupported	T,A,M5,K,P85
P0824	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:207	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P0825	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:239	C	64/80	65/65/65/49	FrameResumePassThrough	M16,E16,P65
P0826	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:260	S	80/80	65/65/0/0	OrdinarySuccessPassThrough	H,K,P20
P0827	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:300	U	30/35	0/0/0/0	Unsupported	T,A,M5,K,P35
P0828	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:342	S	80/80	65/65/49/0	OrdinarySuccessPassThrough	K,P20
P0829	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:386	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0830	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:419	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0831	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:456	U	0/85	0/0/0/0	ResumeClosureSuccessPassThrough	A,M85,E20,K,P85
P0832	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:505	U	0/80	0/0/0/0	ResumeClosureSuccessPassThrough	A,M80,E20,K,P80
P0833	crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:587	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P0836	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:78	U	65/65	0/0/0/0	Unsupported	T,A,K,P65
P0837	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:80	U	65/65	0/0/0/0	Unsupported	T,A,K,P65
P0840	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:151	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0847	crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:86	U	75/75	0/0/0/0	Unsupported	T,A,K,P75
P0904	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:127	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P0934	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:30	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0936	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:64	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P0975	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:98	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P0983	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/options.rs:22	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P0984	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/options.rs:36	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1005	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:139	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1037	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:143	C	56/80	48/48/48/48	OrdinarySuccessPassThrough	M24,E12,P80
P1038	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:159	C	46/70	38/38/38/38	OrdinarySuccessPassThrough	M24,E12,P70
P1070	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs:262	U	67/70	0/0/0/0	Unsupported	T,A,M3,K,P70
P1085	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:122	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1101	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:98	S	74/85	54/54/40/0	OrdinarySuccessPassThrough	M11,K,P31
P1102	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:100	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1107	crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:245	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1116	crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:127	S	72/85	60/60/44/0	OrdinarySuccessPassThrough	M13,D8,K,P25
P1124	crates/rum-runtime/src/runtime/namespace_tools/names/options.rs:89	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1132	crates/rum-runtime/src/runtime/options/builtins.rs:265	C	58/70	50/50/50/50	OrdinarySuccessPassThrough	M12,E12,P70
P1135	crates/rum-runtime/src/runtime/options/builtins.rs:379	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1136	crates/rum-runtime/src/runtime/options/builtins.rs:388	S	54/70	46/46/37/0	OrdinarySuccessPassThrough	M16,E12,K,P24
P1139	crates/rum-runtime/src/runtime/options/core.rs:20	U	54/70	0/0/0/0	Unsupported	T,A,M16,E12,K,P70
P1140	crates/rum-runtime/src/runtime/options/core.rs:25	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1142	crates/rum-runtime/src/runtime/options/core.rs:96	C	66/70	50/50/39/39	OrdinarySuccessPassThrough	M4,E4,P59
P1144	crates/rum-runtime/src/runtime/options/core.rs:119	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1145	crates/rum-runtime/src/runtime/options/core.rs:141	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1146	crates/rum-runtime/src/runtime/options/core.rs:145	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1147	crates/rum-runtime/src/runtime/options/core.rs:193	C	70/70	50/50/36/36	OrdinarySuccessPassThrough	R,P56
P1149	crates/rum-runtime/src/runtime/options/core.rs:218	S	65/70	49/49/39/0	OrdinarySuccessPassThrough	M5,E4,K,P21
P1150	crates/rum-runtime/src/runtime/options/core.rs:230	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1151	crates/rum-runtime/src/runtime/options/core.rs:234	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1152	crates/rum-runtime/src/runtime/options/core.rs:236	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1155	crates/rum-runtime/src/runtime/options/read_phase.rs:92	C	70/70	50/50/36/36	OrdinarySuccessPassThrough	R,P56
P1158	crates/rum-runtime/src/runtime/options/resume_frames.rs:21	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1159	crates/rum-runtime/src/runtime/options/resume_frames.rs:47	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1160	crates/rum-runtime/src/runtime/options/resume_frames.rs:49	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1165	crates/rum-runtime/src/runtime/package_tools.rs:231	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1187	crates/rum-runtime/src/runtime/pattern_matching/core.rs:19	S	75/75	55/55/55/0	OrdinarySuccessPassThrough	K,P20
P1193	crates/rum-runtime/src/runtime/pattern_matching/core.rs:116	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P1231	crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:73	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1232	crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:93	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1238	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:137	S	13/75	11/11/10/0	OrdinarySuccessPassThrough	M62,E18,K,P64
P1239	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:142	S	8/75	8/8/7/0	OrdinarySuccessPassThrough	M67,E20,K,P67
P1240	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:173	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P1267	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:192	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1274	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:271	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P1275	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:279	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1283	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:402	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1284	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:422	S	3/85	3/3/3/0	OrdinarySuccessPassThrough	M82,E20,K,P82
P1285	crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:436	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P1310	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:149	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	X,M60,E16,K,P64
P1312	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:172	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P1323	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/entry.rs:13	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1324	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/entry.rs:22	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1325	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/entry.rs:42	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P1326	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/entry.rs:59	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1331	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:163	U	15/75	0/0/0/0	Unsupported	T,A,M60,E16,K,P75
P1332	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:176	U	15/75	0/0/0/0	Unsupported	T,A,M60,E16,K,P75
P1333	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:177	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	X,M60,E16,K,P64
P1334	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:197	U	15/75	0/0/0/0	Unsupported	T,A,M60,E16,K,P75
P1335	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:198	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	X,M60,E16,K,P64
P1354	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:102	S	15/75	11/11/10/0	OrdinarySuccessPassThrough	M60,E16,K,P64
P1355	crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:119	B	0/75	0/0/0/0	OrdinarySuccessPassThrough	A,M75,E20,K,P75
P1470	crates/rum-runtime/src/runtime/resolution/resume_frames.rs:37	U	2/60	2/2/2/0	ResumeClosureSuccessPassThrough	M58,E20,K,P58
P1471	crates/rum-runtime/src/runtime/resolution/resume_frames.rs:62	U	0/60	0/0/0/0	ResumeClosureSuccessPassThrough	A,M60,E20,K,P60
P1472	crates/rum-runtime/src/runtime/resolution.rs:326	S	30/30	30/30/30/0	OrdinarySuccessPassThrough	K
P1476	crates/rum-runtime/src/runtime/resolution.rs:434	S	2/30	2/2/2/0	FrameResumePassThrough	M28,K,P28
P1477	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:18	U	80/80	0/0/0/0	Unsupported	T,A,K,P80
P1478	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:22	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P1479	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:29	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P1480	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:34	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P1482	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:42	B	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
P1483	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:47	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P1484	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:57	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P1485	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:67	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P1486	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:87	U	0/85	0/0/0/0	Unsupported	T,A,M85,E20,K,P85
P1487	crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:107	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P1496	crates/rum-runtime/src/runtime/scoping/lexical/block.rs:51	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1497	crates/rum-runtime/src/runtime/scoping/lexical/block.rs:92	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1507	crates/rum-runtime/src/runtime/scoping/lexical/module.rs:70	S	55/55	55/55/36/0	OrdinarySuccessPassThrough	K
P1525	crates/rum-runtime/src/runtime/scoping/lexical/with.rs:67	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1526	crates/rum-runtime/src/runtime/scoping/lexical/with.rs:79	S	80/85	60/60/41/0	OrdinarySuccessPassThrough	M5,K,P25
P1527	crates/rum-runtime/src/runtime/scoping/lexical/with.rs:96	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1535	crates/rum-runtime/src/runtime/scoping/packages.rs:315	S	80/85	60/60/60/0	OrdinarySuccessPassThrough	M5,K,P25
P1544	crates/rum-runtime/src/runtime/session/guest_loading/script.rs:73	S	50/85	46/46/46/0	OrdinarySuccessPassThrough	M35,E16,K,P39
P1547	crates/rum-runtime/src/runtime/session/guest_loading/script.rs:128	S	50/85	46/46/46/0	OrdinarySuccessPassThrough	M35,E16,K,P39
P1551	crates/rum-runtime/src/runtime/session/session_eval.rs:182	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1552	crates/rum-runtime/src/runtime/session/session_eval.rs:189	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P1553	crates/rum-runtime/src/runtime/session/session_eval.rs:200	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1555	crates/rum-runtime/src/runtime/session/session_eval.rs:242	U	2/30	0/0/0/0	Unsupported	T,A,M28,K,P30
P1562	crates/rum-runtime/src/runtime/session/session_requests.rs:170	S	30/30	30/30/26/0	OrdinarySuccessPassThrough	X,K
P1563	crates/rum-runtime/src/runtime/session/session_requests.rs:193	B	30/30	0/0/0/0	OrdinarySuccessPassThrough	A,K,P30
P1564	crates/rum-runtime/src/runtime/session/session_requests.rs:204	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1565	crates/rum-runtime/src/runtime/session/session_requests.rs:214	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1566	crates/rum-runtime/src/runtime/session/session_requests.rs:227	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P1567	crates/rum-runtime/src/runtime/session/session_requests.rs:293	S	30/30	29/29/0/0	OrdinarySuccessPassThrough	H,K,P1
P1568	crates/rum-runtime/src/runtime/session/session_requests.rs:309	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P1573	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:22	U	0/85	0/0/0/0	Unsupported	T,A,M85,E20,K,P85
P1574	crates/rum-runtime/src/runtime/suspension/core/abandon.rs:65	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P1575	crates/rum-runtime/src/runtime/suspension/core/fault.rs:126	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P1576	crates/rum-runtime/src/runtime/suspension/core.rs:606	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P1577	crates/rum-runtime/src/runtime/suspension/core.rs:659	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P1578	crates/rum-runtime/src/runtime/suspension/core.rs:753	U	80/80	0/0/0/0	Unsupported	T,A,K,P80
P1579	crates/rum-runtime/src/runtime/suspension/core.rs:776	U	48/80	0/0/0/0	Unsupported	T,A,M32,E8,K,P80
P1580	crates/rum-runtime/src/runtime/suspension/core.rs:788	U	26/30	0/0/0/0	Unsupported	T,A,M4,K,P30
P1581	crates/rum-runtime/src/runtime/suspension/core.rs:825	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P1582	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:49	U	26/30	0/0/0/0	Unsupported	T,A,M4,K,P30
P1583	crates/rum-runtime/src/runtime/test_builtins.rs:510	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1585	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:21	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1586	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:45	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1587	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:69	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1588	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:95	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1589	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:121	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1590	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:147	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1591	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:179	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1592	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:234	U	42/70	0/0/0/0	Unsupported	T,A,M28,E8,K,P70
P1593	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:254	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1594	crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs:278	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1595	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:48	B	0/70	0/0/0/0	FrameResumePassThrough	A,M70,E20,K,P70
P1597	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:74	B	0/70	0/0/0/0	FrameResumePassThrough	A,M70,E20,K,P70
P1599	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:97	B	0/70	0/0/0/0	FrameResumePassThrough	A,M70,E20,K,P70
P1602	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:128	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1603	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:137	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1605	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:167	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1606	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:176	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1608	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:206	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1609	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:215	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1610	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:238	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P1611	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:241	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1613	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:283	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P1615	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:325	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P1616	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:335	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1617	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:364	S	42/70	30/30/0/0	OrdinarySuccessPassThrough	H,M28,E8,K,P40
P1618	crates/rum-runtime/src/runtime/tree_walkers/level.rs:14	S	60/85	52/52/40/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P1619	crates/rum-runtime/src/runtime/tree_walkers/position/options.rs:17	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1620	crates/rum-runtime/src/runtime/tree_walkers/position/options.rs:29	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1621	crates/rum-runtime/src/runtime/tree_walkers/position/options.rs:42	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P1622	crates/rum-runtime/src/runtime/tree_walkers/position/options.rs:46	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1624	crates/rum-runtime/src/runtime/tree_walkers/position/options.rs:67	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1625	crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:21	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1626	crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:47	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1627	crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:76	U	0/80	0/0/0/0	ResumeTypedSuccessPassThrough	A,M80,E20,K,P80
P1628	crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:122	U	42/70	0/0/0/0	Unsupported	T,A,M28,E8,K,P70
P1629	crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:150	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1630	crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs:166	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1634	crates/rum-runtime/src/runtime/tree_walkers/position.rs:139	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1635	crates/rum-runtime/src/runtime/tree_walkers/position.rs:148	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P1636	crates/rum-runtime/src/runtime/tree_walkers/position.rs:178	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P1637	crates/rum-runtime/src/runtime/tree_walkers/position.rs:207	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P1639	crates/rum-runtime/src/runtime/tree_walkers/position.rs:253	S	42/70	30/30/0/0	OrdinarySuccessPassThrough	H,M28,E8,K,P40
P1640	crates/rum-runtime/src/runtime/tree_walkers/position.rs:275	U	42/70	0/0/0/0	Unsupported	T,A,M28,E8,K,P70
P1641	crates/rum-runtime/src/runtime/tree_walkers/position.rs:293	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1642	crates/rum-stdlib/src/arithmetic/stats.rs:43	S	60/85	52/52/41/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P1661	crates/rum-stdlib/src/arithmetic.rs:155	S	60/85	52/52/41/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P1679	crates/rum-stdlib/src/associations/aggregate.rs:33	S	52/85	48/48/28/0	OrdinarySuccessPassThrough	M33,E16,K,P37
P1680	crates/rum-stdlib/src/associations/aggregate.rs:46	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1681	crates/rum-stdlib/src/associations/aggregate.rs:87	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1704	crates/rum-stdlib/src/associations/basic.rs:45	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1708	crates/rum-stdlib/src/associations/basic.rs:219	U	58/70	0/0/0/0	Unsupported	T,A,M12,D12,K,P70
P1729	crates/rum-stdlib/src/associations/grouping/association.rs:275	S	80/85	60/60/45/0	OrdinarySuccessPassThrough	M5,K,P25
P1758	crates/rum-stdlib/src/associations/grouping/list.rs:253	S	80/85	60/60/45/0	OrdinarySuccessPassThrough	M5,K,P25
P1759	crates/rum-stdlib/src/associations/grouping/list.rs:267	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,E8,D8,K,P25
P1760	crates/rum-stdlib/src/associations/grouping/list.rs:287	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P1767	crates/rum-stdlib/src/associations/grouping.rs:155	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1768	crates/rum-stdlib/src/associations/grouping.rs:164	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,E8,D8,K,P25
P1769	crates/rum-stdlib/src/associations/grouping.rs:187	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1770	crates/rum-stdlib/src/associations/grouping.rs:196	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,E8,D8,K,P25
P1771	crates/rum-stdlib/src/associations/grouping.rs:212	S	80/85	60/60/45/0	OrdinarySuccessPassThrough	M5,K,P25
P1772	crates/rum-stdlib/src/associations/grouping.rs:221	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P1773	crates/rum-stdlib/src/associations/grouping.rs:244	S	80/85	60/60/45/0	OrdinarySuccessPassThrough	M5,K,P25
P1774	crates/rum-stdlib/src/associations/grouping.rs:253	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P1779	crates/rum-stdlib/src/associations/key_reshape.rs:28	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,E8,D8,K,P25
P1780	crates/rum-stdlib/src/associations/key_reshape.rs:42	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P1799	crates/rum-stdlib/src/associations/missing/target.rs:98	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1802	crates/rum-stdlib/src/associations/missing/target.rs:145	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1803	crates/rum-stdlib/src/associations/missing/target.rs:148	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1804	crates/rum-stdlib/src/associations/missing/target.rs:210	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1805	crates/rum-stdlib/src/associations/missing/target.rs:261	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1806	crates/rum-stdlib/src/associations/missing/target.rs:331	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1809	crates/rum-stdlib/src/associations/missing/target.rs:587	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1810	crates/rum-stdlib/src/associations/missing.rs:76	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P1811	crates/rum-stdlib/src/associations/missing.rs:212	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P1812	crates/rum-stdlib/src/associations/mutation.rs:187	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1813	crates/rum-stdlib/src/associations/mutation.rs:206	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1814	crates/rum-stdlib/src/associations/mutation.rs:234	C	53/70	49/49/49/40	FrameResumePassThrough	M17,E16,P61
P1815	crates/rum-stdlib/src/associations/mutation.rs:310	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1816	crates/rum-stdlib/src/associations/mutation.rs:349	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1817	crates/rum-stdlib/src/associations/mutation.rs:363	C	53/70	49/49/49/40	FrameResumePassThrough	M17,E16,P61
P1818	crates/rum-stdlib/src/associations/mutation.rs:530	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P1819	crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:35	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1823	crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs:109	S	80/85	60/60/60/0	OrdinarySuccessPassThrough	M5,K,P25
P1833	crates/rum-stdlib/src/associations/resumable/aggregate.rs:57	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P1834	crates/rum-stdlib/src/associations/resumable/aggregate.rs:86	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,E8,D8,K,P25
P1841	crates/rum-stdlib/src/associations/resumable/key_select.rs:27	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1847	crates/rum-stdlib/src/associations/resumable/key_sort.rs:99	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1848	crates/rum-stdlib/src/associations/resumable/key_sort.rs:112	S	68/85	60/60/44/0	OrdinarySuccessPassThrough	M17,D12,K,P25
P1859	crates/rum-stdlib/src/associations/resumable/map.rs:25	S	60/85	60/60/44/0	OrdinarySuccessPassThrough	M25,D20,K,P25
P1860	crates/rum-stdlib/src/associations/resumable/map.rs:41	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P1873	crates/rum-stdlib/src/associations/transforms/key_collections.rs:134	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P1877	crates/rum-stdlib/src/associations/transforms.rs:116	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1879	crates/rum-stdlib/src/control_flow.rs:21	S	80/85	65/65/51/0	OrdinarySuccessPassThrough	M5,K,P25
P1880	crates/rum-stdlib/src/control_flow.rs:29	S	80/85	65/65/45/0	OrdinarySuccessPassThrough	M5,K,P25
P1881	crates/rum-stdlib/src/control_flow.rs:37	S	76/85	65/65/65/0	OrdinarySuccessPassThrough	M9,D4,K,P25
P1882	crates/rum-stdlib/src/control_flow.rs:45	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1883	crates/rum-stdlib/src/control_flow.rs:53	S	80/85	65/65/65/0	OrdinarySuccessPassThrough	M5,K,P25
P1884	crates/rum-stdlib/src/control_flow.rs:62	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1885	crates/rum-stdlib/src/control_flow.rs:71	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1886	crates/rum-stdlib/src/control_flow.rs:80	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1887	crates/rum-stdlib/src/control_flow.rs:88	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1888	crates/rum-stdlib/src/control_flow.rs:96	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1889	crates/rum-stdlib/src/control_flow.rs:104	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1890	crates/rum-stdlib/src/control_flow.rs:113	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1891	crates/rum-stdlib/src/control_flow.rs:297	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1892	crates/rum-stdlib/src/control_flow.rs:305	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1893	crates/rum-stdlib/src/control_flow.rs:313	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1894	crates/rum-stdlib/src/control_flow.rs:321	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1895	crates/rum-stdlib/src/control_flow.rs:329	S	80/85	65/65/42/0	OrdinarySuccessPassThrough	M5,K,P25
P1896	crates/rum-stdlib/src/control_flow.rs:337	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1897	crates/rum-stdlib/src/control_flow.rs:345	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1898	crates/rum-stdlib/src/control_flow.rs:353	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1899	crates/rum-stdlib/src/control_flow.rs:361	S	80/85	65/65/0/0	OrdinarySuccessPassThrough	H,M5,K,P25
P1900	crates/rum-stdlib/src/control_flow.rs:369	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1901	crates/rum-stdlib/src/control_flow.rs:377	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1902	crates/rum-stdlib/src/control_flow.rs:385	S	80/85	65/65/0/0	OrdinarySuccessPassThrough	H,M5,K,P25
P1903	crates/rum-stdlib/src/control_flow.rs:393	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1904	crates/rum-stdlib/src/control_flow.rs:401	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1905	crates/rum-stdlib/src/control_flow.rs:409	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1906	crates/rum-stdlib/src/control_flow.rs:417	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1907	crates/rum-stdlib/src/control_flow.rs:425	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1908	crates/rum-stdlib/src/control_flow.rs:433	S	80/85	65/65/50/0	OrdinarySuccessPassThrough	M5,K,P25
P1909	crates/rum-stdlib/src/control_flow.rs:441	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1910	crates/rum-stdlib/src/control_flow.rs:449	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1911	crates/rum-stdlib/src/control_flow.rs:457	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1912	crates/rum-stdlib/src/control_flow.rs:470	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1913	crates/rum-stdlib/src/control_flow.rs:478	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1914	crates/rum-stdlib/src/control_flow.rs:486	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1915	crates/rum-stdlib/src/control_flow.rs:494	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1916	crates/rum-stdlib/src/control_flow.rs:502	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1917	crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:31	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1918	crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:55	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1919	crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:132	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P1923	crates/rum-stdlib/src/functional_ops/arrays/filter.rs:58	S	80/85	60/60/43/0	OrdinarySuccessPassThrough	M5,K,P25
P1928	crates/rum-stdlib/src/functional_ops/arrays/generators.rs:133	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1929	crates/rum-stdlib/src/functional_ops/arrays/generators.rs:202	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1936	crates/rum-stdlib/src/functional_ops/arrays/mod.rs:31	U	58/70	0/0/0/0	Unsupported	T,A,M12,D12,K,P70
P1937	crates/rum-stdlib/src/functional_ops/arrays/mod.rs:55	U	54/70	0/0/0/0	Unsupported	T,A,M16,E12,K,P70
P1940	crates/rum-stdlib/src/functional_ops/arrays/mod.rs:179	S	74/85	58/58/43/0	OrdinarySuccessPassThrough	M11,E4,K,P27
P1941	crates/rum-stdlib/src/functional_ops/arrays/mod.rs:212	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1949	crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:92	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1950	crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:143	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1955	crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:359	S	58/70	50/50/40/0	OrdinarySuccessPassThrough	M12,D12,K,P20
P1962	crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:15	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1963	crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:29	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1964	crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:39	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1973	crates/rum-stdlib/src/functional_ops/iteration.rs:74	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1974	crates/rum-stdlib/src/functional_ops/iteration.rs:92	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1975	crates/rum-stdlib/src/functional_ops/iteration.rs:109	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1976	crates/rum-stdlib/src/functional_ops/iteration.rs:120	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1977	crates/rum-stdlib/src/functional_ops/iteration.rs:153	S	56/85	52/52/41/0	OrdinarySuccessPassThrough	M29,E16,K,P33
P1978	crates/rum-stdlib/src/functional_ops/iteration.rs:171	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1979	crates/rum-stdlib/src/functional_ops/iteration.rs:188	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1980	crates/rum-stdlib/src/functional_ops/iteration.rs:199	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1981	crates/rum-stdlib/src/functional_ops/iteration.rs:213	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1982	crates/rum-stdlib/src/functional_ops/iteration.rs:223	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1983	crates/rum-stdlib/src/functional_ops/iteration.rs:237	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P1984	crates/rum-stdlib/src/functional_ops/iteration.rs:247	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P1996	crates/rum-stdlib/src/functional_ops/iteration.rs:470	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2009	crates/rum-stdlib/src/functional_ops/iteration.rs:693	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2010	crates/rum-stdlib/src/functional_ops/iteration.rs:715	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2011	crates/rum-stdlib/src/functional_ops/iteration.rs:741	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2012	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:29	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2013	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:31	S	58/85	52/52/41/0	OrdinarySuccessPassThrough	M27,E14,K,P33
P2014	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:45	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2015	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:47	S	56/85	52/52/41/0	OrdinarySuccessPassThrough	M29,E14,D2,K,P33
P2016	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:61	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2017	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:63	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2018	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:72	S	72/85	60/60/48/0	OrdinarySuccessPassThrough	M13,D8,K,P25
P2019	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:97	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2020	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:100	S	68/85	60/60/45/0	OrdinarySuccessPassThrough	M17,D12,K,P25
P2021	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:111	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2022	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:122	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P2023	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:145	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2024	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:147	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2025	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:169	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2026	crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:171	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2070	crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:225	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P2072	crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:294	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2080	crates/rum-stdlib/src/functional_ops/sequence/select.rs:75	S	60/85	52/52/41/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P2081	crates/rum-stdlib/src/functional_ops/sequence/select.rs:96	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2082	crates/rum-stdlib/src/functional_ops/sequence/select.rs:115	S	16/85	12/12/11/0	OrdinarySuccessPassThrough	M69,E16,K,P73
P2083	crates/rum-stdlib/src/functional_ops/sequence/select.rs:128	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2096	crates/rum-stdlib/src/functional_ops/sequence/window.rs:104	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2097	crates/rum-stdlib/src/functional_ops/sequence/window.rs:122	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2098	crates/rum-stdlib/src/functional_ops/sequence/window.rs:142	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2099	crates/rum-stdlib/src/functional_ops/sequence/window.rs:158	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2100	crates/rum-stdlib/src/functional_ops/sequence/window.rs:177	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2117	crates/rum-stdlib/src/functional_ops/targeted_map.rs:177	S	74/85	54/54/42/0	OrdinarySuccessPassThrough	M11,K,P31
P2118	crates/rum-stdlib/src/functional_ops/targeted_map.rs:211	S	54/85	50/50/40/0	OrdinarySuccessPassThrough	M31,E16,K,P35
P2127	crates/rum-stdlib/src/integer_digits.rs:104	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P2128	crates/rum-stdlib/src/integer_digits.rs:180	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P2132	crates/rum-stdlib/src/lazy/machine.rs:27	U	56/80	0/0/0/0	Unsupported	T,A,M24,E16,K,P80
P2133	crates/rum-stdlib/src/lazy/machine.rs:68	U	72/80	0/0/0/0	Unsupported	T,A,M8,D8,K,P80
P2134	crates/rum-stdlib/src/lazy/machine.rs:79	U	56/80	0/0/0/0	Unsupported	T,A,M24,E16,K,P80
P2135	crates/rum-stdlib/src/lazy/machine.rs:107	U	55/75	0/0/0/0	Unsupported	T,A,M20,E14,D2,K,P75
P2136	crates/rum-stdlib/src/lazy/realization.rs:28	S	45/70	39/39/0/0	OrdinarySuccessPassThrough	H,M25,E6,D8,K,P31
P2137	crates/rum-stdlib/src/lazy/source.rs:65	U	55/75	0/0/0/0	Unsupported	T,A,M20,E16,K,P75
P2144	crates/rum-stdlib/src/lazy.rs:422	S	63/75	55/55/44/0	OrdinarySuccessPassThrough	M12,D12,K,P20
P2145	crates/rum-stdlib/src/lazy.rs:430	S	67/75	55/55/44/0	OrdinarySuccessPassThrough	M8,D8,K,P20
P2146	crates/rum-stdlib/src/lazy.rs:446	U	67/75	0/0/0/0	Unsupported	T,A,M8,D8,K,P75
P2155	crates/rum-stdlib/src/logic/truth.rs:19	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2156	crates/rum-stdlib/src/logic/truth.rs:27	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2157	crates/rum-stdlib/src/logic/truth.rs:35	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2158	crates/rum-stdlib/src/logic/truth.rs:43	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2175	crates/rum-stdlib/src/number_theory.rs:137	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
P2176	crates/rum-stdlib/src/number_theory.rs:167	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
P2177	crates/rum-stdlib/src/number_theory.rs:193	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
P2178	crates/rum-stdlib/src/number_theory.rs:219	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
P2179	crates/rum-stdlib/src/number_theory.rs:250	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
P2182	crates/rum-stdlib/src/option_consumers.rs:101	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2183	crates/rum-stdlib/src/option_consumers.rs:140	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2185	crates/rum-stdlib/src/option_consumers.rs:155	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2187	crates/rum-stdlib/src/option_consumers.rs:165	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2196	crates/rum-stdlib/src/option_consumers.rs:198	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2201	crates/rum-stdlib/src/option_consumers.rs:219	S	44/70	40/40/32/0	OrdinarySuccessPassThrough	M26,E16,K,P30
P2202	crates/rum-stdlib/src/option_consumers.rs:222	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2206	crates/rum-stdlib/src/predicates/arrays.rs:48	S	59/85	51/51/39/0	OrdinarySuccessPassThrough	M26,E12,K,P34
P2207	crates/rum-stdlib/src/predicates/arrays.rs:72	S	59/85	51/51/39/0	OrdinarySuccessPassThrough	M26,E12,K,P34
P2223	crates/rum-stdlib/src/predicates/collections.rs:66	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2224	crates/rum-stdlib/src/predicates/collections.rs:83	S	75/75	55/55/36/0	OrdinarySuccessPassThrough	K,P20
P2225	crates/rum-stdlib/src/predicates/collections.rs:98	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2226	crates/rum-stdlib/src/predicates/collections.rs:365	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2227	crates/rum-stdlib/src/predicates/collections.rs:385	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2228	crates/rum-stdlib/src/predicates/collections.rs:482	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2236	crates/rum-stdlib/src/predicates/collections.rs:734	S	80/85	60/60/60/0	OrdinarySuccessPassThrough	M5,K,P25
P2245	crates/rum-stdlib/src/predicates/ordering.rs:68	S	59/85	51/51/39/0	OrdinarySuccessPassThrough	M26,E12,K,P34
P2250	crates/rum-stdlib/src/predicates/ordering.rs:225	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2263	crates/rum-stdlib/src/predicates/ordering.rs:599	S	75/75	55/55/55/0	OrdinarySuccessPassThrough	K,P20
P2264	crates/rum-stdlib/src/predicates/truth.rs:64	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2265	crates/rum-stdlib/src/predicates/truth.rs:72	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2266	crates/rum-stdlib/src/predicates/truth.rs:80	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2275	crates/rum-stdlib/src/random.rs:322	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2277	crates/rum-stdlib/src/rewrite_builtins.rs:19	S	68/85	60/60/40/0	OrdinarySuccessPassThrough	M17,D12,K,P25
P2278	crates/rum-stdlib/src/rewrite_builtins.rs:27	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2279	crates/rum-stdlib/src/rewrite_builtins.rs:35	S	72/85	60/60/50/0	OrdinarySuccessPassThrough	M13,E8,K,P25
P2280	crates/rum-stdlib/src/rewrite_builtins.rs:43	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2281	crates/rum-stdlib/src/rewrite_builtins.rs:51	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2282	crates/rum-stdlib/src/rewrite_builtins.rs:59	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2283	crates/rum-stdlib/src/rewrite_builtins.rs:67	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2284	crates/rum-stdlib/src/rewrite_builtins.rs:75	S	56/85	48/48/32/0	OrdinarySuccessPassThrough	M29,E12,K,P37
P2287	crates/rum-stdlib/src/schema/machine.rs:57	C	30/35	30/30/30/30	OrdinarySuccessPassThrough	M5,P35
P2288	crates/rum-stdlib/src/schema/machine.rs:70	C	25/30	25/25/25/25	OrdinarySuccessPassThrough	M5,P30
P2299	crates/rum-stdlib/src/schema.rs:130	S	75/225	55/55/36/8	OrdinarySuccessPassThrough	R,M150,E40,P178
P2302	crates/rum-stdlib/src/strings/encoding.rs:107	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2312	crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:60	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2328	crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:320	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P2329	crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:328	S	42/70	30/30/0/0	OrdinarySuccessPassThrough	H,M28,E8,K,P40
P2347	crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:62	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2348	crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:109	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P2354	crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:231	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P2363	crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:409	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P2364	crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:418	S	42/70	30/30/0/0	OrdinarySuccessPassThrough	H,M28,E8,K,P40
P2369	crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:535	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2377	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:70	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2378	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:90	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2379	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:138	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P2380	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:139	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2381	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:191	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P2382	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:192	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2385	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:265	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P2386	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:270	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P2387	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:326	U	55/70	0/0/0/0	Unsupported	T,A,M15,E4,K,P70
P2388	crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs:327	S	42/70	30/30/0/0	OrdinarySuccessPassThrough	H,M28,E8,K,P40
P2406	crates/rum-stdlib/src/strings/patterns/search/rules.rs:59	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2407	crates/rum-stdlib/src/strings/patterns/search/rules.rs:83	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2408	crates/rum-stdlib/src/strings/patterns/search/rules.rs:104	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P2409	crates/rum-stdlib/src/strings/patterns/search/rules.rs:131	U	70/70	0/0/0/0	Unsupported	T,A,K,P70
P2410	crates/rum-stdlib/src/strings/patterns/search/rules.rs:132	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2411	crates/rum-stdlib/src/strings/patterns/search/rules.rs:187	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P2412	crates/rum-stdlib/src/strings/patterns/search/rules.rs:188	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P2413	crates/rum-stdlib/src/strings/patterns/search/rules.rs:209	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2414	crates/rum-stdlib/src/strings/patterns/search/rules.rs:219	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P2416	crates/rum-stdlib/src/strings/patterns/search/rules.rs:296	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
P2417	crates/rum-stdlib/src/strings/patterns/search/rules.rs:301	B	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
P2418	crates/rum-stdlib/src/strings/patterns/search/rules.rs:332	U	55/70	0/0/0/0	Unsupported	T,A,M15,E4,K,P70
P2419	crates/rum-stdlib/src/strings/patterns/search/rules.rs:333	S	42/70	30/30/0/0	OrdinarySuccessPassThrough	H,M28,E8,K,P40
P2423	crates/rum-stdlib/src/strings/patterns/search/rules.rs:459	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2429	crates/rum-stdlib/src/strings/patterns/search/rules.rs:574	S	70/70	50/50/50/0	OrdinarySuccessPassThrough	K,P20
P2430	crates/rum-stdlib/src/strings/patterns/search.rs:30	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2431	crates/rum-stdlib/src/strings/patterns/search.rs:40	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2432	crates/rum-stdlib/src/strings/patterns/search.rs:55	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2433	crates/rum-stdlib/src/strings/patterns/search.rs:65	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2434	crates/rum-stdlib/src/strings/patterns/search.rs:80	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2435	crates/rum-stdlib/src/strings/patterns/search.rs:90	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2436	crates/rum-stdlib/src/strings/patterns/search.rs:105	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2437	crates/rum-stdlib/src/strings/patterns/search.rs:126	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2438	crates/rum-stdlib/src/strings/patterns/search.rs:159	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2439	crates/rum-stdlib/src/strings/patterns/search.rs:179	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2440	crates/rum-stdlib/src/strings/patterns/search.rs:288	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2446	crates/rum-stdlib/src/strings/patterns/search.rs:380	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2452	crates/rum-stdlib/src/strings/patterns/search.rs:503	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2453	crates/rum-stdlib/src/strings/patterns/search.rs:523	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2454	crates/rum-stdlib/src/strings/patterns/search.rs:562	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2455	crates/rum-stdlib/src/strings/patterns/search.rs:585	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2456	crates/rum-stdlib/src/strings/patterns/search.rs:625	S	79/85	64/64/48/0	OrdinarySuccessPassThrough	M6,K,P26
P2457	crates/rum-stdlib/src/strings/patterns/search.rs:629	S	80/85	65/65/48/0	OrdinarySuccessPassThrough	M5,K,P25
P2458	crates/rum-stdlib/src/strings/patterns/search.rs:643	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2459	crates/rum-stdlib/src/strings/patterns/search.rs:653	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2460	crates/rum-stdlib/src/strings/patterns/search.rs:668	S	80/85	65/65/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2461	crates/rum-stdlib/src/strings/patterns/search.rs:678	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2462	crates/rum-stdlib/src/strings/patterns/search.rs:710	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2463	crates/rum-stdlib/src/strings/patterns/search.rs:759	S	75/75	55/55/40/0	OrdinarySuccessPassThrough	K,P20
P2464	crates/rum-stdlib/src/strings/scripting/args.rs:10	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2465	crates/rum-stdlib/src/strings/scripting/args.rs:40	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2466	crates/rum-stdlib/src/strings/scripting/html.rs:46	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2467	crates/rum-stdlib/src/strings/scripting/html.rs:65	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2468	crates/rum-stdlib/src/strings/scripting/text_shape.rs:10	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2469	crates/rum-stdlib/src/strings/scripting/text_shape.rs:37	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2470	crates/rum-stdlib/src/strings/scripting/text_shape.rs:61	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2471	crates/rum-stdlib/src/strings/scripting/text_shape.rs:87	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2472	crates/rum-stdlib/src/strings/scripting/text_shape.rs:169	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2473	crates/rum-stdlib/src/strings/scripting/text_shape.rs:188	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2474	crates/rum-stdlib/src/strings/scripting/url.rs:12	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2475	crates/rum-stdlib/src/strings/scripting/url.rs:31	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2476	crates/rum-stdlib/src/strings/scripting/url.rs:51	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2477	crates/rum-stdlib/src/strings/scripting/url.rs:70	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2478	crates/rum-stdlib/src/strings/scripting/url.rs:93	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2479	crates/rum-stdlib/src/strings/scripting/url.rs:112	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2480	crates/rum-stdlib/src/strings/template.rs:54	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2481	crates/rum-stdlib/src/strings/template.rs:76	S	49/70	45/45/36/0	OrdinarySuccessPassThrough	M21,E16,K,P25
P2482	crates/rum-stdlib/src/strings/template.rs:82	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2483	crates/rum-stdlib/src/strings/template.rs:108	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2501	crates/rum-stdlib/src/structure_ops/collection_stats.rs:109	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2505	crates/rum-stdlib/src/structure_ops/collection_stats.rs:262	S	74/85	60/60/44/0	OrdinarySuccessPassThrough	M11,D6,K,P25
P2506	crates/rum-stdlib/src/structure_ops/collection_stats.rs:271	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2507	crates/rum-stdlib/src/structure_ops/collection_stats.rs:280	S	74/85	60/60/44/0	OrdinarySuccessPassThrough	M11,D6,K,P25
P2508	crates/rum-stdlib/src/structure_ops/collection_stats.rs:289	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2509	crates/rum-stdlib/src/structure_ops/collection_stats.rs:298	S	74/85	60/60/44/0	OrdinarySuccessPassThrough	M11,D6,K,P25
P2510	crates/rum-stdlib/src/structure_ops/collection_stats.rs:307	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2511	crates/rum-stdlib/src/structure_ops/collection_stats.rs:327	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2512	crates/rum-stdlib/src/structure_ops/collection_stats.rs:362	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2513	crates/rum-stdlib/src/structure_ops/collection_stats.rs:393	S	74/85	60/60/44/0	OrdinarySuccessPassThrough	M11,D6,K,P25
P2514	crates/rum-stdlib/src/structure_ops/collection_stats.rs:405	S	74/85	60/60/44/0	OrdinarySuccessPassThrough	M11,D6,K,P25
P2524	crates/rum-stdlib/src/structure_ops/dedup.rs:61	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2525	crates/rum-stdlib/src/structure_ops/dedup.rs:109	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2526	crates/rum-stdlib/src/structure_ops/dedup.rs:147	S	74/85	60/60/44/0	OrdinarySuccessPassThrough	M11,D6,K,P25
P2537	crates/rum-stdlib/src/structure_ops/list_lookup.rs:51	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2538	crates/rum-stdlib/src/structure_ops/list_lookup.rs:73	B	0/85	0/0/0/0	OrdinarySuccessPassThrough	A,M85,E20,K,P85
P2544	crates/rum-stdlib/src/structure_ops/list_plumbing.rs:30	U	54/70	0/0/0/0	Unsupported	T,A,M16,E12,K,P70
P2547	crates/rum-stdlib/src/structure_ops/list_plumbing.rs:151	U	54/70	0/0/0/0	Unsupported	T,A,M16,E12,K,P70
P2551	crates/rum-stdlib/src/structure_ops/list_plumbing.rs:255	S	54/70	46/46/46/0	OrdinarySuccessPassThrough	M16,E12,K,P24
P2552	crates/rum-stdlib/src/structure_ops/mod.rs:81	S	70/70	50/50/50/0	OrdinarySuccessPassThrough	K,P20
P2565	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:152	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2566	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:183	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2567	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:201	S	68/85	60/60/44/0	OrdinarySuccessPassThrough	M17,D12,K,P25
P2568	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:220	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2569	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:238	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2570	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:261	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2571	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:308	S	80/85	60/60/44/0	OrdinarySuccessPassThrough	M5,K,P25
P2572	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:331	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2573	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:349	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2574	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:392	S	80/85	60/60/40/0	OrdinarySuccessPassThrough	M5,K,P25
P2575	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:415	S	70/70	50/50/36/0	OrdinarySuccessPassThrough	K,P20
P2576	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:434	S	70/70	50/50/40/0	OrdinarySuccessPassThrough	K,P20
P2590	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:29	S	64/85	60/60/44/0	OrdinarySuccessPassThrough	M21,D16,K,P25
P2591	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:81	U	54/70	0/0/0/0	Unsupported	T,A,M16,D16,K,P70
P2592	crates/rum-stdlib/src/structure_ops/surface.rs:41	S	60/85	52/52/40/0	OrdinarySuccessPassThrough	M25,E12,K,P33
P2593	crates/rum-stdlib/src/structure_ops/surface.rs:83	U	62/70	0/0/0/0	Unsupported	T,A,M8,D8,K,P70
P2595	crates/rum-host/src/session.rs:352	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P2596	crates/rum-host/src/session.rs:373	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P2599	crates/rum-host/src/session.rs:436	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P2600	crates/rum-host/src/session.rs:482	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P2601	crates/rum-host/src/session.rs:557	U	0/85	0/0/0/0	Unsupported	T,A,M85,E20,K,P85
P2602	crates/rum-host/src/session.rs:561	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P2603	crates/rum-host/src/session.rs:592	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P2604	crates/rum-host/src/session.rs:708	U	30/30	182/182/182/0	NativeGuardOutcomeTail	R,K,P28
P2605	crates/rum-jupyter/src/adapter/execute.rs:20	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2606	crates/rum-jupyter/src/adapter/execute.rs:245	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2607	crates/rum-jupyter/src/adapter/execute.rs:248	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2608	crates/rum-jupyter/src/adapter/input_retirement.rs:15	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2609	crates/rum-jupyter/src/adapter/worker.rs:185	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2610	crates/rum-jupyter/src/adapter/worker.rs:339	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2611	crates/rum-jupyter/src/adapter/worker.rs:358	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2612	crates/rum-jupyter/src/adapter/worker.rs:369	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2613	crates/rum-jupyter/src/adapter/worker.rs:374	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2614	crates/rum-jupyter/src/adapter/worker.rs:387	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2615	crates/rum-cli/src/eval_tests.rs:38	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2616	crates/rum-cli/src/eval_tests.rs:58	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2619	crates/rum-cli/src/eval_tests.rs:61	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2620	crates/rum-cli/src/eval_tests.rs:62	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2621	crates/rum-cli/src/eval_tests.rs:66	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2622	crates/rum-cli/src/eval_tests.rs:84	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2623	crates/rum-cli/src/input_retirement.rs:23	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2624	crates/rum-cli/src/main.rs:43	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2625	crates/rum-cli/src/main.rs:67	U	0/35	0/0/0/0	Unsupported	T,A,M35,K,P35
P2626	crates/rum-cli/src/main.rs:92	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2627	crates/rum-cli/src/main.rs:107	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2629	crates/rum-cli/src/main.rs:113	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2630	crates/rum-cli/src/main.rs:115	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2631	crates/rum-cli/src/main.rs:116	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2632	crates/rum-cli/src/main.rs:120	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2633	crates/rum-cli/src/main.rs:122	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2635	crates/rum-cli/src/main.rs:224	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2636	crates/rum-cli/src/main.rs:231	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2637	crates/rum-cli/src/main.rs:235	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2642	crates/rum-cli/src/main.rs:272	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2643	crates/rum-cli/src/main.rs:309	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2644	crates/rum-cli/src/main.rs:327	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2645	crates/rum-cli/src/main.rs:335	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2646	crates/rum-repl/src/lib.rs:138	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2647	crates/rum-repl/src/lib.rs:152	U	30/30	0/0/0/0	Unsupported	T,A,K,P30
P2648	crates/rum-repl/src/lib.rs:181	U	25/30	0/0/0/0	Unsupported	T,A,M5,K,P30
P2649	crates/rum-repl/src/lib.rs:195	U	25/30	120/55/55/0	Unsupported	T,M5,K,P5
P2650	crates/rum-repl/src/lib.rs:287	U	60/80	0/0/0/0	Unsupported	T,A,M20,E20,K,P80
P2651	crates/rum-repl/src/request.rs:134	U	60/80	120/120/120/120	Unsupported	T,M20,E20,P80
P2652	crates/rum-repl/src/request.rs:181	U	60/80	120/120/120/120	Unsupported	T,M20,E20,P80
P2653	crates/rum-repl/src/request.rs:184	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
P2654	crates/rum-repl/src/request.rs:195	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2655	crates/rum-repl/src/request.rs:206	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2656	crates/rum-repl/src/request.rs:219	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
P2657	crates/rum-repl/src/request.rs:252	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
S001	crates/rum-runtime/src/runtime/accounting_mode/mod.rs:238	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
S002	crates/rum-runtime/src/runtime/debug_tools/monitor/options.rs:138	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S003	crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs:50	U	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
S004	crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs:118	U	0/80	0/0/0/0	OrdinarySuccessPassThrough	A,M80,E20,K,P80
S005	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/options.rs:102	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S006	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/options.rs:101	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S007	crates/rum-runtime/src/runtime/namespace_tools/names/options.rs:45	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S008	crates/rum-runtime/src/runtime/namespace_tools/names/options.rs:130	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S009	crates/rum-runtime/src/runtime/options/core.rs:22	U	50/70	0/0/0/0	Unsupported	T,A,M20,E20,K,P70
S010	crates/rum-runtime/src/runtime/options/core.rs:23	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
S011	crates/rum-runtime/src/runtime/options/core.rs:91	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
S012	crates/rum-runtime/src/runtime/options/core.rs:97	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
S013	crates/rum-runtime/src/runtime/options/core.rs:187	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
S014	crates/rum-runtime/src/runtime/options/core.rs:191	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
S015	crates/rum-runtime/src/runtime/options/core.rs:194	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
S016	crates/rum-runtime/src/runtime/options/read_phase.rs:93	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
S018	crates/rum-runtime/src/runtime/package_tools/persist_options.rs:67	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S019	crates/rum-runtime/src/runtime/tree_walkers/count_like/options.rs:53	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S020	crates/rum-stdlib/src/functional_ops/arrays/mod.rs:178	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
S021	crates/rum-stdlib/src/functional_ops/arrays/options.rs:71	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S022	crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:50	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
S023	crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:205	U	0/30	0/0/0/0	OrdinarySuccessPassThrough	A,M30,K,P30
S024	crates/rum-runtime/src/runtime/namespace_tools/names.rs:75	U	0/70	0/0/0/0	OrdinarySuccessPassThrough	A,M70,E20,K,P70
S025	crates/rum-runtime/src/runtime/builtin_adapters.rs:545	U	0/60	0/0/0/0	OrdinarySuccessPassThrough	A,M60,E20,K,P60
618.frameless-transforms.033	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
618.frameless-transforms.076	crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:74	U	0/30	0/0/0/0	Unsupported	T,A,M30,K,P30
618.optional-source-helpers.007	crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs:44	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
618.optional-source-helpers.019	crates/rum-runtime/src/runtime/definitions/incremental_append.rs:8	U	0/70	0/0/0/0	Unsupported	T,A,M70,E20,K,P70
618.optional-source-helpers.023	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:223	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
618.optional-source-helpers.025	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/table_fast_path.rs:11	U	0/75	0/0/0/0	Unsupported	T,A,M75,E20,K,P75
618.optional-source-helpers.027	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:71	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
618.optional-source-helpers.042	crates/rum-runtime/src/runtime/scoping/lexical/block/continuation.rs:573	U	0/80	0/0/0/0	Unsupported	T,A,M80,E20,K,P80
618.effect-questions.004	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:98	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
618.effect-questions.005	crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:103	U	0/65	0/0/0/0	Unsupported	T,A,M65,E20,K,P65
C1.P0942	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:65	N	0/85	0/0/0/0	NotYetSpecific	T,A,M85,E20,K,P85
C1.P0943	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:67	N	0/85	0/0/0/0	NotYetSpecific	T,A,M85,E20,K,P85
C1.P0949	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:148	N	0/85	0/0/0/0	NotYetSpecific	T,A,M85,E20,K,P85
C1.P0950	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:150	N	0/85	0/0/0/0	NotYetSpecific	T,A,M85,E20,K,P85
C1.P0952	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:180	N	0/85	0/0/0/0	NotYetSpecific	T,A,M85,E20,K,P85
C1.P0953	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:190	N	0/85	0/0/0/0	NotYetSpecific	T,A,M85,E20,K,P85
C1.P0956	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all_application.rs:23	N	0/80	0/0/0/0	NotYetSpecific	T,A,M80,E20,K,P80
C1.P0957	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all_application.rs:46	N	0/80	0/0/0/0	NotYetSpecific	T,A,M80,E20,K,P80
```

### 6. The port tables.

The governing manifest design is reused, not replaced. `port/source-entries.jsonl` has one entry for every working-population row. Proof entries carry their frozen plans, source fingerprints, native/helper attempts and raw receipts; their mutant field explicitly says a pausing mutation is inapplicable. The three proposed identity rows have 90 frozen keys, fixture bindings and executed receipts, with evaluated source/setup/answer fingerprints, copied fixture configuration and selected row-specific mutants. They remain pending, and no executed receipt is invented for blocked/deferred tuples. `port/retained-29-equal.tsv` preserves the accepted external settlements.

```json
{
  "source_entries": 1032,
  "settled_proof_rows": 23,
  "proposal_rows": 3,
  "proposal_tuple_bindings": 90,
  "proposal_actual_receipts": 90,
  "proposal_fixture_programs": 90,
  "retained_prior_equal_rows": 29,
  "active_new_equal_rows": 0,
  "owned_rows": 1009,
  "frozen_requirements": 75175,
  "production_manifest_implemented": false
}
```

Concrete files: `port/frozen-required-tuples.tsv`; `port/proposal-fixture-bindings.tsv`; `port/proposal-executed-receipts.tsv`; `port/fixtures/*.rum`, `*.setup.rum`, `*.json`; `port/proofs.jsonl`; `port/boundary-driver.rs`; `ledger-final.jsonl`; `ledger-final-results.json`. Current mutable diagnostic source and generic schema are under `mutant-source/` and `mutant-schema.tsv`; original v1 inputs/binaries remain in `mutant-source-v1/`, `bin-v1/`. Raw outputs were not overwritten by the resumed runs.

The final ledger model allows deferred rows only with C2 Open. It rejects every mandatory corruption independently, including borrowing P0508's raw caught mutant for the complete SAME identity row P0506, and promoting the authentic caught P0410 with an incomplete real matrix.

```text
FINAL LEDGER positive: GREEN identities=1032 proof_rows=23 owned_rows=1009 owner=Open
FINAL LEDGER NEGATIVE C2-done-with-residuals: RED (G15-C2 marked Done while owned residual/proposal rows remain)
FINAL LEDGER NEGATIVE delete-a-row: RED (source ledger identities differ from frozen population)
FINAL LEDGER NEGATIVE duplicate-a-row: RED (duplicate source ledger row)
FINAL LEDGER NEGATIVE promote-surviving-identity-mutant: RED (no authentic own caught mutant: P0506)
FINAL LEDGER NEGATIVE wrong-row-mutant: RED (caught mutant row differs from promoted row: P0506 <- P0508; no authentic own caught mutant: P0506)
FINAL LEDGER NEGATIVE promote-incomplete-matrix: RED (incomplete real required matrix: P0410)
test result: final ledger model positive=1 negatives=6 rejected=6 failures=0; C2 remains Open; production oracle NOT run
```

### 7. Open questions and recommendations.

- **Identity groups:** rule on the two explicitly proved groups (P0506/P0507, P0777). Recommendation: accept the proposed equivalences as a distinct disposition, retain their 90 tuple/own-mutant receipts, and keep the Equal criterion unchanged.
- **409 syntactic Result tails:** recommendation: review their genuine inherited caller transfers and retain every uncovered route/encoding requirement. Their identity-shaped local mutations cannot certify a substantive ancestor tail. Do not broadly promote the group from source syntax.
- **Enabled profiling outcomes:** recommendation: require a native observer of the existing non-duration phase/outcome counts for the profiling-result group. Source `profiling/timing.rs:375-449` shows a visible enabled counter effect, so semantic trace alone is insufficient for a NoObservableTail proposal.
- **Protected encoding gaps:** recommendation: obtain coverage from the existing saved eight batches where already justified, or leave the requirements blocked for an explicit director decision. Do not generate new internal-encoding probes or silently waive the cells.
- **Carrier/closure/native gaps:** recommendation: prioritize the explicit concrete-route table and actual enclosing return boundary before more corpus expansion. Ten survivor-class producers occur in closures; an inner return can leave the outer finish intact. The remaining unsupported carriers and CLI/Jupyter/native adapter paths need source-specific drivers/mutations; none was declared Unreachable from a zero count.
- **Eight new C1 edges:** recommendation: retain their distinct IDs and accepted C1 frame references, and reconcile their own producer/consumer transfer requirements during the next directed C2 work. The C1 frame gate was not recast as eight fabricated new edge settlements.
- **Sequence:** C2 is open. Do not mark it Done, port an all-green manifest or advance CHANGELOG/S10 from this result. A fresh director continuation is needed for more semantic probing after the stop-loss.

Scope and verification: no production source edit, no network request and no version-control command. No new unsafe was added. No source under the protected older scratch scenes was written. No production cargo test suite, two-feature full suspension oracle, benchmark, performance gate, CLI process or Jupyter request lifecycle ran in this resumed session; this is an evidence-only brief. Earlier checkpointed offline bounded builds/probes and the newly listed scratch E1 rebuilds, calibrations, timing rerun and revisits did run. The explicit native REPL lifecycle receipts are retained. The following hashes were checked before the authorized report append; the append procedure also verifies that the exact original brief prefix is preserved and the appended bytes equal the complete REPORT.md.

```text
SOURCE INTEGRITY {"added_or_deleted_files_outside_scratch": 0, "brief_append_only": false, "git_commands_run": 0, "network_requests_run": 0, "original_baseline_unchanged_files": 2092, "original_brief_prefix_sha256": "10f62b56d1c7a2b6672181118ce3f3b6f30aeafc85ba5e442ef3904cd22af97d", "resume_baseline_files": 2094, "resume_unchanged_except_brief": 2093, "source_changes": 0}
```


Final saved-control and artifact verification receipts (script-only recheck; no further semantic executions):

```text
SAVED V2 CONTROL Some baseline=1 SAME; actual own semantic omission hit; observer RED without final-value equality
SAVED V2 CONTROL G15A baseline=1 SAME; actual own semantic omission hit; observer RED without final-value equality
SAVED V2 CONTROL G15B baseline=1 SAME; actual own semantic omission hit; observer RED without final-value equality
SAVED V2 CONTROL C1 baseline=2 SAME; actual own semantic omission hit; observer RED without final-value equality
SAVED V2 CONTROL C1Closure baseline=2 SAME; actual own semantic omission hit; observer RED without final-value equality
FINAL ARTIFACT CHECK identities=1032 residuals=1006 exact_residual_list=PASS proposal_required_binding_receipts=90/90/90 evaluated_program_setup_answer_fingerprints=PASS retained29=UNCHANGED stop_loss=2 ledger_negatives=6/6_RED
FINAL SCOPE CHECK production_source_files_unchanged=2092; C2 remains OPEN; no new Equal or Lost claimed.
```

Native fresh-work control raw flags and artifact hashes:

```json
{
  "flags": [
    {
      "build": "real",
      "raw": "native/guard-fresh-real.jsonl",
      "sha256": "9400d3d50153f7bedd8745c6b3d6dbbc7937a293ca8482a66c48caaaa4205620",
      "id": "623.native.guard.fresh-work",
      "same": true,
      "valid": true
    },
    {
      "build": "disabled",
      "raw": "native/guard-fresh-disabled.jsonl",
      "sha256": "9400d3d50153f7bedd8745c6b3d6dbbc7937a293ca8482a66c48caaaa4205620",
      "id": "623.native.guard.fresh-work",
      "same": true,
      "valid": true
    },
    {
      "build": "mutant",
      "raw": "native/guard-fresh-mutant.jsonl",
      "sha256": "2707db2d8265d0bcbb12c84bdab119c92355c72997a896c15ee75dad41da749c",
      "id": "623.native.guard.fresh-work",
      "same": false,
      "valid": true
    }
  ],
  "own_arm_and_hit": true,
  "Equal": false,
  "reason": "Concrete guard body routes still incomplete"
}
```

Post-append hash verification (verbatim):

```text
SOURCE INTEGRITY {"added_or_deleted_files_outside_scratch": 0, "brief_append_only": true, "git_commands_run": 0, "network_requests_run": 0, "original_baseline_unchanged_files": 2092, "original_brief_prefix_sha256": "10f62b56d1c7a2b6672181118ce3f3b6f30aeafc85ba5e442ef3904cd22af97d", "resume_baseline_files": 2094, "resume_unchanged_except_brief": 2093, "source_changes": 0}
```
