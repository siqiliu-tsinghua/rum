# brief-621 — G15-C design: settle the census by execution, and the obligation-keyed oracle

Scope: **NONE — no source change.** This is a design-only brief. It settles a
census by executed rum-vs-rum probes, designs the oracle manifest that will
keep that census settled, and drafts the follow-up implementation briefs.
Base: the G15-B tree, `dc70be18` (its code is `43a26c72`'s).

## Goal

When this brief is done, three things hold:

- **Every edge whose verdict rests on reading is settled by execution.** An
  edge is a pair of a may-suspend producer and the tail its caller runs
  after it. Each such edge on the current tree carries one of the verdicts
  below, backed by an executed rum-vs-rum probe in a context where a skipped
  tail is observable, or by a cited proof that guest code cannot reach it.
- **Every lost edge has an owner and a repair design.** Lost edges are
  grouped by mechanism, each group with an owner and a repair design.
- **The manifest is designed.** The oracle manifest that makes each
  settlement a standing obligation is designed, and its implementation brief
  is drafted, together with one brief per new repair group.

"Unsettled" is not a verdict, and neither is a verdict reached only by
reading.

## Why this brief exists

| Census | Unsettled at the end | How they were "settled" |
| --- | --- | --- |
| brief-548 | 495 | Left `Unsettled`. |
| brief-583 (the re-census) | 597 | All 597 resolved by source review. It found four Lost (G9, G10) and recorded the rest as 412 TailCorrect, 135 FrameCarries and 46 Boundary (`.temp/brief-583/unsettled-resolution.tsv`), with 14 probes in total. |

Two of brief-583's reading verdicts were wrong, and G15 is what they cost:

- **P1316** (`try_apply_function -> apply_status_object_to_args`) was called
  TailCorrect: "Fixed-point caller evaluates the final Expr". A resumed
  `Return` then escaped the Function body (G15-A, brief-619).
- **P0307** (`evaluate_definition_lhs_active_expr -> evaluate_atom_fast_path`)
  was called FrameCarries: "the driver Await phases, including own-symbol
  cycle handling". The resumed path had no cycle handling (G15-B,
  brief-620).

Both errors have the same shape. The fixed point re-evaluates a returned
value, and a nearby frame exists, but neither runs a Rust continuation that
disappeared. E1's oracle could not see either one, because its contexts
compared top-level values the fixed point repairs (brief-618 §3, "Why
previous work did not falsify these instances").

The same argument may stand behind other verdicts. Two populations rest on
it: every verdict that brief-583's review assigned, and every `FrameCarries`
verdict, since a frame's existence is exactly the claim that failed for
P0307.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`.
- **brief-548:** `docs/dev/codex/brief-548-suspension-continuation-sweep.md`
  (the uncertainty classes are at `:786-791`) and `.temp/brief-548/`, which
  holds the census and the tools `build_graph.py`, `classify.py` and
  `census.tsv`.
- **brief-583:** `docs/dev/codex/brief-583-suspension-oracle-design.md` §2-3
  and `.temp/brief-583/`, which holds the adapted tools (`census.py`,
  `build_graph.py`, `classify.py`, `audit.py`) and these tables:
  - `reviewed-census.tsv` (2 468 rows with the supplemental edges);
  - `unsettled-resolution.tsv` and `historical-unsettled-resolution.tsv`;
  - `supplemental-edges.tsv` and `new-probes.tsv`.
- **brief-618:** `docs/dev/codex/brief-618-g15-status-projection-tail-design.md`
  and `.temp/brief-618/REPORT.md` §3. The **observation model and required
  contexts** table and the **exact context closure** paragraph are accepted
  design; build on them, do not restate them. Its screens are in
  `.temp/brief-618/`:
  - `frameless-direct-evaluate.tsv` (56);
  - `frameless-transforms.tsv` (143);
  - `optional-source-helpers.tsv` (57);
  - `effect-questions.tsv` (20);
  - `cases.tsv` (136 pairs).
- **The oracle:** `crates/rum-stdlib/tests/suspension_oracle/` — the
  manifest, descriptors, contexts and fixtures — and its divergence ledgers.
- **The owners:** the G13 step drafts `.temp/brief-606/draft-*.md` (S10-S54,
  F1-F23, R) and their partition, the work S13 and S19 already own, and
  E2's acceptance criteria (brief-618's review, change 2).
- **The comparator:** the director's probes `.temp/director/*-probe/pairs.py`
  and `.temp/director/depth-leak-probe/`.

## Scope

- **Files you MAY modify:** none under `crates/`, `docs/` (except appending
  your report to this brief), `scripts/`, or anywhere tracked.
- **Scratch:**
  - All scratch goes under `.temp/brief-621/`. That includes copied and
    adapted census tools, generated probe programs, and a probe crate
    outside the workspace that depends on `rum-host` by path, like
    `.temp/director/depth-leak-probe/`.
  - Do not add a scratch test under `crates/`. brief-583 and brief-618
    each did, and it is a source change.
  - Mutation experiments, if any, run on a copied tree under
    `.temp/brief-621/`, never on the working tree.
- Do not edit `.temp/brief-548/`, `.temp/brief-583/` or `.temp/brief-618/`.
  Copy what you adapt.

## Steps

1. **Re-census the current tree, calibrated.**
   - Copy brief-583's tooling into `.temp/brief-621/census/` and run it on
     the G15-B tree, including the supplemental-edge pass.
   - **Calibrate it before using it**, against known answers. The re-run
     must classify every former G1-G12 Lost site and G9/G10 as repaired, and
     must see P1316 and P0307's successors as the repaired G15 sites. Report
     each check. A tool that disagrees with a known answer is fixed, or its
     disagreement is explained, before any row is settled with it.
   - Cross-walk every brief-583 row (all 2 468) and every brief-548
     Unsettled row (495) to a current row or to `Retired`, with the brief or
     commit that deleted the site. No old row disappears unaccounted.
   - New rows added since brief-583 (S1-S9, G11-G15 code) enter the
     population like any other.
2. **Fix the population to settle by execution.** It is the union of:
   - every current row whose verdict is not mechanically established by the
     tool (every row the tool leaves `Unsettled`, which brief-583 resolved by
     reading);
   - every `FrameCarries` row, whether tool- or review-assigned;
   - every review-assigned `TailCorrect` row;
   - brief-618's four screens.

   State the exact count of each and the overlaps, then **ask yourself what
   is excluded and why.** A tool-assigned `TailCorrect` (a strict tail: the
   producer's `Result` is returned whole) moves the obligation to its
   caller. That caller is itself a row, and that is how it is covered.
   Report the rule and its count.
3. **Settle every row.** The verdict vocabulary is closed:

   | Verdict | Evidence required |
   | --- | --- |
   | `Equal` | Executed resumed-vs-queued probes that pause *in this producer*. The pause is attributed by the harness frame census, or by a cited guest-to-site call chain plus an asserted pause count. The probes run in every context brief-618's table assigns to this row's tail policy (name the policy). Pause mid-loop and twice where the variants apply. |
   | `Lost` | A minimized DIFF witness, its mechanism, and the tail it skips with `path:line`. |
   | `Unreachable` | A cited proof that no guest program can make this producer return `InputRequired` (for example, its callee chain evaluates no guest expression), plus the strongest probe that tries to reach it and its observed pause count. Kept as a separate, counted class; I review each one. |
   | `Boundary` | A documented non-resumable host batch API or retirement boundary, citing the documentation or ruling (brief-548's U6, `ValueQ`, monitor callbacks). |
   | `Retired` / `Duplicate` | From step 1's cross-walk. |

   - A probe that never pauses is invalid, never `Equal`.
   - A context that cannot observe this row's tail (a top-level value the
     fixed point repairs) does not count.
   - For every `Equal` row, say in one line which context observes the tail
     and how.
   - Write results incrementally to `.temp/brief-621/settled.tsv`.
4. **Group what is lost.** Group by mechanism, not by builtin. For each group
   give:
   - its root cause, with `path:line`;
   - its witnesses;
   - its owner.

   An owner is either an existing G13 step whose draft already covers the
   site (cite the draft line; the step's brief then must include the
   witness) or a new repair step **G15-C*n***.

   Then state whether S10 can proceed before each new group, and why.
5. **Design the obligation-keyed manifest.** Make brief-618's exact context
   closure concrete. Settle these:
   - **The obligation key** `(producer identity, consumer identity, tail
     policy)`: how it is derived from the census rather than the catalog, and
     how a new producer/consumer edge enters the required set.
   - **The tail-policy enum**: exhaustive, with no default arm. Assign each
     of step 3's rows to a policy, and each policy to its required contexts.
   - **Exactness**: the required tuple set
     `(entry, descriptor, variant, context, consumer_route)`, computed from
     the policies, is compared with the *executed* set. The test is red on an
     omission, a duplicate, a declared-but-unexecuted tuple, or a missing
     pause at the designated phase.
   - **Porting step 3's probes**: each settled row's probe becomes its
     fixture, keyed by its tuple.
   - **The divergence ledger**: how known `Lost` rows with an owner enter as
     immutable baseline with a named promotion, so that a new difference is
     red and never silently baselined.
   - **Mutation controls**:
     - the manifest-level ones from brief-618;
     - semantic ones: remove `Some` on an `Option` stage; skip G15-A's
       Function-stage Return finish; skip G15-B's recovery in one awaited
       variant; one per new Lost group.

     Each must turn the oracle red independently of the final value.
   - **Its run cost**: the time per oracle build, keeping the oracle runnable
     inside the S9 bounds, with a budget if it grows.
6. **Draft the follow-up briefs** in `TEMPLATE.md` shape, as raw material
   for my rewrite:
   - the manifest implementation brief;
   - one brief per new repair group;
   - for each existing owner step, the witness rows its brief must add.

   Propose the sequence relative to `CHANGELOG.md` and S10. My standing
   order is: G15-C's repairs and the manifest, then CHANGELOG, then S10.

## Measurement

None beyond the probes. No timing, instruction or stack work. Probe
programs are rum-vs-rum only: resumed against queued input in the same
build. Never compare against another engine.

## rum / Rust pitfalls (read twice)

- **A verdict by reading is a hypothesis.** Both wrong brief-583 verdicts
  had a plausible reason attached. When the reason is "a frame carries it",
  find the frame's resume and show that it runs the *same* tail as the
  ordinary path. A frame with equal `Input` and `Output` types proves
  nothing about the tail.
- **Calibrate the comparator on a known DIFF before believing an EQUAL.**
  Before the first `Equal` is recorded:
  - build a second copy of your probe crate against
    `.temp/brief-619/base-source/`, the S9 export that precedes both G15
    repairs;
  - show that it reports DIFF on the G15-A witnesses
    (`.temp/director/g15-probe/pairs.py`) and the G15-B witnesses
    (`.temp/director/s-g15b-probe/pairs.py`);
  - show that the current-tree build reports SAME on both sets.

  A comparator that cannot tell those two trees apart cannot settle a row.
- **Probe the structure, not only the value.** Use these contexts:
  - Return absorption;
  - effect-log order;
  - stored definitions;
  - held and spliced results;
  - messages;
  - a second pause;
  - a fresh request after completion.

  brief-618's table says which context applies to which policy.
- **The content filter.** These are semantic equality probes, not crash
  reproductions. If Codex's content filter interrupts a run, checkpoint and
  stop; do not reword the probes.

## Verification (MANDATORY — the evidence is the deliverable)

Paste verbatim:

- the calibration checks of step 1 and of the comparator;
- the population counts of step 2;
- the verdict totals of step 3, overall and per source population;
- every `Lost` witness and every `Unreachable` proof in full;
- a table of `Equal` rows by policy and context;
- the probe run's own test-result lines.

`settled.tsv` must hold one row per population member, with its verdict,
evidence, probe identifiers and policy. A row without a verdict is a failed
deliverable, not a residual.

## Checkpointing — protect the scene

Write `.temp/brief-621/PROGRESS.md` after each step and after every 100
settled rows. It records:

- the counts so far;
- the next row to settle;
- the probe harness's build state;
- whether the working tree is untouched (it must be).

## Stop-loss

The progress counter is **the number of rows settled**. STOP and report if
two consecutive work sessions of a reasonable size settle none. Also STOP if
the calibration of the census tool or of the comparator fails in a way you
cannot explain from the source.

## Hard rules

- Do NOT run git in any form. Leave the tree exactly as you found it; the
  only tracked-file write is appending the report to this brief.
- No new `unsafe`, no network, and nothing outside this repository.
- Rum against rum only (`AGENTS.md` Testing Policy).
- Paste real output. Say plainly what did not run.

## Report format (append to this brief)

1. The re-census: tool calibration, counts, and the cross-walk summary,
   with the cross-walk files named.
2. The population and its exclusion rule.
3. The verdicts: totals, and `Lost` and `Unreachable` in full, with
   `settled.tsv` named.
4. The lost groups and their owners.
5. The manifest design.
6. The drafted briefs, as files under `.temp/brief-621/`, each summarised.
7. Open questions for the director, each with a recommendation.

## Director continuation 1 (2026-10-19) — policy first, then settle by coverage

The run stopped correctly under my stop-loss, and its foundation is
accepted. Accepted:

- the calibrated re-census (2 682 edges; all 86 historical Lost and the G9-G15
  controls reproduce);
- both cross-walks (2 468/2 468 and 495/495);
- the 2 186-member population and its exclusion rule (513 strict tails
  transfer to an indexed caller; 283 without one stay in);
- the comparator calibration (22 DIFF on the pre-G15 export, 34/34 SAME on
  the current tree);
- the exact-span attribution build;
- the four `Equal` LHS rows;
- the three `Lost` rows. They are existing owned work: P0755 belongs to S10
  (T0028), P1489 to S13 (F0528) and P0883 to S19 (F0358). Their witnesses
  join those steps' briefs.

The eight accounting-bridge `Unreachable` proofs are accepted. Each is
pinned to a call site whose closure is concrete and pure, and that is what
`Unreachable` must mean. They are false positives of the may-evaluate graph,
which is worth saying in the report's class description.

**What went wrong was my brief's shape, not the work.** Step 3 required each
row to run every context its *tail policy* demands, but the policies are
step 5's subject. Each row therefore needed a bespoke policy and probe matrix
before it could settle. Taken in ID order, P0001 and P0002 (the accounting
bridge, the hardest rows to reach from guest code) came first and consumed
both sessions. The stop-loss as I worded it then fired at 15 rows. Both
faults are mine. This continuation reorders the work and replaces the
stop-loss.

**Reordered steps** (step 1, step 2 and the 15 settlements stand):

- **A. Tail policy for every row, before any further settlement.**
  - Define the exhaustive tail-policy enum, with no default arm. Seed it
    from brief-618's context table and the policies you already used
    (`DefinitionLhsPreparation`, `MatchingSymbolCycleRecovery`,
    `OptionalStage`, `FunctionResult`).
  - Assign one or more policies to every population row from the source
    that runs after its producer, up to the function's return or the
    frame's boundary. A strict tail inherits its caller's obligation, as
    step 2 established.
  - Each policy maps to its required contexts and variants.
  - Classify mechanically where the shape allows, then review. Report the
    per-policy counts and every row the classifier could not place.
  - This enum *is* step 5's enum, so the manifest design starts from it.
- **B. A context-rich corpus, run once in the attribution build.**
  - Generate probes systematically rather than per row:
    - every E1 descriptor and variant, wrapped in each context family its
      reached rows' policies require (Return boundary after the subject,
      before/middle/after log markers, guest-head answers, held and spliced
      results, Throw and Return in the produced value, a second pause, and
      a fresh request);
    - brief-618's 136 pairs, the 195 legacy controls, your extras and the
      director probe sets;
    - targeted families for population rows no descriptor reaches.
  - Run the whole corpus on both the unmodified build and the attribution
    build. Record the resulting coverage matrix (row × context × variant →
    SAME/DIFF/invalid) in `.temp/brief-621/coverage.tsv`.
- **C. Settle by coverage.**
  - A row is `Equal` when attributed, pausing probes cover every context its
    policies require, all SAME.
  - Any attributed DIFF makes the row a `Lost` candidate: minimize it, then
    record it.
  - An uncovered row gets a targeted probe family. If a bounded search
    cannot reach it, it gets a call-site `Unreachable` proof of the
    accepted kind.
  - Work family by family, ordered by how many rows a family settles. Take
    the hardest singletons last.
- **D. Group the `Lost`, and E. design the manifest (step 5).** The
  manifest's required tuples are the coverage matrix's keys, and its
  fixtures are the corpus's probes. A starts the design and B gives it its
  data, so step 5 is the formalization of A-C plus the exactness test, the
  ledger and the mutation controls. **F. Draft the follow-up briefs (step 6).**
- **Six unexpected extra pauses.** The 331-pair panel reported `invalid=6`.
  Identify each by id and source. Settle each the same way: an extra pause
  is a request-behaviour divergence, and may be `Lost`.

**Stop-loss (replaces the brief's).**

- The work is measured in **batches**. A batch is one probe family, or at
  least 200 executed probes.
- Progress means rows that gain a policy (A), a coverage cell (B) or a
  verdict (C).
- A row that resists one bounded targeted attempt is **parked**, with what
  was tried, and the work moves on. A hard row is never a reason to stop.
- STOP only after two consecutive batches that advance none of the three
  counts, or if a calibration fails.
- Revisit parked rows after C. Any row still without a verdict at the end
  is listed with its evidence, for my ruling. It is not left blank.

**Report.**

- Add a section *Continuation 1* at the end. It holds per-policy counts,
  the coverage summary, verdict totals overall and per source population,
  every `Lost` witness and every new `Unreachable` proof in full, the
  parked list, the manifest design and the drafted briefs.
- Rewrite the opening summary to describe the final state.

Checkpoint after each batch.

## Director continuation 2 (2026-10-19) — resume from the checkpoint

Continuation 1 ran for about 30 minutes and then ended on an upstream
service interruption. Its log is kept as `cdx_brief_621.filter-1.log`. Its
scene is intact, and its work is accepted as checkpointed in
`.temp/brief-621/PROGRESS.md` and `continuation1/batches.jsonl`:

- policies for all 2 186 rows (batches A1-A2);
- about 2.2 million coverage cells across the value, trace, log,
  Return-boundary, Throw, Return-propagation, internal-encoding,
  held-and-splice and consumer-route batches;
- 73 `Unreachable`, 4 `Equal` and 5 `Lost`, with the rest pending.

This is a fresh session. Read continuation 1 above and the checkpoint, then
continue from it. Do not regenerate completed batches.

**One change to scope.** Generate and run no further probes in the
internal-encoding context families: the batches named
`e1-guest_heads.*` in `batches.jsonl`, in which a guest program's value has
the shape of an internal continuation encoding. All eight are complete.

- **What their differences mean.** A difference in those batches, where the
  queued run keeps the value as written and the resumed run interprets it,
  has one mechanism: a lenient legacy decoder at an `Option` slot, reached
  because a typed completion (usually `Some`) did not run on the resumed
  path. That mechanism is G13's subject. The remaining typing steps and
  their flips (F1-F23), and the decoder's removal (R), remove it
  structurally.
- **How to settle them.** Classify those differences **by script only**,
  from the recorded observations, as a table of row, frame or slot identity
  from attribution, the owning G13 step (from `.temp/brief-606/`
  partitions), and count. Owner `G13-transport` is a valid owner for this
  context. Do not minimize or discuss them individually.
- **Escalate anything else.** A difference in those batches that does
  *not* have that signature (the queued and resumed values differ in some
  other way) is a skipped-tail candidate. Settle it through the other
  context families.
- **What a row still needs.** A row still needs its other required contexts
  before it is `Equal`.

Everything else in continuation 1 stands: settle by coverage, targeted
families ordered by leverage, parked rows, D-F, and the batch stop-loss.

---

## Execution report — brief-621 (stopped under the prescribed stop-loss)


**INCOMPLETE. 15 of 2,186 population members are settled: 4 Equal, 3 Lost and 8 Unreachable. The other 2,171 rows deliberately have blank verdicts in `.temp/brief-621/settled.tsv`. This fails the exhaustive-settlement deliverable. Neither the passing probe totals nor the source classifications are a claim that the census is settled.**

The stop-loss applies: “STOP and report if two consecutive work sessions of a reasonable size settle none.” After reaching 15 settlements, the contextual option session and the public retained-prefix fixture session each added zero complete row verdicts. The latter established a new producer route but did not finish its required context matrix. Work stopped there. Steps 5–6 were not reached; no implementation draft is ready for dispatch.

All scratch, including the external Cargo crates, targets, copied source tree and its diagnostic edits, is under `.temp/brief-621/`. No git or network command ran. No scratch test was created under `crates/`, no original Rust/Python/TOML source was edited, and no unsafe was introduced. The only original-tree write is this appended report. Baseline hashing found 2,088 existing files unchanged before the append; the final check below excludes only this brief. The worktree was never queried through git.

### 1. The re-census, calibration and cross-walk


Copied all eight brief-583 Python tools into `census/`. Only the adapted AST inventory, graph, classifier and supplemental discovery were used for the current inventory. The ID-specific reading overlay and historical audit assumptions were not replayed. The AST visitor runs in `census/ast-probe/`, outside the workspace, rather than installing the prohibited temporary integration test.

The initial old-tool pass omitted typed macros. The saved current pass includes `resume_typed_or_suspend!`, `TypedFrame` implementations, associated `Route::Error`/`TypedAction` results and explicit `InputRequired` constructors/forwarders as conservative graph seeds. `FrameAction`-encoding resume plumbing is **not** a strict tail, so that old TailCorrect rule was demoted to Unsettled. The supplemental pass is independent of the historical review overlay and gives its discovered edges Unsettled hypotheses.

```text
   Compiling brief621-ast-probe v0.0.0 (/home/remi/dev/rum/.temp/brief-621/census/ast-probe)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.12s
     Running `.temp/brief-621/census/target/debug/brief621-ast-probe`
AST files=762 functions=8844 calls=68732 macros=3520 frame_records=1142 macro_parse_failures=0
```

```text
functions=8844 may_functions=1840 candidate_pairs=2657
Counter({('rum-runtime', 'unique-name/receiver'): 1546, ('rum-stdlib', 'unique-name/receiver'): 926, ('rum-runtime', 'ambiguous'): 76, ('rum-cli', 'unique-name/receiver'): 30, ('rum-stdlib', 'ambiguous'): 26, ('rum-runtime', 'dynamic-requires-review'): 19, ('rum-repl', 'unique-name/receiver'): 12, ('rum-host', 'unique-name/receiver'): 10, ('rum-jupyter', 'unique-name/receiver'): 10, ('rum-host', 'dynamic-requires-review'): 1, ('rum-cli', 'dynamic-requires-review'): 1})
```

```text
Candidate pairs 2657
Counter({'Unsettled': 1163, 'TailCorrect': 897, 'FrameCarries': 597})
Lost groups Counter()
Fieldless Counter({'FrameCarries': 73, 'TailCorrect': 13, 'Unsettled': 7})
```

The supplemental pass adds 25 edges, giving 2,682 current graph edges. `census/supplement-run.log`, `census/supplemental-edges.tsv` and `census/supplemental-outer-review.tsv` preserve the full pass. Source-index resolution remains conservative, including dynamic and ambiguous edges; it is not rustc resolution and does not establish semantic equality.

Calibration is a structural cross-walk plus executed known controls, not an automatic semantic certificate for every successor. The full individual checks follow. Three obsolete historical matcher wrappers are retired by brief-571; their historical “Removed” disposition is preserved. The G11 panic-finalization checks observe the repaired public witnesses; a list of multiple successor edges does not mean that every listed edge paused in that witness. The G15-B leaf remains Unsettled in the local-frame heuristic because its caller owns recovery; its executed error contexts, not a nearby frame, settle it below.

```text
G5 P0118 Repaired-control-SAME successors=P0126;P0130;P0131;P0127;S002 probes=548.Monitor-options;548.Monitor-EvaluationMonitor;548.Monitor-StepMonitor;548.Monitor-options-twice
G5 P0119 Repaired-control-SAME successors=P0126;P0130;P0131;P0127;S002 probes=548.Monitor-options;548.Monitor-EvaluationMonitor;548.Monitor-StepMonitor;548.Monitor-options-twice
G5 P0120 Repaired-control-SAME successors=P0126;P0130;P0131;P0127;S002 probes=548.Monitor-options;548.Monitor-EvaluationMonitor;548.Monitor-StepMonitor;548.Monitor-options-twice
G5 P0122 Repaired-control-SAME successors=P0126;P0130;P0131;P0127;S002 probes=548.Monitor-options;548.Monitor-EvaluationMonitor;548.Monitor-StepMonitor;548.Monitor-options-twice
G7 P0125 Repaired-control-SAME successors=P0145;P0146 probes=548.Sow-match;548.Sow-twice
G5 P0157 Repaired-control-SAME successors=P0186;P0187;P0188;S004 probes=548.TestReport-option-SameTest;548.TestReport-option-TimeConstraint;548.TestReport-option-MemoryConstraint
G5 P0158 Repaired-control-SAME successors=P0186;P0187;P0188;S004 probes=548.TestReport-option-SameTest;548.TestReport-option-TimeConstraint;548.TestReport-option-MemoryConstraint
G5 P0159 Repaired-control-SAME successors=P0186;P0187;P0188;S004 probes=548.TestReport-option-SameTest;548.TestReport-option-TimeConstraint;548.TestReport-option-MemoryConstraint
G5 P0161 Repaired-control-SAME successors=P0186;P0187;P0188;S004 probes=548.TestReport-option-SameTest;548.TestReport-option-TimeConstraint;548.TestReport-option-MemoryConstraint
G5 P0167 Repaired-control-SAME successors=P0186;P0187;P0188;S004 probes=548.TestReport-two-args;548.TestReport-option-SameTest
G5 P0168 Repaired-control-SAME successors=P0186;P0187;P0188;S004 probes=548.TestReport-two-args;548.TestReport-option-SameTest
G5 P0193 Repaired-control-SAME successors=P0228;P1139;P1140;S009;S010 probes=548.Trace-Heads-option
G7 P0199 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0200 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0201 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0202 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0203 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0204 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0205 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0206 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0208 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0209 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0210 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0211 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G7 P0213 Repaired-control-SAME successors=P0216;P0217;P0214 probes=548.Trace-pattern;548.TraceDetail-nested-pattern;548.TracePrint-nested-pattern;548.Trace-filter-levels;548.Trace-filter-heads;548.TraceDetail-filter-levels;548.TraceDetail-filter-heads;548.Trace-twice;548.TraceDetail-twice
G4 P0342 Repaired-control-SAME successors=P0416;P0467;P0468;P0469 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0344 Repaired-control-SAME successors=P0418;P0467;P0468;P0469 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0350 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0351 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0352 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0353 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0354 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0355 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0356 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G4 P0357 Repaired-control-SAME successors=P0470;P0471 probes=548.Rule-plain;548.RuleDelayed-plain;548.Rule-pattern;548.RuleDelayed-pattern;548.Rule-Blank;548.Rule-PatternTest;548.Rule-Condition;548.Rule-wrapper;548.Rule-head;548.Rule-twice
G5 P0452 Repaired-control-SAME successors=P0572;P1142;P1143;S011;S012 probes=548.CheckAbort-option
G5 P0711 Repaired-control-SAME successors=P0865;P1139;P1140;S009;S010 probes=548.Cases-Heads-option;548.Cases-Heads-after-target
G5 P0714 Repaired-control-SAME successors=P0866;P1139;P1140;S009;S010 probes=548.Cases-Heads-option;548.Cases-Heads-after-target
G5 P0764 Repaired-control-SAME successors=P0920;P1139;P1140;S009;S010 probes=548.DeleteCases-Heads-option;548.DeleteCases-Heads-after-target
G5 P0767 Repaired-control-SAME successors=P0921;P1139;P1140;S009;S010 probes=548.DeleteCases-Heads-option;548.DeleteCases-Heads-after-target
G4 P0774 Repaired-control-SAME successors=P0936;P0937 probes=548.MatchQ-pattern-args;548.MatchQ-pattern-head;548.MatchQ-pattern-twice
G4 P0776 Repaired-control-SAME successors=P0936;P0937 probes=548.MatchQ-pattern-args;548.MatchQ-pattern-head;548.MatchQ-pattern-twice
G4 P0777 Repaired-control-SAME successors=P0936;P0937 probes=548.MatchQ-pattern-args;548.MatchQ-pattern-head;548.MatchQ-pattern-twice
G4 P0779 Retired successors= probes=
G5 P0848 Repaired-control-SAME successors=P1005;P1139;P1140;S009;S010 probes=548.Replace-Heads-option
G8 P0882 Repaired-control-SAME successors=P1039;P0276 probes=548.ReplaceRepeated-predicate;548.ReplaceRepeated-twice
G5 P0962 Repaired-control-SAME successors=P1124;S008 probes=548.Names-SpellingCorrection;548.Names-ResolveContextAliases;548.Names-IgnoreCase
G5 P0963 Repaired-control-SAME successors=P1124;S008 probes=548.Names-SpellingCorrection;548.Names-ResolveContextAliases;548.Names-IgnoreCase
G5 P0964 Repaired-control-SAME successors=P1124;S008 probes=548.Names-SpellingCorrection;548.Names-ResolveContextAliases;548.Names-IgnoreCase
G6 P0973 Repaired-control-SAME successors=P1132;P1135;P1136 probes=548.OptionValue-many;548.OptionValue-twice
G5 P0979 Repaired-control-SAME successors=P1142;P1149 probes=548.Cases-Heads-option;548.ArrayFilter-option;548.CheckAbort-option
G5 P0997 Repaired-control-SAME successors=P1165;P1147;P1148;S013;S014;S015 probes=548.DumpSave-option
G4 P1019 Retired successors= probes=
G4 P1285 Retired successors= probes=
G5 P1385 Repaired-control-SAME successors=P1603;P1139;P1140;S009;S010 probes=548.Count-Heads-option;548.MemberQ-Heads-option;548.FreeQ-Heads-option
G5 P1389 Repaired-control-SAME successors=P1606;P1139;P1140;S009;S010 probes=548.Count-Heads-option;548.MemberQ-Heads-option;548.FreeQ-Heads-option
G5 P1393 Repaired-control-SAME successors=P1609;P1139;P1140;S009;S010 probes=548.Count-Heads-option;548.MemberQ-Heads-option;548.FreeQ-Heads-option
G5 P1401 Repaired-control-SAME successors=P1618;P1139;P1140;S009;S010 probes=548.Level-Heads-option
G5 P1410 Repaired-control-SAME successors=P1635;P1139;P1140;S009;S010 probes=548.Position-Heads-option
G2 P1483 Repaired-control-SAME successors=P1708;P2144 probes=548.LazyMap;548.LazyMap-twice
G1 P1692 Repaired-control-SAME successors=P1921 probes=548.Scan;548.Scan-twice
G8 P1707 Repaired-control-SAME successors=P1937;P1951 probes=548.Dot-chain;548.Dot-twice
G5 P1708 Repaired-control-SAME successors=P1940;P1142;P1143;S011;S012 probes=548.ArrayFilter-option
G2 P1898 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G2 P1899 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G2 P1901 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G2 P1902 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G2 P1904 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G2 P1906 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G2 P1907 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G2 P1908 Repaired-control-SAME successors=P2141;P1472;P1473 probes=548.LazyTable-bound-1;548.LazyTable-bound-2;548.LazyTable-bound-3
G2 P1909 Repaired-control-SAME successors=P2142;P1472;P1473 probes=548.LazyTable-bound-1;548.LazyTable-bound-2;548.LazyTable-bound-3
G2 P1910 Repaired-control-SAME successors=P2143;P1472;P1473 probes=548.LazyTable-bound-1;548.LazyTable-bound-2;548.LazyTable-bound-3
G2 P1913 Repaired-control-SAME successors=P2132;P2133;P2134;P2135;P2137 probes=548.LazyMap;548.LazyNestList;548.LazySelect;548.LazyFoldList;548.LazyDrop-inner;548.LazyMap-inner;548.LazySelect-inner;548.LazyFoldList-inner;548.LazyMap-twice;548.LazyNestList-twice;548.LazySelect-twice;548.LazyFoldList-twice;548.LazyDrop-twice
G3 P2056 Repaired-control-SAME successors=P2287;P2288 probes=548.ConformSchema-leaf;548.ConformSchema-association;548.ConformSchema-default;548.ConformSchema-default-predicate;548.SchemaErrors-leaf;548.SchemaErrors-association;548.SchemaErrors-default;548.Schema-twice;548.Schema-defaults-twice
G3 P2061 Repaired-control-SAME successors=P2287;P2288 probes=548.ConformSchema-leaf;548.ConformSchema-association;548.ConformSchema-default;548.ConformSchema-default-predicate;548.SchemaErrors-leaf;548.SchemaErrors-association;548.SchemaErrors-default;548.Schema-twice;548.Schema-defaults-twice
G3 P2065 Repaired-control-SAME successors=P2287;P2288 probes=548.ConformSchema-leaf;548.ConformSchema-association;548.ConformSchema-default;548.ConformSchema-default-predicate;548.SchemaErrors-leaf;548.SchemaErrors-association;548.SchemaErrors-default;548.Schema-twice;548.Schema-defaults-twice
G3 P2066 Repaired-control-SAME successors=P2287;P2288 probes=548.ConformSchema-leaf;548.ConformSchema-association;548.ConformSchema-default;548.ConformSchema-default-predicate;548.SchemaErrors-leaf;548.SchemaErrors-association;548.SchemaErrors-default;548.Schema-twice;548.Schema-defaults-twice
G3 P2067 Repaired-control-SAME successors=P2287;P2288 probes=548.ConformSchema-leaf;548.ConformSchema-association;548.ConformSchema-default;548.ConformSchema-default-predicate;548.SchemaErrors-leaf;548.SchemaErrors-association;548.SchemaErrors-default;548.Schema-twice;548.Schema-defaults-twice
G3 P2068 Repaired-control-SAME successors=P2287;P2288 probes=548.ConformSchema-leaf;548.ConformSchema-association;548.ConformSchema-default;548.ConformSchema-default-predicate;548.SchemaErrors-leaf;548.SchemaErrors-association;548.SchemaErrors-default;548.Schema-twice;548.Schema-defaults-twice
G3 P2069 Repaired-control-SAME successors=P2287;P2288 probes=548.ConformSchema-leaf;548.ConformSchema-association;548.ConformSchema-default;548.ConformSchema-default-predicate;548.SchemaErrors-leaf;548.SchemaErrors-association;548.SchemaErrors-default;548.Schema-twice;548.Schema-defaults-twice
G8 P2311 Repaired-control-SAME successors=P2544;P2547 probes=548.Differences;548.Differences-twice;548.Differences-order-two
G8 P2312 Repaired-control-SAME successors=P2547;P2551 probes=548.Differences;548.Differences-twice;548.Differences-order-two
G2 P2353 Repaired-control-SAME successors=P2591;P2145 probes=548.LazyTake-consumer;548.LazyPart-consumer
G5 P2354 Repaired-control-SAME successors=P2592;P1139;P1140;S009;S010 probes=548.Depth-Heads-option
G2 P2355 Repaired-control-SAME successors=P2593;P2145 probes=548.LazyFirst-consumer
G10 P0226 Repaired-control-SAME successors=P0249 probes=583.default-decline-twice
G9 P0354 Repaired-control-SAME successors=P0433 probes=583.destructure-result;583.destructure-suffix;583.destructure-twice;583.destructure-nested
G9 P0355 Repaired-control-SAME successors=P0403 probes=583.destructure-result;583.destructure-suffix;583.destructure-twice;583.destructure-nested
G9 P0356 Repaired-control-SAME successors=P0403 probes=583.destructure-result;583.destructure-suffix;583.destructure-twice;583.destructure-nested
G11 capture Repaired-control-SAME successors=P0123 probes=G11.selector;G11.capture
G11 C1 Repaired-control-SAME successors=P0181 probes=G11.report
G11 C2 Repaired-control-SAME successors=P0553;P0554;P0555;P0556;P0557;P0558;P0559 probes=G11.assoc
G11 C3 Repaired-control-SAME successors=P0899;P0900;P0901;P0902;P0903;P0904;P0905;P0906;P0907;P0908;P0909;P0910;P0911;P0912;P0913 probes=G11.delete
G11 quiet Repaired-control-SAME successors=P0691 probes=G11.quiet
G12 head Repaired-control-SAME successors=P2485 probes=G12.head
G15A P1316 Repaired-control-SAME successors=P1477 probes=G15A.01;G15A.02;G15A.03
G15B P0307 Repaired-control-SAME successors=P0368 probes=G15B.01;G15B.02;G15B.03;G15B.04;G15B.12
CENSUS CALIBRATION PASS: 86 historical Lost + 4 G9/G10 + 8 G11/G12/G15 checks; no semantic Equal inferred from classification.
Historical cross-walk: 2468/2468 brief-583 records and 495/495 brief-548 Unsettled records mapped; mappings do not inherit reading verdicts.
```

The same host comparator was built separately against the unchanged current tree and `.temp/brief-619/base-source/`, with offline Cargo and separate targets. It compares complete host outcomes, per-channel output prefixes, prescribed projections, definition snapshots, queue/pending ownership and a fresh request. The pre-repair G15-B failure can terminate before a second planned pause; those four count violations are retained as defects, never accepted as Equal. Six G15-A and sixteen G15-B differences on that export, followed by all 34 current-tree pairs SAME, calibrate the comparator before any Equal settlement.

```text
test result: total=34 SAME=12 DIFF=22 invalid=4
base G15A total=14 SAME=8 DIFF=6 invalid=0
base G15B total=20 SAME=4 DIFF=16 invalid=4
base G15A.01 DIFF pauses=1 valid=True
base G15A.02 DIFF pauses=1 valid=True
base G15A.03 DIFF pauses=1 valid=True
base G15A.04 DIFF pauses=1 valid=True
base G15A.05 DIFF pauses=1 valid=True
base G15A.06 DIFF pauses=1 valid=True
base G15A.07 SAME pauses=1 valid=True
base G15A.08 SAME pauses=1 valid=True
base G15A.09 SAME pauses=1 valid=True
base G15A.10 SAME pauses=1 valid=True
base G15A.11 SAME pauses=1 valid=True
base G15A.12 SAME pauses=1 valid=True
base G15A.13 SAME pauses=1 valid=True
base G15A.14 SAME pauses=1 valid=True
base G15B.01 DIFF pauses=1 valid=True
base G15B.02 DIFF pauses=1 valid=True
base G15B.03 DIFF pauses=1 valid=True
base G15B.04 DIFF pauses=1 valid=True
base G15B.05 DIFF pauses=1 valid=True
base G15B.06 DIFF pauses=1 valid=True
base G15B.07 DIFF pauses=1 valid=True
base G15B.08 DIFF pauses=1 valid=True
base G15B.09 DIFF pauses=1 valid=True
base G15B.10 DIFF pauses=1 valid=True
base G15B.11 DIFF pauses=1 valid=True
base G15B.12 DIFF pauses=1 valid=False
base G15B.13 DIFF pauses=1 valid=False
base G15B.14 DIFF pauses=2 valid=True
base G15B.15 DIFF pauses=1 valid=False
base G15B.16 DIFF pauses=1 valid=False
base G15B.17 SAME pauses=1 valid=True
base G15B.18 SAME pauses=1 valid=True
base G15B.19 SAME pauses=1 valid=True
base G15B.20 SAME pauses=1 valid=True
test result: total=34 SAME=34 DIFF=0 invalid=0
current G15A total=14 SAME=14 DIFF=0 invalid=0
current G15B total=20 SAME=20 DIFF=0 invalid=0
current G15A.01 SAME pauses=1 valid=True
current G15A.02 SAME pauses=1 valid=True
current G15A.03 SAME pauses=1 valid=True
current G15A.04 SAME pauses=1 valid=True
current G15A.05 SAME pauses=1 valid=True
current G15A.06 SAME pauses=1 valid=True
current G15A.07 SAME pauses=1 valid=True
current G15A.08 SAME pauses=1 valid=True
current G15A.09 SAME pauses=1 valid=True
current G15A.10 SAME pauses=1 valid=True
current G15A.11 SAME pauses=1 valid=True
current G15A.12 SAME pauses=1 valid=True
current G15A.13 SAME pauses=1 valid=True
current G15A.14 SAME pauses=1 valid=True
current G15B.01 SAME pauses=1 valid=True
current G15B.02 SAME pauses=1 valid=True
current G15B.03 SAME pauses=1 valid=True
current G15B.04 SAME pauses=1 valid=True
current G15B.05 SAME pauses=1 valid=True
current G15B.06 SAME pauses=1 valid=True
current G15B.07 SAME pauses=1 valid=True
current G15B.08 SAME pauses=1 valid=True
current G15B.09 SAME pauses=1 valid=True
current G15B.10 SAME pauses=1 valid=True
current G15B.11 SAME pauses=1 valid=True
current G15B.12 SAME pauses=2 valid=True
current G15B.13 SAME pauses=2 valid=True
current G15B.14 SAME pauses=2 valid=True
current G15B.15 SAME pauses=2 valid=True
current G15B.16 SAME pauses=2 valid=True
current G15B.17 SAME pauses=1 valid=True
current G15B.18 SAME pauses=1 valid=True
current G15B.19 SAME pauses=1 valid=True
current G15B.20 SAME pauses=1 valid=True
COMPARATOR CALIBRATION PASS: known Return/LHS failures red on the pre-repair export, every current-tree pair green; no Equal rows recorded yet.
```

Both cross-walks are complete as conservative source mappings: `.temp/brief-621/crosswalk-583.tsv` accounts for all **2,468** old records (2,443 raw plus 25 supplemental), and `crosswalk-548-unsettled.tsv` accounts for all **495** earlier Unsettled records. Exact AST expressions retain duplicate ordinals. Changed and coalesced branches map to explicit current replacement sets with the responsible brief; these mappings do not inherit a reading verdict. `refine-crosswalk-run.log` records:

```text
583 2468 {'Mapped': 2468}
548 495 {'Mapped': 495}
```

Calibration artifacts: `census-calibration.tsv`, `census-calibration-run.log`, `calibration-{current,base}.jsonl`, `calibration-extra-results.jsonl` and their run logs. All 195 legacy controls and all 21 extra controls are SAME. New/moved current edges are retained through the raw current inventory rather than borrowing old numeric IDs. Every ID in the settlement ledger refers to that current inventory, except the explicit screen-only members.

### 2. Population and exclusion rule


The final population has **2,186** unique members: 2,169 current graph edges plus 17 screen-only members requiring their own evidence. The tool leaves 1,188 current edges Unsettled (1,163 raw and 25 supplemental). The FrameCarries source population includes both tool and historical review assignments. The review-only TailCorrect population is the 412 historical Unsettled-to-TailCorrect assignments plus the 25 reviewed supplemental edges. Function screens expand to their current producer edges; the original screen records remain in `screen-crosswalk.tsv`. The exact counts, every pairwise overlap and the disjoint membership cells are pasted verbatim:

```text
{
  "raw_edges": 2657,
  "supplemental_edges": 25,
  "total_current_edges": 2682,
  "raw_tool_counts": {
    "Unsettled": 1188,
    "FrameCarries": 597,
    "TailCorrect": 897
  },
  "source_counts": {
    "tool_unsettled": 1188,
    "frame_carries": 752,
    "review_tail_correct": 437,
    "screen_frameless-direct-evaluate": 56,
    "screen_frameless-transforms": 209,
    "screen_optional-source-helpers": 97,
    "screen_effect-questions": 33,
    "strict_tail_without_indexed_caller": 283
  },
  "original_screen_counts": {
    "frameless-direct-evaluate": 56,
    "frameless-transforms": 143,
    "optional-source-helpers": 57,
    "effect-questions": 20
  },
  "screen_only_members": 17,
  "union": 2186,
  "strict_tool_tails": 897,
  "strict_tails_excluded": 513,
  "strict_tails_already_in_population": 384,
  "pairwise_overlaps": {
    "frame_carries & tool_unsettled": 152,
    "frame_carries & review_tail_correct": 5,
    "frame_carries & screen_frameless-direct-evaluate": 13,
    "frame_carries & screen_frameless-transforms": 30,
    "frame_carries & screen_optional-source-helpers": 24,
    "frame_carries & screen_effect-questions": 5,
    "frame_carries & strict_tail_without_indexed_caller": 0,
    "review_tail_correct & tool_unsettled": 433,
    "review_tail_correct & screen_frameless-direct-evaluate": 12,
    "review_tail_correct & screen_frameless-transforms": 83,
    "review_tail_correct & screen_optional-source-helpers": 23,
    "review_tail_correct & screen_effect-questions": 16,
    "review_tail_correct & strict_tail_without_indexed_caller": 0,
    "screen_frameless-direct-evaluate & tool_unsettled": 31,
    "screen_frameless-direct-evaluate & screen_frameless-transforms": 16,
    "screen_frameless-direct-evaluate & screen_optional-source-helpers": 4,
    "screen_frameless-direct-evaluate & strict_tail_without_indexed_caller": 0,
    "screen_frameless-transforms & tool_unsettled": 137,
    "screen_frameless-transforms & screen_optional-source-helpers": 14,
    "screen_frameless-transforms & strict_tail_without_indexed_caller": 0,
    "screen_optional-source-helpers & tool_unsettled": 65,
    "screen_optional-source-helpers & strict_tail_without_indexed_caller": 0,
    "screen_effect-questions & tool_unsettled": 30,
    "screen_effect-questions & screen_frameless-direct-evaluate": 0,
    "screen_effect-questions & screen_frameless-transforms": 14,
    "screen_effect-questions & screen_optional-source-helpers": 5,
    "screen_effect-questions & strict_tail_without_indexed_caller": 0,
    "strict_tail_without_indexed_caller & tool_unsettled": 0
  },
  "disjoint_membership_counts": {
    "tool_unsettled+frame_carries+screen_frameless-direct-evaluate": 6,
    "tool_unsettled+review_tail_correct+screen_frameless-transforms": 67,
    "tool_unsettled+frame_carries": 107,
    "tool_unsettled": 542,
    "frame_carries": 583,
    "tool_unsettled+review_tail_correct": 319,
    "tool_unsettled+review_tail_correct+screen_frameless-direct-evaluate": 9,
    "tool_unsettled+frame_carries+screen_frameless-transforms": 18,
    "tool_unsettled+frame_carries+screen_frameless-direct-evaluate+screen_frameless-transforms": 6,
    "screen_frameless-direct-evaluate": 22,
    "strict_tail_without_indexed_caller": 283,
    "screen_frameless-transforms": 69,
    "tool_unsettled+screen_frameless-direct-evaluate": 3,
    "tool_unsettled+frame_carries+screen_frameless-transforms+screen_effect-questions": 3,
    "tool_unsettled+review_tail_correct+screen_optional-source-helpers": 16,
    "screen_optional-source-helpers": 19,
    "tool_unsettled+frame_carries+screen_optional-source-helpers": 7,
    "tool_unsettled+screen_optional-source-helpers": 25,
    "tool_unsettled+screen_frameless-transforms+screen_optional-source-helpers": 5,
    "screen_frameless-direct-evaluate+screen_frameless-transforms": 3,
    "frame_carries+review_tail_correct": 4,
    "tool_unsettled+screen_frameless-transforms": 16,
    "tool_unsettled+review_tail_correct+screen_frameless-transforms+screen_effect-questions": 10,
    "frame_carries+screen_optional-source-helpers": 12,
    "tool_unsettled+frame_carries+review_tail_correct": 1,
    "tool_unsettled+screen_frameless-direct-evaluate+screen_frameless-transforms": 3,
    "tool_unsettled+review_tail_correct+screen_frameless-direct-evaluate+screen_frameless-transforms+screen_optional-source-helpers": 3,
    "tool_unsettled+review_tail_correct+screen_frameless-transforms+screen_optional-source-helpers+screen_effect-questions": 1,
    "tool_unsettled+screen_effect-questions": 9,
    "tool_unsettled+review_tail_correct+screen_effect-questions": 4,
    "tool_unsettled+review_tail_correct+screen_frameless-transforms+screen_optional-source-helpers": 2,
    "tool_unsettled+review_tail_correct+screen_optional-source-helpers+screen_effect-questions": 1,
    "tool_unsettled+frame_carries+screen_optional-source-helpers+screen_effect-questions": 1,
    "frame_carries+screen_optional-source-helpers+screen_effect-questions": 1,
    "tool_unsettled+screen_optional-source-helpers+screen_effect-questions": 1,
    "tool_unsettled+frame_carries+screen_frameless-transforms+screen_optional-source-helpers": 2,
    "tool_unsettled+frame_carries+screen_frameless-direct-evaluate+screen_frameless-transforms+screen_optional-source-helpers": 1,
    "screen_effect-questions": 2
  }
}
```

What is excluded and why: **513** tool-assigned strict tails return the producer Result whole with `try_depth=0`; no local success transformation is certified by a frame heuristic. Their obligation transfers to every indexed caller. `tail-exclusions.tsv` records immediate caller rows and the population endpoints reached through chains of strict tails. A further **283** strict tails without an indexed incoming caller stay in the population rather than disappearing at a graph boundary. FrameCarries, review-assigned TailCorrect, screen-only records and dynamic/ambiguous calls receive no safe default. No source-review verdict is used as Equal.

One population-join error was corrected before equality settlement: the direct-evaluate screen also contains `evaluate_atom_fast_path`. Matching its actual target removes one spurious screen-only member, so 2,187 became 2,186. `population-correction.tsv` records that single mapping; all pre-correction population/screen/ledger files are preserved as `before-screen-target-correction-*`. The three Lost rows already recorded were preserved. No historical row disappeared.

### 3. Verdicts and executed evidence


`settled.tsv` has exactly one row per final population member, including every missing verdict. Overall totals:

| Verdict | Count |
| --- | ---: |

| Equal | 4 |

| Lost | 3 |

| Unreachable | 8 |

| Boundary | 0 |

| Retired | 0 |

| Duplicate | 0 |

| MISSING | 2171 |


The MISSING count is a failed deliverable, not a seventh settlement verdict. The three historical retired wrappers in calibration are not current population members and are not added to these totals.

Per source population below counts distinct **current population members**, not historical record multiplicity. Populations overlap; totals are not additive.

```text
source	members	Equal	Lost	Unreachable	Boundary	MISSING
tool_unsettled	1188	1	2	8	0	1177
frame_carries	752	4	2	0	0	746
screen_frameless-direct-evaluate	56	1	1	0	0	54
review_tail_correct	437	3	1	8	0	425
screen_frameless-transforms	209	1	1	5	0	202
strict_tail_without_indexed_caller	283	0	0	0	0	283
screen_effect-questions	33	0	1	0	0	32
screen_optional-source-helpers	97	0	3	0	0	94
583-all-mapped-records	1967	4	3	8	0	1952
548-Unsettled-mapped-records	492	4	1	8	0	479
```

#### Every Equal row: policy, contexts and observable tail


The LHS suite runs on the unmodified current build and on a copied tree with exact AST-span guards around 2,604 runtime/stdlib/host edges. The copy logs the active producer stack only at real input creation. All 49 complete observations agree byte-for-byte between these builds; the guards are evidence of producer attribution, not a substitute comparator. `instrumented-edges.tsv`, `lhs-pause-attribution.tsv`, `lhs-attribution-pauses.log` and the full JSON observations preserve the mapping. The source copy has no role in ordinary source integrity or performance claims.

| Row / current site | Tail policy | Required contexts executed | How the tail is observed |
| --- | --- | --- | --- |

| P0360 / `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:285` | DefinitionLhsPreparation | value, error_before_tail, same_type_error_finish, prefix_and_tail, guest_heads, held_and_splice | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. |

| P0362 / `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:341` | DefinitionLhsPreparation | value, error_before_tail, same_type_error_finish, prefix_and_tail, guest_heads, held_and_splice | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. |

| P0364 / `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:415` | DefinitionLhsPreparation | value, error_before_tail, same_type_error_finish, prefix_and_tail, guest_heads, held_and_splice | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. |

| P0368 / `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:543` | MatchingSymbolCycleRecovery | value, error_before_tail, same_type_error_finish, prefix_and_tail | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. |


These are 16 targeted context/variant pairs per awaited pattern/head/argument edge, and all 49 pairs for the shared recovery edge. They include middle-of-LHS suffix effects, two loop iterations, distinct operands, an earlier pause, nested Block, matching and other-symbol cycles, Throw, scalar and Return values, literal internal heads, Hold/Unevaluated/Sequence/Nothing, raw trace equality, stored DownValues/SubValues snapshots, and first/applicable-second-pause cancellation/reset. There is no profiler tail in these LHS driver/recovery functions; enabled raw tracing is an extra observation. Producer-specific trace/phase obligations elsewhere remain uncompleted. A trial with a nested ordinary head reached a different producer; it was replaced by the root LHS-head source before settlement, rather than weakening the attribution assertion.

```text
test result: total=49 SAME=49 DIFF=0 invalid=0
```

```text
SETTLEMENT LHS: {'MISSING': 2179, 'Equal': 4, 'Lost': 3}
test result: 49 valid contextual pairs SAME, zero DIFF; exact-edge pause attribution matches unmodified observations; cancellation/reset at first and applicable second pause passed
```

#### Every Lost witness in full


**P0755 — OptionalStage**. Producer `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:104`; crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:104; evaluate(expr).map(Some) loses the Some before confirmation consumer decodes info.

```rum
Enclose[Confirm[$Failed, (InputString["i"]; System`InternalContinuationSome[5])]]
```

Answers: `1`; asserted pauses: 1. Guest InternalContinuationSome[5] becomes ordinary 5 (Cases: list element; Confirm: Information property) because the skipped Some does not protect it from legacy optional decoding. Owner: S10; .temp/brief-606/draft-S10.md:150,169. Complete recorded comparison, verbatim:

```json
{"id":"lost.confirm.info.2","pauses":["i"],"queued":"Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\", typed: Other { head: \"Failure\", text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }","queued_pauses":[],"resumed":"Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\", typed: Other { head: \"Failure\", text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }","same":false,"valid":true}
```

**P0883 — OptionalStage**. Producer `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:441`; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:448; resume_frames.rs:306-310 returns raw Expr rather than optional collection item.

```rum
Cases[{1},_ :> (InputString["w"];System`InternalContinuationSome[5])]
```

Answers: `1`; asserted pauses: 1. Guest InternalContinuationSome[5] becomes ordinary 5 (Cases: list element; Confirm: Information property) because the skipped Some does not protect it from legacy optional decoding. Owner: S19; .temp/brief-606/draft-S19.md:110,133,165-167. Complete recorded comparison, verbatim:

```json
{"id":"lost.cases.some","pauses":["w"],"queued":"Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"List[InternalContinuationSome[5]]\", typed: List([Other { head: \"InternalContinuationSome\", text: \"InternalContinuationSome[5]\" }]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[InternalContinuationSome[5]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }","queued_pauses":[],"resumed":"Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"List[5]\", typed: List([Integer(5)]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[5]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }","same":false,"valid":true}
```

**P1489 — FunctionResult**. Producer `crates/rum-runtime/src/runtime/scoping/callable.rs:63`; crates/rum-runtime/src/runtime/scoping/callable.rs:77; TryApplyFunctionSuspensionFrame resume :163-164 absorbs Return but omits the ordinary Some.

```rum
Function[(InputString["w"];System`InternalContinuationSome[5])][]
```

Answers: `1`; asserted pauses: 1. Guest InternalContinuationSome[5] becomes ordinary 5 (Cases: list element; Confirm: Information property) because the skipped Some does not protect it from legacy optional decoding. Owner: S13; .temp/brief-606/draft-S13.md:96,106,136-138. Complete recorded comparison, verbatim:

```json
{"id":"lost.function.some","pauses":["w"],"queued":"Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"InternalContinuationSome[5]\", typed: Other { head: \"InternalContinuationSome\", text: \"InternalContinuationSome[5]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"InternalContinuationSome[5]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }","queued_pauses":[],"resumed":"Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"5\", typed: Integer(5) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"5\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }","same":false,"valid":true}
```

```text
test result: total=3 SAME=0 DIFF=3 invalid=0
```

#### Every Unreachable proof in full


These are constrained producer call sites whose closed callee chains cannot evaluate guest expressions. They remain a separate counted class for director review. The held Function clone attempts intentionally have **zero total pauses** and are not Equal; the delayed-rule/consumer attempts have one total pause elsewhere, with **zero pauses in these designated producers**. The source proof, rather than absence in the graph or a zero counter alone, is the justification. Source paths in the proofs are relative to `crates/rum-runtime/src/runtime/`.

| Strongest attempt ID | Full guest source | Answers | Observed total pauses |
| --- | --- | --- | ---: |

| unreachable621.clone.unarmed | `Union[{},SameTest->Function[{x,y},InputString["clone-try"]]]` | [] | 0 |

| unreachable621.clone.armed | `MemoryConstrained[Union[{},SameTest->Function[{x,y},InputString["clone-try"]]],1000000000]` | [] | 0 |

| unreachable621.rule.unarmed | `Union[{1,2},SameTest:>(InputString["rule-try"];SameQ)]` | ['1'] | 1 |

| unreachable621.rule.armed | `MemoryConstrained[Union[{1,2},SameTest:>(InputString["rule-try"];SameQ)],1000000000]` | ['1'] | 1 |

| unreachable621.driver.non_delayed | `MemoryConstrained[Union[{1,2},SameTest->Function[{x,y},(InputString["consumer-try"];False)]],1000000000]` | ['1'] | 1 |


All resumed/queued observation strings of these five attempts are equal, including the two non-pausing attempts. The two no-pause cases are rejected by the pausing-equality predicate, which is why the diagnostic driver reports `invalid=2`; they are used solely for Unreachable evidence. Their native/copy observations are identical. Full raw data: `unreachable-attempt-results.jsonl`, `unreachable-attribution-results.jsonl`, `unreachable-attribution-pauses.log`.

**P0003 / `crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:198`**. into_rule calls finish_armed only with an already successful Expr when formerly unarmed state sees an active limit (bridge.rs:183-211). finish_armed invokes scoped with a concrete closure whose adopt_outcome input is Ok(value), then builds Rule (rule.rs:186-199); it has no guest action. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

**P0013 / `crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:43`**. This edge is in the non-RuleDelayed arm of options/driver.rs:19-46; the outer match excludes precisely the RuleDelayed/arity-two case that could evaluate a guest RHS. evaluate_runtime_option_expr (options/core.rs:225-238) therefore selects clone_result. clone_result -> clone_owned only clones the existing Expr and charges it (accounting_mode/rule.rs:14-43). The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.driver.non_delayed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

**P0023 / `crates/rum-runtime/src/runtime/accounting_mode/rule.rs:20`**. clone_result selects scoped<AccountingShadow> at rule.rs:20 with the concrete clone_owned closure; it cannot receive an arbitrary host Fn. clone_owned (rule.rs:26-43) clones source and enforces memory without evaluation. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

**P0024 / `crates/rum-runtime/src/runtime/accounting_mode/rule.rs:22`**. clone_result selects scoped<Unarmed> at rule.rs:22 with the concrete clone_owned closure; it cannot receive an arbitrary host Fn. clone_owned (rule.rs:26-43) clones source and enforces memory without evaluation. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

**P0029 / `crates/rum-runtime/src/runtime/accounting_mode/rule.rs:112`**. evaluate_in calls value.into_rule only in the Ok CallbackValue arm after cx.evaluate has already completed (rule.rs:111-115). The producer here is into_rule, not that earlier evaluator. into_rule -> finish_rule or finish_armed builds the Rule from an existing value. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

**P0040 / `crates/rum-runtime/src/runtime/accounting_mode/rule.rs:178`**. resume_in calls value.into_rule only in adopt_outcome Ok(value), at rule.rs:176-182. That producer only converts a successful owned value; the exceptional arm is separate and is not included in this proof. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

**P0041 / `crates/rum-runtime/src/runtime/accounting_mode/rule.rs:191`**. finish_armed calls scoped with a literal closure that supplies Ok(value) to adopt_outcome (rule.rs:186-199). scoped (retained.rs:71-104) invokes only that closure and retires its receipt; its generic ability to invoke other closures is irrelevant at this call site. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

**P0042 / `crates/rum-runtime/src/runtime/accounting_mode/rule.rs:193`**. finish_armed calls into_rule only on the Ok CallbackValue arm (rule.rs:191-196). AccountingShadow::ARMED=true prevents reentering finish_armed through the unarmed check; the remaining Rule constructor/charge path cannot evaluate guest code. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it. Strongest attempts: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed. Observed designated producer pauses: **0**; total pauses: 0 clone; 1 delayed-rule/consumer.

```text
test result: total=5 SAME=3 DIFF=2 invalid=2
```

```text
SETTLEMENT: 15 / 2186 (4 Equal, 3 Lost, 8 Unreachable); zero-pause attempts are not Equal
```

#### Remaining probe results and stop-loss


The 331-pair panel is 195 legacy controls plus the 136 unchanged brief-618 cases. It leaves the same 22 known differences after G15-A/B: the optional-info, Function/composition/association/Failsafe/Fold/Nest and Cases marker contexts. Six unexpected extra pauses remain observable divergences and are not Equal. The broad panel’s matching values do not settle the other rows. Exact producer attribution in the copied build observes 756 raw edges, 578 members of the then-current population; none was blanket-promoted.

`calibration-current.log`:

```text
test result: total=34 SAME=34 DIFF=0 invalid=0
```

`calibration-base.log`:

```text
test result: total=34 SAME=12 DIFF=22 invalid=4
```

`calibration-extra-run.log`:

```text
test result: total=21 SAME=21 DIFF=0 invalid=0
```

`legacy-and-618-run.log`:

```text
test result: total=331 SAME=309 DIFF=22 invalid=6
```

`trace-calibration-run.log`:

```text
test result: total=34 SAME=34 DIFF=0 invalid=0
```

`bridge-attempt-run.log`:

```text
test result: total=30 SAME=30 DIFF=0 invalid=0
```

`prefix-attempt-run.log`:

```text
test result: total=4 SAME=4 DIFF=0 invalid=0
```

The independent public host fixture uses `HostSession` with scratch-only `Retain621` and `Retention621` registrations. It creates a nonempty retained prefix using the public retention API, freezes it on input, imports it through `RetentionContinuation`, and observes the final live receipt/byte counts, effects, trace and fresh-request behavior. It reaches P0002 in both modes. Its four Middle/Twice controls do **not** supply all guest-head, held/splice, error and route contexts. The fixture is preserved at `prefix-support.rs`, `probe-prefix/` and `probe-prefix-attribution/` for the director’s next session. It is not installed in production.

```text
P0001: 0 prefix probes paused in producer; modes=
P0002: 4 prefix probes paused in producer; modes=armed,unarmed
Session A: 30 SAME contextual option probes; settled rows 15 -> 15 (0). P0001 only armed, P0002 absent.
Session B: standalone public host retained-prefix builtin compiled; 4 SAME pairs, producer P0002 reached in both modes; settled rows 15 -> 15 (0). Guest-head, held/splice, error, and consumer-route context closure is incomplete; no Equal assigned.
STOP-LOSS: two consecutive settlement work sessions settled zero rows. Stop further probes/settlement and report the incomplete deliverable.
```

Intermediate issues were corrected before recording evidence: the old inventory missed typed macros; initial host argument types needed `&str`; flattening host outputs does not preserve cross-channel prefixes, so prefix checks were changed to the contract’s separate messages/prints/trace channels; historical pre-repair destructuring used only one answer and was corrected to the actual two-pause source; the screen target join was corrected; the LHS head middle fixture reached ordinary evaluation and was corrected; and the standalone native handler signature was corrected to `&SharedArgs`. An exploratory one-argument OptionValue attempt emitted an arity message and was replaced by the valid Union option source; it supplied no proof or Equal verdict. No content-filter interruption occurred.

### 4. Lost mechanism and owners


The three settled Lost edges are one mechanism: an eager Expr-to-Option success constructor is absent after resumption, leaving a literal guest head exposed to the legacy optional decoder. They are **existing owned work**, not three newly invented repair groups. Root locations are `scoping/callable.rs:77` with its raw completion frame at `:163-164`, `kernel_data/control_data/confirmation.rs:104`, and `kernel_data/rewrite_data/cases/collect.rs:448` with raw completion at `collect/resume_frames.rs:306-310`. No claim is made that these are all Lost edges in the uncompleted population.

| Existing owner | Draft coverage already present | Witness its rewritten brief must include |
| --- | --- | --- |

| S10 | `.temp/brief-606/draft-S10.md:150,169` — T0028 optional-info producer | `lost.confirm.info.2`, plus the seven confirmation routes in the saved 618 panel |

| S13 | `.temp/brief-606/draft-S13.md:96,106,136-138` — F0528/M0563 | `lost.function.some`; retain None/empty Some, composition/association/Failsafe and Fold/Nest contexts |

| S19 | `.temp/brief-606/draft-S19.md:110,133,165-167` — F0358/M0374 | `lost.cases.some`; retain None/empty Some collector contexts |


The existing repair direction remains typed Expr->Option completion at the owning producer/consumer seam, preserving Function Return absorption and Cases/info behavior. No new G15-C*n* repair group was established before the stop. Whether S10 may proceed ahead of any **undiscovered** group is unanswered. The standing order remains G15-C repairs and manifest, then CHANGELOG, then S10; this incomplete report does not clear that order.

### 5. Manifest design


**Not completed; step 5 was not reached.** The accepted brief-618 observation model/context closure remains the starting point and is not restated or claimed as a newly implemented design. The saved population, exact producer span guards, four Equal policy assignments, three owned Lost edges and eight separate proofs are raw inputs, not an obligation-keyed manifest. The remaining 2,171 rows lack their execution settlements and most lack a justified tail policy. No exhaustive enum, executed-tuple equality gate, immutable-baseline promotion mechanism, full probe port, mutation-control result or S9 run-cost measurement is claimed.

### 6. Drafted follow-up briefs


**None drafted; step 6 was not reached.** There is no manifest implementation draft and no new repair-group brief to dispatch. The three existing-owner witness additions are listed in section 4 and are preserved in `settled.tsv`. Do not substitute them for the requested completed drafts. No CHANGELOG or S10 source work was started.

### 7. Open questions and recommendations


- **How to resume the failed census?** Recommendation: resume from `PROGRESS.md` and the 15 settlements, rather than restarting or treating the broad passing corpus as coverage. Complete the new retained-prefix fixture’s required matrix and the missing empty-prefix Unarmed consumer route, then continue by source obligation. Keep the two zero-settlement sessions visible in the record.
- **Accept the eight Unreachable proofs?** Recommendation: review each constrained call-site proof in full above; none is inferred merely from an unobserved counter or an absent installed handler.
- **Can the manifest, CHANGELOG or S10 advance?** Recommendation: retain the director’s stated sequence; this report has not finished the evidence required to authorize those follow-ups.

No production/runtime/workspace test suite, no full suspension oracle in either feature build, no semantic or manifest mutation controls, no performance/instruction/stack work and no whole-oracle run-cost gate ran. Cargo builds and the rum-vs-rum probe executions named above did run, all offline and process-group bounded. The report is intentionally a failed exhaustive deliverable with a recoverable scene, not a safe verdict attached to missing work.

Final scope verification, verbatim:

```text
SOURCE INTEGRITY PASS: 2087 other baseline files unchanged; exactly one permitted report append; no added original-tree files; no git command used.
FINAL INCOMPLETE: 15/2186 settled; 2171 missing; stopped after two zero-settlement sessions; steps 5-6 not reached.
```
