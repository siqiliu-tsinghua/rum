# brief-625 — LP01: the logical profiler's contract and its counter oracle

## Goal

This is the first step of the logical-profiler line. Its design is brief-624
and that brief's director review; its trigger is the user's ruling of
2026-10-20 that the profiler counts logical calls across suspension. **This
step changes no counting behaviour.** It freezes the contract and builds the
oracle that every later step (LP02-LP08) is gated by.

When you are done:

- **The contract is written** in `docs/dev/profiling.md`: logical counters,
  execution telemetry, lifecycle equations, schema v2 and the isolated-epoch
  comparison boundary. It is brief-624's design, marked as the target, with
  the current known losses listed as staged.
- **A test-only exhaustive observer** reads every profiler map after a
  subject runs and before any sentinel or projection runs. It is visible
  only under `cfg(test)` or the existing `test-continuation-slots` feature.
- **The counter oracle is ported into the suspension oracle.** It holds:
  - brief-624's 16 minimized witnesses;
  - its 48 correctness programs;
  - every runnable frozen profiling tuple (813 bindings over 650 programs);
  - **all 62 non-encoding tuples that are unbound today, now bound.**

  Each tuple is attributed to its producer and phase. The 240
  internal-encoding requirements stay visible with their G13 owners and are
  not executed.
- **On the unchanged build,** the oracle reproduces exactly today's
  differences as **pinned, owned known losses**. Each loss is keyed by phase,
  site and route, and names its LP step owner. Every semantic receipt stays
  green.
- **The default release build is byte-identical** to the base.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`.
- **brief-624**: its report sections 2 (*The contract*), 3 (*The
  inventory*), 4 (*The mechanism*) and 6 (*The oracle and mutants*), and the
  director review with its five rulings.
- **brief-624's scene** is read-only; copy what you adapt:
  - `.temp/brief-624/DESIGN.md`;
  - `inventory/{path-sites,expanded-paths,routes}.tsv`;
  - `profile/minima-plans.jsonl` (the 16);
  - `correctness/plans.jsonl` (the 48);
  - `profile/tuple-port.tsv` (813 bound, 62 unbound, 240 encoding-owned);
  - `oracle/support/{observe,drive,faults}.rs`, its comparator support.
- **The raw draft** is `.temp/brief-624/drafts/LP01-contract-and-oracle-port.md`.
  This brief governs where they differ.
- **The profiler today:**
  - `crates/rum-runtime/src/runtime/profiling/` (key shapes at
    `mod.rs:351`, `:412`; the outcome mapping at `timing.rs:375-449`);
  - the guest hooks `RumProfileSnapshot`/`RumProfileReset`;
  - `P0817`/`P0818`'s incoming-Throw sampling
    (`kernel_data/control_data/nonlocal.rs:121`).
- **The oracle.** `crates/rum-stdlib/tests/suspension_oracle/`, run with
  `scripts/check-suspension-oracle.sh`.

## Scope

A listed file is permission to do this step's work there. Disclose every
file you edit.

**Files you MAY modify:**

- **Oracle.**
  - `crates/rum-stdlib/tests/suspension_oracle/manifest.rs` and
    `descriptors.rs`, only to register the new counter context and its
    receipts, without weakening any existing check;
  - `crates/rum-stdlib/tests/suspension_oracle/support/mod.rs`,
    `support/observe.rs`, and a new `support/logical_profile.rs`;
  - a new `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile.rs`
    (or a small directory of that name), with its registration in
    `fixtures/mod.rs`;
  - new inventories under `crates/rum-stdlib/tests/suspension_oracle/inventory/`:
    `logical-profile-required.tsv`, `logical-profile-bindings.tsv` and
    `logical-profile-losses.json`.
- **Runtime, test-only.**
  - A new `crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs`
    (and its declaration).
  - In `crates/rum-runtime/src/runtime/profiling/mod.rs`,
    `runtime/mod.rs` and `lib.rs`, only `cfg(test)` or
    `cfg(feature = "test-continuation-slots")` observer wiring and hidden
    forwarding.
  - Nothing that compiles into the default release build.
- **Docs.** `docs/dev/profiling.md` (new).
- **The standing inventory duty** (`TEMPLATE.md`): fingerprints,
  certificates and ownership rows, replayed, never hand-written.

`Expr`/`Symbol`, public host types, codecs, guest internal encodings,
`Cargo.toml` files, `Cargo.lock`, `AGENTS.md`, `TODO.md`, the user guide and
every other brief stay unchanged. **STOP** if the work needs a production
change that compiles into the default build: this step changes no counting.

## Steps

1. **The base, before any edit.**
   - Record the hashes of the default release `rum-cli`, built with one
     codegen unit at its usual path into `.temp/brief-625/target-base-cli`.
   - Run the unmodified oracle once in both builds and record its totals.
2. **The observer.**
   - Read the stats maps and flags exhaustively, after the subject and
     before any sentinel or projection.
   - Keep the existing key and field shapes.
   - Record durations, pauses and segments separately from logical
     counters, and never compare them.
3. **Port and bind.**
   - Port brief-624's 16 and 48 and the 813 bindings.
   - **Bind the 62 unbound non-encoding tuples.** For each, derive a
     source- and route-specific program (Error, Twice, AlreadyResumed,
     Nested, matching or nonmatching cycle) and bind it *before* executing
     it. Prove the producer, the phase and the exact planned requests.
   - Keep every frozen key: a new binding never merges routes or
     variants.
   - **If a binding uncovers a semantic difference**, minimize it and STOP
     that row's adoption. It becomes a prerequisite repair, reported.
4. **Exactness.**
   - Freeze three collections independently: required tuples, bindings
     and executed receipts, each with multiplicity and an exact source
     fingerprint.
   - Add negative controls: a dropped key, a duplicate tuple, an absent
     execution, a missing second pause, and a malformed requirement. Each
     must be red.
5. **Pin the losses.**
   - On the unchanged build, record every counter difference as a known
     loss: phase, site, route, the exact old and new counter vectors, and
     its owning LP step from brief-624's step table.
   - A difference not in brief-624's inventory is a STOP and a report, not
     a new baseline.
6. **The contract document.** Write `docs/dev/profiling.md` from brief-624
   §2 and the director review: the target contract, the lifecycle
   equations, schema v2, epochs, and the staged known losses with owners.

## Verification (MANDATORY — run to green or evidence)

**How to run.** Serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-625/tmp …`.
Bounds: workspace 3 600 s, oracle 1 500 s per build, everything else 900 s.

```
cargo fmt --check
cargo test -p rum-runtime --lib profiling
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
cargo test --workspace --no-fail-fast
```

- Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` and report the
  nine as skips; the director runs them natively.
- **The no-cost proof.** Rebuild the default release `rum-cli` with one
  codegen unit, at the same path, into `.temp/brief-625/target-cand-cli`.
  Its hash must equal step 1's. If it does not, find which default-build
  code changed and STOP.
- Report every `test result:` total, the oracle's fault-occurrence deltas,
  the tuple totals (required, bound and executed, plus the encoding-owned
  count) and the pinned-loss count by owner.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- **A phase key is not a logical identity.** Recursive and repeated calls
  share keys.
- **A source producer row is not a phase cell.** Pin each loss by phase,
  site and route.
- **An early Throw that prevents a planned second pause** makes a Twice
  plan invalid, not passing.
- **Durations and pause or segment counts are telemetry.** Comparing them
  would fail every resumed program for the wrong reason.
- **No counting change sneaks in here.** The release-hash proof is the gate.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-625/`.
- After each step, write `PROGRESS.md` and `state.json`. They record:
  - the verified state;
  - the counters (tuples bound, executed and pinned);
  - any half-edited file;
  - the exact next action.

## Hard rules

- **No `git`, in any form.** No network, no new `unsafe`.
- Modify only the Scope. Do not weaken an oracle `Checks` flag, a
  fault-absence gate, a retention reject or a documented test.
- No internal-encoding probe generation; rum against rum only.
- **Stop-loss is keyed on progress.** The counters are tuples bound, receipts
  executed, and controls red. Stop after two attempts on a compiling tree
  that advance none of them.
- Paste real output. Say plainly what did not run. Leave the tree
  uncommitted.

## Report format (append to this brief)

1. **Files changed**, each with its purpose, and every inventory row
   change.
2. **The observer and its isolation**, with evidence that nothing reaches
   the default build.
3. **The port.**
   - Tuple totals.
   - The 62 new bindings, each with its producer and phase proof.
   - The negative controls.
4. **The pinned losses**, by phase, site, route and owner.
5. **All gates, verbatim**, and the release-hash proof.
6. **Stop-loss counters, deviations and open questions.**


---

## Director continuation 1 (2026-10-20) — the observer's scope, by class

The stop was correct, and the miss is mine: brief-624's
`inventory/auxiliary-storage.tsv` lists the evaluation-cache profiler
storage, and my Scope did not. Step 1's evidence is accepted:

- the base release `rum-cli` hashes;
- the unmodified oracle, 496 tests in both builds with fault delta 0.

**The Scope is widened by class, not by another list.**

- **Test-only accessors and forwarding are permitted in any module that
  owns profiler storage.** That means every storage row in
  `.temp/brief-624/inventory/auxiliary-storage.tsv`, every map under
  `crates/rum-runtime/src/runtime/profiling/`, and any further profiler
  storage you discover. Today that includes:
  - `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs`;
  - `evaluation_cache/profile/mod.rs`;
  - `evaluation_cache/profile/source.rs`.
- **Each addition must compile only under `cfg(test)` or
  `cfg(feature = "test-continuation-slots")`.** It must not change
  visibility or code in the default build. Disclose every file and item
  added.
- **STOP only if** observing some storage would need a change that compiles
  into the default build. The release-hash proof remains the gate.

Everything else in the brief stands. Continue from the checkpoint at
step 2.

---

## Executing-agent report — 2026-10-20 — STOP at observer scope

LP01 is **not complete**. Step 1 is verified. Before editing source for step
2, inspection found that the required exhaustive observer needs test-only
access to private evaluation-cache profiler storage in files outside Scope.
The template's scope hard rule says: “If you believe you must, STOP and write
that in the report instead.” No permission is inferred for those edits.

### 1. Files changed

- This brief: appended this execution report, as requested by Report format.
- No runtime, oracle, inventory, contract-document, manifest, lockfile, or
  other source/document file was edited. No inventory row changed; no source
  edit requiring standing inventory replay was made.
- Evidence/checkpoints are under `.temp/brief-625/`: `entry-hashes.json`,
  `base-cli-build.log`, `base-cli-hashes.json`, `base-oracle.log`,
  `base-oracle-default.log`, `base-oracle-no-default.log`, `PROGRESS.md`, and
  `state.json`. The base artifact is under `target-base-cli/release/rum-cli`.
  The mandatory oracle script also writes its own fixed
  `.temp/suspension-oracle/` scratch directory; both resulting logs were
  copied into this brief's scratch directory. Brief-624's scene is unchanged.

### 2. The observer and its isolation — blocked before implementation

“Reads every profiler map” includes the evaluation-cache profiler storage
explicitly inventoried in `.temp/brief-624/inventory/auxiliary-storage.tsv`.
Brief-624 section 2 retains its logical entry/application/scan counters in
the comparison, and section 6 calls for lexical+evaluation-cache and all
source/detail modes. Omitting that storage would make the observer
non-exhaustive and would omit source-attributed logical counters.

The inaccessible live storage is:

| Storage | Evidence |
| --- | --- |
| Enabled, active and incremental flags | `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:31-33` |
| `SYMBOL_PROFILE_COUNTERS` | Same file, private getter at line 218 and static at line 219 |
| `INVALIDATION_SITE_PROFILE_COUNTERS` | Same file, private getter at line 224 and static at line 226 |
| `INVALIDATION_FAMILY_PROFILE_COUNTERS` | Same file, private getter at line 232 and static at line 234 |
| `SYMBOL_TIMING_PROFILE_COUNTERS` | Same file, private getter at line 240 and static at line 242 |
| `CALL_STABILITY_SOURCE_HEAD_STATS` and its source/timing stacks/depth | `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:20-27` |

The evaluation-cache `profile` module is private
(`evaluation_cache/mod.rs:7`), and its `source` module is also private
(`evaluation_cache/profile/mod.rs:8`). The getters and flags above are not
forwarded by the existing runtime-visible reexports. A module under
`runtime/profiling/tests/` cannot read them through the three allowed
runtime wiring files.

The existing native reporter is not an exhaustive alternative:
`evaluation_cache/profile/report.rs:72` takes 32 symbol rows, `:149` and
`:191` take 80 invalidation rows, `:270` takes 32 timing rows, and `:341`
takes 32 source/head rows. Parsing that output cannot meet the observer
contract, regardless of fixture size.

The smallest scope correction identified is permission for **test-only**
observer accessors in
`crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs` and their
**test-only** forwarding in
`crates/rum-runtime/src/runtime/evaluation_cache/mod.rs`. The existing
source-module visibility already permits its parent profile module to read
its maps. No counting or default-build change is proposed. Both files are
currently outside the explicit Scope, so neither was edited. This is an
access/scope blocker, not a finding that production changes are necessary.

### 3. The port — not started

Read-only inspection of the frozen tuple port reproduced:

```text
required=1115 bound=813 encoding=240
```

The 813 preexisting bindings reference 650 saved programs; 62 non-encoding
tuples remain unbound. These are inventory facts from brief-624, **not new
LP01 bindings or executed receipts**. No encoding-owned tuple was executed.
The 16 minima and 48 correctness plans were inspected but not ported or
executed in LP01. The 62 new bindings, producer/phase/request proofs, three
independent exact collections, and five negative controls are not
implemented. No row was adopted and no semantic difference was found by a
new binding.

### 4. The pinned losses — not started

No LP01 loss ledger was created. Newly pinned count is zero for every
LP02-LP08 owner. Brief-624's frozen losses were inspected without change.
No changed or unexpected counter difference was accepted as a baseline.

### 5. Gates, verbatim, and release-hash evidence

Commands ran serially, offline, with the required single build/test thread
and bounded wrapper. The base release command was:

```text
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-625/tmp CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-625/target-base-cli cargo build --release -p rum-cli
```

Saved output (`base-cli-build.log`), verbatim:

```text
    Finished `release` profile [optimized] target(s) in 1m 49s
```

Base artifact receipt (`base-cli-hashes.json`):

```text
sha256=405e8abd9539cbc30ca14c15bd148884e8eb10c2073ec855bdaf4d15dc995557
size=12857984
codegen_units=1
```

The unmodified oracle command was:

```text
python3 scripts/run-bounded.py --timeout 1500 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-625/tmp scripts/check-suspension-oracle.sh 1500
```

It exited 0. Default build (`base-oracle-default.log`), verbatim:

```text
continuation-slot fault-occurrence delta=0; observations=96246
test result: ok. 496 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.41s
```

No-default build (`base-oracle-no-default.log`), verbatim:

```text
continuation-slot fault-occurrence delta=0; observations=96246
test result: ok. 496 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.94s
```

All 894 fault-occurrence samples in the combined oracle log have delta 0.
These are the existing semantic oracle runs; they are not logical-profile
counter receipts. Those are the only `test result:` totals produced.

The following implementation gates **did not run**, because execution stopped
before source changes:

```text
cargo fmt --check
cargo test -p rum-runtime --lib profiling
scripts/check-suspension-oracle.sh 1500  [candidate run; base run passed above]
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
cargo test --workspace --no-fail-fast
```

The workspace run with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` did not run, so the
nine native tests have **no observed skip receipts** in this run. The candidate
release build and base/candidate hash comparison also did not run. The saved
base hash alone is **not** the no-cost proof. No timing/performance claim is
made.

### 6. Stop-loss counters, deviations and open question

```text
new_tuples_bound=0
logical_profile_receipts_executed=0
negative_controls_red=0
losses_pinned=0
compiling_port_attempts=0
consecutive_no_progress_port_attempts=0
```

The stop-loss threshold was not reached. The explicit scope STOP triggered
first. No git command, network access, unsafe addition, weakened check,
internal-encoding probe, or out-of-scope source change occurred. The only
scratch-location deviation is the mandated oracle script's fixed output
root, recorded above.

Next action for the director: resolve the observer scope for the two
`evaluation_cache` files named in section 2, then resume at step 2 using the
saved base receipts. The contract document and all port/adoption work remain
outstanding. Leave the tree uncommitted.


---

## Executing-agent continuation report — STOP at raw release hash

### 1. Files changed and inventory replay

**LP01 is not complete.** Implementation and all mandatory tests passed, but the raw release hashes differ. Read-only ELF attribution finds no changed default-build instructions: mimalloc embeds different C compilation times. The required raw hash gate remains failed, and the brief requires STOP.

All changed source/data paths below are relative to the repository root. The list was derived from the saved filesystem hashes; no git command was used.

| File | Purpose |
| --- | --- |
| `crates/rum-runtime/src/lib.rs` | Hidden cfg-only snapshot/reset forwarding. |
| `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs` | Cfg-only cache observer forwarding. |
| `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs` | Cfg-only shared/TLS map, flag, scalar and source-alias observation. |
| `crates/rum-runtime/src/runtime/mod.rs` | Cfg-only observer type and runtime forwarding. |
| `crates/rum-runtime/src/runtime/profiling/mod.rs` | Cfg-only observer module declaration and exports. |
| `crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs` | Exhaustive native snapshot and map/flag/ranking/isolation test. |
| `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs` | Cfg-only sharing telemetry access. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Replay only new UnknownConstruct rows from native discovery. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Replay only new UnknownConstruct rows from native discovery. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Replay only new UnknownConstruct rows from native discovery. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | Replay only new UnknownConstruct rows from native discovery. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Replay all 68 generated fingerprint cells, preserving every other field. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Replay C01/C02 certificate cells from both feature builds; no status or disposition change. |
| `crates/rum-stdlib/tests/suspension_oracle/descriptors.rs` | Counter context: native reset after setup, matching timeout/trace configuration, snapshot before probes. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/fingerprints.tsv` | Frozen plan data, fingerprints, phase-site catalog, fixture construction, producer proofs or the isolated five-mode driver. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/mod.rs` | Frozen plan data, fingerprints, phase-site catalog, fixture construction, producer proofs or the isolated five-mode driver. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/phase-sites.tsv` | Frozen plan data, fingerprints, phase-site catalog, fixture construction, producer proofs or the isolated five-mode driver. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/plans.rs` | Frozen plan data, fingerprints, phase-site catalog, fixture construction, producer proofs or the isolated five-mode driver. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/plans.tsv` | Frozen plan data, fingerprints, phase-site catalog, fixture construction, producer proofs or the isolated five-mode driver. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | Register logical-profile fixtures. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-bindings.tsv` | Independent frozen requirement/binding/schema/loss/execution collection. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json` | Independent frozen requirement/binding/schema/loss/execution collection. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-required.tsv` | Independent frozen requirement/binding/schema/loss/execution collection. |
| `crates/rum-stdlib/tests/suspension_oracle/manifest.rs` | Immutable original-column fingerprint and exact counter-context registration. |
| `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs` | Strict collections, key schema, source hashes, indexed-ledger decoding and exact counter comparison; reusable by standalone support clients. |
| `crates/rum-stdlib/tests/suspension_oracle/support/mod.rs` | Register generic logical counter comparison support. |
| `crates/rum-stdlib/tests/suspension_oracle/support/observe.rs` | Port declared-duration trace normalization; preserve other values and ordered events. |
| `docs/dev/codex/brief-625-lp01-logical-profiler-contract-and-oracle.md` | Append this execution report; preserve the director continuation and earlier report. |
| `docs/dev/profiling.md` | Approved target contract, equations, schema v2, epochs and owned staged losses. |

The four ownership partitions gained **117** verbatim discovery rows: 12 attributes, 43 captures, 55 macros and 7 storage/signature rows. No old row moved, disappeared, changed identity, status or disposition. The 79 host-payload rows, their ledger and the 59 self-reaching rows are unchanged. Every added row points to code changed here. The complete file/line/column/construct disclosure follows.

```text
UnknownConstruct	crates/rum-runtime/src/lib.rs	11	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/lib.rs	13	5	conditional public input signature test_logical_profile_snapshot
UnknownConstruct	crates/rum-runtime/src/lib.rs	16	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/lib.rs	18	5	conditional public input signature test_logical_profile_reset
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/mod.rs	63	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	382	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	480	9	unexpanded macro vec! (ends at 480:59)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	481	9	unexpanded macro vec! (ends at 481:15)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	485	9	unexpanded macro vec! (ends at 485:60)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	486	9	unexpanded macro vec! (ends at 486:15)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	490	9	unexpanded macro vec! (ends at 490:67)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	491	9	unexpanded macro vec! (ends at 491:15)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	495	9	unexpanded macro vec! (ends at 495:66)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	496	9	unexpanded macro vec! (ends at 496:15)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	500	9	unexpanded macro vec! (ends at 500:64)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	501	9	unexpanded macro vec! (ends at 501:15)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	505	9	unexpanded macro vec! (ends at 505:62)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	506	9	unexpanded macro vec! (ends at 506:15)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	525	12	unexpanded macro matches! (ends at 532:10)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	534	17	unexpanded macro format! (ends at 534:56)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	535	17	unexpanded macro vec! (ends at 535:29)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	536	17	unexpanded macro vec! (ends at 536:23)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	542	13	unexpanded macro format! (ends at 542:52)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	543	13	unexpanded macro vec! (ends at 543:48)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	545	12	unexpanded macro matches! (ends at 548:10)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	550	17	unexpanded macro format! (ends at 550:56)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	551	17	unexpanded macro vec! (ends at 551:34)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	552	17	unexpanded macro vec! (ends at 552:52)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	557	15	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	558	42	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	561	13	unexpanded macro format! (ends at 561:40)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	564	42	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	570	42	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	573	13	unexpanded macro format! (ends at 573:40)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	576	38	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	579	13	unexpanded macro format! (ends at 579:40)
UnknownConstruct	crates/rum-runtime/src/runtime/mod.rs	110	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/mod.rs	113	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/mod.rs	117	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	556	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	559	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	561	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	9	1	unresolved attribute/expansion derive (Debug , Default , Clone)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	11	18	rum_runtime::runtime::profiling::logical_contract::LogicalProfileSnapshot.logical: unresolved conditional storage: BTreeMap < String , Vec < u64 > >
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	12	20	rum_runtime::runtime::profiling::logical_contract::LogicalProfileSnapshot.durations: unresolved conditional storage: BTreeMap < String , Vec < u64 > >
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	13	20	rum_runtime::runtime::profiling::logical_contract::LogicalProfileSnapshot.telemetry: unresolved conditional storage: BTreeMap < String , String >
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	14	16	rum_runtime::runtime::profiling::logical_contract::LogicalProfileSnapshot.flags: unresolved conditional storage: BTreeMap < String , u8 >
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	16	15	rum_runtime::runtime::profiling::logical_contract::LogicalProfileSnapshot.maps: unresolved conditional storage: BTreeMap < String , usize >
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	20	9	unexpanded macro assert! (ends at 20:68)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	23	18	closure captures: |(k , v)| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	23	27	unexpanded macro format! (ends at 23:52)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	26	44	unexpanded macro format! (ends at 26:63)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	29	9	unexpanded macro assert! (ends at 32:10)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	93	9	unexpanded macro vec! (ends at 93:47)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	94	9	unexpanded macro vec! (ends at 94:15)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	102	9	unexpanded macro vec! (ends at 102:39)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	103	9	unexpanded macro vec! (ends at 103:15)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	281	31	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	282	33	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	283	39	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	284	34	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	285	33	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	286	38	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	287	32	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	288	38	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	290	15	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	292	15	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	293	31	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	294	37	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	296	15	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	297	35	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	298	41	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	299	38	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	300	32	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	301	39	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	302	36	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	303	31	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	304	36	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	306	15	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	307	41	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	308	36	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	309	31	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	311	52	unexpanded macro format! (ends at 311:79)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	313	36	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	316	13	unexpanded macro format! (ends at 316:40)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	319	32	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	321	53	unexpanded macro format! (ends at 321:80)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	323	35	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	326	13	unexpanded macro format! (ends at 326:40)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	329	42	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	332	13	unexpanded macro format! (ends at 332:40)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	335	32	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	339	35	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	343	42	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	349	36	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	351	23	unexpanded macro format! (ends at 355:14)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	357	17	unexpanded macro format! (ends at 357:48)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	362	17	unexpanded macro vec! (ends at 370:18)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	371	17	unexpanded macro vec! (ends at 371:44)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	375	39	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	378	17	unexpanded macro format! (ends at 383:18)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	384	17	unexpanded macro vec! (ends at 384:43)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	385	17	unexpanded macro vec! (ends at 385:30)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	389	36	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	392	17	unexpanded macro format! (ends at 397:18)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	398	17	unexpanded macro vec! (ends at 405:18)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	406	17	unexpanded macro vec! (ends at 411:18)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	415	31	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	418	17	unexpanded macro format! (ends at 423:18)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	424	17	unexpanded macro vec! (ends at 424:43)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	425	17	unexpanded macro vec! (ends at 425:44)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	429	32	closure captures: |s| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	432	17	unexpanded macro format! (ends at 436:18)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	437	17	unexpanded macro vec! (ends at 437:54)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	438	17	unexpanded macro vec! (ends at 438:30)
UnknownConstruct	crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs	446	9	unexpanded macro format! (ends at 446:59)
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/binding_substitution.rs	896	1	unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots"))
```

All 68 local-contract rows changed only their generated final fingerprint cell. C01 and C02 changed only certificate columns 5 and 6. Both generated acceptance reviews had zero rejections, three production instance marks and four separate probe marks. Exact old/new cells and row IDs are in `.temp/brief-625/maintenance-row-changes.json`; the native review files and regeneration logs are retained. No frame/root closed-list row was needed.

File-size assessment: the existing binding-substitution implementation remains in the split-plan range; this brief appends only a cfg-only reader and preserves production bodies/positions for the byte-identity gate. Its future boundary is substitution transforms versus diagnostic readers. The profiling facade remains a cohesive key/stat model; new observation lives in its test module. Cache profiling retains its cfg-only reader beside the private storage; a future extraction can place that reader in a test child module. Test helper files remain grouped by collection/comparison versus fixture/producer responsibilities. No production split was authorized here.

### 2. Observer and default-build isolation

The native observer reads all 24 lexical profiler maps, all four shared cache maps, the cache source/head TLS map, stacks/depths, flags, scalar counters, source aliases, sharing telemetry and the existing representation snapshot. Every map, including an empty map, is recorded; there is no top-N projection. Logical vectors preserve phase/source, builtin arity and definition kind/symbol/index. Durations, `input_required`, physical resume entries and raw execution storage are kept outside logical comparison.

All runtime additions are under `cfg(any(test, feature = "test-continuation-slots"))`. Existing field shapes, source visibility and counting code are unchanged in the default build. Hidden native reset follows setup; snapshot immediately follows the subject and precedes sentinels and projections. The focused observer test verifies map/flag coverage, more-than-ranked-display collection and read isolation. The final one-codegen-unit release .text is byte-identical to the accepted base, but the complete binary hash differs because of native dependency build-time metadata (section 5). This is not accepted as a replacement for the specified raw hash proof.

### 3. Port, bindings and exactness

```text
frozen required tuples=1115
original bindings=813 over 650 programs
new non-encoding bindings=62 over 82 programs
active bound/executed tuples per flag/build cell=875
encoding-owned tuples=240 (G13; not executed)
independent minima=16
independent correctness fixtures=48
programs per flag/build cell=796
flag modes=5; builds=2; executed tuple receipts=8750
```

All original sixteen requirement columns remain byte-equivalent and are independently fingerprinted (`fnv1a64:b41ac6a5dbf0e9a4`). Required, bound and executed collections have independent frozen data, exact source fingerprints and multiplicity checks. The indexed loss ledger contains separate schema and executed-receipt records, and each live execution also records its concrete build identity. All 714 retained programs reproduce both saved counter vectors exactly; no symmetric offsets or counter normalization are accepted.

For the 62 new bindings, the table below identifies every frozen row, producer, phase and program/request plan. Each cycle tuple requires both matching and nonmatching programs. Ground/fast/compiled definition decisions are checked using dispatch counters and actual rule traces; P0509 uses the OwnValue symbol path, and P0817 uses the bypassing-Throw form-resolution arm with incoming Throw sampling. Exact request kinds, prompts and ordinals are checked by the unchanged planned driver. All 82 programs have `LP01 ATTRIBUTED` receipts in both build logs. Decoded exact `.rum` source/setup and request JSON are under `.temp/brief-625/new-bindings/`; the readable table is `.temp/brief-625/new-bindings-review.md`.

| Frozen line | Row | Variant | Context | Producer | Phase | Programs / requests |
| --- | --- | --- | --- | --- | --- | --- |
| 72 | P0256 | Middle | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:231 | definition-rule / down_value_rhs_eval | [625.bind.72.matching](new-bindings/625.bind.72.matching.rum) (1 requests); [625.bind.72.nonmatching](new-bindings/625.bind.72.nonmatching.rum) (1 requests) |
| 73 | P0256 | Twice | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:231 | definition-rule / down_value_rhs_eval | [625.bind.73.matching](new-bindings/625.bind.73.matching.rum) (2 requests); [625.bind.73.nonmatching](new-bindings/625.bind.73.nonmatching.rum) (2 requests) |
| 74 | P0256 | Error | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:231 | definition-rule / down_value_rhs_eval | [625.bind.74.matching](new-bindings/625.bind.74.matching.rum) (1 requests); [625.bind.74.nonmatching](new-bindings/625.bind.74.nonmatching.rum) (1 requests) |
| 75 | P0256 | AlreadyResumed | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:231 | definition-rule / down_value_rhs_eval | [625.bind.75.matching](new-bindings/625.bind.75.matching.rum) (3 requests); [625.bind.75.nonmatching](new-bindings/625.bind.75.nonmatching.rum) (3 requests) |
| 76 | P0256 | Nested | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:231 | definition-rule / down_value_rhs_eval | [625.bind.76.matching](new-bindings/625.bind.76.matching.rum) (2 requests); [625.bind.76.nonmatching](new-bindings/625.bind.76.nonmatching.rum) (2 requests) |
| 88 | P0259 | Twice | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.88.terminal](new-bindings/625.bind.88.terminal.rum) (2 requests) |
| 89 | P0259 | Error | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.89.terminal](new-bindings/625.bind.89.terminal.rum) (1 requests) |
| 90 | P0259 | AlreadyResumed | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.90.terminal](new-bindings/625.bind.90.terminal.rum) (3 requests) |
| 91 | P0259 | Nested | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.91.terminal](new-bindings/625.bind.91.terminal.rum) (2 requests) |
| 143 | P0259 | Twice | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.143.terminal](new-bindings/625.bind.143.terminal.rum) (2 requests) |
| 144 | P0259 | Error | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.144.terminal](new-bindings/625.bind.144.terminal.rum) (1 requests) |
| 145 | P0259 | AlreadyResumed | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.145.terminal](new-bindings/625.bind.145.terminal.rum) (3 requests) |
| 146 | P0259 | Nested | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.146.terminal](new-bindings/625.bind.146.terminal.rum) (2 requests) |
| 152 | P0259 | Middle | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.152.matching](new-bindings/625.bind.152.matching.rum) (1 requests); [625.bind.152.nonmatching](new-bindings/625.bind.152.nonmatching.rum) (1 requests) |
| 153 | P0259 | Twice | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.153.matching](new-bindings/625.bind.153.matching.rum) (2 requests); [625.bind.153.nonmatching](new-bindings/625.bind.153.nonmatching.rum) (2 requests) |
| 154 | P0259 | Error | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.154.matching](new-bindings/625.bind.154.matching.rum) (1 requests); [625.bind.154.nonmatching](new-bindings/625.bind.154.nonmatching.rum) (1 requests) |
| 155 | P0259 | AlreadyResumed | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.155.matching](new-bindings/625.bind.155.matching.rum) (3 requests); [625.bind.155.nonmatching](new-bindings/625.bind.155.nonmatching.rum) (3 requests) |
| 156 | P0259 | Nested | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:334 | definition-rule / down_value_rhs_eval | [625.bind.156.matching](new-bindings/625.bind.156.matching.rum) (2 requests); [625.bind.156.nonmatching](new-bindings/625.bind.156.nonmatching.rum) (2 requests) |
| 168 | P0266 | Twice | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.168.terminal](new-bindings/625.bind.168.terminal.rum) (2 requests) |
| 169 | P0266 | Error | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.169.terminal](new-bindings/625.bind.169.terminal.rum) (1 requests) |
| 170 | P0266 | AlreadyResumed | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.170.terminal](new-bindings/625.bind.170.terminal.rum) (3 requests) |
| 171 | P0266 | Nested | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.171.terminal](new-bindings/625.bind.171.terminal.rum) (2 requests) |
| 223 | P0266 | Twice | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.223.terminal](new-bindings/625.bind.223.terminal.rum) (2 requests) |
| 224 | P0266 | Error | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.224.terminal](new-bindings/625.bind.224.terminal.rum) (1 requests) |
| 225 | P0266 | AlreadyResumed | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.225.terminal](new-bindings/625.bind.225.terminal.rum) (3 requests) |
| 226 | P0266 | Nested | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.226.terminal](new-bindings/625.bind.226.terminal.rum) (2 requests) |
| 232 | P0266 | Middle | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.232.matching](new-bindings/625.bind.232.matching.rum) (1 requests); [625.bind.232.nonmatching](new-bindings/625.bind.232.nonmatching.rum) (1 requests) |
| 233 | P0266 | Twice | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.233.matching](new-bindings/625.bind.233.matching.rum) (2 requests); [625.bind.233.nonmatching](new-bindings/625.bind.233.nonmatching.rum) (2 requests) |
| 234 | P0266 | Error | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.234.matching](new-bindings/625.bind.234.matching.rum) (1 requests); [625.bind.234.nonmatching](new-bindings/625.bind.234.nonmatching.rum) (1 requests) |
| 235 | P0266 | AlreadyResumed | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.235.matching](new-bindings/625.bind.235.matching.rum) (3 requests); [625.bind.235.nonmatching](new-bindings/625.bind.235.nonmatching.rum) (3 requests) |
| 236 | P0266 | Nested | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/definitions/application/down_values.rs:467 | definition-rule / down_value_rhs_eval | [625.bind.236.matching](new-bindings/625.bind.236.matching.rum) (2 requests); [625.bind.236.nonmatching](new-bindings/625.bind.236.nonmatching.rum) (2 requests) |
| 248 | P0314 | Twice | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.248.terminal](new-bindings/625.bind.248.terminal.rum) (2 requests) |
| 249 | P0314 | Error | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.249.terminal](new-bindings/625.bind.249.terminal.rum) (1 requests) |
| 250 | P0314 | AlreadyResumed | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.250.terminal](new-bindings/625.bind.250.terminal.rum) (3 requests) |
| 251 | P0314 | Nested | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.251.terminal](new-bindings/625.bind.251.terminal.rum) (2 requests) |
| 303 | P0314 | Twice | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.303.terminal](new-bindings/625.bind.303.terminal.rum) (2 requests) |
| 304 | P0314 | Error | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.304.terminal](new-bindings/625.bind.304.terminal.rum) (1 requests) |
| 305 | P0314 | AlreadyResumed | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.305.terminal](new-bindings/625.bind.305.terminal.rum) (3 requests) |
| 306 | P0314 | Nested | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:106 | definition-rule / up_value_rhs_eval | [625.bind.306.terminal](new-bindings/625.bind.306.terminal.rum) (2 requests) |
| 322 | P0315 | Middle | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.322.terminal](new-bindings/625.bind.322.terminal.rum) (1 requests) |
| 323 | P0315 | Twice | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.323.terminal](new-bindings/625.bind.323.terminal.rum) (2 requests) |
| 324 | P0315 | Error | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.324.terminal](new-bindings/625.bind.324.terminal.rum) (1 requests) |
| 325 | P0315 | AlreadyResumed | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.325.terminal](new-bindings/625.bind.325.terminal.rum) (3 requests) |
| 326 | P0315 | Nested | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.326.terminal](new-bindings/625.bind.326.terminal.rum) (2 requests) |
| 377 | P0315 | Middle | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.377.terminal](new-bindings/625.bind.377.terminal.rum) (1 requests) |
| 378 | P0315 | Twice | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.378.terminal](new-bindings/625.bind.378.terminal.rum) (2 requests) |
| 379 | P0315 | Error | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.379.terminal](new-bindings/625.bind.379.terminal.rum) (1 requests) |
| 380 | P0315 | AlreadyResumed | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.380.terminal](new-bindings/625.bind.380.terminal.rum) (3 requests) |
| 381 | P0315 | Nested | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/definitions/application/up_values.rs:186 | definition-rule / up_value_rhs_eval | [625.bind.381.terminal](new-bindings/625.bind.381.terminal.rum) (2 requests) |
| 748 | P0509 | Twice | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.748.terminal](new-bindings/625.bind.748.terminal.rum) (2 requests) |
| 749 | P0509 | Error | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.749.terminal](new-bindings/625.bind.749.terminal.rum) (1 requests) |
| 750 | P0509 | AlreadyResumed | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.750.terminal](new-bindings/625.bind.750.terminal.rum) (3 requests) |
| 751 | P0509 | Nested | error_before_tail.last-pause-throw | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.751.terminal](new-bindings/625.bind.751.terminal.rum) (2 requests) |
| 763 | P0509 | Twice | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.763.terminal](new-bindings/625.bind.763.terminal.rum) (2 requests) |
| 764 | P0509 | Error | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.764.terminal](new-bindings/625.bind.764.terminal.rum) (1 requests) |
| 765 | P0509 | AlreadyResumed | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.765.terminal](new-bindings/625.bind.765.terminal.rum) (3 requests) |
| 766 | P0509 | Nested | return_boundary.last-pause-return | crates/rum-runtime/src/runtime/eval_core/mod.rs:146 | runtime / evaluate_inner_unstable.symbol | [625.bind.766.terminal](new-bindings/625.bind.766.terminal.rum) (2 requests) |
| 942 | P0817 | Middle | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:131 | runtime / catch.resume_after_body | [625.bind.942.matching](new-bindings/625.bind.942.matching.rum) (1 requests); [625.bind.942.nonmatching](new-bindings/625.bind.942.nonmatching.rum) (1 requests) |
| 943 | P0817 | Twice | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:131 | runtime / catch.resume_after_body | [625.bind.943.matching](new-bindings/625.bind.943.matching.rum) (2 requests); [625.bind.943.nonmatching](new-bindings/625.bind.943.nonmatching.rum) (2 requests) |
| 944 | P0817 | Error | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:131 | runtime / catch.resume_after_body | [625.bind.944.matching](new-bindings/625.bind.944.matching.rum) (1 requests); [625.bind.944.nonmatching](new-bindings/625.bind.944.nonmatching.rum) (1 requests) |
| 945 | P0817 | AlreadyResumed | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:131 | runtime / catch.resume_after_body | [625.bind.945.matching](new-bindings/625.bind.945.matching.rum) (3 requests); [625.bind.945.nonmatching](new-bindings/625.bind.945.nonmatching.rum) (3 requests) |
| 946 | P0817 | Nested | same_type_error_finish.explicit-cycle-controls-required | crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs:131 | runtime / catch.resume_after_body | [625.bind.946.matching](new-bindings/625.bind.946.matching.rum) (2 requests); [625.bind.946.nonmatching](new-bindings/625.bind.946.nonmatching.rum) (2 requests) |

All five negative controls are red: removal of an actual observed logical key, duplicate real requirement tuple, removal of an actual execution receipt, a Throw that prevents the planned second pause, and a malformed real requirement row. No existing `Checks` flag, fault-absence check, retention reject or documented test was weakened. Development-only describe/partial-run shortcuts were removed from the registered test; every counter run requires all 875 receipts and its frozen ledger.

### 4. Pinned losses

The unchanged implementation has **118,991** pinned phase/site/route cells across flag modes. The same exact vectors are independently checked in both builds. Full producer, consumer-route identity, phase-site/writer locations, owner and resumed/queued vectors are in `crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-losses.json`; its indexed format is documented in `docs/dev/profiling.md`. An expanded one-cell-per-line view is `.temp/brief-625/pinned-losses-expanded.jsonl`; the 74 phase/site groups are in `.temp/brief-625/loss-summary.tsv`. No unexpected difference was admitted. P0817/P0818 incoming-Throw sampling remains untouched.

| Mode | LP03 | LP04 | LP05 | LP06 | LP07 | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| disabled | 0 | 0 | 0 | 0 | 0 | 0 |
| lexical | 14376 | 942 | 1098 | 2317 | 168 | 18901 |
| timing | 14376 | 1300 | 1098 | 5964 | 168 | 22906 |
| cache | 25012 | 2136 | 4982 | 3136 | 190 | 35456 |
| all | 25012 | 2494 | 4982 | 9050 | 190 | 41728 |
| Total | 78776 | 6872 | 12160 | 20467 | 716 | 118991 |

LP02 and LP08 have no invented loss cells: they own the future engine/lifecycle and schema/promotion obligations. Remaining cells stay staged under their owning adoption steps. The developer contract records logical/telemetry separation, lifecycle equations, process-wide epochs, diagnostic memory isolation and additive schema v2 as targets.

### 5. Mandatory gates and release proof — verbatim

Every command ran serially through `scripts/run-bounded.py`, with `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-625/tmp`. Each individual bound is 900 seconds, except oracle 1500 per build and workspace 3600. All mandatory test gates ran on the final implementation and passed. Both release builds ran; the raw equality gate failed. No required command remains unrun, and equality is not claimed. Exact logs are under `.temp/brief-625/`.

`cargo fmt --check` (`gate-fmt-final.log`), exit 0:

```text
(no output; exit 0)
```

`cargo test -p rum-runtime --lib profiling` (`gate-runtime-profiling.log`), exit 0:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
```

`scripts/check-suspension-oracle.sh 1500` (`gate-oracle-final.log`), exit 0:

```text
disabled: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 3.53s
lexical: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 12.99s
timing: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 14.75s
cache: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 15.94s
all: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 18.62s
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.14s
disabled: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 3.41s
lexical: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 12.90s
timing: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 14.68s
cache: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 16.02s
all: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 498 filtered out; finished in 18.67s
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.08s
```

`cargo test -p rum-stdlib --lib ownership_graph` (`gate-ownership.log`), exit 0:

```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 70.55s
```

`cargo test -p rum-stdlib --lib local_production_acceptance` (`gate-acceptance-default.log`), exit 0:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.51s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance` (`gate-acceptance-no-default.log`), exit 0:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.32s
```

`cargo test -p rum-stdlib --lib production_rejection_census_is_complete` (`gate-rejection-census.log`), exit 0:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.91s
```

`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast` (`gate-workspace.log`), exit 0:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.63s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 1185 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 34.41s
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1283.45s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.46s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 499 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.65s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 277.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 52.59s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.43s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.19s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Oracle fault-occurrence receipts and tuple counts, verbatim:

```text
disabled: continuation-slot fault-occurrence delta=0; observations=9552
disabled: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=0
lexical: continuation-slot fault-occurrence delta=0; observations=9552
lexical: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=18901
timing: continuation-slot fault-occurrence delta=0; observations=9552
timing: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=22906
cache: continuation-slot fault-occurrence delta=0; observations=9552
cache: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=35456
all: continuation-slot fault-occurrence delta=0; observations=9552
all: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=41728
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=96248
disabled: continuation-slot fault-occurrence delta=0; observations=9552
disabled: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=0
lexical: continuation-slot fault-occurrence delta=0; observations=9552
lexical: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=18901
timing: continuation-slot fault-occurrence delta=0; observations=9552
timing: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=22906
cache: continuation-slot fault-occurrence delta=0; observations=9552
cache: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=35456
all: continuation-slot fault-occurrence delta=0; observations=9552
all: LP01 required=1115 bound=875 executed=875 encoding-owned=240 programs={"correctness": 48, "frozen": 650, "minimum": 16, "new": 82} counter-losses=41728
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=96248
```

All nine native ZeroMQ cases are **skips**, not native execution credit. A focused `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter --lib native_zeromq_backend -- --nocapture` replay makes the skip receipts explicit; the director owns native execution. Rust reports these early returns as passing tests.

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
```

Release builds use `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p rum-cli` from the same source path, into `target-base-cli` and `target-cand-cli` under this brief's scratch. Actual binary bytes were rehashed after the candidate build:

```text
base sha256=405e8abd9539cbc30ca14c15bd148884e8eb10c2073ec855bdaf4d15dc995557
candidate sha256=5634c5f9972fddd9e0675f27a1f4e114fda781202af065c5ca02067aa490a8c8
base bytes=12857984
candidate bytes=12857984
RAW_RELEASE_HASH_EQUALITY_FAILED
```

The native dependency `libmimalloc-sys 0.1.49` embeds `__DATE__` and `__TIME__` in `c_src/mimalloc/v3/src/options.c:236` (installed registry source). The base contains `00:40:16`; the candidate contains `02:20:49`, both on Oct 20 2026. The complete ELF difference is 23 bytes: four ASCII timestamp digits in `.rodata` at relative range 13320–13328, plus 19 GNU build-ID bytes. Every `.text` byte and every other section is identical. `.temp/brief-625/binary-diff.json` records the exact section/range comparison. No default-build code change was found. No binary was normalized, patched or rebaselined; no build recipe was changed to force equality. The director must settle the reproducible native-dependency build recipe before this gate can pass.

The complete verbatim `test result:` history, including the accepted 496/496 base runs, compilation/collection checks and expected maintenance reds, is `.temp/brief-625/all-test-results.txt`. Final passing gate totals above are not substituted for failed-run evidence.

### 6. Stop-loss, deviations and remaining work

```text
new_tuples_bound=62
total_active_bound=875
executed_flag_build_tuple_receipts=8750
negative_controls_red=5
pinned_loss_cells=118991
consecutive_no_progress_attempts=0
```

The first cycle plan wrote to the log and changed the evaluation epoch, causing an extra queued input; the pause gate rejected it. The cycle plans were corrected and refingerprinted before retry; no planned pause or key was removed. Initial counter-context observations used different timeout/trace setup from the saved harness; restoring that setup reproduces all 714 exact vector pairs, without accepting offsets. A workspace compile check caught a Fixture dependency in shared support; the helper was moved into fixtures, workspace compilation then passed, and all source-bound fingerprints/certificates were regenerated. These development failures and the expected stale-inventory/certificate reds are retained in the raw logs.

The stop-loss threshold was not reached. Every compiling port attempt either added bindings, executed attributed receipts or produced the required red controls; compile-error repair is separately permitted by TEMPLATE.md. No semantic difference was uncovered by a new binding, and no prerequisite semantic repair is pending. No production/default-build change, new unsafe, internal-encoding execution, network access, git command, status/disposition weakening or unrelated edit was made.

Agent-created scratch stays under `.temp/brief-625/`. Existing mandated scripts/tests also use their fixed scratch roots (`.temp/suspension-oracle/`, `.temp/brief-546/certificates/`, `target/brief-544/` and existing workspace-test scratch); their relevant receipts were copied here. Brief-624's scene remains read-only. No temporary probe/build artifact was added to source.

LP01 implementation is ready for review, but LP01 acceptance is blocked by the raw release-hash requirement. Production logical-counting repairs and schema-v2 publication remain intentionally staged for LP02–LP08. The tree is left uncommitted for the director gate. The release-hash STOP is unresolved.

## Director gate record (2026-10-20) — LP01 accepted

**Runs.**

| Run | Session | Tokens | Result |
| --- | --- | ---: | --- |
| Run 1 | `01a15506-…` | 147k | Stopped correctly on scope. My list missed the evaluation-cache profiler storage. |
| Continuation 1 | fork `01a1550c-…` | 751k | Scope widened by class. It implemented everything, then stopped at the raw release-hash gate. |

**The release-identity gate, settled by the director**
(`.temp/director/gate-625/hash2.sh`).

- **Why the worker's gate failed.** The worker's two builds differed only
  in mimalloc's embedded `__DATE__`/`__TIME__` (four `.rodata` digits) and
  the GNU build-ID.
- **Settling it.**
  - I rebuilt the base from an export of `4b891290` and the candidate from
    the working tree.
  - Both builds used one codegen unit and `SOURCE_DATE_EPOCH=1792400000`,
    with every source and target path remapped (`--remap-path-prefix`,
    `-ffile-prefix-map`).
  - The two `rum-cli` binaries are **byte-identical** (`ebc15b75…`). The
    observer reaches no default build.
- **The calibration.** The same comparison without the remap reports
  1.6 M differing bytes, which come from the embedded paths alone. So the
  instrument can tell builds apart.
- **Isolation.** `test-continuation-slots` is enabled only from
  `rum-stdlib`'s dev-dependencies, so it does not reach a normal build.

**Gate on the final tree** (`.temp/director/gate-625/`, `chain.txt`):

- **Formatting and units.** `cargo fmt --check` is clean. typed 24,
  frame_root_trace 15, resume_evaluation_context 11, depth 1.
- **Oracle.** It passes in both builds, fault delta 0 (96 248
  observations). The logical-profile counter runs, ten flag-mode and build
  cells of 9 552 observations each, also have delta 0. No warnings.
- **Census.** 610/99/55/50, unchanged. Compile controls pass.
- **Inventories.** The replays leave every inventory byte-identical.
  Ownership 31, with 117 new `UnknownConstruct` rows, all on cfg-gated code
  this step added and none on a host payload. Local acceptance passes in
  both builds; rejection census passes.
- **Workspace.** 3932/0/59, run natively, with all nine ZeroMQ tests run.
  The warnings are the four pre-existing ones.

**Accepted.**

- The contract in `docs/dev/profiling.md`.
- The cfg-only exhaustive observer, in seven runtime files, every addition
  cfg-gated.
- The counter oracle:
  - 875 bound tuples, 62 of them new, each attributed to its producer and
    phase;
  - 8 750 executed receipts over five flag modes and two builds;
  - the 240 encoding-owned tuples visible and not executed;
  - five red negatives.
- **118 991 pinned loss cells** in 74 phase and site groups, owned by
  LP03-LP07, with none unexpected.

**Lesson.** A release-identity gate must fix every non-semantic input
before it compares bytes: build time, source paths and target paths. Raw
hashes of two builds at different times or paths cannot be equal, so a
gate that demands it measures the build environment.
