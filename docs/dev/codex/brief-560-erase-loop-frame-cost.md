# brief-560 — erase the ordinary-path cost brief-559's loop frames added

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-559 (`e0125af4`) made `Scan`, `OptionValue`, `ReplaceRepeated`,
three-operand `Dot` and `Differences` resume into their own loops after a
suspension. It is correct and gated. But it made the **ordinary path** — a
call that never suspends — more expensive for three of them, in per-iteration
user instructions (pinned `taskset -c 2`, `cpu_core/instructions:u`,
`(count(2N) - count(N)) / N`, N = 20 000):

| Label | Base | Candidate | Delta, sample 1 / sample 2 |
| --- | ---: | ---: | --- |
| `differences-no-input` (`Differences[{1,4,9,16,25}]`) | 9 906 | 10 090 | +1.855% / +1.863% |
| `replacerepeated-no-input` | 40 256 | 40 414 | +0.393% / +0.420% |
| `optionvalue-three-no-input` | 9 765 | 9 802 | +0.384% / +0.386% |

The director reproduced the `Differences` row independently: +1.917%, with
`dot-three-no-input` at -0.026% as the control. Allocation counts and peaks are
identical at both N; this is instruction cost on the path that suspends
nothing. The design said the frames cost nothing unless the call actually
suspends. Make that true. When you are done, all nine labels brief-559 measured
sit at or below the base in both samples (a delta no larger than +0.05% in
either sample counts as at the base), and every brief-559 correctness test
still passes.

## What brief-559 already learned — do not repeat it blindly

brief-559's report §3 records two rejected experiments. `#[inline(always)]` on
the newly shared helpers erased `OptionValue`, reduced `ReplaceRepeated`, and
made `Differences` worse (+2.19%). Moving the error handling into cold helpers
and computing the resume cursor only on error brought `Differences` to +1.10%
but moved `Dot` to +0.114%. Read the saved binaries and sources for both
(`.temp/brief-559/candidate-inline/`, `candidate-cold/`).

## Where the director would look first

In `crates/rum-stdlib/src/structure_ops/list_plumbing.rs` the ordinary path
changed shape: the tight `for pair in items.windows(2)` loop became
`differences_from_pair`, iterating `windows(2).enumerate().skip(next_pair)`
with a `match` per element, behind an extra call layer; the order loop wraps
each row in `resume_or_suspend!`. A common way to make a resumable loop free on
the ordinary path is to **keep the original loop verbatim for the path that
starts at the beginning**, and enter a separate resumed-loop function — the one
that skips to the cursor and appends to a carried prefix — only from a frame's
`resume`. The frame is built in a cold, non-inlined branch reached only when a
call returns `InputRequired`. The same shape may apply to `OptionValue`'s name
loop and `ReplaceRepeated`'s pass loop. These are hypotheses; measure each
change, one variable at a time.

## Measurement — the preserved base is the reference

Use `.temp/brief-559/base/rum-cli` and `rum-cli-alloc` as the base; do not
rebuild them. Build each candidate into its own directory under
`.temp/brief-560/`, keep every candidate binary you measure, and record its
hash. Labels: `.temp/brief-559/perf/*.rum` — `scan-no-input`,
`optionvalue-three-no-input`, `replacerepeated-no-input`, `dot-three-no-input`,
`differences-no-input`, `do-count-null`, `do-iterator-null`,
`table-iterator-null`, `replaceall-simple`, at N = 20 000 and 40 000.
Allocation counts first, then instructions, two samples per (label, N, build),
one label per run, `rum-cli` fed `< /dev/null`. **Every candidate is measured on
all nine labels** — the cold-helper experiment's `Dot` regression is exactly
the kind of neighbour effect this catches. Timing is not needed.

## Scope

- **Objective:** the ordinary-path shape of the five brief-559 sites, so that
  all nine labels meet the threshold, with the brief-559 correctness tests
  unchanged and green.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventories; new `UnknownConstruct` rows for code you add or change are
  expected and disclosed; STOP on a changed or vanished existing row, a new
  `UnknownHostPayload` row, or a row in code you did not touch.
- **Fence:** only the five brief-559 sites and their private helpers. No
  semantic change: `crates/rum-stdlib/tests/loop_continuations.rs` and every
  other test stay exactly as they are and must stay green. No other brief-548
  group, no enforcement work, no `Expr` change, no leases, no new dependency,
  no `unsafe`. Do not edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier briefs
  or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; each
  candidate you measured, what it changed, and its nine-label rows.

## rum / Rust pitfalls (read twice)

1. **One variable per measurement.** Change one thing, measure all nine labels.
2. **An improvement on one label that regresses a neighbour is rejected.**
3. **Instruction counts decide.** Do not substitute timing.
4. **Stop-loss is keyed on progress**: the number of labels above the
   threshold must fall. Stop when two consecutive compiling candidates reduce
   neither that number nor the largest remaining delta, and report the curve.

## Verification (MANDATORY — you run it; iterate to green)

Run everything serially (user instruction): every cargo command with
`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`, never two cargo commands at once,
each timeout derived from a measured serial run of the same shape, through
`python3 scripts/run-bounded.py --timeout <seconds> --`.

```
cargo fmt --check
cargo test -p rum-stdlib --test loop_continuations
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Plus the retention replay in the `--no-default-features` build as the template
describes. Paste commands, bounds, exit statuses and counts verbatim; list
native ZeroMQ skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-560/`. After each candidate write
`.temp/brief-560/PROGRESS.md`: its change, its nine-label rows, the counters,
the exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository, user configuration included. No
  network. Commands: file reads and edits within scope, `rg`/`grep`, `python3`,
  bounded `cargo`, the release `rum-cli`, bounded `taskset`/`perf`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. Every candidate measured, with its nine-label allocation and instruction
   rows, both samples.
3. Verification — every command's verbatim result.
4. Progress counters, disclosures, anything unresolved.
