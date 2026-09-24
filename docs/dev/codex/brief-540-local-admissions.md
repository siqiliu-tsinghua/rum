# brief-540 — local admissions, so lease-correct code can lint clean

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Where this starts

**Continue the working tree exactly as brief-539 left it.** Do not restart,
revert, stash or clean anything. brief-539 built the local lint and it works:
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

## The admissions — local, one at a time, each earned

Each admission is a **local rule about one construct in one body**, not a
traversal. brief-536 gave soundness arguments for most of them when they were
still framed as stages; they carry over. For each: the rule, its soundness
argument in a few sentences, what it refuses at its own edge, a falsifier
watched red then green, and an oracle check of any type or resolution claim it
makes.

- **`?` on a concrete `Result`** whose error type is identical on both sides,
  with both exits and the cleanup of live owners modelled (brief-536 row 6).
- **Primitive operators** — `==`, `+` and the rest the bodies use — only on
  operands proved primitive, with no overloaded implementation (brief-536
  rows 10 and 12).
- **Indexing** a borrowed slice or `Vec`, including the out-of-range panic
  path and its unwind cleanup (brief-537 I04).
- **Iteration** over the specific borrowed iterators the bodies use, with the
  `IntoIterator`, `next` and iterator-drop effects modelled for those exact
  std instances.
- **The protocol's own types** — `RetainedValues<'id, M>`, `RetainedResult`,
  the lease and brand types. A lint that cannot read the protocol it enforces
  is not finished.
- **`str::repeat` and any other std method return type** the bodies need,
  established precisely — or, if that is not possible, recorded as a trusted
  leaf with the full record the existing leaf rules require.

**Scope is set by the probe.** Admit what these four bodies need and nothing
else. If one of them uses something on the never-admit list — `dyn` dispatch,
an open generic, user `Deref` selection without proof, a function-pointer
call — **stop and report it** rather than widening. A test harness closure
*around* the builtin is not one of the four bodies; check which lines actually
belong to them before admitting anything for a closure.

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

After each admission lands, write `target/brief-540/PROGRESS.md` with what is
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
3. Each admission: rule, soundness argument, edge, falsifier red then green,
   oracle check.
4. `LeaseProbe538`: rejections per body falling to zero, and the acceptance
   assertion green, unmodified.
5. C01/C02 predicted versus measured, and confirmation that both raw
   constructions still reject.
6. Verification — every command with verbatim output.
7. Stop-loss / open questions, including anything on the never-admit list
   the probe turned out to need.
