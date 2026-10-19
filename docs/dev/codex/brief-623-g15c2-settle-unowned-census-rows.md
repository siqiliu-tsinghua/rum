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
