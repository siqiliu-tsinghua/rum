# brief-540 — local admissions, so lease-correct code can lint clean

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Where this starts

**Continue the working tree exactly as brief-539 left it.** Do not restart,
revert, stash or clean anything. The first dispatch of this brief stopped with
zero edits, so the tree is unchanged since brief-539; the director has also
preserved it on the side ref `wip/retention-lint-539`, which you must not
touch. brief-539 built the local lint and it works:
nine falsifiers watched red then green, the side-door test proves the verdict
never reads a callee's body, marks are fingerprinted, and only B03, B09 and B12
are installed as contracts.

One required test fails, on purpose, and it stays exactly as written:

```
tests::retention_closure::local::production::local_production_acceptance --- FAILED
```

It asserts that the lease protocol's own probe, `LeaseProbe538`, lints clean.
It does not, and **not because the protocol is wrong**: its four bodies
(`probe`, `start`, `drive`, `resume` in
`crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs`) carry 27 to 52
rejections each, all of them ordinary Rust the lint cannot yet admit — `?`,
indexing, equality and addition, iteration, the protocol's own
lifetime-branded generic types, and a `str::repeat` whose return type the
checker does not establish.

**Make that assertion pass without changing it.** That is the whole
acceptance.

## The correction this brief must implement first

The design stated condition 1 as "constructs no raw owned container". The
director corrected it in `a388d11a`, and brief-539's own evidence is why:

```rust
cx.push(&mut values, Expr::string("prefix538".repeat(150_000)))?;
```

A fresh `String` is moved straight into `Expr::string` and then into
`cx.push` — the lease-charging call — in one expression, with nothing
evaluated in between. That is the correct way to hand a payload to a lease.
Under the construction rule it rejects, and so would every lease-correct push.

Condition 1 is now: **no raw owned container is live across a call that can
run guest code or grow retained state.** Implement it with the place-and-move
model A1 already provides. A container moved into a lease-charging call before
any such call is fine; one held across `evaluate` — the `Vec::new` accumulator
in C01 is exactly that — still rejects.

Which calls "can run guest code or grow retained state" is itself a
classification. Make it explicit and fail closed: a callee not known to be
inert is treated as able to.

## What the 146 rejections actually are — four mechanisms, not one

**Revised after the first dispatch stopped, correctly, on the director's
error.** The first version of this section asked for "primitive operators —
only on operands proved primitive, with no overloaded implementation" while
also requiring the probe to lint clean. The probe's `args[0] ==
Expr::small_integer(1)` is `<Expr as PartialEq>::eq` — an overloaded
operator, as the worker confirmed with stable rustc — so the two requirements
contradicted each other. The worker stopped with zero edits rather than
widen. That was right, and the error was in the classification: `Expr == Expr`
is not an operator admission at all. It is a call to a concrete trait impl,
and those belong to **condition 2**, as contracts.

The director tallied all 146 rejections across the four bodies. They are four
different mechanisms, and the largest is a single root cause:

| Mechanism | ~Count | Where it belongs |
| --- | ---: | --- |
| **Type inference blocked by the protocol's generics** — "exact type is unknown" (76), receiver without provenance (11), "only concrete type arguments are modeled" (9), "generic free call not admitted" (6) | ~100 | **Type machinery.** The protocol is generic over the accounting mode `M` and carries a brand lifetime `'id`. The lint models only concrete types, so everything downstream of a protocol call cascades to unknown. |
| `<Expr as PartialEq>::eq`, `<SharedArgs as Deref>`, `<Vec<Expr> as Index<usize>>`, `Expr::string`, `Expr::small_integer`, `str::repeat` | ~10 | **Condition 2 — contracts** on exactly resolved concrete trait impls and constructors. brief-537 already listed `SharedArgs` deref (I01) and `Vec` index (I04) as test-checked contracts. |
| `?`, primitive operators on proved primitives, `for` / range, closures | ~12 | **Condition 1 — the admitted surface.** |
| "unadmitted cleanup" on dropping `Expr` and `EvalError` | 8 | **Cleanup classification.** Dropping releases; for retention it is not a hold. |

The remaining six are the probe's own siblings (`start`, `drive`) and the
protocol's `retained::scoped`, which carry no covered mark yet. That is
induction working as designed: once a sibling lints clean, its mark exists.

## The work, in this order

### 1. The root cause first — type machinery for the protocol

- **The mode parameter `M` is finite.** brief-526 §3.3 already stated the rule:
  *"The protocol's sealed Armed/Unarmed instantiations are a finite special
  case, not permission for general generic handlers. Enumerate both concrete
  substitutions and their calls."* Check each body under both instantiations.
  Any other open generic stays refused.
- **Lifetime parameters carry no allocation.** The brand `'id` is a
  compile-time construct that rustc's borrow checker already enforces; for
  retention it can be treated as opaque. Say so where you rely on it.
- **Then remeasure before doing anything else.** Predict, in `PROGRESS.md`,
  how many of the ~100 cascade away, then measure. This is the step most
  likely to move the count, and knowing by how much decides the rest.

### 2. Contracts for the concrete calls

For each exactly resolved callee the bodies use, a contract record in
brief-537's format, installed as a mark: its effect (inert, borrowing, or
producing an owned value), its discharge category, a fingerprint, and a
falsifier. The resolution itself is checked by the oracle, with the negative
control. `<Expr as PartialEq>::eq` is inert for retention — it borrows both
sides and allocates nothing — and its contract says so; it is **not** a
precedent for admitting overloaded operators generally.

### 3. The admitted surface

`?` on a concrete `Result` with an identical error type and both exits
modelled (brief-536 row 6); primitive operators on operands **proved**
primitive (rows 10 and 12); `for` and range over the exact std instances the
bodies use; closures only if a closure is genuinely inside one of the four
bodies — check which lines belong to them before admitting anything for a
closure, since a test-harness closure around the builtin is not one of them.

### 4. Cleanup

Classify the drop of `Expr` and `EvalError` for retention: dropping releases
memory and holds nothing. State plainly what that classification does **not**
claim — the destruction of a deep `Expr` is standing debt for stack depth
(`TODO.md`, Standing Engineering Debt), which is a different property.

**Scope is still set by the probe.** Nothing beyond what these four bodies
need. If one of them genuinely needs an unresolvable dispatch — `dyn`, an open
generic other than the finite mode, a function-pointer call — stop and report.

## Acceptance — predict, then measure

Write these into `target/brief-540/PROGRESS.md` **before** the final run:

1. `local_production_acceptance` passes, **unmodified**. All four
   `LeaseProbe538` bodies lint clean.
2. C01 and C02 **still fail** — today 19 and 57 rejections. Predict the new
   counts. The `Vec::new` accumulator at `runtime/options/core.rs:101` and the
   `vec![args[0].clone(), value]` at `:259` must still reject, because both are
   live across evaluation; converting them is the next step, not this one. A
   lint that passes C01/C02 after this brief is broken.
3. Production coverage stays **0**.

Report predicted against measured, reconciled as located rows.

## Scope

**An objective and a fence, not an enumeration.** A MAY list has been short
six times on this project.

**The objective.** Inside `crates/rum-stdlib/src/tests/retention_closure/`
and its inventories and registration, implement the corrected condition 1 and
the admissions `LeaseProbe538` needs — including files the guidance does not
name.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name appears in your report's first section
with `path:line` ranges and one line on why. **An undisclosed edit is the
violation; a disclosed necessary edit is not.**

**The fence — STOP conditions:**

- **All production Rust is read-only**, including the probe itself. You make
  the lint understand the probe; you do not change the probe to suit the
  lint.
- **The acceptance assertion is read-only.** No edit, no relaxation, no
  `#[ignore]`.
- **No traversal.** Every admission is local to one body. No callee body is
  read to decide a caller's verdict.
- **No admission beyond what the four bodies need.**
- The never-admit list stays refused.
- `crates/rum-core/`, all `Cargo.toml` files and `Cargo.lock`. No new
  dependency; the pin is stable 1.85.0.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## rum / Rust pitfalls (read twice)

1. **An admission with a silent fallback is a hole with a green test.** Every
   admission fails closed at its own edge.
2. **`let _: T = expr` coerces, and so does a turbofish argument.** Type
   claims go through the oracle's invariant carrier.
3. **Construction is not the hazard; retention across evaluation is.** Do not
   reintroduce the construction rule under another name.
4. **A callee not known to be inert can run guest code.** Default to that.
5. **The existing thirty checker tests stay green**, and the oracle cache
   keeps working.
6. **Read rustc's and rust-analyzer's source** for `?` desugaring,
   `IntoIterator`, and index lowering rather than re-deriving them. Both are
   MIT/Apache-2.0 and rum is MIT; say what you adapted.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host: the tree as brief-539 left it has thirty
checker tests passing and one failing — the acceptance assertion — and
`cargo test --workspace` at 2 619 passed with that single failure plus nine
environment-blocked ZeroMQ tests in your sandbox.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

**Microbench A/B: not applicable** — production is read-only.

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment. Do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

After each of the four steps lands, write `target/brief-540/PROGRESS.md` with what is
done and verified, the exact next action, the tree's state, and the running
count of `LeaseProbe538` rejections per body. **Your acceptance predictions go
here before the final run.**

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config.
- Respect the fence; outside it, the disclosure duty applies.
- **No new `unsafe`.** No new dependency. No nightly.
- No `#[ignore]`, no feature gate on a required test, no expectation edited
  after measuring.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Scratch goes under `target/brief-540/`.
- **Stop-loss, keyed on progress.** Report two numbers per attempt: total
  `LeaseProbe538` rejections across the four bodies, and required tests
  passing. Stop only when **two consecutive** attempts move neither. A compile
  error you are actively reducing is not a wall.

## Report format (your final message)

1. What changed — per file with line ranges, including every unnamed file,
   with why.
2. The corrected condition 1: how retention across evaluation is decided, and
   how "can run guest code" is classified.
3. Each of the four steps: for the type machinery, the predicted and measured
   cascade; for each contract, its effect, discharge category and oracle
   check; for each admission, rule, soundness argument and edge; for cleanup,
   what the classification claims and does not. Falsifiers red then green
   throughout.
4. `LeaseProbe538`: rejections per body falling to zero, and the acceptance
   assertion green, unmodified.
5. C01/C02 predicted versus measured, and confirmation that both raw
   constructions still reject.
6. Verification — every command with verbatim output.
7. Stop-loss / open questions, including anything on the never-admit list
   the probe turned out to need.
