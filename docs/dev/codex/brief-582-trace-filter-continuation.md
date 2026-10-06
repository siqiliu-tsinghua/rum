# brief-582 — G7 step B: the structured trace filters resume, and the branch's groups close

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`Trace`, `TraceDetail` and `TracePrint` with a pattern or a level / head
filter walk the captured trace tree and match each item; a pattern test that
asks for input resumes **past** the walk. Thirteen sites in
`crates/rum-runtime/src/runtime/debug_tools/trace.rs` — P0199, P0200, P0201,
P0202, P0203, P0204, P0205, P0206, P0208, P0209, P0210, P0211, P0213 — through
`filter_structured_trace_expr` / `_items` / `_leaf` and the detail family down
to `match_expr_owned`. brief-580's design: **one flat filter work stack and one
named frame**, preserving the trace structure, the candidate, pattern and
detail-field order, short-circuiting, the `Hold` and association
reconstruction, deduplication, printing, and `TracePrint`'s original return
value. Lost continuations **13 -> 0** — the last of brief-548's eight groups.

brief-580's report and `.temp/brief-580/draft-B-trace.md` are the raw
material; **this brief governs** where they differ. The rulings in brief-581
stand.

## Context pointers

- brief-580 and its report; `.temp/brief-580/{inventory.md,design.md,inventory-plan.md,performance-plan.md,performance-labels.json,probe.rs,cases.json,oracle.tsv,lifecycle.tsv,held-depth-96.rum}`.
- brief-581 and its report: the shared harness
  (`crates/rum-stdlib/tests/sow_trace_continuations/`) and the structural
  contract (`crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs`).
- `debug_tools/trace.rs` (`:61-151`, `:178-598`, `:652-767`),
  `trace/{builder,options,resume_frames}.rs`, `tree_walkers/level.rs:23-144`,
  `pattern_matching/outcome.rs`, `suspension/core.rs:764-855`; the existing
  trace shape and indentation contracts `runtime/tests/debug_tools.rs:4-266`.

## Scope

**Objective:** the flat trace filter, its frame, the tests, and the standing
and performance gates.

**Edit surface:** `debug_tools/trace.rs` (routing; the old recursive walkers
removed), new `trace/filter.rs`, `trace/filter/{leaf,state}.rs` and private
`trace/filter/tests.rs`, `trace/resume_frames.rs`; the shared harness
`sow_trace_continuations.rs` and its `{support,trace,lifecycle}` children;
`g7_continuation_contract.rs` (flat-state and edge gates with falsifiers);
rum-runtime's frame lists (five derive entries, one frame, named-frame count
**561 -> 562**).

**Inventory exceptions, authorized by name** (`inventory-plan.md`, I03):
retire the old `trace.rs` wrapping-map closures at `:552:26` and `:593:26` and
the `vec!` sites at `:519:23`, `:552:70`, `:559:57`, `:575:25`, `:593:70`,
`:596:32` — **only if each construct is actually gone** — with their
replacements discovered and disclosed in the new modules. `Unsplit:Trace`,
`Unsplit:TraceDetail`, `Unsplit:TracePrint` stay Unsplit/Pending. P0207 and
P0212's dependent routing may resolve without adding Lost counts.

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints and both
certificates (C01/C02 unchanged, marks earned on their merits), ownership
reconciliation, every row disclosed. **STOP** on any other row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code, any retention reclassification.

**Fence:** `trace/builder.rs`, `trace/options.rs`, `reap_state.rs`, Sow (step
A), the matcher, the level collector, the renderers, the suspension core and
public APIs unchanged; the existing capture frame keeps its custom
abandonment. No other group; no leases, `unsafe`, dependencies or lockfile; no
edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs,
`scripts/` or docs. Keep `trace.rs` under 20k characters by extracting the
matching, not unrelated formatting; new modules under 10k each.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect.

## Deliverable

brief-580 design.md §3: the flat driver owns the current level's remaining
siblings and emitted prefix, the parent levels, the pending original item, the
detail-field, candidate and pattern indices, the selected candidates and the
leaf's accepted prefix; it freezes only on the matcher's `InputRequired`. On
resume it applies the answer once: false advances pattern, then candidate,
then detail field; a detail match emits its original association once and
skips the rest of the search. Structured completion wraps original candidates
in `Hold` and keeps the existing deduplication and shape rules. Only root
completion renders and pushes `TracePrint` lines — exactly once — and returns
the saved body result. A second suspension freezes the new state once; pure
abandonment runs no missing match and prints nothing.

## Tests

Red first, recorded before any production edit: brief-580's remaining 38
cases (18 red, 20 controls: twelve trace cancellation cases, seven comparison
controls, and the 96-under-32 held-depth control) through the shared oracle —
value, error, termination, ordered messages, **exact print streams**, `log`,
actual suspension, exact prompts, no owner after, `1 + 1`. Step A's thirteen
stay green: 51 in all. The existing trace shape and indentation contracts
stay green. The structural contract extended to the filter modules, each check
with a falsifier.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. `TracePrint` prints once, at root completion — not per resume.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression. If the gate cannot be met, report with
   attribution; do not switch to per-level frames on your own.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-582/tmp …`. Bounds: focused 150 s; runtime,
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

**Performance gate.** Base: `.temp/brief-582/base/rum-cli` and
`rum-cli-alloc` — the director's copy of brief-581's selected one-unit
candidate, whose production source is the incoming tree (hashes in
`sha256.txt`); do not rebuild them. Candidates with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s trace labels (unfiltered, pattern
filter, level and head filters, `TraceDetail`, `TracePrint`, a nested
evaluation) as scratch workloads, the Sow labels (`.temp/brief-581/perf/`) and
the controls. Each result checked first. **Noise floor first** (base against
itself, two pairs); a label regresses when both candidate samples exceed
`max(+0.05%, floor)`. Allocation counts, then two paired samples of `taskset -c
2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
`/dev/null`, `(count(2N) - count(N)) / N`, raw output first. No timing suite.
At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-582/`; `PROGRESS.md` after the red capture, each
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

1. Changed files and functions, state fields and roots, file sizes.
2. Verbatim red and green output (all 51 cases), the structural contract and
   falsifiers, ownership and certificate results, the broad suites; every
   inventory row change, the retirements named.
3. Thirteen sites repaired (13 -> 0 predicted); the noise floors and every
   measured row.
4. Stop-loss counters, residuals, open questions.

---
