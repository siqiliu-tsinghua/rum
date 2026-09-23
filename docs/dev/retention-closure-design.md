# Retention closure: why traversal failed, and what replaces it

Status: direction record, 2026-09-23. Supersedes the A3-A6 staging drafted in
`docs/dev/codex/brief-532-admitted-surface-design.md` as a *completion* plan.
The measurements behind it are in briefs 533-536.

## The property we are trying to enforce

An armed memory ceiling can be walked through by a builtin that accumulates a
data-dependent payload in a Rust local: the value is in neither the maintained
session total `S` nor the in-flight expression weight `W` until it is
published. `merged_option_exprs_for_symbol` is the canonical instance — it
pushes each evaluated option value into a bare `Vec` while calling the
evaluator, which runs guest code, for the next one.

The census measured the class rather than guessing it: **497 of 643 installed
declarations are in it at minimum**, and of the 56 that are not, 55 are inert
registrations with no handler at all. Essentially every builtin that runs code
holds a payload the ceiling cannot see, because the model charges published
state and the in-flight expression while every handler builds its result
before it becomes either.

## What we built, and what it measured

A required source test implementing a restricted type-and-effect system,
refusing every construct it has not modelled (briefs 527-535). It works: it
found the defect it was pointed at, its rejections fire on demand, and a
compiler-oracle harness — the stable toolchain used as an oracle for
Rust-language facts about our own source — checked 3 365 of 3 365 resolution
claims with zero disagreements, after catching two director errors.

Then brief-536 measured the distance to a zero-unknown option region:

| Cohort | Bodies | Rejection sites |
| --- | ---: | ---: |
| The region's current resolved closure | 4 | 14 |
| After opening two named call sites | 7 | 33 |
| Following edges the checker already resolved | 198 | 897 |
| Following indexed targets on rejected rows, repeatedly | 868 | 5 763 |

`Session::evaluate` alone reaches **190 bodies and 860 sites**. Every region in
this class calls it, because running guest code is what puts a region in the
class.

Five of the fourteen frontier rejections are structurally unadmittable, and
four of those five are rum's own idioms: the resume entry is generic over the
open `OptionReentry` trait, `reentry.resume` is open trait dispatch,
`expr.clone()` is derived `Clone` on `Expr`, and `args.len()` reaches
`Vec<Expr>` through user `Deref` on `SharedArgs` because no inherent accessor
exists.

## Why that is not a tooling shortfall

The obstacle is not resolution quality, which is what briefs 533-535 improved.
It is the **whole-closure** requirement, and that requirement sits on results
that are known to be hard or impossible in general:

- **Aliasing.** Landi proved may-alias is not recursive and must-alias is not
  even recursively enumerable, for languages with conditionals, loops, dynamic
  storage and recursive data structures (Landi, [*Undecidability of static
  analysis*, LOPLAS 1(4):323-337, 1992][landi]; Ramalingam, [*The undecidability
  of aliasing*, TOPLAS, 1994][rama] gives an independent proof). "Is this payload still
  owned here" is an aliasing question.
- **Context-sensitive data dependence.** Undecidable in general; it maps to
  reachability over two interleaved context-free languages, which is not
  context-free (Reps, [*Undecidability of context-sensitive data-dependence
  analysis*, TOPLAS 22(1):162-186, 2000][reps00]).
- **The tractable framework needs a precondition we violate.** IFDS is
  polynomial for finite distributive subset problems **given a known call
  graph** (Reps, Horwitz and Sagiv, [*Precise
  interprocedural dataflow analysis via graph reachability*, POPL 1995,
  pp. 49-61][rhs95]). Open trait dispatch means
  the call graph is exactly what we do not have — which is what the five
  `NeverAdmit` rows are.

Our own growth table has the shape this predicts: each expansion wave still
discovers new targets (16, 24, 15, 10, 23, 9, 8) while the body count climbs,
because with an unknown call graph the only sound move is to follow every
candidate.

**What the theory does not say.** Rice's theorem is often cited to declare
this kind of project impossible. It does not. Rice forbids an analyzer that is
both sound and *complete*; ours is sound and deliberately incomplete — it
rejects whatever it cannot prove. Rice explains why the checker must reject a
great deal. It says nothing about whether disciplined code can be proved.

Nor do these results excuse everything we hit. `Expr: Clone` and
`SharedArgs: Deref` are statically monomorphic in practice and a real compiler
resolves them without effort; they blocked us because our analyzer does not do
trait resolution. That is a tool limitation, and it should not be filed under
undecidability.

## The line we drew in the wrong place

Walking into `Session::evaluate` means trying to prove a property of **guest**
behavior — an arbitrary WL program. That is not merely hard; it is the reason
rum has a *runtime* memory ceiling in the first place.

The static checker was only ever supposed to prove that **the kernel's own
bookkeeping is complete**: that every local retention in a covered body is
charged. It was never supposed to bound the guest. Treating `evaluate` as an
edge to traverse fused those two obligations, and the 868-body closure is the
bill for that.

**The boundary at `evaluate` is not a concession. It is the correct semantic
line.** On the far side is guest behavior, which the runtime ceiling owns.

## The design that replaces traversal

Compositional verification: prove each body **locally**, against declared
contracts at its boundaries, and never compute a transitive closure. This is
the standard escape from the results above, and the existence proof is in the
language we are already using — Rust's borrow checker establishes strong
ownership properties per function, using signatures as contracts, without ever
traversing the call graph. Contract-based modular verification of Rust is
established practice ([Prusti][prusti]).

The strongest form of it, and the one this project should take, moves the
per-body obligation out of analysis and into the **type system**:

- An owned accumulator in a covered body is obtainable **only** through a
  lease-bearing type whose constructor is private. The compiler then enforces
  the property per body, with no analysis at all.
- The checker's remaining job shrinks to a local, syntactic one: *this body
  constructs no raw owned container*. That needs no call closure.
- A callee's own retention is the callee's obligation, discharged by the same
  local rule when it is covered, and by an explicit boundary contract when it
  is not.

brief-531 already demonstrated the mechanism on a different obligation:
`merged_option_exprs_for_system_symbol` takes a typed re-entry descriptor and
returns the consumer's result, so a caller cannot obtain the raw option list
at all, and omitting the descriptor is `error[E0061]` with a checked-in
compile-pass control. That is the compiler enforcing a contract, not an
analyzer proving a theorem.

## What this design does not solve, stated as limits

- **The five `NeverAdmit` idioms do not disappear.** They become obligations
  that must each be discharged: a contract for derived `Clone` on `Expr`, one
  for the `SharedArgs` view, and for `OptionReentry` a contract on the *trait
  method* checked against every implementor.
- **That last one needs a production decision.** `OptionReentry` is public and
  unsealed, so its implementor set is not closed. Either it is sealed, or
  external implementors are stated as outside the guarantee. Both are real
  choices; neither is an analysis question.
- **A contract that nothing checks is a deferral**, and this project has
  repaired that shape repeatedly. Every contract introduced here owes a
  falsifier watched failing, on the standard already in use.
- **Coverage remains 0 until a region is actually proved.** A declining
  frontier, a pending boundary record and a promotion inventory are not
  proofs.

## Open decisions

1. Seal `OptionReentry`, or state external implementors as outside the
   guarantee.
2. Which boundaries get contracts first, and what a contract must contain to
   be checkable rather than declarative.
3. What becomes of the existing checker: demoted from prover to a local lint
   against escape hatches, and how much of briefs 527-535 survives that.

[landi]: https://dl.acm.org/doi/10.1145/161494.161501
[rama]: https://dl.acm.org/doi/10.1145/186025.186041
[reps00]: https://dl.acm.org/doi/10.1145/345099.345137
[rhs95]: https://dl.acm.org/doi/10.1145/199448.199462
[prusti]: https://pm.inf.ethz.ch/publications/AstrauskasBilyFialaGrannanMathejaMuellerPoliSummers22.pdf
