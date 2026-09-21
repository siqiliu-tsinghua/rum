# brief-506 — how much does rum actually intern?

## Goal

Before anything is built, measure what the real system interns. brief-505's
Draft A says outright that it cannot answer this — "do not infer a legitimate
workload working set from that subtraction" — and the director's D1 ruling
(2026-09-22) turned the proposed 8 MiB default from a number into a
measurement obligation. This brief is that measurement.

The deliverable is a census, in bytes and distinct pairs, of what the
process-global symbol interner holds under four conditions. The director sets
the quota default from it. **No production change lands. Nothing is
proposed. Instrumentation built here is throwaway and must be stated as
such.**

## Why this comes first

The mitigation is a process-wide byte cap on the interner. If the ordinary
test suite or the microbench panel already interns more than a plausible cap,
the design is not merely mis-tuned, it is wrong, and that must be known before
Draft A's storage proof is written rather than after. A premise check precedes
the build.

## What is already settled, so you do not re-derive it

- `Symbol` does not change — not its size, `Copy`-ness, id-based fast-accept
  equality, or its lifetime-free `'static`-compatible public type
  (user ruling 2026-09-21, `TODO.md` §The interner ruling of 2026-09-21).
- There is no reclamation and no per-session attribution. The quota is
  process-wide.
- **The charging model is a proxy, and only a proxy** (director ruling D2):
  exact Rust allocation-layout bytes plus a conservative fixed reserve,
  explicitly excluding allocator metadata, rounding and RSS. Use that model
  here so the census is denominated in the same units the quota will be. Say
  so, and say where it under- or over-counts.
- Director-verified 2026-09-22, and you should confirm rather than assume:
  **`Module` gensyms do not grow the interner** — `Symbol::with_identity`
  varies only the numeric identity while `intern_symbol_text` is keyed on
  `(context, name)` (`crates/rum-core/src/expr.rs:80-96,115-122`) — while
  **`Unique[]` mints fresh text on every call**
  (`crates/rum-runtime/src/runtime/scoping/unique.rs:42`).

## The instrumentation

Add a throwaway census to the interner: counts of distinct strings and pairs,
and the D2-model byte total, reported at process exit behind an env var
(`RUM_INTERNER_CENSUS=1` or similar). Keep it simple — this build is never
committed and never measured for speed, so an atomic or even a mutex inside
the existing interner lock is fine. **Do not propose it for landing** and do
not shape it as if it were production diagnostics.

Report the exact byte formula you used, term by term: the `str` bytes, the
`SymbolText` allocations, the `HashSet`/`HashMap` backing storage at the
capacity actually reached, and anything else the model charges. Where the
real container's backing bytes are not exactly knowable from safe Rust, say
so and give a bound with its direction, rather than a number that looks
exact.

## The four measurements

Write each into your report before starting the next.

### 1. Bootstrap

A session created and `install_core` run, nothing else evaluated. Report
distinct strings, distinct pairs, and bytes. This is the floor any cap must
clear by a wide margin, and it is the number the reserved-vocabulary work in
the next brief sizes itself against.

Also report the breakdown between `System\`` and other contexts, and how many
of the pairs come from the stdlib catalog rather than from
`CANONICAL_CORE_HEADS` (`crates/rum-core/src/heads.rs:112`).

### 2. The ordinary test suite

`cargo test --workspace --no-fail-fast`, census enabled. Report the **maximum
over test binaries**, not a sum — the quota is per process and each test
binary is one. Name the binary that holds the maximum.

Expect one pre-existing failure and do not treat it as yours:
`rum-stdlib`'s `tests::ownership_graph::production_inventory_is_exact` is
**green on `main`**; if you see it fail you are not on `main`, so stop and say
so. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if loopback binding is
unavailable in your sandbox, report those native ZeroMQ failures as
environmental and say plainly that those tests did not execute.

Derive the timeout from a recorded run of the same shape; brief-500 used 900 s
against a ~97 s workload and did not time out.

### 3. The microbench panel

`scripts/core-language-microbench.sh`, census enabled, release build. Report
the total at exit. This is the heaviest ordinary workload the repository has,
and it deliberately builds symbol-sorting fixtures at widths 4 000 and 16 000
(`scripts/core-language-microbench.sh:884-891`) — so it is also a realistic
upper bound on how much a legitimate program interns on purpose.

### 4. The language's own unbounded consumer

`Unique[]` in a loop, and separately `Symbol[StringJoin[…, ToString[i]]]` in
a loop, at a few sizes. Report **bytes per call** for each, and then the
question the director actually needs answered:

> at a candidate cap of 8 MiB, and at whatever the bootstrap floor turns out
> to be, **how many `Unique[]` calls does a process get before refusal?**

Give the arithmetic. Give it for 8 MiB and for two other plausible caps of
your choosing, so the director can see the shape of the trade rather than one
point.

## What I want back

1. The four measurements, each with its byte formula and its uncertainty.
2. The `Unique[]` budget table.
3. **Your judgement on whether a process-wide byte cap is a workable
   mitigation at all**, given those numbers — and if it is not, say so
   plainly. A refutation here is worth more than a tuned number, and this
   line has already accepted one refutation on evidence.
4. Anything you found that changes the plan in brief-505 §1.

## Scope

- You MAY add throwaway census instrumentation to
  `crates/rum-core/src/expr.rs`, and whatever the compiler forces from it.
  Every file in the report.
- You MAY write under `target/brief-506/**` and append your report to this
  file.
- **MUST NOT touch**: `scripts/**` — the panel must stay byte-identical or
  its numbers are not comparable to recorded ones; `docs/**` except this
  file; `AGENTS.md`; `TODO.md`; `PROGRESS.md`; `Cargo.toml`; `Cargo.lock`;
  `.git`; the ownership-graph inventory fixtures.
- The instrumentation is throwaway. Leave it in the working tree; the
  director discards it.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. (`git checkout` restores
  from the index, which on uncommitted output is a delete.)
- **No new `unsafe`**, including to inspect a container's backing storage. If
  a number needs `unsafe` to obtain, bound it instead and say which direction
  the bound errs.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Bound anything long with
  `scripts/run-bounded.py --timeout SECONDS -- <command>`, sized from a
  measured run of the same shape.
- **Stop-loss, keyed on progress.** If a measurement fails twice and the
  second attempt produced no more information than the first, stop and report
  what you have. Partial census plus the `Unique[]` budget is still a useful
  result.
- Paste real verbatim output. Never fake or interpolate a number, and never
  quote one you could not take.

## Report format

1. The census: four measurements, byte formulas, uncertainties.
2. The `Unique[]` budget table with its arithmetic.
3. Your verdict on whether the cap is workable.
4. What changed in your understanding of brief-505 §1.
5. Every command run, and the list of files you touched.
