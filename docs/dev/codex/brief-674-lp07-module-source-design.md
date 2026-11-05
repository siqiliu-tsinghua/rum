# Brief 674 — LP07 design: Module records and source-attributed aggregates

**Design for the live tree: `Scope: NONE — no live source, workload,
manifest or version-control change.` Disposable build probes only, under
this brief's root.**
LP06 closed at `5446cacc`: the profiling line's Function/status/builtin
and While work landed, the frame repair returned `try_apply_function` to
no growth, and the tree is green. LP07 opens now from the brief-624 raw
draft and its design record — but that record predates LP06's adopted
keys, token-first recorders and A1/A2 additions, so none of its counts,
line references or coverage numbers may be used as evidence without a
re-derivation on the current source.

This round answers one question with measured evidence: **what exact
form adopts the Module total/body/initializer records and the
source-attributed aggregates on the current tree, at what coverage and
with what disabled-path cost — and what is already closed by LP06?**

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **Current adoption state.** For every LP07 deliverable, whether it is
   already adopted on the current source (with the row and its evidence)
   or still open. The 624 draft's "Module total :69, initializer loop
   :96/:197, initializer evaluate :160/:266, body evaluate :125/:227"
   and its four legacy frames must be re-derived to current `path:line`;
   an adopted cell is named and dropped, not re-opened.
2. **The form.** For each open deliverable, the measured form:
   - Module total/body/initializer records across native reentries, each
     actual initializer/body distinct, a resumed initializer loop using
     the original `RecordId` with a segment alias and never a new logical
     call;
   - initializer completion before store/advance, body completion before
     module-scope cleanup, whole-loop and Module total after
     `finish_module_scope`, through all four legacy frames and repeated
     pauses;
   - `eval_module` and `eval_module_by_source` aliased to total
     completion, with compile-bindings/allocation/init-body substitution
     synchronous and source-attributed entry/application counts intact;
   - source scopes restored for pure work entered after resume, so
     non-suspending logical child keys are attributed exactly as on the
     queued path — no heuristic equal-key reopening and no retrospective
     success synthesis;
   - cancel/reset/abandon storage balance including Module gensym cleanup
     and cleanup that suspends, with only this line's exact ledger losses
     removed.
   Each form carries its measured disabled-path cost (instruction slope
   and stack at one source path, CGU=1, two builds) and its enabled
   coverage row; a form family without a measured exclusion is not
   closed.
3. **The inventory.** The current route groups, path sites and
   definition-rule starts, re-derived from the source and paired against
   the 624 tables; every site whose attribution changes named with its
   `path:line`, and the current `logical-profile-losses.json` cells split
   into this line's losses and the standing ones.
4. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions —
   plus the drafted implementation brief in `TEMPLATE.md` shape.

If a deliverable has no zero-cost form under the pinned constraints, the
record is a measured exhaustion proof with the minimum-cost form named;
"impossible" without a measured exclusion of every family is not a
result.

The carrier semantics, the adopted LP06 keys, the token-first recorders,
the public API, features and serde are **not** reopened.

## Context pointers

- `.temp/brief-624/drafts/LP07-module-and-source-aggregation.md` — the
  raw draft to re-anchor. Read it first, then treat every count and
  `path:line` in it as a claim to re-derive.
- `.temp/brief-624/DESIGN.md` — the contract and mechanism record; its
  remaining decisions; the per-step measurement suite it names.
- `.temp/brief-624/` — `inventory/path-sites.tsv`, `expanded-paths.tsv`,
  `routes.tsv`, `profile/tuple-port.tsv`, `profile/layout.log` and the
  correctness summary. Read-only; copy before adapting a script.
- `.temp/brief-673/report.json` and `.temp/director/brief-673-gate.md` —
  the state LP07 starts from: the landed source, the seven standing
  skips, the timing protocol and its non-deciding ruling.
- Live sites: `crates/rum-runtime/src/runtime/scoping/lexical/module.rs`
  (the Module body, initializer loop, four legacy frames and
  `finish_module_scope`), `profiling/records.rs`,
  `profiling/counters.rs`, `profiling/logical/{mod,record,context,segments}.rs`,
  `evaluation_cache/profile/{source,mod}.rs`, the profiling tests, the
  `suspension_oracle` fixtures and inventory.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md` — the standing duties and
  the testing policy.

## Scope

Files you MAY write: anything under `.temp/brief-674/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-673/`, `.temp/brief-672/`, or any retained root, except
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
- Every count, route group and `path:line` is re-derived from the current
  source; a 624 number is a claim, never evidence.
- A disabled-path cost claim needs the one-source-path, CGU=1,
  two-independent-build instruction/stack measurement; savings elsewhere
  do not substitute. An adopted cell already closed by LP06 is named
  with its row and dropped.
- A form that changes a carrier semantic, an adopted key, the completion
  order, the pushed value, the public API, or hides a loss is rejected,
  with the measured red row named.
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

All scratch under `.temp/brief-674/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-674/report-design.json` in `rum-report/1`, with the
  measured rows, the adoption state, the form families, the inventory
  re-derivation and every open question for the director.
- `.temp/brief-674/drafts/implementation-lp07.md` — the drafted
  implementation brief in `TEMPLATE.md` shape, plus the probe trees,
  binaries and receipts that support it.
- Run `python3 -I scripts/report.py check 674` and return its summary and
  all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one answer
(measured) and the draft's bar list.
