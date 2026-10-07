# brief-594 — G11 step A: shared accumulator chains complete without a uniqueness assumption

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-593 found **three guest-reachable kernel panics** of one class: a
consumer finalizes an `AccumulatorChain` with `Arc::try_unwrap(...)` plus a
panic, inside a frame's `resume`, while that still-rooted frame legitimately
shares an older node (the suspension driver drops a frame only after its
`resume` returns, `runtime/suspension/core.rs:777-786`). Witnesses, each
caught once: `.temp/brief-593/{probe.rs,probe-test.log}`.

| | Source | Panic |
|---|---|---|
| C1 | `TestReport[{TestCreate[InputString["report:1"],"a"],TestCreate[InputString["report:2"],"b"]},"TestsSucceededCount"]`, answers `a`, `b` (queued: `2`) | `debug_tools/test_report/report.rs:116` |
| C2 | with ``Global`assoc=<|"a"-><|"b"->0|>|>``: ``Global`assoc[InputString["assoc:1"],InputString["assoc:2"]]=9``, answers `a`, `b` (queued: `9`) | `kernel_data/association_data.rs:661` |
| C3 | `DeleteCases[{1,2},_?Function[x,(InputString["delete"];False)]]`, answered `a`, `b` (queued: `{1, 2}`) | `kernel_data/rewrite_data/suspended_state.rs:155` |

Repair the **class**, not three sites: one private completion cursor over
`AccumulatorChain<N>` (`N: Clone`) that moves a node when unique and
shallow-clones it when shared, detaching the previous link into the cursor
before handing out the payload — clients never hold a raw `Arc` they must
assert unique. Route the three consumers and `Table`'s existing safe
move-or-clone loop (`branching_loops/iteration/mod.rs:94-103`) through it.

brief-593's report (§4, §5) and `.temp/brief-593/draft-A-shared-accumulator-completion.md`
are the raw material; **this brief governs** where they differ.

## Context pointers

- brief-593 and its report; the 32-site classification
  `.temp/brief-593/uniqueness-classification.tsv` (25 unique by construction,
  4 safe fallbacks, these 3).
- `runtime/accumulator_chain.rs:13-98` (wrapper, link, cold iterative drain);
  the participation gate `rum-stdlib/src/tests/ownership_graph/accumulator_coverage.rs`
  and the destruction-depth certificates; `runtime/suspension/core.rs:764-800`.
- `debug_tools/test_report/report.rs:90-127, 462, 502-507`;
  `kernel_data/association_data.rs:612-671, 847-855` and
  `association_data/resume_frames.rs:73-78`;
  `kernel_data/rewrite_data/suspended_state.rs:103-167`,
  `cases/delete/traversal.rs:514-741`, `cases/delete/resume_frames.rs:443-465`.
- The oracle `crates/rum-stdlib/tests/suspension_oracle/` (runner,
  `descriptors.rs`, `fixtures/`, `inventory/effects.tsv`).

## Scope

**Objective:** the completion cursor, the four consumers routed through it,
the regression descriptors, and the standing and instruction gates.

**Edit surface:** `runtime/accumulator_chain.rs` and one cohesive child
`accumulator_chain/completion.rs` (private to the runtime; sibling tests
allowed); `debug_tools/test_report/report.rs`;
`kernel_data/association_data.rs`; `kernel_data/rewrite_data/suspended_state.rs`;
`branching_loops/iteration/mod.rs` (only to use the shared helper, its tests
unchanged in meaning); in the oracle, one shared-completion fixture module and
its wiring, the role factories needed, and `inventory/effects.tsv` **only** to
attach supplemental descriptors to the existing `Set.target`,
`DeleteCases.patternTest`, `DeleteCases.guard` and the TestReport-adjacent
entries already drained — **the four G11 entries stay owed** (step B attaches
them).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **Authorized
inventory transitions, by name:** removing the three panic closures removes
their `UnknownConstruct` rows — the `unwrap_or_else` closure and its
`panic!` macro at `report.rs:116`, `association_data.rs:661` and
`suspended_state.rs:155` (in `unknown_captures.tsv` and `unknown_macros.tsv`);
if `Clone` is added to the three node types by derive, their existing
node-derive attribute rows change. **STOP** on any other row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code. A new frame type, if any, enters rum-runtime's frame
lists.

**Fence:** suspension rooting and finalizer order unchanged — never drop the
active frame early or unroot it to manufacture uniqueness; `Expr`/`Symbol`
representation, catalog identities and the owed ledger unchanged; the 25
class-(a) sites keep their post-retirement protocol and do not gain `Clone`;
no leases; `scripts/` unchanged. No edits to `AGENTS.md`, `TODO.md`, root
`PROGRESS.md`, earlier briefs or docs.

**Disclosure duty:** every changed function and type, every inventory row,
every measured label, any adjacent defect.

## Deliverable

The cursor owns its remaining spine through `AccumulatorChain` (so an
early-dropped cursor still drains iteratively); each step moves a unique node
or shallow-clones a shared one (payloads `Expr`/`SharedArgs` stay shared),
detaches the previous link, and returns the payload-bearing node. No
uniqueness assertion, no `Arc::strong_count` reasoning. Append stays O(1),
completion O(n); nothing is flattened at a pause. Nodes keep no manual `Drop`;
every owning previous link stays wrapped; the participation gate and the
destruction-depth certificates still pass (they prove link destruction, not
`Expr`-tree destruction).

## Tests

Red first, recorded before any production edit — each panic witness run
**once** through the oracle runner in the Rust harness (queued control first,
then the caught panic); no repeat, no abort probe. Then, green through
`Checks::ALL` with exact prompts, queue exhaustion, state, fresh requests and
lifecycle at the first and second relevant pause: TestReport input,
expected-output and `SameTest` phases each suspending in **two cases**; two
selectors in one association `Set` target; `DeleteCases` rejecting across two
siblings, mixed keep/delete, association children and early limit completion
(the drained descriptors split Decline and Twice — their cross product is what
reaches the assertion). Helper tests: unique move versus shared clone, a
retained snapshot preserved, early cursor drop, shared-tail drop on small
chains. A `rum-host` contract test that these programs complete and leave the
session usable.

## rum / Rust pitfalls (read twice)

1. Do not turn the panic into an `EvalError`, skip a value, or clone the whole
   buffer at each pause. A complete-looking result that lost a prefix is red.
2. No `cfg(test)` completion logic; no `#[should_panic]` blessing a kernel
   panic. Tests use the production helper.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: panic witnesses retired (of 3), regressions
   passing, consumers migrated, regressing labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-594/tmp …`. Bounds: focused 300 s; the oracle 900 s
per build; runtime, ownership and retention 300 s; a one-unit release build
900 s; a metric process 150 s; `cargo test --workspace --no-fail-fast` 3 000 s.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-host
scripts/check-suspension-oracle.sh 900
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Instruction gate.** **Before any production edit**, build the base from the
repository tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into
`.temp/brief-594/base/target-default` and `target-alloc` (`--features
rum-cli/alloc-count`, two arguments) and copy the binaries with hashes; then
candidates the same way in fresh directories. Panel (scratch workloads, each
result checked first): `DeleteCases` wide no-match, wide association no-match
and small leaf delete; association `Set` single-key incremental and two-key;
`Table` count, iterator and small; an ordinary two-case `TestReport`; the
controls `dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). **Noise floor first** (base against itself, two
pairs); a label regresses when both candidate samples exceed `max(+0.05%,
floor)`. Allocation counts, then two paired samples of `taskset -c 2 perf stat
-x, -e cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output first. No timing suite. At most eight
measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-594/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the oracle gate script, the release `rum-cli` on passing programs,
  `taskset`/`perf`, `valgrind --tool=callgrind`. No probe that aborts a
  process; each panic witness runs once.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files, functions and types; split assessment.
2. Verbatim red (once) and green output, the oracle in both builds, ownership,
   certificates and the broad suites; every inventory row change.
3. Panics retired (of 3); the noise floors and every measured row.
4. The four G11 entries still owed; stop-loss counters; open questions.

---
