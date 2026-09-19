# brief-481 — the destruction declaration, proved on entropy providers

Step A of the host-payload destruction contract (plan: brief-480). It
introduces the declaration and makes exactly one payload interface require it.
Entropy is the proving ground because it exercises every property the mechanism
needs — a public trait, ordinary construction, trait-object erasure, shared
ownership through `Arc`, host integration, and external compilation — without
touching callback invocation or the evaluator.

**This step is the first intentional API break.** The user has ruled it
acceptable (2026-09-19): rum has no downstream consumers and `v0.2.0` is
freshly tagged, so the API is deliberately not locked.

## Goal

A safe, public, opt-in marker trait `BoundedDestruction` exists, and
`EntropySource` requires it. An `EntropySource` implementor that does not
declare a stance **fails to compile**, and that failure is certified from a
crate outside this workspace, not only from inside it. All four in-tree
implementors carry the declaration with a written rationale. Entropy behavior,
provider identity, threading bounds and erased construction are unchanged.

## Context pointers

- `AGENTS.md` (operational notes, Testing Policy hard lines) and
  `docs/dev/codex/brief-480-host-payload-destruction-contract.md` for the plan
  this step comes from.
- `crates/rum-runtime/src/runtime/random.rs:141-164` — the trait, the provider,
  and both constructors (`new`, `from_arc`).
- `crates/rum-runtime/src/runtime/kernel_api.rs` — where `EntropyError`,
  `EntropyProvider` and `EntropySource` are re-exported to consumers.
- `crates/rum-host/src/entropy.rs:30` — `StdEntropySource`, the one
  production implementor, reached through a `OnceLock` shared handle at `:22-27`.
- The other three implementors: `crates/rum-stdlib/src/tests/random.rs:77`,
  `crates/rum-stdlib/src/tests/valueq_effect_suppression.rs:47`,
  `crates/rum-host/tests/lifecycle_buildout.rs:116`.
- `crates/rum-runtime/src/runtime/native_stack_guard.rs:19,79` — the
  `compile_fail` doctest precedent. Useful documentation; **not** the
  authoritative certificate here (see Verification).

## Scope

Files you MAY modify:

- **new** `crates/rum-runtime/src/runtime/host_payload.rs` — the trait and its
  rustdoc.
- `crates/rum-runtime/src/runtime/mod.rs` and
  `crates/rum-runtime/src/runtime/kernel_api.rs` — module wiring and re-export.
- `crates/rum-runtime/src/runtime/random.rs` — the supertrait bound only.
- The four implementor files listed above — the declaration plus its rationale.
- **new** `crates/rum-host/tests/host_payload_destruction_contract.rs`, a new
  fixture directory under `crates/rum-host/tests/fixtures/`, and a new
  `scripts/check-host-payload-destruction.py` runner.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` — **position
  drift only**, see the inventory rule below.
- `docs/guide/embedding.md` — only if a sentence there becomes false
  (`:76` names `StdEntropySource`); a minimal correction, not a new section.

Files/areas you MUST NOT touch:

- `crates/rum-runtime/src/runtime/file_io.rs`,
  `crates/rum-runtime/src/runtime/model/session.rs`,
  `crates/rum-runtime/src/runtime/suspension/**` — steps B, C and D.
- The ownership scanner's **code** (`ownership_graph/*.rs`) and its strict-check
  design — step E. Only the pinned `.tsv` may drift here.
- `SECURITY.md`, `docs/host-embedding-and-hardening.md`, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, any other `docs/**`, `docs/dev/codex/**` except your
  own report.
- `crates/rum-core/src/expr.rs` representation, `.git`, git config, `Cargo.lock`.

No default-path cost: this step adds a trait bound and no runtime work. If you
find yourself adding a branch, an allocation or a counter to any evaluation
path, STOP and report.

## The mechanism, as ruled

1. **`BoundedDestruction` is a safe, public, opt-in marker trait** with no
   methods, no blanket implementation, no derive, and no automatic
   implementation from `FrameRootTrace`, `FrameAbandon` or any inventory
   membership. Implementing it *is* the stance; an empty impl block is the
   whole ceremony.
2. **`EntropySource: BoundedDestruction + Send + Sync`.** Preserve `Send +
   Sync` and the `'static` admission in `EntropyProvider::new` exactly as they
   are; the marker adds no threading requirement of its own.
3. **The rustdoc must be honest about what this is.** Required content, in your
   own words:
   - what the implementor promises: a bound on **native stack consumption
     during destruction** of any state the value can reach — its own `Drop`,
     compiler-generated field destruction, captured values, and the
     last-strong-holder case when ownership is shared;
   - what it does **not** promise: no time bound, no allocator guarantee, no
     promise that foreign code cannot panic;
   - that the compiler enforces **that a stance was taken**, never that the
     stance is true, and that **no `unsafe` may ever rely on it** (there is no
     `unsafe` in `rum-core`/`rum-runtime` and this must not become the reason
     for the first).
   Do not name it "certified", "proved" or "verified" anywhere in the rustdoc
   or in a type name. The word for what the ledger records is **declared**.
4. **Each of the four implementations carries a one-line rationale** in a doc
   comment on the impl: why that type's destruction is bounded (for
   `StdEntropySource`, a fieldless unit struct; for the test sources, whatever
   their fields make true). A declaration with no rationale is a rubber stamp
   and is rejected at the gate.

## The inventory rule (read twice)

Editing `random.rs` moves line numbers, and the ownership inventory pins exact
positions. You MAY update the `.tsv` files for that drift, under one condition
that the gate checks: **every row's content is unchanged except its line and
column columns.** No row may be added, removed, reclassified, or have its
witness path altered. If the diff shows anything else — a new
`UnknownHostPayload`, a changed ownership path, a vanished row — STOP and
report it. A drifting snapshot must never become a way to accept a new unknown.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Then the crate-edge certificate, which is the point of this step. A boundary
can hold inside this repository and evaporate outside it: a source scan cannot
establish a property about code that does not exist yet, so the authoritative
evidence is a **compiler error from an independently rooted crate**.

- The runner generates consumer crates under repository-local `target/`, each
  with its own `[workspace]` table, path dependencies on this tree, and a
  separate target directory. Compile with `cargo check --offline`. Do **not**
  execute a consumer program.
- Negative cases, all three of which must fail to compile **for the missing
  declaration specifically**: an `EntropySource` impl with no marker; the same
  passed to `EntropyProvider::new`; the same passed through the erased
  `EntropyProvider::from_arc`.
- Positive controls, which must compile: the same sources with an explicit
  `impl BoundedDestruction`, through both constructors.
- The runner must **read the compiler diagnostics** and reject a failure caused
  by anything else — an unresolved import, a missing method, a dependency
  resolution error. A test that passes because the fixture did not build for an
  unrelated reason is worth nothing.
- Wire the runner into the workspace gate through the new `rum-host` test, run
  offline, with its own target directory. Use `scripts/run-bounded.py --timeout
  SECONDS` and derive the bound from a recorded run of the same shape; say what
  run you derived it from.

## Falsifier (MANDATORY — a check nobody has watched fail is not known to work)

Baseline sensitivity first: **the negative fixtures must be unchanged legacy
source** — an `EntropySource` implementor written the way it is written today —
which compiles on the pre-change tree and fails on the post-change tree. A
fixture that imports `BoundedDestruction` cannot serve as that baseline, since
it fails on the old tree for the trivial reason that the name does not exist.
Report the pre-change compile of those fixtures verbatim.

Then perturb the enforcement: temporarily remove the `BoundedDestruction`
supertrait from `EntropySource`, rerun the certificate runner, and show that
the negative consumer now compiles and the runner goes **red**. Restore it and
rerun to green. Paste both outputs. Removing the *fixture* instead of the
*enforcement* proves nothing.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave everything in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No differential testing against other engines.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- Do not run any workload that reproduces a crash, an overflow or a process
  termination. Nothing in this step needs one.
- **Cost is not an argument** (user ruling 2026-08-27): a count of call sites
  describes the work, it does not rank an option down. Do not propose a
  documentation-only contract, and do not propose a stopgap to be fixed later.
- Minimal diff: no drive-by reformatting, renames, or comment changes outside
  the task.
- Paste real verbatim output. If a run did not reach green, say so — never
  fake it.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — every command, verbatim output, including the workspace test
   total and the certificate runner's cases one by one.
3. The falsifier — the pre-change compile of the legacy fixtures, then the
   enforcement removed (runner red), then restored (runner green), verbatim.
4. The inventory diff — show that only line/column columns moved.
5. Stop-loss / open questions — where you stopped and why.
