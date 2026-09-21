# brief-509 — design: can `Enclose`'s internal tag stop minting text?

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and a drafted implementation brief. Write no code.

## Goal

brief-507 measured that an untagged `Enclose` leaks — one lazy seed plus one
interned string and pair per call, about **117-127 proxy bytes**, never
reclaimed. The allocation is
`fresh_internal_enclose_tag_expr`
(`crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`),
which takes the seed `System\`InternalEncloseTag` and runs it through the
shared fresh-**text** generator `allocate_fresh_symbol`
(`crates/rum-runtime/src/runtime/scoping/unique.rs:81`).

Decide whether that tag can instead be distinguished by **identity on one
shared interned pair**, and return the implementation brief if it can.

## The crux, stated so it can be refuted

A `Module` local could not simply switch, because its printed name is a
**source-resolvable** name: printing `x` with identity 3 as `x$3` does not
tell the resolver how to recreate it, and host snapshots are source text
(brief-507; `crates/rum-host/src/session.rs:185`).

`InternalEncloseTag$n` looks different in kind: the user never writes it, it
is `Internal`-prefixed, and it exists to be *compared*, not *named*. So:

> Does this tag need distinct **text**, or only a distinct **value**?

If only a value, `Symbol::with_identity(SYSTEM_CONTEXT, "InternalEncloseTag", n)`
gives distinctness with one interned pair, because identity participates in
equality (`crates/rum-core/src/expr.rs:250-254`).

**Establish the answer; do not assume it.** The way it fails is if the tag
escapes somewhere that needs to render or re-resolve it by name.

## What you must establish

### 1. Every path the tag reaches

Enumerate creation, the untagged-confirmation rewrite
(`confirmation.rs:299`), matching and selection, nested-`Enclose` boundary
behavior, failure-payload construction, message rendering, and
suspension/resume/abandon. `path:line` plus **actual `.rum` probes**, not
reasoning alone. Note that the third argument suppresses lexical tag
generation (`confirmation/builtins.rs:7`), so both forms must be probed.

### 2. Whether it can escape into anything text-shaped

The disqualifying cases, each to be answered yes or no with evidence: does
the tag ever appear in a **snapshot**, in a **user-visible failure payload**
or message, in `Names`-style introspection, or anywhere a rendered form
could be parsed back into a symbol? A tag that only ever lives inside kernel
frames and is compared by value is safe for identity; one that is rendered
into a persisted or re-resolvable form is not.

### 3. Identity lifetime and reuse

A tag can outlive an ordinary return — it can sit in a suspension frame.
Say which identity kind it should use, whether a **new** kind is required,
and whether identities may be reused after the tag's lifetime ends. **Do not
assume reuse is safe**: a reissued identity that makes a stale tag match a
live one is the same aliasing hazard that killed the reclaimable-interner
option in brief-504. If you cannot prove a lifetime bound, say the
identities must be monotonic and never reused, and state what that costs.

Do **not** treat this tag as a `Module` GC candidate merely because both use
numeric identities.

### 4. Scope discipline

The fix must be at the internal-tag call site. **Do not propose changing
`allocate_fresh_symbol` itself** — that is the shared generator and changing
it would silently change `Unique[]` and `Module` too, one of which is
deliberately fresh-named and the other blocked on a user ruling.

### 5. The corrected consumer statement

After this repair, restate which legitimate constructs still grow the
interner and at what measured rate, so the eventual cap sizing has a current
list. `Module` stays on it until its ruling lands.

## Why this one, now

It is **unblocked** while the `Module` repair waits on a user ruling, it is
on the path to sizing the quota, and if it works it proves the identity
mechanism on a low-stakes internal case before the higher-stakes one. That
last point is a benefit, **not** a licence to broaden this brief into a
general mechanism.

## Settled — do not reopen

- `Symbol` does not change: size, `Copy`-ness, id-based fast-accept
  equality, lifetime-free `'static`-compatible public type (user ruling
  2026-09-21).
- No interner reclamation, no per-session attribution.
- `Unique[]` keeps minting fresh names; that is its documented behavior.

## What I want back

1. The path inventory from §1 and the escape answers from §2, evidence-led.
2. A verdict on the crux, with what would falsify it. **A refutation is an
   acceptable answer.**
3. The identity-lifetime decision from §3, with its reuse stance argued.
4. A drafted implementation brief in `TEMPLATE.md` shape: exact files and
   functions, the test gate, a repeated-`Enclose` census gate proving the
   leak is gone, and the instruction-count neighbours for the A/B.
5. The corrected consumer statement.

## Hard rules

- **No source change.** No `.rs`, no script, no fixture, no doc except
  appending your report to this file. Scratch and `.rum` probes under
  `target/brief-509/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. You may read and reuse the
  census helpers and binaries under `target/brief-506/` and
  `target/brief-507/`; do not overwrite them.
- Bound anything long with `scripts/run-bounded.py --timeout SECONDS`, sized
  from a measured run of the same shape.
- **Stop-loss, keyed on progress.** If a probe fails twice and the second
  attempt produced no more information, stop and report what you have.
- Paste verbatim evidence; never quote a number you could not take.

## Report format

1. Path inventory and escape answers.
2. Verdict on the crux, and what would falsify it.
3. Identity kind, lifetime and reuse stance.
4. The drafted implementation brief.
5. Corrected consumer statement, and any director decisions you hit.
