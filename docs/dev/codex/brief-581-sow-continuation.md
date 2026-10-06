# brief-581 — G7 step A: Sow resumes its tag matching before it commits

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`Sow[value, tag]` matches its tag against every enclosing `Reap` pattern and
then records the value into each accepted bucket. A pattern test that asks for
input resumes past that work: `Reap[Do[Sow[x, x], {x, {1, 2, 3}}],
_?(…InputString…)]` resumes to `{Null, {{1}, {3}}}` where the queued path
returns `{Null, {{1}, {2}, {3}}}`. Repair P0125
(`crates/rum-runtime/src/runtime/debug_tools/reap.rs`, `eval_sow` →
`match_expr_owned`) per brief-580's design: finish every scope and pattern
match — resumed, accepted or declined — then record the value exactly once into
each accepted bucket and return the original value; nothing is committed
before all matches finish. Lost continuations **14 -> 13**; the trace filters
are step B.

brief-580's report and `.temp/brief-580/draft-A-sow.md` are the raw material;
**this brief governs** where they differ.

## Director rulings on brief-580's questions

1. The two-step split and the flat filter choice are approved (step B).
2. I01-I07 are authorized as named in `.temp/brief-580/inventory-plan.md`;
   for this step: I01 (P0125), I02's `SowMatchState` and
   `SowMatchSuspensionFrame`, I03's step-A position moves and additions, I04
   (standing replay), I05 (Sow layout and tests). **Not I06's harness edit:
   `scripts/core-language-microbench.sh` stays unchanged**; the labels are
   scratch workloads.
3. The base is supplied below.
4. The deep held-expression boundary stays with the hardening line; the
   96-under-32 case is a regression control in step B.

## Context pointers

- brief-580 and its report; `.temp/brief-580/{inventory.md,design.md,inventory-plan.md,probe.rs,cases.json,oracle.tsv,lifecycle.tsv,g7-sites.tsv,performance-plan.md,performance-labels.json}`.
- `debug_tools/reap.rs` (`:21-60`, `:62-114`, `:341-368`, `:390-411`),
  `pattern_matching/{core.rs,outcome.rs}`, `suspension/core.rs`.
- briefs 563, 568, 572-579 (the accepted protocol, the noise-floor gate).

## Scope

**Objective:** the resumable Sow matching loop, its frame, its tests, and the
standing and performance gates.

**Edit surface:** `debug_tools/reap.rs` (a child declaration and the
relocation of `eval_sow`) and new `debug_tools/reap/sow.rs` with private
`sow/tests.rs`; new `crates/rum-stdlib/tests/sow_trace_continuations.rs` with
children `support`, `sow`, `lifecycle` (the Sow subset only; the trace
fixtures wait for step B); new
`crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs`
and its `mod` line; rum-runtime's frame lists (two derive entries, one frame,
named-frame count **560 -> 561**).

**Standing duty — always in scope** (`TEMPLATE.md`): replay the fingerprints
and both certificates (C01/C02 unchanged, marks earned on their merits),
reconcile ownership discovery, disclose every row. No old construct retires in
this step; every Reap render frame keeps its identity. **STOP** on any other row
change or disappearance, any new `UnknownHostPayload` or `self_reaching` row,
any new row in untouched code, any retention reclassification.

**Fence:** no change to trace, options, the matcher, the suspension core,
`reap_state.rs` or the Reap render and handler bodies; no other group; no
leases, `unsafe`, dependencies or lockfile; no edits to `AGENTS.md`, `TODO.md`,
root `PROGRESS.md`, earlier briefs, `scripts/` or docs.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect.

## Deliverable

The ordinary two-phase Sow algorithm is unchanged in shape. The frozen state
(only on `InputRequired`, on the error edge) holds the value and tag, the
scope limit and index, the pattern index and count, and the ordered accepted
pairs. On resume, a true answer adds the pending pair once and advances once;
a false answer advances without recording; a later error commits nothing of
this Sow. Completion runs the existing bucket and occurrence-charging loop
once and returns the original value; scope cleanup stays with the existing
Reap frames. Pure abandonment runs no missing match and no bucket commit. A
second suspension freezes the new state once.

## Tests

Red first, recorded before any production edit: brief-580's thirteen Sow cases
(seven red, six controls — `Sow-match`, `Sow-twice`, nested `Reap` scopes with
different tag patterns, an ordinary error after a resume, cancellation at a
suspended tag test, and the controls) through the rum-vs-rum oracle (value,
error, termination, ordered messages and prints, `log`, actual suspension,
exact prompts, no owner after, `1 + 1`). The structural contract for the new
module: the recursion scanner per file and combined, no self-reaching Sow
state, root completeness, each with a falsifier. Every earlier group's suite
stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Nothing is recorded into a bucket until every match has finished.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-581/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test sow_trace_continuations
cargo test -p rum-stdlib --lib g7_continuation_contract
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-581/base/rum-cli` and
`rum-cli-alloc` — the director's copy of brief-579's selected one-unit
candidate, whose production source is the incoming tree (hashes in
`sha256.txt`); do not rebuild them. Candidates with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s Sow labels (Reap and Sow with no tag,
a matching tag, a declining tag, nested scopes) as scratch workloads, plus the
controls `dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). Each result checked first. **Noise floor first**
(the base against itself, two pairs); a label regresses when both candidate
samples exceed `max(+0.05%, floor)`. Allocation counts, then two paired
samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label
per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw output
first. No timing suite. At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-581/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, state fields and roots.
2. Verbatim red and green output, the structural contract and falsifiers,
   ownership and certificate results, the broad suites; every inventory row
   change.
3. P0125 repaired (14 -> 13 predicted); the noise floors and every measured
   row.
4. Stop-loss counters, residuals, open questions.

---

## Director resume note (2026-10-06) — read this first

The first run was cut by an upstream "selected model is at capacity" error
while building candidate 02's alloc-count binary; nothing in the brief
changes. Resume from `.temp/brief-581/PROGRESS.md`: the thirteen Sow cases are
green, candidate 01 is measured (two labels above their floors), candidate 02
(the one `#[inline(always)]` on `drive_sow_matches`) has its default build.
Build its alloc-count variant (`--features` and `rum-cli/alloc-count` as two
arguments), measure it, continue within the same candidate cap and stop-loss,
then run the standing duty and every verification command. The working tree
holds the run's uncommitted edits; keep them.
