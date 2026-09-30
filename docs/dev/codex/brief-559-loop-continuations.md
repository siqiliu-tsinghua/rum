# brief-559 — loops that resume past their own remaining work: Scan, OptionValue, Differences, Dot, ReplaceRepeated

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 4 of the suspension-continuation branch: the first of the brief-548 repair
groups. brief-548 found 86 sites where a builtin suspends for input in the
middle of its work and resumes **past** that work. This brief takes the three
groups that share one mechanism — **an outer loop whose inner operation resumes
correctly, but whose own cursor, completed prefix and final wrapping are not
carried by any frame**:

| Group | Site | Builtin |
| --- | --- | --- |
| G1 | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:452` (`evaluate_scan_parts`, an identity frame at `:464-479`) | `Scan` |
| G6 | `crates/rum-runtime/src/runtime/options/builtins.rs:265` (`resolve_option_value_query` → `resolve_option_value_scalar`) | `OptionValue[f, {a, b, c}]` |
| G8 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:149` (`resume_replace_repeated_loop` → `apply_replace_all_with_plan`) | `ReplaceRepeated` |
| G8 | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:50` (`eval_dot` → `compute_inner`) | `Dot` with three or more operands |
| G8 | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:27` and `:80` (`eval_differences` → `successive_differences` → `difference_expr`) | `Differences` |

When you are done, each of these builtins, suspended for input at any point in
its loop and resumed, produces exactly what it produces when the answers are
queued up front — same value, same side effects, same messages — and brief-548's
confirmed count of lost continuations falls from 86 to 80.

The director re-ran brief-548's 195 probes on `6614e59b`: all 91 divergences
still diverge and all 104 controls still agree (`.temp/director/g-reprobe/`).

## The oracle

The resumed path equals the queued-input path (`Session::queue_input_response`)
in value, the `log` of side effects, and messages. A test proves nothing unless
it asserts that the suspension actually happened. Suspend in the **middle** of
the loop, and suspend **twice**. brief-550 built a permanent harness of exactly
this shape in `crates/rum-stdlib/tests/guest_loading_resume.rs`; reuse its
shape. If a shared helper would serve these and the later groups, you may
extract one under `crates/rum-stdlib/tests/` and disclose it; do not change
what brief-550's tests assert.

## Red tests first

The witnesses and their pre-fix outcomes are in `.temp/brief-548/probes.json`
and `.temp/brief-548/lost.tsv`; the drafts are `.temp/brief-548/draft-G1.md`,
`draft-G6.md` and `draft-G8.md`. At minimum, as permanent tests, each red
before the change:

- `Scan` and `Scan-twice` — resumed `"two"` with `log {1, 2}` against queued
  `Null` with `log {1, 2, 3}`;
- `OptionValue-many` and `OptionValue-twice` — resumed `"two"` against queued
  `{1, "two", 3}`;
- `ReplaceRepeated-predicate` and `ReplaceRepeated-twice` — resumed
  `{h[1], h[2], h[3]}` against queued `TerminatedEvaluation["IterationLimit"]`
  (the iteration limit must still terminate after a resume);
- `Dot-chain` and `Dot-twice` — resumed `{{4, 6}, {4, 6}}` against queued
  `{{12, 18}, {12, 18}}`;
- `Differences`, `Differences-twice`, `Differences-order-two` — resumed `1`
  against the queued difference lists.

Add for each: an ordinary error raised after a resume still behaves as it does
queued; cancellation while suspended in the middle runs no remaining iteration
and leaves the session usable (`1+1` is `2`). Save the failing output under
`.temp/brief-559/red/` before any production change.

## Design constraints

- **Frames carry the loop.** Each driver's frame holds what the loop needs to
  continue — the operator or rules, the remaining parts or operands, the next
  index or iteration count, the completed prefix, and the final wrapping — and
  its `resume` consumes the resumed value and continues the loop. Initial
  execution and resumption share the same step function.
- **Reuse correct inner frames.** `apply_replace_all_with_plan`, `compute_inner`
  and `difference_expr` already resume their own phase correctly where brief-548
  said so; the caller's frame is what is missing.
- **Nothing on the success path.** Freeze or move state into a frame only when
  the call actually returns `InputRequired`; no clone, allocation, lock or
  counter on the ordinary path.
- Every new frame derives `FrameRootTrace`; a missing traversal stays a compile
  error. Keep existing custom abandonment; never run a skipped iteration on
  abandonment.
- **No leases.** The retention line will convert Scan (region R01) and the
  callback-history loops later, on top of the state these frames carry — name
  that state clearly.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-548's report (the mechanism, §4 root causes, §5 enforcement); the drafts
  and evidence above.
- `crates/rum-runtime/src/runtime/suspension/core.rs` — `insert_suspension_frame`
  (`:515`), `drive_resuming_frames` (`:709`), `resume_or_suspend!` (`:784`);
  `eval_dispatch/mod.rs:466` — the generic dispatch frame whose `complete` is
  what currently swallows the remaining work.
- Existing correct loop frames to model on: `Do`, `Table`, `NestList`, `Select`
  resume correctly (brief-548's controls) — read how they carry their state.

## Scope

- **Objective:** the five builtins' continuation frames and step functions; the
  permanent tests; microbench labels for them.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventory positions; only fingerprints, hashes and positions may change, and
  any identity, status, disposition or exclusion change is a STOP.
- **Fence:** only these five sites and their direct callers. No other brief-548
  group (G2-G5, G7), no enforcement work (E1, E2), no evaluator redesign, no
  `Expr` or `Symbol` change, no leases, no overlap-policy change, no new
  dependency, no `unsafe`. Do not loosen an existing test or helper. Do not
  edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier briefs or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; every new
  frame and root field; any consumer or site you find that brief-548 did not
  list — report it with evidence before widening the objective.

## rum / Rust pitfalls (read twice)

1. **A callee frame does not complete its caller.** Carry the cursor, the
   completed prefix, the pending operation and the final wrapping together.
2. **Suspend in the middle and twice**, and require the corrected path to reach
   both prompts. Never replay a completed side effect.
3. **`ReplaceRepeated`'s limit is part of the loop**: an iteration consumed
   before the suspension is still consumed after it.
4. **Stop-loss is keyed on progress**: sites repaired, equality tests passing,
   lifecycle checks passing, compile errors falling. Stop when two consecutive
   compiling attempts move none.

## Verification (MANDATORY — you run it; iterate to green)

Every command through `python3 scripts/run-bounded.py --timeout <seconds> --`,
bounds from measured runs, cargo as `env CARGO_BUILD_JOBS=4
CARGO_NET_OFFLINE=true cargo …`.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Plus the retention replay in the `--no-default-features` build as the template
describes. Paste commands, bounds, exit statuses and counts verbatim; list
native ZeroMQ skips.

**Performance gate** — these are loop drivers, so show the ordinary path costs
nothing. Before editing, build and preserve base binaries under
`.temp/brief-559/base/` into their own target directory, and check that base and
candidate differ and that the base lacks the change. Then, allocation counts
first (`rum-cli/alloc-count`, `RUM_PROFILE_ALLOC=1`); per-iteration
`cpu_core/instructions:u` with `taskset -c 2`, `rum-cli` fed `< /dev/null`,
`(count(2N) - count(N)) / N`, one label per run, at most two samples per
(label, N, build), N at least 20 000 where the iteration limit allows. Labels:
new scratch labels for `Scan`, `OptionValue` with three names, `ReplaceRepeated`,
three-operand `Dot` and `Differences` — each evaluating without any input
request — plus the existing `do-count-null`, `do-iterator-null`,
`table-iterator-null` and `replaceall-simple`. Timing is secondary; if you run
it, check first that no CPU-heavy process competes. A regression on any label is
a defect to fix or report.

## Checkpointing — protect the scene

Scratch under `.temp/brief-559/`. After each milestone write
`.temp/brief-559/PROGRESS.md`: done and verified with evidence, the exact next
action, the tree's state, the counters.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository, user configuration included. No
  network. Commands: file reads and edits within scope, `rg`/`grep`, `python3`,
  bounded `cargo`, the release `rum-cli`, bounded `taskset`/`perf`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges; frames, fields and roots.
2. Verification — every command's verbatim result; red before and green after
   for every permanent test.
3. Allocation and per-iteration instruction rows per label.
4. Lost count before and after, progress counters, disclosures, anything
   unresolved.
