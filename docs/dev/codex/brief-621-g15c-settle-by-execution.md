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

## Director continuation 3 (2026-10-19) — observation is measured, and every open row has an owner

Continuation 2 is accepted for what it established:

- **Coverage.** Policies are assigned for all 2 186 rows, and the corpus
  covers 80 309 probes and 2.79 M cells.
- **Triage.**
  - The internal-encoding differences are classified by script and owned by
    G13's transport steps.
  - The six extra pauses belong to S13 (P1489).
  - The non-encoding differences reduce to the known owned mechanisms plus
    one new group.
- **G15-C1, the new group.** `replace/all.rs:59,127` propagates a suspended
  application before its atom test and `evaluate(replaced)` tail, which shows
  in the ordered trace at an equal value (`min.ReplaceAll.atom`,
  `min.ReplaceAllUnheld.atom`). It is accepted as a new repair group before
  S10; S24 keeps its regression.
- **The rest.** The 10 `Boundary` rows, the manifest design (the key, the
  exhaustive enum, three checked collections, receipts, the ledger and the
  negative-control model), and the C1 and manifest drafts.

The 140 `Unreachable` proofs are accepted in kind. List them by class
(module, callee chain shape and count) so that I can review them by class.

**The 2 027 parked rows are my problem to rule on, and this is the ruling.**

**1. Observation is measured, not stated.**

- Today a row could only become `Equal` through a source-specific *observer
  statement*: which state, phase or effect goes wrong if the tail is
  omitted. That is reading again.
- From now on a row is `Equal` when both hold:
  - its required tuples are SAME on the real build;
  - at least one of them turns red under that row's **tail-skip mutant**,
    built in a copied tree.
- The observer statement stays, but as the mutant's description. The mutant
  decides.
- The four semantic mutants your design already requires (Some, G15-A's
  Return finish, G15-B's recovery, G15-C1's evaluation finish) are instances
  of this rule.

**2. Every parked row gets an owner.**

- Map each parked row to the G13 step whose partition (`.temp/brief-606/`)
  owns its frame or site, using the method that produced the transport
  owner tables.
- **Owned rows.** A G13 step converts that code anyway, and since S5 its
  gate is fixtures red on the base plus mutants. Its brief will list its
  owned rows, and its gate settles them by the rule above.
- **G15-C2 is every parked row that no G13 step owns.** Give its count and
  its composition by policy, module and source population. These rows have
  no later owner, so they are settled before S10.

**3. Ledger entries for owned rows.**

- Add `OwnedObligation { row, owner }` beside the known-loss entries.
- The full gate is green with such an entry present only while its owner
  step is open.
- The owner's promotion requires each of its rows to become a settled
  tuple set, with its mutant caught.
- An entry whose owner is done is red.
- C2's rows enter as `OwnedObligation { owner: G15-C2 }`.

**4. Design the tail-skip mutants.**

- **Shapes, per policy.** Give the generic shape of a tail-skip mutant for
  each policy:
  - for a frame: its resume passes the outcome through without the tail;
  - for a frameless tail: the ordinary path skips the tail, so the mutant's
    queued run equals what the real resumed run would produce;
  - say where a policy has no generic shape.
- **One build.** State whether the mutants can be one copied-tree build
  with per-row switches selected at run time, as the attribution build's
  span guards are.
- **Prototype.** Build that schema for C2's rows and for one G13 step's
  owned rows (S10's, since it is next).
- **Settle C2.** Settle as many of C2's rows as the prototype reaches, under
  the same batch stop-loss.

**5. Revise the drafts.**

- The manifest implementation brief carries the ledger change and the
  mutant criterion.
- Add a G15-C2 settlement brief for whatever C2 still holds.
- Add a per-step owned-row table for each G13 step, with S10's first.
- Propose the sequence: C1, C2, the manifest, then CHANGELOG, then S10, or
  argue for a different order.

**Report.** Add a section *Continuation 3* at the end, and rewrite the
opening summary to describe the final state. Checkpoint per batch, as
before.

---

## Execution report — brief-621 (final state after continuation 3)

**198 of 2,186 rows are semantically settled: 43 Equal, 5 Lost, 140 Unreachable
and 10 Boundary. The other 1,988 are explicit OwnedObligations, with 1,024
assigned to G15-C2 and 964 to future G13 steps. No parked row lacks an owner.**

Continuation 3 adds 39 measured Equal rows (29 C2, 10 S10), each with complete
real SAME tuples and a caught source-row tail-skip mutant. It supplies the
per-class proof review, per-step owner tables, four independent semantic
controls, owner-lifetime ledger model and revised C1/C2/manifest drafts. C2
remains Open; the production manifest gate, CHANGELOG and S10 cannot advance.
See **Continuation 3** at the end and `.temp/brief-621/PROGRESS.md`.

All experiments and drafts stay under `.temp/brief-621/`; no git/network or
production source change occurred. The other 2,087 original files are byte
unchanged. This brief's summary rewrite and report append are the only
original-file edits. The sections immediately below preserve the initial
15-settlement report as historical evidence; the later continuation sections
record subsequent accepted work and corrections.

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

---

## Continuation 1 — continued under director continuation 2

**INCOMPLETE census: 159/2,186 settled — 4 Equal, 5 Lost, 140 Unreachable and 10 Boundary; 2,027 explicitly parked for the director. Policies cover all 2,186 rows. The saved matrix contains 2,792,712 raw cells across 1,309 population rows, not 2,186 execution settlements. The source-obligation manifest design, its exactness model, a new G15-C1 repair draft, a manifest implementation draft and existing-owner additions are now written. They do not clear CHANGELOG or S10.**

This is the current opening summary for the resumed report. The earlier 15-row report above is retained verbatim because the hard rule permits only an append to this brief. Its old stop-loss and unfinished-policy description are superseded by the director continuations and the state here. Accepted census/cross-walk/calibration and completed continuation-1 probe batches were not regenerated. No git command, network request, new unsafe or original source edit ran; all new scratch/build work is under `.temp/brief-621/`.

The continuation permits a resistant row to be parked with evidence and a recommendation. `settled.tsv` still uses only the closed verdict vocabulary for genuine settlements. The parked rows have an explicit `ParkedForDirectorRuling` disposition, policy, attempted routes, raw coverage samples and missing evidence; their verdict is deliberately absent. This remains a failed exhaustive-settlement deliverable, not a new passing verdict named Parked. There was no unexplained calibration failure or content-filter interruption in this continuation, and no claim that the two-zero-batch stop-loss fired. Direct CLI/REPL/Jupyter adapter-route fixtures, independent semantic source mutations and the expanded whole-oracle run-cost gate remain unrun.

### 1. Re-census, calibration and cross-walk (accepted foundation)

The accepted re-census is 2,657 AST edges plus 25 supplemental edges, total 2,682. Both historical cross-walks account for every row: `crosswalk-583.tsv` (2,468/2,468) and `crosswalk-548-unsettled.tsv` (495/495). The current calibrated pass includes typed macros/frames and conservative explicit InputRequired propagation; reading overlays were not inherited as semantic verdicts. The following are preserved real outputs from the accepted foundation, not new runs in continuation 2.

```text
583 2468 {'Mapped': 2468}
548 495 {'Mapped': 495}
```

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

The historical Lost retirement controls are shown individually above; their retirement is not an Equal verdict. Exact current successors for G15-A P1316 and G15-B P0307 are P1477 and P0368. Current source files continue to match the entry baseline, so this continuation reused the accepted calibration rather than re-running it.

### 2. Population, policies and exclusions

```json
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

The exclusion is only a strict whole-Result return with an indexed caller: 513 such edges transfer their obligation to that caller. All 283 strict tails without an indexed caller stay in the population. Screen-only members and review-assigned tails remain in even when their local shape is strict. Counts and overlaps above are the accepted step-2 population, not a union of passing descriptor names.

A1/A2 placed all 2,186 rows; the 33 initially unplaced rows were explicitly reviewed in `continuation1/policy-review.json`, with final `unplaced-policies.tsv` empty. The exhaustive 25-arm enum has no default arm (`continuation1/tail_policy.rs`, `policy-spec.json`). `tail-policies-final.tsv` preserves the prior assignment and refines the 140 concrete proofs to NonSuspending and the 10 documented boundaries to their disposition policies; other rows retain their source-derived obligations. Per-policy memberships overlap.

| policy | rows | contexts | variants |
| --- | --- | --- | --- |
| DefinitionLhsPreparation | 16 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;same_type_error_finish;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| MatchingSymbolCycleRecovery | 1 | value;error_before_tail;prefix_and_tail;same_type_error_finish;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| OptionalStage | 413 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| FunctionResult | 14 | value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;phase_observation;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| ReturnBoundary | 82 | value;return_boundary;error_before_tail;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| ErrorTransform | 52 | value;return_boundary;error_before_tail;prefix_and_tail;same_type_error_finish | Middle;Twice;Error;AlreadyResumed;Nested |
| PrefixLoop | 175 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| DefinitionStore | 56 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| PrepareArgumentsRestore | 37 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| OptionReader | 55 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| ScopeCleanup | 82 | value;return_boundary;error_before_tail;held_and_splice;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| CaptureOutput | 38 | value;return_boundary;error_before_tail;guest_heads;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| PhaseOwnership | 104 | value;return_boundary;error_before_tail;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| MatchDecision | 375 | value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| RebuildTraversal | 790 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| ShapeDecode | 493 | value;guest_heads;held_and_splice;prefix_and_tail;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| ScalarFinish | 46 | value;return_boundary;guest_heads;held_and_splice;prefix_and_tail | Middle;Twice;Error;AlreadyResumed;Nested |
| ConditionalBranch | 58 | value;return_boundary;error_before_tail;guest_heads;prefix_and_tail | Middle;Twice;Error;AlreadyResumed;Nested |
| NativeCallbackBridge | 36 | value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;phase_observation;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| GuestLoading | 15 | value;return_boundary;error_before_tail;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| HostRequestDriver | 97 | value;return_boundary;error_before_tail;prefix_and_tail;phase_observation | Middle;Twice;Error;AlreadyResumed;Nested |
| ForwardResult | 358 | value;return_boundary;error_before_tail;guest_heads;held_and_splice;prefix_and_tail;same_type_error_finish;phase_observation;consumer_routes | Middle;Twice;Error;AlreadyResumed;Nested |
| NonSuspending | 140 |  |  |
| HostBatchBoundary | 2 |  |  |
| RetiredBoundary | 8 |  |  |

Every remaining row has a policy. A source-family classification is not a semantic settlement. In particular ForwardResult roots retain explicit route review, and a source-stack guard seen at the first pause does not prove the disappeared Rust stack persists at the second pause. The guard normally unwinds; source/frame phase provenance, not two identical active stack hits, must establish the resumed route.

### 3. Coverage, verdicts and parked rows

```json
{
  "total_cells": 2792712,
  "states": {
    "SAME": 2568051,
    "DIFF": 23220,
    "invalid": 201441
  },
  "attributed_rows": 1309,
  "distinct_probes": 80309,
  "contexts": {
    "value": {
      "SAME": 115545,
      "DIFF": 468,
      "invalid": 698
    },
    "phase_observation": {
      "SAME": 97082,
      "DIFF": 758,
      "invalid": 1848
    },
    "same_type_error_finish": {
      "SAME": 260
    },
    "prefix_and_tail": {
      "SAME": 112869,
      "invalid": 1136
    },
    "error_before_tail": {
      "SAME": 208222,
      "invalid": 80401
    },
    "return_boundary": {
      "SAME": 321787,
      "invalid": 87864
    },
    "guest_heads": {
      "SAME": 795444,
      "invalid": 18700,
      "DIFF": 21994
    },
    "held_and_splice": {
      "SAME": 812514,
      "invalid": 10645
    },
    "consumer_routes": {
      "SAME": 104328,
      "invalid": 149
    }
  },
  "raw_complete_rows": 471,
  "verdicts": {
    "Pending": 2027,
    "Unreachable": 140,
    "Boundary": 10,
    "Equal": 4,
    "Lost": 5
  }
}
```

Raw matrix keys include descriptor/probe exposures, so cells count repeated observations of different programs as well as new context/variant coverage. They are not a count of independently certified obligations. The matrix retains invalid plans and cross-build differences; neither is Equal. The final table below is per source population; memberships overlap and must not be summed across populations.

| population | members | Equal | Lost | Unreachable | Boundary | ParkedForDirectorRuling |
| --- | --- | --- | --- | --- | --- | --- |
| overall | 2186 | 4 | 5 | 140 | 10 | 2027 |
| frame_carries | 752 | 4 | 4 | 19 | 0 | 725 |
| review_tail_correct | 437 | 3 | 1 | 36 | 0 | 397 |
| screen_effect-questions | 33 | 0 | 1 | 1 | 0 | 31 |
| screen_frameless-direct-evaluate | 56 | 1 | 1 | 0 | 3 | 51 |
| screen_frameless-transforms | 209 | 1 | 1 | 13 | 1 | 193 |
| screen_optional-source-helpers | 97 | 0 | 3 | 5 | 0 | 89 |
| strict_tail_without_indexed_caller | 283 | 0 | 0 | 26 | 0 | 257 |
| tool_unsettled | 1188 | 1 | 4 | 98 | 10 | 1075 |

`settled.tsv` contains exactly 2,186 distinct population IDs. `continuation2/parked.tsv` holds all 2,027 unclosed rows, their attempts and the missing observer/context/route evidence. Of them, 1290 have raw producer exposure and 737 have no designated producer pause. No broad SAME panel or absent counter was used to promote those rows.

The completed held-position control suppressed evaluation and had zero pauses; it was invalid. The new forced-evaluation-within-Hold batch has 2,993 valid SAME pairs and one invalid plan in each build. Last-pause Return/Throw controls preserve earlier planned pauses where the source admits that route. A separate 200-case optional/Flat/Orderless/sequence/default family, 20 public HostSession call/timeout controls and the non-encoding escalation controls are recorded below. The strict source review of 10 attractive equality candidates did not close one: `reviewed-equal-gaps.tsv` records missing producer-phase/recipe evidence. Some intermediate marker edits do not reach the actual producer result, and raw second-pause stack multiplicity is not a continuation proof.

Nine initial armed string-option plans used a string key for the standard IgnoreCase option and left unexpected requests in both queued and resumed execution. They are invalid plans, not nine new Lost witnesses. The nine corrected documented-symbol controls are all SAME with exactly one planned pause. The original invalid observations are preserved and excluded from equality evidence.

#### Equal rows: source policy and tail observer

| id | policy | contexts | observer | probes |
| --- | --- | --- | --- | --- |
| P0360 | DefinitionLhsPreparation | value;error_before_tail;same_type_error_finish;prefix_and_tail;guest_heads;held_and_splice | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. | lhs621.pattern.after_prior;lhs621.pattern.cycle;lhs621.pattern.empty_some;lhs621.pattern.hold;lhs621.pattern.loop;lhs621.pattern.middle;lhs621.pattern.nested;lhs621.pattern.none;lhs621.pattern.nothing;lhs621.pattern.other_cycle;lhs621.pattern.return;lhs621.pattern.scalar;lhs621.pattern.sequence;lhs621.pattern.some;lhs621.pattern.throw;lhs621.pattern.unevaluated |
| P0362 | DefinitionLhsPreparation | value;error_before_tail;same_type_error_finish;prefix_and_tail;guest_heads;held_and_splice | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. | lhs621.head.after_prior;lhs621.head.cycle;lhs621.head.empty_some;lhs621.head.hold;lhs621.head.loop;lhs621.head.middle;lhs621.head.nested;lhs621.head.none;lhs621.head.nothing;lhs621.head.other_cycle;lhs621.head.return;lhs621.head.scalar;lhs621.head.sequence;lhs621.head.some;lhs621.head.throw;lhs621.head.unevaluated |
| P0364 | DefinitionLhsPreparation | value;error_before_tail;same_type_error_finish;prefix_and_tail;guest_heads;held_and_splice | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. | lhs621.arg.after_prior;lhs621.arg.cycle;lhs621.arg.empty_some;lhs621.arg.hold;lhs621.arg.loop;lhs621.arg.middle;lhs621.arg.nested;lhs621.arg.none;lhs621.arg.nothing;lhs621.arg.other_cycle;lhs621.arg.return;lhs621.arg.scalar;lhs621.arg.sequence;lhs621.arg.some;lhs621.arg.throw;lhs621.arg.unevaluated |
| P0368 | MatchingSymbolCycleRecovery | value;error_before_tail;same_type_error_finish;prefix_and_tail | Stored DownValues/SubValues and before/middle/after effect log observe recovery and remaining LHS reconstruction; another fixed-point pass cannot manufacture a missing definition store. Matching vs other-symbol cycle and Throw distinguish the error finish. Middle, Twice, AlreadyResumed, Nested; first/second-pause cancel/reset and a fresh request checked. | lhs621.arg.after_prior;lhs621.arg.cycle;lhs621.arg.empty_some;lhs621.arg.hold;lhs621.arg.loop;lhs621.arg.middle;lhs621.arg.nested;lhs621.arg.none;lhs621.arg.nothing;lhs621.arg.other_cycle;lhs621.arg.return;lhs621.arg.scalar;lhs621.arg.sequence;lhs621.arg.some;lhs621.arg.throw;lhs621.arg.unevaluated;lhs621.head.after_prior;lhs621.head.cycle;lhs621.head.empty_some;lhs621.head.hold;lhs621.head.loop;lhs621.head.middle;lhs621.head.nested;lhs621.head.none;lhs621.head.nothing;lhs621.head.other_cycle;lhs621.head.return;lhs621.head.scalar;lhs621.head.sequence;lhs621.head.some;lhs621.head.throw;lhs621.head.unevaluated;lhs621.pattern.after_prior;lhs621.pattern.cycle;lhs621.pattern.empty_some;lhs621.pattern.hold;lhs621.pattern.loop;lhs621.pattern.middle;lhs621.pattern.nested;lhs621.pattern.none;lhs621.pattern.nothing;lhs621.pattern.other_cycle;lhs621.pattern.return;lhs621.pattern.scalar;lhs621.pattern.sequence;lhs621.pattern.some;lhs621.pattern.throw;lhs621.pattern.unevaluated;lhs621.two_operands |

These four are the director-accepted LHS settlements, backed by the 49 executed pairs, stored definitions, matching/nonmatching error controls, trace, lifecycle and exact source attribution. No additional Equal row was assigned in this continuation.

#### Every Lost witness in full

**P0755 — crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:104; S10; .temp/brief-606/draft-S10.md:150,169**

```json
{
  "id": "lost.confirm.info.2",
  "setup": "",
  "source": "Enclose[Confirm[$Failed, (InputString[\"i\"]; System`InternalContinuationSome[5])]]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "lost.confirm.info.2",
  "pauses": [
    "i"
  ],
  "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\", typed: Other { head: \"Failure\", text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "queued_pauses": [],
  "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\", typed: Other { head: \"Failure\", text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "same": false,
  "valid": true
}
```

.temp/brief-621/minimal-lost-cases.jsonl; .temp/brief-621/minimal-lost-results.jsonl; crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:104; evaluate(expr).map(Some) loses the Some before confirmation consumer decodes info
Tail observed: Guest InternalContinuationSome[5] becomes ordinary 5 (Cases: list element; Confirm: Information property) because the skipped Some does not protect it from legacy optional decoding

**P0883 — crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:441; S19; .temp/brief-606/draft-S19.md:110,133,165-167**

```json
{
  "id": "lost.cases.some",
  "setup": "",
  "source": "Cases[{1},_ :> (InputString[\"w\"];System`InternalContinuationSome[5])]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "lost.cases.some",
  "pauses": [
    "w"
  ],
  "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"List[InternalContinuationSome[5]]\", typed: List([Other { head: \"InternalContinuationSome\", text: \"InternalContinuationSome[5]\" }]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[InternalContinuationSome[5]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "queued_pauses": [],
  "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"List[5]\", typed: List([Integer(5)]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[5]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "same": false,
  "valid": true
}
```

.temp/brief-621/minimal-lost-cases.jsonl; .temp/brief-621/minimal-lost-results.jsonl; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:448; resume_frames.rs:306-310 returns raw Expr rather than optional collection item
Tail observed: Guest InternalContinuationSome[5] becomes ordinary 5 (Cases: list element; Confirm: Information property) because the skipped Some does not protect it from legacy optional decoding

**P0941 — crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:59; G15-C1 (proposed); S24 must retain regression after later conversion**

```json
{
  "id": "min.ReplaceAll.atom",
  "source": "ReplaceAll[1,(x_Integer /; (InputString[\"p\"];True)) :> (x+1)]",
  "setup": "",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": true
}
```

```json
{
  "id": "min.ReplaceAll.atom",
  "lifecycle": [],
  "lifecycle_ok": true,
  "pauses": [
    "p"
  ],
  "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAll\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }, EvaluationPass { before: \"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"InputString[\\\\\\\"p\\\\\\\"]\\\", after: \\\"\\\\\\\"1\\\\\\\"\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]\\\", after: \\\"True\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"RuleApplied { source: \\\"ReplaceAll\\\", input: \\\"1\\\", rule: \\\"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]]\\\", result: \\\"Plus[1, 1]\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]], Plus[x, 1]]]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"log\", after: \"List[]\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"log\\\", after: \\\"List[]\\\" }\" }, HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "queued_pauses": [],
  "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAll\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"InputString[\\\\\\\"p\\\\\\\"]\\\", after: \\\"\\\\\\\"1\\\\\\\"\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]\\\", after: \\\"True\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"RuleApplied { source: \\\"ReplaceAll\\\", input: \\\"1\\\", rule: \\\"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]]\\\", result: \\\"Plus[1, 1]\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]], Plus[x, 1]]]\\\", after: \\\"Plus[1, 1]\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"log\", after: \"List[]\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"log\\\", after: \\\"List[]\\\" }\" }, HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "same": false,
  "valid": true
}
```

```json
{
  "id": "P0941",
  "probe": "min.ReplaceAll.atom",
  "source": "ReplaceAll[1,(x_Integer /; (InputString[\"p\"];True)) :> (x+1)]",
  "inputs": "[\"1\"]",
  "pauses": [
    "p"
  ],
  "resumed_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAll\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }]",
  "queued_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAll\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }, EvaluationPass { before: \"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"2\" }]",
  "mechanism": "kernel_data/rewrite_data/replace/all.rs:59 uses ? on apply_replace_all with no caller frame for the remaining tail. After a rule guard/pattern test pauses, the typed apply_replace_all frames complete with an unevaluated replacement Expr. The caller's self-evaluating-atom test and evaluate(replaced) at :60-70 disappeared. The outer fixed point eventually evaluates Plus and repairs the scalar value, but its public EvaluationPass for the enclosing ReplaceAll call has already been emitted. Queued execution evaluates Plus first; resumed execution reports ReplaceAll -> Plus then Plus -> 2. The copied-tree frame restoring this tail makes the complete public-trace comparison SAME, with the same final scalar 2.",
  "owner": "G15-C1 (proposed); S24 must retain regression after later conversion",
  "ownership": "draft-S24.md:31 scopes this file; :101-104 and :115-118 list existing input/final-evaluation frames only. The missing post-apply producer insertion has no F/M/T row there; use a pre-S10 repair and retain it in S24."
}
```

**P0946 — crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:127; G15-C1 (proposed); S24 must retain regression after later conversion**

```json
{
  "id": "min.ReplaceAllUnheld.atom",
  "source": "ReplaceAllUnheld[1,(x_Integer /; (InputString[\"p\"];True)) :> (x+1)]",
  "setup": "",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": true
}
```

```json
{
  "id": "min.ReplaceAllUnheld.atom",
  "lifecycle": [],
  "lifecycle_ok": true,
  "pauses": [
    "p"
  ],
  "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAllUnheld\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }, EvaluationPass { before: \"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"InputString[\\\\\\\"p\\\\\\\"]\\\", after: \\\"\\\\\\\"1\\\\\\\"\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]\\\", after: \\\"True\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"RuleApplied { source: \\\"ReplaceAllUnheld\\\", input: \\\"1\\\", rule: \\\"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]]\\\", result: \\\"Plus[1, 1]\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]], Plus[x, 1]]]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"log\", after: \"List[]\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"log\\\", after: \\\"List[]\\\" }\" }, HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "queued_pauses": [],
  "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAllUnheld\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"InputString[\\\\\\\"p\\\\\\\"]\\\", after: \\\"\\\\\\\"1\\\\\\\"\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]\\\", after: \\\"True\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"RuleApplied { source: \\\"ReplaceAllUnheld\\\", input: \\\"1\\\", rule: \\\"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]]\\\", result: \\\"Plus[1, 1]\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\\\\\"p\\\\\\\"], True]], Plus[x, 1]]]\\\", after: \\\"Plus[1, 1]\\\" }\" }, HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"log\", after: \"List[]\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"log\\\", after: \\\"List[]\\\" }\" }, HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }], outputs: [HostStructuredOutput { kind: Trace, text: \"EvaluationPass { before: \\\"Plus[1, 1]\\\", after: \\\"2\\\" }\" }, HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "same": false,
  "valid": true
}
```

```json
{
  "id": "P0946",
  "probe": "min.ReplaceAllUnheld.atom",
  "source": "ReplaceAllUnheld[1,(x_Integer /; (InputString[\"p\"];True)) :> (x+1)]",
  "inputs": "[\"1\"]",
  "pauses": [
    "p"
  ],
  "resumed_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAllUnheld\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }]",
  "queued_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAllUnheld\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }, EvaluationPass { before: \"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"2\" }]",
  "mechanism": "kernel_data/rewrite_data/replace/all.rs:127 uses ? on apply_replace_all with no caller frame for the remaining tail. After a rule guard/pattern test pauses, the typed apply_replace_all frames complete with an unevaluated replacement Expr. The caller's self-evaluating-atom test and evaluate(replaced) at :128-138 disappeared. The outer fixed point eventually evaluates Plus and repairs the scalar value, but its public EvaluationPass for the enclosing ReplaceAll call has already been emitted. Queued execution evaluates Plus first; resumed execution reports ReplaceAll -> Plus then Plus -> 2. The copied-tree frame restoring this tail makes the complete public-trace comparison SAME, with the same final scalar 2.",
  "owner": "G15-C1 (proposed); S24 must retain regression after later conversion",
  "ownership": "draft-S24.md:31 scopes this file; :101-104 and :115-118 list existing input/final-evaluation frames only. The missing post-apply producer insertion has no F/M/T row there; use a pre-S10 repair and retain it in S24."
}
```

**P1489 — crates/rum-runtime/src/runtime/scoping/callable.rs:63; S13; .temp/brief-606/draft-S13.md:96,106,136-138**

```json
{
  "id": "lost.function.some",
  "setup": "",
  "source": "Function[(InputString[\"w\"];System`InternalContinuationSome[5])][]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "lost.function.some",
  "pauses": [
    "w"
  ],
  "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"InternalContinuationSome[5]\", typed: Other { head: \"InternalContinuationSome\", text: \"InternalContinuationSome[5]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"InternalContinuationSome[5]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "queued_pauses": [],
  "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"5\", typed: Integer(5) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"5\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
  "same": false,
  "valid": true
}
```

.temp/brief-621/minimal-lost-cases.jsonl; .temp/brief-621/minimal-lost-results.jsonl; crates/rum-runtime/src/runtime/scoping/callable.rs:77; TryApplyFunctionSuspensionFrame resume :163-164 absorbs Return but omits the ordinary Some
Tail observed: Guest InternalContinuationSome[5] becomes ordinary 5 (Cases: list element; Confirm: Information property) because the skipped Some does not protect it from legacy optional decoding

#### Six unexpected legacy pauses

Continuation 1 already identified all six. They retain the one-answer contract and belong to P1489 / S13 F0528/M0563; Function-based Fold/Nest remain conjunctive S3 + S13 gates. The extra request is the divergence; the harness cancellation is cleanup. Full saved records, including source and prompts, follow. These are legacy control witnesses, not newly executed internal-encoding probes.

```json
[
  {
    "id": "618.composition.none",
    "setup": "",
    "source": "Composition[Identity,Function[Null,(InputString[\"w\"];System`InternalContinuationNone[])]][0]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ],
    "actual_resumed_prompts": [
      "w",
      "w"
    ],
    "queued_prompts": [],
    "verdict": "Lost request behavior",
    "row": "P1489",
    "mechanism": "TryApplyFunctionSuspensionFrame omits Some at callable.rs:163-164; legacy Option decoder treats literal None[] or empty Some[] as decline. Caller retries the original callback, creating another request. Final cancellation is harness cleanup, not the original divergence.",
    "owner": "S13 F0528/M0563; original Fold/Nest witnesses remain conjunctive S3 + S13 gates",
    "minimized_probe": "extra.Composition.System`InternalContinuationNone[]",
    "copied_some_repair": "SAME with one p/w pause; no extra planned answer supplied"
  },
  {
    "id": "618.failsafe.none",
    "setup": "",
    "source": "Failsafe[Function[Null,(InputString[\"w\"];System`InternalContinuationNone[])]][0]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ],
    "actual_resumed_prompts": [
      "w",
      "w"
    ],
    "queued_prompts": [],
    "verdict": "Lost request behavior",
    "row": "P1489",
    "mechanism": "TryApplyFunctionSuspensionFrame omits Some at callable.rs:163-164; legacy Option decoder treats literal None[] or empty Some[] as decline. Caller retries the original callback, creating another request. Final cancellation is harness cleanup, not the original divergence.",
    "owner": "S13 F0528/M0563; original Fold/Nest witnesses remain conjunctive S3 + S13 gates",
    "minimized_probe": "extra.Failsafe.System`InternalContinuationNone[]",
    "copied_some_repair": "SAME with one p/w pause; no extra planned answer supplied"
  },
  {
    "id": "618.fold.InternalContinuationNone[]",
    "setup": "",
    "source": "Fold[Function[(InputString[\"w\"];System`InternalContinuationNone[])],0,{1}]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ],
    "actual_resumed_prompts": [
      "w",
      "w"
    ],
    "queued_prompts": [],
    "verdict": "Lost request behavior",
    "row": "P1489",
    "mechanism": "TryApplyFunctionSuspensionFrame omits Some at callable.rs:163-164; legacy Option decoder treats literal None[] or empty Some[] as decline. Caller retries the original callback, creating another request. Final cancellation is harness cleanup, not the original divergence.",
    "owner": "S13 F0528/M0563; original Fold/Nest witnesses remain conjunctive S3 + S13 gates",
    "minimized_probe": "extra.Fold.System`InternalContinuationNone[]",
    "copied_some_repair": "SAME with one p/w pause; no extra planned answer supplied"
  },
  {
    "id": "618.fold.InternalContinuationSome[]",
    "setup": "",
    "source": "Fold[Function[(InputString[\"w\"];System`InternalContinuationSome[])],0,{1}]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ],
    "actual_resumed_prompts": [
      "w",
      "w"
    ],
    "queued_prompts": [],
    "verdict": "Lost request behavior",
    "row": "P1489",
    "mechanism": "TryApplyFunctionSuspensionFrame omits Some at callable.rs:163-164; legacy Option decoder treats literal None[] or empty Some[] as decline. Caller retries the original callback, creating another request. Final cancellation is harness cleanup, not the original divergence.",
    "owner": "S13 F0528/M0563; original Fold/Nest witnesses remain conjunctive S3 + S13 gates",
    "minimized_probe": "extra.Fold.System`InternalContinuationSome[]",
    "copied_some_repair": "SAME with one p/w pause; no extra planned answer supplied"
  },
  {
    "id": "618.nest.InternalContinuationNone[]",
    "setup": "",
    "source": "Nest[Function[(InputString[\"w\"];System`InternalContinuationNone[])],0,1]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ],
    "actual_resumed_prompts": [
      "w",
      "w"
    ],
    "queued_prompts": [],
    "verdict": "Lost request behavior",
    "row": "P1489",
    "mechanism": "TryApplyFunctionSuspensionFrame omits Some at callable.rs:163-164; legacy Option decoder treats literal None[] or empty Some[] as decline. Caller retries the original callback, creating another request. Final cancellation is harness cleanup, not the original divergence.",
    "owner": "S13 F0528/M0563; original Fold/Nest witnesses remain conjunctive S3 + S13 gates",
    "minimized_probe": "extra.Nest.System`InternalContinuationNone[]",
    "copied_some_repair": "SAME with one p/w pause; no extra planned answer supplied"
  },
  {
    "id": "618.nest.InternalContinuationSome[]",
    "setup": "",
    "source": "Nest[Function[(InputString[\"w\"];System`InternalContinuationSome[])],0,1]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ],
    "actual_resumed_prompts": [
      "w",
      "w"
    ],
    "queued_prompts": [],
    "verdict": "Lost request behavior",
    "row": "P1489",
    "mechanism": "TryApplyFunctionSuspensionFrame omits Some at callable.rs:163-164; legacy Option decoder treats literal None[] or empty Some[] as decline. Caller retries the original callback, creating another request. Final cancellation is harness cleanup, not the original divergence.",
    "owner": "S13 F0528/M0563; original Fold/Nest witnesses remain conjunctive S3 + S13 gates",
    "minimized_probe": "extra.Nest.System`InternalContinuationSome[]",
    "copied_some_repair": "SAME with one p/w pause; no extra planned answer supplied"
  }
]
```

#### Eight saved internal-encoding batches: script-only classification

```json
{
  "completed_batches": 8,
  "executed_new_probes": 0,
  "probe_classifications": {
    "InvalidPlanNeedsNonEncodingControl": 394,
    "LegacyOptionalDecodeSignature": 608,
    "EscalateSkippedTailCandidate": 100,
    "CrossBuildReview": 8
  },
  "exposure_rows": 370,
  "row_frame_owner_classes": 569,
  "batches": {
    "e1-guest_heads.nested-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 24,
      "LegacyOptionalDecodeSignature": 15
    },
    "e1-guest_heads.none-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 25,
      "LegacyOptionalDecodeSignature": 15
    },
    "e1-guest_heads.some-empty-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 25,
      "LegacyOptionalDecodeSignature": 15
    },
    "e1-guest_heads.some5-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 24,
      "LegacyOptionalDecodeSignature": 15
    },
    "e1-guest_heads.tail-nested-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 73,
      "LegacyOptionalDecodeSignature": 136,
      "EscalateSkippedTailCandidate": 2,
      "CrossBuildReview": 2
    },
    "e1-guest_heads.tail-none-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 75,
      "LegacyOptionalDecodeSignature": 138,
      "EscalateSkippedTailCandidate": 44,
      "CrossBuildReview": 2
    },
    "e1-guest_heads.tail-some-empty-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 75,
      "LegacyOptionalDecodeSignature": 138,
      "EscalateSkippedTailCandidate": 44,
      "CrossBuildReview": 2
    },
    "e1-guest_heads.tail-some5-current.jsonl": {
      "InvalidPlanNeedsNonEncodingControl": 73,
      "LegacyOptionalDecodeSignature": 136,
      "EscalateSkippedTailCandidate": 10,
      "CrossBuildReview": 2
    }
  }
}
```

No new internal-encoding probe ran. The eight completed batches were read by `continuation2/classify_encoding.py`; `encoding-classifications.tsv` and `encoding-row-frame-owner-counts.tsv` provide the row/frame-or-consumer/slot/partition owner/count tables. The 608 marker-decoding signatures are grouped G13 transport evidence. The 100 non-signature and eight cross-build cases span 20 descriptor families; 68 non-encoding held/store/trace controls for those families are SAME on both builds. Invalid transformed plans stay invalid. No ancestor exposure is independently declared Lost, and none is individually minimized or described. The generated owner additions retain their G13/F1-F23/R obligations.

| owner | exposure_rows | exposure_records | probe_row_frame_count | file |
| --- | --- | --- | --- | --- |
| G13-transport | 94 | 94 | 14530 | .temp/brief-621/witness-additions/G13-transport.tsv |
| S11 | 6 | 6 | 1196 | .temp/brief-621/witness-additions/S11.tsv |
| S12 | 1 | 2 | 232 | .temp/brief-621/witness-additions/S12.tsv |
| S13 | 1 | 1 | 584 | .temp/brief-621/witness-additions/S13.tsv |
| S15 | 7 | 7 | 156 | .temp/brief-621/witness-additions/S15.tsv |
| S18 | 1 | 1 | 464 | .temp/brief-621/witness-additions/S18.tsv |
| S19 | 8 | 9 | 180 | .temp/brief-621/witness-additions/S19.tsv |
| S21 | 4 | 8 | 64 | .temp/brief-621/witness-additions/S21.tsv |
| S24 | 1 | 1 | 8 | .temp/brief-621/witness-additions/S24.tsv |
| S3 | 13 | 13 | 4886 | .temp/brief-621/witness-additions/S3.tsv |
| S35 | 10 | 12 | 288 | .temp/brief-621/witness-additions/S35.tsv |
| S36 | 10 | 12 | 288 | .temp/brief-621/witness-additions/S36.tsv |
| S37 | 19 | 19 | 284 | .temp/brief-621/witness-additions/S37.tsv |
| S38 | 6 | 6 | 80 | .temp/brief-621/witness-additions/S38.tsv |
| S39 | 4 | 6 | 132 | .temp/brief-621/witness-additions/S39.tsv |
| S4 | 2 | 2 | 16 | .temp/brief-621/witness-additions/S4.tsv |
| S42 | 9 | 10 | 110 | .temp/brief-621/witness-additions/S42.tsv |
| S44 | 1 | 1 | 58 | .temp/brief-621/witness-additions/S44.tsv |
| S5 | 4 | 4 | 688 | .temp/brief-621/witness-additions/S5.tsv |
| S50 | 1 | 1 | 4 | .temp/brief-621/witness-additions/S50.tsv |
| S9 | 1 | 1 | 4 | .temp/brief-621/witness-additions/S9.tsv |

#### Every Unreachable proof in full

Unreachable is separate from equality: 140 rows comprise eight accepted accounting closures, 65 accepted pure outcome-label calls, 26 constant-selected arithmetic calls, 23 private interpreted-matcher sites with only Rust test roots, and 18 concrete Some(options) consumer calls. These are constrained-site/guest-root proofs, not a blanket claim that an entire generic module cannot suspend. All designated producer pause counts are zero. Strongest attempts and observed counts are included verbatim below.

**P0003 — crates/rum-runtime/src/runtime/accounting_mode/bridge.rs:198**

into_rule calls finish_armed only with an already successful Expr when formerly unarmed state sees an active limit (bridge.rs:183-211). finish_armed invokes scoped with a concrete closure whose adopt_outcome input is Ok(value), then builds Rule (rule.rs:186-199); it has no guest action. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0013 — crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:43**

This edge is in the non-RuleDelayed arm of options/driver.rs:19-46; the outer match excludes precisely the RuleDelayed/arity-two case that could evaluate a guest RHS. evaluate_runtime_option_expr (options/core.rs:225-238) therefore selects clone_result. clone_result -> clone_owned only clones the existing Expr and charges it (accounting_mode/rule.rs:14-43). The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.driver.non_delayed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0023 — crates/rum-runtime/src/runtime/accounting_mode/rule.rs:20**

clone_result selects scoped<AccountingShadow> at rule.rs:20 with the concrete clone_owned closure; it cannot receive an arbitrary host Fn. clone_owned (rule.rs:26-43) clones source and enforces memory without evaluation. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0024 — crates/rum-runtime/src/runtime/accounting_mode/rule.rs:22**

clone_result selects scoped<Unarmed> at rule.rs:22 with the concrete clone_owned closure; it cannot receive an arbitrary host Fn. clone_owned (rule.rs:26-43) clones source and enforces memory without evaluation. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0029 — crates/rum-runtime/src/runtime/accounting_mode/rule.rs:112**

evaluate_in calls value.into_rule only in the Ok CallbackValue arm after cx.evaluate has already completed (rule.rs:111-115). The producer here is into_rule, not that earlier evaluator. into_rule -> finish_rule or finish_armed builds the Rule from an existing value. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0040 — crates/rum-runtime/src/runtime/accounting_mode/rule.rs:178**

resume_in calls value.into_rule only in adopt_outcome Ok(value), at rule.rs:176-182. That producer only converts a successful owned value; the exceptional arm is separate and is not included in this proof. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0041 — crates/rum-runtime/src/runtime/accounting_mode/rule.rs:191**

finish_armed calls scoped with a literal closure that supplies Ok(value) to adopt_outcome (rule.rs:186-199). scoped (retained.rs:71-104) invokes only that closure and retires its receipt; its generic ability to invoke other closures is irrelevant at this call site. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0042 — crates/rum-runtime/src/runtime/accounting_mode/rule.rs:193**

finish_armed calls into_rule only on the Ok CallbackValue arm (rule.rs:191-196). AccountingShadow::ARMED=true prevents reentering finish_armed through the unarmed check; the remaining Rule constructor/charge path cannot evaluate guest code. The only Mode implementations are sealed Unarmed and AccountingShadow (accounting_mode/lease.rs:110-205). Their receipt/resize operations allocate/count/lock but never evaluate an Expr. reserve/charge (accounting_mode/retained.rs:139-170), clone_child and finish_rule (accounting_mode/rule.rs:28-96), and adopt_outcome (accounting_mode/bridge.rs:142-176) only move/clone/build values and enforce memory. The memory enforcer (debug_tools/resources/constraints.rs:182-206) returns only Ok or MemoryConstraintExceeded. Consequently none of this closed callee chain can construct or receive a fresh InputRequired from guest evaluation. A literal InputString retained in a Function is inert until a later consumer explicitly invokes it.

probes: unreachable621.clone.unarmed;unreachable621.clone.armed;unreachable621.rule.unarmed;unreachable621.rule.armed

producer_pauses: 0

total_pauses: 0 clone; 1 delayed-rule/consumer

**P0250 — crates/rum-runtime/src/runtime/definitions/application/down_values.rs:79**

Concrete call runtime_profile_outcome_for_option_result (& direct_default) at crates/rum-runtime/src/runtime/definitions/application/down_values.rs:79 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:112-125. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P0261 — crates/rum-runtime/src/runtime/definitions/application/down_values.rs:376**

Concrete call runtime_profile_outcome_for_internal_result (& match_result) at crates/rum-runtime/src/runtime/definitions/application/down_values.rs:376 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:127-131. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.default;pure.profile.default.armed

producer_pauses: 0

observations: pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2

entry_count: 4

**P0268 — crates/rum-runtime/src/runtime/definitions/application/mod.rs:109**

Concrete call runtime_profile_outcome_for_result (result) at crates/rum-runtime/src/runtime/definitions/application/mod.rs:109 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:133-143; crates/rum-runtime/src/runtime/eval_core/mod.rs:254-264. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed

producer_pauses: 0

observations: pure.profile.own:producer=0,total=1,queued=0,enters=6;pure.profile.own.armed:producer=0,total=1,queued=0,enters=6;pure.profile.down:producer=0,total=1,queued=0,enters=5;pure.profile.down.armed:producer=0,total=1,queued=0,enters=5;pure.profile.up:producer=0,total=1,queued=0,enters=5;pure.profile.up.armed:producer=0,total=1,queued=0,enters=5;pure.profile.sub:producer=0,total=1,queued=0,enters=5;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=5;pure.profile.default:producer=0,total=1,queued=0,enters=7;pure.profile.default.armed:producer=0,total=1,queued=0,enters=7;pure.profile.lhs:producer=0,total=1,queued=0,enters=6;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=6

entry_count: 68

**P0269 — crates/rum-runtime/src/runtime/definitions/application/mod.rs:130**

Concrete call runtime_profile_outcome_for_result (result) at crates/rum-runtime/src/runtime/definitions/application/mod.rs:130 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:133-143; crates/rum-runtime/src/runtime/eval_core/mod.rs:254-264. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.default;pure.profile.default.armed

producer_pauses: 0

observations: pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2

entry_count: 4

**P0288 — crates/rum-runtime/src/runtime/definitions/application/rhs.rs:23**

Concrete call runtime_profile_outcome_for_expr_result (& rhs_result) at crates/rum-runtime/src/runtime/definitions/application/rhs.rs:23 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.own;pure.profile.own.armed;pure.profile.lhs;pure.profile.lhs.armed

producer_pauses: 0

observations: pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2

entry_count: 8

**P0291 — crates/rum-runtime/src/runtime/definitions/application/rhs.rs:44**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/definitions/application/rhs.rs:44 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.own;pure.profile.own.armed;pure.profile.lhs;pure.profile.lhs.armed

producer_pauses: 0

observations: pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2

entry_count: 8

**P0293 — crates/rum-runtime/src/runtime/definitions/application/rhs.rs:68**

Concrete call runtime_profile_outcome_for_expr_result (& rhs_result) at crates/rum-runtime/src/runtime/definitions/application/rhs.rs:68 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed

producer_pauses: 0

observations: pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.up:producer=0,total=1,queued=0,enters=2;pure.profile.up.armed:producer=0,total=1,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2

entry_count: 16

**P0295 — crates/rum-runtime/src/runtime/definitions/application/rhs.rs:78**

Concrete call runtime_profile_outcome_for_result (& result) at crates/rum-runtime/src/runtime/definitions/application/rhs.rs:78 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:133-143; crates/rum-runtime/src/runtime/eval_core/mod.rs:254-264. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed

producer_pauses: 0

observations: pure.profile.down:producer=0,total=1,queued=0,enters=1;pure.profile.down.armed:producer=0,total=1,queued=0,enters=1;pure.profile.up:producer=0,total=1,queued=0,enters=1;pure.profile.up.armed:producer=0,total=1,queued=0,enters=1;pure.profile.sub:producer=0,total=1,queued=0,enters=1;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=1;pure.profile.default:producer=0,total=1,queued=0,enters=1;pure.profile.default.armed:producer=0,total=1,queued=0,enters=1

entry_count: 8

**P0297 — crates/rum-runtime/src/runtime/definitions/application/rhs.rs:89**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/definitions/application/rhs.rs:89 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed

producer_pauses: 0

observations: pure.profile.down:producer=0,total=1,queued=0,enters=1;pure.profile.down.armed:producer=0,total=1,queued=0,enters=1;pure.profile.up:producer=0,total=1,queued=0,enters=1;pure.profile.up.armed:producer=0,total=1,queued=0,enters=1;pure.profile.sub:producer=0,total=1,queued=0,enters=1;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=1;pure.profile.default:producer=0,total=1,queued=0,enters=3;pure.profile.default.armed:producer=0,total=1,queued=0,enters=3

entry_count: 12

**P0300 — crates/rum-runtime/src/runtime/definitions/application/rhs.rs:123**

Concrete call runtime_profile_outcome_for_expr_result (& outcome) at crates/rum-runtime/src/runtime/definitions/application/rhs.rs:123 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed

producer_pauses: 0

observations: pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.up:producer=0,total=1,queued=0,enters=2;pure.profile.up.armed:producer=0,total=1,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2

entry_count: 24

**P0306 — crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:222**

Concrete call runtime_profile_outcome_for_internal_result (& match_result) at crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:222 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:127-131. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P0411 — crates/rum-runtime/src/runtime/definitions/definitions.rs:39**

Concrete call runtime_profile_outcome_for_expr_result (& rhs_result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:39 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=2;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=2;pure.profile.head:producer=0,total=2,queued=0,enters=2;pure.profile.head.armed:producer=0,total=2,queued=0,enters=2;pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.up:producer=0,total=1,queued=0,enters=2;pure.profile.up.armed:producer=0,total=1,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2;pure.profile.set:producer=0,total=2,queued=0,enters=4;pure.profile.set.armed:producer=0,total=2,queued=0,enters=4;pure.profile.with:producer=0,total=2,queued=0,enters=2;pure.profile.with.armed:producer=0,total=2,queued=0,enters=2;pure.profile.block:producer=0,total=2,queued=0,enters=2;pure.profile.block.armed:producer=0,total=2,queued=0,enters=2;pure.profile.module:producer=0,total=3,queued=0,enters=2;pure.profile.module.armed:producer=0,total=3,queued=0,enters=2;pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2;pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2;pure.profile.do:producer=0,total=2,queued=0,enters=2;pure.profile.do.armed:producer=0,total=2,queued=0,enters=2;pure.profile.while:producer=0,total=2,queued=0,enters=2;pure.profile.while.armed:producer=0,total=2,queued=0,enters=2;pure.profile.for:producer=0,total=2,queued=0,enters=2;pure.profile.for.armed:producer=0,total=2,queued=0,enters=2

entry_count: 72

**P0412 — crates/rum-runtime/src/runtime/definitions/definitions.rs:59**

Concrete call runtime_profile_outcome_for_expr_result (& rhs_result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:59 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=2;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=2;pure.profile.head:producer=0,total=2,queued=0,enters=2;pure.profile.head.armed:producer=0,total=2,queued=0,enters=2;pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.up:producer=0,total=1,queued=0,enters=2;pure.profile.up.armed:producer=0,total=1,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2;pure.profile.set:producer=0,total=2,queued=0,enters=4;pure.profile.set.armed:producer=0,total=2,queued=0,enters=4;pure.profile.with:producer=0,total=2,queued=0,enters=2;pure.profile.with.armed:producer=0,total=2,queued=0,enters=2;pure.profile.block:producer=0,total=2,queued=0,enters=2;pure.profile.block.armed:producer=0,total=2,queued=0,enters=2;pure.profile.module:producer=0,total=3,queued=0,enters=2;pure.profile.module.armed:producer=0,total=3,queued=0,enters=2;pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2;pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2;pure.profile.do:producer=0,total=2,queued=0,enters=2;pure.profile.do.armed:producer=0,total=2,queued=0,enters=2;pure.profile.while:producer=0,total=2,queued=0,enters=2;pure.profile.while.armed:producer=0,total=2,queued=0,enters=2;pure.profile.for:producer=0,total=2,queued=0,enters=2;pure.profile.for.armed:producer=0,total=2,queued=0,enters=2

entry_count: 72

**P0414 — crates/rum-runtime/src/runtime/definitions/definitions.rs:68**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:68 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=2;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=2;pure.profile.head:producer=0,total=2,queued=0,enters=2;pure.profile.head.armed:producer=0,total=2,queued=0,enters=2;pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.up:producer=0,total=1,queued=0,enters=2;pure.profile.up.armed:producer=0,total=1,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=4;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=4;pure.profile.set:producer=0,total=2,queued=0,enters=3;pure.profile.set.armed:producer=0,total=2,queued=0,enters=3;pure.profile.with:producer=0,total=2,queued=0,enters=2;pure.profile.with.armed:producer=0,total=2,queued=0,enters=2;pure.profile.block:producer=0,total=2,queued=0,enters=2;pure.profile.block.armed:producer=0,total=2,queued=0,enters=2;pure.profile.module:producer=0,total=3,queued=0,enters=2;pure.profile.module.armed:producer=0,total=3,queued=0,enters=2;pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2;pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2;pure.profile.do:producer=0,total=2,queued=0,enters=2;pure.profile.do.armed:producer=0,total=2,queued=0,enters=2;pure.profile.while:producer=0,total=2,queued=0,enters=4;pure.profile.while.armed:producer=0,total=2,queued=0,enters=4;pure.profile.for:producer=0,total=2,queued=0,enters=4;pure.profile.for.armed:producer=0,total=2,queued=0,enters=4

entry_count: 82

**P0422 — crates/rum-runtime/src/runtime/definitions/definitions.rs:174**

Concrete call runtime_profile_outcome_for_bool_result (& symbol_result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:174 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/definitions.rs:542-555. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=2;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=2;pure.profile.head:producer=0,total=2,queued=0,enters=2;pure.profile.head.armed:producer=0,total=2,queued=0,enters=2;pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.up:producer=0,total=1,queued=0,enters=2;pure.profile.up.armed:producer=0,total=1,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2;pure.profile.set:producer=0,total=2,queued=0,enters=2;pure.profile.set.armed:producer=0,total=2,queued=0,enters=2;pure.profile.with:producer=0,total=2,queued=0,enters=2;pure.profile.with.armed:producer=0,total=2,queued=0,enters=2;pure.profile.block:producer=0,total=2,queued=0,enters=2;pure.profile.block.armed:producer=0,total=2,queued=0,enters=2;pure.profile.module:producer=0,total=3,queued=0,enters=2;pure.profile.module.armed:producer=0,total=3,queued=0,enters=2;pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2;pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2;pure.profile.do:producer=0,total=2,queued=0,enters=2;pure.profile.do.armed:producer=0,total=2,queued=0,enters=2;pure.profile.while:producer=0,total=2,queued=0,enters=4;pure.profile.while.armed:producer=0,total=2,queued=0,enters=4;pure.profile.for:producer=0,total=2,queued=0,enters=4;pure.profile.for.armed:producer=0,total=2,queued=0,enters=4

entry_count: 76

**P0424 — crates/rum-runtime/src/runtime/definitions/definitions.rs:260**

Concrete call runtime_profile_outcome_for_bool_result (& symbol_result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:260 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/definitions.rs:542-555. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed

producer_pauses: 0

observations: pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2;pure.profile.set:producer=0,total=2,queued=0,enters=2;pure.profile.set.armed:producer=0,total=2,queued=0,enters=2

entry_count: 8

**P0427 — crates/rum-runtime/src/runtime/definitions/definitions.rs:322**

Concrete call runtime_profile_outcome_for_optional_result (& list_result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:322 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/definitions.rs:527-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:233-246. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed

producer_pauses: 0

observations: pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2;pure.profile.set:producer=0,total=2,queued=0,enters=2;pure.profile.set.armed:producer=0,total=2,queued=0,enters=2

entry_count: 8

**P0429 — crates/rum-runtime/src/runtime/definitions/definitions.rs:332**

Concrete call runtime_profile_outcome_for_optional_result (& assoc_result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:332 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/definitions.rs:527-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:233-246. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed

producer_pauses: 0

observations: pure.profile.lhs:producer=0,total=1,queued=0,enters=2;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=2;pure.profile.set:producer=0,total=2,queued=0,enters=2;pure.profile.set.armed:producer=0,total=2,queued=0,enters=2

entry_count: 8

**P0432 — crates/rum-runtime/src/runtime/definitions/definitions.rs:350**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/definitions/definitions.rs:350 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed

producer_pauses: 0

observations: pure.profile.own:producer=0,total=1,queued=0,enters=2;pure.profile.own.armed:producer=0,total=1,queued=0,enters=2;pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=4;pure.profile.default.armed:producer=0,total=1,queued=0,enters=4;pure.profile.lhs:producer=0,total=1,queued=0,enters=4;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=4;pure.profile.set:producer=0,total=2,queued=0,enters=2;pure.profile.set.armed:producer=0,total=2,queued=0,enters=2

entry_count: 32

**P0476 — crates/rum-runtime/src/runtime/eval_call_prepare.rs:262**

Concrete call runtime_profile_outcome_for_expr_result (& evaluated_result) at crates/rum-runtime/src/runtime/eval_call_prepare.rs:262 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=13;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=13;pure.profile.head:producer=0,total=2,queued=0,enters=12;pure.profile.head.armed:producer=0,total=2,queued=0,enters=12;pure.profile.own:producer=0,total=1,queued=0,enters=8;pure.profile.own.armed:producer=0,total=1,queued=0,enters=8;pure.profile.down:producer=0,total=1,queued=0,enters=14;pure.profile.down.armed:producer=0,total=1,queued=0,enters=14;pure.profile.up:producer=0,total=1,queued=0,enters=10;pure.profile.up.armed:producer=0,total=1,queued=0,enters=10;pure.profile.sub:producer=0,total=1,queued=0,enters=22;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=22;pure.profile.default:producer=0,total=1,queued=0,enters=8;pure.profile.default.armed:producer=0,total=1,queued=0,enters=8;pure.profile.lhs:producer=0,total=1,queued=0,enters=8;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=8;pure.profile.set:producer=0,total=2,queued=0,enters=10;pure.profile.set.armed:producer=0,total=2,queued=0,enters=10;pure.profile.with:producer=0,total=2,queued=0,enters=10;pure.profile.with.armed:producer=0,total=2,queued=0,enters=10;pure.profile.block:producer=0,total=2,queued=0,enters=10;pure.profile.block.armed:producer=0,total=2,queued=0,enters=10;pure.profile.module:producer=0,total=3,queued=0,enters=16;pure.profile.module.armed:producer=0,total=3,queued=0,enters=16;pure.profile.function:producer=0,total=1,queued=0,enters=10;pure.profile.function.armed:producer=0,total=1,queued=0,enters=10;pure.profile.catch:producer=0,total=2,queued=0,enters=14;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=14;pure.profile.do:producer=0,total=2,queued=0,enters=10;pure.profile.do.armed:producer=0,total=2,queued=0,enters=10;pure.profile.while:producer=0,total=2,queued=0,enters=30;pure.profile.while.armed:producer=0,total=2,queued=0,enters=30;pure.profile.for:producer=0,total=2,queued=0,enters=30;pure.profile.for.armed:producer=0,total=2,queued=0,enters=30

entry_count: 470

**P0477 — crates/rum-runtime/src/runtime/eval_call_prepare.rs:380**

Concrete call runtime_profile_outcome_for_shared_args_result (& result) at crates/rum-runtime/src/runtime/eval_call_prepare.rs:380 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/eval_call_prepare.rs:542-554. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=14;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=16;pure.profile.head:producer=0,total=2,queued=0,enters=14;pure.profile.head.armed:producer=0,total=2,queued=0,enters=16;pure.profile.own:producer=0,total=1,queued=0,enters=8;pure.profile.own.armed:producer=0,total=1,queued=0,enters=10;pure.profile.down:producer=0,total=1,queued=0,enters=12;pure.profile.down.armed:producer=0,total=1,queued=0,enters=14;pure.profile.up:producer=0,total=1,queued=0,enters=10;pure.profile.up.armed:producer=0,total=1,queued=0,enters=12;pure.profile.sub:producer=0,total=1,queued=0,enters=20;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=22;pure.profile.default:producer=0,total=1,queued=0,enters=12;pure.profile.default.armed:producer=0,total=1,queued=0,enters=14;pure.profile.lhs:producer=0,total=1,queued=0,enters=8;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=10;pure.profile.set:producer=0,total=2,queued=0,enters=12;pure.profile.set.armed:producer=0,total=2,queued=0,enters=14;pure.profile.with:producer=0,total=2,queued=0,enters=14;pure.profile.with.armed:producer=0,total=2,queued=0,enters=16;pure.profile.block:producer=0,total=2,queued=0,enters=12;pure.profile.block.armed:producer=0,total=2,queued=0,enters=14;pure.profile.module:producer=0,total=3,queued=0,enters=18;pure.profile.module.armed:producer=0,total=3,queued=0,enters=20;pure.profile.function:producer=0,total=1,queued=0,enters=12;pure.profile.function.armed:producer=0,total=1,queued=0,enters=14;pure.profile.catch:producer=0,total=2,queued=0,enters=16;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=18;pure.profile.do:producer=0,total=2,queued=0,enters=10;pure.profile.do.armed:producer=0,total=2,queued=0,enters=12;pure.profile.while:producer=0,total=2,queued=0,enters=30;pure.profile.while.armed:producer=0,total=2,queued=0,enters=32;pure.profile.for:producer=0,total=2,queued=0,enters=24;pure.profile.for.armed:producer=0,total=2,queued=0,enters=26

entry_count: 526

**P0480 — crates/rum-runtime/src/runtime/eval_call_prepare.rs:433**

Concrete call runtime_profile_outcome_for_expr_result (& evaluated_result) at crates/rum-runtime/src/runtime/eval_call_prepare.rs:433 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=1;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=1

entry_count: 2

**P0486 — crates/rum-runtime/src/runtime/eval_core/call.rs:47**

Concrete call runtime_profile_outcome_for_expr_result (& produced) at crates/rum-runtime/src/runtime/eval_core/call.rs:47 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.head;pure.profile.head.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.function;pure.profile.function.armed

producer_pauses: 0

observations: pure.profile.head:producer=0,total=2,queued=0,enters=2;pure.profile.head.armed:producer=0,total=2,queued=0,enters=2;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2

entry_count: 12

**P0488 — crates/rum-runtime/src/runtime/eval_core/call.rs:62**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/call.rs:62 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=16;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=18;pure.profile.head:producer=0,total=2,queued=0,enters=16;pure.profile.head.armed:producer=0,total=2,queued=0,enters=18;pure.profile.own:producer=0,total=1,queued=0,enters=12;pure.profile.own.armed:producer=0,total=1,queued=0,enters=14;pure.profile.down:producer=0,total=1,queued=0,enters=16;pure.profile.down.armed:producer=0,total=1,queued=0,enters=18;pure.profile.up:producer=0,total=1,queued=0,enters=14;pure.profile.up.armed:producer=0,total=1,queued=0,enters=16;pure.profile.sub:producer=0,total=1,queued=0,enters=24;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=26;pure.profile.default:producer=0,total=1,queued=0,enters=18;pure.profile.default.armed:producer=0,total=1,queued=0,enters=20;pure.profile.lhs:producer=0,total=1,queued=0,enters=14;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=16;pure.profile.set:producer=0,total=2,queued=0,enters=16;pure.profile.set.armed:producer=0,total=2,queued=0,enters=18;pure.profile.with:producer=0,total=2,queued=0,enters=16;pure.profile.with.armed:producer=0,total=2,queued=0,enters=18;pure.profile.block:producer=0,total=2,queued=0,enters=16;pure.profile.block.armed:producer=0,total=2,queued=0,enters=18;pure.profile.module:producer=0,total=3,queued=0,enters=22;pure.profile.module.armed:producer=0,total=3,queued=0,enters=24;pure.profile.function:producer=0,total=1,queued=0,enters=14;pure.profile.function.armed:producer=0,total=1,queued=0,enters=16;pure.profile.catch:producer=0,total=2,queued=0,enters=18;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=20;pure.profile.do:producer=0,total=2,queued=0,enters=12;pure.profile.do.armed:producer=0,total=2,queued=0,enters=14;pure.profile.while:producer=0,total=2,queued=0,enters=34;pure.profile.while.armed:producer=0,total=2,queued=0,enters=36;pure.profile.for:producer=0,total=2,queued=0,enters=28;pure.profile.for.armed:producer=0,total=2,queued=0,enters=30

entry_count: 646

**P0490 — crates/rum-runtime/src/runtime/eval_core/call.rs:83**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/call.rs:83 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=16;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=18;pure.profile.head:producer=0,total=2,queued=0,enters=16;pure.profile.head.armed:producer=0,total=2,queued=0,enters=18;pure.profile.own:producer=0,total=1,queued=0,enters=12;pure.profile.own.armed:producer=0,total=1,queued=0,enters=14;pure.profile.down:producer=0,total=1,queued=0,enters=16;pure.profile.down.armed:producer=0,total=1,queued=0,enters=18;pure.profile.up:producer=0,total=1,queued=0,enters=14;pure.profile.up.armed:producer=0,total=1,queued=0,enters=16;pure.profile.sub:producer=0,total=1,queued=0,enters=24;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=26;pure.profile.default:producer=0,total=1,queued=0,enters=18;pure.profile.default.armed:producer=0,total=1,queued=0,enters=20;pure.profile.lhs:producer=0,total=1,queued=0,enters=14;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=16;pure.profile.set:producer=0,total=2,queued=0,enters=16;pure.profile.set.armed:producer=0,total=2,queued=0,enters=18;pure.profile.with:producer=0,total=2,queued=0,enters=16;pure.profile.with.armed:producer=0,total=2,queued=0,enters=18;pure.profile.block:producer=0,total=2,queued=0,enters=16;pure.profile.block.armed:producer=0,total=2,queued=0,enters=18;pure.profile.module:producer=0,total=3,queued=0,enters=22;pure.profile.module.armed:producer=0,total=3,queued=0,enters=24;pure.profile.function:producer=0,total=1,queued=0,enters=14;pure.profile.function.armed:producer=0,total=1,queued=0,enters=16;pure.profile.catch:producer=0,total=2,queued=0,enters=18;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=20;pure.profile.do:producer=0,total=2,queued=0,enters=12;pure.profile.do.armed:producer=0,total=2,queued=0,enters=14;pure.profile.while:producer=0,total=2,queued=0,enters=34;pure.profile.while.armed:producer=0,total=2,queued=0,enters=36;pure.profile.for:producer=0,total=2,queued=0,enters=28;pure.profile.for.armed:producer=0,total=2,queued=0,enters=30

entry_count: 646

**P0491 — crates/rum-runtime/src/runtime/eval_core/call.rs:87**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/call.rs:87 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=16;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=18;pure.profile.head:producer=0,total=2,queued=0,enters=16;pure.profile.head.armed:producer=0,total=2,queued=0,enters=18;pure.profile.own:producer=0,total=1,queued=0,enters=12;pure.profile.own.armed:producer=0,total=1,queued=0,enters=14;pure.profile.down:producer=0,total=1,queued=0,enters=16;pure.profile.down.armed:producer=0,total=1,queued=0,enters=18;pure.profile.up:producer=0,total=1,queued=0,enters=14;pure.profile.up.armed:producer=0,total=1,queued=0,enters=16;pure.profile.sub:producer=0,total=1,queued=0,enters=24;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=26;pure.profile.default:producer=0,total=1,queued=0,enters=18;pure.profile.default.armed:producer=0,total=1,queued=0,enters=20;pure.profile.lhs:producer=0,total=1,queued=0,enters=14;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=16;pure.profile.set:producer=0,total=2,queued=0,enters=16;pure.profile.set.armed:producer=0,total=2,queued=0,enters=18;pure.profile.with:producer=0,total=2,queued=0,enters=16;pure.profile.with.armed:producer=0,total=2,queued=0,enters=18;pure.profile.block:producer=0,total=2,queued=0,enters=16;pure.profile.block.armed:producer=0,total=2,queued=0,enters=18;pure.profile.module:producer=0,total=3,queued=0,enters=22;pure.profile.module.armed:producer=0,total=3,queued=0,enters=24;pure.profile.function:producer=0,total=1,queued=0,enters=14;pure.profile.function.armed:producer=0,total=1,queued=0,enters=16;pure.profile.catch:producer=0,total=2,queued=0,enters=18;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=20;pure.profile.do:producer=0,total=2,queued=0,enters=12;pure.profile.do.armed:producer=0,total=2,queued=0,enters=14;pure.profile.while:producer=0,total=2,queued=0,enters=34;pure.profile.while.armed:producer=0,total=2,queued=0,enters=36;pure.profile.for:producer=0,total=2,queued=0,enters=28;pure.profile.for.armed:producer=0,total=2,queued=0,enters=30

entry_count: 646

**P0493 — crates/rum-runtime/src/runtime/eval_core/call.rs:152**

Concrete call runtime_profile_outcome_for_expr_result (& failed) at crates/rum-runtime/src/runtime/eval_core/call.rs:152 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P0495 — crates/rum-runtime/src/runtime/eval_core/call.rs:172**

Concrete call runtime_profile_outcome_for_optional_result (& produced) at crates/rum-runtime/src/runtime/eval_core/call.rs:172 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/definitions.rs:527-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:233-246. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=16;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=18;pure.profile.head:producer=0,total=2,queued=0,enters=16;pure.profile.head.armed:producer=0,total=2,queued=0,enters=18;pure.profile.own:producer=0,total=1,queued=0,enters=14;pure.profile.own.armed:producer=0,total=1,queued=0,enters=16;pure.profile.down:producer=0,total=1,queued=0,enters=20;pure.profile.down.armed:producer=0,total=1,queued=0,enters=22;pure.profile.up:producer=0,total=1,queued=0,enters=24;pure.profile.up.armed:producer=0,total=1,queued=0,enters=28;pure.profile.sub:producer=0,total=1,queued=0,enters=22;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=24;pure.profile.default:producer=0,total=1,queued=0,enters=20;pure.profile.default.armed:producer=0,total=1,queued=0,enters=22;pure.profile.lhs:producer=0,total=1,queued=0,enters=16;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=18;pure.profile.set:producer=0,total=2,queued=0,enters=18;pure.profile.set.armed:producer=0,total=2,queued=0,enters=20;pure.profile.with:producer=0,total=2,queued=0,enters=18;pure.profile.with.armed:producer=0,total=2,queued=0,enters=20;pure.profile.block:producer=0,total=2,queued=0,enters=18;pure.profile.block.armed:producer=0,total=2,queued=0,enters=20;pure.profile.module:producer=0,total=3,queued=0,enters=26;pure.profile.module.armed:producer=0,total=3,queued=0,enters=28;pure.profile.function:producer=0,total=1,queued=0,enters=16;pure.profile.function.armed:producer=0,total=1,queued=0,enters=18;pure.profile.catch:producer=0,total=2,queued=0,enters=20;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=22;pure.profile.do:producer=0,total=2,queued=0,enters=14;pure.profile.do.armed:producer=0,total=2,queued=0,enters=16;pure.profile.while:producer=0,total=2,queued=0,enters=40;pure.profile.while.armed:producer=0,total=2,queued=0,enters=42;pure.profile.for:producer=0,total=2,queued=0,enters=34;pure.profile.for.armed:producer=0,total=2,queued=0,enters=36

entry_count: 740

**P0497 — crates/rum-runtime/src/runtime/eval_core/call.rs:251**

Concrete call runtime_profile_outcome_for_expr_result (& failed) at crates/rum-runtime/src/runtime/eval_core/call.rs:251 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P0499 — crates/rum-runtime/src/runtime/eval_core/call.rs:271**

Concrete call runtime_profile_outcome_for_optional_result (& produced) at crates/rum-runtime/src/runtime/eval_core/call.rs:271 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/definitions.rs:527-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:233-246. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P0510 — crates/rum-runtime/src/runtime/eval_core/mod.rs:149**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/mod.rs:149 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=2;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=2;pure.profile.head:producer=0,total=2,queued=0,enters=2;pure.profile.head.armed:producer=0,total=2,queued=0,enters=2;pure.profile.own:producer=0,total=1,queued=0,enters=4;pure.profile.own.armed:producer=0,total=1,queued=0,enters=4;pure.profile.down:producer=0,total=1,queued=0,enters=2;pure.profile.down.armed:producer=0,total=1,queued=0,enters=2;pure.profile.up:producer=0,total=1,queued=0,enters=4;pure.profile.up.armed:producer=0,total=1,queued=0,enters=4;pure.profile.sub:producer=0,total=1,queued=0,enters=2;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=2;pure.profile.default:producer=0,total=1,queued=0,enters=2;pure.profile.default.armed:producer=0,total=1,queued=0,enters=2;pure.profile.lhs:producer=0,total=1,queued=0,enters=4;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=4;pure.profile.set:producer=0,total=2,queued=0,enters=2;pure.profile.set.armed:producer=0,total=2,queued=0,enters=2;pure.profile.with:producer=0,total=2,queued=0,enters=2;pure.profile.with.armed:producer=0,total=2,queued=0,enters=2;pure.profile.block:producer=0,total=2,queued=0,enters=4;pure.profile.block.armed:producer=0,total=2,queued=0,enters=4;pure.profile.module:producer=0,total=3,queued=0,enters=6;pure.profile.module.armed:producer=0,total=3,queued=0,enters=6;pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2;pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2;pure.profile.do:producer=0,total=2,queued=0,enters=2;pure.profile.do.armed:producer=0,total=2,queued=0,enters=2;pure.profile.while:producer=0,total=2,queued=0,enters=10;pure.profile.while.armed:producer=0,total=2,queued=0,enters=10;pure.profile.for:producer=0,total=2,queued=0,enters=8;pure.profile.for.armed:producer=0,total=2,queued=0,enters=8

entry_count: 120

**P0512 — crates/rum-runtime/src/runtime/eval_core/mod.rs:158**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/mod.rs:158 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=16;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=18;pure.profile.head:producer=0,total=2,queued=0,enters=16;pure.profile.head.armed:producer=0,total=2,queued=0,enters=18;pure.profile.own:producer=0,total=1,queued=0,enters=12;pure.profile.own.armed:producer=0,total=1,queued=0,enters=14;pure.profile.down:producer=0,total=1,queued=0,enters=16;pure.profile.down.armed:producer=0,total=1,queued=0,enters=18;pure.profile.up:producer=0,total=1,queued=0,enters=14;pure.profile.up.armed:producer=0,total=1,queued=0,enters=16;pure.profile.sub:producer=0,total=1,queued=0,enters=24;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=26;pure.profile.default:producer=0,total=1,queued=0,enters=18;pure.profile.default.armed:producer=0,total=1,queued=0,enters=20;pure.profile.lhs:producer=0,total=1,queued=0,enters=14;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=16;pure.profile.set:producer=0,total=2,queued=0,enters=16;pure.profile.set.armed:producer=0,total=2,queued=0,enters=18;pure.profile.with:producer=0,total=2,queued=0,enters=16;pure.profile.with.armed:producer=0,total=2,queued=0,enters=18;pure.profile.block:producer=0,total=2,queued=0,enters=16;pure.profile.block.armed:producer=0,total=2,queued=0,enters=18;pure.profile.module:producer=0,total=3,queued=0,enters=22;pure.profile.module.armed:producer=0,total=3,queued=0,enters=24;pure.profile.function:producer=0,total=1,queued=0,enters=14;pure.profile.function.armed:producer=0,total=1,queued=0,enters=16;pure.profile.catch:producer=0,total=2,queued=0,enters=18;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=20;pure.profile.do:producer=0,total=2,queued=0,enters=12;pure.profile.do.armed:producer=0,total=2,queued=0,enters=14;pure.profile.while:producer=0,total=2,queued=0,enters=34;pure.profile.while.armed:producer=0,total=2,queued=0,enters=36;pure.profile.for:producer=0,total=2,queued=0,enters=28;pure.profile.for.armed:producer=0,total=2,queued=0,enters=30

entry_count: 646

**P0513 — crates/rum-runtime/src/runtime/eval_core/mod.rs:167**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/mod.rs:167 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P0514 — crates/rum-runtime/src/runtime/eval_core/mod.rs:174**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/mod.rs:174 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=18;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=20;pure.profile.head:producer=0,total=2,queued=0,enters=18;pure.profile.head.armed:producer=0,total=2,queued=0,enters=20;pure.profile.own:producer=0,total=1,queued=0,enters=16;pure.profile.own.armed:producer=0,total=1,queued=0,enters=18;pure.profile.down:producer=0,total=1,queued=0,enters=18;pure.profile.down.armed:producer=0,total=1,queued=0,enters=20;pure.profile.up:producer=0,total=1,queued=0,enters=18;pure.profile.up.armed:producer=0,total=1,queued=0,enters=20;pure.profile.sub:producer=0,total=1,queued=0,enters=26;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=28;pure.profile.default:producer=0,total=1,queued=0,enters=20;pure.profile.default.armed:producer=0,total=1,queued=0,enters=22;pure.profile.lhs:producer=0,total=1,queued=0,enters=18;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=20;pure.profile.set:producer=0,total=2,queued=0,enters=18;pure.profile.set.armed:producer=0,total=2,queued=0,enters=20;pure.profile.with:producer=0,total=2,queued=0,enters=18;pure.profile.with.armed:producer=0,total=2,queued=0,enters=20;pure.profile.block:producer=0,total=2,queued=0,enters=20;pure.profile.block.armed:producer=0,total=2,queued=0,enters=22;pure.profile.module:producer=0,total=3,queued=0,enters=28;pure.profile.module.armed:producer=0,total=3,queued=0,enters=30;pure.profile.function:producer=0,total=1,queued=0,enters=16;pure.profile.function.armed:producer=0,total=1,queued=0,enters=18;pure.profile.catch:producer=0,total=2,queued=0,enters=20;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=22;pure.profile.do:producer=0,total=2,queued=0,enters=14;pure.profile.do.armed:producer=0,total=2,queued=0,enters=16;pure.profile.while:producer=0,total=2,queued=0,enters=44;pure.profile.while.armed:producer=0,total=2,queued=0,enters=46;pure.profile.for:producer=0,total=2,queued=0,enters=36;pure.profile.for.armed:producer=0,total=2,queued=0,enters=38

entry_count: 766

**P0516 — crates/rum-runtime/src/runtime/eval_core/mod.rs:191**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/mod.rs:191 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=14;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=14;pure.profile.head:producer=0,total=2,queued=0,enters=12;pure.profile.head.armed:producer=0,total=2,queued=0,enters=12;pure.profile.own:producer=0,total=1,queued=0,enters=8;pure.profile.own.armed:producer=0,total=1,queued=0,enters=8;pure.profile.down:producer=0,total=1,queued=0,enters=16;pure.profile.down.armed:producer=0,total=1,queued=0,enters=16;pure.profile.up:producer=0,total=1,queued=0,enters=14;pure.profile.up.armed:producer=0,total=1,queued=0,enters=14;pure.profile.sub:producer=0,total=1,queued=0,enters=24;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=24;pure.profile.default:producer=0,total=1,queued=0,enters=10;pure.profile.default.armed:producer=0,total=1,queued=0,enters=10;pure.profile.lhs:producer=0,total=1,queued=0,enters=12;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=12;pure.profile.set:producer=0,total=2,queued=0,enters=14;pure.profile.set.armed:producer=0,total=2,queued=0,enters=14;pure.profile.with:producer=0,total=2,queued=0,enters=10;pure.profile.with.armed:producer=0,total=2,queued=0,enters=10;pure.profile.block:producer=0,total=2,queued=0,enters=10;pure.profile.block.armed:producer=0,total=2,queued=0,enters=10;pure.profile.module:producer=0,total=3,queued=0,enters=16;pure.profile.module.armed:producer=0,total=3,queued=0,enters=16;pure.profile.function:producer=0,total=1,queued=0,enters=10;pure.profile.function.armed:producer=0,total=1,queued=0,enters=10;pure.profile.catch:producer=0,total=2,queued=0,enters=14;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=14;pure.profile.do:producer=0,total=2,queued=0,enters=10;pure.profile.do.armed:producer=0,total=2,queued=0,enters=10;pure.profile.while:producer=0,total=2,queued=0,enters=30;pure.profile.while.armed:producer=0,total=2,queued=0,enters=30;pure.profile.for:producer=0,total=2,queued=0,enters=30;pure.profile.for.armed:producer=0,total=2,queued=0,enters=30

entry_count: 508

**P0517 — crates/rum-runtime/src/runtime/eval_core/mod.rs:195**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/eval_core/mod.rs:195 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=14;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=14;pure.profile.head:producer=0,total=2,queued=0,enters=12;pure.profile.head.armed:producer=0,total=2,queued=0,enters=12;pure.profile.own:producer=0,total=1,queued=0,enters=8;pure.profile.own.armed:producer=0,total=1,queued=0,enters=8;pure.profile.down:producer=0,total=1,queued=0,enters=16;pure.profile.down.armed:producer=0,total=1,queued=0,enters=16;pure.profile.up:producer=0,total=1,queued=0,enters=14;pure.profile.up.armed:producer=0,total=1,queued=0,enters=14;pure.profile.sub:producer=0,total=1,queued=0,enters=24;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=24;pure.profile.default:producer=0,total=1,queued=0,enters=10;pure.profile.default.armed:producer=0,total=1,queued=0,enters=10;pure.profile.lhs:producer=0,total=1,queued=0,enters=12;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=12;pure.profile.set:producer=0,total=2,queued=0,enters=14;pure.profile.set.armed:producer=0,total=2,queued=0,enters=14;pure.profile.with:producer=0,total=2,queued=0,enters=10;pure.profile.with.armed:producer=0,total=2,queued=0,enters=10;pure.profile.block:producer=0,total=2,queued=0,enters=10;pure.profile.block.armed:producer=0,total=2,queued=0,enters=10;pure.profile.module:producer=0,total=3,queued=0,enters=16;pure.profile.module.armed:producer=0,total=3,queued=0,enters=16;pure.profile.function:producer=0,total=1,queued=0,enters=10;pure.profile.function.armed:producer=0,total=1,queued=0,enters=10;pure.profile.catch:producer=0,total=2,queued=0,enters=14;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=14;pure.profile.do:producer=0,total=2,queued=0,enters=10;pure.profile.do.armed:producer=0,total=2,queued=0,enters=10;pure.profile.while:producer=0,total=2,queued=0,enters=30;pure.profile.while.armed:producer=0,total=2,queued=0,enters=30;pure.profile.for:producer=0,total=2,queued=0,enters=30;pure.profile.for.armed:producer=0,total=2,queued=0,enters=30

entry_count: 508

**P0518 — crates/rum-runtime/src/runtime/eval_core/mod.rs:251**

Concrete call runtime_profile_outcome_for_result (result) at crates/rum-runtime/src/runtime/eval_core/mod.rs:251 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:133-143; crates/rum-runtime/src/runtime/eval_core/mod.rs:254-264. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=112;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=122;pure.profile.head:producer=0,total=2,queued=0,enters=110;pure.profile.head.armed:producer=0,total=2,queued=0,enters=120;pure.profile.own:producer=0,total=1,queued=0,enters=84;pure.profile.own.armed:producer=0,total=1,queued=0,enters=94;pure.profile.down:producer=0,total=1,queued=0,enters=116;pure.profile.down.armed:producer=0,total=1,queued=0,enters=126;pure.profile.up:producer=0,total=1,queued=0,enters=106;pure.profile.up.armed:producer=0,total=1,queued=0,enters=116;pure.profile.sub:producer=0,total=1,queued=0,enters=174;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=184;pure.profile.default:producer=0,total=1,queued=0,enters=114;pure.profile.default.armed:producer=0,total=1,queued=0,enters=124;pure.profile.lhs:producer=0,total=1,queued=0,enters=102;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=112;pure.profile.set:producer=0,total=2,queued=0,enters=112;pure.profile.set.armed:producer=0,total=2,queued=0,enters=122;pure.profile.with:producer=0,total=2,queued=0,enters=104;pure.profile.with.armed:producer=0,total=2,queued=0,enters=114;pure.profile.block:producer=0,total=2,queued=0,enters=108;pure.profile.block.armed:producer=0,total=2,queued=0,enters=118;pure.profile.module:producer=0,total=3,queued=0,enters=154;pure.profile.module.armed:producer=0,total=3,queued=0,enters=164;pure.profile.function:producer=0,total=1,queued=0,enters=96;pure.profile.function.armed:producer=0,total=1,queued=0,enters=106;pure.profile.catch:producer=0,total=2,queued=0,enters=122;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=132;pure.profile.do:producer=0,total=2,queued=0,enters=84;pure.profile.do.armed:producer=0,total=2,queued=0,enters=94;pure.profile.while:producer=0,total=2,queued=0,enters=250;pure.profile.while.armed:producer=0,total=2,queued=0,enters=260;pure.profile.for:producer=0,total=2,queued=0,enters=216;pure.profile.for.armed:producer=0,total=2,queued=0,enters=226

entry_count: 4498

**P0663 — crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:159**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:159 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.while;pure.profile.while.armed

producer_pauses: 0

observations: pure.profile.while:producer=0,total=2,queued=0,enters=6;pure.profile.while.armed:producer=0,total=2,queued=0,enters=6

entry_count: 12

**P0664 — crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:173**

Concrete call runtime_profile_outcome_for_expr_result (& outcome) at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:173 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.while;pure.profile.while.armed

producer_pauses: 0

observations: pure.profile.while:producer=0,total=2,queued=0,enters=6;pure.profile.while.armed:producer=0,total=2,queued=0,enters=6

entry_count: 12

**P0666 — crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:183**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:183 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.while;pure.profile.while.armed

producer_pauses: 0

observations: pure.profile.while:producer=0,total=2,queued=0,enters=4;pure.profile.while.armed:producer=0,total=2,queued=0,enters=4

entry_count: 8

**P0667 — crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:197**

Concrete call runtime_profile_outcome_for_expr_result (& outcome) at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:197 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.while;pure.profile.while.armed

producer_pauses: 0

observations: pure.profile.while:producer=0,total=2,queued=0,enters=4;pure.profile.while.armed:producer=0,total=2,queued=0,enters=4

entry_count: 8

**P0674 — crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:362**

Concrete call runtime_profile_outcome_for_expr_result (& current_result) at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:362 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.head;pure.profile.own;pure.profile.down;pure.profile.up;pure.profile.sub;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.set;pure.profile.with;pure.profile.block;pure.profile.module;pure.profile.function;pure.profile.catch;pure.profile.while

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=8;pure.profile.head:producer=0,total=2,queued=0,enters=8;pure.profile.own:producer=0,total=1,queued=0,enters=4;pure.profile.down:producer=0,total=1,queued=0,enters=4;pure.profile.up:producer=0,total=1,queued=0,enters=4;pure.profile.sub:producer=0,total=1,queued=0,enters=4;pure.profile.default:producer=0,total=1,queued=0,enters=8;pure.profile.default.armed:producer=0,total=1,queued=0,enters=4;pure.profile.lhs:producer=0,total=1,queued=0,enters=4;pure.profile.set:producer=0,total=2,queued=0,enters=8;pure.profile.with:producer=0,total=2,queued=0,enters=8;pure.profile.block:producer=0,total=2,queued=0,enters=8;pure.profile.module:producer=0,total=3,queued=0,enters=12;pure.profile.function:producer=0,total=1,queued=0,enters=4;pure.profile.catch:producer=0,total=2,queued=0,enters=8;pure.profile.while:producer=0,total=2,queued=0,enters=14

entry_count: 110

**P0675 — crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:376**

Concrete call runtime_profile_outcome_for_expr_result (& outcome) at crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:376 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.head;pure.profile.own;pure.profile.down;pure.profile.up;pure.profile.sub;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.set;pure.profile.with;pure.profile.block;pure.profile.module;pure.profile.function;pure.profile.catch;pure.profile.while

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=8;pure.profile.head:producer=0,total=2,queued=0,enters=8;pure.profile.own:producer=0,total=1,queued=0,enters=4;pure.profile.down:producer=0,total=1,queued=0,enters=4;pure.profile.up:producer=0,total=1,queued=0,enters=4;pure.profile.sub:producer=0,total=1,queued=0,enters=4;pure.profile.default:producer=0,total=1,queued=0,enters=8;pure.profile.default.armed:producer=0,total=1,queued=0,enters=4;pure.profile.lhs:producer=0,total=1,queued=0,enters=4;pure.profile.set:producer=0,total=2,queued=0,enters=8;pure.profile.with:producer=0,total=2,queued=0,enters=8;pure.profile.block:producer=0,total=2,queued=0,enters=8;pure.profile.module:producer=0,total=3,queued=0,enters=12;pure.profile.function:producer=0,total=1,queued=0,enters=4;pure.profile.catch:producer=0,total=2,queued=0,enters=8;pure.profile.while:producer=0,total=2,queued=0,enters=14

entry_count: 110

**P0807 — crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:25**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:25 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.function;pure.profile.function.armed

producer_pauses: 0

observations: pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2

entry_count: 4

**P0808 — crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:51**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:51 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.catch;pure.profile.catch.armed

producer_pauses: 0

observations: pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2

entry_count: 4

**P0809 — crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:86**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:86 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P0811 — crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:98**

Concrete call runtime_profile_outcome_for_expr_result (& body_result) at crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:98 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.catch;pure.profile.catch.armed

producer_pauses: 0

observations: pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2

entry_count: 4

**P0815 — crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:109**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:109 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.catch;pure.profile.catch.armed

producer_pauses: 0

observations: pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2

entry_count: 4

**P0816 — crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:121**

Concrete call runtime_profile_outcome_for_expr_result (& outcome) at crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:121 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.catch;pure.profile.catch.armed

producer_pauses: 0

observations: pure.profile.catch:producer=0,total=2,queued=0,enters=2;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=2

entry_count: 4

**P1473 — crates/rum-runtime/src/runtime/resolution.rs:329**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/resolution.rs:329 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=26;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=30;pure.profile.head:producer=0,total=2,queued=0,enters=26;pure.profile.head.armed:producer=0,total=2,queued=0,enters=30;pure.profile.own:producer=0,total=1,queued=0,enters=22;pure.profile.own.armed:producer=0,total=1,queued=0,enters=26;pure.profile.down:producer=0,total=1,queued=0,enters=22;pure.profile.down.armed:producer=0,total=1,queued=0,enters=26;pure.profile.up:producer=0,total=1,queued=0,enters=24;pure.profile.up.armed:producer=0,total=1,queued=0,enters=28;pure.profile.sub:producer=0,total=1,queued=0,enters=30;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=34;pure.profile.default:producer=0,total=1,queued=0,enters=26;pure.profile.default.armed:producer=0,total=1,queued=0,enters=30;pure.profile.lhs:producer=0,total=1,queued=0,enters=24;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=28;pure.profile.set:producer=0,total=2,queued=0,enters=26;pure.profile.set.armed:producer=0,total=2,queued=0,enters=30;pure.profile.with:producer=0,total=2,queued=0,enters=26;pure.profile.with.armed:producer=0,total=2,queued=0,enters=30;pure.profile.block:producer=0,total=2,queued=0,enters=26;pure.profile.block.armed:producer=0,total=2,queued=0,enters=30;pure.profile.module:producer=0,total=3,queued=0,enters=36;pure.profile.module.armed:producer=0,total=3,queued=0,enters=40;pure.profile.function:producer=0,total=1,queued=0,enters=22;pure.profile.function.armed:producer=0,total=1,queued=0,enters=26;pure.profile.catch:producer=0,total=2,queued=0,enters=26;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=30;pure.profile.do:producer=0,total=2,queued=0,enters=20;pure.profile.do.armed:producer=0,total=2,queued=0,enters=24;pure.profile.while:producer=0,total=2,queued=0,enters=48;pure.profile.while.armed:producer=0,total=2,queued=0,enters=52;pure.profile.for:producer=0,total=2,queued=0,enters=40;pure.profile.for.armed:producer=0,total=2,queued=0,enters=44

entry_count: 1008

**P1475 — crates/rum-runtime/src/runtime/resolution.rs:373**

Concrete call runtime_profile_outcome_for_expr_result (& next_result) at crates/rum-runtime/src/runtime/resolution.rs:373 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=18;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=20;pure.profile.head:producer=0,total=2,queued=0,enters=18;pure.profile.head.armed:producer=0,total=2,queued=0,enters=20;pure.profile.own:producer=0,total=1,queued=0,enters=16;pure.profile.own.armed:producer=0,total=1,queued=0,enters=18;pure.profile.down:producer=0,total=1,queued=0,enters=18;pure.profile.down.armed:producer=0,total=1,queued=0,enters=20;pure.profile.up:producer=0,total=1,queued=0,enters=18;pure.profile.up.armed:producer=0,total=1,queued=0,enters=20;pure.profile.sub:producer=0,total=1,queued=0,enters=26;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=28;pure.profile.default:producer=0,total=1,queued=0,enters=20;pure.profile.default.armed:producer=0,total=1,queued=0,enters=22;pure.profile.lhs:producer=0,total=1,queued=0,enters=18;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=20;pure.profile.set:producer=0,total=2,queued=0,enters=18;pure.profile.set.armed:producer=0,total=2,queued=0,enters=20;pure.profile.with:producer=0,total=2,queued=0,enters=18;pure.profile.with.armed:producer=0,total=2,queued=0,enters=20;pure.profile.block:producer=0,total=2,queued=0,enters=20;pure.profile.block.armed:producer=0,total=2,queued=0,enters=22;pure.profile.module:producer=0,total=3,queued=0,enters=28;pure.profile.module.armed:producer=0,total=3,queued=0,enters=30;pure.profile.function:producer=0,total=1,queued=0,enters=16;pure.profile.function.armed:producer=0,total=1,queued=0,enters=18;pure.profile.catch:producer=0,total=2,queued=0,enters=20;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=22;pure.profile.do:producer=0,total=2,queued=0,enters=14;pure.profile.do.armed:producer=0,total=2,queued=0,enters=16;pure.profile.while:producer=0,total=2,queued=0,enters=44;pure.profile.while.armed:producer=0,total=2,queued=0,enters=46;pure.profile.for:producer=0,total=2,queued=0,enters=36;pure.profile.for.armed:producer=0,total=2,queued=0,enters=38

entry_count: 766

**P1481 — crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:38**

Concrete call runtime_profile_outcome_for_expr_result (& applied_result) at crates/rum-runtime/src/runtime/scoping/callable/status_application.rs:38 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P1490 — crates/rum-runtime/src/runtime/scoping/callable.rs:66**

Concrete call runtime_profile_outcome_for_expr_result (& applied_result) at crates/rum-runtime/src/runtime/scoping/callable.rs:66 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.function;pure.profile.function.armed

producer_pauses: 0

observations: pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2

entry_count: 4

**P1492 — crates/rum-runtime/src/runtime/scoping/callable.rs:86**

Concrete call runtime_profile_outcome_for_expr_result (& outcome) at crates/rum-runtime/src/runtime/scoping/callable.rs:86 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.function;pure.profile.function.armed

producer_pauses: 0

observations: pure.profile.function:producer=0,total=1,queued=0,enters=2;pure.profile.function.armed:producer=0,total=1,queued=0,enters=2

entry_count: 4

**P1508 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:82**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:82 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=2;pure.profile.module.armed:producer=0,total=3,queued=0,enters=2

entry_count: 4

**P1509 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:109**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:109 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P1511 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:129**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:129 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=1;pure.profile.module.armed:producer=0,total=3,queued=0,enters=1

entry_count: 2

**P1512 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:146**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:146 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=1;pure.profile.module.armed:producer=0,total=3,queued=0,enters=1

entry_count: 2

**P1514 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:164**

Concrete call runtime_profile_outcome_for_expr_result (& init_result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:164 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=3;pure.profile.module.armed:producer=0,total=3,queued=0,enters=3

entry_count: 6

**P1515 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:179**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:179 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=1;pure.profile.module.armed:producer=0,total=3,queued=0,enters=1

entry_count: 2

**P1516 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:210**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:210 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.preparation;pure.profile.preparation.armed;pure.profile.head;pure.profile.head.armed;pure.profile.own;pure.profile.own.armed;pure.profile.down;pure.profile.down.armed;pure.profile.up;pure.profile.up.armed;pure.profile.sub;pure.profile.sub.armed;pure.profile.default;pure.profile.default.armed;pure.profile.lhs;pure.profile.lhs.armed;pure.profile.set;pure.profile.set.armed;pure.profile.with;pure.profile.with.armed;pure.profile.block;pure.profile.block.armed;pure.profile.module;pure.profile.module.armed;pure.profile.function;pure.profile.function.armed;pure.profile.catch;pure.profile.catch.armed;pure.profile.do;pure.profile.do.armed;pure.profile.while;pure.profile.while.armed;pure.profile.for;pure.profile.for.armed

producer_pauses: 0

observations: pure.profile.preparation:producer=0,total=2,queued=0,enters=0;pure.profile.preparation.armed:producer=0,total=2,queued=0,enters=0;pure.profile.head:producer=0,total=2,queued=0,enters=0;pure.profile.head.armed:producer=0,total=2,queued=0,enters=0;pure.profile.own:producer=0,total=1,queued=0,enters=0;pure.profile.own.armed:producer=0,total=1,queued=0,enters=0;pure.profile.down:producer=0,total=1,queued=0,enters=0;pure.profile.down.armed:producer=0,total=1,queued=0,enters=0;pure.profile.up:producer=0,total=1,queued=0,enters=0;pure.profile.up.armed:producer=0,total=1,queued=0,enters=0;pure.profile.sub:producer=0,total=1,queued=0,enters=0;pure.profile.sub.armed:producer=0,total=1,queued=0,enters=0;pure.profile.default:producer=0,total=1,queued=0,enters=0;pure.profile.default.armed:producer=0,total=1,queued=0,enters=0;pure.profile.lhs:producer=0,total=1,queued=0,enters=0;pure.profile.lhs.armed:producer=0,total=1,queued=0,enters=0;pure.profile.set:producer=0,total=2,queued=0,enters=0;pure.profile.set.armed:producer=0,total=2,queued=0,enters=0;pure.profile.with:producer=0,total=2,queued=0,enters=0;pure.profile.with.armed:producer=0,total=2,queued=0,enters=0;pure.profile.block:producer=0,total=2,queued=0,enters=0;pure.profile.block.armed:producer=0,total=2,queued=0,enters=0;pure.profile.module:producer=0,total=3,queued=0,enters=0;pure.profile.module.armed:producer=0,total=3,queued=0,enters=0;pure.profile.function:producer=0,total=1,queued=0,enters=0;pure.profile.function.armed:producer=0,total=1,queued=0,enters=0;pure.profile.catch:producer=0,total=2,queued=0,enters=0;pure.profile.catch.armed:producer=0,total=2,queued=0,enters=0;pure.profile.do:producer=0,total=2,queued=0,enters=0;pure.profile.do.armed:producer=0,total=2,queued=0,enters=0;pure.profile.while:producer=0,total=2,queued=0,enters=0;pure.profile.while.armed:producer=0,total=2,queued=0,enters=0;pure.profile.for:producer=0,total=2,queued=0,enters=0;pure.profile.for.armed:producer=0,total=2,queued=0,enters=0

entry_count: 0

**P1518 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:231**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:231 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=1;pure.profile.module.armed:producer=0,total=3,queued=0,enters=1

entry_count: 2

**P1519 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:251**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:251 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=1;pure.profile.module.armed:producer=0,total=3,queued=0,enters=1

entry_count: 2

**P1521 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:270**

Concrete call runtime_profile_outcome_for_expr_result (& init_result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:270 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=1;pure.profile.module.armed:producer=0,total=3,queued=0,enters=1

entry_count: 2

**P1522 — crates/rum-runtime/src/runtime/scoping/lexical/module.rs:278**

Concrete call runtime_profile_outcome_for_expr_result (& result) at crates/rum-runtime/src/runtime/scoping/lexical/module.rs:278 takes only an already produced borrowed Result. There is no guest expression evaluation in the argument expression. The applicable lexical helper(s) are crates/rum-runtime/src/runtime/definitions/application/mod.rs:106-110; crates/rum-runtime/src/runtime/definitions/definitions.rs:557-569; crates/rum-runtime/src/runtime/eval_call_prepare.rs:528-540; crates/rum-runtime/src/runtime/eval_core/mod.rs:248-252; crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:492-504; crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:352-364; crates/rum-runtime/src/runtime/resolution.rs:505-517; crates/rum-runtime/src/runtime/scoping/callable.rs:130-142; crates/rum-runtime/src/runtime/scoping/lexical/mod.rs:60-72. Each returns the closed RuntimeProfileOutcome enum by matching Ok/Err variants; it does not return Result, accept Session, invoke a callback, or construct an input request. Delegating helpers call only the pure runtime_profile_outcome_for_result match in the same module. An existing InputRequired error is labeled RuntimeProfileOutcome::InputRequired; it is not returned as an evaluator error. This edge is a false positive from the may-evaluate graph seeding on the InputRequired token, not a suspension-safe success tail.

probes: pure.profile.module;pure.profile.module.armed

producer_pauses: 0

observations: pure.profile.module:producer=0,total=3,queued=0,enters=1;pure.profile.module.armed:producer=0,total=3,queued=0,enters=1

entry_count: 2

**P1645 — crates/rum-stdlib/src/arithmetic.rs:27**

At crates/rum-stdlib/src/arithmetic.rs:27 the concrete dispatch argument is ArithmeticDataBuiltin::Identity; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1645.Identity.argument;arithmetic.P1645.Identity.inert

producer_pauses: 0

observations: arithmetic.P1645.Identity.argument:producer=0,total=1,enters=2;arithmetic.P1645.Identity.inert:producer=0,total=0,enters=2

entry_count: 4

**P1646 — crates/rum-stdlib/src/arithmetic.rs:35**

At crates/rum-stdlib/src/arithmetic.rs:35 the concrete dispatch argument is ArithmeticDataBuiltin::Plus; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1646.Plus.argument;arithmetic.P1646.Plus.inert

producer_pauses: 0

observations: arithmetic.P1646.Plus.argument:producer=0,total=1,enters=4;arithmetic.P1646.Plus.inert:producer=0,total=0,enters=6

entry_count: 112

**P1647 — crates/rum-stdlib/src/arithmetic.rs:43**

At crates/rum-stdlib/src/arithmetic.rs:43 the concrete dispatch argument is ArithmeticDataBuiltin::Times; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1647.Times.argument;arithmetic.P1647.Times.inert

producer_pauses: 0

observations: arithmetic.P1647.Times.argument:producer=0,total=1,enters=2;arithmetic.P1647.Times.inert:producer=0,total=0,enters=4

entry_count: 10

**P1648 — crates/rum-stdlib/src/arithmetic.rs:51**

At crates/rum-stdlib/src/arithmetic.rs:51 the concrete dispatch argument is ArithmeticDataBuiltin::Power; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1648.Power.argument;arithmetic.P1648.Power.inert

producer_pauses: 0

observations: arithmetic.P1648.Power.argument:producer=0,total=1,enters=2;arithmetic.P1648.Power.inert:producer=0,total=0,enters=2

entry_count: 4

**P1649 — crates/rum-stdlib/src/arithmetic.rs:59**

At crates/rum-stdlib/src/arithmetic.rs:59 the concrete dispatch argument is ArithmeticDataBuiltin::Subtract; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1649.Subtract.argument;arithmetic.P1649.Subtract.inert

producer_pauses: 0

observations: arithmetic.P1649.Subtract.argument:producer=0,total=1,enters=2;arithmetic.P1649.Subtract.inert:producer=0,total=0,enters=2

entry_count: 4

**P1650 — crates/rum-stdlib/src/arithmetic.rs:67**

At crates/rum-stdlib/src/arithmetic.rs:67 the concrete dispatch argument is ArithmeticDataBuiltin::Divide; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1650.Divide.argument;arithmetic.P1650.Divide.inert

producer_pauses: 0

observations: arithmetic.P1650.Divide.argument:producer=0,total=1,enters=2;arithmetic.P1650.Divide.inert:producer=0,total=0,enters=2

entry_count: 4

**P1651 — crates/rum-stdlib/src/arithmetic.rs:75**

At crates/rum-stdlib/src/arithmetic.rs:75 the concrete dispatch argument is ArithmeticDataBuiltin::Abs; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1651.Abs.argument;arithmetic.P1651.Abs.inert

producer_pauses: 0

observations: arithmetic.P1651.Abs.argument:producer=0,total=1,enters=2;arithmetic.P1651.Abs.inert:producer=0,total=0,enters=2

entry_count: 4

**P1652 — crates/rum-stdlib/src/arithmetic.rs:83**

At crates/rum-stdlib/src/arithmetic.rs:83 the concrete dispatch argument is ArithmeticDataBuiltin::Sign; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1652.Sign.argument;arithmetic.P1652.Sign.inert

producer_pauses: 0

observations: arithmetic.P1652.Sign.argument:producer=0,total=1,enters=2;arithmetic.P1652.Sign.inert:producer=0,total=0,enters=2

entry_count: 4

**P1653 — crates/rum-stdlib/src/arithmetic.rs:91**

At crates/rum-stdlib/src/arithmetic.rs:91 the concrete dispatch argument is ArithmeticDataBuiltin::Floor; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1653.Floor.argument;arithmetic.P1653.Floor.inert

producer_pauses: 0

observations: arithmetic.P1653.Floor.argument:producer=0,total=1,enters=2;arithmetic.P1653.Floor.inert:producer=0,total=0,enters=2

entry_count: 4

**P1654 — crates/rum-stdlib/src/arithmetic.rs:99**

At crates/rum-stdlib/src/arithmetic.rs:99 the concrete dispatch argument is ArithmeticDataBuiltin::Ceiling; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1654.Ceiling.argument;arithmetic.P1654.Ceiling.inert

producer_pauses: 0

observations: arithmetic.P1654.Ceiling.argument:producer=0,total=1,enters=2;arithmetic.P1654.Ceiling.inert:producer=0,total=0,enters=2

entry_count: 4

**P1655 — crates/rum-stdlib/src/arithmetic.rs:107**

At crates/rum-stdlib/src/arithmetic.rs:107 the concrete dispatch argument is ArithmeticDataBuiltin::IntegerPart; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1655.IntegerPart.argument;arithmetic.P1655.IntegerPart.inert

producer_pauses: 0

observations: arithmetic.P1655.IntegerPart.argument:producer=0,total=1,enters=2;arithmetic.P1655.IntegerPart.inert:producer=0,total=0,enters=2

entry_count: 4

**P1656 — crates/rum-stdlib/src/arithmetic.rs:115**

At crates/rum-stdlib/src/arithmetic.rs:115 the concrete dispatch argument is ArithmeticDataBuiltin::FractionalPart; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1656.FractionalPart.argument;arithmetic.P1656.FractionalPart.inert

producer_pauses: 0

observations: arithmetic.P1656.FractionalPart.argument:producer=0,total=1,enters=2;arithmetic.P1656.FractionalPart.inert:producer=0,total=0,enters=2

entry_count: 4

**P1657 — crates/rum-stdlib/src/arithmetic.rs:123**

At crates/rum-stdlib/src/arithmetic.rs:123 the concrete dispatch argument is ArithmeticDataBuiltin::Minus; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1657.Minus.argument;arithmetic.P1657.Minus.inert

producer_pauses: 0

observations: arithmetic.P1657.Minus.argument:producer=0,total=1,enters=2;arithmetic.P1657.Minus.inert:producer=0,total=0,enters=2

entry_count: 4

**P1658 — crates/rum-stdlib/src/arithmetic.rs:131**

At crates/rum-stdlib/src/arithmetic.rs:131 the concrete dispatch argument is ArithmeticDataBuiltin::Round; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1658.Round.argument;arithmetic.P1658.Round.inert

producer_pauses: 0

observations: arithmetic.P1658.Round.argument:producer=0,total=1,enters=2;arithmetic.P1658.Round.inert:producer=0,total=0,enters=2

entry_count: 4

**P1659 — crates/rum-stdlib/src/arithmetic.rs:139**

At crates/rum-stdlib/src/arithmetic.rs:139 the concrete dispatch argument is ArithmeticDataBuiltin::Factorial; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1659.Factorial.argument;arithmetic.P1659.Factorial.inert

producer_pauses: 0

observations: arithmetic.P1659.Factorial.argument:producer=0,total=1,enters=2;arithmetic.P1659.Factorial.inert:producer=0,total=0,enters=2

entry_count: 4

**P1660 — crates/rum-stdlib/src/arithmetic.rs:147**

At crates/rum-stdlib/src/arithmetic.rs:147 the concrete dispatch argument is ArithmeticDataBuiltin::Factorial2; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1660.Factorial2.argument;arithmetic.P1660.Factorial2.inert

producer_pauses: 0

observations: arithmetic.P1660.Factorial2.argument:producer=0,total=1,enters=2;arithmetic.P1660.Factorial2.inert:producer=0,total=0,enters=2

entry_count: 4

**P1663 — crates/rum-stdlib/src/arithmetic.rs:197**

At crates/rum-stdlib/src/arithmetic.rs:197 the concrete dispatch argument is ArithmeticDataBuiltin::Denominator; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1663.Denominator.argument;arithmetic.P1663.Denominator.inert

producer_pauses: 0

observations: arithmetic.P1663.Denominator.argument:producer=0,total=1,enters=2;arithmetic.P1663.Denominator.inert:producer=0,total=0,enters=2

entry_count: 4

**P1664 — crates/rum-stdlib/src/arithmetic.rs:205**

At crates/rum-stdlib/src/arithmetic.rs:205 the concrete dispatch argument is ArithmeticDataBuiltin::Numerator; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1664.Numerator.argument;arithmetic.P1664.Numerator.inert

producer_pauses: 0

observations: arithmetic.P1664.Numerator.argument:producer=0,total=1,enters=2;arithmetic.P1664.Numerator.inert:producer=0,total=0,enters=2

entry_count: 4

**P1665 — crates/rum-stdlib/src/arithmetic.rs:213**

At crates/rum-stdlib/src/arithmetic.rs:213 the concrete dispatch argument is ArithmeticDataBuiltin::Mod; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1665.Mod.argument;arithmetic.P1665.Mod.inert

producer_pauses: 0

observations: arithmetic.P1665.Mod.argument:producer=0,total=1,enters=2;arithmetic.P1665.Mod.inert:producer=0,total=0,enters=2

entry_count: 4

**P1666 — crates/rum-stdlib/src/arithmetic.rs:221**

At crates/rum-stdlib/src/arithmetic.rs:221 the concrete dispatch argument is ArithmeticDataBuiltin::Quotient; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1666.Quotient.argument;arithmetic.P1666.Quotient.inert

producer_pauses: 0

observations: arithmetic.P1666.Quotient.argument:producer=0,total=1,enters=2;arithmetic.P1666.Quotient.inert:producer=0,total=0,enters=2

entry_count: 4

**P1667 — crates/rum-stdlib/src/arithmetic.rs:229**

At crates/rum-stdlib/src/arithmetic.rs:229 the concrete dispatch argument is ArithmeticDataBuiltin::PowerMod; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1667.PowerMod.argument;arithmetic.P1667.PowerMod.inert

producer_pauses: 0

observations: arithmetic.P1667.PowerMod.argument:producer=0,total=1,enters=2;arithmetic.P1667.PowerMod.inert:producer=0,total=0,enters=2

entry_count: 4

**P1668 — crates/rum-stdlib/src/arithmetic.rs:237**

At crates/rum-stdlib/src/arithmetic.rs:237 the concrete dispatch argument is ArithmeticDataBuiltin::GCD; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1668.GCD.argument;arithmetic.P1668.GCD.inert

producer_pauses: 0

observations: arithmetic.P1668.GCD.argument:producer=0,total=1,enters=2;arithmetic.P1668.GCD.inert:producer=0,total=0,enters=2

entry_count: 4

**P1669 — crates/rum-stdlib/src/arithmetic.rs:245**

At crates/rum-stdlib/src/arithmetic.rs:245 the concrete dispatch argument is ArithmeticDataBuiltin::LCM; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1669.LCM.argument;arithmetic.P1669.LCM.inert

producer_pauses: 0

observations: arithmetic.P1669.LCM.argument:producer=0,total=1,enters=2;arithmetic.P1669.LCM.inert:producer=0,total=0,enters=2

entry_count: 4

**P1670 — crates/rum-stdlib/src/arithmetic.rs:253**

At crates/rum-stdlib/src/arithmetic.rs:253 the concrete dispatch argument is ArithmeticDataBuiltin::Max; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1670.Max.argument;arithmetic.P1670.Max.inert

producer_pauses: 0

observations: arithmetic.P1670.Max.argument:producer=0,total=1,enters=2;arithmetic.P1670.Max.inert:producer=0,total=0,enters=2

entry_count: 4

**P1671 — crates/rum-stdlib/src/arithmetic.rs:261**

At crates/rum-stdlib/src/arithmetic.rs:261 the concrete dispatch argument is ArithmeticDataBuiltin::Min; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1671.Min.argument;arithmetic.P1671.Min.inert

producer_pauses: 0

observations: arithmetic.P1671.Min.argument:producer=0,total=1,enters=2;arithmetic.P1671.Min.inert:producer=0,total=0,enters=2

entry_count: 4

**P1672 — crates/rum-stdlib/src/arithmetic.rs:269**

At crates/rum-stdlib/src/arithmetic.rs:269 the concrete dispatch argument is ArithmeticDataBuiltin::MinMax; this is not a dynamic arithmetic choice. dispatch at arithmetic.rs:13-20 forwards that constant to crates/rum-runtime/src/runtime/arithmetic/mod.rs:47-82. The selected non-Total arm and its closed helpers operate only on already-prepared Expr data. arithmetic/canonical.rs:8-389 constructs/canonicalizes arithmetic and recursively calls only eval_power for symbolic powers; rounding.rs:6-156, integer_ops.rs:5-227, factorial.rs:5-103, extremum.rs and parts.rs:25-52 perform exact arithmetic, shape tests and value/message construction. builtin_adapters.rs:6-16 implements Identity by cloning its sole argument. None calls guest evaluation, dispatches an arbitrary guest head, invokes a guest callback, or returns an incoming EvalError parameter. The only recursive arithmetic evaluator entry is Total at parts.rs:22; its enum arm is excluded by this call-site constant. Resource enforcers and invalid-call message helpers can report resource/arity failures but do not call guest code. Consequently this concrete producer cannot return InputRequired. Input in an ordinary argument pauses in argument preparation before dispatch, and a held InputString remains inert data in the handler.

probes: arithmetic.P1672.MinMax.argument;arithmetic.P1672.MinMax.inert

producer_pauses: 0

observations: arithmetic.P1672.MinMax.argument:producer=0,total=1,enters=4;arithmetic.P1672.MinMax.inert:producer=0,total=0,enters=2

entry_count: 6

**P1437 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:18**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:18 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1438 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:34**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:34 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1439 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:44**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:44 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1440 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:58**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:58 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1441 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:67**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:67 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1442 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:87**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:87 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1443 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:89**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:89 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1444 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:115**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:115 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1445 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:130**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:130 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1446 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:168**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:168 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1447 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:171**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:171 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1449 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:216**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:216 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1450 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:230**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:230 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1451 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:249**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:249 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1452 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:251**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:251 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1454 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:307**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:307 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1455 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:323**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:323 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1464 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:439**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:439 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1465 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:520**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:520 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1466 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:546**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:546 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1467 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:640**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:640 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1468 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:789**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:789 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P1469 — crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:915**

The concrete edge at crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:915 is in the interpreted-matcher island, not the production compiled matcher. Its two public-within-runtime roots are match_expr_interpreted_owned at :313 and match_expr_interpreted_inner_owned at :339; match_call_args_interpreted_owned is the internal :431 helper. All production references to those roots, all other interpreted methods, and all construction sites of the private interpreted frames are confined to this same module. The exhaustive lexical reference list in continuation2/interpreted-external-references.json has only runtime/tests/definitions_and_rules_part2.rs:45,83, behind the test module. The module functions are private or pub(super), and no builtin/native registration can invoke one from guest syntax. Frame resume methods cannot form an independent root: each private frame can only be constructed by an already-entered interpreted method. Thus no guest request can enter this concrete producer or make it return InputRequired, even though a Rust unit test calling the internal root can exercise its guest guards. The strongest guest attempts cover named, guarded, two-guard, PatternTest, Except, Alternatives, Optional, Flat/Orderless, KeyValuePattern and replacement routes; their real pauses belong to compiled matcher edges, with zero entries and zero pauses in every interpreted producer.

probes: interpreter-attempt.condition;interpreter-attempt.two-conditions;interpreter-attempt.pattern-test;interpreter-attempt.except;interpreter-attempt.alternatives;interpreter-attempt.named;interpreter-attempt.optional;interpreter-attempt.flat-orderless;interpreter-attempt.key-value;interpreter-attempt.replacement

producer_pauses: 0

observations: interpreter-attempt.condition:producer=0,total=1,enters=0;interpreter-attempt.two-conditions:producer=0,total=2,enters=0;interpreter-attempt.pattern-test:producer=0,total=1,enters=0;interpreter-attempt.except:producer=0,total=1,enters=0;interpreter-attempt.alternatives:producer=0,total=2,enters=0;interpreter-attempt.named:producer=0,total=1,enters=0;interpreter-attempt.optional:producer=0,total=0,enters=0;interpreter-attempt.flat-orderless:producer=0,total=1,enters=0;interpreter-attempt.key-value:producer=0,total=1,enters=0;interpreter-attempt.replacement:producer=0,total=1,enters=0

entry_count: 0

**P2181 — crates/rum-stdlib/src/option_consumers.rs:99**

The exact producer call crate :: strings :: eval_align_columns_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:99 supplies Some(options), not an arbitrary Option. The concrete callee is eval_align_columns_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2181.unarmed;options-some.P2181.armed

producer_pauses: 0

observations: options-some.P2181.unarmed:producer=0,total=1,enters=2;options-some.P2181.armed:producer=0,total=1,enters=2

entry_count: 4

**P2184 — crates/rum-stdlib/src/option_consumers.rs:147**

The exact producer call crate :: strings :: eval_html_unescape_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:147 supplies Some(options), not an arbitrary Option. The concrete callee is eval_html_unescape_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2184.unarmed;options-some.P2184.armed

producer_pauses: 0

observations: options-some.P2184.unarmed:producer=0,total=1,enters=2;options-some.P2184.armed:producer=0,total=1,enters=2

entry_count: 4

**P2186 — crates/rum-stdlib/src/option_consumers.rs:162**

The exact producer call crate :: strings :: eval_parse_args_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:162 supplies Some(options), not an arbitrary Option. The concrete callee is eval_parse_args_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2186.unarmed;options-some.P2186.armed

producer_pauses: 0

observations: options-some.P2186.unarmed:producer=0,total=1,enters=2;options-some.P2186.armed:producer=0,total=1,enters=2

entry_count: 4

**P2188 — crates/rum-stdlib/src/option_consumers.rs:167**

The exact producer call crate :: strings :: eval_string_contains_q_with_options (session , head , args , Some (options) ,) at crates/rum-stdlib/src/option_consumers.rs:167 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_contains_q_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2188.unarmed;options-some.P2188.armed

producer_pauses: 0

observations: options-some.P2188.unarmed:producer=0,total=1,enters=2;options-some.P2188.armed:producer=0,total=2,enters=2

entry_count: 4

**P2189 — crates/rum-stdlib/src/option_consumers.rs:174**

The exact producer call crate :: strings :: eval_string_count_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:174 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_count_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2189.unarmed;options-some.P2189.armed

producer_pauses: 0

observations: options-some.P2189.unarmed:producer=0,total=1,enters=2;options-some.P2189.armed:producer=0,total=2,enters=2

entry_count: 4

**P2190 — crates/rum-stdlib/src/option_consumers.rs:177**

The exact producer call crate :: strings :: eval_string_delete_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:177 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_delete_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2190.unarmed;options-some.P2190.armed

producer_pauses: 0

observations: options-some.P2190.unarmed:producer=0,total=1,enters=2;options-some.P2190.armed:producer=0,total=2,enters=2

entry_count: 4

**P2191 — crates/rum-stdlib/src/option_consumers.rs:180**

The exact producer call crate :: strings :: eval_string_ends_q_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:180 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_ends_q_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2191.unarmed;options-some.P2191.armed

producer_pauses: 0

observations: options-some.P2191.unarmed:producer=0,total=1,enters=2;options-some.P2191.armed:producer=0,total=2,enters=2

entry_count: 4

**P2192 — crates/rum-stdlib/src/option_consumers.rs:183**

The exact producer call crate :: strings :: eval_string_free_q_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:183 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_free_q_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2192.unarmed;options-some.P2192.armed

producer_pauses: 0

observations: options-some.P2192.unarmed:producer=0,total=1,enters=2;options-some.P2192.armed:producer=0,total=2,enters=2

entry_count: 4

**P2193 — crates/rum-stdlib/src/option_consumers.rs:186**

The exact producer call crate :: strings :: eval_string_indent_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:186 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_indent_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2193.unarmed;options-some.P2193.armed

producer_pauses: 0

observations: options-some.P2193.unarmed:producer=0,total=1,enters=2;options-some.P2193.armed:producer=0,total=1,enters=2

entry_count: 4

**P2194 — crates/rum-stdlib/src/option_consumers.rs:189**

The exact producer call crate :: strings :: eval_string_match_q_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:189 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_match_q_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2194.unarmed;options-some.P2194.armed

producer_pauses: 0

observations: options-some.P2194.unarmed:producer=0,total=1,enters=2;options-some.P2194.armed:producer=0,total=2,enters=2

entry_count: 4

**P2195 — crates/rum-stdlib/src/option_consumers.rs:191**

The exact producer call crate :: strings :: eval_string_position_with_options (session , head , args , Some (options) ,) at crates/rum-stdlib/src/option_consumers.rs:191 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_position_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2195.unarmed;options-some.P2195.armed

producer_pauses: 0

observations: options-some.P2195.unarmed:producer=0,total=1,enters=2;options-some.P2195.armed:producer=0,total=2,enters=2

entry_count: 4

**P2197 — crates/rum-stdlib/src/option_consumers.rs:201**

The exact producer call crate :: strings :: eval_string_split_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:201 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_split_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2197.unarmed;options-some.P2197.armed

producer_pauses: 0

observations: options-some.P2197.unarmed:producer=0,total=1,enters=2;options-some.P2197.armed:producer=0,total=2,enters=2

entry_count: 4

**P2198 — crates/rum-stdlib/src/option_consumers.rs:203**

The exact producer call crate :: strings :: eval_string_starts_q_with_options (session , head , args , Some (options) ,) at crates/rum-stdlib/src/option_consumers.rs:203 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_starts_q_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2198.unarmed;options-some.P2198.armed

producer_pauses: 0

observations: options-some.P2198.unarmed:producer=0,total=1,enters=2;options-some.P2198.armed:producer=0,total=2,enters=2

entry_count: 4

**P2199 — crates/rum-stdlib/src/option_consumers.rs:210**

The exact producer call crate :: strings :: eval_string_wrap_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:210 supplies Some(options), not an arbitrary Option. The concrete callee is eval_string_wrap_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2199.unarmed;options-some.P2199.armed

producer_pauses: 0

observations: options-some.P2199.unarmed:producer=0,total=1,enters=2;options-some.P2199.armed:producer=0,total=1,enters=2

entry_count: 4

**P2200 — crates/rum-stdlib/src/option_consumers.rs:216**

The exact producer call crate :: strings :: eval_template_apply_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:216 supplies Some(options), not an arbitrary Option. The concrete callee is eval_template_apply_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2200.unarmed;options-some.P2200.armed

producer_pauses: 0

observations: options-some.P2200.unarmed:producer=0,total=1,enters=2;options-some.P2200.armed:producer=0,total=1,enters=2

entry_count: 4

**P2203 — crates/rum-stdlib/src/option_consumers.rs:225**

The exact producer call crate :: strings :: eval_url_decode_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:225 supplies Some(options), not an arbitrary Option. The concrete callee is eval_url_decode_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2203.unarmed;options-some.P2203.armed

producer_pauses: 0

observations: options-some.P2203.unarmed:producer=0,total=1,enters=2;options-some.P2203.armed:producer=0,total=1,enters=2

entry_count: 4

**P2204 — crates/rum-stdlib/src/option_consumers.rs:228**

The exact producer call crate :: strings :: eval_url_encode_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:228 supplies Some(options), not an arbitrary Option. The concrete callee is eval_url_encode_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2204.unarmed;options-some.P2204.armed

producer_pauses: 0

observations: options-some.P2204.unarmed:producer=0,total=1,enters=2;options-some.P2204.armed:producer=0,total=1,enters=2

entry_count: 4

**P2205 — crates/rum-stdlib/src/option_consumers.rs:231**

The exact producer call crate :: strings :: eval_url_parse_with_options (session , head , args , Some (options)) at crates/rum-stdlib/src/option_consumers.rs:231 supplies Some(options), not an arbitrary Option. The concrete callee is eval_url_parse_with_options; its Some branch bypasses merged_option_exprs_for_system_symbol. RetainedOptions is a read-only &[Expr] view, and its IntoIterator is std::slice::Iter (rum-runtime/src/runtime/accounting_mode/options.rs:41-59); neither access nor iteration evaluates delayed RHS expressions. The selected string consumers use only validated string/number/list data and pure option parsing. The relevant closed implementation branches are strings/scripting/text_shape.rs:13-114,172-208; scripting/html.rs:49-96; scripting/args.rs:13-62; scripting/url.rs:15-140; template.rs:57-138; patterns/search.rs:33-105,108-215,506-612,646-789. The pattern predicate function pointers are fixed by their wrapper and accept only character slices, StringPattern and a match configuration; they have no Session or guest callback. StringCount/Delete/Position/Split use the same pure string matcher. The guest-rule-evaluating StringCases and StringReplace arms are separate and are not part of this proof. TemplateApply renders validated template slots as strings without evaluating slot expressions. URL and ParseArgs resource checks inspect sizes and return resource errors only. All invalid-call/reporting paths build messages/values without guest evaluation. Consequently this concrete Some(options) producer cannot return InputRequired. The strongest armed and unarmed delayed-option attempts pause in the preceding option-evaluation phase, then enter this exact producer with a resolved view and no producer pause.

probes: options-some.P2205.unarmed;options-some.P2205.armed

producer_pauses: 0

observations: options-some.P2205.unarmed:producer=0,total=1,enters=2;options-some.P2205.armed:producer=0,total=1,enters=2

entry_count: 4

#### Strongest Unreachable attempt sources

```json
{
  "id": "arithmetic.P1645.Identity.argument",
  "source": "Identity[(InputString[\"arg\"];3)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1645.Identity.inert",
  "source": "Identity[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1646.Plus.argument",
  "source": "Plus[(InputString[\"arg\"];3),4]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1646.Plus.inert",
  "source": "Plus[Hold[InputString[\"inert\"]],4]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1647.Times.argument",
  "source": "Times[(InputString[\"arg\"];3),4]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1647.Times.inert",
  "source": "Times[Hold[InputString[\"inert\"]],4]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1648.Power.argument",
  "source": "Power[(InputString[\"arg\"];3),2]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1648.Power.inert",
  "source": "Power[Hold[InputString[\"inert\"]],2]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1649.Subtract.argument",
  "source": "Subtract[(InputString[\"arg\"];3),2]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1649.Subtract.inert",
  "source": "Subtract[Hold[InputString[\"inert\"]],2]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1650.Divide.argument",
  "source": "Divide[(InputString[\"arg\"];3),2]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1650.Divide.inert",
  "source": "Divide[Hold[InputString[\"inert\"]],2]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1651.Abs.argument",
  "source": "Abs[(InputString[\"arg\"];-3)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1651.Abs.inert",
  "source": "Abs[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1652.Sign.argument",
  "source": "Sign[(InputString[\"arg\"];-3)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1652.Sign.inert",
  "source": "Sign[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1653.Floor.argument",
  "source": "Floor[(InputString[\"arg\"];3/2)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1653.Floor.inert",
  "source": "Floor[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1654.Ceiling.argument",
  "source": "Ceiling[(InputString[\"arg\"];3/2)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1654.Ceiling.inert",
  "source": "Ceiling[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1655.IntegerPart.argument",
  "source": "IntegerPart[(InputString[\"arg\"];3/2)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1655.IntegerPart.inert",
  "source": "IntegerPart[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1656.FractionalPart.argument",
  "source": "FractionalPart[(InputString[\"arg\"];3/2)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1656.FractionalPart.inert",
  "source": "FractionalPart[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1657.Minus.argument",
  "source": "Minus[(InputString[\"arg\"];3)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1657.Minus.inert",
  "source": "Minus[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1658.Round.argument",
  "source": "Round[(InputString[\"arg\"];3/2)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1658.Round.inert",
  "source": "Round[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1659.Factorial.argument",
  "source": "Factorial[(InputString[\"arg\"];3)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1659.Factorial.inert",
  "source": "Factorial[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1660.Factorial2.argument",
  "source": "Factorial2[(InputString[\"arg\"];3)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1660.Factorial2.inert",
  "source": "Factorial2[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1663.Denominator.argument",
  "source": "Denominator[(InputString[\"arg\"];3/2)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1663.Denominator.inert",
  "source": "Denominator[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1664.Numerator.argument",
  "source": "Numerator[(InputString[\"arg\"];3/2)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1664.Numerator.inert",
  "source": "Numerator[Hold[InputString[\"inert\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1665.Mod.argument",
  "source": "Mod[(InputString[\"arg\"];3),2]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1665.Mod.inert",
  "source": "Mod[Hold[InputString[\"inert\"]],2]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1666.Quotient.argument",
  "source": "Quotient[(InputString[\"arg\"];3),2]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1666.Quotient.inert",
  "source": "Quotient[Hold[InputString[\"inert\"]],2]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1667.PowerMod.argument",
  "source": "PowerMod[(InputString[\"arg\"];3),2,5]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1667.PowerMod.inert",
  "source": "PowerMod[Hold[InputString[\"inert\"]],2,5]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1668.GCD.argument",
  "source": "GCD[(InputString[\"arg\"];3),6]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1668.GCD.inert",
  "source": "GCD[Hold[InputString[\"inert\"]],6]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1669.LCM.argument",
  "source": "LCM[(InputString[\"arg\"];3),6]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1669.LCM.inert",
  "source": "LCM[Hold[InputString[\"inert\"]],6]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1670.Max.argument",
  "source": "Max[(InputString[\"arg\"];3),6]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1670.Max.inert",
  "source": "Max[Hold[InputString[\"inert\"]],6]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1671.Min.argument",
  "source": "Min[(InputString[\"arg\"];3),6]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1671.Min.inert",
  "source": "Min[Hold[InputString[\"inert\"]],6]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1672.MinMax.argument",
  "source": "MinMax[(InputString[\"arg\"];3),6]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "arithmetic.P1672.MinMax.inert",
  "source": "MinMax[Hold[InputString[\"inert\"]],6]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.alternatives",
  "source": "MatchQ[1,Alternatives[x_ /; (InputString[\"p\"];False),y_ /; (InputString[\"q\"];True)]]",
  "setup": "",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.condition",
  "source": "MatchQ[1,x_ /; (InputString[\"p\"];True)]",
  "setup": "",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.except",
  "source": "MatchQ[1,Except[x_ /; (InputString[\"p\"];False)]]",
  "setup": "",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.flat-orderless",
  "source": "MatchQ[h[1,2],h[x_ /; (InputString[\"p\"];True),y_]]",
  "setup": "SetAttributes[h,{Flat,Orderless,OneIdentity}]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.key-value",
  "source": "MatchQ[Association[\"a\"->1],KeyValuePattern[\"a\"->(x_ /; (InputString[\"p\"];True))]]",
  "setup": "",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.named",
  "source": "MatchQ[h[1],whole:h[x_ /; (InputString[\"p\"];True)]]",
  "setup": "",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.optional",
  "source": "MatchQ[h[],h[x_. /; (InputString[\"p\"];True)]]",
  "setup": "Default[h]=0",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.pattern-test",
  "source": "MatchQ[1,_?pt]",
  "setup": "pt[x_]:=(InputString[\"p\"];True)",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.replacement",
  "source": "ReplaceAll[h[1],(x_Integer /; (InputString[\"p\"];True)):>x]",
  "setup": "",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "interpreter-attempt.two-conditions",
  "source": "MatchQ[h[1,2],h[x_ /; (InputString[\"p\"];True),y_ /; (InputString[\"q\"];True)]]",
  "setup": "",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2181.armed",
  "source": "MemoryConstrained[(AlignColumns[{{\"a\",\"b\"},{\"cc\",\"d\"}},\"Separator\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];\" | \")]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2181.unarmed",
  "source": "AlignColumns[{{\"a\",\"b\"},{\"cc\",\"d\"}},\"Separator\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];\" | \")]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2184.armed",
  "source": "MemoryConstrained[(HtmlUnescape[\"&amp;\",\"Strict\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2184.unarmed",
  "source": "HtmlUnescape[\"&amp;\",\"Strict\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2186.armed",
  "source": "MemoryConstrained[(ParseArgs[{\"--flag\",\"tail\"},\"ShortOptions\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2186.unarmed",
  "source": "ParseArgs[{\"--flag\",\"tail\"},\"ShortOptions\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2188.armed",
  "source": "MemoryConstrained[(StringContainsQ[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2188.armed.documented-symbol",
  "source": "MemoryConstrained[(StringContainsQ[\"aba\",\"a\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2188.unarmed",
  "source": "StringContainsQ[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2189.armed",
  "source": "MemoryConstrained[(StringCount[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2189.armed.documented-symbol",
  "source": "MemoryConstrained[(StringCount[\"aba\",\"a\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2189.unarmed",
  "source": "StringCount[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2190.armed",
  "source": "MemoryConstrained[(StringDelete[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2190.armed.documented-symbol",
  "source": "MemoryConstrained[(StringDelete[\"aba\",\"a\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2190.unarmed",
  "source": "StringDelete[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2191.armed",
  "source": "MemoryConstrained[(StringEndsQ[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2191.armed.documented-symbol",
  "source": "MemoryConstrained[(StringEndsQ[\"aba\",\"a\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2191.unarmed",
  "source": "StringEndsQ[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2192.armed",
  "source": "MemoryConstrained[(StringFreeQ[\"aba\",\"z\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2192.armed.documented-symbol",
  "source": "MemoryConstrained[(StringFreeQ[\"aba\",\"z\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2192.unarmed",
  "source": "StringFreeQ[\"aba\",\"z\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2193.armed",
  "source": "MemoryConstrained[(StringIndent[\"a\\nb\",\"  \",\"BlankLines\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2193.unarmed",
  "source": "StringIndent[\"a\\nb\",\"  \",\"BlankLines\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2194.armed",
  "source": "MemoryConstrained[(StringMatchQ[\"aba\",\"aba\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2194.armed.documented-symbol",
  "source": "MemoryConstrained[(StringMatchQ[\"aba\",\"aba\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2194.unarmed",
  "source": "StringMatchQ[\"aba\",\"aba\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2195.armed",
  "source": "MemoryConstrained[(StringPosition[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2195.armed.documented-symbol",
  "source": "MemoryConstrained[(StringPosition[\"aba\",\"a\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2195.unarmed",
  "source": "StringPosition[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2197.armed",
  "source": "MemoryConstrained[(StringSplit[\"aba\",\"b\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2197.armed.documented-symbol",
  "source": "MemoryConstrained[(StringSplit[\"aba\",\"b\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2197.unarmed",
  "source": "StringSplit[\"aba\",\"b\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2198.armed",
  "source": "MemoryConstrained[(StringStartsQ[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2198.armed.documented-symbol",
  "source": "MemoryConstrained[(StringStartsQ[\"aba\",\"a\",IgnoreCase:>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2198.unarmed",
  "source": "StringStartsQ[\"aba\",\"a\",\"IgnoreCase\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2199.armed",
  "source": "MemoryConstrained[(StringWrap[\"a b c\",3,\"PreserveNewlines\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2199.unarmed",
  "source": "StringWrap[\"a b c\",3,\"PreserveNewlines\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2200.armed",
  "source": "MemoryConstrained[(TemplateApply[\"hello `name`\",Association[\"name\"->\"world\"],\"Strict\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2200.unarmed",
  "source": "TemplateApply[\"hello `name`\",Association[\"name\"->\"world\"],\"Strict\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2203.armed",
  "source": "MemoryConstrained[(UrlDecode[\"a%20b\",\"FormEncoding\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2203.unarmed",
  "source": "UrlDecode[\"a%20b\",\"FormEncoding\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2204.armed",
  "source": "MemoryConstrained[(UrlEncode[\"a b\",\"FormEncoding\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2204.unarmed",
  "source": "UrlEncode[\"a b\",\"FormEncoding\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];False)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2205.armed",
  "source": "MemoryConstrained[(UrlParse[\"https://example.test/a?q=1\",\"DecodeQuery\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "options-some.P2205.unarmed",
  "source": "UrlParse[\"https://example.test/a?q=1\",\"DecodeQuery\":>(AppendTo[log,\"before\"];InputString[\"option\"];AppendTo[log,\"after\"];True)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ]
}
```

```json
{
  "id": "pure.profile.block",
  "setup": "",
  "source": "Block[{x=(InputString[\"p\"];3)},InputString[\"q\"];x]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.block.armed",
  "setup": "",
  "source": "MemoryConstrained[(Block[{x=(InputString[\"p\"];3)},InputString[\"q\"];x]),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.catch",
  "setup": "",
  "source": "Catch[(InputString[\"p\"];Throw[3,t]),(InputString[\"q\"];t)]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.catch.armed",
  "setup": "",
  "source": "MemoryConstrained[(Catch[(InputString[\"p\"];Throw[3,t]),(InputString[\"q\"];t)]),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.default",
  "setup": "Default[Global`f] := (InputString[\"p\"];3);Global`f[x_.]:=x",
  "source": "Global`f[]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.default.armed",
  "setup": "Default[Global`f] := (InputString[\"p\"];3);Global`f[x_.]:=x",
  "source": "MemoryConstrained[(Global`f[]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.do",
  "setup": "",
  "source": "Do[InputString[\"p\"],{2}]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.do.armed",
  "setup": "",
  "source": "MemoryConstrained[(Do[InputString[\"p\"],{2}]),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.down",
  "setup": "Global`f[x_] := (InputString[\"p\"];x+1)",
  "source": "Global`f[2]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.down.armed",
  "setup": "Global`f[x_] := (InputString[\"p\"];x+1)",
  "source": "MemoryConstrained[(Global`f[2]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.for",
  "setup": "",
  "source": "For[i=0,i<2,i++,InputString[\"p\"]]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.for.armed",
  "setup": "",
  "source": "MemoryConstrained[(For[i=0,i<2,i++,InputString[\"p\"]]),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.function",
  "setup": "",
  "source": "Function[(InputString[\"p\"];Return[3])][]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.function.armed",
  "setup": "",
  "source": "MemoryConstrained[(Function[(InputString[\"p\"];Return[3])][]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.head",
  "setup": "",
  "source": "(InputString[\"h\"];Global`foo)[(InputString[\"p\"];4)]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.head.armed",
  "setup": "",
  "source": "MemoryConstrained[((InputString[\"h\"];Global`foo)[(InputString[\"p\"];4)]),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.lhs",
  "setup": "Global`x := (InputString[\"p\"];1)",
  "source": "Global`f[Global`x]=3",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.lhs.armed",
  "setup": "Global`x := (InputString[\"p\"];1)",
  "source": "MemoryConstrained[(Global`f[Global`x]=3),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.module",
  "setup": "",
  "source": "Module[{x=(InputString[\"p\"];3),y=(InputString[\"q\"];4)},InputString[\"r\"];x+y]",
  "inputs": [
    "1",
    "1",
    "1"
  ],
  "expected_pauses": 3,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.module.armed",
  "setup": "",
  "source": "MemoryConstrained[(Module[{x=(InputString[\"p\"];3),y=(InputString[\"q\"];4)},InputString[\"r\"];x+y]),1000000000]",
  "inputs": [
    "1",
    "1",
    "1"
  ],
  "expected_pauses": 3,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.own",
  "setup": "Global`x := (InputString[\"p\"];3)",
  "source": "Global`x",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.own.armed",
  "setup": "Global`x := (InputString[\"p\"];3)",
  "source": "MemoryConstrained[(Global`x),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.preparation",
  "setup": "",
  "source": "Global`h[(InputString[\"p\"];3),(InputString[\"q\"];4)]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.preparation.armed",
  "setup": "",
  "source": "MemoryConstrained[(Global`h[(InputString[\"p\"];3),(InputString[\"q\"];4)]),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.set",
  "setup": "",
  "source": "Global`f[(InputString[\"lhs\"];1)] = (InputString[\"rhs\"];3)",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.set.armed",
  "setup": "",
  "source": "MemoryConstrained[(Global`f[(InputString[\"lhs\"];1)] = (InputString[\"rhs\"];3)),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.sub",
  "setup": "Global`f[x_][y_] := (InputString[\"p\"];x+y)",
  "source": "Global`f[1][2]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.sub.armed",
  "setup": "Global`f[x_][y_] := (InputString[\"p\"];x+y)",
  "source": "MemoryConstrained[(Global`f[1][2]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.up",
  "setup": "Global`x /: Global`g[Global`x] := (InputString[\"p\"];3)",
  "source": "Global`g[Global`x]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.up.armed",
  "setup": "Global`x /: Global`g[Global`x] := (InputString[\"p\"];3)",
  "source": "MemoryConstrained[(Global`g[Global`x]),1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.while",
  "setup": "",
  "source": "(i=0;While[i<2,InputString[\"p\"];i++];i)",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.while.armed",
  "setup": "",
  "source": "MemoryConstrained[((i=0;While[i<2,InputString[\"p\"];i++];i)),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.with",
  "setup": "",
  "source": "With[{x=(InputString[\"p\"];3)},InputString[\"q\"];x]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "pure.profile.with.armed",
  "setup": "",
  "source": "MemoryConstrained[(With[{x=(InputString[\"p\"];3)},InputString[\"q\"];x]),1000000000]",
  "inputs": [
    "1",
    "1"
  ],
  "expected_pauses": 2,
  "projections": [
    "log"
  ],
  "trace": false
}
```

```json
{
  "id": "unreachable621.clone.armed",
  "source": "MemoryConstrained[Union[{},SameTest->Function[{x,y},InputString[\"clone-try\"]]],1000000000]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ],
  "trace": true
}
```

```json
{
  "id": "unreachable621.clone.unarmed",
  "source": "Union[{},SameTest->Function[{x,y},InputString[\"clone-try\"]]]",
  "inputs": [],
  "expected_pauses": 0,
  "projections": [
    "log"
  ],
  "trace": true
}
```

```json
{
  "id": "unreachable621.driver.non_delayed",
  "source": "MemoryConstrained[Union[{1,2},SameTest->Function[{x,y},(InputString[\"consumer-try\"];False)]],1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": true
}
```

```json
{
  "id": "unreachable621.rule.armed",
  "source": "MemoryConstrained[Union[{1,2},SameTest:>(InputString[\"rule-try\"];SameQ)],1000000000]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": true
}
```

```json
{
  "id": "unreachable621.rule.unarmed",
  "source": "Union[{1,2},SameTest:>(InputString[\"rule-try\"];SameQ)]",
  "inputs": [
    "1"
  ],
  "expected_pauses": 1,
  "projections": [
    "log"
  ],
  "trace": true
}
```

Zero-pause attempts are not Equal. The original pure arithmetic panel intentionally reports 26 invalid zero-pause inert-data attempts; its other 26 pause in argument preparation and all 26 designated handlers were entered without a producer pause. The interpreted guest attempts enter compiled routes and never the private interpreted island; the exhaustive outside references are only `runtime/tests/definitions_and_rules_part2.rs:45,83`. The nine corrected IgnoreCase controls improve the option attempts without changing their concrete Some(options) pure-call proofs.

#### Complete parked list and evidence index

Every ID below has its own explicit record in `continuation2/parked.tsv`, including attempts, source policy, observed contexts/variants, coarse gaps, sample probe IDs and recommendation. `settled.tsv` carries the same disposition and evidence link. A row with no exact observed route is not assigned Unreachable from that absence. Direct CLI/REPL/Jupyter entry-route probes were not run, and their records say so.

```text
P0001 P0002 P0004 P0005 P0006 P0007 P0008 P0010 P0011 P0012 P0014 P0015 P0016 P0017 P0018 P0019 P0020 P0021 P0022 P0025 P0026 P0027 P0028 P0030 P0031 P0032 P0033 P0034 P0035 P0036 P0037 P0038 P0039 P0043 P0044 P0046 P0049 P0050 P0051 P0052 P0053 P0054 P0055 P0056 P0057 P0058 P0059 P0060 P0061 P0062 P0063 P0064 P0065 P0066 P0067 P0068 P0069 P0070 P0071 P0072 P0073 P0074 P0075 P0076 P0077 P0078 P0079 P0080 P0081 P0082 P0083 P0084 P0085 P0086 P0087 P0088 P0089 P0090 P0091 P0092 P0093 P0094 P0095 P0096 P0097 P0098 P0099 P0100 P0101 P0102 P0103 P0104 P0105 P0106 P0107 P0108 P0109 P0110 P0111 P0112 P0113 P0114 P0115 P0116 P0117 P0119 P0121 P0122 P0123 P0125 P0126 P0127 P0128 P0134 P0135 P0137 P0143 P0144 P0145 P0146 P0147 P0148 P0149 P0151 P0153 P0154 P0155 P0156 P0157 P0158 P0159 P0160 P0161 P0162 P0163 P0164 P0166 P0168 P0170 P0172 P0173 P0174 P0175 P0176 P0177 P0179 P0180 P0181 P0183 P0184 P0186 P0187 P0188 P0189 P0190 P0191 P0192 P0193 P0195 P0196 P0197 P0198 P0200 P0201 P0203 P0204 P0205 P0209 P0210 P0213 P0214 P0216 P0217 P0218 P0219 P0220 P0223 P0224 P0229 P0231 P0232 P0233 P0234 P0235 P0236 P0237 P0238 P0239 P0240 P0241 P0242 P0243 P0244 P0245 P0246 P0247 P0248 P0249 P0251 P0252 P0253 P0254 P0255 P0256 P0257 P0258 P0259 P0260 P0262 P0263 P0264 P0265 P0266 P0267 P0270 P0271 P0272 P0273 P0274 P0275 P0276 P0277 P0278 P0281 P0282 P0283 P0284 P0285 P0286 P0287 P0289 P0290 P0292 P0294 P0296 P0298 P0299 P0301 P0302 P0303 P0304 P0305 P0307 P0308 P0309 P0310 P0311 P0312 P0313 P0314 P0315 P0316 P0317 P0318 P0319 P0320 P0321 P0322 P0323 P0325 P0326 P0327 P0328 P0329 P0330 P0331 P0332 P0333 P0334 P0335 P0336 P0338 P0339 P0340 P0341 P0342 P0343 P0344 P0345 P0346 P0347 P0348 P0349 P0350 P0351 P0353 P0354 P0355 P0357 P0358 P0359 P0361 P0363 P0365 P0366 P0367 P0369 P0372 P0376 P0377 P0378 P0379 P0380 P0381 P0382 P0388 P0394 P0395 P0396 P0397 P0398 P0399 P0400 P0401 P0402 P0403 P0404 P0405 P0406 P0407 P0408 P0410 P0413 P0416 P0417 P0418 P0419 P0420 P0423 P0425 P0426 P0428 P0430 P0431 P0433 P0434 P0435 P0436 P0437 P0438 P0439 P0440 P0441 P0442 P0445 P0447 P0448 P0450 P0451 P0452 P0453 P0454 P0455 P0456 P0457 P0458 P0459 P0460 P0461 P0462 P0463 P0464 P0465 P0466 P0467 P0468 P0470 P0471 P0472 P0473 P0474 P0475 P0478 P0479 P0481 P0484 P0485 P0487 P0489 P0492 P0494 P0496 P0498 P0500 P0501 P0502 P0503 P0504 P0505 P0506 P0507 P0508 P0509 P0511 P0515 P0519 P0520 P0521 P0522 P0523 P0524 P0525 P0526 P0527 P0528 P0529 P0530 P0531 P0532 P0533 P0534 P0535 P0536 P0537 P0538 P0539 P0540 P0541 P0542 P0543 P0544 P0545 P0546 P0547 P0548 P0549 P0550 P0551 P0552 P0553 P0554 P0555 P0556 P0557 P0558 P0559 P0560 P0561 P0563 P0566 P0567 P0568 P0569 P0570 P0571 P0573 P0574 P0575 P0576 P0578 P0579 P0580 P0581 P0582 P0584 P0585 P0586 P0587 P0588 P0589 P0590 P0591 P0592 P0593 P0594 P0595 P0596 P0597 P0598 P0599 P0600 P0601 P0602 P0603 P0604 P0605 P0606 P0607 P0608 P0609 P0610 P0611 P0612 P0613 P0614 P0615 P0616 P0617 P0619 P0620 P0621 P0622 P0623 P0624 P0625 P0626 P0627 P0628 P0629 P0630 P0631 P0632 P0633 P0634 P0635 P0636 P0637 P0638 P0639 P0640 P0641 P0642 P0643 P0644 P0645 P0646 P0647 P0648 P0649 P0650 P0651 P0654 P0657 P0662 P0665 P0668 P0669 P0670 P0671 P0672 P0673 P0677 P0678 P0680 P0683 P0684 P0685 P0687 P0691 P0693 P0694 P0695 P0696 P0697 P0699 P0703 P0704 P0705 P0706 P0707 P0708 P0709 P0710 P0711 P0712 P0713 P0714 P0715 P0716 P0717 P0718 P0719 P0720 P0721 P0722 P0723 P0724 P0725 P0726 P0728 P0730 P0732 P0734 P0736 P0738 P0740 P0741 P0743 P0744 P0745 P0746 P0747 P0749 P0750 P0751 P0752 P0754 P0756 P0757 P0758 P0759 P0760 P0761 P0763 P0764 P0765 P0767 P0768 P0769 P0770 P0771 P0772 P0773 P0774 P0775 P0776 P0777 P0790 P0795 P0798 P0801 P0810 P0812 P0813 P0814 P0817 P0818 P0819 P0820 P0821 P0822 P0823 P0824 P0825 P0826 P0827 P0828 P0829 P0830 P0831 P0832 P0833 P0834 P0836 P0837 P0838 P0839 P0840 P0841 P0842 P0843 P0844 P0845 P0846 P0847 P0849 P0850 P0852 P0853 P0854 P0855 P0856 P0857 P0858 P0859 P0861 P0862 P0863 P0864 P0868 P0869 P0870 P0873 P0874 P0875 P0876 P0877 P0878 P0879 P0880 P0881 P0882 P0885 P0887 P0888 P0890 P0891 P0892 P0893 P0894 P0895 P0896 P0897 P0898 P0899 P0904 P0905 P0906 P0908 P0909 P0910 P0911 P0912 P0913 P0914 P0916 P0917 P0918 P0919 P0920 P0922 P0923 P0924 P0927 P0929 P0930 P0931 P0932 P0933 P0934 P0936 P0937 P0938 P0942 P0943 P0947 P0948 P0949 P0950 P0951 P0952 P0953 P0954 P0955 P0956 P0957 P0960 P0965 P0967 P0969 P0971 P0972 P0973 P0975 P0976 P0978 P0980 P0981 P0982 P0983 P0984 P0985 P0986 P0987 P0988 P0989 P0990 P0991 P0992 P0993 P0994 P0995 P0996 P0997 P0998 P1000 P1002 P1003 P1004 P1005 P1006 P1007 P1008 P1009 P1010 P1011 P1014 P1015 P1017 P1020 P1023 P1024 P1025 P1026 P1027 P1028 P1029 P1030 P1031 P1034 P1035 P1036 P1037 P1038 P1039 P1040 P1041 P1044 P1045 P1046 P1047 P1057 P1058 P1059 P1060 P1061 P1062 P1065 P1069 P1070 P1073 P1074 P1077 P1078 P1079 P1080 P1081 P1082 P1085 P1086 P1087 P1088 P1089 P1090 P1091 P1092 P1093 P1094 P1095 P1096 P1097 P1098 P1099 P1100 P1101 P1102 P1103 P1105 P1107 P1108 P1109 P1110 P1113 P1114 P1115 P1116 P1117 P1122 P1124 P1125 P1128 P1130 P1132 P1133 P1135 P1136 P1137 P1138 P1139 P1140 P1141 P1142 P1144 P1145 P1146 P1147 P1149 P1150 P1151 P1152 P1154 P1155 P1157 P1158 P1159 P1160 P1164 P1165 P1169 P1170 P1172 P1173 P1176 P1177 P1178 P1179 P1180 P1181 P1182 P1183 P1184 P1185 P1186 P1187 P1188 P1189 P1190 P1191 P1193 P1195 P1196 P1197 P1198 P1199 P1200 P1201 P1202 P1203 P1214 P1215 P1216 P1217 P1228 P1229 P1231 P1232 P1238 P1239 P1240 P1242 P1244 P1245 P1246 P1250 P1251 P1254 P1255 P1256 P1257 P1258 P1259 P1261 P1262 P1263 P1264 P1265 P1266 P1267 P1269 P1270 P1271 P1272 P1274 P1275 P1278 P1279 P1280 P1281 P1282 P1283 P1284 P1285 P1286 P1287 P1288 P1289 P1290 P1293 P1294 P1296 P1297 P1299 P1300 P1301 P1302 P1303 P1304 P1305 P1306 P1307 P1308 P1310 P1312 P1313 P1314 P1315 P1316 P1317 P1318 P1319 P1321 P1322 P1323 P1324 P1325 P1326 P1331 P1332 P1333 P1334 P1335 P1336 P1337 P1338 P1340 P1341 P1342 P1346 P1347 P1348 P1349 P1350 P1351 P1352 P1353 P1354 P1355 P1356 P1357 P1358 P1359 P1360 P1361 P1362 P1363 P1364 P1365 P1366 P1367 P1370 P1371 P1372 P1373 P1374 P1375 P1376 P1377 P1378 P1396 P1397 P1398 P1399 P1400 P1401 P1402 P1403 P1404 P1405 P1406 P1412 P1413 P1415 P1416 P1423 P1424 P1430 P1431 P1433 P1434 P1470 P1471 P1472 P1474 P1476 P1477 P1478 P1479 P1480 P1482 P1483 P1484 P1485 P1486 P1487 P1488 P1491 P1493 P1494 P1495 P1496 P1497 P1498 P1502 P1503 P1504 P1505 P1506 P1507 P1510 P1513 P1517 P1520 P1523 P1524 P1525 P1526 P1527 P1529 P1531 P1532 P1533 P1534 P1535 P1536 P1537 P1538 P1539 P1540 P1541 P1542 P1543 P1544 P1545 P1546 P1547 P1550 P1551 P1552 P1553 P1555 P1556 P1557 P1558 P1559 P1560 P1562 P1563 P1564 P1565 P1566 P1567 P1568 P1569 P1570 P1571 P1572 P1573 P1574 P1575 P1576 P1577 P1578 P1579 P1580 P1581 P1582 P1583 P1585 P1586 P1587 P1588 P1589 P1590 P1591 P1592 P1593 P1594 P1595 P1597 P1599 P1601 P1602 P1603 P1604 P1605 P1606 P1607 P1608 P1609 P1610 P1611 P1613 P1615 P1616 P1617 P1618 P1619 P1620 P1621 P1622 P1624 P1625 P1626 P1627 P1628 P1629 P1630 P1631 P1633 P1634 P1635 P1636 P1637 P1639 P1640 P1641 P1642 P1643 P1661 P1662 P1673 P1674 P1675 P1676 P1677 P1678 P1679 P1680 P1681 P1682 P1683 P1685 P1686 P1688 P1689 P1691 P1692 P1693 P1695 P1697 P1698 P1699 P1700 P1701 P1702 P1703 P1704 P1705 P1706 P1708 P1709 P1710 P1711 P1712 P1713 P1714 P1715 P1716 P1717 P1720 P1721 P1723 P1724 P1726 P1729 P1732 P1733 P1735 P1736 P1738 P1739 P1740 P1741 P1742 P1743 P1744 P1745 P1746 P1749 P1750 P1752 P1753 P1755 P1758 P1759 P1760 P1761 P1762 P1764 P1765 P1767 P1768 P1769 P1770 P1771 P1772 P1773 P1774 P1775 P1776 P1777 P1778 P1779 P1780 P1782 P1784 P1787 P1788 P1790 P1791 P1792 P1793 P1794 P1795 P1796 P1797 P1798 P1799 P1800 P1801 P1802 P1803 P1804 P1805 P1806 P1807 P1808 P1809 P1810 P1811 P1812 P1813 P1814 P1815 P1816 P1817 P1818 P1819 P1820 P1823 P1825 P1827 P1828 P1829 P1830 P1831 P1832 P1833 P1834 P1835 P1836 P1838 P1839 P1841 P1842 P1843 P1845 P1846 P1847 P1848 P1850 P1852 P1854 P1856 P1857 P1858 P1859 P1860 P1861 P1862 P1864 P1865 P1867 P1868 P1869 P1870 P1871 P1872 P1873 P1874 P1875 P1877 P1879 P1880 P1881 P1882 P1883 P1884 P1885 P1886 P1887 P1888 P1889 P1890 P1891 P1892 P1893 P1894 P1895 P1896 P1897 P1898 P1899 P1900 P1901 P1902 P1903 P1904 P1905 P1906 P1907 P1908 P1909 P1910 P1911 P1912 P1913 P1914 P1915 P1916 P1917 P1918 P1919 P1921 P1922 P1923 P1924 P1925 P1926 P1927 P1928 P1929 P1930 P1931 P1933 P1934 P1936 P1937 P1938 P1939 P1940 P1941 P1943 P1944 P1945 P1946 P1947 P1948 P1949 P1950 P1951 P1952 P1953 P1955 P1956 P1957 P1958 P1959 P1962 P1963 P1964 P1965 P1966 P1967 P1968 P1969 P1970 P1971 P1972 P1973 P1974 P1975 P1976 P1977 P1978 P1979 P1980 P1981 P1982 P1983 P1984 P1985 P1987 P1991 P1993 P1995 P1996 P1997 P1999 P2001 P2003 P2009 P2010 P2011 P2012 P2013 P2014 P2015 P2016 P2017 P2018 P2019 P2020 P2021 P2022 P2023 P2024 P2025 P2026 P2027 P2028 P2029 P2030 P2032 P2036 P2037 P2038 P2039 P2040 P2041 P2042 P2045 P2046 P2047 P2048 P2051 P2052 P2053 P2054 P2055 P2056 P2057 P2058 P2059 P2060 P2061 P2063 P2064 P2066 P2070 P2071 P2072 P2073 P2076 P2077 P2078 P2079 P2080 P2081 P2082 P2083 P2084 P2085 P2086 P2087 P2088 P2089 P2090 P2091 P2092 P2093 P2094 P2095 P2096 P2097 P2098 P2099 P2100 P2103 P2104 P2106 P2107 P2109 P2110 P2114 P2115 P2116 P2117 P2118 P2120 P2121 P2123 P2124 P2125 P2127 P2128 P2129 P2130 P2131 P2132 P2133 P2134 P2135 P2136 P2137 P2138 P2139 P2140 P2141 P2142 P2143 P2144 P2145 P2146 P2147 P2148 P2149 P2150 P2151 P2152 P2153 P2154 P2155 P2156 P2157 P2158 P2159 P2160 P2163 P2164 P2167 P2168 P2171 P2172 P2175 P2176 P2177 P2178 P2179 P2180 P2182 P2183 P2185 P2187 P2196 P2201 P2202 P2206 P2207 P2208 P2209 P2213 P2214 P2216 P2217 P2218 P2219 P2220 P2221 P2222 P2223 P2224 P2225 P2226 P2227 P2228 P2229 P2230 P2233 P2234 P2236 P2237 P2238 P2239 P2240 P2241 P2242 P2243 P2244 P2245 P2247 P2248 P2250 P2253 P2254 P2255 P2256 P2257 P2260 P2262 P2263 P2264 P2265 P2266 P2268 P2269 P2273 P2274 P2275 P2277 P2278 P2279 P2280 P2281 P2282 P2283 P2284 P2285 P2286 P2287 P2288 P2289 P2290 P2291 P2294 P2295 P2296 P2299 P2300 P2301 P2302 P2303 P2304 P2305 P2306 P2307 P2308 P2309 P2310 P2312 P2313 P2315 P2319 P2321 P2322 P2323 P2324 P2325 P2328 P2329 P2330 P2331 P2332 P2333 P2335 P2336 P2337 P2338 P2339 P2340 P2341 P2342 P2343 P2344 P2345 P2347 P2348 P2354 P2357 P2358 P2359 P2360 P2363 P2364 P2365 P2366 P2367 P2368 P2369 P2370 P2371 P2372 P2373 P2374 P2377 P2378 P2379 P2380 P2381 P2382 P2384 P2385 P2386 P2387 P2388 P2389 P2390 P2391 P2393 P2394 P2395 P2396 P2397 P2398 P2399 P2400 P2401 P2402 P2403 P2404 P2405 P2406 P2407 P2408 P2409 P2410 P2411 P2412 P2413 P2414 P2415 P2416 P2417 P2418 P2419 P2420 P2421 P2422 P2423 P2424 P2425 P2426 P2427 P2428 P2429 P2430 P2431 P2432 P2433 P2434 P2435 P2436 P2437 P2438 P2439 P2440 P2441 P2442 P2443 P2444 P2445 P2446 P2447 P2448 P2449 P2450 P2451 P2452 P2453 P2454 P2455 P2456 P2457 P2458 P2459 P2460 P2461 P2462 P2463 P2464 P2465 P2466 P2467 P2468 P2469 P2470 P2471 P2472 P2473 P2474 P2475 P2476 P2477 P2478 P2479 P2480 P2481 P2482 P2483 P2484 P2485 P2486 P2487 P2489 P2490 P2493 P2494 P2495 P2496 P2497 P2498 P2499 P2500 P2501 P2502 P2503 P2505 P2506 P2507 P2508 P2509 P2510 P2511 P2512 P2513 P2514 P2515 P2516 P2521 P2522 P2524 P2525 P2526 P2527 P2528 P2530 P2531 P2533 P2534 P2535 P2536 P2537 P2538 P2539 P2540 P2541 P2542 P2543 P2544 P2545 P2546 P2547 P2548 P2549 P2551 P2552 P2553 P2554 P2555 P2556 P2557 P2558 P2559 P2560 P2562 P2563 P2564 P2565 P2566 P2567 P2568 P2569 P2570 P2571 P2572 P2573 P2574 P2575 P2576 P2577 P2578 P2579 P2580 P2581 P2584 P2586 P2589 P2590 P2591 P2592 P2593 P2594 P2595 P2596 P2597 P2598 P2599 P2600 P2601 P2602 P2603 P2604 P2605 P2606 P2607 P2608 P2609 P2610 P2611 P2612 P2613 P2614 P2615 P2616 P2617 P2618 P2619 P2620 P2621 P2622 P2623 P2624 P2625 P2626 P2627 P2628 P2629 P2630 P2631 P2632 P2633 P2634 P2635 P2636 P2637 P2638 P2639 P2640 P2641 P2642 P2643 P2644 P2645 P2646 P2647 P2648 P2649 P2650 P2651 P2652 P2653 P2654 P2655 P2656 P2657 S001 S002 S003 S004 S005 S006 S007 S008 S009 S010 S011 S012 S013 S014 S015 S016 S017 S018 S019 S020 S021 S022 S023 S024 S025 618.frameless-transforms.033 618.frameless-transforms.076 618.optional-source-helpers.007 618.optional-source-helpers.019 618.optional-source-helpers.023 618.optional-source-helpers.025 618.optional-source-helpers.027 618.optional-source-helpers.033 618.optional-source-helpers.038 618.optional-source-helpers.042 618.optional-source-helpers.043 618.optional-source-helpers.054 618.optional-source-helpers.055 618.optional-source-helpers.056 618.optional-source-helpers.057 618.effect-questions.004 618.effect-questions.005
```

### 4. Lost mechanisms, ownership and S10 ordering

The three accepted optional success losses are one G13 completion mechanism: an eager Expr-to-Option Some constructor does not run after suspension. Concrete owners remain S10/T0028 (P0755; confirmation.rs:104), S13/F0528/M0563 (P1489; callable.rs:77,163-164), and S19/F0358/M0374 (P0883; cases/collect.rs:448 and collect/resume_frames.rs:306-310). Their exact mandatory witness additions and draft citations are in `existing-owner-witness-additions.tsv`. Existing ownership is not moved to a new repair group.

**New G15-C1** owns P0941 and P0946. `replace/all.rs:59,127` propagates a suspended apply_replace_all before running its atom test/evaluate(replaced) tail at :60-70 and :128-138. The fixed point repairs the scalar 2 after the enclosing builtin EvaluationPass has already been emitted. The copied-tree experiment restoring that producer seam makes both full public traces SAME. It also contains the existing S13 completion experiment, but the minimized C1 witnesses use named Condition guards and do not invoke that Function frame. The repair design is one shared post-application semantic finish behind a typed Expr-to-Expr continuation, with distinct source labels and a retained second-pause owner.

S10 cannot precede C1: C1 is an unowned missing insertion that S24’s future file scope does not already repair, and the standing order explicitly puts new G15-C repairs and the manifest before CHANGELOG/S10. S24 must retain the new regression after C1. No additional independently minimized skipped-tail mechanism was established in the valid non-encoding panels; the parked population prevents a claim that C1 is the last one.

| owner | rows | witnesses | existing_scope | required |
| --- | --- | --- | --- | --- |
| S10 | P0755 | lost.confirm.info.2; seven information-consumer routes in legacy/618 controls | .temp/brief-606/draft-S10.md:150,169 (T0028) | Typed Expr-to-Option completion at the information producer; preserve every confirmation consumer route and legitimate decline; exact Some/None shape and request behavior are observed. |
| S13 | P1489 | lost.function.some; continuation1/six-extra-pauses.json; Function/composition/association/Failsafe/Fold/Nest controls | .temp/brief-606/draft-S13.md:96,106,136-138 (F0528/M0563) | Complete callable Return finish and genuine Some exactly once. Six replay witnesses keep their single planned answer. Original Function Fold/Nest witnesses remain conjunctive S3 + S13 gates. |
| S19 | P0883 | lost.cases.some; None and empty-Some collector controls | .temp/brief-606/draft-S19.md:110,133,165-167 (F0358/M0374) | Finish Cases collector typed Option success without losing its collected prefix or interpreting guest data as decline. |
| S24 | P0941;P0946 | min.ReplaceAll.atom;min.ReplaceAllUnheld.atom; G15-C1 two-phase and trace controls | .temp/brief-606/draft-S24.md:31,101-104,115-118 | Retain the already-promoted C1 post-apply evaluation frame/finish during later family conversion. This file scope does not defer the pre-S10 C1 repair. |
| E2 | P0755;P0883;P0941;P0946;P1489;P0360;P0362;P0364;P0368 | G15-A/B controls, owned optional completions, C1 trace at equal scalar, accepted LHS error/store controls | brief-618:641-647 (director change 2) | Keep the no-Try may-suspend result/coordinator and compile-fail controls in E2: plain ?, early return, InputRequired forwarding, wrong frame input and Expr substituted for Option stage. Do not introduce a second dispatch-local Pending type in CM. |

### 5. Obligation-keyed manifest design

# G15-C obligation-keyed oracle design

Status: design, with an executable exactness model. It does not certify the
unfinished census. Read continuation2/final-accounting.json for current counts.
The accepted brief-618 observation model and exact-context-closure ruling are
normative inputs, not redefined here.

## Source identities and discovery

The unit is a source producer/consumer edge, not a builtin catalog entry.
`ObligationKey = (ProducerId, ConsumerSiteId, TailPolicy)`. ProducerId includes
crate, qualified implementation/function, signature and selected dispatch arm
where that arm is concrete. ConsumerSiteId includes the enclosing qualified
function, normalized call-expression token fingerprint and occurrence within
that function. A source span is diagnostic, not identity: moving a function
does not silently delete its obligation. Identical calls at two sites remain
two obligations. Ambiguous resolution is a required review, never an arbitrary
target selected by name.

The implementation must reuse the calibrated census's macro/typed-frame and
supplemental passes. Required inventory includes the 2,682 current graph edges,
the screen cross-walk, and the two historical cross-walks. A new edge, changed
tail fingerprint, unresolved old identity or changed proof makes discovery red
before fixture factories run. A reviewer supplies its policy, route and
applicability; the factory cannot waive it. No ID-specific reading overlay may
turn a new row Equal.

Strict whole-Result tails transfer to their indexed callers. The 513 excluded
strict tails retain explicit links in the inventory; an exclusion disappears
if its caller cannot be found. The 283 without an indexed caller remain root
obligations. Transfers through cycles use a fixed point of caller obligations;
a cycle without a concrete guest/host root needs a proof, not an empty policy.
The population's other strict tails keep the additional screen/review
obligations that caused their inclusion.

## Policies and applicability

The exhaustive enum is continuation1/tail_policy.rs; the source assignments,
including proof-driven refinements, are tail-policies-final.tsv. Every arm is
explicit; adding an enum member breaks the context match until it is assigned.
There is no `Other`, default arm, inferred-empty context set or factory wildcard.
Multiple policies on a row take the union of required contexts and route
obligations. The concrete policy/context table and counts are
tail-policy-counts-final.tsv. Source reasons and pre-refinement assignments are
retained so a proof cannot silently erase obligations.

`NonSuspending`, `HostBatchBoundary` and `RetiredBoundary` are proof/disposition
arms, not empty pausing tests. They require a pinned proof, source fingerprint,
strongest attempt and observed producer pause count, or a cited boundary ruling.
A changed callee or argument invalidates a pure-call-site proof. A changed
retirement implementation invalidates its boundary certificate. Neither a
zero counter nor an uninstalled handler establishes Unreachable.

`ForwardResult` means resolve all source caller policies, not select whatever
contexts already passed. A root must select a concrete API/guest route. The
existing conservative all-context assignment is an audit upper bound until
that root review is explicit; it cannot be used to claim completion.

Required variants are Middle, Twice, Error, AlreadyResumed and Nested, plus
Decline where the producer can decline. Each non-applicable variant has a
source-specific reason. Twice has distinct `two_operands` and
`projection_then_value` phases where both apply. An early Throw that prevents
the second planned pause is an invalid Twice plan; it is not a two-pause pass.
Lifecycle receipts cover cancel and reset at the first and applicable second
pause. Those actions are separate receipts, not successful value comparisons.

Context recipes are concrete leaf IDs, not just a family name:

- value: ordinary success and prescribed projections;
- return_boundary: post-pause produced Return, enclosing suffix/store/loop,
  and source-required projection-versus-value routes;
- error_before_tail: matching/nonmatching propagation and Throw/Return at the
  source-selected phase; last-pause controls preserve earlier planned pauses;
- guest_heads: the four accepted saved literal-data recipes, including the
  callback-terminal forms where an intermediate compound would discard them;
- held_and_splice: held Return, Unevaluated, Sequence, Nothing, ordinary sibling
  positions and an actually evaluated subject inside a held owner;
- prefix_and_tail: distinct before/middle/after effects, pending cursor and
  stored-definition/scope projections;
- same_type_error_finish: matching and nonmatching cycles, Throw, stored
  state, termination/messages and a fresh request;
- phase_observation: ordered semantic trace, enabled phase/outcome counts
  without elapsed durations, and resource/cleanup ownership;
- consumer_routes: a separate source-derived route for each distinct consuming
  type/finish policy, including Function, Enclose string, Confirm failure,
  information consumers, and Inner versus Dot where applicable.

This is not permission to generate more internal-encoding probes in brief-621.
Continuation 2's prohibition remains. Port its eight completed batches as saved
fixtures/observations; classify them using continuation2/classify_encoding.py.
New source-phase gaps stay red or go to the director for a ruling.

## Required tuples and execution receipts

Keep installed-entry equality as its existing independent check. A source
entry is an `ObligationKey`, linked to any installed entries that exercise it.
For each source entry compute:

`(entry, descriptor, variant, leaf_context, consumer_route)`

from the policy, reviewed variant applicability and source route list, before
calling a fixture factory. Descriptor IDs denote concrete recipes, including
native host/API fixtures for source edges with no catalog owner. A catalog
entry's factory may supply several source obligations, but cannot define the
required set by returning a smaller list.

Maintain three independently checked collections: required source tuples,
fixture bindings, and executed receipts. Require exact equality and a
multiplicity of one in each. A receipt is emitted after execution and contains
the tuple, program/setup/answer fingerprints, build mode, phase attribution,
planned and actual request kinds/prompts/ordinals, lifecycle results, complete
observations and raw artifact references. Merely declaring a fixture never
emits a receipt. Duplicate work is detected before conversion to a set.

Every ordinary receipt must pause at the designated producer/phase. A Rust
source-stack guard at the first pause proves the first call, not the entire
resumed continuation. The second pause needs its frame/route provenance, or a
cited guest-to-site chain with asserted phase-specific request count. The
original Rust guard normally unwinds after the first pause: its absence on a
second stack does not prove that the continuation was lost. Queue sentinels,
setup, projection inspection and fresh requests do not count as subject pauses.

`consumer_route = row + function` in the raw coverage file is only an exposure
label. It is not a review of distinct input/output slot types. Likewise a cell
whose `observer_review` is pending is raw evidence, not a certified tuple. The
implementation must check a source-specific observer statement for each row:
which state, error boundary, held shape, suffix effect or phase becomes wrong
if this tail is omitted. An intermediate literal overwritten before the actual
producer returns does not cover that producer's literal-data context.

The nonpausing `Hold[ReleaseHold[...]]` recipe remains invalid. The executed
`Hold[71,Evaluate[subject],79]` family is its replacement when the source admits
that route. It does not by itself observe every earlier evaluator phase;
ordered trace and the row's other policy contexts remain required.

One actual guest program can emit several separately attributed receipts when
it observes several declared obligations. Execute that program once per build,
then fan out only to the predeclared, actually observed keys. Cache immutable
parse/setup plans, not session state or answers across independent cases. Never
deduplicate away a consumer route, phase or variant.

## Porting and the divergence ledger

`settled.tsv` is the port index. Each Equal row's saved probes become immutable
fixture bindings with its observer statement; the four accepted LHS rows have
97 concrete source-tuple bindings in continuation2/accepted-lhs-tuple-bindings.json.
This count is for the accepted LHS submatrix, not a claim of policy closure for
all 2,186 rows. Lost rows bind their minimized witnesses. Pure/boundary rows bind
proofs and attempts. Parked rows have no passing binding and keep the full gate
red until the director resolves them. Retired/Duplicate entries retain their
cross-walk proof instead of being dropped from inventory.

Known losses are immutable ledger entries keyed by obligation, tuple, owner,
program fingerprint and exact structured difference. The entry names a single
promotion (S10, S13, S19, G15-C1, or the classified G13 transport promotion).
An unrecognized difference, changed request count, altered state/trace, changed
owner or extra tuple is red. Test code cannot bless a new baseline by copying
its output. A named promotion requires the listed old DIFFs to become SAME and
removes exactly those ledger entries after review; remaining owners' entries
must remain byte-for-byte stable. A now-SAME entry is also red until its
promotion is acknowledged, avoiding stale baselines.

Encoding-row-frame-owner-counts.tsv is an exposure classification, not 370
causal Lost verdicts. In particular, a caller containing an already-known
broken callback is not independently a lost producer. Invalid transformed
plans are not immutable known-loss baselines. The six accepted legacy replay
witnesses retain their planned one-pause contract: an unexpected second pause
is the actual request-behavior difference, never an extra answer supplied to
make the fixture pass.

## Negative controls and implementation gates

The scratch model on accepted real evidence rejects omission of an applicable
context, omission of a consumer route, declaration without execution, duplicate
tuples, a pause before the producer, replacing two pauses with one, and deleting
a fixture declaration. Its 97-tuple success and seven independent rejections
are in continuation2/manifest-model-controls.json / manifest_model.py. Both
saved ReplaceAll public-trace losses also turn that semantic comparison red at
equal scalar 2. This is a model, not the production oracle implementation.

The implementation must additionally run independent, compiling copied-tree
semantic mutations: omit the Some completion at an Option stage; bypass the
G15-A Function-result Return finish without absorbing projection errors early;
bypass G15-B recovery in exactly one awaited LHS variant; and bypass G15-C1's
post-rewrite evaluation finish. Each uses its named structural observer and
must remain red with the final-value comparison disabled. These independent
source mutations were not run in this continuation. The accepted pre-G15
calibration changes A and B together and is not mislabeled as four independent
mutation experiments.

No production hot-path instrumentation is required by this design. Prefer the
permitted cited guest-to-site chains with asserted phase counts and the
existing diagnostic-copy attribution mechanism. If implementation requires
kernel instrumentation, give it an explicit test feature/env-gated cold path,
prove the ordinary build's cost, and run the required rum-vs-rum A/B; do not
quietly widen a test-only brief into evaluator work.

## Cost and sequence

The current corpus is an exploration matrix, not the production run schedule.
It repeats programs across many ancestor exposures and prints large Rust Debug
traces. The production scheduler shares program executions across declared
source obligations, emits compact receipts, and retains full observations on
failure. Keep all required tuples: sampling is forbidden.

The S9 script accepts a timeout per build; the established invocation is
`scripts/check-suspension-oracle.sh 1500` (brief-617:1471), with default and
no-default builds serial. Budget each expanded oracle build at <=1,500 seconds,
including its bounded build/test command; target <=900 seconds for steady-state
operation and expose the number of actual programs and receipts in output.
If closure exceeds 1,500 seconds, stop for a schedule/budget decision and
reduce duplicate execution/output, never contexts. The old S9 report records
369-test oracle runs of 32.65/32.64 seconds (brief-617:1177-1179). Those are
historical measurements, not a forecast for this design. A time per expanded
oracle build is unmeasured because that implementation does not yet exist;
this brief performs no new timing or performance work.

Sequence: resolve parked source/phase obligations; G15-C1 repair; implement
this manifest and run both full oracle builds/mutations within budget; then
CHANGELOG; then S10. Existing S10/S13/S19 losses stay explicitly owned in the
ledger until their planned promotions. S24 must retain the C1 regression when
it later converts the remaining ReplaceAll frames. F1-F23 and R retain the
script-classified transport obligations. The current incomplete census does
not authorize advancing CHANGELOG or S10.

Verbatim design-model output from the executed checker:

```text
MODEL accepted-LHS required=97 bound=97 executed=97 exactness=PASS
MODEL NEGATIVE omit-applicable-context: RED (required and executed tuple sets differ)
MODEL NEGATIVE omit-consumer-route: RED (required and executed tuple sets differ)
MODEL NEGATIVE declare-but-do-not-execute: RED (declared but unexecuted tuple; required and executed tuple sets differ)
MODEL NEGATIVE duplicate-tuple: RED (duplicate executed tuple)
MODEL NEGATIVE move-pause-before-producer: RED (designated producer/phase absent)
MODEL NEGATIVE replace-twice-with-one-pause: RED (designated pause multiplicity mismatch)
MODEL NEGATIVE delete-fixture-declaration: RED (fixture binding set differs from required source tuples)
MODEL SEMANTIC P0941: RED (public trace; both final scalar values are 2)
MODEL SEMANTIC P0946: RED (public trace; both final scalar values are 2)
test result: design model accepted=1 negative_controls=9 rejected=9 failures=0; full-census closure NOT certified
```

### 6. Drafts and execution evidence

- `.temp/brief-621/draft-G15-C1-rewrite-output-tail.md`: typed post-rewrite finish, both trace witnesses, two-phase/lifecycle gates, inventory replay and full mandatory hot-path A/B.
- `.temp/brief-621/draft-G15-CM-obligation-manifest.md`: source discovery, exhaustive policies/routes, exact required/bound/executed receipts, immutable promotions, porting, metadata and independent semantic negative controls, and <=1,500 seconds per feature build. It requires a ruling on parked rows.
- `.temp/brief-621/draft-existing-owner-additions.md`, `existing-owner-witness-additions.tsv` and 21 `witness-additions/*.tsv` tables: exact accepted witnesses for S10/S13/S19/S24/E2 and script-only grouped transport exposure additions for existing owners. E2 retains the no-Try coordinator and its compile-fail controls; CM does not create another Pending type.

All three are raw material for the director’s rewrite, not dispatch authorization. Proposed sequence remains: resolve parked obligations; C1; complete manifest gate; CHANGELOG; S10, with existing owner promotions and F1-F23/R retaining their recorded obligations.

The probe runs’ own result lines follow. `invalid` is a rejected plan or cross-build review, never a semantic equality. These lines include preserved accepted continuation-1 logs; only continuation2 files and the remaining portions of interrupted batches ran during this continuation. Original completed batches were not regenerated.

.temp/brief-621/calibration-current.log:

```text
test result: total=34 SAME=34 DIFF=0 invalid=0
```

.temp/brief-621/calibration-base.log:

```text
test result: total=34 SAME=12 DIFF=22 invalid=4
```

.temp/brief-621/calibration-extra-run.log:

```text
test result: total=21 SAME=21 DIFF=0 invalid=0
```

.temp/brief-621/lhs-context-run.log:

```text
test result: total=49 SAME=49 DIFF=0 invalid=0
```

.temp/brief-621/minimal-lost-run.log:

```text
test result: total=3 SAME=0 DIFF=3 invalid=0
```

.temp/brief-621/unreachable-attempt-run.log:

```text
test result: total=5 SAME=3 DIFF=2 invalid=2
```

.temp/brief-621/legacy-and-618-run.log:

```text
test result: total=331 SAME=309 DIFF=22 invalid=6
```

.temp/brief-621/continuation1/e1-consumer_routes.route-original-current.log:

```text
test result: total=2994 SAME=2989 DIFF=5 invalid=5 boundary=0
```

.temp/brief-621/continuation1/e1-error_before_tail.return-propagation-current.log:

```text
test result: total=2994 SAME=1827 DIFF=1167 invalid=1167 boundary=0
```

.temp/brief-621/continuation1/e1-error_before_tail.throw-current.log:

```text
test result: total=2994 SAME=1608 DIFF=1386 invalid=1386 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.nested-current.log:

```text
test result: total=2994 SAME=2955 DIFF=39 invalid=24 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.none-current.log:

```text
test result: total=2994 SAME=2954 DIFF=40 invalid=39 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.some-empty-current.log:

```text
test result: total=2994 SAME=2954 DIFF=40 invalid=39 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.some5-current.log:

```text
test result: total=2994 SAME=2955 DIFF=39 invalid=24 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-nested-current.log:

```text
test result: total=2994 SAME=2781 DIFF=213 invalid=75 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-none-current.log:

```text
test result: total=2994 SAME=2735 DIFF=259 invalid=130 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-some-empty-current.log:

```text
test result: total=2994 SAME=2735 DIFF=259 invalid=130 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-some5-current.log:

```text
test result: total=2994 SAME=2773 DIFF=221 invalid=75 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.held-position-current.log:

```text
test result: total=2994 SAME=0 DIFF=2994 invalid=2994 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.hold-return-current.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.nothing-current.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.sequence-current.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.tail-hold-current.log:

```text
test result: total=2994 SAME=2919 DIFF=75 invalid=75 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.tail-sequence-current.log:

```text
test result: total=2994 SAME=2923 DIFF=71 invalid=71 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.unevaluated-current.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-phase_observation.trace-current.log:

```text
test result: total=2994 SAME=2916 DIFF=78 invalid=5 boundary=0
```

.temp/brief-621/continuation1/e1-prefix_and_tail.log-current.log:

```text
test result: total=2994 SAME=2974 DIFF=20 invalid=8 boundary=0
```

.temp/brief-621/continuation1/e1-return_boundary.loop-store-current.log:

```text
run-bounded: timed out after 360s: .temp/brief-621/probe-target/debug/brief621-e1-current
```

.temp/brief-621/continuation1/e1-return_boundary.loop-store-reset-current.log:

```text
test result: total=2994 SAME=2409 DIFF=585 invalid=585 boundary=0
```

.temp/brief-621/continuation1/e1-return_boundary.return-value-current.log:

```text
test result: total=2994 SAME=1827 DIFF=1167 invalid=1167 boundary=0
```

.temp/brief-621/continuation1/e1-value-current.log:

```text
test result: total=3004 SAME=3003 DIFF=1 invalid=1 boundary=10
```

.temp/brief-621/continuation1/e1-consumer_routes.route-original-attribution.log:

```text
test result: total=2994 SAME=2989 DIFF=5 invalid=5 boundary=0
```

.temp/brief-621/continuation1/e1-error_before_tail.return-propagation-attribution.log:

```text
test result: total=2994 SAME=1827 DIFF=1167 invalid=1167 boundary=0
```

.temp/brief-621/continuation1/e1-error_before_tail.throw-attribution.log:

```text
test result: total=2994 SAME=1608 DIFF=1386 invalid=1386 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.nested-attribution.log:

```text
test result: total=2994 SAME=2955 DIFF=39 invalid=24 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.none-attribution.log:

```text
test result: total=2994 SAME=2954 DIFF=40 invalid=39 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.some-empty-attribution.log:

```text
test result: total=2994 SAME=2954 DIFF=40 invalid=39 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.some5-attribution.log:

```text
test result: total=2994 SAME=2955 DIFF=39 invalid=24 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-nested-attribution.log:

```text
test result: total=2994 SAME=2781 DIFF=213 invalid=75 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-none-attribution.log:

```text
test result: total=2994 SAME=2735 DIFF=259 invalid=130 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-some-empty-attribution.log:

```text
test result: total=2994 SAME=2735 DIFF=259 invalid=130 boundary=0
```

.temp/brief-621/continuation1/e1-guest_heads.tail-some5-attribution.log:

```text
test result: total=2994 SAME=2773 DIFF=221 invalid=75 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.held-position-attribution.log:

```text
test result: total=2994 SAME=0 DIFF=2994 invalid=2994 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.hold-return-attribution.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.nothing-attribution.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.sequence-attribution.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.tail-hold-attribution.log:

```text
test result: total=2994 SAME=2919 DIFF=75 invalid=75 boundary=0
```

.temp/brief-621/continuation1/e1-held_and_splice.unevaluated-attribution.log:

```text
test result: total=2994 SAME=2977 DIFF=17 invalid=17 boundary=0
```

.temp/brief-621/continuation1/e1-phase_observation.trace-attribution.log:

```text
test result: total=2994 SAME=2916 DIFF=78 invalid=5 boundary=0
```

.temp/brief-621/continuation1/e1-prefix_and_tail.log-attribution.log:

```text
test result: total=2994 SAME=2974 DIFF=20 invalid=8 boundary=0
```

.temp/brief-621/continuation1/e1-return_boundary.loop-store-reset-attribution.log:

```text
test result: total=2994 SAME=2409 DIFF=585 invalid=585 boundary=0
```

.temp/brief-621/continuation1/e1-return_boundary.return-value-attribution.log:

```text
test result: total=2994 SAME=1827 DIFF=1167 invalid=1167 boundary=0
```

.temp/brief-621/continuation1/e1-value-attribution.log:

```text
test result: total=3004 SAME=3003 DIFF=1 invalid=1 boundary=10
```

.temp/brief-621/continuation1/pure-attribution.log:

```text
test result: total=34 SAME=34 DIFF=0 invalid=0
```

.temp/brief-621/continuation1/pure-current.log:

```text
test result: total=34 SAME=34 DIFF=0 invalid=0
```

.temp/brief-621/continuation1/minimize-attribution.log:

```text
test result: total=24 SAME=1 DIFF=23 invalid=9
```

.temp/brief-621/continuation1/minimize-current.log:

```text
test result: total=24 SAME=1 DIFF=23 invalid=9
```

.temp/brief-621/continuation1/minimize-repair.log:

```text
test result: total=24 SAME=23 DIFF=1 invalid=1
```

.temp/brief-621/continuation2/arithmetic-attribution.log:

```text
test result: total=52 SAME=26 DIFF=26 invalid=26
```

.temp/brief-621/continuation2/arithmetic-current.log:

```text
test result: total=52 SAME=26 DIFF=26 invalid=26
```

.temp/brief-621/continuation2/e1-held_and_splice.tail-nothing-attribution-remaining.log:

```text
test result: total=2994 SAME=2921 DIFF=73 invalid=73 boundary=0
```

.temp/brief-621/continuation2/e1-held_and_splice.tail-nothing-current-remaining.log:

```text
test result: total=2994 SAME=2921 DIFF=73 invalid=73 boundary=0
```

.temp/brief-621/continuation2/e1-held_and_splice.tail-sequence-attribution-remaining.log:

```text
test result: total=1164 SAME=1134 DIFF=30 invalid=30 boundary=0
```

.temp/brief-621/continuation2/e1-return_boundary.tail-return-attribution-remaining.log:

```text
test result: total=2994 SAME=1790 DIFF=1204 invalid=1204 boundary=0
```

.temp/brief-621/continuation2/e1-return_boundary.tail-return-current-remaining.log:

```text
test result: total=2994 SAME=1790 DIFF=1204 invalid=1204 boundary=0
```

.temp/brief-621/continuation2/escalation-attribution.log:

```text
test result: total=67 SAME=67 DIFF=0 invalid=0 boundary=0
```

.temp/brief-621/continuation2/escalation-current.log:

```text
test result: total=67 SAME=67 DIFF=0 invalid=0 boundary=0
```

.temp/brief-621/continuation2/escalation-supplement-attribution.log:

```text
test result: total=1 SAME=1 DIFF=0 invalid=0 boundary=0
```

.temp/brief-621/continuation2/escalation-supplement-current.log:

```text
test result: total=1 SAME=1 DIFF=0 invalid=0 boundary=0
```

.temp/brief-621/continuation2/held-evaluate-attribution.log:

```text
test result: total=2994 SAME=2993 DIFF=1 invalid=1 boundary=0
```

.temp/brief-621/continuation2/held-evaluate-current.log:

```text
test result: total=2994 SAME=2993 DIFF=1 invalid=1 boundary=0
```

.temp/brief-621/continuation2/host-route-attribution.log:

```text
test result: total=20 SAME=20 DIFF=0 invalid=0
```

.temp/brief-621/continuation2/host-route-current.log:

```text
test result: total=20 SAME=20 DIFF=0 invalid=0
```

.temp/brief-621/continuation2/interpreted-attribution.log:

```text
test result: total=10 SAME=9 DIFF=1 invalid=1
```

.temp/brief-621/continuation2/interpreted-current.log:

```text
test result: total=10 SAME=9 DIFF=1 invalid=1
```

.temp/brief-621/continuation2/last-pause-return-attribution.log:

```text
test result: total=2382 SAME=2210 DIFF=172 invalid=172 boundary=0
```

.temp/brief-621/continuation2/last-pause-return-current.log:

```text
test result: total=2382 SAME=2210 DIFF=172 invalid=172 boundary=0
```

.temp/brief-621/continuation2/last-pause-throw-attribution.log:

```text
test result: total=2382 SAME=2196 DIFF=186 invalid=186 boundary=0
```

.temp/brief-621/continuation2/last-pause-throw-current.log:

```text
test result: total=2382 SAME=2196 DIFF=186 invalid=186 boundary=0
```

.temp/brief-621/continuation2/matcher-target-attribution.log:

```text
test result: total=200 SAME=178 DIFF=22 invalid=22
```

.temp/brief-621/continuation2/matcher-target-current.log:

```text
test result: total=200 SAME=178 DIFF=22 invalid=22
```

.temp/brief-621/continuation2/option-some-attribution.log:

```text
test result: total=36 SAME=27 DIFF=9 invalid=9
```

.temp/brief-621/continuation2/option-some-current.log:

```text
test result: total=36 SAME=27 DIFF=9 invalid=9
```

.temp/brief-621/continuation2/option-symbol-attribution.log:

```text
test result: total=9 SAME=9 DIFF=0 invalid=0
```

.temp/brief-621/continuation2/option-symbol-current.log:

```text
test result: total=9 SAME=9 DIFF=0 invalid=0
```

New host route probe crates compiled offline against the unmodified and exact-span attribution libraries. The build logs are `continuation2/host-route-{current,attribution}-build.log`; their only local Rust change is adding call/timeout selection to the scratch harness. No production/runtime/workspace suite, no full production suspension oracle in either feature build, no independent semantic source mutation suite, no new timing/instruction/stack measurement and no expanded whole-oracle cost gate ran here. The design-model checks are not reported as production cargo tests.

### 7. Director rulings and recommendations

- **2,027 parked rows:** keep them unclosed. Resolve source-specific phase/route applicability and observer placement before another broad equality promotion. Prioritize the now-471 coarse-complete rows, but require that the value/error/state actually crosses the designated tail; source guards alone do not certify resumed frame ownership. An adapter row with no executed API route needs its own harness.
- **140 Unreachable proofs:** review the newly added 67 individually as requested; the prior 73 stand as accepted. Their narrow branch/root conditions, strongest attempts and counts are above. Do not turn them into broad module exclusions.
- **Encoding scope:** retain continuation 2’s prohibition. The 608 decoded signatures are grouped by script, and 20 non-signature/cross-build families have non-encoding follow-up controls. Those controls do not justify silently dropping missing source-phase contexts.
- **Repair/manifest sequence:** accept or rewrite the C1 and CM raw drafts, retain S10/S13/S19 witnesses and S24’s regression, and require the independent source mutations and full run-cost gate in implementation. Do not clear CHANGELOG or S10 from this incomplete census.

Scope check before the append:

```text
SOURCE INTEGRITY PASS: 2087 other baseline files unchanged; no original source changes; no VCS command or metadata used.
FINAL INCOMPLETE: 159/2186 settled; 2027 explicit parked records; source-obligation design and follow-up drafts written; exhaustive closure and full oracle gates not achieved.
```



## Continuation 3

**198 of 2,186 rows have semantic verdicts: 43 Equal, 5 Lost, 140 Unreachable
and 10 Boundary. The other 1,988 are explicit OwnedObligations: 1,024 assigned
to G15-C2 and 964 to future G13 steps. There is no ownerless parked row. An
OwnedObligation is not Equal and is not an executed receipt. C2 remains Open.**

This continuation adds 39 measured Equal rows (29 C2, 10 S10), using 2,140
required tuple keys and a caught source-row mutant for every promotion. The
four earlier LHS Equals remain accepted prior evidence; the production port
also needs their individual mutation receipts. The other prior verdicts stand.
All original crates/docs/scripts/root file hashes except this permitted brief
are unchanged (2,087 files). All experiments, copies, binaries and drafts are
under `.temp/brief-621/`. No git, network, unsafe addition, production patch,
workspace scratch test or new encoding-family run occurred.

The governing continuation is complete as a bounded prototype/design report,
not as a full semantic closure of the remaining census. C1 is still a proposed
repair. C2's exact residual has a settlement draft; S10 cannot proceed yet.

### Final accounting and accepted foundation

The calibrated census, both historical cross-walks, population, exclusion rule,
and original comparator calibration were accepted and are reused unchanged.
They were not regenerated. The full old calibration output follows verbatim;
these are preserved results, not newly executed G1-G15 suites.

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

Current graph edges remain 2,682. All 2,468 brief-583 rows and 495 old
Unsettled rows are cross-walked in crosswalk-583.tsv and
crosswalk-548-unsettled.tsv. The 513 excluded strict whole-result tails retain
indexed caller links; 283 without an indexed caller remain roots in the
2,186-row population. No Retired/Duplicate verdict was newly invented.

Per-population totals below count memberships, so overlapping columns must
not be added to obtain the union. `settled.tsv` is the current port index;
`continuation3/ledger.jsonl` gives each row's explicit ledger kind and owner.
Blank semantic verdicts occur only in explicitly owned, unclosed entries.
| source | total | Equal | Lost | Unreachable | Boundary | OwnedObligation |
| --- | --- | --- | --- | --- | --- | --- |
| ALL | 2186 | 43 | 5 | 140 | 10 | 1988 |
| frame_carries | 752 | 43 | 4 | 19 | 0 | 686 |
| review_tail_correct | 437 | 3 | 1 | 36 | 0 | 397 |
| screen_effect-questions | 33 | 0 | 1 | 1 | 0 | 31 |
| screen_frameless-direct-evaluate | 56 | 2 | 1 | 0 | 3 | 50 |
| screen_frameless-transforms | 209 | 2 | 1 | 13 | 1 | 192 |
| screen_optional-source-helpers | 97 | 2 | 3 | 5 | 0 | 87 |
| strict_tail_without_indexed_caller | 283 | 0 | 0 | 26 | 0 | 257 |
| tool_unsettled | 1188 | 6 | 4 | 98 | 10 | 1070 |

### Ownership — source-qualified partition links

The assignment uses enclosing partition frame implementation, an M/T site in
the same qualified function, the frame in the actual call suffix, or a
distinctive helper invoked by that partition's resume body. File-wide scope is
not an ownership proof. Exact frame file and resume_frames parent/child module
qualification distinguish identically named guarded/rules string helpers.
That final correction moved 16 interim C2 rows to S48/S49/S50; its full
cross-walk is ownership-qualification-crosswalk.tsv. No observations or
mutant results were changed. Codec method spelling alone is not a helper link.

The final initial assignment is 974 future-G13 obligations and 1,053 C2 rows.
C2 includes 742 without a partition site/frame link, 310 whose linked typing
step is already Done, and one shared helper without a unique partition-site
owner (its caller candidates are S26/S33). Caller references are retained as
evidence, not treated as ownership of an unpartitioned helper site. C2 is the
explicit conservative owner until its settlement/cross-walk resolves that row.
Ten G13 rows and 29 C2 rows have now become measured Equal regressions.

Every step table is below, S10 first. Empty S1-S9 tables mean no deferred row
is owned by a completed step. Empty F/R owned tables do not erase their
separate saved transport obligations. Per-step incorporation cards are in
continuation3/owner-additions/; each table retains partition row/frame/slot/site
identity and distinguishes open obligations, measured Equal and causal losses.
| step | state | initial_owned | remaining_owned | measured_equal | known_loss | table |
| --- | --- | --- | --- | --- | --- | --- |
| S10 | Open | 38 | 28 | 10 | 1 | .temp/brief-621/continuation3/per-step-rows/S10.tsv |
| S11 | Open | 66 | 66 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S11.tsv |
| S12 | Open | 8 | 8 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S12.tsv |
| S13 | Open | 13 | 13 | 0 | 1 | .temp/brief-621/continuation3/per-step-rows/S13.tsv |
| S14 | Open | 22 | 22 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S14.tsv |
| S15 | Open | 15 | 15 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S15.tsv |
| S16 | Open | 17 | 17 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S16.tsv |
| S17 | Open | 24 | 24 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S17.tsv |
| S18 | Open | 20 | 20 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S18.tsv |
| S19 | Open | 35 | 35 | 0 | 1 | .temp/brief-621/continuation3/per-step-rows/S19.tsv |
| S20 | Open | 29 | 29 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S20.tsv |
| S21 | Open | 36 | 36 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S21.tsv |
| S22 | Open | 9 | 9 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S22.tsv |
| S23 | Open | 11 | 11 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S23.tsv |
| S24 | Open | 17 | 17 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S24.tsv |
| S25 | Open | 39 | 39 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S25.tsv |
| S26 | Open | 32 | 32 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S26.tsv |
| S27 | Open | 14 | 14 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S27.tsv |
| S28 | Open | 18 | 18 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S28.tsv |
| S29 | Open | 14 | 14 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S29.tsv |
| S30 | Open | 23 | 23 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S30.tsv |
| S31 | Open | 17 | 17 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S31.tsv |
| S32 | Open | 27 | 27 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S32.tsv |
| S33 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S33.tsv |
| S34 | Open | 26 | 26 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S34.tsv |
| S35 | Open | 18 | 18 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S35.tsv |
| S36 | Open | 18 | 18 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S36.tsv |
| S37 | Open | 36 | 36 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S37.tsv |
| S38 | Open | 18 | 18 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S38.tsv |
| S39 | Open | 25 | 25 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S39.tsv |
| S40 | Open | 10 | 10 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S40.tsv |
| S41 | Open | 17 | 17 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S41.tsv |
| S42 | Open | 32 | 32 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S42.tsv |
| S43 | Open | 22 | 22 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S43.tsv |
| S44 | Open | 8 | 8 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S44.tsv |
| S45 | Open | 10 | 10 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S45.tsv |
| S46 | Open | 35 | 35 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S46.tsv |
| S47 | Open | 17 | 17 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S47.tsv |
| S48 | Open | 23 | 23 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S48.tsv |
| S49 | Open | 19 | 19 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S49.tsv |
| S50 | Open | 46 | 46 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S50.tsv |
| S51 | Open | 20 | 20 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S51.tsv |
| S52 | Open | 19 | 19 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S52.tsv |
| S53 | Open | 9 | 9 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S53.tsv |
| S54 | Open | 2 | 2 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S54.tsv |
| S1 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S1.tsv |
| S2 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S2.tsv |
| S3 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S3.tsv |
| S4 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S4.tsv |
| S5 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S5.tsv |
| S6 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S6.tsv |
| S7 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S7.tsv |
| S8 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S8.tsv |
| S9 | Done | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/S9.tsv |
| F1 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F1.tsv |
| F2 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F2.tsv |
| F3 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F3.tsv |
| F4 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F4.tsv |
| F5 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F5.tsv |
| F6 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F6.tsv |
| F7 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F7.tsv |
| F8 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F8.tsv |
| F9 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F9.tsv |
| F10 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F10.tsv |
| F11 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F11.tsv |
| F12 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F12.tsv |
| F13 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F13.tsv |
| F14 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F14.tsv |
| F15 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F15.tsv |
| F16 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F16.tsv |
| F17 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F17.tsv |
| F18 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F18.tsv |
| F19 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F19.tsv |
| F20 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F20.tsv |
| F21 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F21.tsv |
| F22 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F22.tsv |
| F23 | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/F23.tsv |
| R | Open | 0 | 0 | 0 | 0 | .temp/brief-621/continuation3/per-step-rows/R.tsv |

S10 has 28 OwnedObligations, ten measured Equal regressions and its separate
accepted P0755/T0028 loss. Its original base-red plus mutants/full gate remains
mandatory. This prototype did not implement or promote S10. S13's P1489 and
six one-answer replay witnesses, S19's P0883, S24's retention of C1 and E2's
coordinator/static controls remain in the existing-owner additions.

### Accepted Unreachable proofs, grouped for review

All 140 previously supplied full proofs and strongest attempts remain in
continuation2/all-unreachable-proofs.json and the preserved preceding report.
No new Unreachable proof was introduced here. Their source constraints are
unchanged. The five classes are: 8 concrete pure accounting bridges (graph
false positives), 65 pure profile-label calls, 26 constant-selected arithmetic
arms, 23 disconnected interpreted-matcher calls, and 18 concrete Some-options
pure consumers. The exact module/chain/count breakdown is:
| proof_class | module | callee_chain_shape | count | rows |
| --- | --- | --- | --- | --- |
| concrete Some-options consumer | crates/rum-stdlib/src/option_consumers.rs | OptionConsumer::resume(Some(options)) → concrete pure string/shape consumer; merged-options evaluator branch bypassed | 18 | P2181;P2184;P2186;P2188;P2189;P2190;P2191;P2192;P2193;P2194;P2195;P2197;P2198;P2199;P2200;P2203;P2204;P2205 |
| concrete pure accounting bridge | crates/rum-runtime/src/runtime/accounting_mode/bridge.rs | sealed accounting mode → concrete pure closure / already successful value → reserve/clone/build/limit check; no Expr evaluation | 1 | P0003 |
| concrete pure accounting bridge | crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs | sealed accounting mode → concrete pure closure / already successful value → reserve/clone/build/limit check; no Expr evaluation | 1 | P0013 |
| concrete pure accounting bridge | crates/rum-runtime/src/runtime/accounting_mode/rule.rs | sealed accounting mode → concrete pure closure / already successful value → reserve/clone/build/limit check; no Expr evaluation | 6 | P0023;P0024;P0029;P0040;P0041;P0042 |
| constant-selected arithmetic | crates/rum-stdlib/src/arithmetic.rs | selected enum/dispatch arm → pure constant arithmetic finish; input expression excluded by that arm | 26 | P1645;P1646;P1647;P1648;P1649;P1650;P1651;P1652;P1653;P1654;P1655;P1656;P1657;P1658;P1659;P1660;P1663;P1664;P1665;P1666;P1667;P1668;P1669;P1670;P1671;P1672 |
| disconnected interpreted matcher | crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs | internal interpreted matcher island → pure syntax/match recursion; only internal Rust-test roots, no guest registration | 23 | P1437;P1438;P1439;P1440;P1441;P1442;P1443;P1444;P1445;P1446;P1447;P1449;P1450;P1451;P1452;P1454;P1455;P1464;P1465;P1466;P1467;P1468;P1469 |
| pure profile label | crates/rum-runtime/src/runtime/definitions/application/down_values.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 2 | P0250;P0261 |
| pure profile label | crates/rum-runtime/src/runtime/definitions/application/mod.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 2 | P0268;P0269 |
| pure profile label | crates/rum-runtime/src/runtime/definitions/application/rhs.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 6 | P0288;P0291;P0293;P0295;P0297;P0300 |
| pure profile label | crates/rum-runtime/src/runtime/definitions/application/sub_values.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 1 | P0306 |
| pure profile label | crates/rum-runtime/src/runtime/definitions/definitions.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 8 | P0411;P0412;P0414;P0422;P0424;P0427;P0429;P0432 |
| pure profile label | crates/rum-runtime/src/runtime/eval_call_prepare.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 3 | P0476;P0477;P0480 |
| pure profile label | crates/rum-runtime/src/runtime/eval_core/call.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 8 | P0486;P0488;P0490;P0491;P0493;P0495;P0497;P0499 |
| pure profile label | crates/rum-runtime/src/runtime/eval_core/mod.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 7 | P0510;P0512;P0513;P0514;P0516;P0517;P0518 |
| pure profile label | crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 6 | P0663;P0664;P0666;P0667;P0674;P0675 |
| pure profile label | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 6 | P0807;P0808;P0809;P0811;P0815;P0816 |
| pure profile label | crates/rum-runtime/src/runtime/resolution.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 2 | P1473;P1475 |
| pure profile label | crates/rum-runtime/src/runtime/scoping/callable.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 2 | P1490;P1492 |
| pure profile label | crates/rum-runtime/src/runtime/scoping/callable/status_application.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 1 | P1481 |
| pure profile label | crates/rum-runtime/src/runtime/scoping/lexical/module.rs | concrete borrowed Result → pure discriminant match → RuntimeProfileOutcome; no guest evaluation | 11 | P1508;P1509;P1511;P1512;P1514;P1515;P1516;P1518;P1519;P1521;P1522 |

### Lost groups remain owned

No new valid real-build DIFF was discovered in this continuation. The five
accepted causal losses remain P0755/S10, P1489/S13, P0883/S19, and
P0941/P0946/G15-C1. The initial three full minimized observations are preserved
in minimal-lost-results.jsonl and below verbatim; the C1 sources and ordered
traces are in continuation1/new-lost-witnesses.json and below. These are saved
accepted witnesses, not newly executed encoding programs. The full 140 proofs
are already verbatim in the preceding report; the class table is the requested
additional review view.

```json
{
  "case": {
    "id": "lost.function.some",
    "setup": "",
    "source": "Function[(InputString[\"w\"];System`InternalContinuationSome[5])][]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ]
  },
  "observation": {
    "id": "lost.function.some",
    "pauses": [
      "w"
    ],
    "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"InternalContinuationSome[5]\", typed: Other { head: \"InternalContinuationSome\", text: \"InternalContinuationSome[5]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"InternalContinuationSome[5]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
    "queued_pauses": [],
    "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"5\", typed: Integer(5) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"5\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
    "same": false,
    "valid": true
  }
}
```


```json
{
  "case": {
    "id": "lost.cases.some",
    "setup": "",
    "source": "Cases[{1},_ :> (InputString[\"w\"];System`InternalContinuationSome[5])]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ]
  },
  "observation": {
    "id": "lost.cases.some",
    "pauses": [
      "w"
    ],
    "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"List[InternalContinuationSome[5]]\", typed: List([Other { head: \"InternalContinuationSome\", text: \"InternalContinuationSome[5]\" }]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[InternalContinuationSome[5]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
    "queued_pauses": [],
    "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"List[5]\", typed: List([Integer(5)]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[5]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
    "same": false,
    "valid": true
  }
}
```


```json
{
  "case": {
    "id": "lost.confirm.info.2",
    "setup": "",
    "source": "Enclose[Confirm[$Failed, (InputString[\"i\"]; System`InternalContinuationSome[5])]]",
    "inputs": [
      "1"
    ],
    "expected_pauses": 1,
    "projections": [
      "log"
    ]
  },
  "observation": {
    "id": "lost.confirm.info.2",
    "pauses": [
      "i"
    ],
    "queued": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\", typed: Other { head: \"Failure\", text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", InternalContinuationSome[5]]]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
    "queued_pauses": [],
    "resumed": "Observation { final_outcome: Success(HostEvalSuccess { value: HostValue { text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\", typed: Other { head: \"Failure\", text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\" } }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"Failure[\\\"ConfirmationFailed\\\", Association[Rule[\\\"MessageTemplate\\\", \\\"`` encountered.\\\"], RuleDelayed[\\\"MessageParameters\\\", List[$Failed]], Rule[\\\"ConfirmationType\\\", \\\"Confirm\\\"], RuleDelayed[\\\"Expression\\\", $Failed], Rule[\\\"Information\\\", 5]]]\" }] }), projections: [Success(HostEvalSuccess { value: HostValue { text: \"List[]\", typed: List([]) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"List[]\" }] })], snapshot: Ok([83, 101, 116, 91, 108, 111, 103, 44, 32, 76, 105, 115, 116, 91, 93, 93, 10]), fresh: Success(HostEvalSuccess { value: HostValue { text: \"2\", typed: Integer(2) }, termination: None, messages: [], prints: [], trace: [], outputs: [HostStructuredOutput { kind: Result, text: \"2\" }] }), pending_cleared: true, queue_empty: true, prefix_ok: true, overlap_ok: true }",
    "same": false,
    "valid": true
  }
}
```


```json
[
  {
    "id": "P0941",
    "probe": "min.ReplaceAll.atom",
    "source": "ReplaceAll[1,(x_Integer /; (InputString[\"p\"];True)) :> (x+1)]",
    "inputs": "[\"1\"]",
    "pauses": [
      "p"
    ],
    "resumed_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAll\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }]",
    "queued_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAll\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }, EvaluationPass { before: \"ReplaceAll[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"2\" }]",
    "mechanism": "kernel_data/rewrite_data/replace/all.rs:59 uses ? on apply_replace_all with no caller frame for the remaining tail. After a rule guard/pattern test pauses, the typed apply_replace_all frames complete with an unevaluated replacement Expr. The caller's self-evaluating-atom test and evaluate(replaced) at :60-70 disappeared. The outer fixed point eventually evaluates Plus and repairs the scalar value, but its public EvaluationPass for the enclosing ReplaceAll call has already been emitted. Queued execution evaluates Plus first; resumed execution reports ReplaceAll -> Plus then Plus -> 2. The copied-tree frame restoring this tail makes the complete public-trace comparison SAME, with the same final scalar 2.",
    "owner": "G15-C1 (proposed); S24 must retain regression after later conversion",
    "ownership": "draft-S24.md:31 scopes this file; :101-104 and :115-118 list existing input/final-evaluation frames only. The missing post-apply producer insertion has no F/M/T row there; use a pre-S10 repair and retain it in S24."
  },
  {
    "id": "P0946",
    "probe": "min.ReplaceAllUnheld.atom",
    "source": "ReplaceAllUnheld[1,(x_Integer /; (InputString[\"p\"];True)) :> (x+1)]",
    "inputs": "[\"1\"]",
    "pauses": [
      "p"
    ],
    "resumed_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAllUnheld\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }]",
    "queued_trace": "[EvaluationPass { before: \"InputString[\\\"p\\\"]\", after: \"\\\"1\\\"\" }, EvaluationPass { before: \"CompoundExpression[InputString[\\\"p\\\"], True]\", after: \"True\" }, RuleApplied { source: \"ReplaceAllUnheld\", input: \"1\", rule: \"Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]]\", result: \"Plus[1, 1]\" }, EvaluationPass { before: \"Plus[1, 1]\", after: \"2\" }, EvaluationPass { before: \"ReplaceAllUnheld[1, RuleDelayed[Condition[Pattern[x, Blank[Integer]], CompoundExpression[InputString[\\\"p\\\"], True]], Plus[x, 1]]]\", after: \"2\" }]",
    "mechanism": "kernel_data/rewrite_data/replace/all.rs:127 uses ? on apply_replace_all with no caller frame for the remaining tail. After a rule guard/pattern test pauses, the typed apply_replace_all frames complete with an unevaluated replacement Expr. The caller's self-evaluating-atom test and evaluate(replaced) at :128-138 disappeared. The outer fixed point eventually evaluates Plus and repairs the scalar value, but its public EvaluationPass for the enclosing ReplaceAll call has already been emitted. Queued execution evaluates Plus first; resumed execution reports ReplaceAll -> Plus then Plus -> 2. The copied-tree frame restoring this tail makes the complete public-trace comparison SAME, with the same final scalar 2.",
    "owner": "G15-C1 (proposed); S24 must retain regression after later conversion",
    "ownership": "draft-S24.md:31 scopes this file; :101-104 and :115-118 list existing input/final-evaluation frames only. The missing post-apply producer insertion has no F/M/T row there; use a pre-S10 repair and retain it in S24."
  }
]
```

C1's missing post-application atom/evaluate finish remains at replace/all.rs:
59 and 127, with tails at 60-70 and 128-138. It precedes S10; S24's future file
conversion retains its regression rather than deferring the repair. The
script-classified internal-encoding differences remain G13 transport work.
None of the eight completed encoding families was replayed, extended,
individually minimized or reclassified by anecdote in this continuation.

### One copied-tree mutant schema and measured settlement

The copied tree is continuation3/mutant-source, generated from unchanged
originals. It contains exact census-span guards, 61 armed frame pass-through
switches and 659 ordinary-success pass-through switches. One runtime row
selector chooses a mutation; 387 schema records are unsupported. This build
has 1,107 source IDs: all 1,091 finally assigned C2/S10 rows plus 16 string rows
subsequently source-qualified to other G13 steps. Their extra evidence is
retained, not counted as C2 ownership. The finally assigned C2 schema is 42
framed, 647 ordinary and 364 unsupported; S10 is 19, 4 and 15 respectively.

A frame switch arms only when its designated producer row is on a real pause
stack, then forwards the actual incoming outcome without the tail. An ordinary
success switch returns the actual queued producer value before the consumer
tail. Errors keep the original path in that generic success schema; exclusively
error tails need specific mutations. Unlike carriers, adapters and local
closure return mismatches remain explicit unsupported work. Mutations stop at
subject completion: setup, projections, queue sentinels and fresh requests
cannot accidentally activate them. Heavy guards/getenv checks stay in copies.

Compiler experiments rejected 19 attempted FrameAction returns inside Result
closures; those rows are explicitly unsupported. The failed compiler logs and
source generation corrections are saved. No unsafe cast/default value or
fabricated panic was used to force a mutation to compile or kill a test.

Per-policy shapes and limitations are exhaustive (25 policies, no default):
| policy | frame_shape | frameless_shape | no_generic_shape |
| --- | --- | --- | --- |
| DefinitionLhsPreparation | Pass the child outcome before updating the awaited work and driving/committing the prepared LHS. | Return the actual prepared child before the rebuild/store tail. | Typed Expr→Expr permits pass-through; state/PreparedDefinitionLhs outputs need a specific phase omission. |
| MatchingSymbolCycleRecovery | Bypass finish_definition_lhs_active_expr in exactly one awaited variant; forward its Err/Ok unchanged. | Bypass the same finish after evaluate_atom_fast_path, before its recovery match. | Error-specific; success-only identity mutations cannot measure matching-cycle recovery. SEM.G15B is the concrete instance. |
| OptionalStage | For equal Option input/output, forward the Option unchanged; for a legacy Expr→Option stage omit its Some encoding. | Skip the actual Some completion/optional normalization at the producer seam. | No universal type-correct Expr→Option pass-through. The prototype Some control inspects the actual native slot before the lenient decoder. Never manufacture None. |
| FunctionResult | At the value-stage frame bypass Return absorption, retaining the required Some output; projection errors keep their distinct phase. | Skip finish_function_application for the same stage while retaining the slot conversion. | Projection and value phases are distinct. Expr→Option requires the explicit conversion; SEM.G15A is the value-stage instance. |
| ReturnBoundary | Forward the complete incoming outcome before the Return absorption/tag boundary. | Return the producer outcome before the Return finish. | Input/output and error types must agree; otherwise omit only the concrete Return match arm in a shared finish. |
| ErrorTransform | Forward the error before the frame maps/captures/normalizes it. | Return the producer error before the matching error-transform tail. | No generic conversion between unlike error/result carriers. Success-only pass-through is insufficient for an exclusively error tail. |
| PrefixLoop | Forward the current child before appending it, advancing the cursor, resuming siblings or rebuilding the prefix. | Return the child before the loop suffix/adoption. | Expr→Vec/aggregate outputs require a specific append/cursor/finish omission; no fabricated empty prefix. |
| DefinitionStore | Forward the prepared result before installing the definition or completing the update. | Return the actual prepared child/RHS before its store tail. | Expr→Expr is generic; compare stored definitions/projections and ordered update effects. |
| PrepareArgumentsRestore | Omit the concrete Unevaluated/Sequence restore or prepared-argument finish while preserving the carrier type. | Skip that same restore after the actual producer returns. | No universal Expr→PreparedArguments pass-through; mutate a real state/restore transition, never invent a prepared object. |
| OptionReader | Bypass the reader/adoption/consumer tail at its actual Option/merged-options stage. | Return a compatible actual child before option consumption. | No universal Expr→MergedOptions/borrowed RetainedOptions mutation. Keep each reentry type/route distinct. |
| ScopeCleanup | Forward the outcome before the explicit scope restore/pop epilogue. | Return before the matching explicit epilogue. | RAII may perform equivalent cleanup: that is a surviving mutant, not evidence. Heterogeneous scope slots need a specific epilogue omission. |
| CaptureOutput | Forward before capturing/draining/restoring messages, prints, trace or the saved output prefix. | Skip the matching capture finish after the producer. | CapturedOutcome is heterogeneous; no generic Expr pass-through across it. Preserve real prefix data instead of injecting defaults. |
| PhaseOwnership | Omit the actual phase-finalization/release transition, forwarding a compatible outcome. | Return before that same phase epilogue. | No universal mutation for RAII/resource/duration owners. Use native phase receipts, never wall-clock differences or invented counters. |
| MatchDecision | Forward before the matcher decision/guard/result normalization or resumed branch continuation. | Return a compatible actual match outcome before decision consumption. | Expr/CompiledMatchOutcome/maps are distinct carriers. Cross-carrier cases need a branch-specific omission, not truth substitution. |
| RebuildTraversal | Forward the child before rebuilding its parent, consuming the saved work list or continuing siblings. | Return the child before the matching rebuild/traversal tail. | Expr→Expr is generic. Traversal-state/Vec outputs require a concrete finish omission. |
| ShapeDecode | Forward the encoded outcome before the decoder/shape finish. | Return before the ordinary decode/shape tail. | Legacy Expr→Expr permits a native slot/shape observer. Typed unlike carriers have no generic pass-through and require a specific phase or compile control. |
| ScalarFinish | Forward the produced scalar before coercion/formatting/Boolean/numeric finish. | Return the compatible scalar before that same finish. | No universal string/bool/number→Expr pass-through; retain the true type or mutate the concrete finishing branch. |
| ConditionalBranch | Forward the condition/child before selecting and running its consumer branch. | Return the child before branch selection. | Generic only for equal carriers. Other cases omit the actual branch phase; do not invent a condition. |
| NativeCallbackBridge | Bypass the real bridge adoption/continuation finish, forwarding only when lifetimes/carriers permit. | Return the compatible producer before bridge completion. | No generic mutation for branded CallbackValue/RetainedResult/receipts. Native retained-prefix fixtures and row-specific safe omissions are required. |
| GuestLoading | Forward before the loader completion/registration/cleanup phase. | Return before that loader finish after the real provider call. | No generic EvalOutcome→Expr/request-response mutation; keep provider/capability configuration and use the concrete loading phase. |
| HostRequestDriver | Skip the actual request response/delivery/retirement finish at the adapter frame. | Return before the corresponding request-driver tail. | No generic kernel→Host/CLI/REPL/Jupyter response conversion. Each adapter needs its real driver and request ownership observer. |
| ForwardResult | When an actual tail exists, bypass it at the inherited consumer frame. | A strict whole-result return has no local tail to omit; follow its source-indexed consumer transfer. | No useful local mutant for a genuine identity tail. A surviving identity does not become Equal; prove/reconcile its inherited obligation or retain C2 ownership. |
| NonSuspending | Not applicable: the concrete constrained producer cannot suspend. | Not applicable. | Reviewed Unreachable proof and strongest attempted producer pause count remain required; no Equal shortcut. |
| HostBatchBoundary | Not applicable to the documented non-resumable batch boundary. | Not applicable. | Documented Boundary and its boundary tests, never a fake resume mutant. |
| RetiredBoundary | Not applicable to retired source. | Not applicable. | Exact cross-walk/retirement evidence; no live tuple or mutation to fabricate. |

The required source-site matrix was frozen before outcome selection:
79,350 planned keys, with fixture selection based on source placement and
stable catalog/probe order, not SAME. Missing/invalid/differing entries stayed
red. The first matrix had two complete rows. Correcting 286 Error/Do answer
plans preserved the exact source and all error/effect/state observations:
the first iteration throws out of Do, so its unreachable second iteration's
answers must not remain queued. Both builds passed 286/286. A final parsed-Input
answer family filled further non-encoding phase gaps without rewriting hidden
or delayed callback source. Of its 66 programs, 64 passed and two were invalid
on both arms after a non-association Return reentered a held target call;
they are not baselined or promoted. No valid real DIFF was hidden.

The mutation baseline re-executed 1,017 selected programs on real and disabled
copies: all SAME, exact public-observation parity. All 1,547 selected row/tuple
bindings had fresh actual producer pauses. Three bounded families executed
1,547 single-row mutation comparisons across 520 rows. The historical family
labels include eight string rows later qualified to S48-S50. Final C2-only
figures: 499 rows reached, 49 mutants caught, 29 complete settlements.

Four site-level promotions were withdrawn at the final route check. Their
passing 240 fragments and caught mutants do not enumerate the concrete generic
Route, accounting Mode or continuation types. They remain owned; the new model
rejects this omission independently. No passing evidence was erased.
| row | generic_parameter | missing_source_routes | evidence |
| --- | --- | --- | --- |
| P0403 | Route: DefinitionResumeRoute | ordinary witness versus resumed continuation-token route | definitions/definitions/destructuring.rs:98; prototype keyed the source site rather than both consuming error/continuation routes |
| P0508 | M: AccountingMode | concrete armed and unarmed accounting modes | eval_core/mod.rs:28; site-only guard does not identify the concrete accounting mode |
| P1147 | C: OptionReadContinuation&lt;Option&lt;Expr&gt;&gt; | each concrete next.finish continuation type | options/core.rs:175; source-site route cannot substitute for the continuation-type inventory |
| P1155 | C: OptionReadContinuation&lt;Expr&gt; | each concrete next.finish continuation type | options/read_phase.rs:87; source-site route cannot substitute for the continuation-type inventory |

The 39 new Equal rows below have 2,140 complete required keys and their own
caught mutants. Independent verification checked 2,075 distinct saved raw
programs on both builds, fresh producer attribution, disabled parity, exact
source hit and the actual red mutation JSON. (The verbatim verification below
is authoritative for the program count.) The observer column is the actual
mutant's description, not a source-reading observability certificate.
| row | owner | policy | required_tuples | mutant_probes | mutation_description |
| --- | --- | --- | --- | --- | --- |
| P0031 | G15-C2 | NativeCallbackBridge;OptionalStage | 80 | E1.2356.phase_observation.trace;E1.2356.prefix_and_tail.log;E1H.2356.held_and_splice.held-evaluate | After this producer returns Ok in the queued arm, return that same value immediately; omit the consumer tail. Errors still take the original path. The source hit and red public comparison measure this omission. |
| P0343 | G15-C2 | MatchDecision | 75 | E1.0833.prefix_and_tail.log;E1H.0833.held_and_splice.held-evaluate;E1T.0833.held_and_splice.tail-sequence | At DriveRewriteRulesOwnedSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0343 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0400 | G15-C2 | DefinitionStore | 70 | E1.0317.prefix_and_tail.log;E1H.0317.held_and_splice.held-evaluate;E1T.0317.held_and_splice.tail-sequence | At FinishHeldUpdateSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0400 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0418 | G15-C2 | DefinitionStore | 70 | E1.0473.prefix_and_tail.log;E1H.0473.held_and_splice.held-evaluate;E1T.0473.held_and_splice.tail-sequence | At RuleLhsCompletionFrame::resume, forward the incoming outcome without its tail; arm only when P0418 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0420 | G15-C2 | DefinitionStore | 70 | E1.0684.prefix_and_tail.log;E1H.0684.held_and_splice.held-evaluate;E1T.0684.held_and_splice.tail-sequence | At EvalRuleSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0420 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0556 | G15-C2 | RebuildTraversal | 70 | E1.2675.prefix_and_tail.log;E1H.2675.held_and_splice.held-evaluate;E1T.2675.held_and_splice.tail-sequence | At ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0556 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0563 | G15-C2 | OptionalStage;ScopeCleanup | 80 | E1.0990.phase_observation.trace;E1.0990.prefix_and_tail.log;E1H.0990.held_and_splice.held-evaluate | At EvalCheckAbortSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0563 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0573 | G15-C2 | ScopeCleanup | 55 | E1.0980.phase_observation.trace;E1.0980.prefix_and_tail.log;E1H.0980.held_and_splice.held-evaluate | At ResumeCheckAbortWithPropagationSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0573 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0576 | G15-C2 | ScopeCleanup | 55 | E1.0935.phase_observation.trace;E1.0935.prefix_and_tail.log;E1H.0935.held_and_splice.held-evaluate | At EvalAbortProtectSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0576 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0578 | G15-C2 | PhaseOwnership;ScopeCleanup | 55 | E1.1225.phase_observation.trace;E1.1225.prefix_and_tail.log;E1H.1225.held_and_splice.held-evaluate | At ResumeCheckAbortAfterPrimarySuspensionFrame1::resume, forward the incoming outcome without its tail; arm only when P0578 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0582 | G15-C2 | ReturnBoundary | 30 | E1.1190.phase_observation.trace;E1.1190.prefix_and_tail.log;E1.1190.return_boundary.loop-store-reset | At EvalAssertSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0582 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0585 | G15-C2 | OptionalStage;ReturnBoundary | 80 | E1.0950.phase_observation.trace;E1.0950.prefix_and_tail.log;E1H.0950.held_and_splice.held-evaluate | At ResumeAssertAfterFunctionSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0585 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0589 | G15-C2 | ReturnBoundary | 30 | E1.0945.phase_observation.trace;E1.0945.prefix_and_tail.log;E1.0945.return_boundary.loop-store-reset | At FinishAssertFunctionSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0589 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0687 | G15-C2 | CaptureOutput | 50 | E1.2046.phase_observation.trace;E1.2046.prefix_and_tail.log;E1.2046.return_boundary.loop-store-reset | At EvalCheckSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0687 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0691 | G15-C2 | CaptureOutput | 50 | E1.2051.phase_observation.trace;E1.2051.prefix_and_tail.log;E1.2051.return_boundary.loop-store-reset | At EvalQuietSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0691 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0699 | G15-C2 | ScopeCleanup | 55 | E1.1230.phase_observation.trace;E1.1230.prefix_and_tail.log;E1H.1230.held_and_splice.held-evaluate | At EvalWithCleanupSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0699 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0706 | G15-C2 | ScopeCleanup | 55 | E1.1180.phase_observation.trace;E1.1180.prefix_and_tail.log;E1H.1180.held_and_splice.held-evaluate | After this producer returns Ok in the queued arm, return that same value immediately; omit the consumer tail. Errors still take the original path. The source hit and red public comparison measure this omission. |
| P0726 | S10 | ReturnBoundary | 30 | E1.1000.phase_observation.trace;E1.1000.prefix_and_tail.log;E1.1000.return_boundary.loop-store-reset | At EvalEncloseSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0726 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0728 | S10 | ReturnBoundary | 30 | E1.1000.phase_observation.trace;E1.1000.prefix_and_tail.log;E1.1000.return_boundary.loop-store-reset | At EvalConfirmSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0728 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0730 | S10 | ReturnBoundary | 30 | E1.1015.phase_observation.trace;E1.1015.prefix_and_tail.log;E1.1015.return_boundary.loop-store-reset | At EvalConfirmBySuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0730 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0732 | S10 | ReturnBoundary | 30 | E1.1025.phase_observation.trace;E1.1025.prefix_and_tail.log;E1.1025.return_boundary.loop-store-reset | At EvalConfirmMatchSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0732 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0734 | S10 | ReturnBoundary | 30 | E1.1030.phase_observation.trace;E1.1030.prefix_and_tail.log;E1.1030.return_boundary.loop-store-reset | At EvalConfirmQuietSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0734 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0736 | S10 | ReturnBoundary | 30 | E1.1005.phase_observation.trace;E1.1005.prefix_and_tail.log;E1.1005.return_boundary.loop-store-reset | At EvalConfirmAssertSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0736 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0747 | S10 | ReturnBoundary | 30 | E1.1010.phase_observation.trace;E1.1010.prefix_and_tail.log;E1.1010.return_boundary.loop-store-reset | At ResumeConfirmByAfterValueSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0747 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0750 | S10 | ReturnBoundary | 30 | E1.1020.phase_observation.trace;E1.1020.prefix_and_tail.log;E1.1020.return_boundary.loop-store-reset | At ResumeConfirmMatchAfterValueSuspensionFrame1::resume, forward the incoming outcome without its tail; arm only when P0750 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0761 | G15-C2 | ReturnBoundary | 30 | E1.0970.phase_observation.trace;E1.0970.prefix_and_tail.log;E1.0970.return_boundary.loop-store-reset | At EvalCatchExceptionsSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0761 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0810 | G15-C2 | ReturnBoundary | 30 | E1.0960.phase_observation.trace;E1.0960.prefix_and_tail.log;E1.0960.return_boundary.loop-store-reset | At EvalCatchSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0810 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0841 | S10 | OptionalStage;ScalarFinish | 70 | E1.1235.prefix_and_tail.log;E1H.1235.held_and_splice.held-evaluate;E1T.1235.held_and_splice.tail-sequence | At FailureVirtualStyledMessageValueSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0841 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0843 | S10 | OptionalStage;ScalarFinish | 70 | E1.1050.prefix_and_tail.log;E1H.1050.held_and_splice.held-evaluate;E1T.1050.held_and_splice.tail-sequence | At FailureMessageFromTemplateSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P0843 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P0937 | G15-C2 | OptionalStage;RebuildTraversal | 70 | E1.0710.prefix_and_tail.log;E1H.0710.held_and_splice.held-evaluate;E1T.0710.held_and_splice.tail-sequence | After this producer returns Ok in the queued arm, return that same value immediately; omit the consumer tail. Errors still take the original path. The source hit and red public comparison measure this omission. |
| P1474 | G15-C2 | PhaseOwnership | 30 | E1.0935.phase_observation.trace | At EvaluateProfiledFixedPointSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P1474 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P1601 | G15-C2 | RebuildTraversal | 70 | E1.0363.prefix_and_tail.log;E1H.0363.held_and_splice.held-evaluate;E1T.0363.held_and_splice.tail-sequence | At ResumeCountAfterTargetSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P1601 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P1604 | G15-C2 | RebuildTraversal | 70 | E1.0462.prefix_and_tail.log;E1H.0462.held_and_splice.held-evaluate;E1T.0462.held_and_splice.tail-sequence | At ResumeMemberQAfterTargetSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P1604 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P1607 | G15-C2 | RebuildTraversal | 70 | E1.0415.prefix_and_tail.log;E1H.0415.held_and_splice.held-evaluate;E1T.0415.held_and_splice.tail-sequence | At ResumeFreeQAfterTargetSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P1607 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P1631 | G15-C2 | RebuildTraversal | 70 | E1.0499.prefix_and_tail.log;E1H.0499.held_and_splice.held-evaluate;E1T.0499.held_and_splice.tail-sequence | At EvalPositionSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P1631 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P1633 | G15-C2 | RebuildTraversal | 70 | E1.0509.prefix_and_tail.log;E1H.0509.held_and_splice.held-evaluate;E1T.0509.held_and_splice.tail-sequence | At ResumePositionAfterTargetSuspensionFrame::resume, forward the incoming outcome without its tail; arm only when P1633 was on the real pause stack. The source hit and red public comparison measure this omission. |
| P2141 | G15-C2 | PhaseOwnership;PrefixLoop | 75 | E1.1901.phase_observation.trace;E1.1901.prefix_and_tail.log;E1H.1901.held_and_splice.held-evaluate | After this producer returns Ok in the queued arm, return that same value immediately; omit the consumer tail. Errors still take the original path. The source hit and red public comparison measure this omission. |
| P2142 | G15-C2 | PhaseOwnership;PrefixLoop | 75 | E1.1921.phase_observation.trace;E1.1921.prefix_and_tail.log;E1H.1921.held_and_splice.held-evaluate | After this producer returns Ok in the queued arm, return that same value immediately; omit the consumer tail. Errors still take the original path. The source hit and red public comparison measure this omission. |
| P2143 | G15-C2 | PrefixLoop | 70 | E1.1926.phase_observation.trace;E1.1926.prefix_and_tail.log;E1H.1926.held_and_splice.held-evaluate | After this producer returns Ok in the queued arm, return that same value immediately; omit the consumer tail. Errors still take the original path. The source hit and red public comparison measure this omission. |

All Equal rows by policy/context, including the four previously accepted
LHS rows, are indexed below. The earlier four's accepted 97-binding/49-case
evidence remains in continuation2/accepted-lhs-tuple-bindings.json; it was not
relabeled as 39 newly mutated rows.
| row | policy | contexts | basis |
| --- | --- | --- | --- |
| P0031 | NativeCallbackBridge;OptionalStage | consumer_routes.route-original;error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0343 | MatchDecision | consumer_routes.route-original;error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0360 | DefinitionLhsPreparation | value;error_before_tail;same_type_error_finish;prefix_and_tail;guest_heads;held_and_splice | accepted earlier LHS evidence; individual mutation port required |
| P0362 | DefinitionLhsPreparation | value;error_before_tail;same_type_error_finish;prefix_and_tail;guest_heads;held_and_splice | accepted earlier LHS evidence; individual mutation port required |
| P0364 | DefinitionLhsPreparation | value;error_before_tail;same_type_error_finish;prefix_and_tail;guest_heads;held_and_splice | accepted earlier LHS evidence; individual mutation port required |
| P0368 | MatchingSymbolCycleRecovery | value;error_before_tail;same_type_error_finish;prefix_and_tail | accepted earlier LHS evidence; individual mutation port required |
| P0400 | DefinitionStore | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0418 | DefinitionStore | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0420 | DefinitionStore | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0556 | RebuildTraversal | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0563 | OptionalStage;ScopeCleanup | consumer_routes.route-original;error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0573 | ScopeCleanup | error_before_tail.last-pause-throw;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0576 | ScopeCleanup | error_before_tail.last-pause-throw;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0578 | PhaseOwnership;ScopeCleanup | error_before_tail.last-pause-throw;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0582 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0585 | OptionalStage;ReturnBoundary | consumer_routes.route-original;error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0589 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0687 | CaptureOutput | error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0691 | CaptureOutput | error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0699 | ScopeCleanup | error_before_tail.last-pause-throw;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0706 | ScopeCleanup | error_before_tail.last-pause-throw;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0726 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0728 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0730 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0732 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0734 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0736 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0747 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0750 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0761 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0810 | ReturnBoundary | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0841 | OptionalStage;ScalarFinish | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0843 | OptionalStage;ScalarFinish | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P0937 | OptionalStage;RebuildTraversal | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P1474 | PhaseOwnership | error_before_tail.last-pause-throw;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P1601 | RebuildTraversal | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P1604 | RebuildTraversal | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P1607 | RebuildTraversal | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P1631 | RebuildTraversal | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P1633 | RebuildTraversal | consumer_routes.route-original;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P2141 | PhaseOwnership;PrefixLoop | error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P2142 | PhaseOwnership;PrefixLoop | error_before_tail.last-pause-throw;guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |
| P2143 | PrefixLoop | guest_heads.tail-nested;guest_heads.tail-none;guest_heads.tail-some-empty;guest_heads.tail-some5;held_and_splice.held-evaluate;held_and_splice.tail-hold;held_and_splice.tail-nothing;held_and_splice.tail-sequence;held_and_splice.unevaluated;phase_observation.trace;prefix_and_tail.log;return_boundary.last-pause-return;return_boundary.loop-store-reset;value.base | new measured mutant |

### Four independent semantic controls

All four compiled in the same copied-tree schema and ran independently with
final-result comparison disabled. Some uses an ordinary guest value 5 and
checks the actual native Option output before the lenient decoder: its public
pair remains SAME while the slot contract turns red. G15-A/B and C1 turn red
through trace/state/effects. C1 still has scalar 2. Some and C1 start with
optional experimental completions because those stages are already Lost on
the real tree; those repairs are OFF in every C2/S10 run and never touch the
workspace. G15-A's first value-frame-only switch did not fire on its projection
pause; the corrected switch also skips the shared value finish reached after
that resume. The guest probe was unchanged and both attempts are retained.

```json
[
  {
    "mutant": "Some",
    "baseline_same": 1,
    "mutant_hit": true,
    "oracle_red_with_final_value_disabled": true,
    "public_mutant_same": 1,
    "native_slot_observer": true,
    "observer": "actual native Option-slot output receipt, before the lenient decoder; no guest encoding value",
    "repaired_experimental_completion": true,
    "cases": [
      "semantic.Some"
    ],
    "baseline_log": "semantic-Some-baseline.log",
    "mutant_log": "semantic-Some-mutant.log"
  },
  {
    "mutant": "G15A",
    "baseline_same": 1,
    "mutant_hit": true,
    "oracle_red_with_final_value_disabled": true,
    "public_mutant_same": 0,
    "native_slot_observer": false,
    "observer": "messages/prints/ordered trace/stored state/projections/request lifecycle, with final outcome value and output Result omitted",
    "repaired_experimental_completion": false,
    "cases": [
      "semantic.G15A"
    ],
    "baseline_log": "semantic-G15A-baseline.log",
    "mutant_log": "semantic-G15A-mutant-r2.log"
  },
  {
    "mutant": "G15B",
    "baseline_same": 1,
    "mutant_hit": true,
    "oracle_red_with_final_value_disabled": true,
    "public_mutant_same": 0,
    "native_slot_observer": false,
    "observer": "messages/prints/ordered trace/stored state/projections/request lifecycle, with final outcome value and output Result omitted",
    "repaired_experimental_completion": false,
    "cases": [
      "semantic.G15B"
    ],
    "baseline_log": "semantic-G15B-baseline.log",
    "mutant_log": "semantic-G15B-mutant.log"
  },
  {
    "mutant": "C1",
    "baseline_same": 2,
    "mutant_hit": true,
    "oracle_red_with_final_value_disabled": true,
    "public_mutant_same": 0,
    "native_slot_observer": false,
    "observer": "messages/prints/ordered trace/stored state/projections/request lifecycle, with final outcome value and output Result omitted",
    "repaired_experimental_completion": true,
    "cases": [
      "min.ReplaceAll.atom",
      "min.ReplaceAllUnheld.atom"
    ],
    "baseline_log": "semantic-C1-baseline.log",
    "mutant_log": "semantic-C1-mutant.log"
  }
]
```

### Ledger change, exactness and drafts

MANIFEST-DESIGN.md and the CM draft now require complete real SAME tuples AND
a caught row mutant. The ledger covers every source identity exactly once.
OwnedObligation entries may defer their full required plan only while their
exact partition owner is Open. Owner-deferred keys are never executed/SAME
receipts. Active required/bound/actually-executed multisets must match with
multiplicity one; existing standing comparisons and known-loss baselines still
run for open owners. No OwnedObligation hides a new difference in them.

An owner promotion activates/passes every owned tuple set, catches each row's
mutant, removes exactly those owned entries, retains their regressions and
only then sets Done. Done with a residual is red. Changing to another Open
owner without the source assignment is red too. New required rows, duplicates,
unexecuted declarations and wrong-phase/missing pauses remain red. E2 keeps
the no-Try coordinator/static controls; no second Pending type is proposed.

The real-data ledger model covers all 2,186 identities, treats the 159 prior
settlements as accepted opaque inputs, checks all 2,140 active keys for the
39 new Equals, and rejects eleven negatives. It is an executed design model,
not the production oracle or a certificate for every deferred key. Its full
output, and the earlier seven-negative/97-tuple model, are retained.

```text
LEDGER MODEL green: {"active_rows": 39, "bound": 2140, "executed": 2140, "owned": 1988, "required": 2140, "source_identities": 2186}; owned entries allowed only with Open owners.
LEDGER NEGATIVE owner-S10-done-with-owned-rows: RED (OwnedObligation owner is not Open: S10)
LEDGER NEGATIVE owner-C2-done-with-owned-rows: RED (OwnedObligation owner is not Open: G15-C2)
LEDGER NEGATIVE delete-owned-row: RED (source ledger identities differ from population)
LEDGER NEGATIVE duplicate-owned-row: RED (duplicate ledger identity)
LEDGER NEGATIVE reassign-to-completed-step: RED (OwnedObligation owner is not Open: S9; owner differs from source partition assignment: P0001)
LEDGER NEGATIVE reassign-to-a-different-open-step: RED (owner differs from source partition assignment: P0001)
LEDGER NEGATIVE promote-complete-SAME-but-surviving-identity-mutant: RED (no authentic caught row mutant: P0107)
LEDGER NEGATIVE borrow-another-rows-caught-mutant: RED (no authentic caught row mutant: P0107)
LEDGER NEGATIVE promote-caught-mutant-with-incomplete-real-matrix: RED (real tuple not SAME: P0410)
LEDGER NEGATIVE promote-site-matrix-with-unbound-concrete-consumer-routes: RED (source consumer-route inventory incomplete: P1147)
LEDGER NEGATIVE declare-but-do-not-execute-one-active-tuple: RED (required/executed tuple multisets differ)
test result: ledger model positive=1 negatives=11 rejected=11 failures=0; full production oracle NOT run
{"baseline_files_unchanged": 2087, "batch": "C3-design-ledger-model", "model": {"active_rows": 39, "bound": 2140, "executed": 2140, "owned": 1988, "required": 2140, "source_identities": 2186}, "negative_controls": 11, "next_action": "Revise CM/C2 briefs and each owner table; append report and verify original-tree integrity.", "verdicts": {"Boundary": 10, "Equal": 43, "Lost": 5, "OwnedObligation": 1988, "Unreachable": 140}}
```

Revised raw drafts in TEMPLATE shape:

- draft-G15-C1-rewrite-output-tail.md: accepted typed post-apply finish repair;
  retain both equal-scalar trace witnesses and independent finish bypass.
- draft-G15-C2-settlement.md: exact 1,024-row residual, 364 initially unsupported
  shapes, adapter/native drivers, concrete type-route gaps, identity transfers,
  full tuples plus mutants, bounded work and exact residual ownership.
- draft-G15-CM-obligation-manifest.md: source discovery, exact active/deferred
  inventory, owner-state/atomic promotions, measured observer criterion,
  immutable known losses and all negative controls; production cost gate.
- draft-existing-owner-additions.md: 78 exact G13 tables, S10 first, with one
  incorporation card per S10-S54. It retains the existing causal/transport
  witnesses and marks the ten newly measured S10 rows as regressions.

Required sequence: **C1, completed C2, the manifest, CHANGELOG, then S10**.
The model's Open-owned green state does not authorize bypassing C2. Newly
discovered repairs needed for C2 closure must also precede that gate.

### Remaining C2 composition and recommendations

The exact list is remaining-owned-rows/G15-C2.tsv, with initial partition
evidence in owned-obligations.tsv and the complete current rows in settled.tsv.
The 1,024 remaining rows break down as follows:

```json
[
  {
    "owner": "G15-C2",
    "initial": 1053,
    "remaining": 1024,
    "classes": {
      "unsupported schema": 364,
      "survived or no mutation hit": 450,
      "compiled but no valid binding reached": 190,
      "caught mutant; incomplete real tuples": 20
    }
  },
  {
    "owner": "S10",
    "initial": 38,
    "remaining": 28,
    "classes": {
      "unsupported schema": 15,
      "compiled but no valid binding reached": 10,
      "survived or no mutation hit": 3
    }
  }
]
```

Policy counts are memberships (policies overlap):

| policy | count |
| --- | --- |
| CaptureOutput | 13 |
| ConditionalBranch | 40 |
| DefinitionLhsPreparation | 12 |
| DefinitionStore | 52 |
| ErrorTransform | 43 |
| ForwardResult | 325 |
| FunctionResult | 10 |
| GuestLoading | 3 |
| HostRequestDriver | 95 |
| MatchDecision | 174 |
| NativeCallbackBridge | 32 |
| OptionReader | 45 |
| OptionalStage | 264 |
| PhaseOwnership | 49 |
| PrefixLoop | 55 |
| PrepareArgumentsRestore | 33 |
| RebuildTraversal | 267 |
| ReturnBoundary | 46 |
| ScalarFinish | 31 |
| ScopeCleanup | 37 |
| ShapeDecode | 139 |

Exact module/file composition (counts sum to 1,024):

| module | count |
| --- | --- |
| crates/rum-cli/src/eval_tests.rs | 8 |
| crates/rum-cli/src/input_retirement.rs | 1 |
| crates/rum-cli/src/main.rs | 22 |
| crates/rum-host/src/session.rs | 11 |
| crates/rum-jupyter/src/adapter/execute.rs | 3 |
| crates/rum-jupyter/src/adapter/input_retirement.rs | 1 |
| crates/rum-jupyter/src/adapter/worker.rs | 6 |
| crates/rum-repl/src/lib.rs | 5 |
| crates/rum-repl/src/request.rs | 7 |
| crates/rum-runtime/src/runtime/accounting_mode/bridge.rs | 3 |
| crates/rum-runtime/src/runtime/accounting_mode/mod.rs | 3 |
| crates/rum-runtime/src/runtime/accounting_mode/options.rs | 2 |
| crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs | 6 |
| crates/rum-runtime/src/runtime/accounting_mode/retained.rs | 3 |
| crates/rum-runtime/src/runtime/accounting_mode/rule.rs | 15 |
| crates/rum-runtime/src/runtime/arithmetic/parts.rs | 1 |
| crates/rum-runtime/src/runtime/builtin_adapters.rs | 62 |
| crates/rum-runtime/src/runtime/builtins_context_loader.rs | 1 |
| crates/rum-runtime/src/runtime/debug_tools/monitor/options.rs | 4 |
| crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs | 3 |
| crates/rum-runtime/src/runtime/debug_tools/test_report/options.rs | 5 |
| crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs | 3 |
| crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs | 2 |
| crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs | 1 |
| crates/rum-runtime/src/runtime/debug_tools/trace/options.rs | 2 |
| crates/rum-runtime/src/runtime/definitions/application/default_values.rs | 16 |
| crates/rum-runtime/src/runtime/definitions/application/down_values.rs | 17 |
| crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs | 2 |
| crates/rum-runtime/src/runtime/definitions/application/replace_all.rs | 10 |
| crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs | 5 |
| crates/rum-runtime/src/runtime/definitions/application/rhs.rs | 10 |
| crates/rum-runtime/src/runtime/definitions/application/sub_values.rs | 9 |
| crates/rum-runtime/src/runtime/definitions/application/up_values.rs | 11 |
| crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs | 5 |
| crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs | 16 |
| crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs | 4 |
| crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs | 11 |
| crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs | 1 |
| crates/rum-runtime/src/runtime/definitions/definition_tags.rs | 6 |
| crates/rum-runtime/src/runtime/definitions/definition_updates.rs | 8 |
| crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs | 3 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | 12 |
| crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs | 6 |
| crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs | 1 |
| crates/rum-runtime/src/runtime/definitions/incremental_append.rs | 1 |
| crates/rum-runtime/src/runtime/definitions/part_assignment.rs | 2 |
| crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs | 1 |
| crates/rum-runtime/src/runtime/definitions/rhs_guards.rs | 18 |
| crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs | 5 |
| crates/rum-runtime/src/runtime/definitions/rule_lhs.rs | 5 |
| crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs | 3 |
| crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs | 1 |
| crates/rum-runtime/src/runtime/eval_call_prepare.rs | 4 |
| crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs | 1 |
| crates/rum-runtime/src/runtime/eval_core/call.rs | 14 |
| crates/rum-runtime/src/runtime/eval_core/mod.rs | 6 |
| crates/rum-runtime/src/runtime/eval_core/symbol.rs | 13 |
| crates/rum-runtime/src/runtime/eval_dispatch/mod.rs | 12 |
| crates/rum-runtime/src/runtime/hold_thread.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/association_data.rs | 6 |
| crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs | 5 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs | 5 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs | 2 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs | 6 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs | 7 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs | 2 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs | 2 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/table_fast_path.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs | 4 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs | 5 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs | 12 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs | 14 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs | 5 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/mod.rs | 4 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs | 20 |
| crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs | 7 |
| crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/options.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/options.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs | 2 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/options.rs | 2 |
| crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs | 2 |
| crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs | 1 |
| crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs | 3 |
| crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs | 1 |
| crates/rum-runtime/src/runtime/lowering.rs | 1 |
| crates/rum-runtime/src/runtime/namespace_tools/names.rs | 1 |
| crates/rum-runtime/src/runtime/namespace_tools/names/options.rs | 3 |
| crates/rum-runtime/src/runtime/options/builtins.rs | 3 |
| crates/rum-runtime/src/runtime/options/core.rs | 18 |
| crates/rum-runtime/src/runtime/options/read_phase.rs | 2 |
| crates/rum-runtime/src/runtime/options/resume_frames.rs | 3 |
| crates/rum-runtime/src/runtime/package_tools.rs | 1 |
| crates/rum-runtime/src/runtime/package_tools/persist_options.rs | 1 |
| crates/rum-runtime/src/runtime/pattern_matching/core.rs | 2 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs | 2 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs | 3 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs | 6 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs | 2 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/entry.rs | 4 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs | 5 |
| crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs | 2 |
| crates/rum-runtime/src/runtime/resolution.rs | 2 |
| crates/rum-runtime/src/runtime/resolution/resume_frames.rs | 2 |
| crates/rum-runtime/src/runtime/scoping/callable/status_application.rs | 10 |
| crates/rum-runtime/src/runtime/scoping/lexical/block.rs | 2 |
| crates/rum-runtime/src/runtime/scoping/lexical/block/continuation.rs | 1 |
| crates/rum-runtime/src/runtime/scoping/lexical/module.rs | 1 |
| crates/rum-runtime/src/runtime/scoping/lexical/with.rs | 3 |
| crates/rum-runtime/src/runtime/scoping/packages.rs | 1 |
| crates/rum-runtime/src/runtime/session/guest_loading/script.rs | 2 |
| crates/rum-runtime/src/runtime/session/session_eval.rs | 9 |
| crates/rum-runtime/src/runtime/session/session_requests.rs | 11 |
| crates/rum-runtime/src/runtime/suspension/core.rs | 6 |
| crates/rum-runtime/src/runtime/suspension/core/abandon.rs | 2 |
| crates/rum-runtime/src/runtime/suspension/core/fault.rs | 1 |
| crates/rum-runtime/src/runtime/suspension/typed/adapter.rs | 2 |
| crates/rum-runtime/src/runtime/test_builtins.rs | 1 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs | 15 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like/options.rs | 1 |
| crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs | 10 |
| crates/rum-runtime/src/runtime/tree_walkers/level.rs | 1 |
| crates/rum-runtime/src/runtime/tree_walkers/position.rs | 7 |
| crates/rum-runtime/src/runtime/tree_walkers/position/options.rs | 5 |
| crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs | 6 |
| crates/rum-stdlib/src/arithmetic.rs | 1 |
| crates/rum-stdlib/src/arithmetic/stats.rs | 1 |
| crates/rum-stdlib/src/associations/aggregate.rs | 3 |
| crates/rum-stdlib/src/associations/basic.rs | 2 |
| crates/rum-stdlib/src/associations/grouping.rs | 8 |
| crates/rum-stdlib/src/associations/grouping/association.rs | 1 |
| crates/rum-stdlib/src/associations/grouping/list.rs | 3 |
| crates/rum-stdlib/src/associations/key_reshape.rs | 2 |
| crates/rum-stdlib/src/associations/missing.rs | 2 |
| crates/rum-stdlib/src/associations/missing/target.rs | 7 |
| crates/rum-stdlib/src/associations/mutation.rs | 7 |
| crates/rum-stdlib/src/associations/resumable/aggregate.rs | 2 |
| crates/rum-stdlib/src/associations/resumable/aggregate/merge_nested.rs | 2 |
| crates/rum-stdlib/src/associations/resumable/key_select.rs | 1 |
| crates/rum-stdlib/src/associations/resumable/key_sort.rs | 2 |
| crates/rum-stdlib/src/associations/resumable/map.rs | 2 |
| crates/rum-stdlib/src/associations/transforms.rs | 1 |
| crates/rum-stdlib/src/associations/transforms/key_collections.rs | 1 |
| crates/rum-stdlib/src/control_flow.rs | 38 |
| crates/rum-stdlib/src/functional_ops/apply_map/mod.rs | 3 |
| crates/rum-stdlib/src/functional_ops/arrays/filter.rs | 1 |
| crates/rum-stdlib/src/functional_ops/arrays/generators.rs | 2 |
| crates/rum-stdlib/src/functional_ops/arrays/mod.rs | 5 |
| crates/rum-stdlib/src/functional_ops/arrays/options.rs | 1 |
| crates/rum-stdlib/src/functional_ops/arrays/tensor.rs | 3 |
| crates/rum-stdlib/src/functional_ops/callable/head_ops.rs | 3 |
| crates/rum-stdlib/src/functional_ops/iteration.rs | 16 |
| crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs | 15 |
| crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs | 2 |
| crates/rum-stdlib/src/functional_ops/sequence/select.rs | 4 |
| crates/rum-stdlib/src/functional_ops/sequence/window.rs | 5 |
| crates/rum-stdlib/src/functional_ops/targeted_map.rs | 2 |
| crates/rum-stdlib/src/integer_digits.rs | 2 |
| crates/rum-stdlib/src/lazy.rs | 3 |
| crates/rum-stdlib/src/lazy/machine.rs | 4 |
| crates/rum-stdlib/src/lazy/realization.rs | 1 |
| crates/rum-stdlib/src/lazy/source.rs | 1 |
| crates/rum-stdlib/src/logic/truth.rs | 4 |
| crates/rum-stdlib/src/number_theory.rs | 5 |
| crates/rum-stdlib/src/option_consumers.rs | 7 |
| crates/rum-stdlib/src/predicates/arrays.rs | 2 |
| crates/rum-stdlib/src/predicates/collections.rs | 8 |
| crates/rum-stdlib/src/predicates/ordering.rs | 3 |
| crates/rum-stdlib/src/predicates/truth.rs | 3 |
| crates/rum-stdlib/src/random.rs | 1 |
| crates/rum-stdlib/src/rewrite_builtins.rs | 8 |
| crates/rum-stdlib/src/schema.rs | 1 |
| crates/rum-stdlib/src/schema/machine.rs | 2 |
| crates/rum-stdlib/src/strings/encoding.rs | 1 |
| crates/rum-stdlib/src/strings/patterns/search.rs | 24 |
| crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs | 3 |
| crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs | 6 |
| crates/rum-stdlib/src/strings/patterns/search/rules.rs | 15 |
| crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs | 10 |
| crates/rum-stdlib/src/strings/scripting/args.rs | 2 |
| crates/rum-stdlib/src/strings/scripting/html.rs | 2 |
| crates/rum-stdlib/src/strings/scripting/text_shape.rs | 6 |
| crates/rum-stdlib/src/strings/scripting/url.rs | 6 |
| crates/rum-stdlib/src/strings/template.rs | 4 |
| crates/rum-stdlib/src/structure_ops/collection_stats.rs | 11 |
| crates/rum-stdlib/src/structure_ops/dedup.rs | 3 |
| crates/rum-stdlib/src/structure_ops/list_lookup.rs | 2 |
| crates/rum-stdlib/src/structure_ops/list_plumbing.rs | 3 |
| crates/rum-stdlib/src/structure_ops/mod.rs | 1 |
| crates/rum-stdlib/src/structure_ops/ordering_sets.rs | 13 |
| crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs | 2 |
| crates/rum-stdlib/src/structure_ops/surface.rs | 2 |

Source-population memberships (overlapping):

| source | count |
| --- | --- |
| frame_carries | 182 |
| review_tail_correct | 261 |
| screen_effect-questions | 16 |
| screen_frameless-direct-evaluate | 42 |
| screen_frameless-transforms | 158 |
| screen_optional-source-helpers | 61 |
| strict_tail_without_indexed_caller | 249 |
| tool_unsettled | 604 |

Recommendations for the director's C2 rewrite: start with caught mutants
whose remaining gaps are concrete routes/non-encoding phases; retain the
encoding restriction unless explicitly changed. Add row-specific native and
adapter mutations where generic carrier conversion is impossible. Resolve
genuine identity/no-tail roots through exact inherited source transfers or a
reviewed disposition; never invent an unrelated destructive mutant to force
P0107-like identities red. Keep C2 Open until its whole required disposition
is approved. The current data does not justify a blanket identity exemption.

### Verbatim execution results, cost and scope

Raw coverage is exposure evidence, not the settled tuple count:

```text
FINAL COVERAGE {"attributed_rows": 1309, "cells": 2805392, "distinct_probes": 80661, "results": {"DIFF": 23220, "SAME": 2580635, "invalid": 201537}, "unattributed_rows": 877}
POPULATION {"Boundary": 10, "Equal": 43, "Lost": 5, "OwnedObligation": 1988, "Unreachable": 140, "source": "ALL", "total": 2186}
POPULATION {"Boundary": 0, "Equal": 43, "Lost": 4, "OwnedObligation": 686, "Unreachable": 19, "source": "frame_carries", "total": 752}
POPULATION {"Boundary": 0, "Equal": 3, "Lost": 1, "OwnedObligation": 397, "Unreachable": 36, "source": "review_tail_correct", "total": 437}
POPULATION {"Boundary": 0, "Equal": 0, "Lost": 1, "OwnedObligation": 31, "Unreachable": 1, "source": "screen_effect-questions", "total": 33}
POPULATION {"Boundary": 3, "Equal": 2, "Lost": 1, "OwnedObligation": 50, "Unreachable": 0, "source": "screen_frameless-direct-evaluate", "total": 56}
POPULATION {"Boundary": 1, "Equal": 2, "Lost": 1, "OwnedObligation": 192, "Unreachable": 13, "source": "screen_frameless-transforms", "total": 209}
POPULATION {"Boundary": 0, "Equal": 2, "Lost": 3, "OwnedObligation": 87, "Unreachable": 5, "source": "screen_optional-source-helpers", "total": 97}
POPULATION {"Boundary": 0, "Equal": 0, "Lost": 0, "OwnedObligation": 257, "Unreachable": 26, "source": "strict_tail_without_indexed_caller", "total": 283}
POPULATION {"Boundary": 10, "Equal": 6, "Lost": 4, "OwnedObligation": 1070, "Unreachable": 98, "source": "tool_unsettled", "total": 1188}
REMAINING {"classes": {"caught mutant; incomplete real tuples": 20, "compiled but no valid binding reached": 190, "survived or no mutation hit": 450, "unsupported schema": 364}, "initial": 1053, "owner": "G15-C2", "remaining": 1024}
REMAINING {"classes": {"compiled but no valid binding reached": 10, "survived or no mutation hit": 3, "unsupported schema": 15}, "initial": 38, "owner": "S10", "remaining": 28}
```

The original 80,309-probe matrix was reused; 352 new context programs add
12,680 cells. Selected baseline replays exist solely to calibrate and exercise
the new mutation schema, not to regenerate completed corpus batches.
`continuation3/c3-calibration-real.log`

```text
test result: total=30 SAME=30 DIFF=0 invalid=0
```

`continuation3/c3-calibration-disabled.log`

```text
test result: total=30 SAME=30 DIFF=0 invalid=0
```

`continuation3/semantic-Some-baseline.log`

```text
test result: total=1 SAME=1 DIFF=0 invalid=0
```

`continuation3/semantic-Some-mutant.log`

```text
test result: total=1 SAME=1 DIFF=0 invalid=0
```

`continuation3/semantic-G15A-baseline.log`

```text
test result: total=1 SAME=1 DIFF=0 invalid=0
```

`continuation3/semantic-G15A-mutant.log`

```text
test result: total=1 SAME=1 DIFF=0 invalid=0
```

`continuation3/semantic-G15A-mutant-r2.log`

```text
test result: total=1 SAME=0 DIFF=1 invalid=0
```

`continuation3/semantic-G15B-baseline.log`

```text
test result: total=1 SAME=1 DIFF=0 invalid=0
```

`continuation3/semantic-G15B-mutant.log`

```text
test result: total=1 SAME=0 DIFF=1 invalid=0
```

`continuation3/semantic-C1-baseline.log`

```text
test result: total=2 SAME=2 DIFF=0 invalid=0
```

`continuation3/semantic-C1-mutant.log`

```text
test result: total=2 SAME=0 DIFF=2 invalid=0
```

`continuation3/loop-error-current.log`

```text
test result: total=286 SAME=286 DIFF=0 invalid=0 boundary=0
```

`continuation3/loop-error-attribution.log`

```text
test result: total=286 SAME=286 DIFF=0 invalid=0 boundary=0
```

`continuation3/mutation-baseline-real.log`

```text
test result: total=1017 SAME=1017 DIFF=0 invalid=0 boundary=3
```

`continuation3/mutation-baseline-disabled.log`

```text
test result: total=1017 SAME=1017 DIFF=0 invalid=0 boundary=3
```

`continuation3/closure-current.log`

```text
test result: total=66 SAME=64 DIFF=2 invalid=2 boundary=0
```

`continuation3/closure-attribution.log`

```text
test result: total=66 SAME=64 DIFF=2 invalid=2 boundary=0
```

`continuation3/mutants-S10-run.log`

```text
ROW MUTANTS S10: rows=13 tuples=39 caught=10 hit=13 unobserved=3
```

`continuation3/mutants-C2-frames-run.log`

```text
ROW MUTANTS C2-frames: rows=36 tuples=107 caught=30 hit=35 unobserved=6
```

`continuation3/mutants-C2-ordinary-run.log`

```text
ROW MUTANTS C2-ordinary: rows=471 tuples=1401 caught=19 hit=440 unobserved=452
```


```text
INDEPENDENT EVIDENCE new Equal=39 required keys=2140 distinct raw programs=2075; real+attribution raw JSON all SAME/valid.
INDEPENDENT EVIDENCE each row: actual producer pause + disabled exact parity + own source hit + public mutant red = PASS (39/39).
INDEPENDENT ACCOUNTING source rows=2186 closed=198 owned=1988 ownerless=0; C2=1024 G13=964.
INDEPENDENT SCOPE original files unchanged=2087; no production source changes.
```

Every individual row runner's own test-result line is preserved here;
the corresponding full JSON/logs remain in continuation3/row-mutants/:
| row | test_result |
| --- | --- |
| P0008 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0012 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0015 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0031 | test result: total=3 SAME=0 DIFF=3 invalid=1 boundary=0 |
| P0046 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0050 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0051 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0052 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0053 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0054 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0055 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0056 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0057 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0058 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0059 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0060 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0061 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0062 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0063 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0064 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0065 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0066 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0067 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0068 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0069 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0070 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0071 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0072 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0073 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0074 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0075 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0076 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0077 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0078 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0079 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0080 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0081 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0082 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0083 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0084 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0085 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0086 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0087 | test result: total=1 SAME=1 DIFF=0 invalid=0 boundary=1 |
| P0088 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0089 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0090 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0091 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0093 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0094 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0095 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0096 | test result: total=1 SAME=1 DIFF=0 invalid=0 boundary=1 |
| P0097 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0098 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0099 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0100 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0101 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0102 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0103 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0104 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0105 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0106 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0107 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0108 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0109 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0110 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0114 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0126 | test result: total=1 SAME=1 DIFF=0 invalid=0 boundary=1 |
| P0128 | test result: total=1 SAME=1 DIFF=0 invalid=0 boundary=1 |
| P0144 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0186 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0187 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0188 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0193 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0203 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0219 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0232 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0233 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0234 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0236 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0249 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0252 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0254 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0255 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0256 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0259 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0262 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0266 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0277 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0283 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0287 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0313 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0314 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0315 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0330 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0331 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0332 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0333 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0334 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0335 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0343 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0355 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0358 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0367 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0376 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0377 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0388 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0395 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0399 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0400 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0402 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0403 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P0407 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0410 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0413 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0416 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0417 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0418 | test result: total=3 SAME=0 DIFF=3 invalid=2 boundary=0 |
| P0420 | test result: total=3 SAME=0 DIFF=3 invalid=2 boundary=0 |
| P0423 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0431 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0447 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0448 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0465 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0468 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0470 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P0478 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0484 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0485 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P0487 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0489 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0506 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0507 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0508 | test result: total=3 SAME=2 DIFF=1 invalid=0 boundary=0 |
| P0509 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0511 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0515 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0524 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0526 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0530 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0536 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0538 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0539 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0541 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0542 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0545 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0547 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0556 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0558 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0560 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0563 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0573 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0575 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0576 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0578 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0580 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0582 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0584 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0585 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0587 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0589 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0599 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0600 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0622 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0624 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0625 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0626 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0687 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0691 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0699 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0706 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0708 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0709 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0726 | test result: total=3 SAME=0 DIFF=3 invalid=1 boundary=0 |
| P0728 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0730 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0732 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0734 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0736 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0738 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0743 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0747 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0750 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0761 | test result: total=3 SAME=0 DIFF=3 invalid=1 boundary=0 |
| P0763 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0770 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0774 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0776 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0777 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0790 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0795 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0798 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0801 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0810 | test result: total=3 SAME=0 DIFF=3 invalid=1 boundary=0 |
| P0814 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0817 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0818 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0820 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0821 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0822 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0825 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0826 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0828 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0840 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0841 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0843 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P0846 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0904 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0934 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0937 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P0975 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P0983 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1005 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1037 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P1038 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P1085 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1101 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1102 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1107 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1116 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1124 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1132 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P1136 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1142 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1144 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1146 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1147 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1149 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1155 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1165 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1187 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1231 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1232 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1238 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1239 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1267 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1275 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1283 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1284 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1310 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1323 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1324 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1326 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1333 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1335 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1354 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1472 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1474 | test result: total=3 SAME=2 DIFF=1 invalid=0 boundary=0 |
| P1476 | test result: total=2 SAME=2 DIFF=0 invalid=0 boundary=0 |
| P1496 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1497 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1507 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1526 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1535 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1544 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1547 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1556 | test result: total=1 SAME=1 DIFF=0 invalid=0 boundary=1 |
| P1558 | test result: total=1 SAME=0 DIFF=1 invalid=1 boundary=1 |
| P1562 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1567 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1601 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1602 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1603 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1604 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1605 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1606 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1607 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1608 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1609 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1611 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1617 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1618 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1619 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1622 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1631 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1633 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1634 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1635 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1639 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1642 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1661 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1679 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1680 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1681 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1704 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1729 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1758 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1759 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1760 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1768 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1770 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1771 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1772 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1773 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1774 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1779 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1780 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1812 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1813 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1814 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1815 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1816 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1817 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P1819 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1823 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1833 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1834 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1841 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1847 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1848 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1859 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1860 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1873 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1877 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1879 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1880 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1881 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1882 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1883 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1884 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1885 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1886 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1887 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1888 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1889 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1890 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1893 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1894 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1895 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1897 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1898 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1899 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1900 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1901 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1902 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1903 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1908 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1909 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1917 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1918 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1919 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1923 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1928 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1929 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1940 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1941 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1950 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1955 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1962 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1963 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1973 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1974 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1977 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1978 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1982 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1984 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P1996 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2009 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2010 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2011 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2013 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2015 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2017 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2018 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2020 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2022 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2023 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2025 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2070 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2080 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2082 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2083 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2096 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2098 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2099 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2100 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2117 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2118 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2136 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2141 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P2142 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P2143 | test result: total=3 SAME=0 DIFF=3 invalid=0 boundary=0 |
| P2144 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2145 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2155 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2156 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2157 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2158 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2182 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2183 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2185 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2187 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2196 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2201 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2202 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2206 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2207 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2223 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2224 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2225 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2226 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2227 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2228 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2236 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2245 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2250 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2263 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2264 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2265 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2266 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2275 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2277 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2278 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2279 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2280 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2281 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2282 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2283 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2284 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2287 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P2288 | test result: total=3 SAME=0 DIFF=3 invalid=3 boundary=0 |
| P2299 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2302 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2312 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2329 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2333 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2339 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2347 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2364 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2368 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2369 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2374 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2377 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2380 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2388 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2391 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2397 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2406 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2407 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2410 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2413 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2419 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2422 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2423 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2428 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2429 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2430 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2431 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2432 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2433 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2434 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2435 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2436 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2437 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2438 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2439 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2440 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2446 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2452 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2453 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2454 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2455 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2456 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2457 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2458 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2459 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2460 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2461 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2462 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2463 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2464 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2465 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2466 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2467 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2468 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2469 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2470 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2471 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2472 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2473 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2474 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2475 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2476 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2477 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2478 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2479 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2480 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2481 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2482 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2483 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2501 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2505 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2506 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2507 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2508 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2509 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2510 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2511 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2512 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2513 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2514 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2524 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2525 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2526 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2537 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2551 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2552 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2565 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2566 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2567 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2568 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2569 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2570 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2571 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2572 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2573 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2574 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2575 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2576 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2590 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |
| P2592 | test result: total=3 SAME=3 DIFF=0 invalid=0 boundary=0 |

No full production oracle, default/no-default oracle gate, workspace test
suite, formatter gate, retention replay, timing/instruction/stack work or
microbench A/B was run in this source-unchanged design continuation. The
expanded production oracle cost remains unmeasured. CM retains <=1,500 seconds
per complete build (target <=900), sharing actual program executions without
sampling requirements. Any future production hot-path change needs its full
rum-vs-rum A/B and explicit source scope.

No unexplained calibration failure, content-filter interruption or two-zero
probe-batch stop-loss occurred. Each executed family added measured coverage;
the prototype's bounded attempts are complete and resistant rows remain owned.
Model/draft/qualification checks are design steps, not fictitious probe batches.
The only original-file changes are this report append and the opening-summary
rewrite expressly required by continuation 3. The source scene stays intact.

## Director review (2026-10-19) — accepted as a design; C1, C2, CM, then CHANGELOG, then S10

**Runs.**

| Run | Session | Tokens | Result |
| --- | --- | ---: | --- |
| Run 1 | `01a151f9-…` | 408k | Stopped under my stop-loss at 15 of 2 186 rows. My brief demanded per-row policy contexts before the policies existed (step 3 before step 5), and taken in ID order the hardest rows came first. Both faults are mine. |
| Continuation 1 | fork `01a15222-…` | 986k | Policies for every row, then about 2.2 M coverage cells. Cut by Codex's content filter after the internal-encoding batches (log `cdx_brief_621.filter-1.log`). |
| Continuation 2 | fresh `01a1523d-…` | 416k | Resumed from the checkpoint and classified the internal-encoding differences by script. 159 rows settled and 2 027 parked. Found G15-C1. |
| Continuation 3 | fork `01a15267-…` | 1.23M | Measured observation by tail-skip mutants in one switchable copied-tree build. 198 rows settled and the other 1 988 assigned to owners: 1 024 to G15-C2 and 964 to future G13 steps. The ledger model and the drafts. |

**Accepted.**

- **The foundation.**
  - The calibrated re-census: 2 682 edges, all 86 historical Lost and the
    G9-G15 controls reproduce.
  - Both cross-walks: 2 468/2 468 and 495/495.
  - The 2 186-row population.
  - The exhaustive 25-policy enum with every row assigned.
  - The corpus: 80 309 probes, 2.79 M cells.
  - The comparator calibrated on the pre-G15 export.
- **The verdicts.**
  - 43 `Equal`, each with a caught row mutant.
  - 5 `Lost`, in four owned groups: S10 (T0028), S13 (F0528), S19 (F0358)
    and the new G15-C1.
  - 140 `Unreachable`, in five classes that each pin a concrete call site:
    accounting bridges 8, profile labels 65, constant-selected arithmetic
    arms 26, the interpreted-matcher island 23, pure options consumers 18.
  - 10 `Boundary`.
- **The design.** The obligation-keyed manifest:
  - an `ObligationKey` that is not a span;
  - three checked collections and receipts;
  - an immutable known-loss ledger with named promotions;
  - `OwnedObligation` entries that are green only while their owner is
    open;
  - eleven ledger negatives, all red.
- **The four drafts** under `.temp/brief-621/`.

**G15-C1, re-probed by the director.** I used this brief's comparator
(`.temp/director/c1-probe/`), whose cases add a log, a Return and a list
target. The ordered trace differs in three cases: `ReplaceAll` on an atom,
`ReplaceAllUnheld` on an atom, and `ReplaceAll` on `{1}`. Without trace, with
a log projection and with a Return in the replacement, the cases are SAME.
The resumed path skips the post-apply atom test and `evaluate(replaced)` at
`replace/all.rs:59,127`. The fixed point repairs the value, but the builtin's
evaluation pass has already been reported. It is a real lost tail, of G15's
class.

**Rulings.**

- **Sequence.** C1 (brief-622), then C2's settlement, then the manifest,
  then `CHANGELOG.md`, then S10. C2 precedes S10 because no later step owns
  its rows.
- **C2's 310 rows in finished steps' code.** These lie in the scope of
  S3-S9, whose gates did not settle them. C2 owns them now. Each finished
  step's gate settled its own frames by fixtures and mutants, not the
  census rows around them; that is the gap this line closes.
- **C2's 450 "survived or no mutation hit" rows are its most important
  class.** C2's brief separates them:
  - a mutant that was **never armed** is missing coverage, and C2 must reach
    it;
  - a mutant that was **armed and survived**, across every required context
    including phase observation, is either a tail with no observable effect
    or a blind spot. C2 must decide which, by a stronger context or by a
    stated equivalence. I rule on each such class.
- **Owned rows of future G13 steps.** Each step's brief carries its table
  from `draft-existing-owner-additions.md`. Its promotion requires every
  owned row settled under the mutant rule.

**Lessons.**

- A design brief that asks for settlement before it defines what settles a
  row produces bespoke work per row and stops early. Define the policy
  first and settle by coverage.
- "Which effect goes wrong if this tail is omitted" is a reading claim.
  A caught mutant is the measurement.
- A row that a finished step's code contains is not settled by that step's
  gate unless the gate's mutants reached it.
