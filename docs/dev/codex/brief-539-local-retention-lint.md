# brief-539 — the local lint: covered means two checks on one body

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

Traversal is retired (`docs/dev/retention-closure-design.md`). Coverage is now
defined **locally**:

> A body is **covered** when both hold, and both are checkable without leaving
> the body:
> 1. it constructs no raw owned container; and
> 2. every call it makes goes either to another covered body, or to a
>    boundary that carries a declared contract.

Build the lint that checks exactly that, one body at a time. **Condition 2 is
a lookup of a mark, not a walk.** Whether a callee is covered is answered by
its mark, which its own lint run earned; the caller never inspects the
callee's body. The transitive property follows by induction, which is the
entire reason this design converges where traversal reached 868 bodies.

The lease protocol it enforces landed in `dc6a1969`: accounted code obtains a
data-dependent owned payload only through lease and retained-value types with
private constructors.

## The two checks, precisely

### Check 1 — no raw owned container

Driven by **type facts**, not by names. The checker already establishes exact
nominal types (A1, brief-534) and proved inherent method selection (A2,
brief-535); use them.

- A body fails when an expression's exact type is an **owned container** and
  the value was not obtained from the lease API.
- The set of owned-container types is a **closed, stated list** in one place,
  with a required test that fails on an entry nothing exercises. An
  expression whose type the checker cannot establish **rejects** — failing
  closed is the point, and "probably not a container" is not a disposition.
- A name list of constructors (`Vec::new`, `vec!`, `collect`, `format!` …) is
  **not** this check. brief-525 named that shape a facade: renaming the call
  or reaching it through an alias defeats it. The rule is about the type of
  what is produced.

### Check 2 — every call reaches a mark

For each call in the body, exactly one of:

- the callee is marked **covered** — its own lint run passed, recorded with a
  fingerprint of its body;
- the callee is a **boundary with a contract** — a record in the format
  brief-537 designed (`docs/dev/codex/brief-537-contract-content.md`, report
  from line 206), carrying its discharge category and a fingerprint;
- the call is a **language construct** covered by an admission rule. brief-537
  separated these from boundaries: `?`, operators, `resume_or_suspend!` and
  closures need *admission rules*, not contracts. Only what A1 and A2 already
  admit is admitted; everything else rejects, honestly, in the report.

Anything else rejects. **Do not inspect a callee's body from the caller's
run.** That is traversal coming back through the side door, and it is the one
thing this brief exists to prevent.

## The marks and what keeps them honest

- **Covered marks** carry a fingerprint of the certified body. A body whose
  source changed is re-linted; a stale mark fails the required test. A mark
  that no lint run produced is not a mark.
- **Contract records** use brief-537's format, with fingerprint, discharge
  category, and the falsifier reference that shows the contract's check can
  fire. A record whose callee changed underneath it fails, loudly.
- Install **only** the contracts that `dc6a1969` actually made dischargeable:
  **B03** (`<R as OptionReentry>::resume`, now runtime-enforced by the TypeId
  set), **B09** (`Session::evaluate`, runtime-enforced for the checkpoint
  property, with its stated exclusions), and **B12**
  (`insert_suspension_frame`, the lease transfer). Every other brief-537
  record stays `Pending` and is **not** a mark. A `Pending` record consumed as
  a mark is the exact failure this project calls a deferral wearing a repair's
  clothes.

## Acceptance — predicted before measured

Write these predictions into `target/brief-539/PROGRESS.md` **before** running
the lint on production, then report predicted against measured, reconciled as
located rows.

1. **The protocol's own probe passes.** The test-only builtin from
   `dc6a1969` (`LeaseProbe538`) is lease-based end to end and must lint clean.
   If it does not, either the lint or the protocol is wrong, and that is the
   most important finding this brief can make.
2. **The option region fails, for the right reasons.** C01
   (`runtime/options/core.rs:87`) and C02 (`:248`) must reject at **both**
   raw constructions — `Vec::new` at `:101` and the `vec![args[0].clone(),
   value]` that brief-537 found at `:259` — plus the unadmitted constructs and
   the calls to boundaries that carry no installed contract. List every
   rejection. A lint that passes C01/C02 today is broken.
3. **Production coverage stays 0.** A fixture lint-clean is fixture
   coverage. No production body is promoted by this brief; converting the
   option region is the next step.

## Falsifiers — watched red, then green

Each saved in full under `target/brief-539/` before quoting.

- A covered-marked body that constructs a raw `Vec` — through a direct call,
  an alias, and a helper returning it — rejects with the span.
- A call from a covered body to a callee with **no** mark rejects.
- A call to a callee whose contract record is **stale** rejects.
- A `Pending` brief-537 record offered as a mark rejects.
- An expression whose type cannot be established rejects rather than passing.
- **The side-door test**: construct a case where inspecting the callee's body
  would change the verdict, and show the lint's verdict does not depend on it.
  This is the check that the lint stayed local.

## Scope

**An objective and a fence, not an enumeration.** A MAY list has been short
six times on this project.

**The objective.** Inside `crates/rum-stdlib/src/tests/retention_closure/`
and its registration and inventories, build the local lint and its marks —
including files the guidance does not name.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name appears in your report's first section
with `path:line` ranges and one line on why. **An undisclosed edit is the
violation; a disclosed necessary edit is not.**

**Guidance**: the checker's type, resolution and effect modules, which the
lint should reuse rather than duplicate; the `inventory/` directory for the
mark and contract records; `crates/rum-stdlib/src/tests.rs`;
`docs/testing-strategy.md` for what the gate now guarantees and does not.

**The fence — STOP conditions:**

- **All production Rust is read-only.** The lint reports; it does not
  convert. A needed production edit is a stop, and would also void this
  brief's "no A/B" basis.
- **No traversal.** No inspection of a callee's body from a caller's run, no
  frontier used as a coverage decision, no transitive closure. The existing
  frontier and census machinery may stay as measurement; it must not decide a
  verdict.
- **No new admissions.** `?`, operators, closures and `resume_or_suspend!`
  stay rejected. Admitting them is separate local work.
- **No promotion of production coverage.**
- `crates/rum-core/`, all `Cargo.toml` files and `Cargo.lock`. No new
  dependency; the pin is stable 1.85.0.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## rum / Rust pitfalls (read twice)

1. **A name list is a facade.** The check is on the produced type.
2. **Unknown type means reject.** Do not default an unresolved expression to
   "not a container".
3. **Condition 2 is a lookup.** If you find yourself reading a callee's body
   to decide a caller's verdict, you have reintroduced traversal.
4. **`Pending` is not a mark.** brief-537 left thirteen of sixteen records
   pending on purpose.
5. **A fingerprint that does not cover what the mark depends on is not a rot
   guard.** The director's own oracle cache once omitted the harness scripts
   from its key; say exactly what each fingerprint covers.
6. **Fixture coverage is not production coverage.** Keep the counts separate
   in every report.
7. **The existing twenty-two checker tests stay green**, and the oracle cache
   keeps working.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host, warm tree, immediately before dispatch:
`cargo test --workspace` green, **2 620 passed, 0 failed**; twenty-two checker
tests green in about 26 s warm. The tree is green at your starting point.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Report the lint's own cost. It is local, so it should be cheap; if the
checker group's warm time grows materially, say so with the number.

**Microbench A/B: not applicable** — production is read-only. Say that rather
than performing the ceremony.

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment; the director settles them. Do not set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

After each milestone write `target/brief-539/PROGRESS.md` with what is done
and verified, the exact next action, the tree's state, and anything the
director needs to take over. **Your acceptance predictions go here before the
lint runs on production.**

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config.
- Respect the fence; outside it, the disclosure duty applies.
- **No new `unsafe`.** No new dependency. No nightly.
- No `#[ignore]`, no feature gate on a required test, no expectation edited
  after measuring.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Scratch goes under `target/brief-539/`.
- **Stop-loss, keyed on progress.** Report two numbers per attempt: falsifiers
  watched red-then-green, and required tests passing. Stop only when **two
  consecutive** attempts move neither. A compile error you are actively
  reducing is not a wall.

## Report format (your final message)

1. What changed — per file with line ranges, including every unnamed file,
   with why.
2. Check 1: the closed owned-container type list and how types are
   established; Check 2: the mark lookup, and the side-door test proving it
   stayed local.
3. The marks: what each fingerprint covers, and the three installed contracts
   with their falsifier references.
4. Predicted versus measured: `LeaseProbe538` clean, C01/C02's full rejection
   list, production coverage 0.
5. The falsifiers, each red then green.
6. Verification — every command with verbatim output, and the lint's cost.
7. Stop-loss / open questions.
