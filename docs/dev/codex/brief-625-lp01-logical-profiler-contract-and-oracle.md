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
