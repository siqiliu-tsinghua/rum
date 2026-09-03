# brief-425 — ten traversals must reach height 4,097 on a 2 MiB stack

Shape: **implementation**. Worker: **cdx (Codex)**.

Second step of the traversal line. brief-424 measured which traversals fall
short of the ruled bound; this converts them.

## The ruling and the target

`MAX_PHYSICAL_EXPR_DEPTH` will be **at least 4,097** (user ruling 2026-09-03),
preserving plan-330's documented 4,096-link contract. The bound cannot be
installed until the traversals below can handle a value at that height on the
smallest stack an embedder gets by default — **2 MiB, which is Rust's own
default for a spawned thread** (`docs/guide/resource-limits-and-sandbox.md:122`).

Director-measured on a 2 MiB stack, release, at physical height 4,097. These ten
do not complete:

`ToString`, `Map`, `Apply`, `ReplaceAll`, `Cases`, `Level`, `MemberQ`, `Count`,
`Position`, `FreeQ`

`ToString`'s ceiling is **1,550-1,600**, by far the worst. The other nine are
recorded only as "below 4,097"; bracketing them is part of this work.

**Do not convert what already clears the bound.** `Expr`'s own `Display`
handles 4,097 on 2 MiB with headroom to roughly 5,000-8,000, and `Depth`,
`LeafCount` and `ByteCount` clear it too. `Display`, the derived traits and drop
glue sit in the implicit-and-universal category where three earlier iterative
designs cost 8-49% and were rejected (`TODO.md:1061-1065`). Leave them alone.

## Start with `ToString`, and here is why it is not merely worst

Error messages reach the fat renderer: `format_message_default_fragment`
(`crates/rum-runtime/src/runtime/messages/message_format.rs:165-176`) calls
`render_input_like_expr` for a `HoldForm`, `HoldCompleteForm`, `Defer` or
`Unevaluated` wrapper. So a message that embeds one of those around a deep value
renders through the path that gives out at 1,550. Until that path is total,
every other depth failure has a reporting route that can itself fail.

The two renderers differ, and only one is short:

- `FullForm` is `Ok(Expr::string(expr.to_string()))`
  (`crates/rum-stdlib/src/structure_ops/surface.rs:78-89`) — `Expr`'s `Display`,
  which clears the bound.
- `ToString` goes through `ToStringFormat` dispatch
  (`crates/rum-stdlib/src/strings/text/builtins.rs:624-640`) into
  `render_to_string_expr_with_precedence`
  (`crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:25-34`).

So this is scoped to the precedence-aware formatter, not to rendering in
general. Establish where its per-level cost actually goes before rewriting it —
if a cheap structural change recovers the needed factor without a full explicit
stack, say so and take it.

## Deliverable

1. **Each of the ten completes at physical height 4,097 on a 2 MiB thread**, in
   release.
2. **A bracket for each**, before and after, so the report shows what each
   conversion bought. Bracket the nine that brief-424 left at "below 4,097".
3. **A release-only depth test per converted traversal**, using the existing
   harness `run_on_two_mib_thread`
   (`crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:375-383`) and
   marked `#[ignore = "release-only deployment-stack probe; run explicitly with
   --release"]`, matching plan-330's precedent
   (`left_chain_lowering.rs:562-587`). Debug frames are fatter; a 4,097 test
   must not run in the ordinary debug gate.
4. **A statement, per traversal, of what you changed and why that was the
   cheapest change that reaches the target.** An explicit work stack is one
   answer; a smaller frame is another; tail-position restructuring is another.

## What must not change

This is the part I will gate hardest.

- **No observable behavior change except that deeper values now work.** Same
  results, same messages, same evaluation order.
- **`Cases`, `Level` and `Position` produce results in traversal order** and
  take level specifications. An iterative rewrite must preserve the exact
  order and the exact level semantics, including negative and `Infinity` specs.
- **`MemberQ`, `FreeQ` and `Count` short-circuit.** Preserve *where* they stop,
  not merely what they answer — a pattern can carry a `Condition` whose guard
  runs during matching.
- **`Map` and `Apply` are `Listable`-adjacent and level-aware.** Preserve
  argument evaluation order and held-argument handling.
- **Do not weaken an existing test** to accommodate a rewrite. If a test's
  expectation genuinely must move, STOP and report it with the old and new
  values; that is my decision.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs`
- `crates/rum-stdlib/src/strings/text/builtins.rs` — `ToString` only
- `crates/rum-stdlib/src/functional_ops/apply_map/` — `Map`, `Apply`
- `crates/rum-runtime/src/runtime/tree_walkers/` — `Level`, `Position`
- the rewrite paths for `ReplaceAll` and `Cases`
  (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs`,
  `crates/rum-stdlib/src/rewrite_builtins.rs`,
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`)
- `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs` — `MemberQ`,
  `Count`, `FreeQ`; `position.rs` and `level.rs` for `Position` and `Level`
- module roots of any of the above whose exported types change
- test files and test module roots in either crate
- this brief, to append your report

A file that needs only the mechanical follow-on of a signature change is in
scope. Anything more, **STOP and report**.

Files you MUST NOT touch: `crates/rum-core/src/expr.rs`'s `Display`, derived
traits, or drop glue; the memory-growth classifications or the `Preflight`
allowlist; `debug_tools/resources/`; any `ResourceLimits` default or the stride;
`AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending your report;
`.git`, git config.

## Hard rules

- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **Do not raise a stack, spawn a wider thread, or move work off-thread** to
  reach the target. The traversal must fit the budget it is given.
- **No default-path cost.** These are ordinary builtins on ordinary paths; an
  explicit work stack that allocates per call where recursion allocated nothing
  is a cost. Measure it (below) rather than assuming it is free.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **Bound every probe run** with `scripts/run-bounded.py --timeout SECONDS`,
  sized from a prior measured run of the same shape.
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-422 gate.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline --workspace
cargo test --offline --release -- --ignored
```

The third runs the release-only depth probes, including plan-330's existing
ones; they must stay green alongside yours.

Then a microbench A/B, base against candidate in one window, because ten
ordinary builtins changed shape:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the rows for labels exercising the converted builtins and their
same-tier neighbours, plus `replaceall-simple` and the matcher rows.
**Name your control labels before looking at the candidate numbers.** A
whole-run offset moves every label together and a lopsided direction split is
not evidence; if the spread is noisy, raise the iteration count rather than
reading the noise. Profiling env off for every timing number.

## Report format

1. What changed — per traversal, with the mechanism and why it was the cheapest
   change that reached the target.
2. The bracket table: each traversal before and after, on a 2 MiB stack.
3. Semantic preservation: how you established order, level and short-circuit
   behaviour are unchanged, beyond the suite passing.
4. Verification — every command, verbatim output.
5. The A/B rows with named controls.
6. Anything whose expectation had to move — reported, not decided.
7. Stop-loss / open questions.
