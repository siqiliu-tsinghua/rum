# brief-522 — one field, end to end: mode plus maintenance behind it

## Goal

Prove the architecture on a **single field** before it is built across 52.

Restore brief-518's accounting-mode route, put maintenance of **one owner —
the evaluation stack — behind the mode**, and measure all three axes. If the
slice works, the remaining 51 fields are repetition; if it does not, we have
spent one run instead of a line.

## What the three refutations established, so none is repeated

- **brief-518 — the mode route costs 0.465994% / 0.697458% / 0.622776%
  unarmed** and works across all four crossings. Its acceptance criterion was
  wrong, not its measurement; under the user's corrected ruling (2026-09-22)
  about 0.7% unarmed is acceptable, because the memory ceiling is a
  documented MUST for untrusted input rather than off-by-default diagnostics.
  Its prototype is preserved at
  `target/accounting-route-518/prototype.patch`. **Restore it.**
- **brief-519 — inferring freshness from existing counters is fail-open**:
  42 of 52 walked fields have no covering counter, and it reused a stale low
  total on the reproduction itself. **Do not infer anything.**
- **brief-521 — maintaining unconditionally costs +2.6193% / +3.0584% /
  +2.3845% for the evaluation stack alone**, one field of 52. That is what
  the mode is for: **maintenance is 3-5x the mode's own cost, so it must not
  run unarmed.** brief-521's probe is at
  `target/brief-521/stack-probe.patch` and its 52-field charge/release
  inventory survives its refutation — **reuse that inventory**.

**The director's framing error is recorded so it is not repeated**: the claim
that "if we pay unarmed, the mode buys nothing" was wrong. The mode does not
buy a cheaper branch; it buys maintenance being conditional.

## The slice

1. **Restore the mode route** from brief-518's patch. It proved the four
   crossings; re-verify they still pass, do not re-derive them.
2. **Maintain the evaluation stack behind the mode.** brief-521 implemented
   exactly this owner unconditionally; reuse its charge and release
   boundaries — `push_accounted_evaluation_entry`,
   `replace_accounted_evaluation_stack`, the evaluator push, resume and
   abandon transfers, and rollback restoration — and make them run **only
   when armed**.
3. **Use the maintained value for that owner's term** in the armed
   checkpoint, leaving every other field walked as today. The checkpoint
   becomes partly maintained and partly walked; that is expected for a slice
   and must be stated as such.
4. **Keep the walker authoritative** for `MemoryInUse[]`-style observation.
   brief-517 built it as the oracle for exactly this.

## The cold-arm hazard, which is a first-class question

If maintenance runs only while armed, the total must be **built from scratch
when a session arms** — one full walk per arming. `MemoryConstrained` can be
entered and left repeatedly, so a loop of the shape

```
Do[MemoryConstrained[cheap, limit], {N}]
```

pays N walks. Today that shape also walks at every checkpoint inside, so the
order should be unchanged — but **measure it, do not assume it**. Report:

- the cost of a single arming (the cold walk),
- the `MemoryConstrained`-in-a-loop shape above at N = 16, 128 and 1024,
  against today's `main`,
- and whether any realistic shape ends up **worse** than today. If one does,
  that is a finding that outranks the slice's success.

## The drift falsifier, which must be watched failing

For the maintained owner: assert **maintained stack term == walker's stack
term** at every armed checkpoint under test configuration, across workloads
exercising evaluator push, resume, abandon, rollback and error exits.

Then watch it fail: **delete one charge site, show red; delete one release
site, show red the other way; restore both, show green.** Paste all three.
A drift detector nobody has seen fire is not known to work.

## The measurement

Recipe throughout: `taskset -c 2 perf stat -e instructions:u -x, --`,
`cpu_core/instructions/u`, `100.00` multiplex, three runs per arm, release,
profiling env OFF, both arms built at the same path, stdin closed. Verify
counts scale with the workload. **Explain any anomaly before accepting it.**
**Do not run the timing panel.**

- **Unarmed** — `target/brief-501/smoke50000.rum` plus two evaluation-heavy
  labels lifted verbatim from `scripts/core-language-microbench.sh`, scaled
  slopes. **Bar: about 0.7%**, and since only the mode runs unarmed, the
  number should land near brief-518's. **If it lands near brief-521's 2.4-3.1%
  instead, maintenance is leaking onto the unarmed path — stop and report.**
- **Armed** — the nine shapes at `target/brief-514/armed-*.rum`, unchanged.
  Report against brief-514's table and brief-520's ceiling. A slice that
  maintains one owner of 52 removes only that owner's share; say what share
  that is rather than presenting a partial number as the design's result.
- **Cold arm**, as above.

## Scope

MAY modify: what brief-518's patch touches, the accounting modules under
`crates/rum-runtime/src/runtime/debug_tools/resources/`, the evaluation-stack
charge and release sites brief-521 identified, their module declarations, the
corresponding runtime tests, and `docs/guide/resource-limits-and-sandbox.md`.

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. Not compiler-forced and not listed is a stop.

Do NOT touch: the other 51 owners — **this is a slice**; `Expr`/`SharedExpr`/
`SharedArgs`; the interner; the 165 per-builtin `Incremental` declarations;
`scripts/**`; `AGENTS.md`; `TODO.md`; `PROGRESS.md`; `Cargo.toml`;
`Cargo.lock`; `.git`; the ownership-graph inventory fixtures.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

**Expected on `main`:** 2 509 passed, 0 failed, 57 ignored on a host that can
bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; report the
nine native ZeroMQ failures as environmental if your sandbox denies loopback,
and say plainly they did not execute. Any other failure is yours.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`.

**Append your report to this file**; keep raw evidence under
`target/brief-522/`.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. Leave changes in the
  working tree; the director gates.
- **No new `unsafe`.**
- **No inferring freshness from existing counters** — that is brief-519's
  refuted route.
- **Do not weaken the occurrence model.** Shared subtrees are charged per
  occurrence.
- **Do not delete or weaken the walker.**
- **Do not extend to other owners.** One field, end to end.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No network. Touch nothing outside this repo; read `target/brief-5*/`,
  `target/accounting-route-518/` and `target/measure-achievable-520/` freely,
  overwrite nothing there.
- **Stop-loss, keyed on progress.** Same step failing twice with no new
  information means stop and report.
- Paste real verbatim output, including failures.

## Report format

1. Unarmed numbers first, against the ~0.7% bar, with the leak check.
2. The drift falsifier, all three runs.
3. Armed numbers, with this owner's share stated rather than implied.
4. Cold-arm numbers, including the loop shape, and whether anything is worse
   than today.
5. Verification, verbatim, with the complete failure set.
6. Anything you refused, and open questions.
