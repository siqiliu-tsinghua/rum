# Brief 778 — G15-CM4 Stage D repair: the source A/B continuation — the capability-mode anchor, the re-anchored tables and the complete six-case gate

**Implementation brief, staged (A–F), one gate.** It continues ruling 786 (gate c136) and brief
777 (`cc7ea89f`). Brief 777 built the install tree and proved its stages A–D; its stage E
stopped correctly: the copied-source admission feeds `AuthenticatedSource::source` (the
baseline originals) to a class loader whose seal source brief 770 switched to
`class-provenance.tsv` for **both** loader modes. This brief carries the accepted install tree,
repairs the loader so the anchor follows the byte source, re-anchors the tables through the
single writer with the new declared `guest.rs` delta, rebuilds the baseline and the real
image, and runs the complete six-case source gate and the deferred 774 acceptance. A
front-half stop does not skip later evidence by shrinking it.

## Goal

When this brief is done, in `.temp/brief-778/` only:

- The carried install tree (copied from `.temp/brief-777/install`, tree hash `e2c8bfbe…`,
  5,850 files) with exactly two source edits beyond the carry:
  `fixtures/obligations/guest.rs` (the capability-mode anchor) and
  `mutations/source_auth/tests.rs` (the arm-aware admission assertion and the anchor
  falsifier), and the four re-anchored artifacts (`overlays.tsv`, `mutants.tsv`,
  `inventory/emission_chain.tsv`, `additions.tsv`). The live tree and
  `.temp/brief-777/` stay byte-untouched.
- The class-form loader reads the **class** anchor on the workspace path and the **baseline**
  anchor (`provenance()`) on the capability path; `AuthenticatedSource::source` and its route
  test are unchanged; no check is weakened.
- The tables are re-anchored by the single writer with **exactly one** content delta from the
  carried tables: `guest.rs`'s postimage cells become the repaired class form's digest (both
  arms); the chain's `guest.rs` row follows; `mutants.tsv`'s `source_schema` follows;
  `additions.tsv`'s chain pin follows. `class-provenance.tsv` stays `8c8b2232…`; the eight
  additions stay the same eight.
- The constructed baseline and the real image (baseline ⊎ the eight additions) are rebuilt
  from the new installed bytes with the eight drift paths asserted.
- The six-case source gate is complete: six release builds, six receipts, the validator
  controls on all six, the copied admission on the disabled and armed images in both
  configurations with the anchor repair (`PASS` lines), the observer capability, the negative
  pairings, and the six evidence cases recorded with their verdict lines.
- The deferred 774 acceptance is run on the armed image: F3's re-run (`problems=0
  digest=PASS`), F4's corrected message quoted, the flag-off workspace run with the two named
  skips and `RUM_CM740_BASELINE`, and `cargo fmt --check`; every red classified, and nothing
  outside the named classes.

Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Context pointers

- Ruling `.temp/director/ruling-786.md`; brief 777 (`docs/dev/codex/brief-777-g15cm4-source-ab.md`,
  committed `cc7ea89f`) and its report `.temp/brief-777/report.json` (sections `admission`),
  `PROGRESS.md` and logs (`stage-a.log`, `stage-b.log`, `stage-b-check.log`, `stage-c.log`,
  `stage-d.log`, `stage-e.log`, `stage-e-admission-diagnosis.log`).
- The carry: `.temp/brief-777/install/` — tree `e2c8bfbe5800b1d2…`; `overlays.tsv`
  `67df386d…` (147 rows); `mutants.tsv` `0f0c7805…` (152 lines); `additions.tsv` `21e9652b…`
  (8 rows); the chain `inventory/emission_chain.tsv` `8151a157…` (152 rows); the class table
  `8c8b2232…`; `provenance.tsv` `965eca10…`; the ledger 93 lines / 92 entries; the baseline
  `.temp/brief-777/baseline/` `93fd8ca5…` (5,842 files); the receipts
  `.temp/brief-777/receipts/default-real.receipt` (7 lines, `arm=real`) and
  `default-disabled.receipt` (154 lines).
- The loader: `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/guest.rs` —
  `programs()` at 84, `load_programs()` at 1319, `programs_with_mutation` at 1326,
  `load_programs_with` at 1332, the canonical-source loop at 1351, the guest-route checks at
  1430, the guest-dependency branch at 1547. The live preimage (`46e8c0c6…`? no — `a07af227…`,
  2,696 lines) is **not edited**.
- The source authority: `mutations/source_auth/mod.rs` — `verify` 107–216, `authenticate`
  236, `AuthenticatedSource::source` 262 (returns `originals`, the baseline bytes, after the
  workspace postimage check); `tests.rs` — `cm_mutation_source_copied_admission` (the failing
  test, now at ~397), `cm_mutation_source_route_postimage` (the originals contract),
  `cm_mutation_observer_copied_capability` (uses `programs_with_mutation`), `control` (the
  in-memory falsifiers); `view.rs` (`AuthenticatedSource` fields are `pub(super)`; the tests
  module may construct one); `schema.rs` (pins, `file_postimage` — the table is a
  **declaration**, the verifier checks the declared bytes against the preimage and the
  workspace); `coverage/bindings.rs` (`provenance()` and `class_provenance()`).
- The diagnosis: 62 sealed paths, 8,997 install-only rows (canonical-source 2, guest-route
  8,940, guest-dependency 55), zero live-only, zero neither; the two canonical-source
  postimage cells (`crates/rum-host/src/session.rs`, `crates/rum-jupyter/src/adapter/worker.rs`)
  and the three arm files' armed cells differ from the preimages; the same 9,136 sealed rows
  are already green against the constructed baseline in `verify`'s `BaselineSeal` loop (the
  stage-E controls), so the capability mode is provably green once it reads `provenance()`.
- The single writer: `.temp/brief-777/reanchor.py` (read-only) — `GUEST_OLD`/`GUEST_NEW`, the
  one-constant `ARM` rules, `ADDITIONS`, `EXPECT`, the `--check` KATs. Copy and adapt it.
- The 776 workspace red reference: `.temp/brief-776/logs/stage-d-workspace.log` (4,488 passed,
  85 failed, 71 ignored; the failing test names are the reference set for stage F).

## Scope

- You MAY write only under `.temp/brief-778/`: `install/`, `images/`, `baseline/`, the
  scripts, the logs, the evidence and the report. The live tree, `.temp/brief-777/` (the
  carry source), `.temp/brief-774/scene` (the accepted 776 final state, walk 5,818 /
  `48dd8fe2…`), `.temp/brief-749/attempt-7/copies`, `.temp/brief-766/`, `.temp/brief-768/`,
  `.temp/brief-769/`, `.temp/brief-774/` (the F3 script is run, not edited) and
  `.temp/brief-776/` are read-only.
- You MAY change, **in the 778 install tree only**:
  - `crates/rum-stdlib/tests/suspension_oracle/fixtures/obligations/guest.rs` — the loader
    anchor repair below; nothing else;
  - `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/tests.rs` — the
    arm-aware assertion in `cm_mutation_source_copied_admission`, its verdict line, and one
    anchor falsifier; the existing controls are unchanged;
  - `mutations/source_auth/overlays.tsv`, `inventory/obligations/mutants.tsv`,
    `src/tests/ownership_graph/inventory/emission_chain.tsv`, `mutations/source_auth/additions.tsv`
    — through the single writer only, never by hand.
- No other file may differ from the carry beyond the four artifacts and the two source edits;
  a disclosed extra stops for a ruling.

## The loader repair (exact semantics)

The seal anchor follows the **byte source**: the workspace path reads the class-form tree, so
it checks the class table; the capability path reads the baseline originals, so it checks the
baseline table. Concretely (the exact spelling is yours; these semantics are not):

- `load_programs_with` takes the seal table as a parameter: `seals: &[Vec<&'static str>]`
  (or an equivalent shared slice), and no longer reads any accessor itself.
- `load_programs()` passes `class_provenance()` (the 770 anchor for the class-form workspace).
- `programs_with_mutation` passes `provenance()` (the baseline anchor; the capability hands
  `originals`).
- `provenance()` keeps its consumers (`source_auth`, `known_losses`, `proofs`, `guest.rs`'s
  `c52-native-values` seal); `class_provenance()` keeps the workspace mode; `source()` and
  `cm_mutation_source_route_postimage` are untouched.
- No check, table, count or predicate changes otherwise.

**The declared delta KAT.** The new class form must equal the carried class form
(`1a2c5ff5…`) with exactly these textual edits applied, each anchor occurring exactly once at
application time:

1. workspace call site — before `    load_programs_with(&|path| {` insert
   `    let seals = crate::oracle::obligations::bindings::class_provenance()?;` and pass
   `&seals` as the first argument;
2. capability call site — before
   `    load_programs_with(&|path| source.source(path)).map(Arc::new)` insert
   `    let seals = crate::oracle::obligations::bindings::provenance()?;` and pass `&seals`;
3. the signature gains `    seals: &[Vec<&'static str>],` as its first parameter;
4. the in-function read `    let seals = crate::oracle::obligations::bindings::class_provenance()?;`
   immediately before `    let seal: Vec<_> = seals` is removed.

The writer applies the edit list mechanically; the result is the install's `guest.rs`, whose
digest equals the table's new postimage cells (both arms; `guest.rs` is arm-independent). Any
anchor count ≠ 1, any byte difference, or any other changed line stops.

**The test repair.** In `cm_mutation_source_copied_admission`:

- the copied admission (the baseline anchor) stays `unwrap()` and asserts 9,060 programs and
  P0543 = 55 — on the disabled **and** armed images;
- the plain loader's assertion becomes arm-aware: on `armed`, `programs()` is `Ok` with 9,060
  programs (the class-form tree against the class table — the 772 by-use property); on
  `disabled`, `programs()` is `Err(SourceIdentity)` (the class anchor refuses the shipped
  configuration);
- the verdict line records the per-arm truth — expect exactly
  `CM4 COPIED SOURCE ADMISSION PASS arm=disabled programs=9060 P0543=55 strict_loader=SourceIdentity`
  and
  `CM4 COPIED SOURCE ADMISSION PASS arm=armed programs=9060 P0543=55 strict_loader=class_form_ok`;
- one anchor falsifier: an in-memory `AuthenticatedSource` whose `originals` carry the
  class-form bytes for a sealed path, with matching `postimages` — `programs_with_mutation`
  must be `Err(SourceIdentity)` (the baseline anchor refuses class bytes); print it.

## The re-anchoring (single writer)

Copy `.temp/brief-777/reanchor.py` to `.temp/brief-778/reanchor.py`; the writer's inputs are
the 778 install tree, the live preimages, the carried tables and the 768 artifact. It must:

- derive every cell from trees, never hand-write hex; the 147 payloads are the install files;
  the three arm files' disabled payloads are the armed payloads with the one-constant
  substitution (KAT, byte-exact); `guest.rs`'s payload is the repaired class form (both arms);
- assert the 147 preimage pins equal the live digests; the two canonical-source and three arm
  JSON cells; every payload decodes to its digest;
- assert exactly one content delta from the carried table (`guest.rs`'s columns 2–5) and
  exactly one from the 768 artifact relative to the carried table's single delta;
- re-derive `mutants.tsv` (the live table with column 7 = the new overlays digest on all 151
  records), the chain (the carried 152 rows with the `guest.rs` row replaced by the new
  armed-cell digest; every other row equal to the carry; each row equals its path's armed cell
  or its addition pin), and `additions.tsv` (the same eight rows; the chain row's digest and
  length updated);
- assert `class-provenance.tsv` is byte-unchanged (`8c8b2232…`) and the eight additions are
  the same eight.

## The baseline and the images

- **Baseline** (unchanged rule): copy the install tree, delete the eight additions, restore
  each of the 147 overlay files from the live file only after its digest equals the table's
  preimage; assert the six untouched drift paths at live bytes on both sides, the two tables
  re-anchored on both sides, `baseline ⊎ additions == install` (keys), and the tree hashes.
  The baseline's `guest.rs` is the live preimage (untouched).
- **Images**: `real` = the baseline **plus the eight additions** (the corrected bar: the bare
  baseline cannot compile the test target because the adopted `bindings.rs` `include_str!`s
  the class table; the real image's non-addition bytes must equal the baseline's, each
  addition at its pin, and its tree hash recorded); `armed` = the install tree; `disabled` =
  the install tree with the three arm-differing files replaced by the decoded disabled cells
  (re-hashed, the reader re-applied). Complete copies, no symlinks.

## The six-case source gate

Carry brief 777's stage E design on the new tree, with the corrected bars:

- **Builds**: six release builds, one per (configuration, image) — `default` and
  `no-default` × `real`, `disabled`, `armed` — through the 743 adapter over
  `scripts/probe-build.py` (release, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, stable
  `.temp/probe/active`), serially; each build's receipt (`cache_hit`, `cargo_exit`, `sha256`,
  `tree`, `source`) checked and the binary digest recorded.
- **Receipts**: one per (configuration, arm), field lines sorted (`arm`, `baseline_tree`,
  `binary`, `codegen_units=1`, `configuration`, `profile=release`, `tree`). The `real`
  receipt carries **no** overlay lines and `arm=real` (a build receipt; `verify` requires the
  147 overlay keys, so the real receipt never authenticates — do not attempt it). The
  disabled/armed receipts carry the 147 overlay rows (`file`, preimage pin, the image file's
  digest) and must equal the table's pins and cells.
- **Runs** (resolve each test's exact name with `<binary> --list`; run `--exact --nocapture`):
  - `cm_mutation_source_authentication_controls` on all six images (both arms' negative
    lines, `CM4 SOURCE AUTH ADDITIONS PASS arm=… rows=8`, `CM4 SOURCE AUTH VALIDATOR PASS
    arm=… negative_controls=10 build_receipts=0`);
  - `cm_mutation_source_copied_admission` on the disabled and armed images in both
    configurations — the two exact PASS lines above, plus the anchor falsifier's line;
  - `cm_mutation_observer_copied_capability` on the armed images
    (`CM4 OBSERVER COPIED CAPABILITY PASS …`);
  - the negative pairings: the disabled receipt against the armed tree → `WrongArm`; the
    armed receipt against the disabled tree → `Postimage`; a receipt with one wrong overlay
    digest → `Postimage`; a tree with one addition removed → `UnlistedFile` — via
    `cm_mutation_source_gate_result` with `RUM_CM740_GATE`/`RUM_CM740_EXPECT` (or the
    equivalent in-process control), each line quoted;
  - the six evidence cases (brief 777's list) recorded **completely** — no FLAGGED evidence;
    a case that cannot run stops.
- Expected totals: `programs=9060`, `P0543=55` on every admission; the controls' 20 negative
  arms per run; the disabled image's overlay set 147/147.

## The deferred 774 acceptance (stage F, on the armed image)

- **F3**: one fresh flag-on class-lease module run on the armed image —
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test --offline --locked -j 8 -p rum-stdlib --test
  suspension_oracle class_lease --no-fail-fast -- --nocapture --test-threads=8` — expecting
  `test result: ok. 197 passed; 0 failed; 0 ignored; 0 measured; 592 filtered out`, then
  `python3 -I .temp/brief-774/receipt_diff.py .temp/brief-772/logs/class-lease-fixtures.log
  <the fresh log>` expecting `RECEIPT DIFF PASS rows=97 lost=0 new_miss=0 withheld_bad=0
  digest=PASS`; also re-run `--selftest`.
- **F4**: re-quote `HOST PAYLOAD LEDGER: 92 rows; 13 storage edges (3 Declared, 8 Unbounded,
  2 Local); 79 PublicInput` from the carry's green ledger test (no re-run required if the
  line is quoted from `.temp/brief-777/logs/stage-a-ownership.log` and the carry's file is
  byte-unchanged — it is).
- **The workspace and fmt**: one flag-off workspace run on the armed image —
  `cargo test --offline --locked -j 8 --workspace --no-fail-fast -- --nocapture --skip
  class_lease_positive_c5_default_profiled_finish --skip class_lease_positive_p0492` with
  `RUM_CM740_BASELINE` set to the constructed baseline — and `cargo fmt --check`.
- **Red classification**: every red and ignored test is listed and classified. Acceptable:
  (i) the environment's nine native ZeroMQ loopback refusals; (ii) tests red in the 776
  stage-D workspace reference set (`.temp/brief-776/logs/stage-d-workspace.log`) — the
  class-form-versus-baseline inventory/settlement suites (the retention census and oracles,
  `read_inventory`, the definition-store call-site inventory, the recursion and depth
  registries, the frame-root-trace repository rule, the resume-evaluation boundary, the
  typed-slot census, lp03/04/05/648, the Q25 proof/selector rows, the obligation
  receipt/binding suites, the scratch-copy falsifiers); (iii) `cargo fmt --check` red only on
  `cm740_probe/q25_ownership.rs` and `q25_p2379_proof.rs` (the render brief owns their
  formatting with F5). Any other red — and any red not in the reference set — stops. The full
  green is the render brief's bar.

## Stages and component bounds

- **A** (500 s): entry re-authentication of the carry (the tree hash `e2c8bfbe…` / 5,850, the
  four artifacts' digests, the class table, the ledger, the chain, the receipts, the baseline
  `93fd8ca5…`, the scene 5,818/`48dd8fe2…`), the copy to `.temp/brief-778/install/` and its
  hash equality.
- **B** (800 s): the loader repair, the declared-delta KAT, the test repair, the anchor
  falsifier, and one `-p rum-stdlib` compile gate for them.
- **C** (700 s): the single writer, the re-anchored four artifacts, the one-delta assertions
  against the carry and the 768 artifact, and the verifier's green re-run on the new tables
  (the controls and the addition falsifiers, one compile).
- **D** (400 s): the baseline rebuilt, the eight assertions, the 749-source control, the real
  image (baseline ⊎ additions) and the tree hashes.
- **E** (1,900 s): the six builds, the six receipts, the runs, the negative pairings and the
  six evidence cases.
- **F** (700 s focused + the workspace bucket): F3's flag-on run and receipt diff, F4's
  quoted line, the workspace run and `cargo fmt --check`.
- A projected component overrun stops for a director ruling; never a scope cut. Unspent time
  returns.

## Environment

- Every cargo command runs directly inside the dispatch scope at `-j8` (no nested
  `memory-scope.sh`); tests at `RUST_TEST_THREADS=8`; `--no-fail-fast` stays a cargo option
  (before `--`); `--offline --locked`; timing-sensitive runs serial and alone.
- `TMPDIR` under `.temp/brief-778/tmp`; never `/tmp`. All long commands bounded (the
  `scripts/run-bounded.py` pattern).
- No network; no new dependency; no new `unsafe` in `rum-core`/`rum-runtime`; no default-path
  cost; no new program, lane, selection, policy or epoch label; no tag.
- No git invocation of any kind; never read `~/.config/ccds/api-key`.
- The six cases run serially (the probe `ACTIVE` path is single).

## Verification and acceptance

The brief passes only with all of: the carry's hash equality; the loader repair with the
declared-delta KAT byte-exact and the two anchors read by the correct modes; the test repair's
two exact PASS lines and the anchor falsifier watched red; the re-anchored tables with exactly
the declared cells changed and the verifier's controls and addition falsifiers green on them;
the baseline with the eight assertions and the 749-source control; the real image as baseline ⊎
additions; the six-case gate complete (builds, receipts, runs, pairings, six evidence cases);
F3's `problems=0 digest=PASS` and selftest; the workspace's fully classified red list; and the
fmt state. Quote every verdict line verbatim in the report; disclose every deviation.

## Checkpointing and stops

Checkpoint after every stage (`.temp/brief-778/PROGRESS.md`, logs under
`.temp/brief-778/logs/`). Stop, with the exact evidence, on: a carry pin mismatch; an anchor
count ≠ 1 or any byte beyond the declared delta; a missing or extra addition; a re-anchoring
cell beyond the declared deltas; a KAT failure; `Postimage`, `Preimage`, `UnlistedFile`,
`BaselineSeal`, `Inventory` or `OverlaySet` after the correct construction; a six-case run
that fails with any error other than the declared negative pairings; a workspace red outside
the reference set; a fmt red outside the two Q25 carrier files; or a projected overrun. A stop
is a correct terminal state; never weaken a rule, skip a file, or copy one arm onto the other
to pass.

## Hard rules

Never weaken `Preimage`, `Postimage`, `BaselineSeal`, `Inventory`, `OverlaySet` or
`UnlistedFile`; never reseal or edit `class-provenance.tsv`; never change `source()` or its
route test; never hand-write overlay hex (the writer derives every cell from a tree); never
use the 749 source copy's tables as a baseline source; never take a preimage from anything but
the live tree after the digest check; never touch the live tree or the sealed scene; no
receipt from synthetic data; no importable receipt; Q25 stays open; 744/745/746 stay held;
CM5 stays unauthorized.

## Budget

Focused **5,500 s** + workspace **2,700 s** + epoch **0**, carved from the stage reserve
(21,047.05878847017 → **12,847.05878847017 s**). Component bounds
500/800/700/400/1,900 plus stage F's 700; unspent returns; a projected overrun stops for a
director ruling, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-778/report.json`, appended as stage `A / c137`; `summary`,
`changes`, `deviations`, `questions`, the six evidence cases, every verdict line quoted, and
the final message `scripts/report.py summary`. The director gates from raw evidence.

## Gate c136 note (for the reader)

Brief 777 certified the install tree's construction through stage D and stopped correctly at
the copied admission. Ruling 786's F6 is the whole defect: brief 770's seal-source switch
moved both loader modes to the class table; the capability mode must read the baseline table
because `source()` hands the baseline originals. The class table is not resealed (the
two-anchor rule stands) and `source()` is not changed (its route test pins the originals).
The real-image bar is corrected to baseline ⊎ additions: the bare baseline cannot compile the
test target because the adopted `bindings.rs` `include_str!`s the class table, and the
verifier's own key-set rule is `copied == baseline ⊎ additions`. This brief finishes the
source A/B; the probe and hot-path A/B, the deterministic render (F5, the Q25 formatting, the
inventory regeneration, the full workspace green, the source-gate re-run), the re-freeze, the
fresh epoch, the Stage C/C10 readmission and a further 744 restart follow, each gated.
