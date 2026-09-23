# brief-538 — the lease protocol, the activation bridge, and a closed reentry set

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

Build the mechanism that actually closes the hole. The static checker was only
ever the enforcement that nobody bypasses it; **the fix is the lease**. Three
pieces, proved together on a test-only builtin, with **no production region
converted yet**:

1. **The lease protocol** — the only way to hold a data-dependent owned
   payload across evaluation in accounted code, with private constructors.
2. **The activation bridge** — an evaluation from accounted code exposes the
   live outer prefix, so a limit that arms *later* still sees it.
3. **A runtime-closed reentry set** — the kernel's option loop invokes only
   `OptionReentry` implementations that were registered, and refuses any
   other.

The design is `docs/dev/retention-closure-design.md`, and the contracts this
brief must make dischargeable are defined in
`docs/dev/codex/brief-537-contract-content.md` (report from line 206).

## Why these three and not the conversion

The user ruled the order on 2026-09-23: pin the contracts (done, brief-537),
then build the protocol against them (this brief), then the local lint that
enforces them. Converting the option region comes after, onto a protocol whose
correctness is already proved.

Of brief-537's sixteen boundary contracts, this brief makes three dischargeable:

- **B09 — `Session::evaluate`**: *"Run with a live caller prefix already visible
  to accounting."* The activation bridge is exactly this.
- **B12 — `insert_suspension_frame`**: the lease held in a frame must transfer
  on a real suspension and be preserved on resume.
- **B03 — `<R as OptionReentry>::resume`**: the one contract classified
  *assumed*. This brief moves it to **runtime-enforced**.

## The three pieces

### 1. The lease protocol

- A lease and a retained-value type, both with **private constructors**, so
  accounted code cannot obtain an owned accumulator any other way. Operations:
  reserve, charge, release, transfer.
- **Every lifetime exit closed**: normal publication, early return,
  `EvalError`, panic unwind, drop, suspension, resume, abandonment. A lease
  alive across a suspension transfers **exactly once**. A lease held in a
  suspension frame is a root, and `SuspensionFrame: FrameRootTrace` makes a
  missing traversal a compile error — do not keep the lease out of the frame
  to avoid that.
- **`R` enters admission only.** Both admission entries in
  `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs` —
  `enforce_memory_limit_for_expr` at `:154`, which computes `S + W` today, and
  `enforce_active_memory_limit_for_projected_bytes` — gain the `R` term. `R`
  never enters `S`, `S`'s formula is not changed to absorb it, and `R` does not
  appear in `MemoryInUse[]`, `MaxMemoryUsed[]` or `ByteCount`. `S`'s *reading*
  may move because the accountant's own storage is charged; that is required.
- **Erasure when unarmed**, on the mode seam A-E built. Name where it is
  proven. The admission check already sits behind `current_memory_limit`;
  keep every new cost behind that same branch.

### 2. The activation bridge

`AccountingContext::evaluate` (`crates/rum-runtime/src/runtime/accounting_mode/mod.rs:76`)
returns a bare `Result<Expr, EvalError>` today, so it is not a bridge. Replace
or extend it so that evaluation from accounted code:

- exposes the caller's live prefix to accounting **before** guest code runs;
- returns a lease-bearing outcome that must be adopted before retention or the
  next step.

The case that makes this non-trivial, and that a naive lease taken at entry
fails: **an outer builder accumulates while unarmed, and a later callback arms
a nested `MemoryConstrained`.** At the moment of arming, the accumulated
prefix is a Rust local that neither `S` nor `W` can see. The bridge must make
it visible then. brief-524 §4 records this as an acceptance condition; it is
not optional.

### 3. The runtime-closed reentry set

A compile-time seal is **not expressible** here: `OptionReentry` is defined in
`rum-runtime` and its production implementor `OptionConsumer` lives in
`rum-stdlib` (`crates/rum-stdlib/src/option_consumers.rs:85`), and Rust
visibility is crate-granular — there is no "visible to exactly these two
crates". A `#[doc(hidden)]` supertrait would admit external crates too and
would only look sealed. The user ruled, on 2026-09-23, for a **runtime** close
instead:

- `rum-runtime` holds the set of permitted reentry types, keyed by `TypeId`.
  The trait already requires `'static`, so `TypeId` is available.
- `rum-stdlib` registers `OptionConsumer` at install.
- The option loop checks that the `R` it was given is registered **before**
  calling `resume`, and refuses otherwise with a named, catalogued message.
- State the guarantee precisely: *the kernel's option loop invokes only
  registered reentries*. External code calling `R::resume` directly does not
  go through the kernel and is outside this guarantee.
- If the set lives on `Session`, the exhaustive field schema
  (`occurrence/schema.rs:19-42`) will refuse to compile until the new field is
  classified. That is the barrier working; classify it honestly, and charge
  its storage if it is not genuinely fixed-size.

## Falsifiers — watched red, then green

Each with a compile-pass control built first where it is a `compile_fail`, and
complete output saved under `target/brief-538/` before quoting.

- **`compile_fail` proofs on the protocol**: a forged or private lease
  constructor, a reused moved retained value, an unleased raw extraction, a
  branded handle escaping its lifetime. Pin the expected error.
- **The activation falsifier**: accumulate a large prefix unarmed, arm a nested
  limit inside a later callback, and show admission trips — with a
  **prefix-free control that does not trip**. A final-result-only rejection
  does not discharge this.
- **The lifetime matrix**: each exit above, including suspension-transfer
  exactly once and abandonment, with the lease's charge observed released —
  not assumed released.
- **The reentry set**: the in-tree test implementor `ListConsumer`
  (`crates/rum-runtime/src/runtime/options/tests/resume.rs:6`) refused while
  unregistered, accepted once registered.
- **The `R`/`S` separation**: an `R`-only activity leaves `MemoryInUse[]`,
  `MaxMemoryUsed[]` and `ByteCount` unchanged, while admission sees it.

## Scope

**An objective and a fence, not an enumeration.** A MAY list has been short
six times on this project.

**The objective.** Inside `crates/rum-runtime/` and `crates/rum-stdlib/`,
build the protocol, the bridge and the reentry set, proved on a test-only
builtin — including files the guidance does not name.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name appears in your report's first section
with `path:line` ranges and one line on why. **An undisclosed edit is the
violation; a disclosed necessary edit is not.**

**Guidance**: `runtime/accounting_mode/`, `runtime/debug_tools/resources/`
(the admission hook and the maintained-owner accessors), `runtime/options/`
(the reentry check), `runtime/suspension/` and frame definitions,
`rum-stdlib`'s install path and `option_consumers.rs` for registration, the
occurrence schema and frame inventories the barriers require, owning tests,
and `docs/guide/resource-limits-and-sandbox.md` if armed admission behaviour
changes.

**The fence — STOP conditions:**

- **`crates/rum-core/`**, in particular `expr.rs`. Representation is fixed.
- All `Cargo.toml` files and `Cargo.lock`. No new dependency; the pin is
  stable 1.85.0.
- **The `S` model.** Folding `R` into the maintained total to avoid a new hook
  is the shortcut this fence exists to prevent.
- **No production region conversion.** C01/C02, `CompoundExpression`'s
  `compound_from`, and every other N witness stay as they are. Proving the
  protocol on a test-only builtin is the deliverable.
- **No lint, no coverage promotion.** Those come after.
- The interner, gensym, snapshot format. `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`.

## rum / Rust pitfalls (read twice)

1. **A lease taken at entry records nothing** when arming happens later. The
   activation case is the whole point.
2. **A resumed frame must not assume unique ownership.** `Arc::try_unwrap`
   demanding uniqueness has panicked this kernel before.
3. **Do not collapse a `suspend_with_frame*` into an eager call** to simplify
   the ownership story.
4. **Shared children are multiple occurrences** under the occurrence model;
   transferring one `Arc` clone does not discharge the others.
5. **Default-path cost is unconditional cost.** A branch or atomic load paid
   on every unarmed call is a regression; keep new work behind the armed
   branch and prove the erasure.
6. **`compile_fail` passes when the code fails for the wrong reason.** Build
   the control first and pin the error.
7. **A certificate must force the transformation it names.** Assert the
   admission actually tripped *because of the prefix*, with the control that
   has none.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host, warm tree, immediately before dispatch:
twenty-two checker tests green in about 26 s; `cargo test --workspace` green
in about 97 s with no failures. The tree is green at your starting point; any
red is yours.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

**Measurement — this brief touches admission, so the gate applies.**
Allocation counts first, via `scripts/core-memory-microbench.sh`; this host's
cores are 1.27x apart and one timing pair cannot gate the panel. If the
counters do not move, do not quote a time. If they do, run
`scripts/core-language-microbench.sh` base and candidate at the same binary
path, four passes each, profiling env off, normalized against named untouched
labels — never the suite median. **Report the unarmed delta against both your
own base and the line's cumulative unarmed baseline of
+0.965% / +3.239% / +1.018%.** A bar is a data point, not a stop condition.

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment; the director settles them. **Do not** set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

After each milestone write `target/brief-538/PROGRESS.md` with what is done
and verified, the exact next action, the tree's state — does it compile, which
tests are green, which files are half-edited — and anything the director needs
to take over. Prefer to leave the tree compiling at each checkpoint.

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config.
- Respect the fence; outside it, the disclosure duty applies.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- No `#[ignore]`, no feature gate on a required test, no test weakened to get
  green.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Scratch goes under `target/brief-538/`.
- **Stop-loss, keyed on progress.** Report two numbers per attempt:
  falsifiers watched red-then-green, and required gates passing. Stop only when
  **two consecutive** attempts move neither. A compile error you are actively
  reducing is not a wall.

## Report format (your final message)

1. What changed — per file with line ranges, including every unnamed file,
   with why.
2. The protocol: types, operations, and how each lifetime exit is closed, with
   the `path:line` of each.
3. The bridge: how the prefix becomes visible before guest code, and the
   activation falsifier red then green against its control.
4. The reentry set: where it lives, how it is charged or exempted, and the
   `ListConsumer` falsifier.
5. Verification — every command with verbatim output.
6. Measurement — allocation counts first; timing only if counters moved; the
   unarmed delta against both bases.
7. Which of B03, B09 and B12 are now dischargeable, and what remains pending.
8. Stop-loss / open questions.
