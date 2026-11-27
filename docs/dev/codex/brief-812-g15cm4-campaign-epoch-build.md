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

## Director gate (c219, 2026-11-27) — Amendment 8's Stage C STOP at the caller audit is accepted; A0 closed by hash identity, all 20 drives passed under the class-form presentation, and the exact-stamp replay is retired in favour of the schedule-data rebase (defect #50, mine)

Amendment 8 was dispatched 07:07:09 as unit `brief-812-cdx-20261127-070709-125205` (task `bdse7ey0p`, log `.temp/logs/cdx_brief_812.log`; Codex CLI 0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a21a1c-e0b4-7453-b031-cce5f51f6545`) and returned rc 0 at 07:24:59, 211,453 tokens, peak 3,185,291,264 B, oom 0/0, verdict **STOP**. **A0 PASS** (hash identity against the Amendment 6 record: the seal `b9e200e9…` 403,636 / 15,243,990,447 B, the four 806e parents, the three four-edit children, the three final roles, the five native F13 cells, the observer manifest `50cc4a95…`, the reused twelve labels, lane trees, both 205-case selectors, the diagnostic `a76545f3…` and the `default-armed` drive build `e539fb09…` on `d73aa794…`; the c217 readback `fb5f95f1…` at 64 payloads / 2,512,516 B; and the two strict appends printed `C217 APPEND PASS` — `TODO.md` `c7ceb34c…` 1,144,386 B and this brief `77c7cc8b…` 111,479 B — with the live entry census `df312ad6…` at 5,903 paths). **Stage C PASS through the drives:** all four carrier cohorts (`CARRIER PASS` trees `d73aa794…`/`2271c3ee…`/`5cf8f5ce…`/`6eafd27a…`, binaries `e539fb09…`/`fcb29953…`/`fd7802f7…`/`f0358d08…`), 20/20 drive preflights (`DRIVE PREFLIGHT PASS … class_paths=199 replaced=0` for armed and lane-armed with `compiled==presented`; `replaced=2` — `eval_dispatch/mod.rs` and `profiling/logical/segments.rs` — for disabled and lane-disabled; `remaining_mismatches=0`), all 20 drive verdicts (the ten armed drives `armed=true fired=true` with exactly one `CM740_SELECTOR_ARMED`, one `CM740_SOURCE_HIT` and one `CM742_JOIN_HIT_ID`/`CM742_QUEUE_HIT_ID` at `pair=1`, `producer_occurrence=0`, generations 2/2/2/3/4, in the `SELECTOR_ARMED < SOURCE_HIT < id` order; the ten disabled drives inert at 60/60/60/45/45 plans with `class_hits=0 queue_hits=0 join_hits=0`). **The STOP** at the third declared caller of the default-armed caller audit: P0214's 754 witness pins `TraceFilterSuspensionFrame::resume` to `61ee27cb…`, the drive tree carries `7ef923b9…`, and the sealed class form's `let _class_row = crate::cm740_probe::class_lease::begin_row(&self);` at line 74 is the whole difference; Stages D–F unentered, zero campaign credit, zero member cells, zero schedule credit. **The record I verified:** seal `7ddd4988…` (71,604 payloads / 2,643,088,516 B, every payload re-hashed equal, 0 missing 0 bad), payload manifest `0e93cd11…` (16,046,571 B), report `9786a84e…` (1,051,329 B; `report.py check 812 --against-git` OK 1,960 citations), PROGRESS `cef96396…` (3,600 B), summary `d5d28d0e…` (582 B), finding `781f733d…` (86,729 B), charge `9657a283…` (44,690 B), exit audit `62d5dc6a…` (3,643 B), envelope 14 rows `ce7484e9…`; `LIVE EXIT PASS paths=5903 added=0 removed=0 changed=0 entry=exit=df312ad6…`; zero git attempts, network, member cells, new catches, class admissions or schedule credit; `CLOCK` A 57.518699715/300, C 527.109938125/6,500; memory peak 3,185,291,264 B oom 0/0. **My own reproduction:** the 747 index gives `7ef923b9…` (drive, lines 69–79) against `61ee27cb…` (real role, lines 57–66); the three witness paths are declared `overlays.tsv` keys (preimages `9f6f2db8…`, `f7bb2520…`, `00e72695…` equal to the real files; postimages `de09398e…`, `038a0d88…`, `ff6a1273…` equal to the role files); and I classified all 288 witnesses independently — **244 EXACT** (the stamp is the drive function's token hash; the 92 with a 749 witness path are all at that path), **40 ORDINARY** (the stamp is the ordinary function's token hash in the real role, the drive tree carries the declared render: P0214/P0216 ×4 unique, and eight sibling rows P2379/P2381/P2385/P2387/P2409/P2411/P2416/P2418 ×4 whose stamp names two token-identical ordinary siblings, each with its own same-owner same-name drive counterpart), **4 RENDER** (P1070 ×4: a 754-era render form present in neither tree, resolved uniquely by its `EDGE JOIN cut=52 … driven_target=select_guarded_replace_part_targets_suspended_cm740_driven` to `SelectGuardedReplacePartTargetsOwnedSuspensionFrame::resume`), zero unknowns. **Defect #50 (mine):** Amendment 7/8 replayed the 754 stamps — a 754-era render output — on the D.2-rendered drive trees; the stamp survives for 244, equals the ordinary body for 40, and for P1070 is a 754-era render form no current tree reproduces (the 754-era renderer died with the c154 clean-temp); the exact-stamp bar was unsatisfiable by construction and the brief's own identity rule fired — the STOP is the rule working, no worker error. Charge: 1,062.556287 s of 26,600 (A 57.518699715, C 527.109938125; pre-A0 84.119820752; the close interval 393.807819945 charged); returned 25,672.951639438 s, cumulative 39,865.159308 of 65,538.110947. Records: THIS gate section; the A8 report/PROGRESS move to `.temp/director/812-stop-8/` (re-hashed `9786a84e…` / `cef96396…`), the log archives `.temp/logs/archive/cdx_brief_812.stop-8.log`; the amendment-8 workdir stays as the stop-8 sealed record.

## Amendment 9 — the caller-identity rebase; Stages C–F complete (implementation continuation)

**Scope.** Execute Amendment 7's Stages C–F from the accepted Amendment 6 products, in the fresh workdir `.temp/cm4-campaign-epoch/amendment-9/` (`W` below). No source edit; every build and measurement lands under `W`; the sealed parents and the 754/797/806e/812-8 records stay read-only. Reused by hash identity, no re-run: the twelve canonical labels, the three spot identities, the two 205-case Rust selector runs, the `default-disabled` diagnostic (`a76545f3…`), the `default-armed` drive build (`e539fb09…` on `d73aa794…`), and — new at this amendment — the three further drive builds of the Amendment 8 record (`default-disabled` `2271c3ee…` / `fcb29953…`, `lane-armed` `5cf8f5ce…` / `fd7802f7…`, `lane-disabled` `6eafd27a…` / `f0358d08…`), each re-authenticated through the A8 seal before use. The 20 drives execute in this attempt on those sealed trees (no attempt-owned execution is reused; every drive log is this attempt's).

**A0 (300 s).** Close by hash identity against the Amendment 8 record: the seal `7ddd4988…` (71,604 payloads / 2,643,088,516 B), the payload manifest `0e93cd11…`, report `9786a84e…`, PROGRESS `cef96396…`, summary `d5d28d0e…`, finding `781f733d…`, charge `9657a283…`, exit audit `62d5dc6a…`, envelope `ce7484e9…` (14 rows); the four drive builds above against their `AUXILIARY BUILD PASS`/`CARRIER PASS` receipts; and the live census against `.temp/cm4-campaign-epoch/amendment-8/live-exit-stat.json` (`df312ad6…`, 5,903 paths), permitting only `TODO.md` (prefix `c7ceb34c…`, 1,144,386 B) and this brief (prefix `77c7cc8b…`, 111,479 B) to differ, each verified as a strict append and printed `C218 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…`; any other difference stops.

**Stage C (6,500 s).** As Amendment 7 wrote it — the drive pin preflight on the four sealed trees (the program-table seal and 199 `class-provenance.tsv` paths, `replaced=0` armed/lane-armed, `replaced=2` disabled/lane-disabled, restore and re-hash after every run), the 20 drives with the five armed identities per configuration and the inert disabled cohorts, the ten supplemental and 26 withholding fixtures, the three peels (19,184 bytes/level + 3,360 intercept, depth 48 = 924,192), the private-frame E0603 falsifier, the `W/control` generator/continuation/twin/pure-compiler package, the two ordinary importers' fourteen signed compiler controls on the fresh real ELFs with their actual rustc argv/stderr and zero schedule credit, and the reconciliation of 12 canonical + 1 independent + 4 drive plus the itemized functional CLI and workspace/producer jobs — with the caller audit replaced as follows.

**The caller-identity rebase.** The 754 `SCHEDULED CALLER PRESENT` token stamps were computed on the 754-era drive render; the frozen drive trees are the D.2 class-form render of the same ordinary source, and the exact-stamp replay is retired. For each of the 288 witnesses (72 rows × default/lane × armed/disabled), bind the caller in the frozen drive tree and print one row, with exactly these relations:

- **EXACT** — the 754 stamp resolves to exactly one function with the row's name in the drive tree; where the 749 record carries a witness path (`executed=`), the function must be at that path.
- **ORDINARY** — the stamp does not resolve in the drive tree but resolves in the real role to the function(s) with the row's name at the witness path; each must have exactly one same-owner same-name counterpart in the drive tree at the same path; the path must be a declared `overlays.tsv` key whose preimage equals the real file and whose postimage equals the drive file; the caller is the counterpart set — one member for a unique stamp, both members of a token-identical sibling pair otherwise.
- **RENDER** — the stamp resolves in neither tree; permitted only for the ten supplemental C103 rows, bound by the row's `EDGE JOIN` record (`incoming=` path and `driven_target=` call), which must resolve to exactly one function at that path (the call named by the target, or its plain stem, with both branches of the driven if/else counted once).

Every row prints `SCHEDULED CALLER BOUND tree=… row=… relation=EXACT|ORDINARY|RENDER path=… owner=… function=… body=… calls=…`, the per-tree totals must read `scheduled_rows=62 supplemental_edge_fix_rows=10 bound=72 missing=0 unknown=0`, and a zero-candidate row, a multi-candidate row outside the declared sibling pairs, a stamp with no same-owner same-name counterpart, a bound path outside the overlay declaration, or any relation outside the three classes stops. **The 26 withholding fixtures** are rebased the same way — `(role, cut, caller)` from the 754 lines, the cut's incoming edge from the 749 `EDGE JOIN` where recorded, the withheld copy confined to `W`, the row's program re-run, and the expected failure required to fire (`runtime_catches=0`, the retained guard sites reported); a fixture that passes stops. Binding a row here never waives its Q25 STOP status (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418 stay declared STOPs).

**Stage D (14,400 s), Stage E (3,000 s), Stage F (2,400 s).** As Amendment 5 wrote them (this brief lines 223–301), including every bound, cost and stop: the nine CLI companions and both frozen programs with the exact 120-pair, 5,519-byte normalized stdout; the six workspace cases with the ordered seven skip pairs, the nine host groups, the current target-by-target comparison, the exact same-binary adjudications and the six ledgers; the full time normalization, falsifier and hardware rules of lines 260–270; the six-build subject freeze, the ordinary mode-specific importers, the zero-build-credit controls, the spots and named negatives; the six fresh C10 runs (0/151 on the build-only control, 1/150 on C10; 55 real keys, one hit, zero settlements, partial-index exit 101); no lane build in the index; and the child epoch identity, the complete payload/report seal, the live exit census and the continuous charge.

**Charge.** Amendment 8 charged 1,062.556287 s of its 26,600 s allocation (A 57.518699715, C 527.109938125; pre-A0 84.119820752, the close interval 393.807819945); 25,537.443713 s returned, and with the 135.507926044 s left unallocated at c218 the pool is **25,672.951639438 s**. This amendment allocates A0 300 / C 6,500 / D 14,400 / E 3,000 / F 2,400 = **26,600 s** = the pool + an explicit **927.048360562 s top-up** (the user's standing rule, 2026-11-19); the authorized total becomes **66,465.159308 s**; the cumulative before dispatch is **39,865.159308 of 66,465.159308**. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** Unchanged from Amendment 8: the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, warning, parser vector, producer, position/identity, visibility, build/spot, raw receipt, functional or comparison mismatch, a missing bound input, an unavailable required interference proof, a drive pin preflight mismatch, a first failed drive, a projected overrun or a stage deadline.

## Director gate (c220, 2026-11-27) — Amendment 9's Stage C STOP at P2611 is accepted; A0 closed by hash identity, all 20 carrier drives passed, and the caller-identity rebase bound 54 rows before my flat overlay clause blocked three legitimate EXACT rows (defect #51, mine)

Amendment 9 was dispatched 07:41:51 as unit `brief-812-cdx-20261127-074151-148399` (task `b6knhlaux`, log `.temp/logs/cdx_brief_812.log`; Codex CLI 0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a21a3c-a4df-75c3-a31e-1420b5e2453b`) and returned rc 0 at 07:53:10, 159,377 tokens, peak 717,185,024 B, oom 0/0, verdict **STOP**. **A0 PASS** (hash identity against the Amendment 8 record: the seal `7ddd4988…` 71,604 / 2,643,088,516 B, the moved report/PROGRESS, envelope/summary/finding/charge; the four drive builds `e539fb09…`/`fcb29953…`/`fd7802f7…`/`f0358d08…` on `d73aa794…`/`2271c3ee…`/`5cf8f5ce…`/`6eafd27a…`; the twelve labels, the three spot identities, both 205-case selectors and the `a76545f3…` diagnostic; and the two strict appends printed `C218 APPEND PASS` — `TODO.md` `c643a373…` 1,147,989 B and this brief `96c10975…` 124,321 B — at 5,903 live paths `07b6ae3e…`). **Stage C PASS through the drives:** 20/20 drive preflights, 20/20 drives on the four sealed trees (ten armed votes with exactly one `CM740_SELECTOR_ARMED`, one `CM740_SOURCE_HIT` and one join/queue id at `pair=1`, `producer_occurrence=0`, generation 2/2/2/3/4, in the `SELECTOR_ARMED < SOURCE_HIT < id` order; ten disabled cohorts inert at 60/60/60/45/45, 540 plan passes), 20 restorations, zero credit. **The caller audit bound 54 rows** (44 EXACT + 10 ORDINARY; every printed body matches my c219 747-derived classification, and the ten ORDINARY rows print the declared drive-side counterparts of their 754 ordinary stamps) before the STOP at the first shared-file caller: **P2611** `observe_segment` at `crates/rum-stdlib/tests/suspension_oracle/adapter_facade.rs`, body `6d162986…`, calls=6, the 749 witness line 26 inside the function at lines 11–33, the file byte-identical in the real role and the drive trees (`2099dfa0…`), absent from both overlay serializations and the R4 map — so Amendment 9's flat stop phrase "a bound path outside the overlay declaration" fired on a valid EXACT binding; stages D–F unentered. **The record I verified:** seal `0df7c9a1…` (70,533 payloads / 1,554,767,115 B, all re-hashed equal in 2.1 s, 0 missing 0 bad), payload manifest `5ff81fce…` (15,823,925 B), report `7347863e…` (1,227,186 B; `report.py check 812 --against-git` OK 2,102 citations), PROGRESS `8759e2b6…` (3,575 B), summary `b65ecdca…` (653 B), finding `88bf2ee6…` (151,185 B), charge `8ec3881e…` (66,270 B), exit audit `b9a11c75…` (4,066 B), envelope 14 rows `18cd9505…`; `LIVE EXIT PASS paths=5903 added=0 removed=0 changed=0 entry=exit=07b6ae3e…`, and my own walk reproduces the 5,903-path set exactly (0 missing, 0 extra, 0 value differences); `git status` clean at `f31872cf`; zero `.git-attempts`, zero live changes, network, member cells, new catches, class admissions or schedule credit; `CLOCK` A 13.773259329/300, C 259.474732430/6,500 (pre-A0 98.658461422; the close interval 301.108348333 charged); memory peak 717,185,024 B oom 0/0. **My own reproduction:** the rows with non-overlay paths are exactly P2611, P2612 (`observe_segment`, `adapter_facade.rs`, real==drive `2099dfa0…`) and G15B (`resume`, `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs`, real==drive `20e1f69d…`, witness line 174, body `655fb3dd…`), all three unique-stamp EXACT bindings whose witness lies inside the bound function. **Defect #51 (mine):** Amendment 9's stop list flattened the ORDINARY-only overlay requirement into a global condition; the overlay declaration is the rebase leg for stamps that do not resolve in the drive tree, and a resolved stamp whose function is at the 749 witness path needs no overlay — the STOP is the rule working, no worker error; fourth of the #48–#50 class. Charge: 673.014808771 s of 26,600 (A 13.773259329, C 259.474732430; pre-A0 98.658461422; the close interval 301.108348333 charged); returned 25,926.985191229 s, cumulative 40,538.174116333 of 66,465.159308. Records: THIS gate section; the A9 report/PROGRESS move to `.temp/director/812-stop-9/` (re-hashed `7347863e…` / `8759e2b6…`), the log archives `.temp/logs/archive/cdx_brief_812.stop-9.log`; the amendment-9 workdir stays as the stop-9 sealed record.

## Amendment 10 — the EXACT-relation scoping fix; Stages C–F complete (implementation continuation)

**Scope.** Execute Amendment 9's Stages C–F from the accepted Amendment 6 products in the fresh workdir `.temp/cm4-campaign-epoch/amendment-10/` (`W` below). No source edit; every build and measurement lands under `W`; the sealed parents and the 754/797/806e/812-9 records stay read-only. Reused by hash identity, no re-run: the twelve canonical labels, the three spot identities, the two 205-case Rust selector runs, the `default-disabled` diagnostic (`a76545f3…`), and the four drive builds of the Amendment 8 record (`default-armed` `e539fb09…` / `d73aa794…`, `default-disabled` `fcb29953…` / `2271c3ee…`, `lane-armed` `fd7802f7…` / `5cf8f5ce…`, `lane-disabled` `f0358d08…` / `6eafd27a…`), each re-authenticated through the A9 seal before use. The 20 drives execute in this attempt on those sealed trees (no attempt-owned execution is reused; every drive log is this attempt's).

**A0 (300 s).** Close by hash identity against the Amendment 9 record: the seal `0df7c9a1…` (70,533 payloads / 1,554,767,115 B), the payload manifest `5ff81fce…`, report `7347863e…`, PROGRESS `8759e2b6…`, summary `b65ecdca…`, finding `88bf2ee6…`, charge `8ec3881e…`, exit audit `b9a11c75…`, envelope `18cd9505…` (14 rows); the four drive builds above against their `CARRIER PASS` receipts; and the live census against `.temp/cm4-campaign-epoch/amendment-9/live-exit-stat.json` (`07b6ae3e…`, 5,903 paths), permitting only `TODO.md` (prefix `c643a373…`, 1,147,989 B) and this brief (prefix `96c10975…`, 124,321 B) to differ, each verified as a strict append and printed `C219 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…`; any other difference stops.

**Stage C (6,500 s).** As Amendment 9 wrote it — the drive pin preflight on the four sealed trees (the program-table seal and 199 `class-provenance.tsv` paths, `replaced=0` armed/lane-armed, `replaced=2` disabled/lane-disabled, restore and re-hash after every run), the 20 drives with the five armed identities per configuration and the inert disabled cohorts, the ten supplemental and 26 withholding fixtures, the three peels (19,184 bytes/level + 3,360 intercept, depth 48 = 924,192), the private-frame E0603 falsifier, the `W/control` generator/continuation/twin/pure-compiler package, the two ordinary importers' fourteen signed compiler controls on the fresh real ELFs with their actual rustc argv/stderr and zero schedule credit, and the reconciliation of 12 canonical + 1 independent + 4 drive plus the itemized functional CLI and workspace/producer jobs — with the caller-identity rule as follows.

**The caller-identity rebase.** The 754 `SCHEDULED CALLER PRESENT` token stamps were computed on the 754-era drive render; the frozen drive trees are the D.2 class-form render of the same ordinary source, and the exact-stamp replay is retired. For each of the 288 witnesses (72 rows × default/lane × armed/disabled), bind the caller in the frozen drive tree and print one row, with exactly these relations:

- **EXACT** — the 754 stamp resolves to exactly one function with the row's name in the drive tree; where the 749 record carries a witness path (`executed=`), the function must be at that path. No overlay requirement applies: the file may be role-specific or identical across roles, and the row prints `path_in_overlays=true|false` with the real and drive file hashes.
- **ORDINARY** — the stamp does not resolve in the drive tree but resolves in the real role to the function(s) with the row's name at the witness path; each must have exactly one same-owner same-name counterpart in the drive tree at the same path; the path must be a declared `overlays.tsv` key whose preimage equals the real file and whose postimage equals the drive file; the caller is the counterpart set — one member for a unique stamp, both members of a token-identical sibling pair otherwise.
- **RENDER** — the stamp resolves in neither tree; permitted only for the ten supplemental C103 rows, bound by the row's `EDGE JOIN` record (`incoming=` path and `driven_target=` call), which must resolve to exactly one function at that path (the call named by the target, or its plain stem, with both branches of the driven if/else counted once).

Every row prints `SCHEDULED CALLER BOUND tree=… row=… relation=EXACT|ORDINARY|RENDER path=… owner=… function=… body=… calls=…`, the per-tree totals must read `scheduled_rows=62 supplemental_edge_fix_rows=10 bound=72 missing=0 unknown=0`, and the stops are scoped: a zero-candidate row; a multi-candidate row outside the declared sibling pairs; a stamp with no same-owner same-name counterpart; **an ORDINARY row whose path is not a declared `overlays.tsv` key or whose preimage/postimage cell fails**; a RENDER row whose `EDGE JOIN` record does not resolve to exactly one function at its `incoming=` path; or any relation outside the three classes. A bound path outside the overlay declaration stops only an ORDINARY row — an EXACT or RENDER binding needs no overlay leg. The three shared-file rows named by the c220 gate are pinned acceptance cells: P2611 and P2612 (`observe_segment`, `crates/rum-stdlib/tests/suspension_oracle/adapter_facade.rs`, real==drive `2099dfa0…`, witness line 26 inside lines 11–33, body `6d162986…`, calls=6) and G15B (`resume`, `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs`, real==drive `20e1f69d…`, witness line 174, body `655fb3dd…`). **The 26 withholding fixtures** are rebased the same way — `(role, cut, caller)` from the 754 lines, the cut's incoming edge from the 749 `EDGE JOIN` where recorded, the withheld copy confined to `W`, the row's program re-run, and the expected failure required to fire (`runtime_catches=0`, the retained guard sites reported); a fixture that passes stops. Binding a row here never waives its Q25 STOP status (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418 stay declared STOPs).

**Stage D (14,400 s), Stage E (3,000 s), Stage F (2,400 s).** As Amendment 5 wrote them (this brief lines 223–301), including every bound, cost and stop: the nine CLI companions and both frozen programs with the exact 120-pair, 5,519-byte normalized stdout; the six workspace cases with the ordered seven skip pairs, the nine host groups, the current target-by-target comparison, the exact same-binary adjudications and the six ledgers; the full time normalization, falsifier and hardware rules of lines 260–270; the six-build subject freeze, the ordinary mode-specific importers, the zero-build-credit controls, the spots and named negatives; the six fresh C10 runs (0/151 on the build-only control, 1/150 on C10; 55 real keys, one hit, zero settlements, partial-index exit 101); no lane build in the index; and the child epoch identity, the complete payload/report seal, the live exit census and the continuous charge.

**Charge.** Amendment 9 charged 673.014808771 s of its 26,600 s (A 13.773259329, C 259.474732430; pre-A0 98.658461422, the close interval 301.108348333); 25,926.985191229 s returned, and with nothing left unallocated at c219 the pool is **25,926.985191229 s**. This amendment allocates A0 300 / C 6,500 / D 14,400 / E 3,000 / F 2,400 = **26,600 s** = the pool + an explicit **673.014808771 s top-up** (the user's standing rule, 2026-11-19); the authorized total becomes **67,138.174116771 s**; the cumulative before dispatch is **40,538.174116333 of 67,138.174116771**. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** Unchanged from Amendment 9: the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, warning, parser vector, producer, position/identity, visibility, build/spot, raw receipt, functional or comparison mismatch, a missing bound input, an unavailable required interference proof, a drive pin preflight mismatch, a first failed drive, a projected overrun or a stage deadline.

## Director gate (c221, 2026-11-27) — Amendment 10's Stage C STOP at P1070 (cut 52) is accepted; A0 closed by hash identity, all 20 carrier drives passed, and the caller rebase bound 68 rows including the three c220 pins before my RENDER rule counted two `resume` methods (defect #52, mine)

Amendment 10 was dispatched 07:58:36 as unit `brief-812-cdx-20261127-075836-157995` (task `blr9ap2n7`, log `.temp/logs/cdx_brief_812.log`; Codex CLI 0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a21a4b-fb43-7dc0-a5b3-13fbd2c09eda`) and returned rc 0 at 08:07:44, 155,034 tokens, peak 605,982,720 B, oom 0/0, verdict **STOP**. **A0 PASS** (the Amendment 9 record by hash identity: the seal `0df7c9a1…` 70,533 / 1,554,767,115 B with both moved files re-verified at `.temp/director/812-stop-9/`, the four drive builds `e539fb09…`/`fcb29953…`/`fd7802f7…`/`f0358d08…` on `d73aa794…`/`2271c3ee…`/`5cf8f5ce…`/`6eafd27a…`, the twelve labels, three spot identities, both 205-case selectors and the `a76545f3…` diagnostic, and the two strict appends printed `C219 APPEND PASS` — `TODO.md` `17043df5…` 1,152,127 B and this brief `4f7ee6ea…` 137,183 B — at 5,903 live paths `07b6ae3e…`). **Stage C PASS through the drives:** 20/20 drive preflights (`class_paths=199`; `replaced=0` armed/lane-armed, `replaced=2` disabled/lane-disabled, `remaining_mismatches=0`), 20/20 drives on the four sealed trees (ten armed votes with exactly one `CM740_SELECTOR_ARMED`, one `CM740_SOURCE_HIT` and one join/queue id — P0267/P0312/P0323 `CM742_JOIN_HIT_ID`, P1470/P1471 `CM742_QUEUE_HIT_ID` — at `pair=1`, `producer_occurrence=0`, generation 2/2/2/3/4, in the `SELECTOR_ARMED < SOURCE_HIT < id` order; ten disabled cohorts inert at 60/60/60/45/45, 540 plan passes), 20 restorations, zero credit. **The caller audit bound 68 rows** (58 EXACT + 10 ORDINARY; every EXACT body equals my c219 754-derived stamp; the ORDINARY rows print their declared drive-side counterparts and the overlay preimage/postimage cells), and the three acceptance cells pinned at c220 **passed exactly** — P2611/P2612 `observe_segment` at `adapter_facade.rs` (`path_in_overlays=false`, real==drive `2099dfa0…`, body `6d162986…`, witness line 26 inside 11–33, calls=6) and G15B `resume` at `definition_lhs_preparation/resume_frames.rs` (`path_in_overlays=false`, real==drive `20e1f69d…`, body `655fb3dd…`, witness line 174 inside 165–176, calls=4) — before the STOP at **P1070, cut 52**: the EDGE JOIN record names `incoming=…guarded/resume_frames.rs:110` and `driven_target=select_guarded_replace_part_targets_suspended_cm740_driven`; the file (`ff6a1273…` in all four drive trees) holds two `resume` methods — the one at 100–121 contains the incoming line 110 and the 749 witness line 111 and calls the target at 112 and the plain stem at 117 (the two branches of the driven if/else); a second `resume` at 220–233 calls the plain stem at 230 for a different frame — so the Amendment 10 rule ("exactly one function at that path, the target or its plain stem") counted two owners; stages D–F unentered. **The record I verified:** seal `4e5f2ba7…` (70,547 payloads / 1,555,056,551 B, all re-hashed equal in 2.06 s, 0 missing 0 bad), payload manifest `c8ebb0ce…` (15,896,674 B), report `c538c9f6…` (1,302,283 B; `report.py check 812` OK 2,157 citations), PROGRESS `19f3474b…` (3,468 B), summary `b832965f…`, finding `901f229d…` (177,002 B), charge `38d51765…` (66,836 B), exit audit `4906d671…` (4,066 B), envelope 14 rows; `LIVE EXIT PASS paths=5903 added=0 removed=0 changed=0 entry=exit=076e8a2b…`, and my own walk reproduces the 5,903-path set exactly (0 missing, 0 extra, 0 value differences), with the A9 census differing in exactly the two c220 appends; `git status` clean at `b60ce7ca`; no `.git-attempts`, zero live changes, CONFINEMENT zeros, `OPERATION EXIT PASS completed=42 status_counts={0: 41, 1: 1}` (the 1 is the caller audit's STOP); the four C01 certificate reds are the known ordering artifact (FLAGGED: the native selector writes the current certificate before comparing the stale carried table; both second checks green after adoption, the original red/green lines quoted); `CLOCK` A 19.508612363/300, C 162.352083326/6,500 (pre-A0 89.012740512; the continuous charge to the seal 539.771123214); memory peak 605,982,720 B oom 0/0. **The A9 record is intact:** its 70,533 payloads re-hash with exactly the two moved files differing (re-verified at `.temp/director/812-stop-9/`). **Defect #52 (mine):** the RENDER bullet universalized the "or its plain stem" alternative into a file-wide uniqueness test; the witness — the `incoming=` line and the 749 `executed=` line — is what locates the caller, and the callee alternative only validates the within-function call site; the plain-stem call of a different frame is a legitimate independent call and not a competitor; the STOP is the rule working, no worker error; fifth of the #48–#51 class. The worker's own readback confirms the cardinality (`target_only_owners=1 target_or_plain_stem_owners=2 call_sites=3 enclosing_recorded_line_owners=1`). Charge: 539.771123214 s of 26,600 (A 19.508612363, C 162.352083326; pre-A0 89.012740512; the continuous charge to the seal 539.771123214); returned 26,060.228876786 s; the c220 charge text's total 67,138.174116771 differed from cumulative + allocation by 438 ns (a μs-rounding slip in the A9 total) and is reconciled: cumulative 40,538.174116333 + allocation 26,600 = authorized total 67,138.174116333. Records: THIS gate section; the Amendment 10 report/PROGRESS move to `.temp/director/812-stop-10/` (re-hashed `c538c9f6…` / `19f3474b…`), the log archives `.temp/logs/archive/cdx_brief_812.stop-10.log`; the amendment-10 workdir stays as the stop-10 sealed record.

## Amendment 11 — the RENDER-relation witness fix; Stages C–F complete (implementation continuation)

**Scope.** Execute Amendment 10's Stages C–F from the accepted Amendment 6 products in the fresh workdir `.temp/cm4-campaign-epoch/amendment-11/` (`W` below). No source edit; every build and measurement lands under `W`; the sealed parents and the 754/797/806e/812-10 records stay read-only. Reused by hash identity, no re-run: the twelve canonical labels, the three spot identities, the two 205-case Rust selector runs, the `default-disabled` diagnostic (`a76545f3…`), and the four drive builds of the Amendment 8 record (`default-armed` `e539fb09…` / `d73aa794…`, `default-disabled` `fcb29953…` / `2271c3ee…`, `lane-armed` `fd7802f7…` / `5cf8f5ce…`, `lane-disabled` `f0358d08…` / `6eafd27a…`), each re-authenticated through the Amendment 10 seal before use. The 20 drives execute in this attempt on those sealed trees (every drive log is this attempt's).

**A0 (300 s).** Close by hash identity against the Amendment 10 record: the seal `4e5f2ba7…` (70,547 payloads / 1,555,056,551 B) with its two moved files re-rooted to `.temp/director/812-stop-10/` — `report.json` `c538c9f6…` (1,302,283 B) and `PROGRESS.md` `19f3474b…` (3,468 B), each printed `STOP10 MOVED PASS path=… bytes=… sha256=…` and the seal read `STOP10 SEAL PASS files=70547 bytes=1555056551 envelope=14 moved=2`; the payload manifest `c8ebb0ce…` (15,896,674 B), summary `b832965f…`, finding `901f229d…` (177,002 B), charge `38d51765…` (66,836 B), exit audit `4906d671…` (4,066 B), envelope `4e5f2ba7…` (14 rows); the four drive builds above against their `CARRIER PASS` receipts; and the live census against `.temp/cm4-campaign-epoch/amendment-10/live-exit-stat.json` (`076e8a2b…`, 5,903 paths), permitting only `TODO.md` (`17043df5…`, 1,152,127 B) and this brief (`4f7ee6ea…`, 137,183 B) to differ, each verified as a strict append and printed `C220 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…`; any other difference stops.

**Stage C (6,500 s).** As Amendment 10 wrote it — the drive pin preflight on the four sealed trees (the program-table seal and 199 `class-provenance.tsv` paths, `replaced=0` armed/lane-armed, `replaced=2` disabled/lane-disabled, restore and re-hash after every run), the 20 drives with the five armed identities per configuration and the inert disabled cohorts, the ten supplemental and 26 withholding fixtures, the three peels (19,184 bytes/level + 3,360 intercept, depth 48 = 924,192), the private-frame E0603 falsifier, the `W/control` generator/continuation/twin/pure-compiler package, the two ordinary importers' fourteen signed compiler controls on the fresh real ELFs with their actual rustc argv/stderr and zero schedule credit, and the reconciliation of 12 canonical + 1 independent + 4 drive plus the itemized functional CLI and workspace/producer jobs — with the caller-identity rules as follows.

**The caller-identity rebase.** As Amendment 10: the 288 witnesses (72 rows × default/lane × armed/disabled) bind by these relations, printing one `SCHEDULED CALLER BOUND tree=… row=… relation=EXACT|ORDINARY|RENDER path=… owner=… function=… body=… calls=…` row each.

- **EXACT** — unchanged and overlay-free: the 754 stamp resolves to exactly one function with the row's name in the drive tree; where the 749 record carries a witness path it must be that path; the row prints `path_in_overlays=true|false` with the real and drive file hashes. The three shared-file cells pinned at c220 stay acceptance cells: P2611/P2612 (`observe_segment`, `crates/rum-stdlib/tests/suspension_oracle/adapter_facade.rs`, real==drive `2099dfa0…`, witness line 26 inside 11–33, body `6d162986…`, calls=6) and G15B (`resume`, `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs`, real==drive `20e1f69d…`, witness line 174 inside 165–176, body `655fb3dd…`).
- **ORDINARY** — unchanged: the stamp does not resolve in the drive tree but resolves in the real role to the function(s) with the row's name at the witness path; each must have exactly one same-owner same-name counterpart in the drive tree at the same path; the path must be a declared `overlays.tsv` key whose preimage equals the real file and whose postimage equals the drive file; the caller is the counterpart set — one member for a unique stamp, both members of a token-identical sibling pair otherwise.
- **RENDER** — corrected at c221. Permitted only for the ten supplemental C103 rows, bound by the row's `EDGE JOIN` record (`incoming=` path and `driven_target=` call). The binding is the function that contains the `incoming=` line at that path; when the 749 record carries a witness (`executed=path:line`), that line must lie inside the same function. Inside that function the call named by `driven_target=` must occur, or the plain stem of that call — the two branches of the driven if/else counted once — and the call must resolve to exactly the declared target (or the declared ordinary method). A plain-stem call in any other function of the file is not a candidate and does not compete; the candidate is never the file's every caller of the stem.

**The RENDER stops.** Stops are: no function at the incoming path contains the incoming line; the containing function contains neither the target call nor its plain stem; the target call resolves to anything but the declared target; a recorded witness line outside the containing function. A RENDER row's `EDGE JOIN` evidence (`incoming=`, `driven_target=`, the 749 witness) is printed with the bound row. **P1070 (cut 52) is a pinned acceptance cell:** `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs`, function `SelectGuardedReplacePartTargetsOwnedSuspensionFrame::resume` at lines 100–121, body `267019e62c032db66e886f0ca08600481e2446899bd3ea24b69eb598e4c53808`, the driven target called at line 112 and its plain stem at line 117, the incoming line 110 and the 749 witness line 111 inside it; the second `resume` at lines 220–233 (`SelectGuardedReplacePartTargetsSuspendedSuspensionFrame`, plain stem at line 230) is not a candidate.

**Totals and the withholding fixtures.** Per tree the audit prints `scheduled_rows=62 supplemental_edge_fix_rows=10 bound=72 missing=0 unknown=0`; a zero-candidate or multi-candidate row outside the three relations stops. The 26 withholding fixtures are rebased the same way — `(role, cut, caller)` from the 754 lines, the cut's incoming edge from the 749 `EDGE JOIN` where recorded, the withheld copy confined to `W`, the row's program re-run, and the expected failure required to fire (`runtime_catches=0`, the retained guard sites reported); a fixture that passes stops. Binding a row here never waives its Q25 STOP status (P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418 stay declared STOPs).

**Stage D (14,400 s), Stage E (3,000 s), Stage F (2,400 s).** As Amendment 5 wrote them (this brief lines 223–301), including every bound, cost and stop: the nine CLI companions and both frozen programs with the exact 120-pair, 5,519-byte normalized stdout; the six workspace cases with the ordered seven skip pairs, the nine host groups, the current target-by-target comparison, the exact same-binary adjudications and the six ledgers; the full time normalization, falsifier and hardware rules of lines 260–270; the six-build subject freeze, the ordinary mode-specific importers, the zero-build-credit controls, the spots and named negatives; the six fresh C10 runs (0/151 on the build-only control, 1/150 on C10; 55 real keys, one hit, zero settlements, partial-index exit 101); no lane build in the index; and the child epoch identity, the complete payload/report seal, the live exit census and the continuous charge.

**Charge.** Amendment 10 charged 539.771123214 s of its 26,600 s (A 19.508612363, C 162.352083326; pre-A0 89.012740512; the continuous charge to the seal 539.771123214); 26,060.228876786 s returned, and the pool is **26,060.228876786 s**. The c220 charge text's total 67,138.174116771 differed from cumulative + allocation by 438 ns (a μs-rounding slip in the A9 total); the reconciled ledger is cumulative 40,538.174116333 + allocation 26,600 = authorized total 67,138.174116333. This amendment allocates A0 300 / C 6,500 / D 14,400 / E 3,000 / F 2,400 = **26,600 s** = the pool + an explicit **539.771123214 s top-up** (the user's standing rule, 2026-11-19); the authorized total becomes **67,677.945239547 s**; the cumulative before dispatch is **41,077.945239547 of 67,677.945239547**. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** Unchanged from Amendment 10: the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, warning, parser vector, producer, position/identity, visibility, build/spot, raw receipt, functional or comparison mismatch, a missing bound input, an unavailable required interference proof, a drive pin preflight mismatch, a first failed drive, a projected overrun or a stage deadline.

## Director gate (c222, 2026-11-27) — Amendment 11's Stage C prefix passes (A0 by hash identity, 20/20 drives, the 288-row caller audit with the P1070 pinned cell, and the 26 static withholding falsifiers) and the STOP at the extra runtime twin P0813 exposes my compressed withholding clause (defect #53, mine)

Amendment 11 was dispatched 08:15:42 as unit `brief-812-cdx-20261127-081542-169255` (task `b86z7ax6f`, log `.temp/logs/cdx_brief_812.log`; Codex v0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a21a5b-a223-7a60-b7d9-5086e267976e`) and returned rc 0 at 08:37:35, 235,901 tokens, peak 2,811,736,064 B, oom 0/0, verdict **STOP**. **A0 PASS** (the Amendment 10 record by hash identity: `STOP10 MOVED PASS` for `report.json` `c538c9f6…` 1,302,283 B and `PROGRESS.md` `19f3474b…` 3,468 B at `.temp/director/812-stop-10/`, `STOP10 SEAL PASS files=70547 bytes=1555056551 envelope=14 moved=2` `4e5f2ba7…`; the four parents `cb9205a6…`/`dda9146e…`/`30cd48fc…`/`df0db63c…` with manifests, the four-edit children `689b9188…`/`f9661a93…`/`450b9e2f…`, the retained real F13 `8d6ba3f0…` with receipt `d20465a0…` and raw `6a376127…`, the design `da2fdf55…` 30/632, the render inputs, the observer `50cc4a95…`, the final roles `b0bdcee9…`/`084f8cc8…`/`d5267244…` with the five native F13 cells, the twelve labels and three lane trees; the two strict appends printed `C220 APPEND PASS` — `TODO.md` `17043df5…` 1,152,127 B + 4,498 B = `cdb684c1…` 1,156,625 B, this brief `4f7ee6ea…` 137,183 B + 15,274 B = `3beba127…` 152,457 B — at `LIVE ENTRY PASS paths=5903 sha256=1f725c2c…`). **Stage C passed through the drives and the caller audit:** 20/20 preflights (`class_paths=199`; `replaced=0` armed/lane-armed, `replaced=2` disabled/lane-disabled, `remaining_mismatches=0`, 20 restorations); 20/20 drives with exactly one `CM740_SELECTOR_ARMED` and one `CM740_SOURCE_HIT` per armed row and the join/queue ids at `pair=1`, `producer_occurrence=0`, generation 2/2/2/3/4 (P0267/P0312/P0323 joins, P1470/P1471 queues), disabled cohorts inert 60/60/60/45/45 (`class_hits=0 queue_hits=0 join_hits=0`), zero credit; the fresh caller audit per tree `bound=72 missing=0 unknown=0` with `EXACT=61 ORDINARY=10 RENDER=1` and `pinned_cells=4`, **the c221 RENDER fix validated on its pinned acceptance cell** — P1070 binds `function=resume` lines 100–121, body `267019e6…`, `incoming=…guarded/resume_frames.rs:110`, `executed=…:111`, `resolved=select_guarded_replace_part_targets_suspended_cm740_driven@112->…guarded.rs@605, select_guarded_replace_part_targets_suspended@117->…guarded.rs@539`, `logical_edges=1`, the second `resume` at 220–233 never a candidate — and the three c220 pins exact (P2611/P2612 `adapter_facade.rs` `2099dfa0…` `path_in_overlays=false` body `6d162986…` witness 26 in 11–33; G15B `20e1f69d…` body `655fb3dd…` witness 174 in 165–176); **the 26 withholding fixtures passed as the 754 record defines them** — static falsifiers, every withheld copy printing `WITHHOLDING SOURCE REJECTION role=… cut=… caller=… selected_driven_edges=0 retained_typed_ordinary=1 retained_guard_sites=N runtime_catches=0` and `STATIC WITHHOLDING PASS fixtures=26 arms=2 cuts=13 selected_edges_rejected=26 campaigns=0 catches=0`, after the first locator run (the opening line matched twice at disabled cut 58 P0509) was corrected to the archived full expression, which resolves uniquely at line 291 (disclosed). **The STOP** fired at the extra runtime twin `class-armed-15-withheld-P0813`: the withheld cut-15 image (the driven `Catch` dispatch in `control_data/mod.rs` replaced by the plain ordinary call, 4,619 → 4,467 B `12ce818b…`, with `mutants.tsv` and `overlays.tsv` re-rendered) ran the native positive fixtures; P0812's synchronous group failed exactly (exit 101, `FIXTURE POSITIVE_MISS row=P0812 receipt`, halves equal to the positive parent), P0813's typed group passed (exit 0, both halves `status=ok`, a committed `CLASS_RECEIPT … kind=EvalCatchSuspensionFrame issuer=Typed owner=P0813 occurrence=1 slot=1` consumed by `TypedAdapter`, `FIXTURE VERDICT row=P0813 mode=positive PASS hits=0`, halves equal), and "a fixture that passes stops" stopped Stage C; the earlier P0812 runtime votes run had used the campaign omission selector, which `class_lease::install` excludes, and is retained as an ordinary measurement (disclosed). **The record I verified:** seal `ca5f4962…` (89,292 payloads / 2,791,859,070 B, all re-hashed equal in 2.68 s, 0 missing 0 bad), payload manifest `26c2e191…` (20,403,426 B), report `974b52eb…` (1,527,260 B; `report.py check 812` OK 1,916 citations), PROGRESS `2a2a7c97…` (5,384 B), summary `cf479681…`, finding `2be0731f…` (137,437 B), charge `1bb8301f…`, exit audit `cb1368e4…` (4,077 B), envelope 14 rows `7b297cbc…` (every row re-hashed); `LIVE EXIT PASS paths=5903 added=0 removed=0 changed=0 entry=exit=1f725c2c…`, my own walk exactly the 5,903-path set (0 missing, 0 extra, 0 value differences), the Amendment 10 census differing in exactly the two c220 appends; `git status` clean at `7980bf98`; `OPERATION EXIT PASS completed=77 status_counts={0: 73, 1: 3, 101: 1}` (the three exit-1 are the retained locator failure, the omission-selector measurement and the stop itself; the 101 is the expected P0812 twin failure); the four C01 certificate reds are the known ordering artifact with the review/check pairs green; CONFINEMENT zeros; memory peak 2,811,736,064 B oom 0/0. **Defect #53 (mine):** the 754 record states "The 26 typed withholding fixtures are static falsifiers" with the expected fields `selected_driven_edges=0 retained_typed_ordinary=1 retained_guard_sites=N runtime_catches=0`; my Amendments 9–11 compressed that into "the row's program re-run, and the expected failure required to fire" without naming the instrument or its group, so the worker built the native class-lease positive fixture as the row's program and required its failure — but a typed-group fixture is satisfied by the retained ordinary path (`retained_typed_ordinary=1`), making the bar unsatisfiable by construction. The STOP is the rule working; no worker error; sixth of the #48–#53 class. The two earlier apparent stops (the P0509 locator, the omission-selector votes run) are accepted as retained, disclosed measurements, not campaign stops. **Charge:** 1,301.501577813 s of 26,600 (A 21.622752027, C 1,015.321284131; D–F 0; the two retained premature CLOCK marks at 454795.194 and 455069.936; dispatch→A 71.872 s); returned **25,298.498422187 s**; cumulative **42,379.446817360 s**. Records: THIS gate section; the Amendment 11 report/PROGRESS move to `.temp/director/812-stop-11/` (re-hashed `974b52eb…` / `2a2a7c97…`), the log archives `.temp/logs/archive/cdx_brief_812.stop-11.log`; the amendment-11 workdir stays as the stop-11 sealed record.

## Amendment 12 — the withholding fixture restored to the 754 static falsifier; Stages C–F complete

**Scope.** Execute Amendment 11's Stages C–F from the accepted Amendment 11 products in the fresh workdir `.temp/cm4-campaign-epoch/amendment-12/` (`W` below). No source edit; every build and measurement lands under `W`; the sealed parents and the 754/797/806e/812-10/812-11 records stay read-only. Reused by hash identity, no re-run: the twelve canonical labels, the three spot identities, the two 205-case Rust selector runs, the `default-disabled` diagnostic (`a76545f3…`), and the four drive builds of the Amendment 8 record (`default-armed` `e539fb09…` / `d73aa794…`, `default-disabled` `fcb29953…` / `2271c3ee…`, `lane-armed` `fd7802f7…` / `5cf8f5ce…`, `lane-disabled` `f0358d08…` / `6eafd27a…`), each re-authenticated through the Amendment 11 seal before use. The 20 drives execute in this attempt on those sealed trees (every drive log is this attempt's).

**A0 (300 s).** Close by hash identity against the Amendment 11 record: the seal `ca5f4962…` (89,292 payloads / 2,791,859,070 B) with its two moved files re-rooted to `.temp/director/812-stop-11/` — `report.json` `974b52eb…` (1,527,260 B) and `PROGRESS.md` `2a2a7c97…` (5,384 B), each printed `STOP11 MOVED PASS path=… bytes=… sha256=…` and the seal read `STOP11 SEAL PASS files=89292 bytes=2791859070 envelope=14 moved=2`; the payload manifest `26c2e191…` (20,403,426 B), summary `cf479681…`, finding `2be0731f…` (137,437 B), charge `1bb8301f…`, exit audit `cb1368e4…` (4,077 B), envelope `7b297cbc…` (14 rows); the four drive builds above against their `CARRIER PASS` receipts; and the live census against `.temp/cm4-campaign-epoch/amendment-11/live-exit-stat.json` (`1f725c2c…`, 5,903 paths), permitting only `TODO.md` (`cdb684c1…`, 1,156,625 B) and this brief (`3beba127…`, 152,457 B) to differ, each verified as a strict append and printed `C221 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…`; any other difference stops.

**Stage C (6,500 s).** As Amendment 11 — the drive pin preflight on the four sealed trees (the program-table seal and 199 `class-provenance.tsv` paths, `replaced=0` armed/lane-armed, `replaced=2` disabled/lane-disabled, restore and re-hash after every run), the 20 drives with the five armed identities per configuration and the inert disabled cohorts, the ten supplemental edge-fix rows; **the caller audit is re-authenticated by hash identity against the Amendment 11 record** — its `jobs/generator-callers` raw log and the 288 bound rows (per tree `scheduled_rows=62 supplemental_edge_fix_rows=10 bound=72 missing=0 unknown=0`, `EXACT=61 ORDINARY=10 RENDER=1`, `pinned_cells=4`), the P1070 pinned acceptance cell and the three shared pins — and re-runs only if any input pin differs; then the three peels (19,184 bytes/level + 3,360 intercept, depth 48 = 924,192), the private-frame E0603 falsifier, the `W/control` generator/continuation/twin/pure-compiler package, the two ordinary importers' fourteen signed compiler controls on the fresh real ELFs with their actual rustc argv/stderr and zero schedule credit, and the reconciliation of 12 canonical + 1 independent + 4 drive plus the itemized functional CLI and workspace/producer jobs.

**The 26 withholding fixtures (corrected).** They are the 754 record's static falsifiers — "The 26 typed withholding fixtures are static falsifiers" — each expected as `WITHHOLDING FIXTURE EXPECTED FAILURE role=… cut=… caller=… selected_driven_edges=0 retained_typed_ordinary=1 retained_guard_sites=N runtime_catches=0`. The withheld copy suppresses the cut's incoming edge and keeps the whole frozen body; its re-scan must reject the selected edge (one `WITHHOLDING SOURCE REJECTION role=… cut=… caller=… selected_driven_edges=0 retained_typed_ordinary=1 retained_guard_sites=N runtime_catches=0` line per fixture) and the pass line must read `STATIC WITHHOLDING PASS fixtures=26 arms=2 cuts=13 selected_edges_rejected=26 campaigns=0 catches=0`. The Amendment 11 static pass (its pinned raw log, re-hashed) may be reused by hash identity. **The native class-fixture twins on a withheld source are diagnostics, never a bar:** a twin that passes — its group's contract satisfied by the retained ordinary path (`retained_typed_ordinary=1`) — is printed as a diagnostic row and does not stop; only a twin whose program halves differ from the positive parent, or a withheld copy whose re-scan binds the suppressed edge, stops.

**Stages D (14,400 s), E (3,000 s), F (2,400 s).** As Amendment 5 wrote them (this brief lines 223–301), including every bound, cost and stop: the nine CLI companions and both frozen programs with the exact 120-pair, 5,519-byte normalized stdout; the six workspace cases with the ordered seven skip pairs, the nine host groups, the current target-by-target comparison, the exact same-binary adjudications and the six ledgers; the full time normalization, falsifier and hardware rules of lines 260–270; the six-build subject freeze, the ordinary mode-specific importers, the zero-build-credit controls, the spots and named negatives; the six fresh C10 runs (0/151 on the build-only control, 1/150 on C10; 55 real keys, one hit, zero settlements, partial-index exit 101); no lane build in the index; and the child epoch identity, the complete payload/report seal, the live exit census and the continuous charge.

**Charge.** Amendment 11 charged 1,301.501577813 s of its 26,600 s (A 21.622752027, C 1,015.321284131; D–F 0; the two retained premature CLOCK marks at 454795.194 and 455069.936; dispatch→A 71.872 s); 25,298.498422187 s returned, and the pool is **25,298.498422187 s**. This amendment allocates A0 300 / C 6,500 / D 14,400 / E 3,000 / F 2,400 = **26,600 s** = the pool + an explicit **1,301.501578 s top-up** (the user's standing rule, 2026-11-19); the authorized total becomes **68,979.446817547 s**; the cumulative before dispatch is **42,379.446817360 of 68,979.446817547**. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** As Amendment 11: the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, warning, parser vector, producer, position/identity, visibility, build/spot, raw receipt, functional or comparison mismatch, a missing bound input, a drive pin preflight mismatch, a first failed drive, a static withholding row whose fields differ from the 754 record, a projected overrun or a stage deadline.

## Director gate (c223, 2026-11-27) — Amendment 12 closes A0 by hash identity and passes the whole Stage C prefix (20 fresh drives, the 288-row caller audit re-authenticated, the 26 static falsifiers, three peels, E0603, the resolver, the fourteen signed compiler controls, the pure compiler fixture), then stops on an extra control it built: the 740-era c7 ledger signature of P0813 does not reproduce on the class-form tree; the bar was not this brief's, the accepted records carry P0813 on the typed adapter (defect #54, mine), and the finding — the P0813 suspension-path mutation is unobservable on the final tree — is recorded for the 744 gate

Amendment 12 was dispatched 08:48:41 as unit `brief-812-cdx-20261127-084839-189648` (task `bjnv4t8cw`, log `.temp/logs/cdx_brief_812.log`; Codex v0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a21a79-cdd6-72f2-a89c-e2c7445816c4`) and returned rc 0 at 09:14:39, 335,034 tokens, peak 2,992,566,272 B, oom 0/0, verdict **STOP**. **A0 PASS** (the Amendment 11 record by hash identity: `STOP11 MOVED PASS` for `report.json` `974b52eb…` 1,527,260 B and `PROGRESS.md` `2a2a7c97…` 5,384 B at `.temp/director/812-stop-11/`, `STOP11 SEAL PASS files=89292 bytes=2791859070 envelope=14 moved=2 sha256=ca5f4962…`, the design seal `da2fdf55…`, the four 806e role trees and four-edit children, the four drive builds `e539fb09…`/`fcb29953…`/`fd7802f7…`/`f0358d08…`; the two strict appends printed `C221 APPEND PASS` — `TODO.md` `cdb684c1…` 1,156,625 B + 5,393 B = `6d76194e…` 1,162,018 B, this brief `3beba127…` 152,457 B + 13,711 B = `b8f0a5a8…` 166,168 B — at `LIVE ENTRY PASS paths=5903 authorized_appends=2 sha256=950c8be6…`). **Stage C passed through the whole prefix:** 20/20 fresh drives (`fresh_drive_runs=20`; ten armed rows `CM742C81 DRIVE VERDICT … armed=true fired=true` with joins P0267/P0312/P0323 at generation 2 and queues P1470/P1471 at generations 3/4, `pair=1`, `producer_occurrence=0`; ten disabled cohorts inert 60/60/60/45/45; the four layouts' `EMBEDDED SOURCE EXIT PASS … changes=0` and `CARRIER RECEIPTS VERIFIED runs=5 class_paths_per_run=199 replaced_per_run=0/2 restorations=5`), the caller audit re-authenticated (`CALLER READBACK PASS … bound=72 missing=0 unknown=0 EXACT=61 ORDINARY=10 RENDER=1 pinned_cells=4` per tree; `CARRIER BUILD RECONCILIATION canonical=12 independent_diagnostic=1 drives=4 drives_reused=4 fresh_drive_runs=20 campaign_credit=0`), the 26 static falsifiers (`STATIC WITHHOLDING PASS fixtures=26 arms=2 cuts=13 selected_edges_rejected=26 campaigns=0 catches=0`, the two withheld twins printed as diagnostics — cut 15 P0812 withheld exit 101, P0813 withheld exit 0, halves equal), the three peels (`PEEL PASS role=real|disabled|armed points=7 bytes_per_level=19184.0000 intercept=3360.0000 depth48_bytes=924192 default_limit=1048576 corpus=pass`), the private-frame E0603 falsifier (exit 101, `error[E0603]: function record_descendant_frame is private`), the resolver control (2 passed), the fourteen signed compiler controls (seven per mode, each with a rustc receipt) and the pure compiler fixture (1 passed per mode); the standing replays were re-authenticated by hash identity (the readback log byte-identical to Amendment 11's `184a3fb7…`), and the four C01 certificate reds remain the known ordering artifact. **The STOP** fired on an extra control the worker built: `GENERATOR CONTROL COMPARISON STOP fixture=generator-default-armed-P0813-Middle-complete check=producer-arm expected=CM740_SELECTOR_ARMED row=P0813 phase=owned-returned-InputRequired prompt="native623.one" observed=[]` — the armed P0813 complete positive emitted zero arms, zero source hits and zero identities, its worker exited 0 with `predicate=Ok(())`, `public_output_exact=true`, `native_difference=false`, while the frozen c7 positive requires `Err(Output)`, `false`, `true`; my own re-run reproduces it exactly (0 `CM740_SELECTOR_ARMED`, 0 `CM740_SOURCE_HIT`, 1 `CM740_OWN_PREDICATE predicate=Ok(())`, `CM740_OBSERVATION_END row=P0813 records=1 arm=armed`, 790 filtered), and the three roles' observations are byte-identical (`OBSERVATION EQUALITY half=resumed … 058e0ce1…`, `half=queued … 6dca671f…`, output 13). **The mechanism** (the worker's own witnesses, which I reproduced): `arm_source` returns early unless `producer_owner(l, row)` finds a P0813 activation (`lib.rs:1537–1611`, early return 1563–1564); the suspension path opens only `enter("P0812")` (`nonlocal.rs:139`) and calls `returned("P0813", …)` at :1178 without a P0813 scope, while `around("P0813", …)` at :167 is the `Ok(value)` leg — so no token is issued, `frame("P0813")` (:755) is false and the `ForwardActualCatchBodyBeforeDispatch` hit (:756) never runs: the row's suspension-path mutation is unobservable on the final tree. **Defect #54 (mine):** the brief's `W/control` clause named the package but not the comparison authority, and the worker — reading `draft-epoch.md` ("correct ordered source slot on the positive"), `draft-744.md` ("Require the native positive catches") and the 756 falsifier literally — made the 740-era c7 ledger signature the epoch's positive bar; the accepted records carry P0813 on the typed adapter (756: verdict "missing-link own construction/resume", carrier decision "typed adapter construction/resume", eleven of the 18 historical rows the same; 757 `class-97.tsv`: `design_verdict=design-carried`, `method=typed adapter construction/resume`, `physical_credit=0 new receipts`; 754/755: `CM4 OWN OMISSION PortBlocked row=P0813 missing_evidence=RawArtifact caught=false`; the Amendment 11 typed class fixture passed with a committed `EvalCatchSuspensionFrame` receipt consumed by `TypedAdapter`), and the epoch build cannot edit the sealed source, so the bar is unsatisfiable by construction — seventh of the #48–#54 class, no worker error. **The finding is recorded, not waived:** whether the 744 shakedown accepts the typed-adapter carrier for the eleven missing-link rows or the source must be repaired (the 756 prescription "placing the selected owner on that real Err route") is the shakedown gate's first question. **The record I verified:** seal `30d4be68…` (77,297 payloads / 3,943,792,889 B, all re-hashed equal in 3.79 s, 0 missing 0 bad), payload manifest `149d4353…` (17,438,904 B), report `f6ba8a9a…` (1,974,284 B; `report.py check 812` OK 3,148 citations), PROGRESS `fe081fc7…` (5,933 B), summary `e41326ff…`, finding `97970798…` (20,605 B), charge `10d2b989…`, exit audit `e76eb322…` (4,076 B), envelope 14 rows (every row re-hashed); `LIVE EXIT PASS paths=5903 added=0 removed=0 changed=0 entry=exit=950c8be6…`, my own walk exactly the 5,903-path set (0 missing, 0 extra, 0 value differences); `git status` clean at `c22df084`; `OPERATION EXIT PASS completed=74 status_counts={0: 70, 1: 3, 101: 1}` (the three exit-1 are the retained real-peel scratch reader, the retained compiler-receipt scratch reader and the stop; the 101 is the expected E0603 falsifier); the two disclosed deviations accepted (the original raw receipts retained, no native operation repeated); CONFINEMENT zeros; peak 2,992,566,272 B oom 0/0. **Charge:** 1,546.857638885 s of 26,600 (A 26.525142873, C 798.643060094; pre-A0 97.189205341; the post-stop close 624.500215922 charged); returned **25,053.142361302 s**; cumulative **43,926.304456245 s**. Records: THIS gate section; the Amendment 12 report/PROGRESS move to `.temp/director/812-stop-12/` (re-hashed `f6ba8a9a…` / `fe081fc7…`), the log archives `.temp/logs/archive/cdx_brief_812.stop-12.log`; the amendment-12 workdir stays as the stop-12 sealed record.

## Amendment 13 — the `W/control` package on the authorized authority; Stages C–F complete

**Scope.** Execute Amendment 12's Stages C–F from the accepted Amendment 12 products in the fresh workdir `.temp/cm4-campaign-epoch/amendment-13/` (`W` below). No source edit; every build and measurement lands under `W`; the sealed parents and the 754/797/806e/812-10/812-11/812-12 records stay read-only. Reused by hash identity, no re-run: the twelve canonical labels, the three spot identities, the two 205-case Rust selector runs, the `default-disabled` diagnostic, the four drive builds, the twenty fresh drives' raw receipts, the 288-row caller audit raw receipt and its re-authentication, the 26 static withholding verdicts and the two withheld-twin diagnostics, the three peels and their corpus jobs, the private-frame E0603 falsifier, the resolver build and its two controls, the fourteen signed compiler controls and the pure compiler fixture, and the standing replay readback — each re-authenticated through the Amendment 12 seal before use.

**A0 (300 s).** Close by hash identity against the Amendment 12 record: the seal `30d4be68…` (77,297 payloads / 3,943,792,889 B) with its two moved files re-rooted to `.temp/director/812-stop-12/` — `report.json` `f6ba8a9a…` (1,974,284 B) and `PROGRESS.md` `fe081fc7…` (5,933 B), each printed `STOP12 MOVED PASS path=… bytes=… sha256=…` and the seal read `STOP12 SEAL PASS files=77297 bytes=3943792889 envelope=14 moved=2`; the payload manifest `149d4353…` (17,438,904 B), summary `e41326ff…`, finding `97970798…` (20,605 B), charge `10d2b989…`, exit audit `e76eb322…` (4,076 B), envelope (14 rows); the four drive builds above against their `CARRIER PASS` receipts; and the live census against `.temp/cm4-campaign-epoch/amendment-12/live-exit-stat.json` (`950c8be6…`, 5,903 paths), permitting only `TODO.md` and this brief to differ, each verified as a strict append and printed `C222 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…`; any other difference stops.

**Stage C (6,500 s).** As Amendment 12 — the drive pin preflight on the four sealed trees, the 20 drives, the ten supplemental edge-fix rows, the caller audit re-authenticated by hash identity, the 26 withholding fixtures, the three peels, the private-frame E0603 falsifier — and then the `W/control` package on the corrected authority:

- **`generator-check.py`** with the exact interface of draft 3 (the subject supplies all input paths by digest): its positive and one-fault input tables, compiler and source manifests, and raw receipts. The one-fault variants are wrong/duplicate anchor, wrong tail/destructor, missing resumed edge, **P0813 initial-only**, **P2593 immediate-only**, overlapping rewrites, fabricated None/error, cross-constituent token and disabled difference; each is a control-only data row or isolated rendered source fixture, never a campaign role, and the checker must reject or classify each with its own receipt. The structural controls compare the parsed constructor/receiver, tail and destructor, immediate/resumed edges and nonoverlapping spans against the typed source plan; a single mismatched anchor, omitted edge, fabricated option/error or owner is rejected before render.
- **The dynamic controls are the authorized live positive/withheld twins:** the class-lease positive and withheld pairs and the 26 static falsifiers' declared rows. Their authorized rows are the campaign's records — the class lease's typed receipt/consume (`CLASS_RECEIPT … issuer=Typed owner=<row> occurrence slot …`, consumed at `TypedAdapter`, `FIXTURE VERDICT … mode=positive`), the static falsifier fields, and the `WITHHELD TWIN DIAGNOSTIC` rows. **The 740-era c7 historical ledger signature is not the epoch's bar.** It is the shakedown's oracle: the three named complete programs (P0813 Middle, P2593 Middle, the original P2593 Error) run on the three roles in both modes as **recorded measurements** — each `GENERATOR MEASUREMENT …` line carries its role, mode, program, tuple, arms, hits, predicate, public/native flags and raw receipt; a missing arm/hit or a different predicate is recorded as data and never stops; the epoch publishes nothing from them.
- The measured finding from Amendment 12 is carried forward and restated in the report: on the final D.2 tree the armed P0813 suspension-path ledger arm and hit are absent and the three roles' resumed/queued observations are byte-identical; 756 records the row (and ten others of the 18 historical rows) as `missing-link own construction/resume` with the carrier decision `typed adapter construction/resume`, 757 carries them as design-carried (`0 new receipts`), and 754/755 carried them as own omissions (`PortBlocked missing_evidence=RawArtifact`). Whether the shakedown accepts the typed-adapter carrier or the source must be repaired is the 744 gate's first question; the epoch records the measurement.

**Stages D (14,400 s), E (3,000 s), F (2,400 s).** As Amendment 5 wrote them (this brief lines 223–301), including every bound, cost and stop: the nine CLI companions and both frozen programs with the exact 120-pair, 5,519-byte normalized stdout; the six workspace cases with the ordered seven skip pairs, the nine host groups, the current target-by-target comparison, the exact same-binary adjudications and the six ledgers; the full time normalization, falsifier and hardware rules of lines 260–270; the six-build subject freeze, the ordinary mode-specific importers, the zero-build-credit controls, the spots and named negatives; the six fresh C10 runs (0/151 on the build-only control, 1/150 on C10; 55 real keys, one hit, zero settlements, partial-index exit 101); no lane build in the index; and the child epoch identity, the complete payload/report seal, the live exit census and the continuous charge.

**Charge.** Amendment 12 charged 1,546.857638885 s of its 26,600 s (A 26.525142873, C 798.643060094; the pre-A0 97.189205341 s and the post-stop close 624.500215922 s charged; the two disclosed scratch-reader deviations); 25,053.142361302 s returned, and the pool is **25,053.142361302 s**. This amendment allocates A0 300 / C 6,500 / D 14,400 / E 3,000 / F 2,400 = **26,600 s** = the pool + an explicit **1,546.857638698 s top-up** (the user's standing rule, 2026-11-19); the authorized total becomes **70,526.304456245 s**; the cumulative before dispatch is **43,926.304456245 of 70,526.304456245**. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** As Amendment 12: the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, warning, parser vector, producer, position/identity, visibility, build/spot, raw receipt, functional or comparison mismatch, a missing bound input, a drive pin preflight mismatch, a first failed drive, a static withholding row whose fields differ from the 754 record, a projected overrun or a stage deadline. A generator *measurement*'s missing arm/hit or different predicate is recorded, never a stop.

## Director gate (c224, 2026-11-27) — Amendment 13's cdx attempt passes A0 by hash identity, re-authenticates the retained Stage C evidence, completes all 18 named generator measurements, and is killed by Codex's safety filter — the second refusal on this brief; no amendment is issued and the route returns to the user

Amendment 13 was dispatched 09:31:05 as unit `brief-812-cdx-20261127-093105-213595` (task `b6kg0cohz`, log `.temp/logs/cdx_brief_812.log`; Codex v0.171.0, `gpt-6.2-astra`, effort max, workspace-write, session `01a21aa0-a93d-7633-94f6-5fa6cfc3b765`) and was killed by the safety filter at 09:37:22, rc 1, 141,426 tokens, peak 1,071,312,896 B, oom 0/0. **A0 PASS** (by hash identity against the Amendment 12 record): `C222 APPEND PASS path=TODO.md old_bytes=1162018 old_prefix=6d76194e… appended_range=[1162018,1167757) appended_bytes=5739 new_sha256=336a763f7e449b07b942e6dc2feccaf617a4e706fb485cb11c3289c6f8f4e97d new_bytes=1167757` and `C222 APPEND PASS path=docs/dev/codex/brief-812-g15cm4-campaign-epoch-build.md old_bytes=166168 old_prefix=b8f0a5a8… appended_range=[166168,181169) appended_bytes=15001 new_sha256=0302bc38dac716c355a5d4eb3dc84861e4bfcf76341db031fab5602c9a18123a new_bytes=181169`, at `LIVE ENTRY PASS paths=5903 authorized_appends=2 sha256=68dc1e40e52db20791745474afc8c959686e13b790b14249b1ba417f22002cfe`; I re-hashed the live `TODO.md` (`336a763f…`, 1,167,757 B), this brief (`0302bc38…`, 181,169 B) and the entry census (`68dc1e40…`, 792,152 B) — all equal. **`prepare-roles` exit 0** (`FRESH ROLE PASS role=real files=5851 tree=b0bdcee9…`, `role=disabled … 084f8cc8…`, `role=armed … d5267244…`; `OFFLINE CACHE PASS`). **The retained Stage C evidence re-authenticated** through the stop-12 manifest: `retained-stage-c` try 1 exited 1 on the tool's own `relative_to(ROOT)` binding of the absolute rustc path (`ValueError: '/home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin/rustc' is not in the subpath of '/home/remi/dev/rum'`; raw `c12bf60c…` retained); the worker rewrote the reader (`retained-first.py`) and `retained-stage-c-absolute-pins` exited 0, re-reading the 20 drive verdicts from the A12 raw receipts (10 armed `fired=true`, 10 disabled `inert=true`, cohorts 60/60/60/45/45). **`STAGE C PREFIX PASS labels=12 independent=1 drive_builds=4 drive_runs=20 caller_rows=288 static_rejections=26 twin_diagnostics=2 peels=3 resolver_tests=2 compiler_invocations=14 pure_fixtures=2 fresh_campaign_cells=0`.** **The generator inputs and AST index**: `generator-inputs` exit 0 (raw `dc067af7…`), `generator-ast-inputs` exit 0 (raw `9cb4940e…`), `control/parsed/receipt.json` `6729b5c9…` (source index `f7740005…`, output `1315f9f5…`); `native-inputs.json` `7bc2fab1…` (50,629 B). **All 18 generator measurement jobs exit 0** (`generator-measurements` raw `3a7762fe…`, 18 `COMMAND` lines; `native-complete.json` `3e92c69c…`, 4,356 B, 18 rows — every hash re-verified): **P0813 Middle** `arms=0 hits=0 predicate=Ok(()) public_output_exact=true native_difference=false` in all six cells; **P2593 Middle and P2593 Error** real/disabled `arms=0 hits=0 Ok(()) true false`, armed `arms=1 hits=0 Ok(()) true false` with `CM740_SELECTOR_ARMED row=P2593 phase=producer-InputRequired prompt="First.lazyConsumer:2"` — the 740-era c7 ledger signatures do not reproduce, and the rows stand as **recorded measurements with receipts, zero schedule credit** (Amendment 13's re-scoping; defect #54's correction holds). **The filter**: `2026-11-27T01:37:22.410628Z … Turn error: This content was flagged for possible cybersecurity risk` (three occurrences), on the model call after the 01:37:04 exec while the worker assembled the report material (brief-742 continuation findings, brief-756 frame-authorization, brief-747 source_index, brief-807f functional.py, brief-809 M-real-functional.json, brief-798 capture-plan) and wrote `parse_sources.py`; zero live changes (`git status` clean at `30225586`; every write under `.temp/`), no git attempts, zero credit. **Charge:** 377.466020 s of 26,600 (pre-A0 77.294, A0 27.652, C 240.805, close 31.715); returned **26,222.533980 s**; cumulative **44,303.770476245 of 70,526.304456245**. **This is the second Codex refusal on brief 812** (the first at 05:19:41 on Amendment 7); per the 2026-11-13 ruling (never reword to evade; a second refusal is a STOP for the director) and the c217 reservation, **no amendment is issued and the resume route is the user's decision**: (a) wait for grk (≈2026-11-29); (b) director execution under the 2026-11-24 precedent; (c) another route (second Codex account / Daybreak access / qcn-kimi budget). The worker chain is exhausted for this content — cdx refused twice, grk out until ≈2026-11-29, agy cannot run the mount-namespace stages (c217: `bwrap --ro-bind` fails `Function not implemented` under its seccomp, and the Stage C generator measurements use exactly that bind to present the fresh role trees at the six oracle ELFs' embedded path), mst until ≈2026-12-01. Records: ruling 876; this gate section; the TODO milestone; the attempt record `.temp/director/812-filter-2/attempt.json` (identity, charge breakdown, 26 job receipts, artifact pins); the log archived `.temp/logs/archive/cdx_brief_812.filter-2.log` (`b8951bd1…`, 752,932 B); the amendment-13 workdir stays in place as the attempt's checkpoint for whichever route resumes.

## Director gate (c225, 2026-11-27) — the user routes the brief 812 resume to agy at `gemini-4-carbon-high` effort high; Amendment 14 executes the `W/control` remainder of Stage C on the Amendment 13 checkpoint, build-free and mount-free, with Stages D–F in separately gated amendments

The user: 「现在agy里有额度，用gemini 4 carbon 的 high，你验收时盯紧点。」 (agy has quota now; use `gemini-4-carbon-high` at effort high; watch the gate tightly.) This resolves the c224 route reservation in favour of agy, with the 2026-11-21 fabrication caution and the 629 guard in full: every verdict line copied from a named raw receipt, and the director re-runs what is re-runnable. The Amendment 13 checkpoint is reusable by hash identity (`.temp/director/812-filter-2/attempt.json` `1d3d7dab…`, 7,627 B: 26 job receipts, the three generator artifacts, the charge breakdown), so the resume continues from it rather than from the top.

**Why the split.** Ruling 869's environment fact stands: agy's seccomp refuses mount namespaces (`bwrap`: `Creating new namespace failed: Function not implemented`), and ruling 875/c223 changed Amendment 13's Stage C to the authorized authority. The Stage C remainder after the checkpoint — the `W/control` package (`generator-check.py`, its positive and one-fault tables, the compiler and source manifests, the structural controls), the dynamic-control record (the class-lease typed receipts, the 26 static verdicts, the two withheld-twin diagnostics — re-authentication, not new runs), the measured finding and the reconciliation — is **build-free and mount-free**, so it runs inside agy's sandbox unchanged. Stages D–F need the presentation decisions the design already permits (`source-proof.md`: "Validate mount identity or use byte-identical physical child copies"; draft-epoch.md: "a read-only bind or verified exact copy") and are sized against agy's 5-hour windows; each gets its own amendment and gate.

**Amendment 14** (below) allocates A0 300 / C 6,500 / F0 400 = **7,200 s** of the 26,222.533980 s pool left by gate c224; the pool returns whatever it does not spend.

## Amendment 14 — the `W/control` package on the Amendment 13 checkpoint; Stages D–F in later amendments

**Scope.** Execute the remainder of Amendment 13's Stage C (this brief lines 1397–1405) from the Amendment 13 attempt's checkpoint, in the fresh workdir `.temp/cm4-campaign-epoch/amendment-14/` (`W`). Stages D, E and F are **not** in this amendment; they follow in separately gated amendments. No source edit and no build: this amendment is scratch orchestration and data work outside every role tree. Every write lands under `W`; the sealed parents and the 754/756/757/797/806e/811/812-10…-13 records stay read-only.

**Execution environment (agy).** The run is sandboxed: no network, the repository writable through the `write_file` grant, **no mount namespaces** (`bwrap` is refused: `Creating new namespace failed: Function not implemented`, ruling 869), no user-bus systemd scopes, no subagents. Do not attempt mounts, scope creation, network or subagents. Keep scratch under `W`. Write the checkpoint, the raw receipts and the report as you go: a run cut by a usage wall keeps only what is already on disk.

**A0 (300 s).** Close by hash identity against the Amendment 13 attempt's record, `.temp/director/812-filter-2/attempt.json` (`1d3d7dab…`, 7,627 B): re-hash all 26 job `raw.log` files under `.temp/cm4-campaign-epoch/amendment-13/jobs/<job>/` against `jobs[*].raw` and `jobs[*].raw_bytes`, and their `finish.json` exit codes against `jobs[*].exit`; re-hash the three artifacts against `artifacts` (`control/generator/native-inputs.json` `7bc2fab1…` 50,629 B, `control/generator/native-complete.json` `3e92c69c…` 4,356 B, `control/parsed/receipt.json` `6729b5c9…` 2,551 B); print `A13 CHECKPOINT PASS jobs=26 artifacts=3 attempt=1d3d7dab…`. Then re-hash the Amendment 13 entry census `.temp/cm4-campaign-epoch/amendment-13/live-entry-stat.json` (`68dc1e40…`, 792,152 B) and take the same live census (5,903 paths): every path equal except `TODO.md` and this brief, each a strict extension printed `C224 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…`, with `TODO.md`'s old side `336a763f…`/1,167,757 B and this brief's `0302bc38…`/181,169 B. Any other difference stops.

**Stage C remainder (6,500 s).**

- **`W/control/generator-check.py`** with the exact command line of draft 3 (`.temp/brief-811/draft-744.md:50`): `python3 -IB W/control/generator-check.py --epoch <subject> --signatures <constituents.json> --output <generator-control-receipts.json>`. The subject supplies every input path and digest; the checker reads nothing outside it; stdlib only; deterministic output bytes; exit 0 only on a complete pass, a named error otherwise. It is an epoch deliverable and the 744 driver re-runs it unchanged.
  - **Positive table:** the three named complete programs (P0813 Middle, P2593 Middle, the original P2593 Error) × the three roles × both modes, taken by hash identity from the Amendment 13 checkpoint's recorded measurements (`native-complete.json` `3e92c69c…`, 18 rows with each row's raw receipt) — never re-derived and never re-run.
  - **One-fault table:** wrong anchor, duplicate anchor, wrong tail, wrong destructor, missing resumed edge, P0813 initial-only, P2593 immediate-only, overlapping rewrites, fabricated None/error, cross-constituent token and disabled difference. Each is a control-only data row or an isolated rendered source fixture, never a campaign role; the checker must reject or classify each with its own receipt naming the expected and the observed verdict.
  - **Compiler and source manifests:** every input path/bytes/sha256, the fourteen signed compiler receipts (by hash identity) and the three role source trees.
  - **Structural controls:** parse the unique constructor/receiver, tail and destructor, immediate/resumed edges and non-overlapping spans, and compare against the typed source plan; a single mismatched anchor, omitted edge, fabricated option/error or owner is rejected before render. The sealed 756 parser (`anchors.py`, `controls.py`, `calibrate.py`) is the authority: copy its semantics, never loosen them, and disclose any adaptation. The four C1/G15A/G15B/Some rows of `.temp/brief-756/structural-controls.tsv` stay declared static references.
- **The dynamic-control record** (re-authentication, no new runs): the class-lease typed receipt/consume rows (`CLASS_RECEIPT … issuer=Typed owner=<row> occurrence slot …`, consumed at `TypedAdapter`) and `FIXTURE VERDICT … mode=positive` rows for their declared rows; the 26 static falsifier verdict rows with the 754 fields; the two `WITHHELD TWIN DIAGNOSTIC` rows. Reconcile the counts with zero unknown (26 fixtures, 2 arms, 13 cuts; the class-lease pairs; the two twins as diagnostics), print each row with its archival path and hash, and stop with the exact missing record if any declared row is absent — never manufacture a run. **The 740-era c7 historical ledger signature is not the epoch's bar** (Amendment 13's correction, defect #54): the three complete programs' missing arm/hit or different predicate is recorded data, never a stop.
- **The measured finding**, restated in the report: on the final D.2 tree the armed P0813 suspension-path ledger arm and hit are absent and the three roles' resumed/queued observations are byte-identical (`OBSERVATION EQUALITY` rows); 756 records the row (and ten of the 18 historical rows) as `missing-link own construction/resume` with the carrier decision `typed adapter construction/resume`, 757 carries them as design-carried with `0 new receipts`, and 754/755 carried them as own omissions (`PortBlocked missing_evidence=RawArtifact`). Whether the 744 shakedown accepts the typed-adapter carrier or the source must be repaired is the 744 gate's first question; the epoch records the measurement.
- **The reconciliation:** one table naming every Amendment 13 Stage C item with its disposition — reused by hash identity from the Amendment 13 checkpoint (the twelve labels and the three spot identities, both 205-case selector runs, the `default-disabled` diagnostic, the four drive builds, the twenty drive raw receipts, the 288-row caller audit and its re-authentication, the 26 static withholding verdicts, the two withheld-twin diagnostics, the three peels, the E0603 falsifier, the resolver control, the fourteen signed compiler controls, the pure compiler fixture, the standing readbacks, the three fresh role trees `b0bdcee9…`/`084f8cc8…`/`d5267244…`, the generator inputs and AST index) versus completed here (the control package and its receipts, the dynamic-control record, this reconciliation, the report) — zero unknown.

**F0 (400 s).** Stage close: `W/report.json` (rum-report/1; every verdict line copied from its named raw receipt with its hash; `python3 -I scripts/report.py check W/report.json` clean; the detached summary), `W/charge.json`, `W/exit-audit.json`, `W/PROGRESS.md`, the payload seal over every artifact produced, the live exit census equal to the entry census (zero changes), zero `.git-attempts`, zero campaign credit, zero member cells.

**Charge.** The pool after gate c224 is **26,222.533980 s**; this amendment allocates A0 300 / C 6,500 / F0 400 = **7,200 s** of it; the unspent remainder returns to the pool for Stages D–F. Authorized total **70,526.304456245 s**; cumulative before dispatch **44,303.770476245 s**. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** As Amendment 13 (this brief line 1411): the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, parser vector, producer, identity, receipt, structural-control or comparison mismatch, a missing bound input, a needed mount namespace or scope, a source edit, a build, a new dependency, network, subagent use, a projected overrun or a stage deadline. A generator *measurement*'s missing arm/hit or different predicate is recorded, never a stop. A verdict without its copied receipt line is not evidence.

## Director gate (c225, 2026-11-27) — Amendment 14 is DONE: A0 closes by hash identity, the `W/control` package is complete, deterministic and falsified in both directions, and Stage D follows as Amendment 15 under the copy presentation

The Amendment 14 attempt (unit `brief-812-agy-20261127-141350-345161`, conversation `8748fa83-c15c-4484-9917-8d532fe53e05`, agy 1.5.17, `gemini-4-carbon-high` effort high, `proceed-in-sandbox`) ran 2026-11-27 14:13:50–≈14:58:27 +0800, one turn, rc 0, memory peak 1,260,949,504 B, oom 0/0; verdict DONE. **A0 PASS** by hash identity: all 26 A13 job `raw.log` files against `jobs[*].raw`/`raw_bytes` and their `finish.json` exits; the three artifacts (`native-inputs.json` `7bc2fab1…`, `native-complete.json` `3e92c69c…`, `control/parsed/receipt.json` `6729b5c9…`); the two `C224 APPEND PASS` rows (TODO.md `336a763f…` 1,167,757 B + 5,267 B `10673d88…` = `47104370…` 1,173,024 B; this brief `0302bc38…` 181,169 B + 15,817 B `692022c7…` = `61d92161…` 196,986 B) at `LIVE ENTRY PASS paths=5903 sha256=9484ba10…`. My own metadata walk reproduces the 5,903-path census exactly and entry == exit byte-for-byte.

**Stage C completed.** `W/control/generator-check.py` (36,415 B `15671e59…`, draft-3 CLI, stdlib only, `SubjectReader` digest/byte confinement over 5,156 pins, deterministic 13,648 B `781c84a9…`); the positive table (18 rows by hash identity from `native-complete.json` — every raw equals its A13 job raw, P0813 six cells arms=0 hits=0, P2593 armed six cells arms=1 hits=0 with the recorded `CM740_SELECTOR_ARMED` producer-InputRequired arm); the one-fault table (18 rows: the 11 named Stage C faults plus 7 dynamic token faults — my own 11 standalone re-runs exit 2 with the exact recorded reject lines and stdout equal to each raw); the compiler manifest (14 signed `rustc` invocations, 70 pinned fields re-hashed) and source manifest (the three fresh role trees `b0bdcee9…`/`084f8cc8…`/`d5267244…`, 5,851 files each, re-hashed file-by-file against the A5 sealed manifests); the structural controls (151 plans — every plan's anchor hash/line/quote matches exactly one sealed role tree, 151/151; 97 union rows; the 4 static reference rows C1/G15A/G15B/Some with all 63 unique sites verbatim at their relocated lines in at least two role trees; the four disclosed post-D.2 adaptations are relocations and equivalences — the checker still enforces the exact P0813/P2593 declared sets and rejects both edge-class faults); the dynamic-control record (97 class-lease rows, 26/2/13 static withholding with the exact pass line, 2 withheld twins, unknown=0, manufactured runs 0, zero credit); the reconciliation (19 reused + 4 completed = 23 groups, unknown=0); and the measured finding restated from the A13 raws (12 `OBSERVATION EQUALITY` rows, `MEASURED FINDING PASS … missing_link_historical_rows=11 typed_adapter_positive_pass=true`; the 754 caller line and the 755 omission line both verbatim in their own records).

**The two retained try-1 failures are accepted as internal corrections**: `generator-check-positive` exit 2 (`MissingImmediateEdge:C10.prepared.profiled_consumer` — the checker rejecting its own first subject; the materializer v2 corrected a typed-plan declaration whose edge set is resumed-only, and the final checker still rejects the initial-only/immediate-only faults) and `stage-c-reconciliation` exit 1 (the omission line looked up in the 754 report, corrected to the 755 report where it exists — the 754 report holds the caller line). Both stop records are kept under `W`.

**F0.** `report.py check` OK 230 citations (plain and `--against-git`); seal 158 payloads / 15,761,457 B every hash re-verified; `seal.sha256` matches report `d6ec4b90…`, PROGRESS `c81ff39b…`, summary `58937b00…`, charge `9e011127…`, exit-audit `8af79f8e…`, payloads `e2309300…`; live exit census equal to entry; git clean at `f62e9a38`; CONFINEMENT zeros; zero `.git-attempts` (the two git-fence lines are agy's own `merge-base` probes); the four C01 certificate ordering artifacts settled with four green post-adoption check pairs. **Charge** 2,672.206650125 s of 7,200 (A0 0.676233, C 2,232.127495984, pre-A0 432.432188611, the F0 close 6.970502746); unused 4,527.793349875 s returns; pool **23,550.327329875 s**; cumulative **46,975.977126370 s of 70,526.304456245 s**; zero campaign credit.

## Amendment 15 — Stage D (functional and six-case acceptance) on agy, presented by byte-identical physical copies

**Scope.** Execute Stage D exactly as this brief lines 223–270, in the fresh workdir `W = .temp/cm4-campaign-epoch/amendment-15/`. No source edit; every write under `W`; the sealed parents (amendment-5…14, the A13 role/freeze trees, briefs 743/744/754/755/756/757/797/806e/811/812 and their director records) stay read-only.

**Execution environment (agy).** The run is sandboxed: no network, the repository writable through the `write_file` grant, **no mount namespaces** (`bwrap` refused, ruling 869), no user-bus scope creation, no subagents. Present every required source tree as a **byte-identical physical copy under `W`**, verified by tree hash against its source pin before and after each use: a read-only bind **or verified exact copy** (`draft-epoch.md`), "byte-identical physical child copies" (`source-proof.md`). The same-source presentation of line 252 is the verified copy, never a mount. A path that cannot be expressed as an exact copy under `W` is a named STOP. The dispatch scope is MemoryMax 24G (raised for the six workspace cases; the 797 recipe's measured peak is 19.41 GB); if a case still cannot fit, lower `-j`/test threads with a per-case disclosure — targets, binaries and results unchanged. Keep the checkpoint, raw receipts and report on disk as you go.

**A0 (300 s).** Close by hash identity against the Amendment 14 record: `report.json` `d6ec4b90…` 152,549 B, `seal.json` `af9ab901…` 4,900 B, `payloads.tsv` `e2309300…` 23,650 B, `charge.json` `9e011127…`, `exit-audit.json` `8af79f8e…`, `summary.txt` `58937b00…`, `PROGRESS.md` `c81ff39b…`, the checkpoint `attempt.json` `1d3d7dab…`, the control package manifest `0f94531d…`, the generator receipts `781c84a9…`, and the three role trees (`b0bdcee9…`/`084f8cc8…`/`d5267244…`, 5,851 files each). Then the live census against `.temp/cm4-campaign-epoch/amendment-14/live-entry-stat.json` (792,152 B, sha256 `9484ba10…`): every path equal except TODO.md and this brief, each a strict extension of its Amendment-14-entry bytes printed `C225 APPEND PASS path=… old_bytes=… old_prefix=… appended_range=[n,N) appended_bytes=… appended_sha256=… new_sha256=… new_bytes=…` (TODO.md old side `47104370…`/1,173,024 B; this brief `61d92161…`/196,986 B); any other difference stops. Print `A14 CHECKPOINT PASS`.

**Stage D (14,400 s), every bound of lines 223–270:**
- **Nine fresh CLI companions**: `cargo build --offline --locked -j8 --release -p rum-cli F` per (real, disabled, armed) × (default, no-default, lane) build input under the same release environment; companion ELF receipts; the three spots bound to the corresponding real source/configuration; separate from the 12-label budget.
- **The 18 functional runs**: each companion on `.temp/brief-795/panel/canonical/{functional-output.rum,core-language-microbench.rum}` via `scripts/run-bounded.py --timeout 250 -- <child-rum-cli> --file <program>`, profile off, the 798 capture-plan's ordinary settings, stdin closed, no background heavy load; fixed bytes and canonical bytes `dc6af977…`; exit 0, exact stderr, all 120 ordered (label,iteration) pairs, all nonclock bytes identical, normalized stdout 5,519 bytes and SHA-256 `c193d87f…`; normalize only the seconds field with the sealed anchored grammar at `.temp/brief-807f/tools/functional.py:20`; verify both sides of the changed-label/iteration/nonclock-byte falsifier.
- **The six fresh workspace cases** — the three arms × both feature modes, the uninstrumented real/source reference and the two composed arms, per the 754 protocol: 414 targets, the seven ordered `--skip` pairs and nine host-only groups from `workspace-gate.json`; freeze the full target list, inherited exclusions and admissible role differences before opening output; enumerate the entire current copied workspace in advance and reconcile every old/new target by name and binary identity, including duplicated names; no blanket 48/49 subtraction; execute with `scripts/run-bounded.py --timeout 2700 -- cargo test --offline --locked -j8 --workspace F --no-fail-fast -- --nocapture <the seven ordered --skip NAME pairs>`; the same-source presentation on each side is the verified copy; each inherited switched follow-up on the exact same binary and one-thread loopback errno reproductions only where applicable; register the 806e class-form execution differences by exact current test name and independent paired receipt — a differing count alone is insufficient; ruling 762's one-thread same-binary compiler-oracle adjudication retains original and repeat; emit six per-target ledgers, raw digests, zero unexplained differences and the combined six-comparison manifest.
- **The time-normalization, falsifier and hardware rules of lines 260–270 unchanged**: functional observations make no hardware-timing decision; a priced hardware comparison is not taken here (a newly required one pre-freezes the v2.5 parent and stops at its allocation gate); no timing or hardware credit from old draws; a newly required hot-path edit is a scope STOP.

**F0 (400 s).** Stage close: `W/report.json` under `python3 -I scripts/report.py check`, `W/charge.json`, `W/exit-audit.json`, `W/PROGRESS.md`, the payload seal over every artifact produced, the live exit census equal to the entry census, zero `.git-attempts`, zero campaign credit, zero member cells.

**Charge.** Pool after gate c225 is **23,550.327329875 s**; this amendment allocates A0 300 / D 14,400 / F0 400 = **15,100 s**; the unspent remainder stays for E+F. Authorized total **70,526.304456245 s**; cumulative before dispatch **46,975.977126370 s**. Charge is continuous from dispatch; every stage box is measured from its own CLOCK entry; a stage stops at its deadline.

**Stops.** As Amendment 14: the thirteen Q25 STOPs are P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418; briefs 744, 745 and 746 stay held; CM5 stays unauthorized; stop on a pin, sourceset, parser vector, producer, identity, receipt, structural-control or comparison mismatch, a missing bound input, a needed mount namespace or scope, a source edit, a new dependency, network, subagent use, a projected overrun or a stage deadline. A workspace or functional difference without its exact current test name and paired receipt stops; a verdict without its copied receipt line is not evidence.
