# brief-504 — design: close the interner finding without touching `Symbol`

`Scope: NONE — no source change.` This is a design-only brief. You survey,
reason, and return a plan plus drafted follow-up briefs. You write no code.

## Goal

The 2026-09-02 security audit's HIGH finding is still open: the symbol
interner is process-global, leaks by construction, and is invisible to the
per-session memory ceiling. The remedy that was ruled — session-scoped
ownership, option A1 — has now been **refuted by measurement**, and its
ruled fallback A2 does not close the finding. Design what does, under a
constraint that did not exist when A1 was chosen.

Return a plan, its open questions, and the follow-up briefs to execute it.

## The finding, unchanged

- `symbol_interner()` (`crates/rum-core/src/expr.rs:47-55`) is a process-wide
  `OnceLock<Mutex<SymbolInterner>>`; `intern_text` (`:57-70`) `Box::leak`s
  every new name and `intern_symbol_text` (`:80-96`) leaks a `SymbolText` per
  `(context, name)` pair. Neither `strings` nor `pairs` has a removal path.
- `current_memory_proxy_bytes`
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50`) walks
  session-owned structures and never mentions the interner, so
  `eval_memory_limit` cannot see the growth.
- `HostSession::reset()` (`crates/rum-host/src/session.rs:161-174`) rebuilds
  the session and does not touch it.
- Director-reproduced: under a `{Pure}`-only policy, eight requests each
  interning 20 000 fresh names took RSS from 5.6 MB to 57.5 MB monotonically,
  and `reset()` returned only to 47.0 MB. For a long-lived multi-tenant host
  this is an exhaustion vector no documented kernel mechanism can close.

## The constraint the spike established — treat these as facts, not as things to re-derive

Briefs 499-503 built A1's representation and measured it. The reports are in
this directory; the numbers you need:

- **Any owning `Symbol` costs at least +10.93% instructions on ordinary
  evaluation.** That is `V-arc` — `Arc<SymbolText>` with owned strings but
  stage1's dense-`u32` equality retained — and it is the *cheapest owning
  variant that can exist*, because it keeps the fast-accept equality that an
  owning representation normally has to give up.
- The full A1 candidate is **+14.33%**: +10.93% from the owning
  representation and +3.42% from the equality demotion, adding with a
  residual under 0.1 percentage points across twelve workloads.
- `V-arc` is **not a valid A1**. Its dense-id equality is sound only because
  the interner remains process-global and assigns one id per distinct text.
  Session-scoped ownership destroys that guarantee, so a real A1 pays both
  halves.
- This project has rejected representation changes at 2-7%, 5-14% and 30%,
  and accepted +0.714% and +0.606% only to remove a process kill.

**Therefore: `Symbol` stays a 24-byte `Copy` handle with a dense id and an
id-based fast-accept equality. The ownership and the accounting must live
somewhere else.** A proposal that changes `Symbol`'s size, its `Copy`-ness,
or its equality fast path is refuted before it is written; say so and move
on rather than re-litigating it.

Two things the spike did **not** refute, and you may use: the migration to a
non-`Copy` `Symbol` is nearly free in itself (+0.06% to +0.84%), and the
depth contract survives an `Arc` (release slope 3.36% *smaller*, margin
44.84% over 4 097). Neither rescues A1, but do not cite them as obstacles.

## What I want back

### 1. The option space, scored

At least these, plus any you find. For each: what the per-session ceiling can
see, what `reset()` reclaims, what a multi-tenant host actually gets, the
hot-path cost measured against the +10.93% floor above, **enforceability —
compiler-enforced or convention-enforced**, and what it breaks.

- **A2 as ruled** — account the global table and refuse, replacing the `u32`
  `expect` at `:88` with a recoverable refusal. Score it honestly, including
  brief-414's own objection that charging the whole table makes one tenant
  pay for another while charging only insertion deltas leaves permanent
  uncharged bytes after `reset()`.
- **Reclaimable global interner** — `Weak` entries, or epoch/generation
  invalidation, with `Symbol` unchanged. **The hazard you must analyse
  head-on: a `Symbol` that has escaped into a host's `Expr` outlives the
  session, and a reclaimed dense id that is later reissued makes that
  `Symbol` silently denote different text.** A design that cannot exclude
  that is not a candidate; say so plainly rather than hedging.
- **Session-scoped dense ids with a mandatory ingress import** — `Symbol`
  keeps its layout and `Copy`, each session owns its table, and every symbol
  crossing into a session is imported. The question that decides it is
  whether that boundary can be made a compile error rather than a rule people
  remember; `lower_syntax_expr`
  (`crates/rum-runtime/src/runtime/lowering.rs:7-18`) is public and takes a
  resolver closure rather than a `Session`, which is where to start looking.
- **Bound and refuse without reclaiming** — cap the interner, fail closed
  with a message naming the limit, and leave the bytes unreclaimed. This does
  not close the finding but converts unbounded growth into a documented
  bounded limit; score what it does and does not buy.
- **Kernel does nothing; document and mitigate in the host.** Score it
  honestly rather than as a strawman — if it is genuinely the best available,
  say so and say what the documentation must then state.

### 2. The escaped-symbol lifetime question, answered once

Every reclaiming option collides with it, so answer it as its own section
with evidence: where can a `Symbol` outlive the session that interned it?
Start from request responses
(`crates/rum-runtime/src/runtime/model/requests.rs:89-134`), suspension
frames (`crates/rum-runtime/src/runtime/suspension/core.rs:74-87`), snapshot
and restore, and the jupyter worker thread
(`crates/rum-jupyter/src/adapter/worker.rs:312-341`). Say which of these can
hold a `Symbol` past a `reset()` today.

### 3. A recommendation, with the reasoning that produced it

One option, argued. If your recommendation is that no option closes the
finding at acceptable cost, recommend that — a refutation is a result, and
this line has already accepted one. The standard here is correctness,
semantic loss, whether the fix is structurally unavoidable, enforceability
and measurability. **Cost of implementation is not an argument**: this
project does not score an option down for its size, and "398 call sites" is a
description of the work, not an objection to it. A fix the compiler enforces
beats one that relies on people remembering, even an order of magnitude more
expensive.

### 4. Drafted follow-up briefs

In `TEMPLATE.md` shape, usable as drafts — not step descriptions. The
director rewrites them before dispatch; they are raw material.

Every claim in all four sections carries `path:line` evidence.

## Method requirements

- Read `AGENTS.md` first, and the three reports in this directory
  (`brief-501`, `brief-502`, `brief-503`) before proposing anything.
- **Re-probe before you rely on a stale statement.** brief-414's Gap A
  analysis is a year of this codebase old in places, and its microbench line
  citations were already stale once. Where you rely on one of its claims, say
  you verified it and how.
- Where you assert a cost, say whether it is measured, inferred from the
  spike's numbers, or a guess. Do not present the third as the second.
- Name what would falsify your recommendation.

## Hard rules

- **No source change.** No `.rs` file, no script, no fixture. The deliverable
  is the plan.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout`, or
  touch git config or the index.
- You MAY append your report to this file and write scratch under
  `target/brief-504/`. Nothing else.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- Paste verbatim evidence for anything you measured or ran. Never fake a
  number, and never quote one you could not take.

## Report format

1. The option table, scored on the axes named above.
2. The escaped-symbol lifetime answer.
3. The recommendation, its reasoning, and what would falsify it.
4. The drafted follow-up briefs.
5. Open questions and anything you could not settle from the source.
