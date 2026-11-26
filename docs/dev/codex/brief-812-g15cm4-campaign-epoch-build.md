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
