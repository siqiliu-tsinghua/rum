# brief-574 — erase G4's ordinary-path cost

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

briefs 572 (`d2e0641c`) and 573 (`14f8b46c`) closed group G4: the rule
left-hand-side walk and the held matcher-pattern resolver are flat work stacks
that resume into their work. Both are correct and gated. But calls that never
suspend got dearer. Against the **pre-G4** one-codegen-unit base, from
`.temp/brief-573/perf/against-pre-G4.tsv`:

| Label | Delta, sample 1 / 2 |
| --- | --- |
| `matchq-integer-success` / `-fail` (`MatchQ[5, _Integer]`-shaped; hot) | +2.76% / +2.77%; +2.85% / +2.85% |
| `cases-pattern-child` | +2.51% / +2.35% |
| `g4-rule-blank-constraint` | +6.22% / +6.20% |
| `g4-rule-skew-depth-8` | +4.82% / +4.84% |
| `g4-matchq-skew-depth-8` | +6.27% / +6.28% |
| `g4-matchq-prepared-call` | +4.92% / +4.92% |
| `g4-matchq-pattern-head` / `-child` | +4.21%; +4.01% |
| `g4-cases-pattern-child` | +2.42% / +2.29% |
| `downvalue-definition` | +0.36% / +0.12% |

Several labels got faster (`g4-rule-wrapper` -7.9%, `g4-rule-pattern-child`
-2.9%, `g4-rule-skew-depth-32` -2.3%), and the controls sit at the base.
Allocation counts or bytes rose on the held-pattern labels (e.g.
`g4-matchq-skew-depth-8` 60 -> 63 allocations, 7 312 -> 10 224 bytes).

**Target:** every label of `.temp/brief-573/perf/labels.json` (23) within
+0.05% of the pre-G4 base (a label regresses when **both** samples exceed
+0.05%), every test unchanged and green. A residual is reported with its
attribution to work the flat walks must do that the recursions did not.

## Fixed — not negotiable in this brief

- **Both flat walks and their depth contracts.** No native recursion in
  proportion to depth; `rule_pattern_depth_contract.rs` and the private depth
  tests stay unchanged and green. **No regression is bought back by restoring
  recursion.**
- **Semantics.** `crates/rum-stdlib/tests/rule_pattern_continuations.rs` and
  its children, and every other test, stay exactly as they are.

## Where the director would look first — hypotheses, measure each

1. **`MatchQ[5, _Integer]` costs ~140 more instructions and touches neither
   walk's interesting part** — the pattern is a bare `Blank`. In
   `resolve_held_matcher_pattern`
   (`kernel_data/rewrite_data/cases/pattern_resolution.rs`), the root is
   cloned out of `strip_hold_pattern`, a state with a `Vec` is built, the loop
   is entered, and the now-iterative `contains_pattern_syntax` runs its loop.
   A root that is itself a recognized pattern head, or contains no pattern
   syntax, needs none of that — but **no second resolution algorithm**: the
   same driver, entered without building what it does not need.
2. **Clones made only for the error edge.** The evaluate branches clone the
   expression on the success path (`self.evaluate(expr.clone())`) so a
   suspension can capture it; recover it in the cold capture helper instead.
   brief-566 and brief-569 removed the same shape from `LazyTable` and the
   schema walk.
3. **Vector growth.** The skewed-depth labels reallocate the work stack and
   prefix vectors (bytes 3 256 -> 9 048 for the rule skew); reserve from
   known arity, reuse buffers, or avoid the prefix for untouched arguments.
4. **The rule walk's short-rule transitions** (`definitions/rule_lhs.rs`):
   brief-572 §4 attributes the `Blank` rule's cost to driver self-cost (562),
   value acceptance (234), completion (96) and work-`Vec` drop (58), against a
   recursive helper of 278.

**Attribute before you change**: `valgrind --tool=callgrind` on base and
candidate, each total calibrated against `perf stat` on the same binary.

## Measurement

- **Base:** `.temp/brief-572/base/rum-cli` and `rum-cli-alloc` (pre-G4, one
  codegen unit; hashes in `sha256.txt`); do not rebuild them.
- **Candidates:** the working tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`,
  default and `--features rum-cli/alloc-count` (two arguments), into a fresh
  target directory each under `.temp/brief-574/`; check the hash changed; keep
  every measured binary, its hash and its source. Measure the incoming tree once
  the same way as candidate 00.
- **Panel:** `.temp/brief-573/perf/` — `labels.json`, workloads and checks
  (check each result first). brief-573's scripts are a working harness to copy.
- **Method:** allocation counts first, then two paired samples of `taskset -c
  2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
  `/dev/null`, `(count(2N) - count(N)) / N`, raw output saved before parsing.
  Any candidate you keep, and the final one, runs all 23 labels. No timing.

## Scope

- **Objective:** the ordinary-path shape of `definitions/rule_lhs.rs` and
  `rule_lhs/`, `cases/pattern_resolution.rs` and `pattern_resolution/`, the
  delegating code in `definitions/definitions.rs` and `cases/match_q.rs`, and
  the two iterative helpers (`definition_lhs_contains_pattern_syntax`,
  `contains_pattern_syntax`, `strip_hold_pattern`).
- **Standing duty — always in scope** (`TEMPLATE.md`): replay the fingerprints
  and both certificates, reconcile the ownership inventories, keep
  rum-runtime's frame lists exact. New `UnknownConstruct` rows for code you add
  or change are expected and disclosed; STOP on any other changed or vanished
  existing row, a new `UnknownHostPayload` or `self_reaching` row, or a row in
  code you did not touch.
- **Fence:** no semantic change and no test edit; no change to the matcher
  proper (compile, match, codecs), the normalizer, the evaluator or `Expr`; no
  other suspension group; no dependency, `unsafe`, depth cap or recursion; the
  release profile unchanged. No edits to `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  candidate — what it changed, its attribution, its rows.

## rum / Rust pitfalls (read twice)

1. **One variable per candidate.**
2. **A candidate that makes another label regress is rejected.**
3. **Instruction counts decide.**
4. **Stop-loss is keyed on progress:** the number of regressing labels or the
   largest remaining delta must fall. Stop after two consecutive compiling
   candidates that reduce neither; at most ten measured candidates. **Use the
   budget**: stopping early while candidates still make progress needs a
   stated reason.
5. A second ordinary code path beside the general one needs a falsifier
   proving both agree on every semantic case, or do not add it.
6. `#[inline(always)]` is not a fix by itself (brief-560: +1.17%).

## Verification (MANDATORY — you run it; iterate to green)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-574/tmp …`. Bounds: focused 150 s; a one-unit
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

Scratch under `.temp/brief-574/`. After each candidate write
`.temp/brief-574/PROGRESS.md`: its change, attribution, rows, counters, the
exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads and
  edits within scope, `rg`/`grep`, `python3`, bounded `cargo`, the release
  `rum-cli`, bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind` with
  `callgrind_annotate`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. The attribution for each regressing family, base against incoming.
3. Every candidate measured, with its rows (all 23 for kept candidates), both
   samples.
4. Verification — every command's verbatim result.
5. Progress counters, disclosures, and the attribution of anything left.

---
