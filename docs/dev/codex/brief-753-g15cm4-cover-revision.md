# Brief 753 — CM4 authorized cover revision (the six live cover rows)

**Director-dispatched, one step, one gate.** This applies the six cover revisions that brief
752 proved — P0249, P0339, P0350, P0351, P1155, P1610 — to the live coverage table, and
regenerates the successor selection records in the scene. It changes no source, renders
nothing, builds nothing and touches no epoch. The old selection scenes stay read-only
reference records: never edit an allowlist to bless drift.

## Goal

When this brief is done, the six own coverage cells in
`crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutant-coverage.tsv` carry the
proven replacement programs and tuples, byte-exactly as recorded in
`.temp/brief-752/evidence/revision-and-forecast.json`; the other 249 records, the header, the
row order and the line endings are byte-identical; a successor `selected-own-tuples.json` and
`frozen-schedule.json` are generated under `.temp/brief-753/successor/` by replaying the c7
coverage join, showing exactly the six changes plus P2593's retained override against the
historical C1 list; and the report proves the live diff is exactly those six lines with the six
new programs still bound and the policy and schema records untouched.

## Context pointers

- Read `AGENTS.md`; `.temp/director/ruling-753.md` (this gate); `.temp/director/ruling-752.md`.
- The authority for every substitution: `.temp/brief-752/report.json` and its evidence
  `.temp/brief-752/evidence/revision-and-forecast.json` — per row `live_coverage_line.line`,
  `proposed_tsv_line`, `new_selected_record`, `new_schedule_record`, `record_origins`,
  `regeneration`; the cuts and partition in `.temp/brief-752/evidence/added-cuts.json` and
  `file-partition.tsv`; the zero-unknown table in
  `.temp/brief-752/evidence/hit-side-edges-and-values.tsv`.
- The live join rule: `.temp/brief-740/c7/freeze_requirements.py:25-32` (coverage 255 rows;
  own programs resolved from `programs.tsv`; the P2593 override) and
  `.temp/brief-740/c1/assemble_lane_mutations.py` for the programs.tsv record shape.
  **Read-only.**
- The live registries: `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/`
  `programs.tsv`, `bindings.tsv`, `mutant-coverage.tsv`; the policy
  `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/observer-policy.tsv`.
- The parse shape: `crates/rum-stdlib/tests/suspension_oracle/mutations/schema.rs:70-100`
  (`include_str!`, header equality, six columns, unique `(kind,row,tuple)`, hex decode).
- The sealed epoch and all `.temp/brief-740/*`, `.temp/brief-743/*` scenes and everything
  sealed (c76–c99): **read-only reference records.**

## Scope (`Scope: LIVE DATA, six lines`)

- MAY modify: exactly six records of
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutant-coverage.tsv`
  (lines 13, 37, 40, 41, 96, 115 as measured today), substituting each recorded
  `live_coverage_line.line` with the recorded `proposed_tsv_line` and nothing else.
- MAY create/modify: everything under `.temp/brief-753/` only.
- MUST NOT: touch any other live file, tracked or untracked; edit `.temp/brief-740/*`, the
  sealed epoch, any older scene, the policy or schema tables, `programs.tsv` or
  `bindings.tsv`; render a cut or a postimage; build, cargo, compile or run an epoch binary;
  git; network; a new dependency; `unsafe`. No row reclassified, no predicate weakened, no
  hit moved.

## Deliverable

1. **The six substitutions.** For each row read the current live line and assert it equals
   `cover_revision[].live_coverage_line.line` byte-for-byte; if any differs, STOP before
   writing (the live table drifted). Build the complete revised content in memory, assert the
   header equals `kind\trow\ttuple\tprogram\tbar\tconfigurations`, assert 255 records, assert
   exactly the six lines differ from the current file (249 identical), then write once and
   re-read to verify. Report each old and new line verbatim.
2. **The invariant proof.** After the write: 255 records (151 own + 100 pair + 4 structural);
   the six rows carry the new program and the decoded new tuple; every other line
   byte-identical; the file's header, row order and newline convention preserved; P2593's row
   untouched; the policy digest `3953aa9f0b03…` and the overlays schema
   `22751919dbcb…` unchanged; the sealed epoch copy of `mutant-coverage.tsv` untouched
   (digest `6665e35b06c7…`); the before/after digest of the untracked live table recorded as
   evidence identity.
3. **The binding proof.** For each new program: exactly one record in `programs.tsv` at the
   recorded registry line with the matching row, family, variant, context, consumer route and
   descriptor; exactly one `bound` record in `bindings.tsv` for the same row/program; the
   recorded `new_selected_record` fields match the registry record; the tuple hex in the
   proposed line decodes to the new tuple.
4. **The successor selection records** under `.temp/brief-753/successor/`:
   - `frozen-schedule.json`: the 255 coverage records in file order
     (`{kind,row,tuple,program,bar,configurations}`, tuple as the file's hex), equal to the
     pre-revision schedule except the six rows, which equal the recorded
     `new_schedule_record`s.
   - `selected-own-tuples.json`: the 151 own records in coverage order, each the full
     programs.tsv record in the c7 shape (`program, family, row, variant, context,
     consumer_route, descriptor, source_sha256, setup_sha256, pauses, trace, canonical_input,
     canonical_fingerprint`); equal to `.temp/brief-740/c7/selected-own-tuples.json` except
     exactly the six rows, which equal the recorded `new_selected_record`s.
   - `changes.json`: the six row IDs with old/new program and tuple; the exact change list
     against `.temp/brief-740/c1/selected-own-tuples.json`, which must be exactly
     `P0249, P0339, P0350, P0351, P1155, P1610, P2593` in coverage order (the six plus the
     retained override), with P2593's program unchanged.
   - State that these are successor records; the `c1`/`c2`/`c7` and epoch scenes were not
     edited (verify mtimes/digests before and after).
5. **The parse-shape check.** In the scene, replicate the `schema.rs` assertions over the
   revised table: header equality, six tab-separated columns per record, unique
   `(kind,row,tuple)`, every non-`-` tuple decodes from hex, no `guest_heads.` substring. State
   that no build runs, because the change is data consumed by an `include_str!` parse whose
   consumers (attempt 3's render and epoch audits) build next; a malformed row is caught by
   this check now and by the compiler/test lane at attempt 3.
6. **The revision record.** The exact six old→new assignments in the report's `changes` entry:
   the one live path, the six line numbers, each old line and new line, and the before/after
   digest. `changes` must be non-empty and name only that file.

## Hard rules

- The only live write is the six-line substitution in `mutant-coverage.tsv`. Any other live
  write is a breach; if the substitution cannot be made byte-exactly, STOP without writing.
- Preserve kind, bar, configurations, row order and header exactly. Preserve the file's
  trailing-newline convention.
- The old scenes are reference records: compare against them, never edit them. The successor
  records live only under `.temp/brief-753/`.
- No render, no postimage, no epoch, no build, no cargo, no git, no network, no `unsafe`.
  Scratch in `.temp/`, never `/tmp`.
- The standing state stays untouched: pair `(45,5,false)` Open at zero admitted receipts; C2
  118 Residual + 0 Lost; 51 owners and five source duties Open; membership 28/3/7/113;
  `CHANGELOG.md` and S10 blocked; CM5 unauthorized; everything sealed (c76–c99) stays sealed;
  745/746 held.
- Save every command's full output under `.temp/brief-753/`; paste verbatim evidence for every
  claim (line numbers, old/new lines, registry lines, digest values).

## Verification

- The six old lines match the 752 records exactly; the six new lines equal the recorded
  `proposed_tsv_line`s exactly; 255 records; exactly six lines changed and 249 unchanged;
  header, order and line endings preserved.
- The six new programs are bound and registered with the recorded fields; the tuples decode.
- The successor `selected-own-tuples.json` differs from the c7 file in exactly the six rows;
  `frozen-schedule.json` differs from the pre-revision schedule in exactly the six rows; the
  change list against C1 is exactly the six plus P2593; the epoch and old scenes are untouched.
- The policy and schema digests are unchanged; the sealed epoch's coverage digest is unchanged.
- The parse-shape check passes on the revised table.
- Budget: 1,200 s focused from the stage reserve, 0 workspace, 0 epoch; every bounded command
  debits by monotonic elapsed time; a projected overrun stops for a director ruling, never a
  scope cut.
- Before finishing run `python3 scripts/report.py check 753` (without `--against-git`; the
  edited file is untracked, so the `changes` entry carries the path and the six line pairs)
  and fix every PROBLEM; the final message is only `python3 scripts/report.py summary 753`.
  STOP at the revision and the successor records: no render, no epoch, no dispatch of 749
  attempt 3, Stage B/C or 744/745/746.

## Report format

`rum-report/1` at `.temp/brief-753/report.json`; sections: the six substitutions (verbatim old
and new lines), the invariant proof, the binding proof, the successor records and the seven-way
change list, the parse-shape check, the untouched-state proof, the Open ledger state;
`changes` = the one live path with the six line pairs; disclosures for anything unexpected.
