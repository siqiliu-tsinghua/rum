# Brief 767 — G15-CM4 derived-inventory reconciliation: the complete inventory and design (design-only, `Scope: NONE`)

**Director-dispatched, one stage, one gate, no production change.** This continues ruling 775
and brief 766's accepted STOP. Brief 766 repaired the emitter, re-emitted the 27-file class
source, built the non-admission image green on repair 2 of 8, and ran the class fixtures.
They stop in `guest::programs()`: `load_programs_with` fails the provenance canonical-source
check at ordinal 1 (`crates/rum-host/src/session.rs`: sealed `ef78a1de…`, 33,545 B, equal to
the live file; both scene copies `7127298d…`, 36,731 B; the generator never edited it),
with ordinal 7 (`crates/rum-jupyter/src/adapter/worker.rs`) the same shape and 139 of 141
seals matching. The director's wider finding: the committed byte-pinning inventories are
baseline-derived and the accepted class-form pipeline changed their subjects. In the scene
copy 60 of the 199 provenance-pinned paths differ and they are exactly the files the
accepted 749 render changed (2 canonical-source, 56 guest-route-only, 1 guest-dependency
`runtime/profiling/logical/segments.rs`); the source A/B's `overlays.tsv` pins 135 files
whose postimages are stale for the 21 overlay files the 766 emission changed, and the
emission's 3 files outside the overlay set would fail `UnlistedFile`; the render changed 9
files beyond the overlay schema. **This brief must independently re-derive all of that,
close every row with zero unknowns, and design the reconciliation.** `Scope: NONE`:
inventory and design only; no source, inventory, generator, render, selection or epoch edit;
no build; no binary execution.

## Goal

When this brief is done, in `.temp/brief-767/` only: a **complete, zero-unknown inventory of
every committed inventory that pins source bytes, every consumer that checks those pins, and
every pinned path's disposition** under the accepted pipeline (baseline live tree → 749
render → 766 emission → the future deterministic render); the exact certificate that proves
each transformation preserved the canonical program text and the ordinary bodies; and the
**reconciliation design** — the decision among certified re-anchoring, authenticated-baseline
loading and route-text pins (or a staged combination), with its consequences for the
deterministic render and for the final live tree, and the drafts of the implementation
briefs. Every claim cites `path:line` or a raw log line. No live edit of any kind.

**Row-level stops.** A `stop` verdict is a row result, recorded with the exact missing fact;
it does **not** halt the scan of the remaining rows. Only a calibration failure (a known
answer derived wrongly) or an unresolvable chain link stops the brief.

## Context pointers

- Read `AGENTS.md`; brief 623's "Campaign invariants"; rulings
  `.temp/director/ruling-775.md` (this gate), `ruling-774.md`, and the accepted design
  `.temp/brief-756/design.md`, `.temp/brief-757/decision.md`.
- **The accepted pipeline (read-only).** The baseline tree is the live working tree (every
  pinned path verified by the director: all 199 provenance pins match it, all 135 overlay
  preimages match it). The accepted render is `.temp/brief-749/attempt-7/copies/{source,armed,disabled}`
  (`source` = the baseline; `armed`/`disabled` differ in 144 files; the director measured
  `render − overlays` = 9 files). The accepted emission is `.temp/brief-766/copy` (5,815
  files; 27 changed/added against the 764 copy) with its evidence
  `.temp/brief-766/evidence/class-forms.json` (24 inverses with byte pre/postimages and 97
  rows), `class-forms.json`'s 3 new files, `generator-edit*.json`, and
  `.temp/brief-766/logs/reemit-3.log` (the verdict lines).
- **The provenance inventory.** `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/provenance.tsv`
  (4,803,954 B, committed; the director's scan: 199 distinct pinned paths, 141
  canonical-source rows, 8,940 guest-route rows, 55 guest-dependency rows, one program-table
  row; the live tree matches all 199). Its producer is the archived
  `.temp/logs/archive/cdx_brief_739.fifteenth-run-q18-stop.brief/install_guests.py` (the
  canonical-source audit `:46-72` — 129 historical-equal + 4 probe-only differences, factory
  body unchanged — and the guest-route digest derivation `:102` `sha((root / source).read_bytes())`).
- **The provenance consumers.** In the accepted scene copy
  `.temp/brief-766/copy/crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/guest.rs`:
  `programs()` `:84`, `load_programs()` `:1319-1326`, `programs_with_mutation()` `:1329-1331`,
  `load_programs_with()` `:1332`, the canonical-source loop `:1351-1358` (every
  canonical-source row, unconditionally), the guest-route check `:1428-1442`, the
  guest-dependency check `:1560-1580`; and
  `mutations/source_auth/mod.rs`: `verify` `:106`, the inventory equality `:118-131`, the
  baseline provenance seal `:132-140`, the source seals `:141-158` (checked against the
  **baseline** snapshot), the overlay preimage/postimage checks `:173-196`, `authenticate`
  `:222-240` (`before = Snapshot::read(baseline)`, `after = Snapshot::read(workspace())`),
  `AuthenticatedSource::source` `:242-256` (checks the working file against the copied
  snapshot's digest and returns the **original** bytes); `coverage/bindings.rs:94` and
  `:106` (the compiled-in tables).
- **The overlay inventory.** `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv`
  (135 files × preimage + disabled/armed postimages; `schema.rs:9` includes it; `:24` pins;
  `:28` `file_postimage`; `manifests.rs:2` the 9 Cargo.toml `PINS`, unaffected by the
  render). The director measured: all 135 preimages equal the live tree; all 135 are
  render-changed; the emission's 24 changed files contain 21 overlay files (stale
  postimages) and 3 outside the set (`suspension/typed/chain.rs`,
  `suspension/typed/witness.rs`, `tests/suspension_oracle.rs`).
- **The class fixtures.** `.temp/brief-766/copy/crates/rum-stdlib/tests/suspension_oracle/class_lease_fixtures.rs:32-34`
  (the `guest::programs()` call that stops) and the 97×2 test structure it generates.
- **The measured state.** `.temp/brief-766/logs/source-identity-diagnosis.log` (the first
  link and the 139/141 statement), `logs/class-lease-fixtures.log` and
  `logs/class-lease-fixtures-threads-1.log`, `evidence/entry-identity.json`,
  `.temp/director/ruling-775.md` (the director's counts and the four blocks).

## Deliverables

1. **The byte-pinning inventory, complete.** Every committed or compiled-in artifact that
   pins source-file bytes by digest or by full postimage, with its producer, its consumers
   and the exact check: `provenance.tsv` (each row kind and the rows the loader and
   `source_auth::verify` actually check), `overlays.tsv` (preimage and both postimages),
   `manifests::PINS`, `coverage/bindings.rs`'s compiled-in tables, and any other pin found
   (disclose every one, including any in the untracked live state such as
   `inventory/obligations/{mutants,promotions,mutant-coverage}.tsv` and
   `mutations/semantic-programs.tsv`). For each: what the pin means, where the bytes come
   from, and every code path that compares them. Zero unknown kinds and zero unknown
   consumers; a check that cannot be confirmed is a named stop, not a guess.
2. **The per-path disposition, zero unknown.** For every pinned path: baseline = pin (yes or
   the exact mismatch); changed by the 749 render (yes/no; if yes, the preimage and postimage
   digests from `.temp/brief-749/attempt-7/copies/{source,armed,disabled}`); changed by the
   766 emission (yes/no; if yes, the pre/post digests from `class-forms.json`); in the
   overlay set (yes/no); checked by which consumer. The nine render files outside the
   overlay schema are classified with the reason they are outside (observer families? a
   different lane?) and a decision whether the source A/B must cover them. Reproduce the
   director's counts from the raw tables, not from this brief.
3. **The certificate.** For each transformation in the pipeline, the evidence that it
   preserves the canonical program text and the ordinary bodies: the render's per-file
   preimage→postimage map, the emitter's `ORDINARY BODY PASS functions=840
   shifted_paired=118` and `SPLIT PASS`, the 97 `ROW VALIDATE PASS` lines, the route-recipe
   containment, and what can be recomputed from the pinned baseline bytes at re-anchor time
   (the ordinary body per function, the route text, the overlay application). State exactly
   what each certificate forces and what it does not; a certificate that does not force the
   transformation it names is rejected.
4. **The reconciliation decision.** Evaluate at least these candidates against the standard
   (correctness, semantic loss, structural necessity, enforceability, measurability; a fix
   the compiler or a derive enforces beats one that relies on memory): (a) a certified,
   generated re-anchoring of the pins on the emitted tree, with the baseline→render→emission
   certificate recorded per file; (b) loading the program table through the authenticated
   source path (`AuthenticatedSource::source` returns the baseline bytes after checking the
   working tree) so the fixtures verify the transformation at run time; (c) re-expressing
   the pins over the canonical route text instead of whole files; or a staged combination.
   Decide, with the reasons the others are rejected, and specify: what the class fixtures
   load and from where; what the source A/B verifies on the emitted tree and with which
   baseline; what the deterministic render must regenerate or install (so the final live
   tree loads its own program table and every `guest::programs()` consumer passes); and how
   the regenerated artifacts are produced (a prescribed generator with a fail-closed
   certificate, never a hand edit). Do not weaken any guard and do not choose a stopgap as
   the final answer.
5. **The implementation brief drafts.** In `TEMPLATE.md` shape, the next implementation
   brief(s) with their exact bars, stop conditions, invariants, falsifiers and budgets: the
   inventory reconciliation, then the fixtures/Q25/replays/source A/B/probe/hot-path, and
   the deterministic render's inventory obligation. Keep them one gated step each.
6. **The evidence package and the report.** The inventory table, the disposition table, the
   certificates, the decision record, the drafts, the budget (focused only; no workspace, no
   epoch), and the zero-unknown statement. `scripts/report.py check 767` must pass.

## Hard rules

- `Scope: NONE`: no edit anywhere in the repository; work and scratch under `.temp/brief-767/`
  only; the live tree is read-only; the accepted scenes, the epoch and everything sealed are
  read-only; no build, no test execution, no binary execution; `.temp` only, never `/tmp`;
  no network; no new dependency; no git; never read `~/.config/ccds/api-key`.
- Re-derive every number from the raw tables and logs; quote the producer lines and the
  consumer lines verbatim. The director's counts in this brief are claims to check, not
  facts to copy.
- A wrong derivation in a calibration case stops the brief; a row that cannot be closed is
  a named stop with the exact missing fact, and the scan continues.
- No new `unsafe`; no default-path cost; no new program, lane, selection, policy or epoch
  label; no receipt is importable; Q25's thirteen rows stay named stops; 744/745/746 stay
  held; CM5 stays unauthorized.

## Budget

**focused 2,000 s + workspace 0 s + epoch 0**, carved from the corrected stage reserve
(34,770.78278847017 → **32,770.78278847017 s**); unspent returns; a projected overrun stops
for a director ruling, never a scope cut. The scene's stop line is `remaining < stack_1x`
(cold samples of pending commands plus the 180 s close) or `remaining < 3× the next command
sample`. No build is authorized.

## Report format

`rum-report/1` JSON at `.temp/brief-767/report.json` via `scripts/report.py`, sections:
inventory, consumers, disposition, certificates, decision, drafts, budget. The summary must
state the zero-unknown result, the decision and its consequences, and the exact open facts.
