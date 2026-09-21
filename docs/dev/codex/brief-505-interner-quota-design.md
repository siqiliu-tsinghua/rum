# brief-505 — design: a bounded, loud interner under the 2026-09-21 ruling

`Scope: NONE — no source change.` Design-only. You survey, reason, and return
a plan plus drafted follow-up briefs. You write no code.

## Goal

The user ruled on 2026-09-21, after brief-504's option survey: **`Symbol`
keeps its lifetime-free public type**, on top of its 24-byte `Copy`
representation and id-based fast-accept equality. brief-504 established that
under those combined constraints **no in-process reclaiming fix exists**, so
the interner finding is not closed — it is converted from unbounded invisible
growth into a **bounded, loud** limit.

Design that conversion, completely, and return the briefs to execute it.

## What is ruled, and therefore not open

Do not re-propose any of these. If your plan requires one, say so and stop
rather than working around the ruling.

- **`Symbol` does not change.** Not its size, not `Copy`, not the id-based
  fast-accept equality, and **not its lifetime-free `'static`-compatible
  public type**. A generative brand, a lifetime parameter, an `Arc`, or a
  session-scoped id are all excluded. The measured floor for any owning
  representation is +10.93% instructions on ordinary evaluation
  (briefs 501-503).
- **No reclamation.** A reclaimable global interner cannot exclude escaped
  handles: a reissued dense id makes a `Symbol` that escaped into a host's
  `Expr` silently denote different text. `reset()` will not free interner
  bytes and the documentation will say so.
- **No per-session attribution of interner bytes.** They are shared; charging
  them to one session makes one tenant pay for another. The quota is
  process-wide and must not pretend otherwise.

## What the mitigation must be

- A process-wide interner **byte** quota. Count-only is insufficient because
  names vary in length; the budget must cover arbitrary text, the
  `SymbolText` pairs, and the hash-table growth that holds them.
- A **recoverable refusal** replacing the `u32::try_from(pairs.len()).expect(...)`
  panic at `crates/rum-core/src/expr.rs:88`.
- **Preflight before anything is leaked.** `intern_text` (`:57-70`) and
  `intern_symbol_text` (`:80-96`) must decide admission *before* the
  `Box::leak`, not after.
- **Existing names stay usable at the cap.** Refusal applies to inserting a
  new name, never to looking up one already interned.
- **Refusal coverage made compiler-enforced**, not remembered: the only API
  that can insert an arbitrary name returns `Result`, and every constructor
  and serde path is converted to it.
- **Reserved bootstrap and error symbols**, interned before the quota can
  bind, so a refusal can always be reported — a refusal that cannot construct
  its own message is a second failure.

## The questions that decide the plan

### 1. Which constructors become fallible, and what a refusal means at each site

`Symbol::new`, `Symbol::with_identity`, `Symbol::from_name` and
`Symbol::system` (`crates/rum-core/src/expr.rs:108-138`) plus
`Expr::symbol`, `Expr::contextual_symbol` and `Expr::system_symbol`
(`:801-812`) are the public constructors. A director regex over the four
`Symbol::` names finds **179 call sites**; brief-500 showed that such a count
is a lower bound on the real ripple, so **produce the inventory, do not trust
that number.**

Settle, with evidence:

- Which of these can stay **infallible** because they cannot insert arbitrary
  text — `Symbol::system` over `CANONICAL_CORE_HEADS`
  (`crates/rum-core/src/heads.rs:112`) is the obvious candidate, but
  `Symbol::system` also falls through to `Symbol::new` for a non-canonical
  name (`:132-136`), so say exactly what the safe subset is and how the type
  system keeps a caller inside it.
- For each fallible call site **class**, what a refusal becomes: an
  `EvalError` in the evaluator, a message, a parse diagnostic, a host error.
  `rum-core` has no `EvalError`, so name the error type that lives there and
  where it is mapped.
- The sites with **no error channel at all**. There will be some. For each,
  say whether the channel can be created, or the site can be proven unable to
  insert, or it must keep a panic — and if it must, why that is not simply
  moving the panic.

### 2. Serde, which cannot be waved through

`Deserialize for Symbol` (`crates/rum-core/src/expr.rs:303-320`) interns
attacker-supplied text from a snapshot. It is the one path where refusal is
most needed and where the error channel is fixed by the trait. Say how it
refuses, and what that means for `restore_from_bytes`, whose own audit
finding (`SECURITY.md`, the 2026-09-02 LOW) already says restoring untrusted
bytes runs them under the session's policy.

### 3. The quota's shape

- Who sets it, when, and can it change after the first intern? A process-wide
  knob configured once at startup behaves very differently from one a session
  can raise.
- What is the default, and is a default that never fires in practice the same
  fail-open mistake as the 6 MiB native stack guard on a 2 MiB thread
  (`TODO.md` §The native stack budget)? Argue the number, do not pick one.
- How does a host **observe** current usage? It cannot be in
  `current_memory_proxy_bytes`
  (`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50`),
  which is per-session, so name where it goes and what reads it.
- Is the accounting exact or an estimate? If an estimate, state the direction
  of its error; an under-estimate is a fail-open.

### 4. The falsifier

Any check this project builds ships with a checked-in proof that it can fire
(`TODO.md` §memory-accounting line; the rule is that an instrument never seen
to fail is not known to work). Design the falsifier: what test drives the
interner to the cap and asserts the refusal, how it avoids leaking a real
process-global quota into every other test in the binary, and how a future
change that silently stops enforcing is caught.

**This is the hardest part of the design and you should treat it as such**: a
process-global, never-reclaimed quota is exactly the kind of state that makes
tests order-dependent. If the honest answer is that the quota must be
injectable for tests, say so and design that seam.

### 5. Documentation obligations

The mitigation is only honest if the limits are stated. Draft the exact
claims that must appear, and where: that interner bytes are process-global
and outside the per-session ceiling; that `reset()` does not reclaim them;
that one tenant can permanently exhaust the shared quota; and that host
process isolation remains the pattern for hard multi-tenant isolation.
Candidate homes: `SECURITY.md`, `docs/host-embedding-and-hardening.md`,
`docs/guide/resource-limits-and-sandbox.md`.

Note the adjacent audit criticism while you are there — **no document says
which state is per-request and which is per-session** — and say whether this
work should answer it or whether it is a separate brief. Do not answer it
here.

## What I want back

1. The option-free plan: ordered steps, files and functions per step, the
   gate test per step, and how the ruled invariants stay intact.
2. The **call-site inventory**, produced rather than estimated, classified by
   what a refusal becomes there.
3. The falsifier design, per §4.
4. Risks and open questions, each one named as a decision for the director
   rather than resolved silently — brief-504 did this correctly and it is why
   there is a ruling to work under.
5. Drafted follow-up briefs in `TEMPLATE.md` shape, usable as drafts. The
   director rewrites them; they are raw material.

Every claim carries `path:line` evidence.

## Method requirements

- Read `AGENTS.md`, then `TODO.md` §The interner ruling of 2026-09-21, then
  brief-504's report in this directory.
- **Re-probe rather than inherit.** Where you rely on a claim from brief-414
  or brief-504, say you verified it and how. brief-414's Gap A analysis has
  already been found stale in places.
- Where you assert a cost, say whether it is measured, inferred, or a guess.
- Name what would falsify your plan.

## Hard rules

- **No source change.** No `.rs` file, no script, no fixture, no doc except
  appending your report to this file. Scratch under `target/brief-505/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout`, or
  touch git config or the index.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- Paste verbatim evidence for anything you ran. Never quote a number you
  could not take.

## Report format

1. The plan.
2. The call-site inventory.
3. The falsifier design.
4. Risks and open questions, as director decisions.
5. The drafted follow-up briefs.
