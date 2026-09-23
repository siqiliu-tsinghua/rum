# brief-534 — A1: exact types, variants and patterns, so a constructor resolves

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

The census measured where the checker is blind. On the N frontier, after
method syntax, the largest class is **`path <path> has <n> exact bindings`
— 539 unique sites**: the checker cannot resolve `Some`, `None`, `Ok` or
`Err`. It also cannot follow the aggregate and place semantics needed to prove
a constructor's payload, which is why those rejections cascade.

Give it exact nominal identity, named fields and variants, reference layers
and checked provenance — the foundation every later stage rests on — and
admit the standard enum constructors on top of it. **Admit nothing else.**

**Acceptance is a predicted number.** After this brief the legacy frontier
must read `OPTION_FRONTIER 14` — RET001 1, RET002 12, RET003 1 — over the same
four bodies in three files, `PUBLIC_WRAPPER 9`, union 23, production coverage
0. If your result differs, reconcile it as **located added and removed rows**;
do not edit an expected total to make a discrepancy disappear.

## The instrument you must use

brief-533 built the compiler-as-oracle harness, and it earned its keep
immediately: it caught the director's own type-claim technique being wrong.
`assert_type::<T>(expr)` does **not** pin a type by unification —
`assert_type::<&[bool]>(x)` accepts `x: &[bool; 1]`, because the argument
position is a coercion site. The claim is now carried in an invariant
position, where a wrong type is `E0308` at the right line.

**Every type and resolution claim this brief adds is checked by that
harness**, with its negative control — a different same-named definition
mutated, and no error required at the site. A claim the harness has not
checked is a claim nobody has watched. Report agreements and **every
disagreement**; a disagreement is a finding, not a failure to hide.

## How to build it

The user ruled on 2026-09-23 that this analyzer is ours because it need not be
general — only what rum's covered regions use. That is licence to specialize,
not licence to guess:

- **Read rustc's and rust-analyzer's source** for how they represent nominal
  types, variants, places and provenance, and adapt what fits. Do not
  re-derive a solved problem. Both are MIT/Apache-2.0 and rum is MIT: code
  copied carries its notice, adapted-in-spirit carries nothing. Say in the
  report what you adapted and from where.
- Specialize where rum's code is narrow. A rule that only has to hold for the
  types rum's covered regions actually use is a legitimate rule; say so
  explicitly where you rely on it, so a later widening knows what it breaks.

## Deliverable

1. **Separate nominal identity from retention state.** Today `Ty` fuses the
   two, so a type fact cannot exist without an ownership claim. They must be
   independent: **a type may be known while the expression's effects are still
   rejected**, and the diagnostic census may report the former while only a
   complete ownership proof certifies the latter.

2. **Named fields, variants, reference layers and checked provenance.** A
   place comes from a checked parameter, a named field or pattern projection,
   an explicit local annotation consistent with its initializer, or a resolved
   return signature — never from a method name, an owning module, a lone
   candidate, or the fact that the crate compiles.

3. **The standard enum constructors**, resolved across valid aliases and
   prelude paths **without stealing a local binding of the same name**. Note
   that `None` is not a call and `Some` in a pattern is not a call either;
   construction and pattern matching are different obligations.

4. **The minimal place semantics the controls need**: exhaustive branch and
   `let`-else ownership transitions, explicit returns, and nested
   `Raw`/`Callback`/`Retained` facts propagated rather than flattened to
   `Fixed`. Partial moves keep the siblings' obligations. Reject unsupported
   guards, shadowing and cleanup rather than approximating them.

5. **Trusted records keep their exact instantiation.** Audit and used-record
   keys include the exact type instantiation; an unused, stale or wildcard
   record stays fatal, and `inventory/leaves.tsv` still may hold only entries
   a fixture exercises.

## Falsifiers — watched red then green

- The new valid control is **rejected before** the implementation and
  **accepted after** — show both runs.
- A local binding named `Some` or `Ok` must not be resolved to the standard
  intrinsic; watch that fixture red with namespace selection disabled.
- Payload propagation disabled independently, so its negative test fails.
- Remove one of two concrete exercised-instance records and show the unused
  sibling record is still fatal.
- Each with pinned kind, span, target and witness. Save complete output under
  `target/brief-534/` before quoting.

## Scope

**An objective and a fence, not an enumeration.** A MAY list has been short
six times on this project.

**The objective.** Inside `crates/rum-stdlib/src/tests/retention_closure/`
and its registration, build the type and place foundation and the constructor
admission — including files the guidance does not name.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name appears in your report's first section
with `path:line` ranges and one line on why. **An undisclosed edit is the
violation; a disclosed necessary edit is not.**

**Guidance**: the checker's modules — `model.rs`, `types.rs`, `resolve.rs`,
`effects.rs`, `leaves.rs`, `census/`, `compiler_oracle/` — plus
`crates/rum-stdlib/src/tests.rs` for registration and
`docs/testing-strategy.md` for what the gate does and does not guarantee.

**The fence — STOP conditions:**

- **All production Rust is read-only**, in every crate. This brief changes no
  behavior; production files are evidence. A needed production edit is a stop,
  and it would also void this brief's "no A/B" basis.
- **Admit only what this brief names.** No method-call syntax, no `?`, no
  operators, no closures, no macro expansion — those are A2 onward, and
  admitting one early makes the predicted acceptance number meaningless.
- `crates/rum-core/`, all `Cargo.toml` files and `Cargo.lock`. No new
  dependency; the pin is stable 1.85.0 and nothing here needs nightly.
- No inventory promotion, no lease, no `R` term, no admission change.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## rum / Rust pitfalls (read twice)

1. **`None` is not a value constructor call**, and `Some` in a pattern is not
   a function call. Two different obligations.
2. **An expected `Result` return type can establish a constructor instance
   while the payload's clone still rejects.** Do not let the former license
   the latter.
3. **A borrow of `Expr` or `Session` is not automatically charged**, nor safe
   to mutate. Provenance is a precondition, not a receipt.
4. **Match exhaustiveness is not a cleanup proof.**
5. **`let _: T = expr` coerces**, and so does a turbofish argument position.
   Use the invariant carrier the oracle now has.
6. **Do not bless a whole family.** Not all `Option`/`Result` instances have
   fixed payloads, not all `std` operations are safe, not all borrowed runtime
   values are charged.
7. **The twelve existing checker tests must stay green**, at their current
   speed. If a fixture's diagnostic moves, explain the move — do not weaken
   its contract.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host, warm tree, immediately before dispatch:
twelve checker tests green, `cargo test --workspace` green in **78 s** with no
failures. The tree is green at your starting point; any red is yours.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Proposed required tests: `enum_constructors_preserve_payloads`,
`variant_shadowing_never_uses_std_intrinsics`,
`partial_moves_keep_sibling_obligations`. Compile every control first.

Paste verbatim: all checker tests, the falsifier transcripts, the oracle's
agreements and disagreements, and the **located** difference between the
baseline frontier and the new one — row by row, added and removed.

**Microbench A/B: not applicable** — production is read-only, so no hot path
moves. Say that rather than performing the ceremony.

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment; the director settles them. Do not set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

Codex quota is low and a run may be cut short. After each milestone write
`target/brief-534/PROGRESS.md` with what is done and verified, the exact next
action, the tree's state — does it compile, which tests are green, which files
are half-edited — and anything the director needs to take over. Prefer to
leave the tree compiling at each checkpoint.

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config.
- Respect the fence; outside it, the disclosure duty applies.
- **No new `unsafe`.** No new dependency. No nightly.
- No `#[ignore]`, no feature gate on a required test, no permissive mode, no
  expected total edited to hide a discrepancy.
- **Testing Policy is a hard line** (`AGENTS.md`). Using rustc as an oracle
  for Rust-language facts about rum's own source is a director ruling and not
  that policy's banned shape.
- Scratch goes under `target/brief-534/`.
- **Stop-loss, keyed on progress.** Report two numbers per attempt: the
  frontier's current rejection count, and required tests passing. Stop only
  when **two consecutive** attempts move neither. A compile error you are
  actively reducing is not a wall, and a checker rejection is the instrument
  working.

## Report format (your final message)

1. What changed — per file with line ranges, including every unnamed file,
   with why; and what you adapted from rustc or rust-analyzer.
2. The separation: how nominal identity and retention state are now
   independent, and where a type is known while effects still reject.
3. The falsifiers, each red then green, with the disabled-check runs.
4. The oracle: every claim checked, agreements and disagreements.
5. Verification — every command with verbatim output.
6. The located frontier difference and whether it matches the predicted
   `OPTION 14 (1/12/1)`, `PUBLIC 9`, union 23, coverage 0.
7. Stop-loss / open questions.
