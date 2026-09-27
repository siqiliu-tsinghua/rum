# brief-542 — the option region's local reads: C02 from 34 to 15

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

First of the four steps brief-541 planned
(`docs/dev/codex/brief-541-option-region-conversion-plan.md`, report from line
165; read its full row table). Make C02's **read-only** operations locally
checkable, without changing production:

- the borrowed `let Expr::Call { head, args } = expr else { … }` with a
  diverging `else`, preserving reference layers and cleanup — a borrowed
  pattern moves no field and mints no owner;
- the exact helper predicate `is_system_symbol_name`
  (`runtime/expr_runtime_helpers.rs:420-423`) as an installed contract;
- the `&SharedExpr -> &Expr` coercion as contract **I02**, which is a real
  call to a concrete `Deref` impl, not a free coercion;
- `bool && bool` with short-circuit flow, both branches' ownership states
  joined;
- confirming the existing `len` / index / `Clone` adapters resolve once field
  types are known — no new general method search.

**Predicted end state, written here before any work: C01 17, C02 15**, with
removed rows C02-02, 03, 05, 06, 07, 08, 09, 10, 11, 12, 13, 17, 18, 19, 20, 26,
27, 28, 29. Both raw constructions — `core.rs:101` and `:259` — still reject.
Production coverage stays 0. Any other movement gets a located
reconciliation, never an edited expectation.

## Director rulings this brief carries

- **U2 — keep brief-540's narrow records.** B04, B05, I01, I04 and the P
  records are installed and discharged *for the probe's exact instances*. They
  stay as they are. Their narrow strength must not be read as a quantitative
  proof anywhere else, and every still-`Pending` record remains a non-mark.
- **U4 — no quantitative claim without proof.** Where a contract touches
  `Expr` clone or construction, state only what its evidence establishes.
  Fieldwise `Clone` gives a type, not a capacity receipt.
- **U5 — option-specific workloads are authorized** for later briefs; this
  one is production-read-only and needs none.
- **U1 — the generic `R`** is settled for step 544 (a runtime
  certified-registration domain). It does not affect this brief; do not
  anticipate it.

## Scope

**An objective and a fence, not an enumeration.**

**The objective.** Inside the retention checker
(`crates/rum-stdlib/src/tests/retention_closure/`) and its inventories and
registration, implement exactly the support these C02 operations need,
including tests, oracle controls and contract records — including files the
guidance does not name.

**Guidance.** `local/mod.rs` and `flow.rs` own bindings and ownership;
`surface.rs` operators and indexing; `calls.rs` / `protocol.rs` exact
operation selection; `contracts.rs` and the inventory the nineteen-field
records. If you touch `flow.rs` (about 22.7k characters), split it by semantic
responsibility rather than by line count.

**The disclosure duty.** Every touched path appears in your report with line
ranges and why. An undisclosed edit is the violation; a disclosed necessary
one is not.

**The fence — STOP conditions:** all production Rust; all manifests and the
lockfile; `rum-core`; the interner, gensym, snapshots and `S`; `AGENTS.md`,
`PROGRESS.md`, `TODO.md`. No macro admission, no general `Deref` / `Index` /
trait / generic admission, no `Pending` record used as a mark, no promotion.

## Falsifiers — each watched red, then green

Wrong borrowed field or an extra reference layer; a non-diverging `else`; a
partial move out of the borrowed pattern; a changed concrete `Deref` impl; a
competing imported trait; the `&&` right-hand side's ownership on each branch;
a stale contract; a `Pending` record offered as a mark. And both original
`RET001` controls must remain red. Type and selection claims go through the
oracle's invariant carrier with the same-name negative control.

## Pitfalls

1. `usize == usize` is primitive; `Expr == Expr` is a trait call — a contract.
2. Unknown-versus-`Vec` is not evidence of coercion.
3. A compiler result is reusable only with a complete matching fingerprint.
4. Clone's known type is not a quantitative receipt.

## Verification (MANDATORY)

Director baseline on this tree: 37 checker tests green, C01 17, C02 34;
`cargo test --workspace` 2 635 passed, 0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Pass `< /dev/null` to anything that runs `rum-cli`. Only C02's count
expectation may change, and only to the predeclared 15. **Microbench A/B: not
applicable** — production is read-only.

Nine native ZeroMQ tests cannot bind loopback in your sandbox; report them as
skipped-by-environment. Do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

All scratch, raw output and `PROGRESS.md` go under `.temp/brief-542/`. After
each milestone record changed files, observed counts, falsifiers red/green,
compile state and the exact next action.

## Hard rules

No `git`. No production edits. No new `unsafe`, dependency or nightly. No
traversal — a caller's verdict never reads a callee's body. No general
admission, no `Pending` waiver, no weakened contract, no ignored failure.
Testing Policy is a hard line; using rustc as an oracle for Rust facts about
this repository is a director ruling and not its banned shape. Stop-loss is
keyed on progress: stop only after two consecutive attempts produce neither a
new falsifier nor a lower unresolved count.

## Report format

Changed paths and why; every command with verbatim rows; 17/15 predicted
versus observed, as located rows; each admission or contract and its
falsifier; remaining obligations and anything unsettled. Append to this file
and leave the tree uncommitted.
