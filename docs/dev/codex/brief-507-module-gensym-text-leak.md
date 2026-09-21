# brief-507 — design: can a `Module` local stop minting interned text?

`Scope: NONE — no source change.` Design-only. You survey, reason, and
return a plan plus drafted follow-up briefs. You write no code.

## Goal

brief-506 measured that **every `Module` activation permanently adds one
interned string and one pair, about 120 bytes, and nothing ever reclaims
it** — the interner has no removal path. `Module` is the most ordinary
construct in the language, so on a long-lived host the interner grows with
work done, not with anything abusive.

Design whether a `Module` local can be made distinct **without minting fresh
interned text**, using the identity mechanism that already exists, and what
that costs and breaks. Return a plan, its open questions, and the follow-up
briefs.

## Why this, before the quota

The security-hardening line's HIGH finding is that interner bytes are
invisible to the memory ceiling and never reclaimed. The ruled mitigation is
a process-wide byte cap with a recoverable refusal (see `TODO.md` §The
interner ruling of 2026-09-21 and the D1-D7 rulings). brief-506's census
showed that with `Module` leaking, **a cap is a timer on ordinary
operation** rather than a defence against abuse: a long-lived host refuses
after enough legitimate work. Remove the dominant legitimate consumer and
the cap becomes a real backstop.

The cap is not cancelled. It resumes after this, sized against a system
where ordinary code does not leak.

## What is already settled — do not reopen

- **`Symbol` does not change**: not its size, `Copy`-ness, id-based
  fast-accept equality, or its lifetime-free `'static`-compatible public
  type (user ruling 2026-09-21). Any proposal that changes the
  representation is refuted before it is written.
- No reclamation from the interner, and no per-session attribution of
  interner bytes.
- **`Unique[]` is not in scope for removal.** Returning a symbol with a
  fresh *name* the caller can use is its documented behavior. If your design
  happens to help it too, say so, but do not make it a requirement.

## The mechanism that already exists, and the crux

`Symbol::with_identity(context, name, identity)` interns `(context, name)`
**once** and varies only a `u64` identity
(`crates/rum-core/src/expr.rs:80-96,115-122`). `active_symbol_for_name`
already uses it (`crates/rum-runtime/src/runtime/resolution.rs:128-138`),
and serde already serializes `identity` as its own field
(`crates/rum-core/src/expr.rs:287-320`), so snapshots already carry it.

`Module` does not use it. Its chain is `scoping/lexical/module.rs:46-50` →
`allocate_fresh_symbol` (`scoping/unique.rs:81-82`) →
`allocate_generated_symbol` → **`Symbol::new` with `format!` fresh text**
(`scoping/unique.rs:36-42`).

**The crux: today the `$n` suffix is interned text. Can it become a display
concern instead** — the symbol carries `("Global\`", "x")` plus identity
`n`, and renders as `x$n` when shown? Answer that with evidence, and if the
answer is no, say exactly what forbids it.

## The questions that decide the plan

### 1. What actually depends on a gensym's interned text

Inventory it, do not reason about it in the abstract. At minimum:
`Display`/`qualified_name`/`fully_qualified_name`
(`crates/rum-core/src/expr.rs:140-180`), trace and message rendering,
`Information` and name introspection, `SymbolName`-style predicates and any
pattern that matches on a name string, the gensym GC's candidate tracking
(`crates/rum-runtime/src/runtime/scoping/gensym_gc.rs`), `symbol_generations`
(`crates/rum-runtime/src/runtime/resolution.rs:128-138`), snapshot and
restore, and sealed scoping's bit-62 identity partition
(`docs/dev/sealed-scoping-design.md`).

The director found **five** source locations asserting a printed `name$n`
(`crates/rum-host/src/lifecycle_contract_tests.rs:183`,
`crates/rum-runtime/src/runtime/tests/scoping_unique.rs:17,21,50`,
`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:250`) — treat
that as a lower bound found by one regex, not as the inventory.

### 2. Does anything round-trip the text back into a symbol

A rendered `x$3` that can be **re-parsed** into a symbol is the dangerous
case: display-only identity breaks it, because parsing `x$3` would produce
the interned text `x$3` rather than `x` with identity 3. Search for it
honestly — source parsing, snapshot restore, `ToExpression`, message
substitution, anything that takes a printed name back. If such a path
exists, say whether it must keep working and what that forces.

### 3. Identity allocation and the bit-62 partition

Sealed scoping partitions identity space at bit 62 and keeps sealed
identities out of `symbol_generations`. Say how a per-activation `Module`
identity is allocated, whether it collides with that partition or with
`Unique[]`'s, and whether identities can be **reused** after a gensym is
swept — and if they can, what stops a stale `Symbol` from silently denoting
a live one. That hazard killed the reclaimable-interner option in brief-504
and it applies here in miniature.

### 4. Cost

`module-local` is a microbench label
(`scripts/core-language-microbench.sh:497`). Identity allocation should be
cheaper than `format!` plus interning, but say whether it is measured,
inferred or a guess — and specify the A/B the implementation must pass,
using the instruction-count recipe rather than the timing panel.

### 5. What it does to the census

If `Module` stops interning, restate what the census's numbers become:
which workloads still grow the interner, and what the dominant remaining
legitimate consumer is. That number is the input to sizing the cap later.

## What I want back

1. The plan: ordered steps, files and functions per step, gate test per
   step, and how each ruled invariant stays intact.
2. The dependency inventory from §1 and §2, produced rather than estimated.
3. A clear verdict on the crux, with what would falsify it.
4. Risks and open questions, each named as a decision for the director
   rather than resolved silently.
5. Drafted follow-up briefs in `TEMPLATE.md` shape, as raw material.

Every claim carries `path:line` evidence.

## Method requirements

- Read `AGENTS.md`, `TODO.md` §The interner ruling of 2026-09-21 including
  the census entry, and brief-506's report.
- **Re-probe rather than inherit.** The director's last premise about
  `Module` was wrong and brief-506 caught it; assume nothing stated here
  about a call path without confirming it, including the five assertion
  sites above.
- Where you assert a cost, say whether it is measured, inferred or a guess.
- Name what would falsify your recommendation. **A refutation is an
  acceptable answer** — if `Module` cannot stop interning without breaking
  documented behavior, say so, and the line goes back to sizing the cap with
  that fact documented.

## Hard rules

- **No source change.** No `.rs` file, no script, no fixture, no doc except
  appending your report to this file. Scratch under `target/brief-507/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Bound anything long with
  `scripts/run-bounded.py --timeout SECONDS -- <command>`, sized from a
  measured run of the same shape.
- Paste verbatim evidence for anything you ran; never quote a number you
  could not take.

## Report format

1. The plan.
2. The dependency and round-trip inventories.
3. The verdict on the crux and what would falsify it.
4. Risks and open questions, as director decisions.
5. The drafted follow-up briefs.
