# brief-497 — bound AST-to-`Expr` lowering

The repair for the fail-open diagnosed in brief-496. Implementation brief; the
diagnosis is settled and is not to be re-derived.

## What is settled — take these as given

- **The phase is AST-to-`Expr` lowering.** Director-verified independently of
  the worker's source sequencing: a script reading
  `Print["before"]; x = Hold[f[1][1]…6000…]; Print["after"];` prints `before`,
  never prints `after`, and dies with `thread 'main' has overflowed its stack`.
  So the overflow is inside the deep form's processing, **not** at drop time,
  which would follow the second print.
- **The shape reaches it from ordinary source.** A call-head chain boxes the
  prior head without touching `MAX_DELIMITER_NESTING_DEPTH` or
  `MAX_OPERATOR_CHAIN_DEPTH` (both 128), so at 300 applications
  `Hold[f[1][1]…]` is accepted while the nested `Hold[f[f[f[…]]]]` of the same
  depth is refused in band.
- **The measured ceiling**, `ulimit -s 2048`, three runs per point: 4 097 and
  5 000 survive 3/3; **6 000 is killed 3/3**. That is 1.22x-1.46x the ruled
  contract height, failing open.
- **A parser cap cannot replace this repair**, which is the worker's argument
  and the director accepts it: lowering is publicly callable —
  `lower_syntax_expr` (`runtime/lowering.rs:7`) and the session path
  (`runtime/resolution.rs:67`) — so a host can hand in manually constructed
  syntax of any depth. **No parser cap is part of this brief**, and no
  user-visible language change: once lowering fails closed, deep chains produce
  an in-band error instead of a process kill, which is the whole requirement.

## Goal

`lower_syntax_expr_with` (`runtime/lowering.rs:13`) bounds its own recursion.
Above the bound it returns an `EvalError` — **fail closed** — and below it the
lowered result is byte-for-byte what it is today. The public free function and
the session path both inherit the bound.

## The mechanism — this line's established pattern, and why it applies

`lower_syntax_expr_with` takes `&SyntaxExpr` and a `resolve_symbol` closure and
**takes no `Session`**, so it cannot run user code. That is exactly the
condition under which this line permits a conversion to *redo* its traversal
(the rule from briefs 459-475). Use the proven shape: a **bounded recursive
front plus a cold iterative tail**, not a depth parameter threaded through
every arm.

It already returns `Result<Expr, EvalError>`, so failing closed needs no
signature change at the public boundary. Pick the error variant that an
embedder can classify and say why you picked it; do not invent a new one if an
existing depth/stack variant fits.

Two constraints from the same line, learned the expensive way:

- **On a hot function, move locals rather than control flow.** Two functionally
  complete iterative conversions were rejected at 2-7% because a depth
  parameter and an `Option` return on a much-consulted function is not free;
  outlining the cold branches instead cost nothing. Prefer the cold-outlined
  shape.
- **Put the check where the growth is.** The same strided poll cost ~8% at a
  walk's entry and nothing on the only branch that actually recurses.

## The certificate — it must force the tail, on a branched fixture

A certificate that a spine lowers is not enough; nothing weaker than the
following proves the iterative tail ran:

- a private `*_with_front_depth` entry point;
- a **branched** fixture — a call-head chain whose arguments also nest, so the
  traversal is a tree and not a single spine;
- front `0` → the tail handles everything; front `16` → the front handles the
  shallow part; **the lowered result identical in both, and identical to the
  current implementation's**;
- a unique deepest element whose lowering must propagate, asserted exactly —
  not a summary like `Depth` or `Length`, which is correct even when the
  transform never ran.

**Falsifier (mandatory):** make the tail a no-op (or return `Exhausted`
unconditionally) and show the branched certificate go red on the exact result,
then restore **by hand** — `git checkout` restores from the index and deletes
uncommitted work — and show it green.

**Probe the premise before relying on it**: if a proposed certificate would
pass identically on the pre-conversion build, it proves nothing and must be
redesigned. Say in the report which of your assertions fail on the old build.

## The ceiling gate — the numbers must move, and in the right direction

Report, at `ulimit -s 2048`, three runs per point, for both the pre-change and
post-change binaries:

| source | required after |
|---|---|
| `Hold[f[1]…]` at 6 000 | **in-band error or success — not a process kill** |
| `Hold[f[1]…]` at 4 097 and 5 000 | still succeed, same result as today |
| `Hold[f[f[…]]]` nested at 300 | still refused in band, message unchanged |

Do not push past 6 000 looking for the new ceiling; that bracket is the
director's.

## Performance

Lowering runs once per source form, so an instruction-count A/B is required
before this is credited — briefs 426, 431 and 434 were all rejected at the A/B
after being functionally complete.

```
taskset -c 2 perf stat -e instructions:u -x, -- <rum-cli> <workload>.rum < /dev/null
```

Read `cpu_core/instructions/u`, require `100.00` multiplex coverage, build both
arms at the same path, release profile, profiling env OFF, three runs per arm,
judge the delta against the observed within-arm spread (~0.02% on this host).
Run the CLI with stdin closed or probes hang. Allocation counters must be
identical.

**Two workloads, and the split is what decides acceptance.** The first dispatch
measured **+0.608%** on a parse-and-lower workload — but measured the
*superseded* layout and never re-measured the final one, so there is currently
no number for what is in the tree.

- **Lowering-dominated**: many ordinary small forms, parsing and lowering
  rather than evaluating. This is the worst case for this change.
- **Evaluation-dominated**: a small program that does real work, so lowering is
  a vanishing fraction. This is what almost every real program looks like.

Report both, for the layout actually in the tree. Acceptance is judged on the
split, exactly as the accumulator-chain work was: a cost confined to the
workload the change is about, with the ordinary case unmoved, is acceptable on
this line's precedent — brief-453 took +0.714% and brief-475 +0.606% to remove
a process kill, and this removes one too. A cost that shows up on the
evaluation-dominated workload is a different matter and must be reported as
such.

**Ignore the timing panel.** Its controls have moved +3.7% to +12.45% on this
host; it cannot resolve a question this size. Instruction counts decide, and if
counters and instructions agree, no timing number overrules them.

## Scope

Files you MAY modify: `crates/rum-runtime/src/runtime/lowering.rs` and
`crates/rum-runtime/src/runtime/lowering/binary.rs`, plus the test module for
the certificate. Touch the session path (`runtime/resolution.rs`) only if the
bound cannot otherwise reach it, and say why.

- `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` — **added
  2026-09-21 after a correct stop-loss.** The first dispatch forbade these, so
  the workspace gate could not pass: editing `lowering.rs` shifts pinned rows.
  That was the director's omission, the fifth of this shape on this line. They
  are now in scope under the disclosure rule: regenerate as discovery requires,
  list **every** content change with its witness, classified (a) position drift
  and closure/macro rows from this brief's own edits, or (b) anything else —
  and **(b) is a stop**, as is any new `UnknownHostPayload`, any removed row,
  or any reclassification. Never hand-edit a row to make a check pass.

MUST NOT touch: `crates/rum-syntax/**` — **no parser cap, no AST change**;
`accumulator_chain.rs`, `host_payload.rs`, `crates/rum-derive/**`, the
suspension traits, the ownership scanner; `rum-core`; `SECURITY.md`,
`AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**`, `docs/dev/codex/**`
except your own report; `.git`, git config, `Cargo.lock`.

The standing-debt correction in `TODO.md` is the **director's** edit; propose
the wording in your report instead.

## Verification (MANDATORY)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

If your sandbox cannot bind loopback, say so and name the skip you used.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run `git stash` /
  `git checkout` on a modified file.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **Do not probe past 6 000** or drive any new process termination.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No `#[cfg(test)]` divergence in production code.**
- **No default-path cost** beyond what the A/B shows and the director accepts.
- **Read the failure, not the exit status**: rum exits 1 on a syntax error and
  124 on timeout, and an exit-status-only probe already turned timeouts into
  phantom stack overflows once on this line.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27) for a weaker repair, but
  a measured hot-path regression is a defect to report, not to ship.
- If the same obstacle occurs twice, STOP and report.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one line why, and which error variant
   you chose for the closed failure and why.
2. The certificate: the branched fixture, both front depths, the exact-result
   assertions, and which of them fail on the pre-change build.
3. The falsifier, red then restored green, verbatim.
4. The ceiling table, before and after, verbatim.
5. The A/B: workload, both arms, spreads, delta, allocation counters.
6. `cargo fmt --check` and the workspace total.
7. Your proposed wording for the `TODO.md` standing-debt correction.
8. Stop-loss / open questions.
