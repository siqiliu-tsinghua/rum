# Brief 676 — LP08 design: schema-v2 snapshots and the non-encoding counter-manifest promotion

**Design for the live tree: `Scope: NONE — no live source, workload,
manifest, inventory or version-control change.` Disposable build probes
only, under this brief's root.**
LP07 closed at `a9cb067d`: the Module total/body/initializer records and
the source aggregates landed, Amendment 3 removed the orphaned outcome
helper, and the tree is green. LP08 opens now from the brief-624 raw
draft and its design record — but that record predates LP03-LP07's
adopted keys, their token-first recorders and the amended loss-ledger
bar (brief-637), so none of its counts, line references or coverage
numbers may be used as evidence without a re-derivation on the current
source.

This round answers one question with measured evidence: **what exact
form publishes the additive schema-v2 snapshot and promotes the
complete non-encoding counter requirement set on the current tree, at
what coverage and with what disabled-path cost — and which deliverables
are already closed by LP03-LP07?**

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **Current adoption state.** For every LP08 deliverable, whether it is
   already satisfied on the current source (with the row and its
   evidence) or still open, and the current `path:line` of every named
   site. The 624 draft's "print_lexical_profile_snapshot /
   print_runtime_path_timing_summary / print_builtin_timing_summary;
   eval_rum_profile_snapshot/reset; exhaustive manifest promotion" and
   its proposed files are claims to re-derive, not a map. An adopted
   cell is named and dropped, not re-opened.
2. **The form.** For each open deliverable, the measured form:
   - the additive schema-v2 snapshot: a labelled epoch header with
     validity, every logical key exactly once in stable key order with
     the family's final vector plus explicit open/abandoned/cancelled
     accounting, and separate labelled telemetry and duration sections;
     the existing ranked human timing displays remain additional
     summaries only — neither rank nor the current 64-row cap may
     truncate the logical rows; `RumProfileSnapshot` still returns
     `Null` and does not finish active calls; no new guest builtin and
     no new public host type;
   - the lifecycle cases: a snapshot taken mid-evaluation (its own
     entered invocation is open while it runs), more than 64 distinct
     keys, repeated equal keys, nested reset, old-epoch markers, a
     Session resumed on another thread, and Drop-before-summary;
   - the counter manifest: every schema field reconciled against
     `inventory/other-counters.tsv`, every timing phase family and
     route site reconciled against discovery; no runtime/builtin/
     definition calls/ok/err/some/none/return/throw may be reclassified
     as telemetry to gain parity; input_required, pause, segment and
     duration fields stay separate and explicitly excluded;
   - the activation: required/bound/executed keys exercised
     independently of fixture generation as `phase_observation.counters`
     alongside trace, preserving every variant, route, owner and exact
     multiplicity; all 16 minima, the 48 correctness fixtures, every
     active non-encoding frozen tuple and every added phase/flag/build/
     lifecycle route execute; the 240 encoding-owned requirements are
     preserved without a new encoding;
   - the promotion: its exact semantics, the exact files and rows it
     changes, and atomicity. It may fire only when every active matrix
     and every own mutation is green. The current ledger holds 103,759
     records — 7,960 key records, 91,424 loss cells and 4,375 executed
     receipts, stage `unpromoted known losses` — so the record must map
     every remaining loss cell to the promotion that retains it and
     prove that none is in the LP08 promotion set; a cell that belongs
     to the set blocks the promotion, and the record then names the
     blocking rows and the exact work or the ruling it needs. Widening
     the promotion set or rebaselining a cell is not a result;
   - the line's own mutants: counter red with semantic equality
     disabled for every adopted phase/site/route — completion
     suppression, duplicate completion, wrong Some/None/Return/Throw, a
     missing resumed mirror or source restoration, token mispair, and
     omitted reset/drop cleanup — with the real candidate SAME and zero
     structural faults. A base loss is never mutant credit.
   Each form carries its measured disabled-path cost (instruction slope
   and stack at one source path, CGU=1, two independent builds) and its
   enabled coverage row; a form family without a measured exclusion is
   not closed.
3. **The inventory.** The current route groups, path sites and
   definition-rule starts, re-derived from the source and paired
   against the 624 tables; every site whose attribution changes named
   with its `path:line`; and the current loss ledger split into the
   cells the promotion retains and any cell this line must act on.
4. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions —
   plus the drafted implementation brief in `TEMPLATE.md` shape.

If a deliverable has no zero-cost form under the pinned constraints, the
record is a measured exhaustion proof with the minimum-cost form named;
"impossible" without a measured exclusion of every family is not a
result.

The carrier semantics, the LP03-LP07 adopted keys, the token-first
recorders, the public API (except the additive schema v2 specified in
`docs/dev/profiling.md`), features and serde are **not** reopened.

## Context pointers

- `.temp/brief-624/drafts/LP08-snapshot-manifest-promotion.md` — the raw
  draft to re-anchor. Read it first, then treat every count and
  `path:line` in it as a claim to re-derive.
- `docs/dev/profiling.md` — the approved target contract: the snapshot
  schema-v2 section, the telemetry/duration separation, the lifecycle
  equations and the owner table.
- `.temp/brief-624/DESIGN.md`, `inventory/path-sites.tsv`,
  `expanded-paths.tsv`, `routes.tsv`, `profile/tuple-port.tsv`,
  `profile/layout.log`, `correctness/summary.tsv` — the boundary
  inventory; read-only, copy before adapting a script.
- `.temp/brief-621/MANIFEST-DESIGN.md` — the promotion model: an owner
  promotion activates every one of its rows atomically, the 240
  encoding-owned tuples keep their disposition, and C2/S10 keep their
  separate full obligations after this counter promotion.
- `.temp/brief-673/report.json`, `.temp/director/brief-673-gate.md`,
  `.temp/brief-675/report*.json` and
  `docs/dev/codex/brief-675-lp07-source-stage-module-completion.md` —
  the state LP08 starts from: the landed LP06 frame repair, the LP07
  source gate, the seven standing skips and the non-deciding timing
  ruling.
- Live sites: `crates/rum-runtime/src/runtime/profiling/`
  (`summary.rs`, `report_timing.rs`, `report_rebuild.rs`, `state.rs`,
  `reset.rs`, `formatting.rs`, `records.rs`, `counters.rs`, `logical/`),
  `runtime/debug_tools/basic.rs`,
  `runtime/evaluation_cache/profile/report.rs`; the oracle:
  `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors}.rs`,
  `support/logical_profile.rs`, `fixtures/logical_profile/`,
  `inventory/logical-profile-{required,bindings}.tsv` and
  `inventory/logical-profile-losses.json`; the host lifecycle test
  `crates/rum-host/tests/logical_profile_lifecycle.rs`.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md` — the standing duties and
  the testing policy.

## Scope

Files you MAY write: anything under `.temp/brief-676/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-675/`, `.temp/brief-674/`, or any retained root, except
  to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing.

## Hard bars

- The design's instruments are calibrated before any form is judged: the
  instruction slope, the stack probe and the oracle must reproduce a
  known current row. A probe that cannot reproduce the finding cannot
  judge a form.
- Every count, route group and `path:line` is re-derived from the
  current source; a 624 number is a claim, never evidence.
- A disabled-path cost claim needs the one-source-path, CGU=1,
  two-independent-build instruction/stack measurement; savings elsewhere
  do not substitute. An adopted cell already closed by LP03-LP07 is
  named with its row and dropped.
- A form that changes a carrier semantic, an adopted key, the completion
  order, the pushed value, the public API, or hides a loss is rejected,
  with the measured red row named.
- The promotion's precondition is answered on the frozen ledger. A
  rebaseline, a weakened assertion, a widened promotion set or a
  "retained" cell without its named retaining promotion is not an
  answer.
- No live source edit, no version-control command, no network, no new
  `unsafe`, no manifest or workload change.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit for every comparison;
serial Callgrind, CPU-pinned, release, every `RUM_PROFILE_*` cleared
except the declared modes. A timeout is not data. Persist raw output
before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-676/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-676/report-design.json` in `rum-report/1`, with the
  measured rows, the adoption state, the form families, the inventory
  re-derivation and every open question for the director.
- `.temp/brief-676/drafts/implementation-lp08.md` — the drafted
  implementation brief in `TEMPLATE.md` shape, plus the probe trees,
  binaries and receipts that support it.
- Run `python3 -I scripts/report.py check 676` and return its summary and
  all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one answer
(measured) and the draft's bar list.
