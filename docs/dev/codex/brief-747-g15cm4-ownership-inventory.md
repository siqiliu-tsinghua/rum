# Brief 747 — G15-CM 4: the complete ownership inventory for the 151-row generator (design-only)

Worker: `cdx`. Dispatched by `scripts/dispatch-brief.sh 747`.

## Goal

Produce the **complete per-row ownership and transport inventory** for the
CM4 generator over all 151 frozen own rows, plus the composition plan and
the drafted implementation briefs, so that the implementation can be
dispatched **once** instead of discovering one ownership frontier per run.
This is a design-only brief: `Scope: NONE` — no maintained source, inventory
or schema change. The deliverable is an entry-pinned, machine-readable
matrix with a **zero-unknown-rows** statement, a decision on Q24 for every
dependent row, and a named `PortBlocked` class wherever a binding cannot be
expressed under the standing constraints.

The immediate question the inventory must settle is Q24 from brief 742's
fifteenth run: the Q23 carrier keys a token by
`SuspendedEvaluation.generation`, but `SuspensionSlot::next_generation` is
slot-local (`Session::empty` builds a fresh slot per session,
`model/session.rs:115`), so two distinct slots both install generation 1
and the unrelated append scope consumes the token while the intended one
loses its hit. The inventory must select exactly one of: (1) an
authenticated single-slot execution domain proved from the actual runner
sources, with the equal-generation cross-slot control explicitly ruled
outside that domain and the falsifier retained in its original form; (2) a
source-bound owner namespace that disambiguates actual slots or suspensions
at both the arm and the publication using existing objects on existing
paths (for example the request-interrupt control that
`begin_input_suspension` already registers against the queue), with no new
field, accessor, allocation, overlay path or production surface and with
the cross-slot falsifier passing; or (3) the J class and every dependent
row named `PortBlocked`, with the bar unchanged. No global uniqueness may
be claimed from a per-slot counter, and the required falsifier may not be
replaced.

## Context pointers

- Read `AGENTS.md`; `docs/dev/codex/TEMPLATE.md`; brief 741 (the Q9
  generation method: 151-row/ten-class taxonomy, five-step generation
  rule, 38-row class observation policy, 510-case attempt plan); brief 742
  in full, especially the Continuation 1–15 rulings (Q10–Q24) — they are
  the recipe ladder to be applied, not re-litigated.
- Brief 623's `### Continuation 90–101` sections carry the run-by-run
  state and the standing campaign invariants.
- The 90-file overlay set:
  `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv`
  (single source of truth for the instrumentable surface; every recipe
  must stay inside it, and every maintained source must be read from the
  live tree).
- The current partial generator and its audits:
  `crates/rum-stdlib/tests/suspension_oracle/mutations/` (the uncommitted
  23-entry partial CM4 test layer) and the scratch audits under
  `.temp/brief-742/continuation-*/` (read-only raw material; reuse the
  entry-pinned audit form: live source + entry hashes + extracted-statement
  controls).
- The frozen pins: the 18 caught + 4 stop rows per mode with their
  historical signatures (brief 740's c7 campaign logs), plus every later
  pin named by Q10–Q24; the 151 source/tail/destructor joins, the 90
  preimages / 180 postimages and the 5,399 maintained files must
  authenticate unchanged.

## Scope

- Files you MAY modify: `NONE — design only`. All output under
  `.temp/brief-747/` (scratch logs, generators, matrix files, report).
- Files you MUST NOT touch: everything under `crates/`, `docs/`,
  `scripts/`, `Cargo.toml`, `Cargo.lock`, `.git`, and any `.temp/` content
  outside `.temp/brief-747/`. You may READ anything.
- No network. No `git`. No campaign, epoch, acceptance, mutant build or
  inventory rewrite. No production change of any kind: the live sources
  and the working tree must be byte-identical to entry (the 23-entry
  partial CM4 test layer stays exactly as found).
- If you believe a binding requires a new production field, accessor,
  allocation or overlay path, that row is `PortBlocked` and named — never
  an exception.

## Deliverable

1. **The matrix** (machine-readable, e.g. `.temp/brief-747/inventory.tsv`
   + a JSON index): one row per frozen own row (151), with, for each
   pinned arm and each pinned hit and each generated recipe form:
   - row id, class, run family (default/no-default), side
     (queued/unqueued), and the frozen signature references;
   - the pinned arm event and hit event: site (`path:line`), phase,
     prompt, selector, observer/predicate;
   - the **actual executing object** and its construction identity (the
     runtime object, not a name or a label);
   - the **actual source edge** connecting them, as a chain of existing
     call edges with `path:line`, including the retirements, drops,
     borrows and lifetime bounds that the recipe must survive;
   - the transport recipe (one of Q10–Q23, or a newly specified one with
     the same form: issue site, carry, consume site, invalidation);
   - the isolation falsifier(s) and expected outcomes (`unrelated 0 /
     intended 1`, wrong generation/row/episode/path, queued zero), and
     the extracted-statement controls that must execute the audited
     statements rather than helper simulations;
   - the preserved nodes and the full-file AST restoration claim;
   - the shared-file composition: which other rows' recipes touch the
     same file, statement or call expression (e.g. `call.rs`'s prepared
     and profile forms, `core.rs`'s multiple sites, `profile_queue.rs`'s
     append/publication), and the exact order they compose in;
   - status: `expressible` with the recipe name, or
     `PortBlocked::<name>` with the missing edge/fact named.
2. **The Q24 decision** as a separate, argued section: the option chosen
   (single-slot domain, owner namespace, or named PortBlocked), the
   entry-pinned evidence, the retained cross-slot falsifier and its
   result, and the list of every row whose recipe depends on it.
3. **The generator plan**: the exact generated file set (paths), the
   renderer inputs and outputs, the deterministic double-generation
   procedure (two roots byte-identical), the mechanical `source_schema`
   transition (`f9604852…` to the new value, with its derivation), and
   the standing replay order (fingerprints, production acceptance both
   modes, ownership exactness, rejection census, LP01, typed-slot census)
   before the workspace gates.
4. **Zero-unknown-rows statement**: every one of the 151 rows classified;
   no row normalized; any row that cannot be classified is itself a named
   STOP with the reason.
5. **Drafted implementation briefs** in `TEMPLATE.md` shape: (a) the
   generator implementation (per-class recipes, helper, audits, the
   complete 151-plan bar), (b) the schema emission and authentication,
   (c) the replay + workspace gate stage. These are drafts; the director
   rewrites and commits them.

## Verification (MANDATORY — you run it)

Design-only, so the evidence is the deliverable and its reproducibility:

- A script under `.temp/brief-747/` regenerates the matrix from the live
  sources and the entry hashes, and the run pastes the reproduced table's
  checksum twice (two roots byte-identical).
- Every `path:line` citation resolves in the live tree; every quoted
  statement is the live statement (entry-pinned hashes recorded).
- The failed/attempted instrumentation runs and their corrections are
  retained, with the verdict lines; nothing is deleted or relabelled.
- `git status --porcelain` at exit is identical to entry (the 23-entry
  partial tree); `source_schema` unchanged; `production_changes=0`.
- Report as `.temp/brief-747/report.json` in `rum-report/1`, with STOP,
  checks, citations and the budget block; paste the key verdict lines
  verbatim.

## Budget and stop rules

- One focused envelope of **1,500 s** (allocated from the stage's
  69,000 s; no workspace envelope is granted and no workspace-wide test
  run is authorized). Charge measured serial command seconds plus the
  0.5 s bookkeeping and 60 s preparation/final-reporting reserve; stop
  before exceeding the envelope.
- One cargo command at a time, offline/locked, jobs 8, inside the 16G
  scope, every long command through `scripts/run-bounded.py`.
- Stop-loss keyed on progress: if two consecutive attempts classify no
  new row, stop and report the classified set and the remainder. Do not
  keep re-deriving the same frontier; a row that resists classification
  is `PortBlocked` with the exact missing fact, and the report names it.
- No implementation follows this brief. If the inventory cannot be
  completed, checkpoint `.temp/brief-747/PROGRESS.md` (done, next action,
  tree state) and say exactly what remains.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any maintained source, inventory or schema; your output
  is scratch and a report.
- Testing Policy is a hard line (`AGENTS.md`): no third-party engine
  comparisons, no committed foreign outputs.
- No new `unsafe` anywhere you touch.
- Minimal footprint: no drive-by edits, no reformatting, nothing outside
  `.temp/brief-747/`.
