# brief-514 — check the memory model at every armed checkpoint

## Goal

An 8 MB `MemoryConstrained` currently lets
`Nest[StringJoin[#, #] &, seed, 16]` complete, returning 65 536 000
characters at ~358 MB peak RSS, with no abort and no message — while the same
ceiling aborts a single large `StringRepeat`. The limit is armed and works;
a sequence of individually non-amplifying steps walks through it, because the
full model walk happens once per 64 armed checks
(`crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:9`).

Implement brief-513's option A: **while a limit is armed, every existing
checkpoint checks the model.** Then measure what that costs, on both paths,
and report.

## What this is not

- **Not a new heuristic.** If the measured armed cost is unacceptable, that
  is a **report-and-stop**, not a licence to invent adaptive sampling,
  sampling by expression size, or any other policy. brief-513 established
  that last-sample adaptive stride has no useful factor bound; do not
  reintroduce it under another name.
- **Not a per-builtin patch.** A `StringJoin` precheck would catch this
  particular example and is a real narrow remedy, but a collection of
  named-builtin patches is not a repair of the sampling gap. Out of scope.
- **Not the `Incremental` contract.** That is the long-term route and its
  accounting model is unsettled; `TODO.md` records it as unbuilt. Out of
  scope.
- **Not the interner quota.** Separate line, blocked on a user ruling.

## The repair

1. Inside `enforce_active_resource_limits_for_expr`, within the existing
   `if let Some(limit) = self.current_memory_limit` branch, perform the
   projected walk on **every** armed check instead of one in 64. Retain the
   time-limit ordering, the saturating arithmetic, the existing proxy model,
   and the unstrided projected-byte prechecks exactly as they are.
2. Remove `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE` and the countdown field if
   nothing else uses them — and say in the report whether removing the field
   changed `Session`'s layout, because that can move the default path even
   though no algorithm did.
3. **Replace the false comment** at `constraints.rs:5-8`. It currently claims
   "between sampled walks, overshoot is bounded by the existing per-step
   element, integer, and iteration budgets". Those bound counts; the quantity
   at risk is bytes. Replace it with the exact guarantee the new code makes,
   including its limitation: a single construction larger than the limit is
   still only caught where a projected precheck exists.

## The falsifier, which must be watched failing

Add a test that the reproduction aborts in band under its ceiling, paired
with the control that already aborts. Then **demonstrate the test failing on
the unchanged behavior** — restore the stride locally, show the red, restore
the repair — and paste both runs. A check nobody has watched fail is not
known to work; this project requires the proof, not the intention.

Keep the fixture small enough that the suite stays fast: the seed and
doubling count only need to cross the ceiling, not reach 65 MB.

## The measurement, which decides acceptance

Two questions, reported separately, because they have different acceptance
arguments.

**Default path (no limit armed).** brief-513's source reading says the
countdown and walk already live inside the armed branch, so replacing its
interior adds no algorithmic work when unarmed. That is a source claim, not a
measurement — layout, inlining and the removed field can still move it.
Measure it:

```
taskset -c 2 perf stat -e instructions:u -x, -- <binary> <workload>.rum < /dev/null
```

`cpu_core/instructions/u`, `100.00` multiplex coverage, three runs per arm,
release profile, profiling env OFF, both arms built at the same path, stdin
closed. Use `target/brief-501/smoke50000.rum`, which is calibrated
(base reads 1 376 080 468 instructions, within-arm spread 0.00005%), plus at
least two symbol- and evaluation-heavy labels lifted verbatim from
`scripts/core-language-microbench.sh`. **Any default-path regression is a
defect**, not a cost offset by an armed safety benefit.

**Armed path (a limit set).** This is the real question and brief-513 warns
it may be expensive, especially with retained trees, since the walk is
O(session). Measure at least: a small session, a session holding a large
retained structure, and a loop doing many armed checkpoints. Report the
numbers; do not judge them. **A large armed cost is a director decision, not
your stop condition** — report it with the shape of the curve.

Verify the instrument before trusting it: instruction counts must scale with
the workload's `N`. A `Do` over 200 000 iterations silently hits the 100 000
iteration ceiling and measures process startup instead.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`
- the runtime test module that owns resource-limit behavior, for the
  falsifier
- `docs/guide/resource-limits-and-sandbox.md`, if the guarantee it documents
  changes
- this file, for your report

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. A change that is not compiler-forced and not listed is a stop.

Do NOT touch: `crates/rum-stdlib/**`, the builtin catalog or its growth
declarations, `Expr`/`SharedExpr`/`SharedArgs`, `scripts/**`, `AGENTS.md`,
`TODO.md`, `PROGRESS.md`, `Cargo.toml`, `Cargo.lock`, `.git`, the
ownership-graph inventory fixtures.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

**Expected state on `main`:** 2 491 passed, 0 failed, 57 ignored on a host
that can bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if
your sandbox denies loopback you will see nine native ZeroMQ failures —
report them as environmental and say plainly those tests did not execute.
Any other failure is yours.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`; a recorded
run of the same shape took about 160 s.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. Leave changes in the
  working tree; the director gates and commits.
- **No new `unsafe`.**
- **No default-path cost.** The diagnostics hard line is unchanged: no
  unconditional walk, counter, mutex, allocation or branch paid when no limit
  is armed.
- **Do not weaken or delete a test to make the suite pass.** A test that
  encodes the strided behavior is a contract conflict to surface.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Do not overwrite anything
  under `target/brief-501/`, `target/brief-502/`, `target/brief-508/` or
  `target/brief-513/`; read them freely.
- Do not measure while another CPU-heavy workload runs. Build first, measure
  after.
- **Stop-loss, keyed on progress.** If the same step fails twice and the
  second attempt produced no more information, stop and report.
- Paste real verbatim output, including failures.

## Report format

1. What changed, per file, with line ranges and a one-line why; plus whether
   removing the countdown field moved `Session`'s layout.
2. The falsifier, with both runs: green on the repair, red on the restored
   stride.
3. Default-path numbers, with the scaling check that proves the workloads ran.
4. Armed-path numbers, with the shape of the cost — reported, not judged.
5. Verification, verbatim, with the complete failure set.
6. Anything you refused, and open questions.
