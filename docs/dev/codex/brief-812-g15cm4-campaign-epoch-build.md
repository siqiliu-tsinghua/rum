# Brief 812 — G15-CM4 campaign epoch build — implementation

**Director-dispatched, one implementation stage, one gate.** Read `.temp/brief-811/source-proof.md`,
`falsifiers.json`, `r4-dispositions.json`, `indices.json`, `producer-commands.json`,
`workspace-gate.json`, `qualification.json`, `schedule-design.json`, `zero-unknown.json`, and the
three design drafts `draft-epoch.md`, `draft-readmission.md`, `draft-744.md` (the sealed 811
record, report `58b35673…`, seal `da2fdf55…`). This brief is draft 1, the campaign epoch build and
qualification. Rulings 858, 859, 860 and 861 settle the re-grounding, the role-aware source proof,
the zero-additions reader and the F13 native binding. No tag, no network, no new dependency, no
new `unsafe` in `rum-core`/`rum-runtime`, no git. Scratch under `.temp/cm4-campaign-epoch` (`W`);
never `/tmp`. The 811 record, the four 806e freeze role roots and the preserved ancestors are
read-only; the live tree is build input only and is not edited.

## Why

The campaign epoch must be rebuilt on the post-D.2 tree: the 754 epoch's raw evidence was lost
(c154), its qualification is stale, and the sealed source-auth contract could not authenticate the
six-build index against one baseline. The 811 design fixes all three: the role-aware proof takes
the real role tree as the baseline with zero additions, the reader change accepts exactly the
declared-zero 32-byte table (`file\tsha256\tbytes\n# additions=0\n`, `33478d6e…`) with every
malformed form rejected, and the F13 row binds only to the six native producer runs on the final
tree in the three roles (the 806e hashes stay parent pins; the parser-only projection is
inadmissible). This brief builds and qualifies that epoch: one immutable child epoch with a
passing role-aware source proof, the twelve new release labels, functional equivalence, six fresh
workspace cases and both-mode raw-artifact authentication. It publishes zero class admissions and
zero new catches; the readmission and the 744 restart follow under briefs 813 and 814.

## Goal

Publish `W/epoch-identity.json` — one immutable child epoch with: a passing role-aware source
proof (real = baseline; zero additions; 165 overlays with the native real preimages and the
native disabled/armed postimages); the twelve fresh release labels at one stable scratch path,
one codegen unit, offline; functional equivalence on the two frozen programs; six fresh workspace
cases with every old/new target reconciled; and both-mode raw-artifact authentication of the
six-build index and the C10 zero-credit calibration. Zero member cells, zero schedule credit.

## Inputs and pins

- **The 811 design record** at `.temp/brief-811/` (read-only): the 632 `design-pins.json`, the
  five design documents, the three drafts, the 30-payload seal (report `58b35673…`, seal
  `da2fdf55…`), the entry/design/validation logs.
- **The four 806e parent roots** at `.temp/brief-806/finish-e/freeze/roles/{baseline,real,disabled,armed}`
  (5,842 / 5,851 / 5,851 / 5,851 files; trees `cb9205a6…` / `dda9146e…` / `30cd48fc…` /
  `df0db63c…`; manifests `ffbfe330…` / `1e02d222…` / `b782aa97…` / `3dc7f9bc…`), and
  `.temp/brief-806/finish-e/freeze/identity.json`.
- **The surviving witnesses**: 754 report `2377f246…`; 755 authentication entry == closing
  `9911a614…`, schedule `3e4532cf…`, class admission `6d556f4b…`, packet index `3e1644aa…`,
  ordinary negatives `e42ced67…`; 744 report `314ca69d…`; 743 report `6394a163…` and
  `report-c94.json` `de91efc5…`; 807f policy `08c03054…`; 809 report `ba3a733b…`; the recorded
  epoch digest `7af7b461…` stays historical.
- **The build-recipe records**: `.temp/brief-797/tools/seal.py:17` (the overlay wire format) and
  `adopt.py:67–122`; `.temp/brief-755/authenticate.py:59–118`; `.temp/brief-754/report.json`
  (the six workspace cases, the twelve labels, the 20 drives); `.temp/brief-806/evidence/jobs/`
  (the native producer argv patterns) and `.temp/brief-806/completion/{real_review.py,role_reviews.py,render.py,render-pins.json}`;
  `.temp/brief-807f/tools/budget.py:14–16`; `.temp/brief-809/evidence/M-real-functional.json`.
  The 807f calibration/draw history and the 809 no-landing decision are not campaign evidence;
  857 funds publication and excludes fastpath cost.

## Scope, deliverables and execution environment

Authorize only isolated child sources, proof fixtures and scene-local orchestration under `W`.
Handwritten Rust is limited to `mutations/source_auth/{schema.rs,mod.rs,tests.rs}`; generated
tables and their standing outputs are adopted only through producers. No production edit,
sealed-scene edit, live tool edit or hand-edited inventory. No live install. No old ELF, packet
or receipt reuse. All writes under `W`; the report at `.temp/brief-812/report.json`.

The dispatch scope is `MemoryMax=16G`. Run every long command under
`scripts/run-bounded.py --timeout N --`, stdin closed, with the workspace's offline 1.85.0
toolchain, edition 2024, jobs 8 and test threads 8 unless a named one-thread control applies.
Clear inherited `RUM_CM739`/`RUM_CM740`/`RUM_PROFILE`/`CARGO_PROFILE` variables and rustflags;
set `CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=8 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 RUST_TEST_THREADS=8 RUM_TEST_BUILTIN_MEMORY_GROWTH=0 PYTHONDONTWRITEBYTECODE=1`
and scratch `TMPDIR`/`TMP`/`TEMP` under `W`. A denied new unit is not permission for an
unbounded build. A memory-killed job is retried serially exactly as in the 806e lib-gate
precedent (the OOM recorded, the serial rerun disclosed; the 16 GiB bound is never raised).
Pin every source/target path and capture the exact argv, toolchain, flags, features, exit,
monotonic times and build hash per operation.

## Stages

### A. Freeze the input and contract (1,200 s)

Recheck the 811 seal and design pins, the c154 losses, the nine witnesses and the four 806e
manifests; reconcile the checkout census against the 811 exit (`24e487cb…`, 5,902 files): expect
exactly `TODO.md` and the 811 brief modified and this brief added (5,903 files), with the 48
extra docs, three interpreter cache files and four size-changed shared paths journaled from
`qualification.json`; any other difference is a STOP. The 5,851 frozen guest keys are the build
projection. For every key, materialize a new non-linked, symlink-free scratch tree from the
dispatch projection; preserve shared inputs from the admitted snapshot; for the 165 role paths
use the exact matching parent arm bytes to construct `R` and `D`. Require equal child arm keys,
all 199 base source seals matching `R`, the nine historical additions byte-identical across
guests, and no unclassified source difference. Store actual before/after path,size,digest lists
and a Snapshot-compatible tree digest (`source_auth/files.rs:11–65`, files before directories,
each sorted, streaming `F NUL relpath NUL bytes`).

Implement the reader, the private loader seam and the controls exactly as `source-proof.md`
items 1–5: the byte-exact zero branch at `parse_additions` (the literal `ZERO_ADDITIONS`
compared before `str::lines()`; the ordinary path intact), the one call-site seam in
`source_auth/mod.rs` with the production wrapper supplying only `schema::additions` and the
loader having no runtime/environment/receipt/public override, and the tests' independent
nonempty fixture (`fixture/source-proof-addition.bin`, byte `0x78`, length 1, SHA-256
`2d711642…`) plus the zero-layout positive, the omitted/extra/altered controls and the retained
malformed cases. Do a two-sided calibration in scratch — the original parser accepts the fixture
and rejects the literal; the changed parser accepts exactly the literal and rejects every named
malformed control — preserving the acceptance/failure site for every case in `falsifiers.json`.
Complete `cargo fmt --all -- --check` and all handwritten sources before B.

### B. Native inventories, R4 and role closure (7,200 s)

For role in `R,D,A` and feature in `default,no-default` (in that order), use a separate compile
target and writable review destination; `F` is no argument or `--no-default-features`:

```
scripts/run-bounded.py --timeout 650 -- cargo test --offline --locked -p rum-stdlib --lib F tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --ignored --exact --nocapture
scripts/run-bounded.py --timeout 650 -- cargo test --offline --locked -p rum-stdlib --lib F tests::retention_closure::local::contract_tests::local_concrete_contract_controls -- --exact --nocapture
scripts/run-bounded.py --timeout 650 -- cargo test --offline --locked -p rum-stdlib --lib F tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

Archive each `target/brief-544/local-contracts.tsv` before the work path is reused. Adopt only
native F13 bytes after the 68 fingerprints × 18 columns, the 62 green/red controls and per-role
mode agreement; rebuild the library test for the adopted input before testing its marks. Take
native promotion reviews for hash=true/false, merge complementary cells by body/instance, take a
second green check with the resulting certificate table, and carry the real zero-overlay
promotion convention against the new real proof baseline; preserve C01/C02, all identities,
anchors, 2 bodies / 3 marks / 4 probes and the review/check equality, and retain the negative
evidence.

Run the exact native argv patterns recorded under `.temp/brief-806/evidence/jobs/` for
ownership-review (four selectors, seven outputs), d-class-{default,no-default,negatives},
d-lp01{,-producer,-negatives}, d-typed-receipts and the rejection census; `producer-commands.json`
gives every argv and its source receipt. Translate active/output/target roots into `W`, feed the
child review files, and never execute a historical wrapper with its old destination. Recompute
the 152 armed chain hashes from the final named keys, regenerate the two 202-hash class
projections and require 2,422 target keys with no differences. Replay ownership positions,
chain, class, LP01, typed receipts and the rejection census before the first acceptance suite;
require 28 NativeFault, 3 DeferredPublication, 7 ProfileReturn and 113 ordinary members with all
ten members in their recorded classes; require 151 own + 100 pair + 4 structural selections per
mode, 9,060 unique program inputs, 9,511 ordered constituents and 151 own tuple joins against the
755 schedule. Protect existing dispositions, source seals, statement text and owner classes.
Require the rejection frontier 1,195 (10 RET001, 1,147 RET002, 38 RET003; PUBLIC_WRAPPER=16) or
stop with the diff keyed by kind/file/callee/reason. An otherwise-red stale
fingerprint/position/certificate is an ordering artifact: record the original line, adopt only a
valid producer output and repeat; a semantic, key or disposition change stops.

For all 190 R4 rows compare parent/producer/child for all guests, recording identity rows too,
and apply `r4-dispositions.json`; rows 171 (F13, overlay data line 156), 172 (promotion), 182
(class), 183 (mutants), 184 (additions), 185 (the seam), 186 (overlays), 187 (schema) and 188
(tests) carry explicit dispositions. Re-emit all ten prior render outputs from actual inputs,
including any changed position/flat-context dependency. Two independent serializations must
agree byte-for-byte: the 165 ordered overlays, all 330 full decoded images, the 32-byte empty
additions table, the 151 mutant source_schema hashes plus the external digest file
(`source_schema.txt` = 64 hex + LF). Preimage at R4 171 = native `R`; postimages = native `D`/`A`.
Generate per-role file manifests, compare actual differences to the overlay keys, reject any
undeclared path, and keep the nine historical R4 origin classifications separate from the
importer's zero child added keys.

Rebuild the observation metadata from the child roles and the sealed `mutants.tsv`,
`observer-policy.tsv`, `programs.tsv`, `mutant-coverage.tsv` and actual program bytes; re-freeze
the observer manifest with the 806e table `3953aa9f…` as a physical comparator and the 807f v2.5
policy `08c03054…` as the interference parent. Audit each child's carrier/caller and policy
key/tail, including NativeEqual precedence. The sealed 743 generator's hard-coded deleted 742
render and its switched ELF list cannot be acquisition inputs. Emit fresh policy, membership,
selections, matrix and tuples with paths, sizes and SHA-256.

### C. Compile the label journal and prove the builds (7,800 s)

Hold one stable scratch compilation path under `W`; present each complete candidate there by a
read-only bind or verified exact copy. Build in this serial order, exactly once per label:
`default-real default-disabled default-armed default-spot no-default-real no-default-disabled
no-default-armed no-default-spot lane-real lane-disabled lane-armed lane-spot`. At the stable
source path run:

```
scripts/run-bounded.py --timeout 1400 -- cargo test --offline --locked -j 8 -p rum-stdlib --message-format=json --test suspension_oracle --no-run --release F
```

with an outer label/service bound of 1,495 s (as 754). Extract the unique `suspension_oracle`
executable from Cargo's compiler-artifact, hash it and keep it at `W/bin/<label>-oracle`. Force a
genuine crate recompilation at the real input for each spot (an isolated fresh cache or
selective invalidation) while maintaining the identical compile/target path; every label has
`cache_hit=false`, start and finish journal rows and its own compiler/warning receipt; the spot
hash equals real; the non-spot real, disabled and armed ELFs differ within a configuration.
Precompute the allowed warning families and source-site relocations against the 754 report and
the final scoped sources; stop on missing/extra/unknown or a new family.

The lane is a P0543-only compiler presentation of each authenticated guest with exactly the nine
before/after Cargo manifests in the surviving 797 lane-manifest diff and `source_auth/manifests.rs:2`;
verify both digests and the raw diff, then set the four `RUM_CM739_LANE_*` fields from the fresh
lane source, manifest diff and ELF. It has three derived physical roots, real==spot, and no seat
in the six-build class index. Never substitute a lane ELF for a default/no-default importer; the
physical lane build's source proof admits the nine manifest substitutions under
`source_auth/artifacts.rs:46–62` and `source_auth/mod.rs:210–231`; do not promote an additional
lane delta.

One separately labelled `default-disabled` admission-diagnostic build uses a distinct stable
scratch path; it is the thirteenth **receipt**, never a journal label or a substitute for the six
indexed ELFs. Verify a foreign binary/receipt is rejected. The two mode-specific importers are
the freshly compiled `default-real` and `no-default-real` oracle test binaries, with zero
additional importer builds. Distinguish the three primary guest roots, their three lane views,
and four separately journaled auxiliary drive views/builds (default and lane × disabled/armed)
for the 754 carrier probes; drive-only files live outside import trees. Execute the 20 drives and
check five armed identities (P0267/P0312/P0323 joins and P1470/P1471 queues), the inert disabled
cohorts 60/60/60/45/45, 62 scheduled callers, ten supplemental and 26 withholding fixtures,
preserving the three peels of 19,184 bytes/level plus 3,360 intercept (depth 48 = 924,192) and
the private-frame E0603 falsifier. Reconcile 12 canonical + 1 independent + 4 drive, plus the
separately itemized functional CLI and workspace/producer jobs. No old charge is transferred.

Before freezing the subject, deliver the shakedown's control package at `W/control` (scratch
orchestration outside every role tree): `generator-check.py` with the exact interface of draft 3
(the subject supplies all input paths by digest), its positive and one-fault input tables,
compiler and source manifests, and raw receipts; the two-test resolver control built from
`.temp/brief-742/continuation_controls.rs` in a scratch directory resolving its three relative
imports into the new role source, re-issued from `continuation-controls-build.json` with the
current offline lockfile's actual rlibs, requiring the generic receiver AmbiguousDispatch and
concrete-resolution calibrations; the structural/dynamic generator controls; the P2593 Error and
P0813 initial-only/P2593 immediate-only twins and the other named variants; and the pure
compiler fixture. For each ordinary feature mode run the fresh real ELF's
`oracle::mutations::source_auth::tests::cm_mutation_source_private_types` selector with its seven
positive/negative source inputs, actual rustc argv/stderr and signed outcomes: the epoch owns the
fourteen compiler invocations, zero schedule credit, and the shakedown re-verifies the receipt
rather than re-running them. All auxiliary builds carry their own source/ELF receipts and zero
campaign labels.

### D. Functional and six-case acceptance (14,400 s)

Build a fresh paired CLI with `cargo build --offline --locked -j8 --release -p rum-cli F` for
each distinct (real,disabled,armed) × (default,no-default,lane) build input under the same
release environment: nine companion ELF receipts; the three spots bind to the corresponding
real source/configuration; these are separate from the 12-label budget. For each companion run
`scripts/run-bounded.py --timeout 250 -- <child-rum-cli> --file <program>` on
`.temp/brief-795/panel/canonical/{functional-output.rum,core-language-microbench.rum}` with
profile off, the 798 capture-plan's ordinary settings, stdin closed and no background heavy
load. Verify the fixed bytes and the canonical bytes
`dc6af977130ad0e85280d080d199b268e24bacb30c6eda6756f2a2e6a56bd633`; require exit 0, exact
stderr, all 120 ordered (label,iteration) pairs, all nonclock bytes identical, normalized stdout
5,519 bytes and SHA-256 `c193d87f337d36a1f4d194e216adb8b33f7f58e0147582d9e2ace9434d498bed`.
Normalize only the seconds field using the sealed anchored grammar in
`.temp/brief-807f/tools/functional.py:20`; verify both sides of a changed-label/iteration/
nonclock-byte falsifier.

Run the six fresh workspace cases — the three arms × both feature modes, the uninstrumented
real/source reference and the two composed arms, per the 754 protocol: 414 targets, the seven
ordered `--skip` pairs and the nine host-only groups from `workspace-gate.json`. Freeze the full
cargo target list, inherited exact exclusions and admissible role differences before opening
output; enumerate the entire current copied workspace in advance and reconcile every old/new
target by name and binary identity, including duplicated target names; no blanket 48/49
subtraction. Execute each case with:

```
scripts/run-bounded.py --timeout 2700 -- cargo test --offline --locked -j8 --workspace F --no-fail-fast -- --nocapture <the seven ordered --skip NAME pairs>
```

Use the same source mounts, complete fixture parents and P0543 presentation on each side; run
each inherited switched follow-up on the exact same binary and collect one-thread loopback errno
reproductions only where applicable. Register the 806e class-form execution differences by exact
current test name and independent paired receipt; a differing count alone is insufficient;
ruling 762's one-thread same-binary compiler-oracle adjudication retains both the original and
the repeat. Emit six per-target ledgers, raw digests, zero unexplained differences and the
combined six-comparison manifest.

Functional observations make no hardware-timing decision. If a priced hardware comparison is
actually required, pre-freeze the v2.5 parent `08c0305485bd795d6226c8497a1ff0d037cf12918a73da88eb48c2b107f89685`
with fresh ELF/program bindings and obtain its measured-run allocation before that gate: isolated
CPU14, sibling 15, drivers 0–13, passive 100 Hz schedstat/wait4, effective ≥50 Hz, maximum gap
0.100 s, tree agreement ≤2/HZ per distinct task; each sibling gated on the realized 807f reducer
(E,F ≤0.005T+2/HZ and the 1-s clipped F peak ≤0.050+2/HZ); a quiet second screened at ≤5%+2
ticks; the budget verdict persisted before reading output; the nine-cell calibration (quiet
ADMIT; each one-shot injection REJECT with sealed burn containment); the exact capped retries
and unavailable rules. For retired instructions: three fixed orders, event
`cpu_core/instructions/u`, running >99%, new median inside the fresh control interval. No timing
or hardware credit from old draws; a newly required hot-path edit is a scope STOP.

### E. Build admission and calibration (3,000 s)

Before any campaign member runs, materialize the six authenticated builds plus the spots in the
finite read-only view of `source-proof.md`. Collect the build-only both-mode importer controls
and every named mutation in `falsifiers.json`; bind configuration to the trusted compiler receipt
as well as the index field and reject relabels even when the two TSV fields agree; record
exported failures by their named error and exact stage, not exit alone. Create a
content-addressed build/epoch **subject** containing the fresh freeze, the twelve-label journal,
the selections, the ordered matrix, the negatives, the functional evidence and the
six-comparison manifest; give this immutable subject hash to both calibration workers and the
importers; later attestations point to this subject and their raw artifacts.

Acquire C10 default then no-default using the six freshly bound builds and the row/tuple/
environment protocol of draft 2; take a complete new 6-build/6-run C10 index. Run both
same-mode real importers at
`oracle::mutations::receipts::cm_own_mutant_receipts_exact --exact --nocapture --test-threads=8`.
Require only C10 with `real_keys=55`, `hits=1`, `required=151 caught=1 PortBlocked=150
settlements=0`, the complete complement, the final blocked assertion and the partial-index exit
101; the six-build auth-only control has 0 caught, all 151 blocked, no campaign credit. Preserve
the 6-build skeleton for the later 60 class run lines (class 10/141 is earned in draft 2). Seal
the source and build after the positive proof in both modes, every negative rejected, both
forced spots validated, C10 zero-credit and all output reread.

### F. Publication and close (2,400 s)

Publish fresh `W/epoch-identity.json`, the subject, the acceptance attestation and the immutable
payload seal. Stage readbacks include exact per-file pre/post hashes, source and build
configuration, SHA-256 of each emitted binary, importer, program, schedule and raw log,
retention through the final campaign gate, and entry/exit live metadata equality. Account zero
member cells and record the next authorized gate only.

## Stops, hard rules, charge and report

Stop on a pin, sourceset, parser vector, producer, position/identity, visibility, build/warning/
spot, raw receipt, functional or comparison mismatch, a missing bound input, an unavailable
required interference proof, a first failed label, a projected overrun or a stage deadline. No
scope cuts, hidden retries, stale certificates, invented rows, network, dependencies, `unsafe`
or git; no live edit, sealed-scene edit or hand-edited inventory; the 811 record, the 806e
freeze and the preserved ancestors stay read-only; Q25's thirteen STOPs stand; 744/745/746 remain
held; CM5 is unauthorized.

**Charge.** 36,000 s = the 811 pool's 1,235.083 s + an explicit 34,764.917 s top-up (user ruling
「预算不够就追加吧」); the authorized total becomes 63,164.917 s and the cumulative 27,164.917.
Stage boxes A 1,200 / B 7,200 / C 7,800 / D 14,400 / E 3,000 / F 2,400; every command has its own
measured-shape bound and must fit the remaining stage; unspent time returns.

**Report.** `.temp/brief-812/report.json` as `rum-report/1`, stage `campaign-epoch`; full
commands, measured counters and every exact verdict; all replays before the first acceptance;
separate semantic/zero-credit controls; every immutable clock, next cell and independent cost.
Run `python3 -I scripts/report.py check .temp/brief-812/report.json`, detach the summary, seal,
rehash, audit and return only the report summary.

**Checkpointing.** Append `CLOCK stage=<A|B|C|D|E|F> t=<monotonic_ns>` to
`.temp/brief-812/PROGRESS.md` at every stage entry and exit. Each stage's box is measured from its
own CLOCK entry; the charge is continuous from dispatch; a stage reaching its deadline stops
there with the exact missing cell and its elapsed time, never kept open. Save each immutable cell
before replacing a staging tree; at each checkpoint record the exact next cell and any live dirt
attributable to this brief.

## Director gate (c210, 2026-11-27) — the Stage B STOP is accepted: the real role cannot build a no-default lib-test target, so the F13 binding is corrected to native-per-role plus the calibrated projection

The attempt (`brief-812-cdx-20261127-014159-4142336`, 01:41:59–01:56, rc 0, verdict STOP)
delivered Stage A complete and one Stage B cell. I verified: the stop seal's 30,084 payloads
re-hash equal (1,742,945,265 B; seal `cc3818f4…`, report `79ae58b0…`, PROGRESS `6e421647…`,
charge `a6c5d94c…`); `report.py check` OK 341; the live tree is 5,903 files, entry == exit ==
now byte-identical; the 632 design pins, the 811 seal (`58b35673…`/`da2fdf55…`), the 36 c154
losses and the 10 witnesses; the four 806e parents (5,842/5,851/5,851/5,851; manifests
`ffbfe330…`/`1e02d222…`/`b782aa97…`/`3dc7f9bc…`); the dispatch census (5,903, `modified=2
added=1`); the four edits per child role tree (byte-identical across the three roles); the
reader (`schema.rs:51` literal, `:54–56` the exact return, the ordinary path intact;
`mod.rs:107–125` the seam with the production wrapper; `tests.rs:32–40` the in-memory fixture
and its six negatives; `additions.tsv` = `33478d6e…`); the two-sided parser calibration (4
binaries × 49 vectors = 196 `PARSER PASS`, changed `zero-positive → empty` at `:55`, all 47
malformed/truncation vectors `→ OverlaySet`, original `zero-positive → OverlaySet` at `:60`,
the no-default logs byte-identical to default); three `cargo fmt --all -- --check` runs exit
0; the real/default F13 `8d6ba3f0…` (68 rows, 18 invariant columns), which **I re-derived
independently from the tree** (context `f669b7ae5a458672`; 34 dependencies + 1,651 crate
files; byte-equal); charge 895.734 s of 36,000 (A 364.758/1,200, B 177.303/7,200); memory
peak 2.58 GB of 16 GiB, zero OOM; zero git attempts; `git status` clean at `909a217c`; zero
adoptions, labels, C10 runs or credit.

The STOP is correct and the cause is a construction fact: the **lib test** target of the
frozen real role cannot compile in no-default. `crates/rum-stdlib/src/tests/ownership_graph/
reconcile.rs:10` imports `sha2::{Digest, Sha256}` unconditionally; the real role's frozen
`crates/rum-stdlib/Cargo.toml` (`6dcfaa9f…`, the class form's preimage at R4 row 128) keeps
`sha2` optional behind the default-on `hash` feature and has no sha2 dev-dependency, so
`--no-default-features` drops the dependency and rustc emits E0432 before any test runs. The
two copied arms carry the class form's own manifest (`67be9610…`, `sha2` dev-dependency
present, added to live at `e66e8531`). The real role is the uninstrumented pre-class-form
reference; a no-default lib-test configuration does not exist there and cannot be created
without rewriting the class form's preimage, which parent immutability and the 165 overlaid
paths forbid. **Defect #44 (mine):** the 811 design's "two identical F13 byte streams per
role / six outputs" (`source-proof.md:24`) and this brief's per-(role, mode) lib-test loop
plus "native promotion reviews for hash=true/false" mandated those executions without a
buildability check against the frozen roles — the same class as #42/#43.

## Amendment 1 — the F13 matrix, the real zero-overlay promotion, and the resumed Stage B

The stopped attempt's R/default F13 cell is retained and authenticated by hash identity; no
other cell is credited. Resume in the same scene `W = .temp/cm4-campaign-epoch`; the stop
record under `W/stop/` stays immutable.

**Stage A0 (200 s) — authenticate by hash identity, then proceed.** The moved report
`79ae58b0…` and PROGRESS `6e421647…` at `.temp/director/812-stop-1/`; the 30,082 in-place
payloads of `W/stop/payloads.tsv` (the two moved rows re-hash equal at the new root); the
seal `cc3818f4…`; the live census 5,903 equal to `W/stop/live-exit-stat.json`; the four
parents and the three child role trees (`689b9188…`, `f9661a93…`, `450b9e2f…`); the four
edits; the retained R/default F13 `8d6ba3f0…` with its receipt `d20465a0…` and raw
`6a376127…`; `git status` clean. Any difference stops.

**Stage B (6,822.697 s) — the corrected F13, controls and promotion, then Stage B unchanged.**

1. **Buildability matrix (first, before any producer).** For each role and mode create or
   reuse the review destination and run `cargo test --offline --locked -p rum-stdlib --lib
   --no-run F` (plus a `--no-run` default leg); the matrix must hold exactly: R/default,
   D/default, D/no-default, A/default, A/no-default build; R/no-default fails with E0432 at
   `reconcile.rs:10`. Any deviation in either direction stops and returns to the gate.
2. **F13 native cells (five).** R/default is retained; run D/default, D/no-default,
   A/default, A/no-default with the brief's producer command in each destination. Require for
   every output: 68 rows, the 18 identity columns, row order, header and comments preserved;
   the arms' two streams byte-identical within each role; the 62 discharged red/green
   falsifier files after each controls run (`local_concrete_contract_controls`): exactly the
   62 `.red-green.txt` files under `target/brief-540/falsifiers`, each `RED stale contract
   fingerprint` / `GREEN exact current mark`.
3. **Independent projection (falsifier, never a binding).** With the sealed
   `.temp/brief-790/overlay_digest.py` `f13_bytes`, calibrate on each frozen parent role tree
   (`f13_bytes(parent, {})` must reproduce real `217bec91…`, disabled `40e92684…`, armed
   `d3588fb9…`), then compute `f13_bytes(parent, {the four edits' bytes})` and require it to
   equal every native stream of that role, including the retained real one. A disagreement
   stops; the adopted bytes are always the native producer's (ruling 860 stands).
4. **Adopt** each role's native F13 into its role tree and both mode workspaces before the
   controls/acceptance runs, as before.
5. **Promotion (corrected to the historical convention).** Real: no producer; require
   `real/promoted.tsv == baseline/promoted.tsv` (`4337950c…`) and record it as the zero-overlay
   control. Disabled and armed: native `local_production_acceptance` review and check in both
   hash modes, merge complementary cells by body/instance with the parent role table as the
   carry, adopt the merged table, and require review/check equality, C01/C02, 2 bodies / 3
   marks / 4 probes, zero rejections and the retained negative evidence.
6. Stage B's remaining items stand unchanged: the ownership review (its recorded recipe is
   the default-mode lib test on the armed role, which builds), the 152-chain recomputation,
   both class projections, LP01, typed receipts, the rejection census, the 190 R4 rows, the
   ten re-emitted outputs, the two independent serializations and the observer re-freeze.

**Stages C–F unchanged** (7,800 / 14,400 / 3,000 / 2,400 s), including the buildability the
six release labels and the two importer test binaries already require (integration targets,
not the lib test target).

**Charge.** The stopped attempt: 895.734 s of 36,000, unspent 35,104.266 s, cumulative
28,060.651 of 63,164.917. This amendment allocates A0 200 / B 6,822.697 / C 7,800 / D 14,400
/ E 3,000 / F 2,400 = 34,622.697 s inside that unspent; no new top-up. Its own charge is
continuous from its dispatch; the report at `.temp/brief-812/report.json` covers this attempt
and cites the stopped record; every stage box is measured from its own CLOCK entry.

## Director gate (c211, 2026-11-27) — the A0 STOP is accepted: the live-census pin was taken before gate c210's own commit, so Amendment 2 closes A0 by hash identity and resumes Stage B unchanged

The Amendment 1 attempt (unit `brief-812-cdx-20261127-020931-4157751`) ran 2026-11-27 02:09:31
→ 02:15, rc 0, 68,926 tokens, peak 254,267,392 B, oom 0/0, verdict STOP at Stage A0 after
88.612 s of 200. I verified: this attempt's seal 25 payloads / 1,688,101 B `a5bfaa5f…` with all
12 `seal.sha256` rows equal; report `478ef432…` (44,629 B, check OK 57), PROGRESS `89ae27d4…`;
entry == exit == now `bcbd3e14…` at 5,903 paths; against the stopped census `80af125f…`
exactly two cells differ — `TODO.md` (`b854d5e5…` + 4,498 B `6d21c462…` = `1f4c7203…`) and
this brief (`12508298…` + 7,456 B `3657f0b7…` = `02715ca5…`), both gate c210's appends (I
re-derived the old blobs at `909a217c` and the live files at `542aa932`; the diff is a pure
147-line insertion in exactly those two files); the stopped record re-authenticates
(`6c5fac1d…`, `cc3818f4…`, 30,084 files / 1,742,945,265 B, the two moved rows at
`.temp/director/812-stop-1/`, four parents, three child trees
`689b9188…`/`f9661a93…`/`450b9e2f…`, retained F13 `8d6ba3f0…` + receipt `d20465a0…` + raw
`6a376127…`, 811 seal `da2fdf55…` / 632 pins); zero source edits, replays, builds, suites, C10
runs, labels, member cells, credit or git invocations. Charge 375.018 s (A0 88.612/200,
dispatch→A0 25.948, the close interval 260.458). **Defect #45 (mine):** the A0 bar pinned the
census taken before gate c210's own commit, which appended to the two documents every gate must
edit — the pin was unsatisfiable by construction; the STOP is the rule working. Amendment 2
closes A0 by hash identity with the permitted appends named and resumes Stage B unchanged.

## Amendment 2 — the A0 closure by hash identity and the resumed Stage B

**Stage A0 (200 s) — closure, then proceed.** Re-hash the stop-2 record at
`.temp/director/812-stop-2/` (report `478ef432…` 44,629 B, PROGRESS `89ae27d4…` 322 B) against
this attempt's seal `a5bfaa5f…` (6,813 B; the two moved payloads re-hash equal at the new root;
the other 23 rows stay put); the stop-1 record (`stop/payloads.tsv` `6c5fac1d…`,
`stop/seal.json` `cc3818f4…`, the moved report `79ae58b0…` and PROGRESS `6e421647…` at
`.temp/director/812-stop-1/`); the four parents (`cb9205a6…`, `dda9146e…`, `30cd48fc…`,
`df0db63c…` with manifests `ffbfe330…`, `1e02d222…`, `b782aa97…`, `3dc7f9bc…`); the three
child role trees (`689b9188…`, `f9661a93…`, `450b9e2f…`, 5,851 files each, four edits); the
retained R/default F13 `8d6ba3f0…` with receipt `d20465a0…` and raw `6a376127…`; the 811
design seal `da2fdf55…` (30 payloads, 632 pins); and the live census against
`.temp/cm4-campaign-epoch/amendment-1/live-exit-stat.json` (`bcbd3e14…`, 792,151 B, 5,903
paths), where the only permitted differences are `TODO.md` and this brief, each verified as a
strict append and printed as `C211 APPEND PASS path=… old_bytes=… old_sha256=… added_bytes=…
new_bytes=… new_sha256=…`; any other difference stops. `git status` clean at the gate commit.

**Stage B (6,822.697 s) — as Amendment 1, unchanged.**

1. Buildability matrix first, exactly as Amendment 1 item 1.
2. Five native F13 cells, exactly as Amendment 1 item 2.
3. The calibrated independent projection as a falsifier, never a binding, exactly as
   Amendment 1 item 3.
4. Adoption, exactly as Amendment 1 item 4.
5. The corrected promotion, exactly as Amendment 1 item 5.
6. Stage B's remaining items, exactly as Amendment 1 item 6.

**Stages C–F unchanged** (7,800 / 14,400 / 3,000 / 2,400 s).

**Charge.** The Amendment 1 attempt charged 375.018 s of 36,000 (A0 88.612 of 200); unspent
34,729.248 s; cumulative 28,435.669 of 63,164.917. This amendment allocates A0 200 / B 6,822.697
/ C 7,800 / D 14,400 / E 3,000 / F 2,400 = 34,622.697 s inside that unspent; no new top-up. Its
charge is continuous from its dispatch; the report at `.temp/brief-812/report.json` covers this
attempt and cites the stop-2 record; every stage box is measured from its own CLOCK entry.

## Director gate (c212, 2026-11-27) — the Stage B pin STOP is accepted: the P0543 lane pairs are role-relative after the class form, and the copied-arm lane branch cannot reach the row-128 overlay; Amendment 3 designs the two-stage lane model first

The Amendment 2 attempt (unit `brief-812-cdx-20261127-021929-4163331`) ran 2026-11-27 02:19:29
→ 02:32, rc 0, 161,415 tokens, peak 2,882,793,472 B, oom 0/0, verdict STOP at Stage B. I
verified: the seal 83,936 payloads / 7,111,155,719 B `37ac1d36…` with **every payload re-hashed
equal** (0 missing, 0 bad) and all 14 envelope rows; report `70a17f86…` (check OK 169),
PROGRESS `a75466eb…`; A0 PASS (the stop-2 seal `a5bfaa5f…` with the two moved rows re-hash
equal, the stop-1 record `6c5fac1d…`/`cc3818f4…`, four parents, three child trees, retained F13
`8d6ba3f0…`, 811 design `da2fdf55…`/632 pins, the two c211 strict appends verified against HEAD
`85909f0f`); entry == exit `013f4294…` at 5,903 paths; **the six-leg buildability matrix PASS**
(real/default `1955e78c…`, real/no-default the expected E0432 at `reconcile.rs:10`,
disabled/default `659047df…`, disabled/no-default `3bb6c315…`, armed/default `2dd61cdd…`,
armed/no-default `87af548a…`; zero tests run, zero producers); **the pin STOP** — I recomputed
all nine lane pairs independently: eight agree, row 128 differs, `67be9610…`→`7327bd4e…` (797,
class form) vs `6dcfaa9f…`→`7976deb8…` (verifier, real), the difference being exactly one
28-byte `sha2 = { workspace = true }` dev-dependency line, and the substitution maps each guest
to its lane image; the source consequences (`artifacts.rs:55–58` Manifest; the ordinary
Postimage at `mod.rs:192–196` before the lane branch at `:220`/`:227`/`:231–237`). Charge
773.009 s (A0 52.153, B 236.784, close 380.030). **Defect #46 (mine):** the design asserted the
two lane authorities agree and that the verifier admits the nine substitutions, without
checking either against the class-form artifacts; the brief's own "verify both digests and the
raw diff" found it, before any producer. Amendment 3 designs the two-stage role-relative lane
model; implementation follows.

## Amendment 3 — the P0543 lane model design (design-only, no source edits)

**Scope.** Design only: zero source edits, zero builds, zero producers, zero replays, zero
suites, zero adoptions, zero git. Every output under `W/amendment-3/`; the report at
`.temp/brief-812/report.json`; the stopped record at `.temp/director/812-stop-3/`.

**Stage A0 (200 s) — closure by hash identity.** The stop-3 seal `37ac1d36…` (83,936 payloads
/ 7,111,155,719 B, manifest `f887ba91…`; the report `70a17f86…` and PROGRESS `a75466eb…`
re-hash equal at `.temp/director/812-stop-3/`); the stop-2 and stop-1 records; the four
parents; the three child role trees; the retained F13; the 811 design seal; and the live
census against `.temp/cm4-campaign-epoch/amendment-2/live-exit-stat.json` (`013f4294…`, 5,903
paths), where the only permitted differences are `TODO.md` and this brief as strict c213
appends (old prefix + appended range + new hash printed as `C213 APPEND PASS`); any other
difference stops. `git status` clean at the gate commit.

**Stage B0 (1,200 s) — the lane model design.** One design record (`lane-model.md`) with
`path:line` evidence and one falsifier matrix (`lane-falsifiers.json`):

1. **The two records and the role-relative pairs.** State the 797 diff's nine pairs and
   `manifests.rs`'s nine PINS; the eight that agree; row 128's two pairs (`6dcfaa9f…` →
   `7976deb8…` real; `67be9610…` → `7327bd4e…` class form); and the exact relation (the
   797 before is the real guest plus the single `sha2 = { workspace = true }` dev-dependency
   line; the substitution `["test-builtin-memory-growth", "test-continuation-slots"]` →
   `["test-continuation-slots"]` occurs exactly once per manifest and maps each guest to its
   lane image). Derive and state the lane image function.
2. **The verifier contract.** The role-relative table shape and the generated `manifests.rs`
   producer (inputs, output bytes, its pin); the `source_auth/mod.rs` restructure: in the lane
   configuration the nine manifests route to the lane branch (including the row-128 overlay —
   the guest is the baseline for the real arm and the ordinary postimage derivation for the
   copied arms; the copied file must equal the role's lane image; the receipt carries the lane
   post for the nine; the ordinary default/no-default configurations are unchanged); the error
   variants and the fail-closed order; `schema.rs` unchanged.
3. **The falsifier matrix.** Positive: the three lane guests, lane-real == lane-spot, the nine
   pairs, the 28-byte line. Negative, each with its one-fault input, required verdict and error
   site: a missing/extra/wrong substitution per manifest; the cross-role pair in both
   directions (including the c212 STOP's own line); the receipt ordinary-vs-lane swap; an
   ordinary configuration carrying a lane manifest; the eight manifests unchanged; a lane guest
   that is not its role's guest.
4. **The lane build plan.** The four labels (`lane-real`, `lane-disabled`, `lane-armed`, and
   `lane-spot` == `lane-real`); the three derived physical roots; the nine substitutions; the
   `RUM_CM739_LANE_*` fields; no seat in the six-build class index; the twelve-label journal
   unchanged otherwise.
5. **The ripple.** The verifier and generator edits are under `crates/`: the F13 flat context
   moves, so the native producers run only on the final sources; the review destinations are
   re-rendered from the role trees plus the revised edits; the buildability matrix is re-run;
   the observer re-freeze covers the mutated verifier files; the 62 controls, the promotion,
   the eight non-lane builds and the drives are unchanged.
6. **The implementation draft** (`draft-amendment-4.md`): the exact edits, the producer and its
   pin, the falsifier runs, the F13 cells and the rest of Stage B, stages C–F unchanged.

A STOP fires on any remaining unknown, on any edit outside the authorized files without a
named generated-table adoption, and on any falsifier that cannot be made to fire.

**Stage F0 (300 s) — seal and report.** The design record and findings under `W/amendment-3/`;
the report citing every evidence line; the seal with every payload; the exit audit (zero live
changes, zero source edits, builds, producers, replays, suites and git attempts).

**Charge.** The Amendment 2 attempt charged 773.009 s of 36,000 (A0 52.153 of 200, B 236.784 of
6,822.697); unspent 33,956.238 s; cumulative 29,208.679 of 63,164.917. This amendment allocates
A0 200 / B0 1,200 / F0 300 = **1,700 s** inside that unspent; no new top-up. The implementation
amendment that follows will allocate A0 200 / B 6,822.697 / C 7,800 / D 14,400 / E 3,000 /
F 2,400 = 34,622.697 s, with its top-up (about 2,366 s at a full design spend) made explicit at
its dispatch under the 2026-11-19 rule. Charge is continuous from dispatch; every stage box is
measured from its own CLOCK entry.

## Director gate (c213, 2026-11-27) — the B0.1 STOP is accepted: the two-feature literal exists only at row 128, the eight singletons are `features = []`; the archived 797 constructor is the lane image function and Amendment 4 continues the design from it

The Amendment 3 attempt (unit `brief-812-cdx-20261127-024107-4178488`) ran 2026-11-27 02:41:07
→ 02:52, rc 0, 144,514 tokens, peak 359,161,856 B, oom 0/0, verdict STOP at Stage B0 item 1.
I verified: the seal 39 payloads / 1,871,036 B `bc7a931c…` (all re-hashed equal) and the 13
envelope rows; report `99e13901…` (check OK 138), PROGRESS `05e1857c…`, charge `ec3b744d…`;
A0 PASS (the stop-3 seal `37ac1d36…` with the two moved rows re-hash equal, the stop-2 and
stop-1 seals, four parents, three child trees, the retained F13 `8d6ba3f0…`, the 811 design
`da2fdf55…`/632 pins, and the gate c212 strict appends — `TODO.md` `cbb60278…` + 3,314 B
`484b4c94…` = `f92ad7fc…`, the brief `f6546e18…` + 7,100 B `aabcfa85…` = `c970cbe3…` — which
I re-derived from `git show 85909f0f:`/`git show 64b5cb47:`); entry == exit `e70e2449…` at
5,903 paths; `git status` clean at HEAD `64b5cb47`; zero edits, builds, producers, replays,
suites, adoptions or git attempts. Charge 665.241 s (A0 51.565/200, B0 297.995/1,200, F0
222.166/300; close 222.492). **Defect #47 (mine):** B0.1's two-feature literal was row 128's
only; the eight singletons' lane form is `features = []` (−28 B), so 24 of 27 cells were
no-ops and the missing-substitution control could not fire. I recomputed all 27 cells against
the archived constructor `.temp/brief-797/tools/gate/lane_build.py:10–15` (`manifest_bytes`):
27/27 reproduce the recorded pairs (real row 128 `6dcfaa9f…` 982 B → `7976deb8…` 952 B; class
form `67be9610…` 1,010 B → `7327bd4e…` 980 B). Amendment 4 continues the design with the
constructor as the lane image function.

## Amendment 4 — the lane model design continued (design-only, no source edits)

**Scope.** Design only: zero source edits, zero builds, zero producers, zero replays, zero
suites, zero adoptions, zero git. Every output under `W/amendment-4/`; the report at
`.temp/brief-812/report.json`; the stopped record at `.temp/director/812-stop-4/`.

**Stage A0 (200 s) — closure by hash identity.** The stop-4 record: the seal `bc7a931c…` (39
payloads / 1,871,036 B), the report `99e13901…` and PROGRESS `05e1857c…` (re-hash equal at
`.temp/director/812-stop-4/`), the charge `ec3b744d…`; the stop-3/stop-2/stop-1 records and
their seals; the four parents; the three child role trees; the retained F13; the 811 design
seal; and the live census against `.temp/cm4-campaign-epoch/amendment-3/live-exit-stat.json`
(`e70e2449…`, 5,903 paths), where the only permitted differences are `TODO.md` and this brief
as strict appends from the c212 state (`f92ad7fc…`, `c970cbe3…`) — old prefix + appended
range + new hash printed as `C213 APPEND PASS`; any other difference stops. `git status` clean
at the gate commit.

**Stage B0 (1,200 s) — the lane model design.** The archive from Amendment 3
(`lane-model.md`, `lane-falsifiers.json`, `pair-evidence.json`, `pin-producer.py.txt`) is
retained evidence, not an admitted design; the design record (`lane-model.md`) and the
executable falsifier matrix (`lane-falsifiers.json`) are written fresh under `W/amendment-4/`:

1. **The lane image function.** State the archived constructor `manifest_bytes`
   (`.temp/brief-797/tools/gate/lane_build.py:10–15`) as the single function: for every line
   beginning `rum-runtime =` or `rum-host =` that contains `features = [`, delete
   `"test-builtin-memory-growth", ` then `, "test-builtin-memory-growth"` then
   `"test-builtin-memory-growth"`; exactly nine files change and every other byte is
   identical; the computed pairs must equal the role's recorded pair (the 797 wire for the
   class form, `manifests.rs` for the real) exactly once per path — fail closed on either.
   State the corrected table: the eight singletons `["test-builtin-memory-growth"]` → `[]`
   (−28 B); row 128 two-feature → continuation-only (−30 B); role-relative only at row 128
   (`6dcfaa9f…` 982→952 `7976deb8…`; `67be9610…` 1,010→980 `7327bd4e…`). The
   missing-substitution input (the guest itself) is rejected on **all nine**.
2. **The verifier contract.** The role-relative table shape and the named producer (inputs:
   the 797 wire `ffed5667…`, the legacy table `e8740d51…`, the three role roots and the
   row-128 overlay row; output bytes pinned by their own hash; a generated-table adoption
   named in the commit). The `source_auth/mod.rs` restructure: in the lane configuration the
   nine manifests are all checked by the lane branch (the ordinary overlay loop does not check
   them there — row 128 included); the guest is the baseline's bytes for the real arm and for
   the eight non-overlay manifests on the copied arms, and the ordinary postimage derivation
   for row 128 on the copied arms; the copied file must equal the role's lane image; the
   receipt's row-128 overlay entry carries (pin, lane post) while the other 164 entries and
   the ordinary default/no-default configurations are unchanged; `schema.rs` unchanged; the
   branch must be proven reachable for row 128 with exactly one checking site per manifest.
3. **The falsifier matrix.** Positive: the three lane guests, lane-real == lane-spot, the nine
   pairs, the 28-byte and 30-byte lines. Negative, each with its one-fault input, required
   verdict and error site: for each of the nine paths the guest itself (missing substitution);
   an extra or wrong substitution; the cross-role pair in both directions (including the c212
   STOP's own line); the receipt ordinary-vs-lane swap; an ordinary configuration carrying
   lane bytes; the eight manifests left unchanged; a lane guest that is not its role's guest.
   Every case must be executable under the constructor rule (Amendment 3's static matrix
   proved 24 of 27 positives infeasible — the corrected rule must make all of them feasible).
4. **The lane build plan.** The four labels (`lane-real`, `lane-disabled`, `lane-armed`, and
   `lane-spot` == `lane-real`); the three derived physical roots; the nine substitutions; the
   `RUM_CM739_LANE_*` fields; no seat in the six-build class index; the twelve-label journal
   unchanged otherwise.
5. **The ripple.** The verifier and generator edits are under `crates/`: the F13 flat context
   moves, so the native producers run only on the final sources; the review destinations are
   re-rendered from the role trees plus the revised edits; the buildability matrix is re-run;
   the observer re-freeze covers the mutated verifier files; the 62 controls, the promotion,
   the eight non-lane builds and the drives are unchanged.
6. **The implementation draft** (`draft-amendment-5.md`): the exact edits, the producer and its
   pin, the falsifier runs, the F13 cells and the rest of Stage B, stages C–F unchanged.

A STOP fires on any remaining unknown, on any edit outside the authorized files without a
named generated-table adoption, and on any falsifier that cannot be made to fire.

**Stage F0 (300 s) — seal and report.** The design record and findings under `W/amendment-4/`;
the report citing every evidence line; the seal with every payload; the exit audit (zero live
changes, zero source edits, builds, producers, replays, suites and git attempts).

**Charge.** The Amendment 3 attempt charged 665.241 s of 1,700 (A0 51.565 of 200, B0 297.995 of
1,200, F0 222.166 of 300; the close interval 222.492 charged); unspent 1,034.759 s; pool
**33,290.997 s**; cumulative 29,873.920 of 63,164.917. This amendment allocates A0 200 / B0
1,200 / F0 300 = **1,700 s** inside that pool; no new top-up. The implementation amendment
that follows will allocate A0 200 / B 6,822.697 / C 7,800 / D 14,400 / E 3,000 / F 2,400 =
34,622.697 s, with its explicit top-up (3,031.700 s at a full design spend) at its dispatch
under the 2026-11-19 rule. Charge is continuous from dispatch; every stage box is measured
from its own CLOCK entry.

## Director gate (c214, 2026-11-27) — the lane model design is accepted: the archived constructor reproduced all 27 role pairs, the 211-case matrix is executable with the missing-substitution control firing on all nine paths, and the generated table is pinned at 2,390 B `0b7ffbe5…`

The Amendment 4 attempt (unit `brief-812-cdx-20261127-025916-4188007`) ran 2026-11-27
02:59:16 → 03:16, rc 0, 172,136 tokens, peak 416,980,992 B, oom 0/0, verdict DONE.
I verified: the seal 73 payloads / 7,057,217 B `b39ecb6d…` (all re-hashed equal) and the 13
envelope rows; report `34e63803…` (310,631 B; check OK 1,052), PROGRESS `b3e4509b…`, summary
`ec78b7b8…`, charge `cdf52b6a…`, design readback `f1f6753c…`; A0 PASS (the four stopped
records with their seals `cc3818f4…`/`a5bfaa5f…`/`37ac1d36…`/`bc7a931c…` and the two moved
rows equal, the 811 design `da2fdf55…`/30 payloads/632 pins, the four parents, the three child
trees, the retained F13 `8d6ba3f0…`/`d20465a0…`/`6a376127…`, and the gate c213 appends —
`TODO.md` `f92ad7fc…` + 2,618 B `cad5a933…` = `d6591d96…`, the brief `c970cbe3…` + 8,149 B
`8e6f8f2f…` = `482cb33b…` — re-derived from `git show 64b5cb47:`); entry == exit `2fc5bcbe…`
at 5,903 paths (my own census re-run equal 5,903/5,903); `git status` clean at HEAD
`79bc225c`; zero edits, builds, producers, replays, suites, adoptions or git attempts.
I re-ran the design myself: the archived constructor reproduces **27/27** pairs (eight
singletons −28 B; row 128 real `6dcfaa9f…`/982 → `7976deb8…`/952, class `67be9610…`/1,010 →
`7327bd4e…`/980); the class wire `ffed5667…`/13,404 is regenerated byte-for-byte and the real
wire `66a8e8bd…`/13,346 is the class wire less the 58 serialized characters of the 28-byte
`sha2` dev-dependency line; the lane trees `40147b53…`/`a55b447b…`/`2ba74bd2…` with
`lane-spot` == real; the generated table 2,390 B `0b7ffbe5…` (producer `2cfcea30…`);
`model.py verify` gives 37 accepted / 174 rejected / 0 unexpected, with the 27
missing-substitution inputs rejected on all nine paths per role. Charge 1,041.494 s
(A0 40.013/200, B0 849.130/1,200, F0 106.549/300; close 106.916). **Amendment 5** implements
the design.

## Amendment 5 — implement and qualify the P0543 role-relative source proof (implementation)

**Scope.** Implementation. The lane model design is accepted as delivered: the archived
constructor as the lane image function, the role-relative `PINS`/`COPIED_PINS` table through
the named producer, the two-stage verifier (`mod.rs` hoisted lane bool, ordinary overlay loop
skipping only row 128 in lane, the single `COPIED_PINS` pass, the final equality loop skipping
the checked sets), the executable matrix, and the ripple. `schema.rs`, `artifacts.rs`, the
zero-additions bytes and the private loader stay as authenticated. The generator executable
and its raw receipts live in scene tooling; only its generated Rust table is installed in the
copies. No git in any form; the director owns version control.

**Material and confinement.** Use `W=$PWD/.temp/cm4-campaign-epoch`, `D=$W/amendment-4`, and a
fresh `N=$W/amendment-5` for every writable role, review, target, control, renderer and
receipt. `SA=crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth`. The live
workspace, the 811 record, the four 806e roots, the stopped records and all design payloads
stay read-only. Preserve the projected 5,851 keys in each guest, with real as the baseline
(`689b9188…`, `f9661a93…`, `450b9e2f…`; four edits each).

Reapply brief 812's offline Rust 1.85.0, locked dependencies, edition 2024, eight jobs and
test threads, single release codegen unit, disabled incrementality and memory-growth
instrumentation, clean environment, scratch temp directories, 16 GiB bound, serial work and
process-group timeouts. Pin cwd, build/target roots, exact argv and environment, monotonic
start/finish, raw output, status, features, compiler and any binary for each operation. Use
closed stdin. The original one-thread controls and serial OOM accounting apply. Stop at the
first unsupported discrepancy, projected overrun or stage deadline. Record each entry and exit
CLOCK, the next immutable cell and attributable live changes.

**Clocks and charge.** Allocate A0 200 / B 6,822.697 / C 7,800 / D 14,400 / E 3,000 / F 2,400
seconds, total **34,622.697**. The explicit top-up at this dispatch is **2,373.193947 s**
(pool 32,249.503053 s; authorized total 65,538.110947 s) under the 2026-11-19 rule. All prior
charge stays on its own attempt.

**Stage A0 (200 s) — closure by hash identity.** Authenticate the Amendment 4 record: the
seal `b39ecb6d…` with its 73 payloads / 7,057,217 B and 13-row envelope, the report
`34e63803…` and PROGRESS `b3e4509b…` (re-hash equal at `.temp/director/812-done-5/`), the
charge `cdf52b6a…`; the four stopped records (stop-1 30,084 payloads `cc3818f4…`, stop-2
`a5bfaa5f…`, stop-3 83,936 payloads `37ac1d36…`, stop-4 `bc7a931c…`); the 811 design seal
`da2fdf55…` / 30 payloads / 632 pins; the four parents (`cb9205a6…`/5,842, `dda9146e…`,
`30cd48fc…`, `df0db63c…`/5,851) and the three child role trees (`689b9188…`, `f9661a93…`,
`450b9e2f…`); the retained R/default F13 tuple (`8d6ba3f0…`, `d20465a0…`, `6a376127…`).
Recheck `pair-evidence.json`, `lane-falsifiers.json`, `lane-model.md`, `model.py`,
`render_lane_pins.py.txt` and `draft-amendment-5.md` at their sealed hashes, and the physical
inputs: the 797 wire `ffed5667…`, the legacy table `e8740d51…`, the constructor `b80da290…`,
the row-128 line `eb93a0fc…`. Reconcile the 5,903-path live census with
`.temp/cm4-campaign-epoch/amendment-4/live-exit-stat.json` (`2fc5bcbe…`): the only permitted
differences are `TODO.md` and this brief as strict appends from `d6591d96…` and `482cb33b…`,
each printed as `C214 APPEND PASS` with old prefix, appended range and new hash; any other
difference stops. `git status` clean at the gate commit.

**Stage B (6,822.697 s) — implement the source proof and prepare the final inputs.** The
source presentation, including the single named table adoption, takes its charge inside B.
Complete it before the first native compile.

1. **The final handwritten changes in fresh copies.** Relative to the four-edit child root the
   handwritten delta is only `SA/mod.rs` and `SA/tests.rs`. Add the copied-arm lane routing and
   tests specified below. Use the generated-table adoption `SA/manifests.rs — generated P0543
   role-relative manifest pins` identically in all three copies. Record the adoption name,
   producer digest, four input file identities, three input roots and output digest in the
   director's commit and in the role ledger. Keep `SA/schema.rs`, `SA/artifacts.rs`, the
   zero-additions bytes and the private loader from the authenticated input. Preserve the 199
   seals, the historical nine-added-key classification, all source keys and the 165 role
   differences.
2. **The table.** Copy the bytes of `D/render_lane_pins.py.txt` to `N/tools/render_lane_pins.py`
   and require SHA-256 `2cfcea30376c969b2bc331e1fa17e8ef627586adb47cf4bdd0e35a461f38996c`.
   Run the following bounded command from the workspace with the stated clean environment,
   creating the empty output staging directory first. Use the **authenticated pre-edit** inputs
   for the producer; compare its raw stdout against the independent `ground.py` byte
   specification and its pin; preserve stderr, pin and adopt only after exit 0.

   ```sh
   scripts/run-bounded.py --timeout 30 -- python3 -IB "$N/tools/render_lane_pins.py" \
     --wire "$PWD/.temp/brief-797/amendment6/lane/lane-manifest-diff.json" \
     --legacy "$W/roles/real/$SA/manifests.rs" \
     --constructor "$PWD/.temp/brief-797/tools/gate/lane_build.py" \
     --row128 "$W/amendment-2/pin-evidence/real-R4-128.tsv" \
     --real "$W/roles/real" --disabled "$W/roles/disabled" --armed "$W/roles/armed" \
     < /dev/null > "$N/table-output/manifests.rs" 2> "$N/table-output/stderr"
   ```

   The output must be 2,390 bytes, SHA-256 `0b7ffbe53c56f4e62a3c1f400b01120c564dc32a2d4914f5091d97d45356467a`.
   The arrays are `PINS` for real and `COPIED_PINS` for the shared copied pair columns.
   Keeping the real array's compatible tuple type leaves its existing `artifacts.rs:55–58`
   consumer valid. Journal all common edits with actual before/after digests. Format and run
   `cargo fmt --all -- --check` on each final input. Reconstruct every review destination from
   that exact role plus the revised files. Compare all paths and digests before each compiler
   invocation. A subsequent source change invalidates the affected review, buildability and
   native F13 evidence.

3. **The handwritten verifier and fixtures.** In `SA/mod.rs:186–198`, calculate
   `lane = receipt.fields["configuration"] == "p0543-unchecked"` and retain the exact compiled
   overlay set. In the ordinary overlay loop, immediately continue for a lane path in
   `COPIED_PINS`. Leave all other checks in sequence. Keep `load()?`, additions validation and
   the late configuration validity check at `:199–224`. After that check, for lane iterate
   `COPIED_PINS` once. Take the eight non-overlay originals from the real baseline. At row 128
   check baseline digest and receipt pre against its overlay pin, then derive the ordinary
   bytes with `schema::file_postimage(file, before, A::ENABLED)?`. Match ordinary guest digest,
   physical lane digest, and finally the optional overlay receipt post. Return `Manifest` at
   either role-pair mismatch, `Preimage` at the overlay pre and `Postimage` at the receipt
   post. Preserve `file_postimage`'s own errors. The final unlisted equality loop skips checked
   overlay keys and the checked lane set. The paths form a disjoint partition. See
   `lane-model.md` for the real branch, exact error order and original source locations.

   Extend `SA/tests.rs` with the exact integration selector
   `oracle::mutations::source_auth::tests::cm_mutation_lane_authentication_controls`. Use the
   real baseline and read-only staged role snapshots. Invoke the existing private
   `verify::<Disabled/Armed>` (and the public-to-the-oracle artifact reader for real), with
   valid controls also submitted to `artifacts::artifact` for the copied roles. Exercise the
   same class-form receipt grammar as in the physical runs. Provide scope-local fixtures at
   already admitted roots. Build fixtures and rehash disposable envelopes within the test
   harness. Preserve the typed production wrapper and its sole additions loader.

4. **The falsifier matrix.** Consume `lane-falsifiers.json` (SHA-256
   `52b46cc12f1a1d0a2495e67ed4124e0ba5eb7707199715a821ec4f622ac3d14f`) under both ordinary
   importer modes. The full 211-case model has 37 positives and 174 negatives. Its pure
   producer-pair and wrong-root guards run against the independent generator boundary. Drive
   every Rust-layer case through the actual verifier. Generate the eight singleton edits and
   the two row-128 images from their permitted guest bytes. For each case record the sole input
   delta, typed verdict, checking site, role and configuration. Require the positive twin, each
   per-path missing/extra/wrong failure, both cross-role directions including the verbatim c212
   witness, the eight omissions together, the non-injective wrong guest, swapped receipt
   pre/post, the whole foreign guest, and every ordinary-configuration control. Test both
   receipt directions on the exact physical bytes that pass the other checks. Use the exposed
   typed result and the single-site traversal contract to establish nine lane checks, zero
   ordinary visits to row 128 in lane, 164 other ordinary overlays in copied lane, all 165 in
   copied ordinary, and zero real overlay entries. A static partition check plus independent
   per-key one-fault proofs suffices. The former failing dual-literal projection and old
   receipt order stay calibrated in the archived c212/c213 evidence.

   The runnable model defaults to read-only verification, with an optional single `--case ID`.
   Its observations include raw byte edits and full path traces. Invoke the compiled selector
   on each newly built ordinary real oracle, under
   `scripts/run-bounded.py --timeout 180 -- <mode-real-ELF> oracle::mutations::source_auth::tests::cm_mutation_lane_authentication_controls --exact --nocapture --test-threads=8`.
   Use the forced real/spot ELF readbacks for binary identity. Run the model and the producer
   guards in a disposable review directory where the input or output is intentionally faulty.
   Carry these as zero-credit source-proof controls alongside the existing 811 negative
   controls.

5. **The buildability matrix, native inventories and source closure.** The first native
   operation is the six-leg buildability matrix on final sources. Iterate real, disabled,
   armed, each default then no-default. Isolate the six compile targets and six writable
   review destinations. For `F` use no extra argument or `--no-default-features`. Run
   `scripts/run-bounded.py --timeout 650 -- cargo test --offline --locked -p rum-stdlib --lib --no-run F`.
   Require five builds. Real/no-default must exit 101 with just E0432 at
   `crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs:10:5` and zero tests run.
   Preserve both sides of this discriminator. The release integration target has a separate
   contract.

   Run fresh real/default, disabled/default, disabled/no-default, armed/default and
   armed/no-default with the selector below, each at its role/mode compile target and review
   cwd. The emitted path must belong to that same review. Archive the native bytes, build,
   source identity and log. The retained `8d6ba3f0…` is a predecessor witness; the revised
   sources require a new real/default run. The real/no-default cell remains the buildability
   falsifier.

   ```text
   scripts/run-bounded.py --timeout 650 -- cargo test --offline --locked -p rum-stdlib --lib F tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --ignored --exact --nocapture
   scripts/run-bounded.py --timeout 650 -- cargo test --offline --locked -p rum-stdlib --lib F tests::retention_closure::local::contract_tests::local_concrete_contract_controls -- --exact --nocapture
   scripts/run-bounded.py --timeout 650 -- cargo test --offline --locked -p rum-stdlib --lib F tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
   ```

   Before adoption require 68 rows with the original 18 identity columns, header, comments and
   order. Require both copied role streams to agree within each role. Independently calibrate
   `.temp/brief-790/overlay_digest.py:72–100` on the three parent roles (`217bec91…`,
   `40e92684…`, `d3588fb9…`). Feed the actual per-role final input delta, including `mod.rs`,
   `tests.rs` and `manifests.rs` as well as the inherited schema edit. Require equality to all
   five native outputs. Install those native bytes in each role and both review destinations.
   Rebuild before checking marks. Run the second selector on the five buildable cells. Require
   the exact 62 discharged `.red-green.txt` files per cell with `RED stale contract
   fingerprint` and `GREEN exact current mark`. Confirm the real zero-overlay promotion by byte
   equality to its baseline `4337950c…`. Use the third selector for disabled and armed in both
   modes, merge complementary body/instance cells from review onto the parent's carry, adopt
   and check a second time. Require review/check equality, both anchors C01/C02, 2 bodies, 3
   marks, 4 probes and zero rejections. Preserve the original negative outputs.

   Use `.temp/brief-811/producer-commands.json` for the original exact argv and source receipt
   of each remaining job. Translate every writable source/output/target root and feed the new
   review files. Take the armed/default ownership review's four selectors and seven outputs,
   recompute 152 chain hashes, run both 202-source class projections and their negatives, LP01
   producer/acceptance/negatives, typed receipts and the rejection census. Require 2,422 target
   keys, 1,195 rejection facts (10 RET001, 1,147 RET002, 38 RET003, 16 PUBLIC_WRAPPER), and no
   unexplained key, position, source text or disposition difference. Re-render all ten prior
   output paths from the actual inputs. Replay all affected standing inventories **before the
   first acceptance suite**. Quote the exact replay verdicts. A red solely from a stale
   fingerprint, certificate or position is an ordering artifact: preserve its original line and
   repeat after valid native adoption. A changed semantic result stops the gate.

   Compare every one of the 190 R4 rows across parent, producer and all guests, including
   identities and rows 171–172, 182–188. Track the additional *common* generated-table adoption
   separately. Require two independent wire serializations, 165 ordered overlay keys, 330
   decoded full images, native F13 pre at row 171 and native class posts, the literal 32-byte
   zero-additions wire, 151 mutant schema cells and the 64-hex-plus-LF external digest. Row
   128's ordinary pre and post stay ordinary in `overlays.tsv`; the lane adjustment belongs to
   the physical receipt only.

   Re-freeze observation metadata against the three complete final source manifests, including
   the verifier and generated table. Verify the 806e physical table `3953aa9f…` and the
   separate 807f v2.5 interference parent `08c03054…`. Audit carriers, callers, key/tail and all
   NativeEqual precedence collisions. Require the 28/3/7/113 membership partition, all ten
   preserved classes, 151 own + 100 pair + 4 structural selections per mode, 9,060 unique
   program inputs, 9,511 ordered constituents and 151 own joins against 755. The deleted 742
   inputs and the switched ELF list remain historical. Retain every emitted path, byte count,
   digest and raw evidence. A verifier edit after freeze restarts the applicable standing
   evidence on final sources.

**Stage C (7,800 s).** Apply brief 812 lines 164–304 and Amendment 1 lines 415–417, including
every original bound, cost and stop, with the lane bindings: twelve fresh labels at one stable
source and target path, offline release with one codegen unit, ordered default-real/disabled/
armed/spot, no-default-real/disabled/armed/spot, then lane-real/disabled/armed/spot; the exact
`suspension_oracle --no-run --release` Cargo command, 1,400 s inner and 1,495 s outer bounds,
the precomputed warning/site catalogue, explicit compiler artifact, exact ELF digest and
start/finish journal with `cache_hit=false`. For each spot force a genuine rebuild at the same
paths; compare each spot to real and distinguish the three role ELFs per configuration. For
each lane role use the nine exact compiler substitutions and its own fresh raw diff; the three
derived physical roots and the four `RUM_CM739_LANE_*` values are defined in `lane-model.md`.
The twelve-label journal carries all four lane labels; the six-build class index has only
ordinary labels.

For the scene-local **lane receipt writer**, adapt the seven-field and sorted-triple wire in
`.temp/brief-797/tools/gate/receipts.py:34–51`. Read the final trusted ordinary overlay table
and role source, independent compiler receipt, actual lane files, ELF, raw diff and both
Snapshot trees. Set `configuration=p0543-unchecked`. Check the guest/image byte pair for every
manifest. Real emits zero triples. Disabled and armed emit exactly 165. Only for the stdlib
row, preserve the ordinary real pin and encode the verified **physical** lane post. Each other
copied triple uses the independently decoded ordinary arm post. Decode with the field/order
and key/cardinality predicates at `receipts.py:53–80`, then require exact per-field and
per-triple comparison against independent input bytes. Exercise the receipt swap controls in
both directions. Keep `overlays.tsv` ordinary and the two real importer ELFs ordinary.

Carry the separate default-disabled diagnostic receipt and foreign-binary rejection, the four
auxiliary drive builds and 20 drives, five armed identities, disabled cohorts, caller/fixture
counts and three memory peels. Preserve the private-frame E0603 falsifier. Freeze the original
`W/control` generator/continuation/twin/pure-compiler package, including the two ordinary
importers' fourteen signed compiler controls. The eight ordinary journal labels, the
independent build and the drives retain their existing contracts.

**Stage D (14,400 s).** Acquire nine fresh CLI companions, verify both frozen programs, fixed
stderr and the exact 120-pair, 5,519-byte normalized stdout. Execute all six workspace cases
with the ordered seven skip pairs, nine host groups, current target-by-target comparison,
exact same-binary adjudications and six ledgers. Pin the preregistered admissible differences
and raw output digests. Apply the full time normalization, falsifier and hardware rules in
brief 812 lines 223–270.

**Stage E (3,000 s).** Freeze the six-build subject, validate the ordinary mode-specific
importers, zero-build-credit controls, spots and all named negatives. Acquire six fresh C10
runs on the bound ordinary builds. Both importers must give 0/151 on the build-only control
and 1/150 on C10, 55 real keys, one hit, zero settlements and the expected partial-index
status 101. No lane build enters this index.

**Stage F (2,400 s).** Publish the child epoch identity only after the full raw-artifact
proofs, qualification and immutable readbacks. Seal the complete payload and report, verify
the live exit census and charge every operation. Class admissions, member cells, new catches
and schedule credit are zero. The thirteen Q25 STOPs, the 744/745/746 holds and CM5 hold carry
forward.

## Director gate (c215, 2026-11-27) — the Stage C warning STOP is accepted: the 754 contract was measured on the pre-install sources, the post-install roles carry the D.2 diagnostic delta (real +1, each copied arm +11), and Amendment 6 re-bases the contract on the authenticated 797 release transcripts

The Amendment 5 attempt (unit `brief-812-cdx-20261127-032622-8556`) ran 2026-11-27
03:26:22 → 04:08:55, rc 0, 427,211 tokens, peak 3,119,050,752 B, oom 0/0, verdict STOP.
I verified: the seal 91,654 payloads / 14,828,579,933 B `b3e24975…` (all re-hashed equal)
and the 14 envelope rows; report `eb3177d5…` (1,218,807 B; check OK 5,833), PROGRESS
`b5531ba7…`, summary `971eb70d…`, charge `1b87434b…`; A0 PASS (the Amendment 4 record, the
four stopped records, the 811 design, the four 806e parents, the three four-edit children,
the retained F13, and the two c214 appends as strict appends — `TODO.md` `d6591d96…` +
2,866 B = `e47d04d0…`, the brief `482cb33b…` + 21,957 B = `1df00ae3…`); entry == exit
`02856220…` at 5,903 paths (0 added, 0 removed, 0 changed); `git status` clean at HEAD
`6d8254e8`; zero live changes, git invocations, member cells, catches or schedule credit.
Stage B delivered and I read its code: the three role deltas are exactly `mod.rs`
`b29014ce…`→`63ead2d1…`, `tests.rs` `cde634e6…`→`9f5aea09…` and `manifests.rs`
`e8740d51…`→`0b7ffbe5…` (2,390 B, producer `2cfcea30…`, independent spec equal); the six-leg
buildability matrix with the real/no-default E0432 at `reconcile.rs:10:5`; the five native
F13 cells (`04cd25a9…`, `4b2daee7…` ×2, `349f6783…` ×2) with 62+62 controls each and the
independent projection equal; the real zero-overlay promotion `4337950c…` and the four C01
review ordering artifacts with green check rounds; the seven standing replays (2,422 class
keys both modes, census 1,195 = 10/1,147/38/16, chain 152, LP01 1,115, 15 typed receipts);
`MODEL PASS cases=211 positives=37 negatives=174` and the six producer guards; the R4 render
(190 rows, 165 overlays, 330 images, two serializers, `bd1e9479…`/14,068,709 B, additions 0);
the observer re-freeze (`3953aa9f…`, `08c03054…`, 151/28/3/7/113, 9,060/9,511/151). Stage C
stopped on `default-real`: Cargo exit 0 and a fresh release ELF `ac842db5…` (73,107,432 B,
`cache_hit=false`, 104.101 s) with one `dead_code` warning (`class_provenance` at
`coverage/bindings.rs:112`) against the frozen allowance 0. I reproduced the chain: the 754
contract (`2377f246…`) is the pre-install surface (0/130/132 on a 5,806-file presentation
`a5084dd2…`); the real `coverage/bindings.rs` `faa15189…` is the 806e parent file and has
the declaration with zero callers (each copied arm's `25ea77fa…` has two class-form callers);
the 797 post-install release transcript (log `e3a8ee84…`, tree `022d60af…`) already records
exactly 1 real, 141 disabled and 143 armed diagnostics, and every post-754 addition sits in
a file byte-identical to the final source (real 1 `dead_code`; per arm 2 `Cell`
`unused_imports` + 9 `dead_code`). Charge 2,544.393 s (A0 62.153/200, B 2,149.892/6,822.697,
C 123.477/7,800; D/E/F 0; pre-A0 55.526, close 153.345); unspent 32,078.303622 s; cumulative
33,459.807325 of 65,538.110947. **Defect #48 (mine):** the warning bar re-applied the
pre-install contract to the post-D.2 sources and was unsatisfiable for every role; the
resolving record was already a catalogue input. **Amendment 6** re-bases the contract on the
authenticated post-install transcripts and continues C–F at 27,776.523 s, no top-up.

## Amendment 6 — C–F on the post-install warning contract (implementation continuation)

**Scope.** The accepted Amendment 5 B products stand and the Stage C STOP is accepted. No
source re-edit and no design change: the lane model, the source presentation, the rendered
tables, the observer freeze and the standing replays are as sealed. This amendment re-bases
the one input that stopped Stage C — the warning contract — and then completes Stage C and
stages D–F exactly as Amendment 5 wrote them (brief 812 lines 164–304, Amendment 1 lines
415–417 and the Amendment 5 stage texts).

**Material, confinement and clocks.** Use `W=$PWD/.temp/cm4-campaign-epoch`,
`A5=$W/amendment-5`, and a fresh `N=$W/amendment-6` for every writable role, review, target,
control, renderer, label and receipt. `SA=crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth`.
The live workspace, the 811 record, the four 806e roots, the stopped records and all
Amendment 4/5 payloads stay read-only. Preserve Amendment 5's offline Rust 1.85.0, locked
dependencies, edition 2024, eight jobs and test threads, single release codegen unit,
disabled incrementality and memory-growth instrumentation, clean environment, scratch temp
directories, 16 GiB bound, serial work and process-group timeouts. Pin cwd, build/target
roots, exact argv and environment, monotonic start/finish, raw output, status, features,
compiler and any binary for each operation. Use closed stdin. The original one-thread
controls and serial OOM accounting apply. Stop at the first unsupported discrepancy,
projected overrun or stage deadline. Record each entry and exit CLOCK, the next immutable
cell and attributable live changes. Only the director operates version control.

**Stage A0 (300 s) — closure by hash identity.** Authenticate the Amendment 5 record after
its director relocation: the seal `b3e249751d10ac011c9aa29f74b573a9d7a5533577c6ccee8f79aa780ed4abe2`
(91,654 payloads / 14,828,579,933 B; re-hash every payload and the 14 envelope rows; the
report and PROGRESS are at `.temp/director/812-stop-5/report.json` `eb3177d5…` and
`PROGRESS.md` `b5531ba7…`, re-hashed equal); charge `1b87434b…`; the four 806e parents
(baseline `cb9205a6…`/5,842; real `dda9146e…`; disabled `30cd48fc…`; armed `df0db63c…`/5,851
each), the three four-edit children (`689b9188…`, `f9661a93…`, `450b9e2f…`), the retained
real/default F13 `8d6ba3f0…` with receipt `d20465a0…` and raw `6a376127…`, the 811 design
`da2fdf55…`/30 payloads/632 pins; the five native F13 cells `04cd25a9…`, `4b2daee7…` (both
disabled modes), `349f6783…` (both armed modes); the render tables (`overlays.tsv` `bd1e9479…`
/14,068,709 B, `mutants.tsv` `32bb4f7f…`, `additions.tsv` `33478d6e…`, `source_schema.txt`
`015ca3c3…`), the R4 ledger `05f3cf3b…`, the ten-output record, the observer manifest
`50cc4a95…`; the final role trees (`b0bdcee9…`/`084f8cc8…`/`d5267244…`) with their freeze
manifests (`8ae746d2…`/`318ef7c7…`/`cbd1bd3c…`). Reconcile the live census against
`02856220…` at 5,903 paths, permitting only this gate's two strict appends from the c214
state — `TODO.md` (`e47d04d0…`, 1,129,911 B) and this brief (`1df00ae3…`, 73,595 B) — each
verified by slicing at the old size, matching the old prefix hash, and printed as
`C215 APPEND PASS` with old prefix, appended range and new hash. Any other difference stops.
Print `A0 PASS`.

**Stage C (7,676.523 s).** Stages C–F of Amendment 5 (brief 812 lines 164–304 and the
Amendment 5 text) apply unchanged, with one re-based input and the twelve labels rebuilt
fresh in this attempt.

1. **Warning contract v2.** For each role build the catalogue from two authenticated
   authorities: the 754 report's warning policy (the `WARNING CLASS/REFERENCE/SITES` lines,
   families and counts 0 real / 130 disabled / 132 armed, 124 parentheses per copied role)
   and the 797 source-gate post-install release transcript for `default-<role>`
   (`.temp/brief-797/amendment6/source-gate/builds/default-<role>.json` and its
   `probe/logs/<key>.log`; require cargo exit 0, `--release`, no cache hit). Relocate every
   transcript diagnostic's primary spans onto the final role files by difflib matching
   blocks with exact source-text equality; a missing mapping, a changed source line, a
   changed message or a changed family stops. The allowed multiset is the relocated
   transcript set; the 754 contract (families and counts, the 124-parentheses sites rule and
   the 0/130/132 totals) must remain a sub-multiset of it. The frozen allowed counts are
   **real 1** (`dead_code` 1), **disabled 141** (124 `unused_parens`, 4 `unused_imports`,
   1 `non_camel_case_types`, 2 `non_snake_case`, 1 `unused_must_use`, 9 `dead_code`) and
   **armed 143** (the same plus 2 `unexpected_cfgs`). Record the post-754 delta per role
   line by line (real +1 `dead_code` `class_provenance` at `coverage/bindings.rs:112`; each
   copied arm +2 `Cell` `unused_imports` + 9 `dead_code` at `cm740_probe/class_lease.rs`,
   `class_rows.rs`, `q25_ownership.rs`, `suspension/core.rs:708`), each with its verified
   site, message and final file digest. Freeze the catalogue and print its digest. Nothing
   else may be added later.
2. **Twelve fresh labels.** Build in the Amendment 5 order (`default-real default-disabled
   default-armed default-spot no-default-real no-default-disabled no-default-armed
   no-default-spot lane-real lane-disabled lane-armed lane-spot`) at one stable source and
   target path, exactly once per label, with the exact `cargo test --offline --locked -j 8
   -p rum-stdlib --message-format=json --test suspension_oracle --no-run --release F`
   command, 1,400 s inner and 1,495 s outer bounds, `cache_hit=false`, one release codegen
   unit, the start/finish journal, the explicit compiler artifact and the exact ELF digest.
   For each label the warning receipt must equal its role's frozen allowed multiset exactly:
   no missing, no extra, no unknown family, the site and source-text checks, and the
   parentheses counts. A warning verdict FAIL stops. For each spot force a genuine rebuild
   at the same paths; compare each spot to real and distinguish the three role ELFs per
   configuration. For each lane role derive the physical lane presentation from the final
   role root with the nine exact `manifest_bytes` substitutions (the archived constructor
   `.temp/brief-797/tools/gate/lane_build.py:9–15`), verify the nine before/after pairs and
   the raw diff, and set the four `RUM_CM739_LANE_*` values; `lane-spot` == real. The
   twelve-label journal carries all four lane labels; the six-build class index has only
   ordinary labels.
3. **Lane receipts.** Adapt the seven-field and sorted-triple wire in
   `.temp/brief-797/tools/gate/receipts.py:34–51` to the final table and sources. Set
   `configuration=p0543-unchecked`; check the guest/image pair for every manifest; real
   emits zero triples, disabled and armed exactly 165; only row 128 carries the verified
   physical lane post, every other copied triple the independently decoded ordinary arm
   post; decode with the field/order and key/cardinality predicates at `receipts.py:53–80`
   and require exact per-field and per-triple equality; exercise both swap directions.
   Keep `overlays.tsv` ordinary and the two real importer ELFs ordinary.
4. **Rust selector runs.** Stage the selector fixture at an admitted scratch root: the
   ordinary and lane snapshots of the three roles, the twelve label ELFs and receipts,
   `cases.tsv` with SHA-256 `b91020d7…` (205 rows) and `snapshot-order.txt` (5,851 keys).
   Invoke the compiled selector
   `oracle::mutations::source_auth::tests::cm_mutation_lane_authentication_controls` on the
   built `default-real` and `no-default-real` ELFs under `scripts/run-bounded.py
   --timeout 180 -- <mode-real-ELF> … --exact --nocapture --test-threads=8` with
   `RUM_CM740_LANE_CONTROLS=<fixture>`; require the exact
   `P0543 RUST MATRIX PASS cases=205 accepted=37 rejected=168 producer_guards=6
   row128_ordinary_lane_visits=0 copied_lane_overlays=164 copied_ordinary_overlays=165
   real_overlays=0 manifest_checks=9` line from both. Use the forced real/spot ELF
   readbacks for binary identity. These are the two release-importer controls Amendment 5
   left pending; the six producer-pair and wrong-root guards run at the generator boundary
   as sealed in Amendment 4.
5. **Carry** the separate `default-disabled` diagnostic receipt and foreign-binary
   rejection, the four auxiliary drive builds and 20 drives, the five armed identities, the
   disabled cohorts, the caller/fixture counts and the three memory peels; preserve the
   private-frame E0603 falsifier; freeze the `W/control` generator/continuation/twin/
   pure-compiler package including the two importers' fourteen signed compiler controls.
   The eight ordinary journal labels, the independent build and the drives retain their
   existing contracts.

**Stage D (14,400 s), Stage E (3,000 s), Stage F (2,400 s).** As Amendment 5 wrote them,
including every bound, cost and stop: the nine CLI companions and both frozen programs,
fixed stderr and the exact 120-pair 5,519-byte normalized stdout; the six workspace cases
with the ordered seven skip pairs, nine host groups, current target-by-target comparison,
exact same-binary adjudications and six ledgers; the full time normalization, falsifier and
hardware rules of brief 812 lines 223–270; the six-build subject freeze, the ordinary
mode-specific importers, zero-build-credit controls, spots and named negatives; six fresh
C10 runs with both importers at 0/151 on the build-only control and 1/150 on C10 (55 real
keys, one hit, zero settlements, partial-index status 101); no lane build in the index; and
the child epoch identity, complete payload/report seal, live exit census and continuous
charge. Class admissions, member cells, new catches and schedule credit are zero.

**Charge.** The Amendment 5 attempt charged 2,544.393377992 s of 34,622.697 (A0 62.153/200,
B 2,149.892/6,822.697, C 123.477/7,800; D/E/F 0; pre-A0 55.526, close 153.345); unspent
**32,078.303622008 s**; pool after the top-up 34,622.697000185 s; cumulative
33,459.807324807 of 65,538.110947. This amendment allocates A0 300 / C 7,676.523 / D 14,400 /
E 3,000 / F 2,400 = **27,776.523 s** of that unspent amount; **no new top-up**; the remaining
4,301.780622 s stays unallocated. Charge is continuous from dispatch; every stage box is
measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** The thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381,
P2385, P2387, P2409, P2411, P2416, P2418. Briefs 744, 745 and 746 stay held; CM5 stays
unauthorized. Stop on a pin, sourceset, warning (missing/extra/unknown family, site or
source-text mismatch, a changed message, a changed 754 family or count, or a warning verdict
failing), parser vector, producer, position/identity, visibility, build/spot, raw receipt,
functional or comparison mismatch, a missing bound input, an unavailable required
interference proof, a first failed label, a projected overrun or a stage deadline.

## Director gate (c216, 2026-11-27) — the Stage C drive STOP is accepted: the 754 pin-hold presented the ordinary source bytes to the class-form loader; Amendment 7 re-bases the drive's presentation on the class-form authority

Amendment 6 (this brief, implementation continuation, 27,776.523 s of the unspent epoch
allocation, no top-up) was dispatched 2026-11-27 04:20:07 as unit
`brief-812-cdx-20261127-042007-45219` (background task `basarvvbn`, log
`.temp/logs/cdx_brief_812.log`; Codex CLI 0.171.0, `gpt-6.2-astra`, effort max,
workspace-write, session `01a21983-f473-7ca3-a4bb-52abd08ecffb`) and returned rc 0 at 05:02,
425,207 tokens, peak 6,716,817,408 B, oom 0/0, verdict **STOP**.

A0 PASS (82.455 s): the moved Amendment 5 record re-authenticated (report `eb3177d5…` 1,218,807 B,
PROGRESS `b5531ba7…`, seal `b3e24975…` 91,654 / 14,828,579,933 B, charge `1b87434b…`), the
four 806e parents and three four-edit children, the retained real/default F13, the 811 design
`da2fdf55…`, the render inputs and the observer manifest `50cc4a95…`, the final role trees (real
`b0bdcee9…`, disabled `084f8cc8…`, armed `d5267244…`) and the five native F13 cells; the two
c215 appends as strict appends (`TODO.md` `e47d04d0…` + 4,178 B = `06cbc0d1…`; this brief
`1df00ae3…` + 14,802 B = `99149770…`). The warning contract v2 held: the catalogue froze at
`05c456a9…` (real 1 / disabled 141 / armed 143, the post-754 delta recorded line by line), all
twelve labels passed with `cache_hit=false`, Cargo exit 0, one codegen unit and their own
receipts (`missing=0 extra=0 unknown_families=none source_mismatches=0`), the three spots
reproduced the real ELFs, the twelve receipts and eight opposite-post swap rejections passed,
the three lane trees differ from their roles in exactly the nine manifests, both 205-case Rust
selectors passed with the exact matrix line on the freshly built `default-real`/`no-default-real`
ELFs (fixture `b91020d7…`), and the `default-disabled` diagnostic binary accepted its own
receipt and rejected the indexed binary at `Binary`. Stage C then stopped at the first carrier
drive, `default-armed` P0267: `CM742C81 DRIVE RED programs table rejected: SourceIdentity`,
exit 101 before plan selection. The worker had applied the historical 754 pin-hold, presenting
the real role's ordinary bytes over 60 paths of the armed drive tree (`presented=c7a3d9d6…`,
`compiled=d73aa794…`); the installed class-form loader rejected
`crates/rum-host/src/session.rs` at `class-provenance.tsv` line 254 (expected `7127298d…`,
supplied `ef78a1de…`), 2 canonical rows of 141. The record: seal 403,636 payloads /
15,243,990,447 B `b9e200e9…` (all re-hashed, 14 envelope rows), report `b0be6df3…`
(1,404,532 B, check OK 2,047), PROGRESS `dbba8210…`, summary `2cc643e7…`, finding `30a8e13f…`,
charge `b531f7ff…`; entry == exit `49bd8faf…` at 5,903 paths, `git status` clean at `3a705b64`;
zero live changes, git invocations, member cells or credit. Charge 2,545.405144187 s of
27,776.523 (A0 82.455/300, C 1,970.313/7,676.523; pre-A0 50.042, close 442.596); unspent
25,231.117856 s; returned 29,532.898478 s; cumulative 36,005.212469 of 65,538.110947.

**The substance and defect #49 (mine).** The loader (`fixtures/obligations/guest.rs:1319–1363`,
with the guest-route and dependency checks at `:1429–1445` and `:1547–1558`) resolves sources
through `class_provenance()` — the class table `70a6da97…`, identical in all three role trees,
whose 141 canonical-source rows carry `LEDGER_PIN 73ec2ff5…` — and its pins are the **class
form** (the armed bytes). The 199 pinned paths are the ordinary table's key set with different
hashes: the real tree satisfies the ordinary authority exactly but misses the class authority
at 60 paths, the armed tree satisfies the class authority everywhere, and the disabled tree
satisfies every canonical row but misses two non-canonical loader inputs
(`eval_dispatch/mod.rs`, guest-route for the P0543/P0544/P0508/P1147 rows;
`profiling/logical/segments.rs`, the C10 dependency). The 754 pin-hold presented ordinary
(real) bytes — satisfiable only against the pre-install `provenance()` loader — and on the
post-install tree it mismatches the class loader at exactly two canonical rows
(`session.rs:254`, `worker.rs:260`), stopping the drive. My reproduction: the two tables share
the 199-key set; the real tree has 2 canonical mismatches, armed 0, disabled 0 canonical /
2 non-canonical; the presented tree `c7a3d9d6…` is rebuilt byte-exactly from the compiled tree +
real bytes at the 60 differing paths; and **the corrected presentation passes the drive live**
— the sealed drive binary `e539fb09…` on the compiled tree `d73aa794…` loaded, fired and
passed in 3.28 s (`CM742_JOIN_HIT_ID program="CM740-GUEST" pair=1 activation=[("P1568", 1),
("P1577", 0), ("P0267", 3)] producer_occurrence=0 generation=2 namespace=…`,
`CM742C81 DRIVE VERDICT row=P0267 armed=true fired=true`, tree unchanged). **Defect #49
(mine):** the drive clause carried the 754 pin-hold (the ordinary authority) onto the
post-install tree without the class-authority rebase — the same defect class as #48, one stage
later. The STOP is the rule working; no worker error.

## Amendment 7 — the class-form drive presentation and the C–F completion (implementation continuation)

**Scope.** Continue brief 812 from the accepted Amendment 6 products. No source edit; every
build and measurement lands under `.temp/cm4-campaign-epoch/amendment-7/`; the sealed parents
and the 754/797/806e records stay read-only. The twelve canonical labels, the three spot
identities, the two 205-case Rust selector runs, the `default-disabled` diagnostic and the
`default-armed` drive build (`e539fb09…` on `d73aa794…`) are reused by hash identity — no
re-run, no rebuild; their receipts are re-read from the Amendment 6 seal.

**A0 (300 s).** Close by hash identity against the Amendment 6 record: the seal `b9e200e9…`
(403,636 payloads / 15,243,990,447 B), report `b0be6df3…`, PROGRESS `dbba8210…`, summary
`2cc643e7…`, finding `30a8e13f…`, charge `b531f7ff…`; the moved report/PROGRESS at
`.temp/director/812-stop-6/`; the four 806e parents, three children, five native F13 cells and
the observer manifest; and the live census against `49bd8faf…`, permitting only `TODO.md` and
this brief to differ, each verified as a strict append and printed as `C216 APPEND PASS` (old
prefix, appended range, new hash); any other difference stops. The live entry census is taken
after this gate's commit.

**Stage C (6,500 s). The drive's source presentation is the loader's authority.** For each of
the four drive trees (default and lane × disabled/armed: the role or lane tree plus the
reviewed `cm742_c81_drive.rs` helper and the `obligations/mod.rs` hook — two drive-only
differences), before every run: verify the loader's own predicate set on the drive tree — the
`program-table` seal and every one of the 199 pinned paths of `class-provenance.tsv`
(canonical-source, guest-route and guest-dependency) — presenting, where the drive tree differs
from the class-form pin, the **class-form bytes (the armed role tree's files)** at exactly those
paths, and print the compiled tree, the presented tree and the replaced count; require zero
remaining mismatches and stop on any. The computed presentation is 0 paths for the armed and
lane-armed trees and 2 for the disabled and lane-disabled trees
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`,
`crates/rum-runtime/src/runtime/profiling/logical/segments.rs`); any other set stops. The 754
ordinary pin-hold (real-byte presentation) is unavailable on the post-install tree and must
never be re-applied; its two-row rejection (`session.rs:254`, `worker.rs:260`) is this
amendment's STOP. After each run, restore the compiled tree and require its tree hash equal to
the build tree. Then execute the 20 drives (four trees × P0267/P0312/P0323/P1470/P1471) and
check the five armed identities per configuration (join rows P0267/P0312/P0323, queue rows
P1470/P1471: exactly one source hit and one class id with `pair=`, `producer_occurrence=`,
`generation=`, `namespace=` and the `SELECTOR_ARMED < SOURCE_HIT < id` order), the inert
disabled cohorts 60/60/60/45/45 (`class_hits=0 queue_hits=0 join_hits=0`), the 62 scheduled
callers present in the frozen drive trees, the ten supplemental and 26 withholding fixtures,
the three peels (19,184 bytes/level + 3,360 intercept, depth 48 = 924,192), and the
private-frame E0603 falsifier; then freeze the `W/control` generator/continuation/twin/
pure-compiler package (the `generator-check.py` interface with positive and one-fault tables,
the two-test resolver control with the generic-receiver AmbiguousDispatch and
concrete-resolution calibrations, the structural/dynamic generator controls, the P2593/P0813
twins, the pure compiler fixture) and run the two ordinary importers' fourteen signed compiler
controls on the fresh real ELFs with their actual rustc argv/stderr, zero schedule credit.
Reconcile 12 canonical + 1 independent + 4 drive, plus the itemized functional CLI and
workspace/producer jobs. Each auxiliary build carries its own source/ELF receipt and zero
campaign labels.

**Stage D (14,400 s), Stage E (3,000 s), Stage F (2,400 s).** As Amendment 5 wrote them,
including every bound, cost and stop: the nine CLI companions and both frozen programs, fixed
stderr and the exact 120-pair, 5,519-byte normalized stdout; the six workspace cases with the
ordered seven skip pairs, nine host groups, current target-by-target comparison, exact
same-binary adjudications and six ledgers; the full time normalization, falsifier and hardware
rules of brief 812 lines 223–270; the six-build subject freeze, the ordinary mode-specific
importers, zero-build-credit controls, spots and named negatives; six fresh C10 runs with both
importers at 0/151 on the build-only control and 1/150 on C10 (55 real keys, one hit, zero
settlements, partial-index status 101); no lane build in the index; and the child epoch
identity, complete payload/report seal, live exit census and continuous charge. Class
admissions, member cells, new catches and schedule credit are zero.

**Charge.** The Amendment 6 attempt charged 2,545.405144187 s of 27,776.523 (A0 82.455/300,
C 1,970.313/7,676.523; D/E/F 0; pre-A0 50.042, close 442.596); unspent **25,231.117856 s**;
returned 29,532.898478 s of the 32,078.303622 s pool; cumulative 36,005.212469 of
65,538.110947. This amendment allocates A0 300 / C 6,500 / D 14,400 / E 3,000 / F 2,400 =
**26,600 s** of that returned pool; **no new top-up**; the remaining 2,932.898478 s stays
unallocated. Charge is continuous from dispatch; every stage box is measured from its own CLOCK
entry; a stage stops at its deadline.

**Stops.** The thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385,
P2387, P2409, P2411, P2416, P2418. Briefs 744, 745 and 746 stay held; CM5 stays unauthorized.
Stop on a pin, sourceset, warning, parser vector, producer, position/identity, visibility,
build/spot, raw receipt, functional or comparison mismatch, a missing bound input, an
unavailable required interference proof, a drive pin preflight mismatch, a first failed drive,
a projected overrun or a stage deadline.

## Director gate (c217, 2026-11-27) — Amendment 7's agy attempt stopped at the Stage A0 deadline; the A0 substance is proven by the post-stop readback; agy's sandbox cannot run the Stage D/E source mounts

Brief 812 Amendment 7 was first dispatched on cdx at 05:14:49 (unit `brief-812-cdx-20261127-051449-85202`) and died at 05:19:41 with Codex's safety-filter refusal (`ERROR: This content was flagged for possible cybersecurity risk`, three log occurrences) after A0 PASS (the relocated A6 seal `b9e200e9…`, 403,636 / 15,243,990,447 B; two `C216 APPEND PASS`) and after the five default-armed carrier drives passed under the class-form presentation (P0267/P0312/P0323/P1470/P1471, `CM742C81 DRIVE VERDICT … fired=true`) while `build-drive-default-disabled` was compiling; zero live changes, no git attempts, the sealed Amendment 6 record untouched (0 modified files). The attempt's 292.672 s wall is booked; its record is at `.temp/director/812-filter-1/` and its log at `.temp/logs/archive/cdx_brief_812.filter-1.log`. Per the 2026-11-13 ruling the brief was not reworded; per the 2026-11-23 chain (grk out until ≈2026-11-29, mst after 2026-12-01) it was re-dispatched fresh on agy `gemini-4-carbon-high` at 05:24:53 as unit `brief-812-agy-20261127-052453-92226` (task `b79fsvhc4`, agy 1.5.17, conversation `31b19479-6d6c-4a7d-bd2c-0a6ee4a7edbb`; fence `proceed-in-sandbox`, `write_file(/home/remi/dev/rum)`; log `.temp/logs/agy_brief_812.log`).

The agy attempt returned rc 0 at 06:07 (2,564 s wall, one turn, 855,328 tokens, peak 1,034,289,152 B, oom 0/0) with verdict **STOP** at the Stage A0 deadline: `CLOCK stage=A` 299.168249590 s of 300. `a0-seal` PASS in 96.975 s (the relocated Amendment 6 record re-authenticated: 403,636 payloads / 15,243,990,447 B `b9e200e9…`, moved report `b0be6df3…` and PROGRESS `dbba8210…`, summary `2cc643e7…`, finding `30a8e13f…`, charge `b531f7ff…`). `a0-bindings` try 1 exited 1 after 30.554 s on the attempt's own pre-F13 lane-prefix constant (`('lane tree', 'real')`); corrected to the A6 post-F13 lane trees (`89d2c2af…`/`6055ae6c…`/`4b62cfc2…`), try 2 was given 19.785 s and timed out (exit 124) while the 14-tree sequential hashing needs ~33.6 s. The read-only post-stop readback ran the unmodified a0.py to exit 0 in 33.584 s, confirming every 806e/child/F13/design pin, the reused Stage C cells and both `C216 APPEND PASS` rows (`TODO.md` `a704d73a…`, this brief `5b4d0749…`), with the live entry census `fb5f95f1…` at 5,903 paths. **The attempt also established that `bwrap --ro-bind` fails with `Creating new namespace failed: Function not implemented` under agy's seccomp (`Seccomp=2`, `NoNewPrivs=1`); the six reused disabled/armed A6 oracle ELFs** (`1cf0b570…`, `b2f2c88c…`, `f7282d0a…`, `1d4b22a8…`, `36416adc…`, `bd8e73b3…`) **embed `.temp/cm4-campaign-epoch/amendment-6/canonical/source` (sealed as roles/real `b0bdcee9…`), so Stages D and E cannot run on agy as written**; the same source mounts are available to cdx (attempts 1–6) and on the director's host. The record: report `9b9ce967…` (645,153 B; 64-payload seal `08fdf555…` / 2,512,516 B, all re-hashed equal, final readback mismatches=0, live differences 0; `report.py check` OK 1,068 citations), PROGRESS `ea2337f7…`, finding `b595efb2…`, payloads `36a64d48…`, charge 2,504.718108962 s (A 299.168249590; pre-A0 1,319.156675152; close 886.393184220); the exit audit: ten SOURCE EXIT PASS trees unchanged, `OPERATION EXIT PASS completed=6 status_counts={1: 2, 124: 1, 0: 3}`, `LIVE EXIT PASS paths=5903 added=0 removed=0 changed=0 entry=exit=fb5f95f1…`, memory peak 1,034,289,152 B oom 0/0, `CONFINEMENT EXIT PASS attributable_live_changes=0 worker_version_control_invocations=0 network=0 member_cells=0 new_catches=0 class_admissions=0 schedule_credit=0` (the two git-fence entries are agy's own `merge-base` probes); the sealed Amendment 6 record 0 files modified; `git status` clean; zero campaign credit. Charge: the cdx 292.672 s + the agy 2,504.718108962 s = 2,797.390551962 s of the 26,600 s allocation; unspent 23,802.609448038 s; returned pool 26,735.507926044 s; cumulative 38,802.603020956 of 65,538.110947. **No new director defect:** the A0 box is unchanged and was ample for cdx (A0 112 s); the overrun came from the attempt's own pre-F13 constant consuming 30.554 s, and the substance is proven by the 33.584 s readback; the D/E constraint is an execution-environment fact (mount namespaces), not a brief fault and not a scope cut. Records: THIS gate section; the report/PROGRESS moved to `.temp/director/812-stop-7/` (re-hashed `9b9ce967…` / `ea2337f7…`), the log archived `.temp/logs/archive/agy_brief_812.stop-7.log` (`34073550…`) with its `.git-attempts` (`d0882382…`); the amendment-7 workdir stays in place as the stop-7 sealed record (1.1 MB), so a resumed attempt must use a fresh workdir. **The resume route is the user's decision** — director execution under the 2026-11-24 precedent, a wait for grk (≈2026-11-29), or another route — and no amendment is issued until then.

## Director gate (c218, 2026-11-27) — the route decision: the user reset cdx's usage window and directs the remaining Amendment 7 work to cdx; Amendment 8 resumes Stages C–F in a fresh workdir

After gate c217 (ruling 869) left the resume route to the user, the user reset cdx's usage window and directed the work back to cdx (「等一下，我发现cdx还有个重置点，刚才重置了，所以现在cdx能用了，所以还是用cdx吧。」). The 05:19:41 safety-filter refusal stands recorded; the brief is not reworded (2026-11-13 ruling), and Amendment 7's scope, bars, materials and stop rules stand unchanged. The agy stop-7 record (`.temp/cm4-campaign-epoch/amendment-7/`, its `live-exit-stat.json` `fb5f95f1…` at 5,903 paths) and the cdx filter-1 record (`.temp/director/812-filter-1/`, including the reference `drives.py`/`drive_build.py`/`scene.py`/`launch.py`/`advance.py`/`payload.py`/`carrier_sequence.py` whose five default-armed drives passed at 05:19) stay read-only.

## Amendment 8 — the same Amendment 7 work resumed on cdx (implementation continuation)

**Scope.** Execute Amendment 7's Stages C–F exactly as written there, from the accepted Amendment 6 products, in the fresh workdir `.temp/cm4-campaign-epoch/amendment-8/` (`W` below). No source edit; every build and measurement lands under `W`; the sealed parents and the 754/797/806e records stay read-only. The twelve canonical labels, the three spot identities, the two 205-case Rust selector runs, the `default-disabled` diagnostic (`a76545f3…`) and the `default-armed` drive build (`e539fb09…` on `d73aa794…`) are reused by hash identity — no re-run, no rebuild; their receipts are re-read from the Amendment 6 seal. The five default-armed drives that passed in the filter-1 attempt are re-run in this attempt: all 20 drives execute here, and no attempt-owned result is reused.

**A0 (300 s).** Close by hash identity against the Amendment 6 record and the c217 readback: the seal `b9e200e9…` (403,636 payloads / 15,243,990,447 B), report `b0be6df3…`, PROGRESS `dbba8210…`, summary `2cc643e7…`, finding `30a8e13f…`, charge `b531f7ff…`; the moved report/PROGRESS at `.temp/director/812-stop-6/`; the four 806e parents, the three children, the five native F13 cells, the observer manifest and the retained auxiliaries; and the live census against `.temp/cm4-campaign-epoch/amendment-7/live-exit-stat.json` (`fb5f95f1…`, 5,903 paths), permitting only `TODO.md` (prefix `a704d73a…`, 1,138,712 B) and this brief (prefix `5b4d0749…`, 99,741 B) to differ, each verified as a strict append and printed as `C217 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…`; any other difference stops. The live entry census is taken after this gate's commit.

**Stage C (6,500 s).** As Amendment 7 wrote it. The drive's source presentation is the loader's authority: on each of the four drive trees verify the program-table seal and all 199 `class-provenance.tsv` paths (canonical-source, guest-route, guest-dependency) against the loader's own predicate set, present the armed role's class-form bytes at exactly the differing paths — 0 for the armed and lane-armed trees, 2 for the disabled and lane-disabled trees (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`, `crates/rum-runtime/src/runtime/profiling/logical/segments.rs`) — print the compiled tree, presented tree and replaced count, stop on any other set or any remaining mismatch, and restore and re-hash the compiled tree after each run. Then the 20 drives (four trees × P0267/P0312/P0323/P1470/P1471) with the five armed identities per configuration (join rows P0267/P0312/P0323, queue rows P1470/P1471; exactly one source hit and one class id with `pair=`, `producer_occurrence=`, `generation=`, `namespace=` and the `SELECTOR_ARMED < SOURCE_HIT < id` order), the inert disabled cohorts 60/60/60/45/45 (`class_hits=0 queue_hits=0 join_hits=0`), the 62 scheduled callers in the frozen drive trees, the ten supplemental and 26 withholding fixtures, the three peels (19,184 bytes/level + 3,360 intercept, depth 48 = 924,192), the private-frame E0603 falsifier, the `W/control` generator/continuation/twin/pure-compiler package, the two ordinary importers' fourteen signed compiler controls on the fresh real ELFs with their actual rustc argv/stderr and zero schedule credit, and the reconciliation of 12 canonical + 1 independent + 4 drive plus the itemized functional CLI and workspace/producer jobs. Each auxiliary build carries its own source/ELF receipt and zero campaign labels. The 754 ordinary pin-hold is never re-applied.

**Stage D (14,400 s), Stage E (3,000 s), Stage F (2,400 s).** As Amendment 5 wrote them (this brief lines 223–301), including every bound, cost and stop: the nine CLI companions and both frozen programs with the exact 120-pair, 5,519-byte normalized stdout; the six workspace cases with the ordered seven skip pairs, the nine host groups, the current target-by-target comparison, the exact same-binary adjudications and the six ledgers; the full time normalization, falsifier and hardware rules of lines 260–270; the six-build subject freeze, the ordinary mode-specific importers, the zero-build-credit controls, the spots and named negatives; the six fresh C10 runs (0/151 on the build-only control, 1/150 on C10; 55 real keys, one hit, zero settlements, partial-index exit 101); no lane build in the index; and the child epoch identity, the complete payload/report seal, the live exit census and the continuous charge.

**Charge.** Amendment 7's 26,600 s allocation spent 2,797.390551962 s in the two dead attempts; 23,802.609448038 s returned, and with the 2,932.898478 s that stayed unallocated at c216 the returned pool is **26,735.507926044 s**. This amendment allocates A0 300 / C 6,500 / D 14,400 / E 3,000 / F 2,400 = **26,600 s** of it; **no new top-up**; 135.507926044 s stays unallocated. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** Unchanged from Amendment 7: the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, warning, parser vector, producer, position/identity, visibility, build/spot, raw receipt, functional or comparison mismatch, a missing bound input, an unavailable required interference proof, a drive pin preflight mismatch, a first failed drive, a projected overrun or a stage deadline.
