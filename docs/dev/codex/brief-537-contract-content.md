# brief-537 — design: what a boundary contract must contain to be checkable

`Scope: NONE — no source change.` Design-only. Survey, reason, return a
contract definition and the option region's exact boundary set. Write no code.
**Do not run `git` in any form**, including while first loading this
repository.

## Goal

Traversal is retired. `docs/dev/retention-closure-design.md` records why — the
option region's closure reaches 868 bodies and 5 763 sites without converging,
because every region in this class calls `Session::evaluate` and proving a
property past that point means proving guest behaviour, which is the runtime
ceiling's job.

Composition replaces it, and that document redefines coverage locally:

> A body is **covered** when it constructs no raw owned container, and every
> call it makes goes either to another covered body or to a boundary that
> carries a declared contract.

Everything now rests on the second clause. **Define what a contract must
contain to be checkable rather than declarative**, and prove the definition is
workable by enumerating the option region's actual boundary set.

## Why this is the first step and not the second

This project has repaired "a declaration moved into a contract nothing
enforces" more than once, and named it: *a reclassification into an unenforced
contract is a deferral wearing a repair's clothes.* If the lease protocol is
built before the contract language is pinned, it will produce contracts that
read well and check nothing.

So: contracts first, protocol second, and the local lint that enforces them
third. That is the user's ruling of 2026-09-23.

## What I want back

### 1. The contract's content

What fields must a boundary contract carry so that a checker, and a reader,
can both use it? At minimum distinguish:

- **What the caller may assume** — the guarantee, in terms of owned payloads,
  capacity, transfers and exceptional exits.
- **What the callee must satisfy** — the obligation.
- **What is explicitly outside** — guest-private allocation, a host builtin's
  private Rust storage, anything the contract does not cover. This field is
  not optional; a contract with nothing outside it is almost certainly
  wrong.

State the fields, and for each say what a checker does with it. A field
nothing consumes is documentation, and should be labelled as such rather than
smuggled in as a guarantee.

### 2. The discharge taxonomy — this is the crux

Every contract has a callee side, and the question is **how that side is
discharged**. Classify each into exactly one of:

- **Compiler-checked** — a private constructor, an exhaustive pattern, a
  derive. Holds for every build.
- **Test-checked** — a required test observes it. State what it observes and
  on which inputs, because that is the strength it actually has.
- **Runtime-enforced** — the property is checked on the execution that
  happens. This is the honest answer for anything past `Session::evaluate`.
- **Assumed, with a stated limit** — we assume it and say so.

**The last category is where this project has failed before, so it carries
extra duties.** Every entry must be enumerated, justified, and must say
**what goes wrong if the assumption is false** — who would observe it, and
how. If that list is long, the design is wrong; say so rather than growing it.

Note that the three shapes already shipped map onto the first three
categories, and `docs/dev/retention-closure-design.md` tabulates them with
their real strengths. `Preflight` is a worked example of a contract that reads
stronger than it is — do not repeat its shape without labelling it.

### 3. The option region's exact boundary set

The concreteness test, and the reason this design might work where traversal
did not. The four bodies are C01 (`runtime/options/core.rs:87`), C02
(`:248`), `is_system_symbol_name` and `expr_as_symbol`.

The director's rough count of their **direct** boundaries is five or six once
language constructs are set aside: `Expr::clone` (derived `Clone`),
`SharedArgs::len` (through user `Deref`, no inherent accessor),
`Option::map` and `Option::unwrap_or` with a literal closure,
`OptionReentry::resume` (open trait), and `Session::evaluate`. The lease API
replaces `Vec::new`.

**Confirm or correct that enumeration from the census data**, and for each
boundary give the proposed contract and its discharge category. Then state the
number plainly, against the 868 bodies traversal required.

Separate the language constructs — `?`, `&&`, `==`, `resume_or_suspend!` —
from the boundaries. Those need **admission rules**, not contracts, and saying
which is which is part of the answer.

### 4. What stops a contract rotting

`inventory/leaves.tsv` already carries a dependency fingerprint, and the
existing test rejects a stale, wildcard or unexercised record. Say what
invalidates a boundary contract, and what makes the invalidation loud. A
contract whose callee changed underneath it must fail, not drift.

### 5. A falsifier per contract kind

For each kind you propose, the proof that its check can fire: what is
mutated, what goes red, and what the diagnostic says. An instrument nobody has
watched fail is not known to work, and the compiler-oracle harness
(`compiler_oracle/`, reusable since `da874c94`) is available to check any
Rust-language claim a contract makes about our own source.

### 6. The syntactic form in this repository

TSV rows, an attribute, a trait, a derive, or something else. Pick, with
reasons, and use the precedents rather than inventing: `leaves.tsv` and its
fingerprints, `catalog/memory_growth_contract.rs`'s declaration table,
`FrameRootTrace` and `AccumulatorChainNode` as derives, and the `Session`
field schema's exhaustive pattern with no `..`.

### 7. Where a contract cannot be made checkable

A legitimate answer, and I want it stated rather than papered over. If a
boundary can only ever be "assumed", say which and what that costs. In
particular: `OptionReentry` is public and unsealed, so its implementor set is
not closed — a contract on the trait method can be checked against every
implementor **in this repository** and against none outside it. Say what that
means for the guarantee, and what sealing would change. **Do not propose the
production change; state the consequence and leave the decision to the
director.**

### 8. Open questions for the director

## Settled — do not reopen

- Traversal and whole-closure proof are retired. Do not propose reviving
  A3-A6 as a completion plan.
- The line targets the N class; P is a documented bound.
- The analyzer is ours, specialized for rum; read rustc and rust-analyzer
  freely, and the compiler oracle checks our claims.
- Coverage stays 0 until a region is actually proved. No promotion here.
- `rum-core`, the interner, gensym, the snapshot format and the `S` model are
  out of scope. **No production change of any kind is proposed by this
  brief** — including sealing a trait.

## rum / Rust pitfalls (read twice)

1. **A contract nothing consumes is documentation.** Label it as such; do not
   let it read as a guarantee.
2. **`Preflight` is the worked example of over-reading.** What its validator
   establishes is that a projection exists and was not violated on the inputs
   the tests ran.
3. **An unsealed public trait has no closed implementor set**, and no amount
   of in-repository checking changes that.
4. **The census data is in the repository** — use it rather than re-deriving,
   and say which tables you read.
5. **A count derived by regex is suspect here.** Cross-check against live
   source reads and say which rows you checked by hand.

## Scope

- Files you MAY modify: **NONE**, with two exceptions:
  - append your report to **this** file,
    `docs/dev/codex/brief-537-contract-content.md`;
  - write scratch artifacts under `target/brief-537/`.
- Everything else is read-only.

## Verification (MANDATORY — the evidence IS the deliverable)

- State every command you ran; persist raw output under `target/brief-537/`
  before parsing it.
- Paste the boundary enumeration and the discharge classification as tables.
- Run the existing checker tests once to confirm the tree is as described:
  twenty-two green, warm in roughly 26 s.
- State explicitly anything you could not settle.

## Checkpointing — protect the scene

After each milestone write `target/brief-537/PROGRESS.md` with what is done,
the exact next action, and anything the director needs to take over.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the two exceptions in Scope.
- **No code.** If you are editing a `.rs` file, you have left the brief.
- No production change proposed as a deliverable; consequences only.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Do not resolve a hard boundary by enlarging the "assumed" category.

## Report format (your final message, appended to this file)

1. What changed — "design only, no source change", plus scratch artifacts.
2. Verification — commands and the verbatim tables.
3. The contract definition, field by field, with what consumes each.
4. The discharge taxonomy, with the "assumed" list enumerated and justified.
5. The option region's boundary set, its count, and each boundary's contract.
6. Rot guards, falsifiers per kind, and the chosen syntactic form.
7. Where a contract cannot be checkable, and what that costs.
8. Stop-loss / open questions.
