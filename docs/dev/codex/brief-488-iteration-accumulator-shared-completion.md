# brief-488 — a resumed iteration completes instead of panicking

The repair for the defect diagnosed in brief-487. Implementation brief; the
diagnosis is settled and is not to be re-derived.

## What brief-487 established — take these as given

- **The reproducer**, smaller than the one the diagnosis brief supplied:

  ```rust
  let mut session = session_with_test_control_builtins();
  session.eval_request("Table[InputString[], 2]").unwrap_err();
  session.resume_eval_request_with_input("0").unwrap_err();
  session.resume_eval_request_with_input("1").unwrap(); // kernel panic
  ```

- **The retaining holder**, named and evidenced:
  `Session.resuming_suspension` → `SuspendedEvaluation.frames.front()` →
  `ResumeSuspendedTableIterationsSuspensionFrame.state.collected.tail`. The
  driver legitimately roots the frame while the continuation runs, so the
  accumulator's tail is genuinely shared at completion. **The sharing is not a
  leak and not an accident.**
- **The failing condition** is completion while the rooted frame already holds
  a nonempty linked tail. One suspension switches to the persistent
  accumulator but does not by itself trigger it; two suspensions inside the
  same first element do not either.
- **`Sum` and `Product` share the defect** through the same accumulator and
  completion logic (`iteration_aggregate.rs:199`, frame state cloned at
  `:324`). **`Do` does not** — `SuspendedDoIterations` has no accumulator.
- **The embedder reaches it**: through `HostSession`, the run returns
  `HostFailureCode::Panic`, the next evaluation returns `Poisoned`, and
  `reset()` recovers. **This corrects brief-487's framing, which the director
  had written**: the host's `catch_unwind` (`rum-host/src/session.rs:661`)
  does classify the panic, so panic isolation works. The defect is therefore
  not "an unclassifiable crash" but **an ordinary valid program poisoning its
  session instead of completing** — which is a defect of the same weight and a
  more accurate description.

## Goal — repair option 1, as recommended and accepted

Make conversion support shared nodes: move the payload out of a uniquely owned
node, **clone it out of a shared one**, and never panic. The uniqueness
requirement is an incorrect precondition, not an invariant worth defending.

The argument that decides it, and which the brief keeps: the accumulator's own
frozen prefix already behaves exactly this way — `SharedArgs::into_vec`
(`rum-core/src/expr.rs:647`) moves when unique and clones when shared — so the
linked suffix demanding uniqueness makes one struct internally inconsistent
with itself. This rests on ownership correctness, not on cost.

Options 2-4 are settled and closed: releasing the holder before completion
breaks root visibility unless deferred through `PureFrameFinalizer`, and an
`EvalError` conversion leaves a valid program failing, which is a stopgap and
was rejected as the recommendation. Do not reopen them. If implementation
shows option 1 cannot work, **STOP and report** rather than substituting one.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs`
  — `PersistentExprAccumulator::into_vec` and, if needed, the node.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs`
  — the aggregate completion at `:199` if it needs the same treatment; say so
  if it does not.
- The test files where the certificates live:
  `crates/rum-runtime/src/runtime/tests/iterative_loop_drivers.rs` and
  `crates/rum-host/src/tests/eval_contract.rs`.
- `crates/rum-host/src/session.rs` — **rustdoc only**, if a sentence about
  resume behavior becomes false. No behavior change in `rum-host`.

MUST NOT touch: `crates/rum-runtime/src/runtime/accumulator_chain.rs` and the
`rum-derive` accessor derive (brief-485, finished); the suspension driver and
frame protocol; `rum-core` beyond reading `SharedArgs::into_vec`;
`host_payload.rs`; the ownership scanner; `SECURITY.md`, `AGENTS.md`,
`PROGRESS.md`, `TODO.md`, other `docs/**`, `docs/dev/codex/**` except your own
report; `.git`, git config, `Cargo.lock`.

**Note on brief-486.** Its conversion of this same type is parked in
`stash@{0}` and is **not** in the tree. You will not see `AccumulatorChain`
here; do not add it. Its instruction to preserve the uniqueness panic is
obsolete as of this brief — that is the director's to fix, not yours.

## The certificate — a genuine before/after pair, which is rare here

1. **Kernel-level**, in `iterative_loop_drivers.rs`: the reproducer above,
   asserting the **exact** collected result, not merely that it did not panic.
   A `Table` that returns the wrong list would pass a "did not panic" test.
2. **The `Sum`/`Product` siblings**, if they share the defect: the same shape,
   asserting the exact numeric result.
3. **Host-level**, in `eval_contract.rs`: the same program through
   `HostSession::eval` and `resume_with_input`, asserting a **successful**
   outcome and that the session is not poisoned afterwards.
4. Paste each one's **failure on the pre-repair tree** and **success after**.
   This is one of the rare cases where a true before/after pair exists — use
   it, and paste both halves verbatim.

Also assert the behavior that the panic was protecting, if any: that the
collected values are correct when the tail *is* uniquely owned, i.e. the
ordinary non-suspended path is unchanged.

## Verification (MANDATORY)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

Both bounds come from this session's full-workspace gate runs on this host,
which completed well inside them.

**No microbench A/B is required, and you must verify the premise rather than
assume it**: an unsuspended `Table` freezes its values into the prefix and
leaves the tail empty, so `into_vec`'s loop does not run at all on the
ordinary path, and the new branch is reached only after a suspension. Confirm
that from source. If you find the loop running on an unsuspended completion,
STOP and report — the A/B then becomes required.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run any `git stash`
  operation. `stash@{0}` holds parked work; leave the stash list untouched.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **Do not write a new crash or panic probe.** The supplied reproducer and its
  variants are the certificate.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No default-path cost.** The unsuspended completion path must be unchanged.
- Preserve the suspend/resume continuation shape; do not collapse a
  `suspend_with_frame*` into an eager call.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27). No stopgap; an
  `EvalError` conversion is not a repair.
- Minimal diff: no drive-by reformatting, renames, or unrelated cleanup.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Each certificate's pre-repair failure and post-repair success, verbatim.
3. `cargo fmt --check` and the workspace total, verbatim.
4. The A/B premise: what you confirmed from source about the unsuspended path.
5. Whether `Sum`/`Product` needed the same change, with evidence either way.
6. Stop-loss / open questions.
