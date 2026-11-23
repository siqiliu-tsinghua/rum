# Brief 777 — G15-CM4 Stage D repair: the source A/B on the installed tree

**Implementation brief, staged (A–F), one gate.** It continues rulings 777 and 785 and brief
767's draft `.temp/brief-767/drafts/05-source-ab.md`. The carried scene `.temp/brief-774/scene`
is sealed at brief 776's accepted state; this brief builds its successor, the **install tree**:
the live tree plus the accepted emission (the 768 re-anchored 147-row overlay set), the five
class additions, the class render and provenance table, and brief 776's adopted ownership
reconciliation. On that tree it proves the source gate: `source_auth::authenticate` must accept
the installed tree for both arms in both feature modes against a **constructed baseline built
from installed bytes**, with a fail-closed addition set and the eight drift paths preserved.
A front-half stop does not skip later evidence by shrinking it.

## Goal

When this brief is done:

- The install tree exists at `.temp/brief-777/install/` and is exactly the live tree with the
  declared install set (the 147 overlays at the re-anchored table's postimages, the eight
  additions, the twelve adopted paths, the two re-anchored tables, the F4 message fix) and
  nothing else; the live tree, the sealed scene and every `.temp/brief-76*` and
  `.temp/brief-74*` evidence tree are byte-untouched.
- The overlay table is re-anchored to the installed tree: 147 rows, with **exactly one**
  declared content delta from the 768 artifact (`.temp/brief-768/evidence/overlays.tsv`,
  `73297529…`): `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/guest.rs`,
  whose postimage cells become the installed class-form byte
  `1a2c5ff56e2c99d759b196a71de78fc62d13048f9879290d9186bd4a7c986129` (both arms; the 768
  cells carry the superseded `0e449d44…`). Every other row is byte-identical to the 768
  artifact, every preimage equals the live file digest, and every hex cell decodes to its sha.
- The re-anchored `mutants.tsv` equals the live table with **only** the `source_schema` column
  replaced by the new table digest (all 151 records; the other seven columns verified against
  the seal).
- `source_auth` carries a committed `additions.tsv` and a **fail-closed** addition-set rule:
  the eight files the install carries beyond live, the class table
  (`class-provenance.tsv`) among them, each pinned by digest; any other extra, any missing
  addition, and any addition in the baseline is `UnlistedFile`.
- The constructed baseline is built from installed bytes: the install tree minus the eight
  additions, with the 147 overlays restored to their preimage bytes (taken only from the live
  tree after the digest check). The **eight drift paths** are asserted explicitly: six keep the
  live entry bytes on both sides, `mutants.tsv` and `overlays.tsv` carry the re-anchored
  tables on both sides, and a control against the 749 source copy proves no stale source byte
  entered the baseline.
- The six-case source gate passes: the disabled and armed images authenticate in both feature
  modes with real receipts (arm, baseline tree, binary, codegen 1, profile release,
  configuration, tree, 147 overlay rows), the real image equals the baseline, the validator
  controls and the observer capability pass, and the six evidence cases are recorded with
  their verdict lines.
- The ordered fixes land: **F4** (the ledger message counts `Local`), **F3** (the receipt-diff
  checker becomes a substring match, self-tested, and the re-run shows `problems=0`). **F5**
  and the two Q25 files' formatting are deferred to the render brief by ruling 786's amendment
  (recorded at the gate): the module move shifts `lib.rs`'s line anchors and the retention
  fingerprints, so it must land with the inventory regeneration, and the render brief re-runs
  this gate on the final tree.
- The deferred 774 acceptance's gates are run on the install tree: one flag-on class-lease
  module run (`197 passed; 0 failed` under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`), the receipt
  diff with the fixed checker, one flag-off workspace run with the two named skips, and
  `cargo fmt --check`. The workspace's reds must be exactly the named inventory-regeneration
  class and the environment's loopback refusals; the full green belongs to the render brief.

Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Context pointers

- Rulings 777 and 785; `.temp/brief-767/drafts/05-source-ab.md` (the design);
  `.temp/brief-768/report.json` sections 3, 4, 6, 7 (the overlays, the addition set, the
  whole-tree declaration, the generator); `.temp/brief-769/report.json` section 5; brief 776's
  report and `docs/dev/codex/brief-776-…md` (the adopted scene).
- The consumer: `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/mod.rs` —
  `verify` at lines 107–216 (`OverlaySet` 125–133, the inventory rule 134–139, the seals
  140–175, the pins 176–188, key-set equality 189–190, the non-overlay rule 192–214),
  `authenticate` 218–241, `AuthenticatedSource::source` 243–259; `schema.rs` (`pins`,
  `file_postimage`, the one-constant hex cells); `files.rs` (`Snapshot` excludes
  `.git`/`target`/`.temp`/`__pycache__`/`.mypy_cache`, no symlinks); `tests.rs`
  (`cm_mutation_source_authentication_controls`, `cm_mutation_source_copied_admission`,
  `cm_mutation_observer_copied_capability`, `cm_mutation_source_export`, the route and type
  controls); `artifacts.rs` (`artifact(…, arm)`); `manifests.rs` (the nine pins, the
  unchecked lane only).
- The 768 evidence: `.temp/brief-768/evidence/overlays.tsv` (147 rows, `73297529…`),
  `additions.tsv` (five rows, `78106c66…`), `mutants-reanchored.tsv` (152 lines,
  `17355964…`), `class-provenance.tsv` (`8c8b2232…`), `inputs.tsv`, `source_schema.txt`.
  Note the 768 `additions.tsv` predates brief 774: `class_rows.rs` and
  `class_lease_fixtures.rs` carry stale digests there; the installed finals are `6da1dd03…`
  and `475d0edb…`.
- The drift enumeration: `.temp/brief-768/logs/entry.log` lines 18–48 and
  `.temp/brief-769/logs/entry.log` lines 18–52 (`DRIFT PATH`, `DRIFT SUMMARY content=8
  live_only=… source_only=0`); the eight are `CLAUDE.md`, `TODO.md`,
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutants.tsv`,
  `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv`,
  `docs/dev/codex/brief-623-g15c2-settle-unowned-census-rows.md`,
  `docs/dev/codex/brief-743-g15cm4-observation-policy.md`, and the two
  `tools/claude-mod/rum-director/.claude-plugin/types/*/index.d.ts` files. The 749 source copy
  is `.temp/brief-749/attempt-7/copies/source` (its `overlays.tsv` is `5c3b02a3…` — never a
  baseline source; its `mutants.tsv` is `e8b3b6e4…`).
- The build and receipt precedent: `scripts/probe-build.py` (the canonical build; release,
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, stable `.temp/probe/active`, content-addressed
  cache; **do not modify**), its 743 adapter `.temp/brief-743/epoch/probe_build.py`, and the
  receipt writer `.temp/brief-743/c90/build_block.py` (fields sorted; `overlay\tfile\tbefore\tafter`
  rows; `baseline_tree` and `tree` from `probe-build.tree_hash`). `scripts/run-bounded.py` for
  bounded commands. The accepted source-gate precedent:
  `.temp/brief-749/attempt-7/logs/432-closed-source-gate.log` and `128-source-gates.log`.
- The scene's accepted finals: class table (`class-provenance.tsv`) `8c8b2232…`,
  `provenance.tsv` `965eca10…`, ledger 92 entries / 93 lines, chain 152 rows
  (`crates/rum-stdlib/src/tests/ownership_graph/inventory/emission_chain.tsv`, guest.rs row
  already `1a2c5ff5…`), `class_rows.rs` `6da1dd03…`, `class_lease_fixtures.rs` `475d0edb…`,
  `class_lease.rs` `8f5ba488…`, `q25_ownership.rs` `43416be6…`, `q25_p2379_proof.rs`
  `ee789084…`.

## Scope

- You MAY write only under `.temp/brief-777/`: `install/`, `images/`, `baseline/`, the scripts,
  the logs, the evidence and the report. The live tree, `.temp/brief-774/scene`,
  `.temp/brief-749/attempt-7/copies`, `.temp/brief-766/…`, `.temp/brief-768/…`,
  `.temp/brief-769/…`, `.temp/brief-774/…` and `.temp/brief-776/…` are read-only.
- You MAY change, **in the install tree only** (never the live tree, never the sealed scene):
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/` — the new
    `additions.tsv`, the `verify` addition-set rule, its falsifier tests; nothing that weakens
    `Preimage`, `Postimage`, `BaselineSeal`, `Inventory`, `OverlaySet`, `UnlistedFile` or the
    arm bijection;
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv` — the
    re-anchored 147-row table from the single writer;
  - `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutants.tsv` — the
    re-anchored table from the single writer;
  - `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs` — the F4 message
    only; the ledger table and the other five tables are byte-unchanged.
- `.temp/brief-774/receipt_diff.py` — the F3 substring fix and a `--selftest`; every other
  line unchanged.
- No other file may differ from the install rule below; a disclosed extra stops for a ruling.

## The install tree (construction — exact)

1. Copy the live tree (`/home/remi/dev/rum`) excluding `.git`, `target`, `.temp`,
   `__pycache__`, `.mypy_cache`, symlinks and special files, to `.temp/brief-777/install/`.
   Record the live walk: the file set (5,840 files at entry), the tree hash (the
   `probe-build.tree_hash` algorithm), and the 23 inherited `git status` entries unchanged.
2. Read the sealed scene `.temp/brief-774/scene` (5,816 files at entry) and compute the
   live-versus-scene delta: `live_only` must be exactly the `docs/dev/codex/` brief documents
   absent from the scene (30 at entry: briefs 744 and 748–776; the count is derived, never
   pinned), and `scene_only` must be exactly the eight additions (`cm740_probe/class_lease.rs`,
   `cm740_probe/class_rows.rs`, `cm740_probe/q25_ownership.rs`,
   `suspension_oracle/class_lease_fixtures.rs`, `suspension_oracle/q25_p2379_proof.rs`,
   `suspension_oracle/inventory/obligations/class-provenance.tsv`,
   `crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs`,
   `crates/rum-stdlib/src/tests/ownership_graph/inventory/emission_chain.tsv`).
3. Install over the copy, and nothing else:
   - the **147 overlay paths** at the re-anchored table's armed bytes (equal to the scene's
     files except `guest.rs`, whose class-form byte the writer takes from the scene);
   - the **eight additions** from the scene;
   - the **twelve adopted paths** — every scene file that differs from live and is neither an
     overlay row nor one of the eight drift paths: `ownership_graph/mod.rs`,
     `ownership_graph/host_payload_ledger.rs`, the six ownership fact tables and the ledger
     (`ownership_graph/inventory/{self_reaching,host_payloads,unknown_attributes,unknown_captures,unknown_macros,unknown_storage,host_payload_ledger}.tsv`),
     `retention_closure/inventory/local-contracts.tsv`,
     `retention_closure/inventory/promoted.tsv`, and
     `suspension_oracle/coverage/bindings.rs` — derive the list from the delta, assert it has
     exactly these twelve members (the delta is 167 differing plus eight scene-only paths;
     167 = 147 overlay rows + the eight drift paths + these twelve non-overlay paths), and
     stop on any other member;
   - the two **regenerated tables** from stage B, replacing the live/source copies:
     `mutations/source_auth/overlays.tsv` and `inventory/obligations/mutants.tsv`;
   - the **six drift paths left at live bytes**: `CLAUDE.md`, `TODO.md`, the two
     `docs/dev/codex/brief-*.md` paths and the two `.d.ts` files (never the scene's stale
     bytes);
   - the F4 message edit (stage A, in the copy).
4. Assert the install tree's delta against live is exactly: 147 overlay files (all except
   `guest.rs` already equal to the scene), the eight additions, the twelve adopted (the F4
   line lives in one of them), the two regenerated tables and the new `additions.tsv` — 170
   paths; every other path byte-equal to live. Assert the 30 live-only docs are present at
   live bytes.
5. Entry pins: the scene's class table `8c8b2232…`, `provenance.tsv` `965eca10…`, the ledger
   93 lines / 92 entries, the chain 152 rows, the two class finals `6da1dd03…`/`475d0edb…`,
   and the scene's own walk. Re-hash the six ownership tables and the ledger from the scene;
   any divergence from brief 776's `INSTALL … equal=true` record stops.

## The re-anchoring (one writer, declared deltas only)

Write `.temp/brief-777/reanchor.py` (the only writer of the three tables). It:

- reads the install tree's 147 overlay files and the 768 artifact;
- asserts every preimage cell equals the live file digest and, for the 146 rows other than
  `guest.rs`, every sha/hex cell equals the 768 cell;
- re-derives `guest.rs`'s two sha cells and two hex cells from the install tree's byte
  (sha256 of the bytes; hex lowercase; the decoded bytes re-hash to the sha);
- writes the 147-row table with the 768 header
  `file\tpreimage\tdisabled_sha256\tarmed_sha256\tdisabled\tarmed`, rows sorted by file;
- applies the recorded one-constant rule to the three arm-differing files as a KAT: the rule
  recovered from the 768 rows (`lib.rs`: `pub const ENABLED: bool = !cfg!(disabled) &&
  !cfg!(live);` ↔ `pub const ENABLED: bool = false;`; `eval_dispatch/mod.rs` and
  `segments.rs`: `if true` ↔ `if false`) must reproduce the 768 disabled cell from the 768
  armed cell on all three rows, and the derived disabled bytes must equal the install tree's
  `images/disabled` bytes;
- writes `mutants.tsv` = the live `mutants.tsv` (`14bd4326…`) with only the `source_schema`
  column replaced by the new overlays digest on all 151 records; asserts the other seven
  columns equal the live table's and the file's 152 lines;
- writes `additions.tsv` under the install tree's `source_auth/`, header
  `file\tsha256\tbytes`, eight rows sorted by file — the five 768 additions with the two
  installed finals corrected (`class_rows.rs` `6da1dd03…`, `class_lease_fixtures.rs`
  `475d0edb…`) plus `class-provenance.tsv` `8c8b2232…`, `reconcile.rs`, and
  `emission_chain.tsv`; every digest recomputed from the install tree;
- re-reads every written file and re-hashes it; prints one line per artifact with its sha and
  row count, and the count of differing cells against the 768 artifact: exactly the
  `guest.rs` row's four cells.

The ownership chain table, the six fact tables and the ledger are **unchanged** in the install
tree (the F4 edit is a message; no discovered fact moves). Assert: 152 chain rows; the
ledger's 92 entries and its 79/8/3/2 dispositions; the `EXACT INVENTORY` counts
61 / 92 / 10134 after the install (stage A runs the two ownership tests once in the install
tree to prove it).

## The verifier extension (fail-closed)

In the install tree's `source_auth/mod.rs`:

- `additions.tsv` is read by the verifier (like `schema::rows()`, via `include_str!`); a
  malformed table (wrong header, duplicate file, non-hex digest, bytes ≠ file length) is
  `AuthError::OverlaySet`.
- After the overlay pin loop and before the non-overlay loop, the key-set rule becomes:
  `copied.files.keys() == baseline.files.keys() ⊎ additions.keys()` **exactly**; every
  addition is present in `copied`, absent from `baseline`, and its digest equals the pinned
  digest and its length the pinned length; a baseline path missing from `copied` is
  `UnlistedFile`; an extra path that is not an addition is `UnlistedFile`; an addition
  present in the baseline is `UnlistedFile`. The inventory rule, the seals and the manifest
  lane keep their current semantics for baseline files.
- No new arm, configuration, field or receipt line; the eight rows apply to both arms (the
  additions are not arm-differing and no addition may appear in `receipt.overlays`).

Falsifiers (hermetic where possible; every red arm watched): an extra unpinned file →
`UnlistedFile`; a missing addition → `UnlistedFile`; an addition at a wrong digest →
`UnlistedFile`; an addition also present in the baseline → `UnlistedFile`; a malformed
`additions.tsv` → `OverlaySet`; a receipt that lists an addition as an overlay → `OverlaySet`;
and the green arm: the eight admitted at their pins. Also keep brief 776's falsifier suite
green.

## The constructed baseline and the eight

Build `.temp/brief-777/baseline/` by copying the install tree (same exclusions), deleting the
eight additions, and restoring each of the 147 overlay files with the **live** file's bytes
only when its digest equals the table's preimage (a mismatch stops). Then assert, explicitly:

- for each of the six untouched drift paths: the baseline byte equals the install byte equals
  the live entry byte (digest recorded), and differs from the 749 source copy's byte where the
  source differs;
- for `mutants.tsv` and `overlays.tsv`: both sides carry the re-anchored table (equal bytes,
  the new digests), and neither equals the 749 source copy's byte (`e8b3b6e4…` / `5c3b02a3…`);
- every other baseline path is byte-equal to the install tree's, and the key sets satisfy
  `baseline ⊎ additions == install`;
- the baseline's tree hash and the install tree's tree hash (the `probe-build.tree_hash`
  algorithm), plus a control that the baseline's `overlays.tsv` is **not** `5c3b02a3…` and
  not `73297529…` but the re-anchored table.

## The six-case source gate

- **Images.** `real` = the baseline; `armed` = the install tree; `disabled` = the install tree
  with the three arm-differing files replaced by the disabled cells (decoded, re-hashed, and
  the reader re-applied). Each image is its own directory tree; the images are complete copies
  (no symlinks) and carry the same eight additions.
- **Builds.** Six binaries, one per (configuration, image): the 743 adapter pattern over
  `scripts/probe-build.py` (`cargo test --offline --locked -j 8 -p rum-stdlib
  --message-format=json --test suspension_oracle --no-run --release`, plus
  `--no-default-features` for `no-default`), with the stable `.temp/probe/active` path; serially,
  never concurrently; each build's receipt (`cache_hit`, `cargo_exit`, `sha256`, `tree`,
  `source`) checked, and the built binary's digest recorded.
- **Receipts.** One per (configuration, arm) written by the gate script from the tree that the
  binary compiled in (`probe-build`'s `ACTIVE`, whose Snapshot hash equals the image's):
  `field` lines sorted (`arm`, `baseline_tree`, `binary`, `codegen_units=1`,
  `configuration`, `profile=release`, `tree`) and 147 `overlay` lines (`file`, preimage pin,
  digest of the image's file), sorted; the `real` receipt carries no overlay lines and
  `arm=real`.
- **Runs** (each with the case's env: `RUM_CM740_BASELINE`; for disabled/armed also
  `RUM_CM740_RECEIPT`; the observer row for the armed case). Resolve each test's exact name
  with `<binary> --list` and run it `--exact --nocapture` (the 743 pattern; never a substring
  filter):
  - per configuration: `cm_mutation_source_authentication_controls` on all three images (the
    validator controls and every negative arm);
  - per configuration: `cm_mutation_source_copied_admission` on the disabled and armed images
    (`CM4 COPIED SOURCE ADMISSION PASS arm=… programs=9060 P0543=55 strict_loader=SourceIdentity`);
  - per configuration: `cm_mutation_observer_copied_capability` on the armed image
    (`CM4 OBSERVER COPIED CAPABILITY PASS …`) and its disabled-tree negative
    (`WrongArm`);
  - negative pairings: the disabled receipt against the armed tree and the armed receipt
    against the disabled tree (the first `WrongArm`, the second `Postimage`); a receipt with
    one wrong overlay digest → `Postimage`; a receipt missing an addition row is not
    constructible (the additions are not overlay rows) — instead a tree with an addition
    removed → `UnlistedFile`.
- **Six evidence cases** (record each with its verdict lines): (1) the disabled arm accepted,
  (2) the armed arm accepted, (3) the extended overlay set — every one of the 147 files at its
  postimage over its preimage, (4) the emission files outside that set — the eight additions
  present at their pins and absent from the baseline, (5) distinct postimages where the arms
  differ — the three files' cells differ and the arm bijection holds (the swap controls),
  (6) the addition rows — the fail-closed red and green arms above.

## The ordered fixes and the deferred gates

- **F4**: in the install tree's `host_payload_ledger.rs`, the printed composition counts
  `Local` explicitly (the message must print 79 PublicInput / 8 Unbounded / 3 Declared /
  2 Local; the checks keep their exact semantics). Re-run
  `production_host_payload_ledger_is_complete_and_attributed`: green, with the corrected line
  quoted.
- **F3**: correct `.temp/brief-774/receipt_diff.py`'s digest rule to a substring match; add a
  `--selftest` (a glued `… FIXTURE DIGEST_KAT PASS` line passes; a corrupted digest fails);
  re-run against a fresh flag-on class-lease module run on the armed image — `cargo test
  --offline --locked -j 8 -p rum-stdlib --test suspension_oracle … cm_… -- --nocapture
  --test-threads=8` under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` — expecting
  `test result: ok. 197 passed; 0 failed; 0 ignored; 0 measured; 592 filtered out`, then
  `problems=0 digest=PASS`.
- **The deferred 774 acceptance**: one flag-off workspace run on the armed image,
  `cargo test --offline --locked -j 8 --workspace --no-fail-fast -- --nocapture --skip
  class_lease_positive_c5_default_profiled_finish --skip class_lease_positive_p0492`, with
  `RUM_CM740_BASELINE` set to the constructed baseline; `cargo fmt --check` on the armed image.
  Red classification: the environment's nine loopback refusals and the named
  inventory-regeneration class — the retention census (`unreconciled source file:
  …/cm740_probe/class_rows.rs`), `read_inventory`'s new `_cm740_driven` readers, the
  definition-store call-site inventory, the recursion registries, the frame-root-trace rule,
  the resume-evaluation boundary, the typed-slot census, lp03/04/05/648 and the Q25 selector
  rows; plus the two Q25 carrier files' `cargo fmt` diff, which the render brief owns with F5.
  Any other red, and any source_auth red, stops. The full workspace green is the render
  brief's bar, not this brief's.

## Stages and component bounds

- **A** (900 s): entry re-authentication (the scene pins, the six fact tables and the ledger,
  the chain, the class finals, the live walk), the live-versus-scene delta audit, the install
  tree construction with its delta assertions, and the two ownership tests in the install tree
  (`cargo test --offline --locked -j 8 -p rum-stdlib --lib -- production_inventory_is_exact
  production_host_payload_ledger_is_complete_and_attributed`).
- **B** (1,200 s): the re-anchoring writer, its KATs, the re-reads, and the diff against the
  768 artifact.
- **C** (900 s): the verifier extension, `additions.tsv` consumption, the falsifiers, and the
  one `-p rum-stdlib` compile/test gate for them.
- **D** (600 s): the constructed baseline, the eight assertions, the 749-source control, and
  the tree hashes.
- **E** (2,400 s): the six builds, the four receipts, the runs, the negative pairings and the
  six evidence cases.
- **F** (workspace bucket, 2,700 s): F4, F3 with its selftest, the flag-on run, the receipt
  diff, the workspace run and `cargo fmt --check`.
- A projected component overrun stops for a director ruling; never a scope cut. Unspent time
  returns.

## Environment

- Every cargo command runs directly inside the dispatch scope at `-j8` (no nested
  `memory-scope.sh`); tests at `RUST_TEST_THREADS=8`; `--no-fail-fast` stays a cargo option
  (before `--`); `--offline --locked`; timing-sensitive runs are serial and alone.
- `TMPDIR` under `.temp/brief-777/tmp`; never `/tmp`. All long commands bounded (the
  `scripts/run-bounded.py` pattern).
- No network; no new dependency; no new `unsafe` in `rum-core`/`rum-runtime`; no default-path
  cost; no new program, lane, selection, policy or epoch label; no tag.
- No git invocation of any kind; never read `~/.config/ccds/api-key`.
- The six cases run serially (the probe `ACTIVE` path is single).

## Verification and acceptance

The brief passes only with all of: the install-tree construction and its delta assertions;
the re-anchoring with exactly the declared cells changed; the fail-closed addition set with
every falsifier arm watched; the constructed baseline with the eight assertions and the
749-source control; the six-case gate with its quoted PASS and negative lines; F4's corrected
message with the ledger test green; F3's selftest and `problems=0`; the workspace run with the
exact classified red list; and the recorded `cargo fmt` state. Quote every verdict line
verbatim in the report; disclose every deviation and every file the install touched.

## Checkpointing and stops

Checkpoint after every stage (`.temp/brief-777/PROGRESS.md`, logs under
`.temp/brief-777/logs/`). Stop, with the exact evidence, on: a scene pin mismatch; a delta
audit member outside the declared sets; a re-anchoring cell beyond the declared deltas; a
KAT failure; `Postimage`, `Preimage`, `UnlistedFile`, `BaselineSeal`, `Inventory` or
`OverlaySet` after the correct construction; a missing or extra addition; a six-case run that
fails with any error other than the declared negative pairings; a workspace red outside the
named class; a source_auth red in the workspace; or a projected overrun. A stop is a correct
terminal state; never weaken a rule, skip a file or copy one arm onto the other to pass.

## Hard rules

Never weaken `Preimage`, `Postimage`, `BaselineSeal`, `Inventory`, `OverlaySet` or
`UnlistedFile`; never hand-write overlay hex (the writer derives every cell from a tree); never
use the 749 source copy's tables as a baseline source; never take a preimage from anything but
the live tree after the digest check; never let the baseline inherit the 749 source's drift
bytes for the eight; never touch the live tree or the sealed scene; no receipt from synthetic
data; no importable receipt; Q25 stays open; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

Focused **6,000 s** + workspace **2,700 s** + epoch **0**, carved from the stage reserve
(25,869.36678847017 → **17,169.36678847017 s**). Component bounds 900/1,200/900/600/2,400 s;
unspent returns; a projected overrun stops for a director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-777/report.json`, appended as stage `A / c136`; `summary`,
`changes`, `deviations`, `questions`, the six evidence cases, every verdict line quoted, and
the final message `scripts/report.py summary`. The director gates from raw evidence.

## Gate c135 note (for the reader)

Brief 776's stage D established the scene's baseline gap: the source_auth family cannot
authenticate in a 749-source-based scene, and the derived inventories describe the live
source. This brief replaces the scene with the install tree — the live tree plus the accepted
emission — so the gap's source_auth half closes here and the inventory half closes in the
render brief.
