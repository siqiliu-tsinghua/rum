# brief-442 — shrink the stability walk's native frame

**Shape: implementation.** One variable: where the locals of the stability walk
live. No control-flow change, no depth parameter, no iterative fallback.

## Goal

The recursive stability walk costs about **811 bytes of native stack per
expression level**. At that rate a freshly rebuilt tree of physical height
4,097 needs about 3.3 MB and cannot fit the 2 MiB embedder stack at all, let
alone the 1 MiB default evaluator budget.

The dominant frame is `cached_expr_stability_with_bloom` at **584 bytes**, and
that is not because the function is large — it is because the entire chain
below it is inlined into it, so its frame is the union of every branch:
thirteen data-constructor arms, the cache lookup, entry validation, insertion,
the head-effect computation, bloom accumulation, and the profiling tokens. The
descent that actually runs for a deep rebuilt tree touches almost none of that:
cache miss, head stable, no data-constructor arm, no head effect, walk the
arguments.

Outline the cold branches so the hot descent pays only for itself. **Target:
≤256 bytes of native stack per level**, which is what puts height 4,097 inside
the 1 MiB budget (`1,048,576 / 4,097 = 255.9`).

Two shapes have already been rejected for this defect and you are not to
attempt either. brief-440 and brief-441 made the walk iterative behind a
depth-bounded front; both were functionally correct and both cost 2-7% on the
evaluator's hottest label families, because a depth parameter and an `Option`
return on the most-consulted function in the system is not free. Both are
preserved in `stash@{0}` and `stash@{1}`. **This brief changes where locals
live, not how control flows.**

## What I measured — treat as given, do not re-measure by crashing

Release `rum-cli` at HEAD, bisected ceilings for `ReplaceAll[x, 1 -> 2]` on a
value built by `x = 1; Do[x = {x}, {i, N-1}]`. `ReplaceAll`'s own traversal is
already iterative, so this isolates the stability walk:

| stack | last ok | first fail |
|---|---|---|
| 1024 KiB | 1,696 | 1,708 |
| 2048 KiB | 2,988 | 3,000 |

Slope `(2048-1024) KiB / (2988-1696) levels` = **811 B/level**.

Frame prologues from `objdump -d --no-show-raw-insn`, counting each leading
`push %r` as 8 bytes plus the first `sub $N,%rsp`:

| bytes | symbol |
|---|---|
| **584** | `evaluation_cache::stability::…::cached_expr_stability_with_bloom` |
| 168 | `eval_core::stability::failure_details_would_normalize` |
| 120 | `eval_core::stability::…::failure_data_stability_with_bloom` |
| 120 | `eval_core::stability::…::symbol_is_atom_evaluation_stable` |
| 56 | `eval_core::stability::hold_all_literal_args_are_stable` |

`call_stability_with_bloom`, `call_stability_uncached_with_bloom`,
`data_constructor_stability_with_bloom` and `all_args_stability_with_bloom` have
no separate symbols in the release artifact — they are inlined into the 584-byte
frame. That inlining is the finding.

Reproduce the table with:

```text
objdump -d --no-show-raw-insn target/release/rum-cli \
  | awk '/^[0-9a-f]+ <.*stability.*>:$/{f=$2} /sub .*,%rsp/{if(f){print f, $0; f=""}}'
```

## The red target is already in the working tree

`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs` already carries the
three repaired certificates and their three shallow companions, restored from
the rejected work. The companions pass at HEAD; the deep certificates do not.
At HEAD the release certificate run ends:

```text
thread 'map-depth-4097' has overflowed its stack
fatal runtime error: stack overflow
process didn't exit successfully: ... (signal: 6, SIGABRT: process abort signal)
```

That is your target: make that run pass. **Do not modify that test file.**

## Deliverable

### 1. Outline the cold branches

Move the branches that a deep descent does not execute out of the hot frame,
behind `#[cold] #[inline(never)]` (or `#[inline(never)]` alone where `cold` is
wrong because the branch is common but shallow). Candidates, in the order I
expect to matter:

- the thirteen data-constructor arms in
  `crates/rum-runtime/src/runtime/eval_core/stability.rs:36-79` and the
  per-arm helpers they call;
- cache **insertion** and the rewritten-dirty-skip bookkeeping
  (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:293-329`);
- the profiling and timing paths — `call_stability_timing_start`,
  `record_call_stability_source_head_*` — which are off by default but whose
  locals are in the union;
- the miss-outcome classification arms
  (`crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:231-292`).

Judge each by re-reading the prologue after the change, not by expectation.

### 2. Do not change control flow or semantics

- **No depth parameter, no `Option`-wrapped return, no begin/finish split, no
  iterative fallback.** Those are the two rejected shapes.
- The counters must come back byte-identical. From the brief-440 gate on the
  same panel workload: `stability_checks=664`, `call_stability_hits=132`,
  `call_stability_misses=216`, `insertions=138`,
  `rewritten_dirty_skips=0`. Produce this line first and compare it.
- The five invariants are unchanged by construction if you only move code:
  pointer-key lookup and stable-only insertion, the early-unstable
  short-circuit, bloom union order, data-constructor and head-effect order
  before the argument walk, and cache insertion order. Say for each that it is
  untouched, or explain why it moved.
- `#[cold]` on a function that is actually hot is a regression, not a saving.
  If a candidate turns out to be on the hot descent, leave it inline and say so.

### 3. Report the achieved per-level cost

Report the new prologue table and the resulting per-level total. Do **not**
bisect a ceiling by crashing: the certificate run is the acceptance test, and
an abort there is the stop-loss result, not a measurement to refine.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
cargo test -p rum-stdlib --release --lib tests::iterative_traversal_depth -- --ignored
```

Run the workspace gate **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`. If loopback
binding is unavailable, report the nine failures as environmental and say
plainly that those tests did not execute.

The release certificate run is the acceptance test. All twelve must pass,
including the three repaired ones.

**A/B is mandatory.** Outlining changes code layout, and this project has
measured layout alone moving labels by up to 4.45%. Four passes
base/candidate/base/candidate in one window, same-binary spread reported first;
a layout-control binary with the hashes checked if a label still looks real;
within-run quotients against untouched neighbours if the whole panel shifts.
Base is HEAD. Panel: `downvalue-exact-miss-repeat`,
`downvalue-pattern-miss-repeat`, `matcher-single-tiny`, `cases-wide-no-match`,
`rewrite-guarded-hit`, `replaceall-simple`, plus the untouched index
`do-count-null`, `do-iterator-null`, `symbol-ownvalue-read`, `set-ownvalue`,
`list-literal-small`, `replace-wide-no-match`. Those first five are the
families the two rejected shapes regressed; this change must not regress them.

**Stop-loss.** Stop and report, without a second revision, if: the counters
change; the certificate run still aborts after one revision; the per-level
total does not reach 256 bytes — report the floor you did reach, it is a
useful number either way; or any panel label regresses beyond its own
same-binary spread.

## Explicitly out of scope

- Making the walk iterative or bounded in any form.
- A strided native-stack poll inside the walk. A shrunk frame still ends the
  process above whatever height it now reaches, and that is a separate decision
  I will make with your measured number in hand.
- Splitting any file, though three are over the organization threshold
  (`eval_core/stability.rs`, `evaluation_cache/tests.rs`, `memory.rs`). A split
  changes layout and would confound this A/B; I will brief it separately.
- Any traversal conversion from brief-439's steps 2-8.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/eval_core/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs`,
  and a module root that must change only because a symbol it re-exports moved.
- **General accept-vs-stop criterion**: moving a function between files inside
  the stability cluster is in scope; any other non-test source change is a stop.
- Do not modify any test file, including
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`.

## Hard rules

- **Do not write, generate, or run any program whose purpose is to exhaust the
  stack.** The certificate suite is a test, not a probe; if it aborts, that is
  a result to report, not something to explore further.
- Do NOT `git commit`, branch, push, or touch git config. Do not apply, drop or
  otherwise touch the stashes.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`.
- No default-path cost for diagnostics or off-by-default features.

## Report format

The before/after prologue table and the achieved per-level total; the counter
line proving semantic neutrality; verbatim `cargo fmt --check`, the three
`cargo test` runs, and the release certificate run; the full A/B panel with the
same-binary column first; and an explicit verdict against the 256-byte target.
