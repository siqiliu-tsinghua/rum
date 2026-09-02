# brief-417 — every builtin declares how it can grow, and a wrong declaration fails

Shape: **implementation**. Worker: **cdx (Codex)**.

Gap B, step 1. Rewritten from your own brief-414 draft; read that report's
§3 inventory before starting, because it is the authority for the
classification and this brief does not restate it.

## Goal

Two things must be true when you are done, and the second is the one that
matters:

1. Every builtin declaration carries a memory-growth contract, and **omitting
   it does not compile**.
2. **A wrong `NonAmplifying` declaration fails a test.** A classification that
   nothing can falsify is a convention wearing a type's clothing, and this
   project has replaced enough of those to know the difference.

No new prechecks, no behaviour change, no limit change. This brief builds the
structure that briefs 418 and 419 fill in.

## Why the second requirement exists

Under compile pressure, the cheapest way to make 709 construction sites build
again is to write `NonAmplifying` at every one of them. That would produce a
large diff, a green gate, and no information. Your own brief-414 report said it
plainly: "`NonAmplifying` remains a reviewed assertion." I am not accepting it
as one.

## Context pointers

- `AGENTS.md` first.
- **`docs/dev/codex/brief-414-memory-accounting-gaps.md` §3** — your inventory
  of every multiplicative site, with `path:line`, growth factor, and current
  precheck. This is the source of truth for classification. Do not re-derive it
  from scratch and do not silently disagree with it: if a row is wrong, say so
  in the report as a correction, with evidence.
- `crates/rum-runtime/src/runtime/metadata.rs:296-460` — `BuiltinSpec`, its
  seven public fields and its nine `const` constructors.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:133-220` — dispatch is
  already generic over `CallBuiltinSpec: Copy`, implemented for `BuiltinSpec`
  (`:198`) and `SuspendedBuiltinSpec` (`:212`). It reads only `handler()`,
  `capability()` and `name()`.
- `crates/rum-runtime/src/runtime/kernel_api.rs:69` — `BuiltinSpec` is public
  API.
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` — the enforcement
  shape that just landed (brief-415): a macro emitting both a checked-in
  inventory and an exhaustive pattern, so an omission stops compiling. Copy
  the idea, not the code.
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:269-333` —
  `expr_memory_proxy_bytes`, which you will need for requirement 2.

## Director-measured, so you do not have to re-derive it

- Construction sites: **697 constructor calls + 12 struct literals = 709**,
  across `BuiltinSpec::{new, with_hold, with_attributes, special,
  special_with_hold, special_with_attributes, ordinary, ordinary_with_hold,
  ordinary_with_attributes}`. `ordinary` alone is 335.
- No crate outside `rum-runtime` constructs a `BuiltinSpec` today.

Verify both if you like; report it if I am wrong.

## Deliverable

### 1. A mandatory contract on the declaration type

A `MemoryGrowthContract` with no `Default`, no setter, and no builder method —
a **required parameter of every `BuiltinSpec` constructor**, so a new builtin
cannot be declared without choosing. A builder method would be forgettable,
which defeats the point.

Start from your drafted variants (`NonAmplifying`, `Preflight`, `Incremental`,
`Delegated`) but treat them as a proposal, not a specification: name and shape
them for what the inventory actually contains. If the inventory needs a variant
you did not draft, add it and say why. Briefs 418-419 will attach projector
functions to `Preflight`; leave room for that without building it now.

### 2. The classification, from the inventory

Every installed builtin classified, driven by brief-414 §3. Any installed
builtin the inventory does not cover is a **finding to report**, not a blank to
fill with `NonAmplifying`. List them.

### 3. A sorted, checked-in invariant

Covering every declaration and its contract, so that a future change to a
classification shows up as a reviewable diff line rather than a silent edit —
the `frame_root_trace` and `session_memory` shape.

### 4. The falsifier — the load-bearing deliverable

Under a test or armed mode only, a check that catches a handler declared
`NonAmplifying` whose result is materially larger than its inputs.

**Do not do this with a full session walk per handler.** The walk is why the
stride exists; running it around every builtin call would be brutal even in
armed mode. The cheap local form is enough: compare
`expr_memory_proxy_bytes(result)` against the summed
`expr_memory_proxy_bytes` of the arguments, and fail a `NonAmplifying`
declaration whose result exceeds that sum by more than a small stated
allowance. That catches the class of mistake this deliverable exists for — a
builtin that multiplies its input and claims it does not.

State its limits honestly in the report: it does not see session-state growth,
and it cannot bound an arbitrary host callback. Both are fine. It only has to
make a wrong declaration falsifiable, and it will be exercised by the whole
existing 2 291-test suite, which is what makes deliverable 2 trustworthy
instead of decorative.

**Then run it and report what it caught.** If it fires on nothing at all,
suspect the check before believing the classification — write in the report how
you confirmed it can fire, for example by deliberately misdeclaring one builtin
and observing the failure, then reverting. A falsifier that has never been seen
to fail is not known to work. This project has been burned four separate times
by an instrument that reported success while measuring nothing.

### 5. Numbers for the hot path

`BuiltinSpec` flows through ordinary dispatch **by `Copy`**, so a bigger
declaration type is a bigger per-dispatch copy of a field dispatch never reads.
Report `std::mem::size_of::<BuiltinSpec>()` before and after, and run the
microbench A/B below.

**If the A/B regresses, STOP and report.** Do not refactor dispatch onto a lean
record in this brief to rescue the number — that is a second variable and a
separate decision. `SuspendedBuiltinSpec` already shows what a lean record
looks like; if the measurement says we need one on the hot path, I will brief
it.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/metadata.rs`
- `crates/rum-runtime/src/runtime/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/builtin_catalog.rs` and every `.rs` beneath
  `crates/rum-runtime/src/runtime/builtin_catalog/` (18 files)
- `crates/rum-runtime/src/runtime/session/session_state.rs`
- `crates/rum-runtime/src/runtime/tests/mod.rs` and a new test module beneath
  `crates/rum-runtime/src/runtime/tests/`
- `crates/rum-stdlib/src/lib.rs`, `crates/rum-stdlib/src/catalog/mod.rs` and
  every `.rs` beneath `crates/rum-stdlib/src/catalog/` (32 files)
- any other file that only needs its `BuiltinSpec` construction updated to pass
  the new argument — that is mechanical and expected. **Anything beyond a
  construction-site update, STOP and report.**

Files you MUST NOT touch:

- `crates/rum-runtime/src/runtime/debug_tools/resources/` — the accounting is
  settled by brief-415; you read `expr_memory_proxy_bytes`, you do not change
  it or the stride or any `ResourceLimits` default.
- `crates/rum-core/src/expr.rs`'s `Expr`/`Symbol` representation.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending your report
  to this brief, `.git`, git config, `Cargo.lock`.

## Hard rules

- **No behaviour change.** No new precheck actually refuses anything in this
  brief; no message changes; no limit, default or stride changes. If a
  classification makes you want to add a precheck now, that is brief 418.
- **No default-path cost.** The falsifier must not run when no ceiling is armed
  and no test mode is on, and it must not add an allocation, a lock, or a
  non-trivial branch to ordinary dispatch. The disabled path is a single cheap
  check that returns immediately.
- **Preserve suspension/resume shape.** Do not collapse a `suspend_with_frame*`
  into an eager call. If the contract must survive a suspension, note that
  `SuspendedBuiltinSpec` (`session/session_state.rs:139-153`) keeps three
  fields and drops the rest — decide deliberately and say what you decided.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **Do not weaken an existing test to make the build pass.**
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **This host can bind loopback**; the nine native ZeroMQ tests in `rum-jupyter`
  genuinely run here — I observed 9 of 9 during the brief-416 gate. If your
  sandbox denies loopback and you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so
  and mark every native result unobserved. Never report a rate for a test you
  did not watch run.
- Minimal diff outside the mechanical construction-site updates.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline -p rum-runtime
cargo test --offline -p rum-stdlib
cargo test --offline --workspace
```

Then the crate-edge proof, which a source scan cannot give: build a throwaway
crate under `/tmp` that depends on `rum-runtime` by path and tries to construct
a `BuiltinSpec` **without** the contract argument. **Paste the compiler error.**
A boundary that holds only inside this repository is not a boundary; the
brief-396 review established that a scan of in-repository callers cannot prove
a property about code that does not exist yet. Remove the crate afterwards.

Then the microbench A/B, base build against candidate build in one window:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the per-label `rum_seconds` rows for the dispatch-sensitive labels and
their same-tier neighbours: the literal, symbol-ownvalue, set-ownvalue and
list-literal rows, the exact and pattern downvalue hit/miss rows, and the
matcher and `replaceall-simple` rows. **Name your control labels before
judging** — a whole-run offset moves every label together, and a lopsided
direction split is not evidence. Profiling env off for every timing number.

## Report format

1. What changed — per file, line ranges, one-line why. Group the mechanical
   construction-site updates; do not list 709 of them.
2. The contract type: its variants, and why they differ from your draft.
3. The classification: how you derived each from the inventory, and every
   installed builtin the inventory did not cover.
4. **The falsifier: how it works, what it caught, and your evidence that it
   can fire.**
5. `size_of::<BuiltinSpec>()` before and after, plus the microbench rows with
   named controls.
6. Verification — every command, verbatim output, including the crate-edge
   compiler error.
7. Stop-loss / open questions.
