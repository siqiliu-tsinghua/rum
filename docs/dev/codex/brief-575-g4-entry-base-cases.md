# brief-575 — G4's ordinary path, second round: small entries over out-of-line drivers

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-574 measured all ten candidates and kept candidate 09: against the
pre-G4 one-codegen-unit base, 10 of 23 labels still regress, among them the
hot `matchq-integer-success` / `-fail` (director-reproduced on candidate 09:
+1.11% / +1.11%), `g4-rule-blank-constraint` (+5.09%) and
`g4-matchq-skew-depth-8` (+4.53%). Its attribution (§2) names the cost precisely. For
`MatchQ[5, _Integer]` the old recursive resolver and classifier cost 57 + 55
self instructions; the flat driver costs 226, plus 25 for acceptance and 26 to
drop an empty work `Vec` — for a root that is a recognized pattern head, which
the driver never needs to walk. For a short `Blank` rule the old recursive
helper cost 278 and the driver 770, plus finish 96 and work-`Vec` drop 58.

That is the cost of entering a large driver, not of doing work. brief-574
could not attack it because the director's briefs forbade a "second
resolution algorithm". That rule stays — but handling a classification's own
base cases before entering the driver is not a second algorithm: the old
recursive code did exactly that as its first step. **Target:** every label of
`.temp/brief-573/perf/labels.json` within +0.05% of the pre-G4 base (both
samples), every test unchanged and green; a residual is reported with its
attribution.

## The structural direction — measure it, one variable at a time

The traversal-depth line's lesson (`CLAUDE.md`): on the hottest function, move
locals and cold bodies, not control flow — outlining the cold branches cost
nothing and cut the frame 3.17x where an in-loop rework cost 2-7%.

1. **A small entry for each walker** — `resolve_held_matcher_pattern` and
   `evaluate_rule_lhs` — that settles the classification's base cases itself
   and returns: for the held resolver, a root that is a recognized (opaque)
   pattern head and a root with no pattern syntax (evaluate, then normalize);
   for the rule walk, whatever base cases its classifier settles at the root
   (`evaluate_rule_lhs` already returns early for a pattern-free left-hand
   side — check what else is settled at the root). Everything else calls the
   driver, kept out of line (`#[inline(never)]` and, where the attribution
   supports it, `#[cold]` on the capture paths). The entry must not grow a
   second copy of the walk: it decides only what the driver's first step would
   decide.
2. **No work `Vec` until there is a child to push** — the empty-`Vec` drop
   (26 and 58 instructions) disappears when the state is created on first
   descent.
3. **The rule driver's short-rule transitions** — `finish` and acceptance per
   §2 — after 1 and 2 are measured.

**A falsifier is required for 1**: a test that drives every base-case shape
(opaque pattern head, pattern-free root, each rule-root base case) through the
**driver** directly and through the **entry**, and asserts identical results
— so the two can never drift.

## Fixed — not negotiable

Both flat walks and their depth contracts (`rule_pattern_depth_contract.rs`,
the private depth tests); the semantics and every test; capture only on
`InputRequired`; no recursion, no depth cap, no `unsafe`.

## Measurement

Exactly brief-574's: base `.temp/brief-572/base/` (pre-G4, one unit; do not
rebuild); candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directories
under `.temp/brief-575/`, hashes checked; panel `.temp/brief-573/perf/` (23
labels, results checked first); allocations, then two paired samples of
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label per
process, stdin `/dev/null`, `(count(2N) - count(N)) / N`; any kept candidate
and the final one on all 23 labels; callgrind attribution calibrated against
`perf stat`. Measure the incoming tree once as candidate 00. No timing.

## Scope

- **Objective:** the entries and drivers of `definitions/rule_lhs.rs` (and
  `rule_lhs/`) and `cases/pattern_resolution.rs` (and `pattern_resolution/`),
  their callers in `definitions.rs` and `cases/match_q.rs`, the iterative
  classifiers, and the falsifier test (in the walkers' private test modules).
- **Standing duty — always in scope** (`TEMPLATE.md`). New `UnknownConstruct`
  rows for code you add or change are expected and disclosed; STOP on any other
  changed or vanished existing row, a new `UnknownHostPayload` or
  `self_reaching` row, or a row in untouched code.
- **Fence:** no semantic change; no change to existing tests; no matcher,
  normalizer, evaluator or `Expr` change; no other group; no dependency; the
  release profile unchanged. No edits to `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or docs.
- **Disclosure duty:** every change with line ranges and why; every candidate
  with its attribution and rows.

## rum / Rust pitfalls (read twice)

1. One variable per candidate; a candidate that makes another label regress is
   rejected; instruction counts decide.
2. Stop-loss keyed on progress (regressing labels or the largest delta must
   fall); stop after two consecutive compiling candidates that reduce neither;
   at most ten measured candidates; stopping early while progressing needs a
   stated reason.
3. `#[inline(always)]` on the driver is the opposite of the direction here.

## Verification (MANDATORY — you run it; iterate to green)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-575/tmp …`. Bounds: focused 150 s; a one-unit
release build 900 s; a metric process 150 s; runtime and retention 300 s;
`cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test rule_pattern_continuations
cargo test -p rum-stdlib --lib rule_pattern_depth_contract
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

Plus the retention replay in both feature sets. Paste commands, bounds, exit
statuses and counts verbatim; list native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-575/`; `PROGRESS.md` after each candidate.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Nothing outside this repository; no network. Commands: reads and scoped
  edits, `rg`/`grep`, `python3`, bounded `cargo`, the release `rum-cli`,
  bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind`. No probe that
  aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). **No new `unsafe`.**
- Paste real verbatim output; say plainly what did not run.

## Report format (append to this brief)

1. What changed, per file with line ranges.
2. Attribution for each regressing family, base against incoming.
3. Every candidate with its rows (all 23 for kept ones), both samples.
4. Verification, verbatim.
5. Counters, disclosures, attribution of anything left.

---
