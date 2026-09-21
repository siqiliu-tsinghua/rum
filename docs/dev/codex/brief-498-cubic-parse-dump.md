# brief-498 — the parse dump is cubic, and recurses

A leftover of the superlinear-elimination line, found while closing the
lowering fail-open. The diagnosis below is complete and director-measured;
**do not re-derive it.**

## The defect

`ParsedSource::debug_dump` (`rum-syntax/src/document.rs:377-380`) renders the
syntax tree with Rust's alternate `Debug`:

```rust
writeln!(&mut output, "{:#?}", self.syntax).unwrap();
```

`{:#?}` wraps the writer in another pad adapter **per nesting level**, so every
byte written passes through O(depth) layers. With a call-head chain
(`f[1][1][1]…`, which the parser's 128 counters do not bound — see brief-496),
that is cubic.

Director-measured, `taskset -c 2 perf stat -e instructions:u`, release CLI,
`rum --parse`:

| applications | instructions | ratio per doubling |
|---:|---:|---:|
| 100 | 1 363 439 898 | — |
| 200 | 10 459 548 864 | 7.67 |
| 400 | 81 949 207 374 | 7.84 |
| 800 | 648 797 287 375 | 7.92 |

**×7.9 per doubling is 2³ — cubic.** Output bytes grow ×3.88 per doubling —
quadratic (213 KB, 807 KB, 3.1 MB at n = 50/100/200), which for an indented
dump of a depth-n spine is *inherent*. So the work is **one order worse than
the output it produces**, and that gap is the defect. A 100-application chain —
a 300-character program — already costs 1.36 **billion** instructions; at about
1 000 the command exceeds a 60-second timeout.

**There is very likely a second defect on the same path.** Derived `Debug` on
`SyntaxExprKind` recurses through `Box<SyntaxExpr>` and `Vec<SyntaxExpr>`, so
the dump's recursion depth is the tree depth and nothing guards it. Confirm
whether the dump can overflow the stack rather than merely being slow — the
cubic cost makes a direct deep probe impractical, so **establish it from the
source and from a small-stack run at a depth you can actually finish**, not by
driving a process to termination.

## Goal

The dump's work is at most proportional to the bytes it emits, and its
recursion is bounded. `rum --parse` on a 1 000-application chain completes
promptly instead of timing out.

## The format is not the target — match it

Reproduce the current output **byte for byte**. The existing tests pin it
(`rum-cli/tests/cli.rs:81`, `rum-syntax/src/document.rs:480`), and two
production callers consume it: `rum-cli/src/main.rs:67` (`--parse`) and
`rum-repl/src/commands.rs:122`. The defect is the cost and the recursion, not
the shape of the output.

If byte-identical reproduction of `{:#?}`'s layout proves impractical, **STOP
and report** with what differs — a deliberate format change to a debug surface
is the director's call, not a workaround you take on the way past.

## The gates

1. **The exponent must change, and be shown.** Re-measure at the same
   n = 100 / 200 / 400 / 800 with the same recipe, before and after, and report
   both ratio columns. The requirement is that the after-ratios approach **~4
   per doubling** — work proportional to output — not merely that some
   constant improved. A faster cubic is a fail.
2. **`rum --parse` at 1 000 applications completes** within a stated bound
   where it previously exceeded 60 s. Report the time and the instruction
   count.
3. **Output identical.** Diff the dump before and after at several depths and
   at least one *shape-varied* fixture (lists, associations, patterns, binary
   sugar, slots — the tree the existing tests build), and paste the empty diff.
4. **Bounded recursion**: a depth that would previously have risked the stack
   either completes or fails in band. Say which, and how you bounded it — this
   line's pattern is a guarded front plus a cold iterative tail, and the
   emitter takes no `Session`, so redoing its traversal is permitted.
5. `cargo fmt --check` and the full workspace suite.

## Scope

Files you MAY modify: `crates/rum-syntax/src/document.rs` (the dump and its
tests) and, only if the emitter belongs there, a new private module beside it.
`crates/rum-cli/src/main.rs` and `crates/rum-repl/src/commands.rs` **only** if
the signature must change — say why, and prefer keeping it.

- `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` — under the
  disclosure rule: regenerate as discovery requires, list **every** content
  change with its witness, classified (a) position drift and closure/macro
  rows from this brief's own edits, or (b) anything else — **(b) is a stop**,
  as is any new `UnknownHostPayload`, any removed row, or any
  reclassification. Never hand-edit a row to make a check pass. Note that
  `rum-syntax` is now a scanner root (brief-495), so edits there move rows.

MUST NOT touch: the parser and its depth counters — **no parser cap**;
`rum-runtime`'s lowering (brief-497 just landed there); `accumulator_chain.rs`,
`host_payload.rs`, `crates/rum-derive/**`, the suspension traits, the ownership
scanner's `.rs` code; `rum-core`; `SECURITY.md`, `AGENTS.md`, `PROGRESS.md`,
`TODO.md`, other `docs/**`, `docs/dev/codex/**` except your own report; `.git`,
git config, `Cargo.lock`.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run `git stash` /
  `git checkout` on a modified file.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No crash, overflow, past-ceiling, or fatal-process probe.**
- **No new `unsafe`.**
- **No `#[cfg(test)]` divergence in production code.**
- **Read the failure, not the exit status**: rum exits 1 on a syntax error and
  124 on timeout, and an exit-status-only probe already turned timeouts into
  phantom stack overflows once on this line.
- **Bound every long run** with `scripts/run-bounded.py`, and derive the bound
  from a recorded run of the same shape — the pre-change numbers above are
  yours to size with.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27).
- If the same obstacle occurs twice, STOP and report.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one line why.
2. The exponent table, before and after, with both ratio columns.
3. The 1 000-application result: time and instructions, against the previous
   timeout.
4. The output diffs, empty, across depths and the shape-varied fixture.
5. What you established about the dump's recursion, and how you bounded it.
6. `cargo fmt --check` and the workspace total.
7. The inventory diff, classified.
8. Stop-loss / open questions.
