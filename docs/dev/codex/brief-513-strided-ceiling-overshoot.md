# brief-513 — design: the memory ceiling that a loop walks straight through

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and drafted implementation briefs. Write no code.

## The finding, director-reproduced 2026-09-22 with a calibrated control

The 2026-09-02 audit's second HIGH. Both halves re-probed; one of them needs
correcting before any repair is designed.

**It reproduces, and the ceiling does nothing:**

```
gSeed = StringRepeat["x", 1000];
gOut = MemoryConstrained[Nest[StringJoin[#, #] &, gSeed, 16], 8000000];
Print[StringLength[gOut]];
```

An 8 MB modelled ceiling; the expression **completes successfully**, returns a
65 536 000-character string, and peaks at **~350 MB RSS**. No abort, no
message.

**The control proves the ceiling is armed and does work:**

```
gOut = MemoryConstrained[StringRepeat["x", 100000000], 8000000];
```

Same 8 MB ceiling, single large allocation → **`$Aborted`**.

So the mechanism is not a dead limit. It is a limit that a sequence of
individually-innocent steps walks straight through.

## The audit's mechanism is wrong, and the repair depends on this

The audit said `eval_string_join`
(`crates/rum-stdlib/src/strings/text/builtins.rs:13-42`) "has no
projected-byte precheck at all, unlike `StringRepeat`". That framing points
at the wrong repair.

`StringJoin` is declared `Preflight`
(`crates/rum-stdlib/src/catalog/memory_growth_contract.rs:546`), and the
enforcement for that contract is: a `Preflight` builtin whose result exceeds
its arguments plus an allowance, and which did not project, panics
(`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:403-415`). **`StringJoin`
does not amplify per call** — concatenation rearranges bytes, so
`result_bytes ≈ argument_bytes` — which is why the enforcement correctly
passes it and why it is not on
`KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS` (`eval_dispatch/mod.rs:7-20`).

The overshoot comes from **repeating a non-amplifying step between strided
full walks**: `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64`
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:9`),
and a doubling reaches 2 GB from 1 MB in about eleven steps. Adding a
precheck to `StringJoin` would not have caught this.

**The comment above the stride states the false premise plainly**
(`constraints.rs:5-8`): "between sampled walks, overshoot is bounded by the
existing per-step element, integer, and iteration budgets." Those budgets
bound **counts**. The quantity at risk is **bytes**. Whatever else this line
produces, that comment must stop asserting a bound that does not exist.

## What I want designed

### 1. The option space, scored

For each: what it bounds, what the residual overshoot becomes, its
**measured or estimated** hot-path cost, and how it is falsified.

- **Unstride the full walk while a limit is armed.** Simplest and strictest.
  The stride exists for a reason — establish what walking every armed check
  actually costs, rather than assuming it is prohibitive.
- **Adaptive stride.** Sample every time once the last observed usage is
  within some factor of the limit, stride widely when far below. Bounds
  overshoot to roughly that factor instead of leaving it unbounded. Say what
  the factor must be and what it costs.
- **An incremental byte counter** maintained at construction sites rather
  than walked. This is the `Incremental` contract
  (`crates/rum-runtime/src/runtime/metadata.rs:346-348`), which
  `TODO.md`'s memory-accounting entry records as **still unbuilt** — so
  scope it honestly rather than treating it as available.
- Anything else you find. Include the option of bounding only the
  *doubling-shaped* case if you can define it precisely, and say why that is
  or is not principled.

### 2. The hot-path question, answered with a number or an explicit gap

The stride is a performance decision, so its removal or narrowing is a
hot-path change and this project gates those with measurement, not
intention. Specify the A/B the implementation must pass — instruction counts
via `taskset -c 2 perf stat -e instructions:u`, not the timing panel — and
name the labels. Note that the check only runs **when a limit is armed**, so
say plainly whether the default path moves at all; if it does not, the
acceptance argument is very different and should be made explicitly rather
than left implied.

### 3. The falsifier

This project's standing rule: a check nobody has watched fail is not known
to work. Design the test that fires on the reproduction above, and the proof
that it fails when the repair is removed. The reproduction and its control
are both given above — use them.

### 4. The scope boundary

`MemoryConstrained` is the in-language frame; `eval_memory_limit` is the
per-request ceiling. Say whether the repair covers both, and do **not**
silently widen into the process-global interner quota, which is a separate
line blocked on a user ruling (`TODO.md` §The interner ruling of 2026-09-21).

## Method requirements

- Read `AGENTS.md`, `TODO.md` §Opened by the 2026-09-02 security audit, and
  the memory-accounting entries before proposing anything.
- **Re-probe rather than inherit.** The audit's mechanism was wrong here and
  the director's own `Module` premise was wrong two briefs ago. Verify any
  claim you lean on, including the ones in this brief, and say how.
- Where you assert a cost, say whether it is measured, inferred or a guess.
- Name what would falsify your recommendation.

## Hard rules

- **No source change.** No `.rs`, no script, no fixture, no doc except
  appending your report to this file. Scratch and `.rum` probes under
  `target/brief-513/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Bound anything long with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run of
  the same shape.
- A release `rum-cli` built from `main` is at `target/brief-502/rum-base`;
  reuse it rather than rebuilding, and do not overwrite anything under
  `target/brief-502/` or `target/brief-508/`.
- **Stop-loss, keyed on progress.** If a probe fails twice and the second
  attempt produced no more information, stop and report what you have.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. Confirmation or correction of the mechanism stated above.
2. The option table, scored.
3. The recommendation, its reasoning, and what would falsify it.
4. The falsifier design and the required A/B.
5. Drafted follow-up briefs, and any director decisions you hit.
