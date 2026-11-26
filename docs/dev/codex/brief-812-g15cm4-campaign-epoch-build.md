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
