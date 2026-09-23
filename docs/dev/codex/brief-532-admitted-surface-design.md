# brief-532 — design: which Rust the checker admits, and which it never will

`Scope: NONE — no source change.` Design-only. Survey, reason, return a plan
and drafted implementation briefs. Write no code. **Do not run `git` in any
form**, including while first loading this repository.

## Goal

The retention checker is built, enforced, and pointed at the option region.
It cannot see that region, and the director measured why. The full rejection
tally at `OPTION_FRONTIER` today:

```
RET001  x1     std::vec::Vec::new in merged_option_exprs_for_symbol  — the defect itself
RET002  x15    ordinary Rust the admitted surface cannot resolve
RET003  x1     one unexpanded macro
```

The fifteen are not exotic. They are:

- **method-call syntax** — "receiver/autoref/trait effects are outside
  admitted resolution", the largest bucket;
- **`path Some has 0 exact bindings`** and **`path Ok has 0 exact bindings`**
  — `std` enum variant constructors;
- **"implicit Try/FromResidual/error ownership not modeled"** — the `?`
  operator;
- **"closure capture/callback effects unresolved"**;
- **"primitive operand proof or exact operator/conversion impl required"** —
  operators on primitives.

brief-526 predicted this: *"It starts restrictive enough to reject most
existing bodies. This is the migration cost."* The measurement now says what
that migration actually means — writing kernel code with no `?`, no `Option`
pattern, no method calls. That is not a restricted style; it is a dialect,
and it would be imposed on the hottest paths in the system, permanently, for
every region this line ever covers.

**Decide which idioms the checker admits soundly, in what order, and which it
refuses forever.** The direction is already ruled and is not reopened: unknown
constructs reject, and closure is the destination.

## The two routes, and why this is not a free choice

- **Grow the admitted surface.** The checker learns an idiom, with a
  soundness argument per idiom. Production stays idiomatic; every future
  region benefits. The risk is that each admission is a new way to be wrong,
  and the checker is the trusted computing base.
- **Rewrite covered regions into the surface.** No new checker risk. The cost
  is unidiomatic kernel code, per region, forever — and the transitive closure
  means the dialect spreads to every callee.

brief-526 chose the second by default. **The measurement is grounds to revisit
that, not to abandon either.** Some idioms are soundly admissible with real
arguments; some are genuinely unresolvable in `syn` and must stay rejected,
with the region rewritten instead. Say which is which, per idiom, with the
argument — do not answer "grow it" or "rewrite it" wholesale.

## What I want back

### 1. A broader rejection tally than mine

Mine is one region's frontier. Produce the distribution across the production
scan the checker already performs: rejection code, reason, and frequency, and
the same tally restricted to the N-class regions this line must eventually
cover. The ordering of the work should follow measured frequency, not
intuition. Say how you derived it and how you cross-checked the counts — a
regex-derived count in this repository has been wrong more than once.

### 2. Per idiom: admit, refuse, or rewrite — with the argument

At minimum cover method-call syntax, `std` enum variant paths (`Some`,
`None`, `Ok`, `Err`), the `?` operator on a concrete `Result`, primitive
operators, closures, and the `resume_or_suspend!` macro. For each:

- **The soundness argument**, if you propose admitting it. `syn` gives
  syntax, not resolution; an admission whose correctness rests on "we can
  tell what this call does" must say exactly how, and what it does when it
  cannot. **Every admission must still fail closed on its own unresolved
  cases** — an idiom admitted with a silent fallback is worse than a
  rejection, because the rejection is honest.
- **The failure mode when the argument is wrong**: what gets accepted that
  should not, and how a reader would find out.
- **The falsifier**, in the shape already used: a compiled fixture whose
  rejection is pinned by kind and span, watched red, then green, plus the
  mutation that proves the check can still fire.
- **The reach**: how many of the measured rejections it removes.

Method-call syntax deserves its own treatment. brief-526 rejected it because
autoref, autoderef and trait selection are outside `syn`. If you propose
admitting a subset — an inherent method on an exact nominal receiver whose
type is established by a checked parameter, field or prior return — state
precisely where the subset ends and what happens at that edge.

### 3. What stays refused forever, stated as a property

`dyn Trait`, arbitrary generics, function-pointer calls, and anything else
you conclude cannot be resolved. These are the regions that must be rewritten
rather than admitted, and that is a legitimate answer — say so plainly so the
implementation briefs can plan the rewrite instead of waiting for a checker
feature that will never come.

### 4. The trusted-leaf question this raises

`Some`, `Ok` and `Vec::new` are not user code. Some of the fifteen might be
answered by trusted-leaf records rather than by grammar. Say which, and keep
brief-526 §3.5's rule: the unit is one resolved function plus its exact type
instantiation and preconditions — never a crate, module, trait or method
name, and never an allowlist of whole modules. A trusted record that nothing
exercises must still be rejected by the existing test.

### 5. Order and staging

The sequence that unblocks C01/C02 soonest without admitting anything you
cannot argue for, and what is enforced after each step. Say explicitly which
step makes the option region's frontier drop, and to what — a predicted
number I can check against the run.

### 6. Drafted implementation briefs

In `TEMPLATE.md` shape, scoped to current files, each with gate tests and
falsifiers. Scope sections must state **an objective plus a fence and a
disclosure duty**, not an enumeration — a MAY list has been short six times
on this project.

### 7. Open questions for the director

## Settled — do not reopen

- **Closure is the destination** and unknown constructs reject. The user
  ruled this on 2026-09-23; a permissive mode is not a proposal.
- **No `rustc_private` driver.** Stable 1.85.0 cannot host it; this was
  settled in brief-526 with its costs stated.
- **No new dependency, no toolchain change.** `syn`, `quote` and
  `proc-macro2` with `span-locations` are already dev-dependencies of
  `rum-stdlib`.
- **The line targets the N class.** P is a documented bound, not work.
- **No leases or admission changes** are designed here; that is D2b.
- `rum-core`, the interner, gensym and the snapshot format are out of scope.

## rum / Rust pitfalls (read twice)

1. **`syn` is syntax, not resolution**, and a proc-macro-style analysis runs
   before type checking. A rule about a method receiver is a syntactic rule
   wearing a semantic costume unless you say how the type is established.
2. **An idiom admitted with a silent fallback is a hole with a green test.**
   Fail closed inside each admission too.
3. **A count derived by regex is suspect here.** Cross-check against live
   source reads and say which you checked.
4. **`compile_fail` passes when the code fails for the wrong reason.** Build
   the compile-pass control first and pin the error.
5. **Do not propose production control-flow changes to make analysis easier**
   as though they were free; a covered region sits on hot paths, and the
   traversal-depth line's rule stands — move locals, not control flow.
6. **The checker is the trusted computing base now.** Every admission
   enlarges it. Say so where it applies rather than presenting an admission
   as pure gain.

## Scope

- Files you MAY modify: **NONE**, with two exceptions:
  - append your report to **this** file,
    `docs/dev/codex/brief-532-admitted-surface-design.md`;
  - write scratch artifacts under `target/brief-532/`.
- Everything else is read-only. No source change, no test change, no doc
  change, no `Cargo.toml` change.

## Verification (MANDATORY — the evidence IS the deliverable)

No source change, so no test gate. Instead:

- State every command you ran; persist raw output under `target/brief-532/`
  **before** parsing it.
- Paste the rejection tally verbatim, both overall and restricted to the
  N-class regions, with the method that produced it.
- Paste the predicted post-step frontier numbers so the director can check
  them against a real run.
- State explicitly anything you could not settle.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the two exceptions in Scope. If you believe
  you must, STOP and write that in the report instead.
- Touch nothing outside this repo. No network. Commands limited to file
  reads, `rg`/`grep`, `cargo test` for the existing checker tests only, and
  `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No code.** If you are editing a `.rs` file, you have left the brief.
- Do not resolve a hard idiom by widening the trusted set to a module.

## Report format (your final message, appended to this file)

1. What changed — "design only, no source change", plus scratch artifacts.
2. Verification — commands and the verbatim tallies.
3. Per-idiom dispositions with soundness arguments, failure modes,
   falsifiers and reach; what stays refused forever; the staging with
   predicted frontier numbers.
4. The drafted implementation briefs.
5. Stop-loss / open questions.
