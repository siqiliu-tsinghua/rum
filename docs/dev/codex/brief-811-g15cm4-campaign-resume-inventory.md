# Brief 811 — CM4 class campaign resume: the epoch, the Stage C/C10 readmission and the 744 restart — design-only

**Director-dispatched, one design-only stage, one gate.** This plans the class campaign's
resumption after the D.1/D.2 line: the campaign epoch rebuilt on the current tree, the Stage
C/C10 readmission, and the 744 shakedown restart. It writes no code and runs no build or
capture. `Scope: NONE` of production.

## Goal

When this brief is done: the campaign's qualification state is determined from the sealed
records (the 754 epoch acceptance, the class repair line 756-797, the 806 install, the 807f
fresh epoch, the 805/809 results) with the exact evidence; the required epoch rebuild on the
current tree is specified as a step-by-step plan (the twelve release labels and three trees,
the observation policy, the build rules, the functional equality, the qualification gates);
the Stage C/C10 readmission is specified (the ten class members, the 20 logical own cells and
their ledger treatment against the 510-cell schedule, the C10 packet, the importers, the
ordinary negatives); the 744 restart is specified (the 244 cells: 18 own rows × 2, the 100
pair selections × 2, the four structural controls × 2, the negative controls, the re-freeze
for 745); every input is authenticated by hash and every unknown is either resolved or named
with the exact missing measurement; and the three implementation briefs (the epoch build, the
readmission, the 744 restart) are drafted in this template's shape. No production change.

## Context pointers

- Read `AGENTS.md`; rulings 759/762/763 (the repaired epoch), 766-783 (the class repair line),
  784-797 (the render/install), 798-809 (the D.2/residual line), 851/855/856 (the fresh epoch
  and the closed residual line), 857 (the two cost decisions: publication funded, fastpath
  excluded).
- The accepted but now stale campaign epoch: `.temp/brief-754/attempt-3/epoch/cm754-epoch-identity.json`
  `7af7b461…` (binding build `809b6473…`, freeze `c724c9ab…`, the six-comparison manifest
  `c3bc2616…`, the twelve release labels `45916ecc…`/`011d61bd…`/`6919c52a…`/`45916ecc…`/
  `b3b8b1e1…`/`c3d2d5d2…`/`c736d3c4…`/`b3b8b1e1…` plus the lane `b92b3ed2…`/`caaa5b6a…`/
  `c286c776…`/`b92b3ed2…`, the three trees `97eaeb45…`/`b41d1873…`/`bd8b7548…` at 5,810
  files); `.temp/brief-754/attempt-2/epoch/` (the thirteen build receipts and compiler logs,
  `cm754-blocks.jsonl` `ee5bb36c…`, the observation policy `3953aa9f…`, the render schema
  `5c3b02a3…`, `cm754-membership.json` `2a17e399…`, `cm754-selections.json` `8c061b27…`,
  `cm754-program-matrix.json` `5c15fc12…`, `cm754-own-tuples.json` `0202baf6…`).
- The schedule and admission authority: `.temp/brief-755/full-schedule-ledger.json`
  `3e4532cf…` (the 510-cell schedule; the `stage=744` subsequence), `.temp/brief-755/class-admission.json`
  `6d556f4b…`, `.temp/brief-755/packet-index.json` `3e1644aa…`, the class importers
  `5baa3124…`-bound proofs, the C10 calibration `f46ccfd3…` (`verified`, `class_cells=0`,
  `shakedown_credit=0`), the c96 table `bef35931…` and index `8bbd71a7…` (historical, never a
  credit).
- The observation policy and admission protocol: brief 743 (Stage C: the ten class members,
  all-or-nothing, 20 cells; the C10 calibration; the importers' exact lines) and brief 744
  (the shakedown's 244 cells; the 18 own rows; the negative controls; the held 745/746).
- The post-754 changes: the D.2 package's nine live paths (brief 805; the joint price
  13,556,424), the ten rendered outputs and the R4 verifier (brief 806; `R4 LIVE VERIFY PASS
  mapping=190 … content_delta=3 doc_extras=36`; the four role roots baseline `cb9205a6…`,
  real `dda9146e…`, disabled `30cd48fc…`, armed `df0db63c…`), the class repair line's final
  state (756-797: the Q25 contract, the fixtures, the source A/B, the render/install), the
  fresh epoch's policy and receipts (807f: the v2.5 policy `08c03054…`, the nine qualified
  draws, the 120 labels), and the sealed 809 record (report `ba3a733b…`, seal 13,108).
- The Q25 contract's thirteen named stops (P0214, P0216, P1070, P2348, P2354, P2379, P2381,
  P2385, P2387, P2409, P2411, P2416, P2418); 744/745/746 held until their gates; CM5
  unauthorized.
- Read the sealed records, never the live checkout, for every program, selection and baseline;
  the live tree is authenticated only as the build input.

## Stage A — entry authentication (600 s)

Authenticate, by hash identity, the records above: the 754 epoch identity and its twelve
labels and three trees; the 743/755 observation policy, schedule, admission, packet index and
C10 packet; the 744 attempt-3 record and the consumed-cell ledger; the class repair line's
final accepted state (its seals and receipts); the 806 install (the ten outputs, the verifier
line, the role roots); the 805 landed sources; the 807f record (the policy, the qualified
draws); the 809 record (the report, seal, pins). Census the live tree and record the exact
file count and digest the epoch build would start from. Hash only evidence identity and
untracked `.temp` artifacts (never re-hash git-tracked source, per the standing ruling); use
mtime+size pre-screens for the census. Record every pin PASS/STOP and the exit census.

## Stage B — the qualification state and the delta (1,500 s)

Determine, from the records and with exact evidence, the campaign's qualification state:
whether the 754 epoch plus the class repair line plus the 806 install plus the 807f fresh epoch
constitute a qualified epoch for the campaign, or the campaign epoch must be rebuilt on the
current tree (with the evidence for the answer, the ruling numbers, the hashes and the deltas
since the 754 epoch). Inventory the delta: the nine D.2 live paths, the ten rendered outputs,
the class repair line's changes to the class-form render/observation, the live crates, and
every 754-epoch input (the observation policy, the render schema, the membership, the
selections, the program matrix, the own tuples) with its current validity.

Specify the epoch rebuild: the twelve release labels and the three trees, the one-codegen-unit
offline build rule, the functional equality (the 120 ordered labels `c193d87f…` and the
canonical program), the six-comparison manifest, the qualification gates (the v2.5
interference budget where a hardware-timed run is required; the functional/observational rules
where not), and the exact build counts and pins.

Specify the readmission: the ten class members, the 20 logical own cells, the ledger treatment
against the 510-cell schedule (resolve whether a readmission re-spends or re-credits the 20
cells, with the sealed schedule as authority), the C10 packet (re-taken or re-verified by
identity), the importers, the ordinary negatives, and the entry/exit gates.

Specify the 744 restart: the 244 cells (18 own rows × 2, the 100 pair selections × 2, the four
structural controls × 2), the negative controls (the eight observer, ten source-auth and four
Q7; the generator continuation/token negatives), the schedule's `stage=744` subsequence, the
re-freeze of the 246 for 745, and the held Q25 rows. Name every missing measurement.

## Stage C — the plan and the three implementation briefs (1,300 s)

Produce the step-by-step plan: the epoch build (files/functions per step, the invariants, the
gate tests, the call-site/ripple inventory, the risks), the readmission, and the 744 restart,
each with `path:line` evidence and the exact commands, inputs and bars. Draft the three
implementation briefs in the TEMPLATE shape (the epoch build, the readmission, the 744
restart), each with its goal, context pointers, stages, stop rules, hard rules, charge and
report format. Answer every open question with a recommendation; no `TBD`, no
`director-selected X`, no unstated choice.

## Stage D — the zero-unknown check (500 s)

Verify the plan against the evidence: every input pinned, every schedule cell accounted, every
ledger number reconciled (20 + 244 + 246 = 510), every pinned hash re-checked from the
records, every claim carrying `path:line` or a sealed record, every unknown named with the
exact missing measurement. Reconcile the cell arithmetic and the epoch's build counts. A plan
that depends on an unauthenticated or an unstated input is a STOP, not a pass.

## Stage E — report, check, seal (300 s)

Write `.temp/brief-811/report.json` as `rum-report/1` with the pins, the qualification
determination, the delta, the three plans and drafts, the open questions and the charge; check
it with `python3 -I scripts/report.py check .temp/brief-811/report.json`; detach the summary;
seal and rehash; audit the exit.

## Stop rules

Stop with the exact evidence on: an entry pin mismatch; a qualification determination that
needs a measurement not in the record; a schedule/ledger reconciliation that does not close; a
missing or contradictory record; a plan step that would edit the live tree, a sealed artifact
or a tool; a proposed hand edit of a derived file; a dependency on network, a new dependency or
`unsafe`; a projected overrun of a stage box; a stage reaching its deadline. A stop is a
correct terminal state.

## Hard rules

Design-only (`Scope: NONE`): no source edits, no builds, no captures, no acceptance suites, no
git of any kind, no network, no new dependency, no `unsafe`; `.temp` only, never `/tmp`; never
read `~/.config/ccds/api-key`; never edit a sealed artifact; the 743/744/754/755 and 798-809
records stay sealed and must survive to the epoch's final gate; Q25's thirteen named stops
stand; 744/745/746 held until their gates; CM5 unauthorized.

## Charge

4,200 s of the 6,041.152 s pool: **A 600 / B 1,500 / C 1,300 / D 500 / E 300**. 810 stays
cancelled; the implementation briefs carry their own explicit top-ups at dispatch (user ruling
2026-11-19). No production change; the design work charges the pool only.

## Report format

`rum-report/1` at `.temp/brief-811/report.json`, stage `A / c206`; `summary`, `changes`,
`deviations`, `questions`, every verdict line quoted, and the final message
`scripts/report.py summary`. The director gates from raw evidence.

## Checkpointing

Append `CLOCK stage=<A|B|C|D|E> t=<monotonic_ns>` to `.temp/brief-811/PROGRESS.md` at every
stage entry and exit. Each stage's box is measured from its own CLOCK entry; the charge is
continuous from dispatch; a stage reaching its deadline stops there with the exact missing cell
and its elapsed time, never kept open. Save each immutable cell before replacing a staging
tree; at each checkpoint record the exact next cell and any live dirt attributable to this
brief.
