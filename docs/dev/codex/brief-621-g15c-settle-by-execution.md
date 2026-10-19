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
