# brief-535 — A2: resolve a method call without asking the compiler

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

Method-call syntax is the single largest thing the checker cannot see, and the
census measured it precisely: **1 166 of the N frontier's 2 732 rejection
sites**, spread over **259 definitions in 33 files**, worst case 36 in one
body. Production-wide it is 28 186 of 57 661. There is no small set of helpers
whose repair collapses it.

Admit a **proved** subset of inherent method calls, and reject the rest
loudly. This is the step the line stands or falls on.

## The crux, stated so it is not skipped

`syn` gives syntax. The compiler is not available to tell you which method was
selected. So every admission here is a claim you must be able to prove from
source, and the one that will break a careless implementation is this:

> **An earlier trait candidate can beat a later inherent reference method.**

"There is exactly one inherent method with this name" is therefore **not** a
proof of selection. You must enumerate the trait candidates in scope in order
to *exclude* them — which means indexing trait impls even though trait
dispatch itself stays rejected. `OptionReentry` is a public, unsealed trait,
and sealing it would be a production change and a stop.

Admit an inherent method only when all of these are proved:

- **The receiver has an exact nominal type with reference and place
  provenance** from a checked parameter, a named field or pattern projection,
  a local annotation consistent with its initializer, or a resolved return
  signature. Never from a method name, an owning module, a lone candidate, or
  the fact that the crate compiles.
- **The method definition is unique, visible and unconditional**, and either
  monomorphic or an already-closed instance under the finite-instance rule.
  Establish the canonical impl owner and the `Self` substitution.
- **The receiver adjustment is modelled explicitly** — the built-in
  dereferences, reborrows and autoref, in candidate order. Stop before user
  `Deref`/`DerefMut`, unsizing and arbitrary self types; those reject.
- **No trait candidate in scope could win.** If you cannot establish that,
  reject.

Generic work stays confined to a finite set of closed, reached instances,
with type-growing or unknown cases rejected. The open public generic remains
unproved; only installed specializations are closed.

## The oracle is mandatory here, and it is now affordable

`da874c94` made an oracle run reusable when nothing it consumes has changed:
the checker group went from 121.5 s to 15.3 s, and the key includes the claim
files the checker itself writes, so any resolution you move re-runs the
compiler.

**Every method-resolution claim this brief newly admits is checked by the
oracle, with its negative control** — mutate the claimed target and require an
error at that exact site; mutate a *different* same-named definition and
require **no** error there. A claim the oracle has not checked is a claim
nobody has watched, and this instrument has already caught the director once.

If full coverage of the newly admitted claims exceeds what the harness can run
in its timeout, **report the number and the measured cost**. Do not quietly
sample. I will rule on it; you will not.

## Acceptance — predict before you measure

Resolving a method **exposes its body**, so new rejections appear behind every
newly resolved edge. A smaller frontier is not the goal and a larger one is
not failure. What I require is that the movement be **predicted, then
measured**:

1. Before running the final census, write your predicted `OPTION_FRONTIER`,
   `PUBLIC_WRAPPER` and N-frontier numbers into
   `target/brief-535/PROGRESS.md`, with the reasoning.
2. Then run it, and report predicted against measured, reconciled as
   **located added and removed rows** — never by adjusting an expectation
   after the fact.

The current baseline, reproduced by the director: `OPTION_FRONTIER 14`
(RET001 1, RET002 12, RET003 1) over four bodies in three files,
`PUBLIC_WRAPPER 9`, production coverage **0**. Coverage stays 0; this brief
promotes nothing.

## Falsifiers — watched red then green

- **A name collision**: two same-named inherent methods on different types,
  where picking by name would choose wrong. Pin `RET002` at the call.
- **A trait candidate that beats an inherent method**, so the checker must
  reject rather than resolve.
- **A user `Deref` receiver**, which must reject rather than be adjusted.
- **An allocating helper reached only through a newly resolved method** — pin
  `RET001` at the helper with the full witness chain. Resolution that exposes
  a retention defect is the point; a method body returning `Vec` or calling
  `evaluate` is still a defect, and resolving it is progress in visibility
  only.
- Each with its compile-pass control built first, and each guard disabled
  separately so its own negative test fails. Save complete output under
  `target/brief-535/`.

## Scope

**An objective and a fence, not an enumeration.** A MAY list has been short
six times on this project.

**The objective.** Inside `crates/rum-stdlib/src/tests/retention_closure/`
and its registration, implement proved inherent-method resolution and its
witnesses — including files the guidance does not name.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name appears in your report's first section
with `path:line` ranges and one line on why. **An undisclosed edit is the
violation; a disclosed necessary edit is not.**

**Guidance**: the checker's `resolve.rs`, `types.rs`, `effects.rs`,
`model.rs`, `census/`, `compiler_oracle/`, its fixtures and inventories;
`crates/rum-stdlib/src/tests.rs`; `docs/testing-strategy.md`.

**The fence — STOP conditions:**

- **All production Rust is read-only**, in every crate. Production files are
  evidence. Sealing `OptionReentry`, changing a dispatch, or reshaping control
  flow to make selection easier are each a stop, not implied authority.
- **Admit only what this brief names.** No `?`, no operators, no closures, no
  macro expansion, no blanket trait allowance.
- `crates/rum-core/`, all `Cargo.toml` files and `Cargo.lock`. No new
  dependency; the pin is stable 1.85.0 and nothing here needs nightly.
- No promotion, no lease, no `R` term, no admission-policy change.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## How to build it

The user ruled on 2026-09-23 that this analyzer is ours because it need not be
general. **Read rustc's and rust-analyzer's method-resolution code** — the
candidate assembly and probe order is exactly the solved problem you should
not re-derive by guessing. Both are MIT/Apache-2.0 and rum is MIT: code copied
carries its notice, adapted-in-spirit carries nothing. Say what you adapted
and from where.

Specialize where rum's code is narrow. A rule that only has to hold for the
receiver types rum's covered regions actually use is legitimate — state
plainly where you rely on that, so a later widening knows what it breaks.

## rum / Rust pitfalls (read twice)

1. **Rust compiling does not tell `syn` which method was selected.** A
   successful build is not a resolution proof.
2. **`Copy` and `FrameRootTrace` bounds are not retention contracts**, and a
   generic return signature does not close a body.
3. **A newly reachable helper must be checked or rejected automatically**,
   never silently absent. Reaching further must not quietly widen what is
   accepted.
4. **A count derived by regex is suspect here.** Cross-check against live
   source reads and say which you checked.
5. **The eighteen existing checker tests stay green**, and the oracle cache
   must keep working — if your change makes every run a cache miss, say so
   with the measured cost.
6. **`compile_fail` passes when the code fails for the wrong reason.** Build
   the control first, pin the error.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host, warm tree, immediately before dispatch:
eighteen checker tests green, **15.3 s warm / ~120 s cold**;
`cargo test --workspace` green in **63 s** with no failures. The tree is green
at your starting point; any red is yours.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Proposed required tests: `nominal_method_selection_is_exact`,
`receiver_adjustments_fail_closed`, `installed_instances_do_not_cross`.

Paste verbatim: all checker tests, every falsifier transcript red-then-green,
the oracle's agreements and **every disagreement**, the fresh full and N
census, and the predicted-versus-measured frontier reconciliation.

**Microbench A/B: not applicable** — production is read-only, so no hot path
moves. Say that rather than performing the ceremony; if you find you need a
production edit, that statement stops being true and you must STOP.

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment; the director settles them. Do not set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

After each milestone write `target/brief-535/PROGRESS.md` with what is done
and verified, the exact next action, the tree's state — does it compile, which
tests are green, which files are half-edited — and anything the director needs
to take over. **Your frontier prediction goes here too, before you measure.**
Prefer to leave the tree compiling at each checkpoint.

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config.
- Respect the fence; outside it, the disclosure duty applies.
- **No new `unsafe`.** No new dependency. No nightly.
- No blanket trait allowance, no permissive autoderef search, no name-based
  impl selection, no generic depth cap, no signature-only success summary.
- No `#[ignore]`, no feature gate on a required test, no expectation edited
  after measuring.
- **Testing Policy is a hard line** (`AGENTS.md`). Using rustc as an oracle
  for Rust-language facts about rum's own source is a director ruling and not
  that policy's banned shape.
- Scratch goes under `target/brief-535/`.
- **Stop-loss, keyed on progress.** Report two numbers per attempt: method
  sites proved, and required tests passing. Stop only when **two consecutive**
  attempts move neither. A compile error you are actively reducing is not a
  wall, and a checker rejection is the instrument working.

## Report format (your final message)

1. What changed — per file with line ranges, including every unnamed file,
   with why; and what you adapted from rustc or rust-analyzer.
2. The selection proof: receiver provenance, candidate order, how trait
   candidates are excluded, and what the finite-instance rule closes.
3. The falsifiers, each red then green, with disabled-guard runs.
4. The oracle: every newly admitted resolution checked, agreements and
   disagreements, and the measured cost of that coverage.
5. Verification — every command with verbatim output.
6. The census and the **predicted versus measured** frontier, reconciled as
   located added and removed rows.
7. Stop-loss / open questions, including which dispatch properties remain
   unproved.
