# brief-520 — how much of the 43x can a maintained total actually remove?

`Scope: measurement only.` A deliberately-incorrect stub is built to
establish an upper bound. **Nothing here is a candidate to land**, and the
stub must be removed before you finish.

## Why this comes before the work

The user ruled (2026-09-22) that the armed memory ceiling is the security
model's own cost, not a niche opt-in — `SECURITY.md` requires a host
evaluating untrusted scripts to arm it — and that an unarmed cost of up to
about **0.7%** is therefore acceptable if it buys an affordable armed path.
brief-518's measurement stands; only its acceptance criterion was wrong.

But brief-518's route is the **enabler**, not the fix. It carries an
accounting mode; the 2.26x-43.34x is the **O(session) walk**, which only a
maintained total removes, several briefs later. **Spending 0.7% on plumbing
whose payoff is unmeasured would be backwards.**

So: measure the payoff's **ceiling** first. If even the best imaginable
maintained total does not remove most of the 43x, the line is not worth the
0.7% and the answer is different.

## The measurement

Build an armed variant in which the session half of the checkpoint is
**stubbed to O(1)** — have `current_memory_proxy_bytes()` return a cached or
constant value instead of walking. This is **deliberately wrong**: it does
not enforce anything correctly, and it exists only to bound the timing. Say
so in the report, in those words.

The expression term (`expr_memory_proxy_bytes(expr)`) and everything else
stay exactly as they are; only the O(session) term is removed.

Then measure, against today's `main`:

- The **nine armed shapes** brief-514 used, so the numbers compose directly
  with its table: sessions of bootstrap-only, `Range[1000]` and
  `Range[10000]`, each at N = 16, 128 and 1024 checkpoints. Those workloads
  are at `target/brief-514/armed-*.rum`; reuse them unchanged.
- Report, per shape: `main` instructions, stubbed instructions, and the
  **fraction of the added cost removed**. brief-514's base and final columns
  are the reference; state them alongside so the three-way comparison is
  visible.

Also report the **residual**: with the session walk gone, what does an armed
checkpoint still cost, and what is it made of? That residual is the floor
the real implementation would land on, and it decides whether the remaining
gap is worth C, D and E.

## What I want back

1. The three-way table and the fraction of the armed overhead the stub
   removes, per shape.
2. The residual per-checkpoint cost with the walk removed, and its
   composition.
3. **Your verdict on whether the line is worth the 0.7%**, argued from those
   numbers. If the stub removes only part of the overhead, say what the real
   ceiling of the design is.
4. Anything that makes the stub an *over*-estimate of the achievable win —
   work a real maintained total would have to do that the stub skips, above
   all the charge and release bookkeeping at mutation sites. The stub pays
   none of that, so the true win is strictly smaller; bound it if you can.

## Hard rules

- **The stub is throwaway.** Remove it before you finish and prove the tree
  is clean — `git status --porcelain` in the report.
- Do NOT restore or re-run brief-518's prototype here; this brief answers a
  different question and mixing them makes both unreadable. The patch stays
  where it is at `target/accounting-route-518/prototype.patch`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- Instruction counts only, via `taskset -c 2 perf stat -e instructions:u -x,
  --`; `cpu_core/instructions/u`, `100.00` multiplex, three runs per arm,
  release, profiling env OFF, both arms built at the same path, stdin closed.
  **Do not run the timing panel.**
- Verify counts scale with the workload before trusting them, and **explain
  any anomaly before accepting it** — brief-519's favourable numbers turned
  out to be an inlining artifact, and that was only caught because the
  anomaly had to be explained rather than banked.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No network. Touch nothing outside this repo; read `target/brief-5*/` and
  `target/accounting-route-518/` freely, overwrite nothing there.
- **Stop-loss, keyed on progress.** Same step failing twice with no new
  information means stop and report.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. The three-way table, per shape, with the fraction removed.
2. The residual cost and its composition.
3. The verdict on whether the 0.7% buys enough.
4. Why the stub over-estimates, and by roughly how much if you can bound it.
5. `git status --porcelain` proving the stub is gone.
