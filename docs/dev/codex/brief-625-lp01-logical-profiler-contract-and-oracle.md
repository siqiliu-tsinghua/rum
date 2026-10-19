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
