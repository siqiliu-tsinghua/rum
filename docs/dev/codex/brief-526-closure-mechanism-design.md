# brief-526 — design: the closure mechanism for unbounded local retention

`Scope: NONE — no source change.` Design-only. Survey, reason, return a
mechanism design and drafted implementation briefs. Write no code.

## Goal

brief-525 censused all 643 installed declarations and found that a
participation barrier closes nothing: its own candidate table records **zero
local-payload closure** for every mechanism it named, because nothing in
Rust's type system stops a handler allocating a bare `Vec`. Its conclusion
was that only a **compiler-resolved allocation/effect check** can actually
close the class, and it declined to treat that as pre-approved.

**The user has now approved it (2026-09-23).** Build the design: a check,
resolved at compile or required-test time, under which a handler in a covered
region **cannot** retain a data-dependent payload without a lease, and under
which anything the check cannot resolve fails closed.

## Two rulings that set this brief's shape

**Ruling 1 — the line targets the N class only.** brief-525 split the class
in two:

- **N (`InClass-NoProjection`)** — growth the guest drives, with nothing able
  to bound it beforehand. 78 new witnesses plus an unsplit portion of the 147
  carried rows. **This is the line's target**: it is the only half that can
  be walked through without limit.
- **P (`InClass-ProjectableFromChargedInput`)** — bounded by a constant
  factor of an already-charged quantity. 272 identified rows. The ceiling
  admits a bounded multiple more than it should; a guest cannot drive it.
  **P is not this line's work.** It becomes a named, documented bound.

So the mechanism's **required reach is the N regions**. A design that reaches
further is acceptable; a design that must cover all 643 before it protects
anything is over-scoped and is a stop.

**Ruling 2 — compiler closure is the committed target, not a candidate.**
Do not return a recommendation between "participation barrier" and "real
closure". Return the closure design. Its demonstration obligations (below)
are acceptance conditions of building it, not a decision point that could
turn back.

## What already exists to build on, and what each one proves

This repository has three working examples of a class kept closed by the
compiler rather than by memory. Read all three before proposing anything:

- `crates/rum-derive/src/lib.rs:113` — `FrameRootTrace`, which makes a
  missing suspension-frame traversal a compile error, with an invariant test
  recording every handwritten impl.
- `crates/rum-derive/src/lib.rs:73` — `AccumulatorChainNode`, the ownership
  line's drain participation gate.
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs:19-42`
  — the exhaustive `Session` pattern with no `..`: an unclassified field
  cannot compile.

`rum-derive` is already a proc-macro crate with `syn` (`full`, **`visit`**),
`quote` and `proc-macro2` (`crates/rum-derive/Cargo.toml:14-17`). The
toolchain is pinned to stable **1.85.0** (`rust-toolchain.toml`), so a
`rustc_private` lint driver is not available without breaking that pin — say
so explicitly if you propose one, with what it would cost.

**And read the lesson that limits all three.** The ownership line's byte
guard constrained the call stack while the host *destructor* walked a graph
nothing modelled, which is why both real drop hazards were found by accident.
A barrier that models the wrong graph is worse than no barrier, because it
looks enforced. State plainly which graph your mechanism models and which one
the defect actually lives on.

## What I want back

### 1. The mechanism

Which form, given the 1.85.0 stable pin and the existing `rum-derive`:

- an attribute macro over handler and helper bodies that parses with `syn`
  and rejects anything outside a restricted surface;
- a required test that parses the crate source and enforces the same surface
  over declared regions and their callees;
- both, with different jobs;
- something else you can defend.

For the ones you reject, say why in one line each. Name the chosen form's
failure mode when it is wrong.

**An attribute macro over a hot handler must not change generated code.** If
it is anything other than identity-transforming plus reject-only, say so and
justify it; a macro that alters what the evaluator compiles needs the
microbench A/B, and that changes the line's shape.

### 2. Call closure — the hard part

A body-local check sees one body. The defect lives in helpers:
`merged_option_exprs_for_symbol` (`crates/rum-runtime/src/runtime/options/core.rs:87-97`)
is the worst path of 31 declarations, and it is a helper, not a handler.

- How does the check resolve a call to its definition, given that `syn` sees
  syntax and not name resolution? Method calls, trait dispatch, imported
  paths, macros, aliases and closures all have to be answered.
- **What happens to what it cannot resolve.** The only acceptable answer is
  that it fails closed, and the cost of that answer is that the covered
  region starts by rejecting nearly everything. Say how large the transitive
  closure of the N regions actually is, from the source — this is the number
  that sizes the line.
- How the covered region set is declared, and what stops it quietly
  shrinking. A region removed from the set must be as loud as a bare `Vec`
  inside one.

### 3. The trusted leaf set — where a facade would grow

Some operations must be allowed to allocate without a lease. Define that set,
how it is declared, and how each member's bound is stated.

brief-525 named the facade shapes explicitly: a name grep, "has a projection
call", a count of pushes, or **an allowlist of whole modules**. A trusted set
that grows by adding modules is the failure mode this brief exists to avoid.
Say what stops it.

### 4. The demonstration — six shapes, each shown rejected

The design must show, concretely, how the check rejects each of these inside
a covered region. Name the construct and the rejection:

1. a bare `Vec` that accumulates in a loop;
2. a `collect()` reached through an alias or an iterator adaptor chain;
3. a `String` grown by `format!`/`write!`;
4. `entry().or_insert_with(...)` growth on a map;
5. an allocating helper extracted out of the body and called;
6. a callback result retained without a lease — the
   `merged_option_exprs_for_symbol` shape.

For each, say whether the rejection is *by construction* (the surface cannot
express it) or *by analysis* (the check must recognize it), and note that
the second kind is only as good as the resolution in §2.

### 5. The falsifier

Per the standing rule, a check nobody has watched fail is not known to work.
Name the checked-in proofs that each of the six rejections fires — the guard
substrate's `compile_fail` doctests are the existing model — and say which
are `compile_fail`, which are a red required test, and how each is watched
going red before it goes green.

### 6. Reach and stated limits

What the mechanism does **not** reach, stated as a property rather than
discovered later: host-registered builtins through the public `BuiltinSpec`
surface, construction inside `rum-core`, anything behind a trait object, and
whatever else the design cannot see. brief-525 already records that the
kernel cannot see a host builtin's private Rust accumulator; keep that limit
explicit rather than implying coverage.

### 7. Staging

How this lands incrementally. It must protect something real early — the
31-declaration option path (`C01`/`C02` in brief-525 §6) is the obvious first
covered region — rather than requiring the whole closure before anything is
enforced. Give the order, and say what is enforced after each step.

### 8. Drafted implementation briefs

In `TEMPLATE.md` shape, scoped to current files, each with its gate tests,
its falsifier and its A/B labels where a hot path is touched. Include the
lease/container protocol only to the extent the closure mechanism constrains
its shape — the protocol itself and the callback activation bridge are
already designed in brief-524 §4 and are not re-derived here.

### 9. Open questions for the director

Anything you cannot settle, and any decision you think is mine.

## Settled — do not reopen

- **P is out of scope.** Do not design projections, do not propose measuring
  factors, do not widen the mechanism to cover P "while we are there".
- `R` enters admission only; the `S` **model** is unchanged, though `S`
  readings may move as the accountant's own charged storage grows.
- The callback **activation bridge** is an acceptance condition of the
  protocol work, already ruled.
- **`ToExpression` is ruled**: a measured projection from the charged input
  string, not a syntax/token model. It is a P row and therefore not this
  brief's business.
- The occurrence model stands. `Symbol`, the interner, the gensym quota and
  the snapshot format are out of scope.
- No new `unsafe`. No dependency added without saying exactly what it buys
  and what the 1.85.0 pin costs.

## rum / Rust pitfalls (read twice)

1. **`syn` gives you syntax, not resolution.** Any design whose soundness
   rests on "we can tell what this call does" must say how, and must fail
   closed when it cannot.
2. **A proc-macro attribute runs before type checking.** It cannot know a
   receiver's type; a method-call-based rule is a syntactic rule wearing a
   semantic costume.
3. **A `compile_fail` doctest passes when the code fails to compile for the
   wrong reason.** Pin the expected error, or the falsifier is a false
   positive waiting to happen.
4. **The evaluator's hottest loops are inside the regions you want to
   cover.** Anything that changes generated code there needs the microbench
   A/B, and the traversal-depth line's rule applies: move locals, not
   control flow.
5. **A region list is a convention unless something makes it loud.** The
   project has repaired "a list someone must remember" three times now.
6. **Do not equate a helper group with an edit site.** brief-525 is explicit
   that its 163 work packages are planning groups, not proven single edits.

## Verification (MANDATORY — the evidence IS the deliverable)

No source change, so no test gate. Instead:

- State every command you ran; persist raw output under `target/brief-526/`
  before parsing it.
- Paste the transitive closure size of the N regions, with the method that
  produced it and how you cross-checked it.
- Paste, verbatim, the six rejection shapes against your chosen mechanism,
  each marked by-construction or by-analysis.
- State explicitly anything you could not settle.

## Scope

- Files you MAY modify: **NONE**, with two exceptions:
  - append your report to **this** file,
    `docs/dev/codex/brief-526-closure-mechanism-design.md`;
  - write scratch artifacts under `target/brief-526/`.
- Everything else is read-only. No source change, no test change, no doc
  change, no `Cargo.toml` change.

## Hard rules

- Do NOT run `git` at all, and do not `commit`, branch, push, or touch git
  config. Nothing in this brief needs git. `pwd` is fine.
- Do NOT modify any file outside the two exceptions in Scope. If you believe
  you must, STOP and write that in the report instead.
- Touch nothing outside this repo. No network. Commands limited to file
  reads, `rg`/`grep`, `cargo metadata`/`cargo tree` if you need the
  dependency facts, and `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No code.** If you are editing a `.rs` file, you have left the brief.
- Do not resolve a hard question by widening the trusted set.

## Report format (your final message, appended to this file)

1. What changed — "design only, no source change", plus the scratch
   artifacts under `target/brief-526/`.
2. Verification — the commands and the verbatim evidence named above.
3. The mechanism, call closure, trusted set, six demonstrations, falsifiers,
   stated limits, staging, and the drafted implementation briefs.
4. Stop-loss / open questions.
