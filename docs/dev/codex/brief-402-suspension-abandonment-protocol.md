# brief-402 — implement: the closed suspension-abandonment protocol

Shape: **implementation**. It lands the protocol only; it migrates no caller,
so the public leak brief-401 demonstrated must still reproduce unchanged when
you are done. That is a gate item, not an oversight.

## Where this came from

brief-401 established, on the committed tree and through public `rum-host`
API alone, that discarding a live suspension drops its saved continuation
frames without unwinding them. Every scope those frames own survives into the
next request: `Block` bindings, `Module` activations and gensym reclamation,
loop iterator scopes, `OptionValue` scopes, open `Reap`/`Monitor` scopes,
time/memory constraint nesting, trace and capture checkpoints, `AbortProtect`
depth, `BlockRandom` PRNG state, and partial package installs. The leaked
`AbortProtect` depth lets evaluation continue past `Abort[]`; the leaked
constraint deadline terminates an unrelated later evaluation.

brief-401 also refuted the repair brief-400 assumed. Driving the saved frames
with `Err(EvalError::Cancelled)` is **not** an unwind: ordinary frames consume
errors and run work. `EvaluationData` converts cancellation into a successful
association (`debug_tools/evaluation_data.rs:67-125`), `Check` may evaluate its
fallback (`kernel_data/control_data/check_quiet.rs:91-129`), `Monitor` refreshes
its display (`debug_tools/monitor.rs:80-84`). Abandonment therefore needs its
own closed protocol that never calls `SuspensionFrame::resume`.

## The design decision I changed from the plan draft

brief-401's draft asked for a mandatory no-default abandonment method on
`SuspensionFrame`. The tree has **377 `impl SuspensionFrame for` blocks across
115 files** (74 in `rum-runtime`, 41 in `rum-stdlib`) — I counted before
writing this. Roughly all of them are pure computation frames whose correct
answer is "discard". 377 handwritten identical methods would bury the ~15
frames that actually matter in noise, and review could not tell a considered
`Discard` from a copy-paste one.

Do it as a **supertrait plus an explicit derive** instead:

- `SuspensionFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static`.
- `FrameAbandon` has one required method and **no blanket impl and no default
  body**. A frame that classifies neither way does not compile.
- A `#[derive(PureFrameAbandon)]` (name yours, but it must read as a claim,
  not as boilerplate) generates the discarding impl. It is re-exported
  alongside `FrameRootTrace` from `runtime/kernel_api.rs:114` so `rum-stdlib`
  reaches it the same way it reaches the existing derive; `rum-stdlib` has no
  direct `rum-derive` dependency and must not gain one.

This keeps the property compiler-enforced — including across the crate edge,
which a source scan cannot reach (`SuspensionFrame` is `pub`, if
`#[doc(hidden)]`) — while making the stateful inventory a one-line grep: the
frames that do *not* derive the pure marker. That grep is what a reviewer
reads, so the derive must be a claim someone can be wrong about.

Everything else in brief-401 §4 stands, including the `WithCleanup` cleanup
allowlist. I am keeping that in this brief rather than deferring it: the
allowlisted action, the driver worklist, and the repository allowlist gate are
all driver machinery, and a later brief that reopens the closed action enum
and the driver would rework exactly what this one freezes.

## Goal

Land a runtime-private, type-directed abandonment protocol that structurally
restores every frame-owned kernel state without resuming ordinary continuation
work and without touching host input. Every one of the 377 frames declares its
behavior; the declaration is enforced by the compiler, not by a convention or
a source scan. No production caller is migrated in this brief, so no public
behavior changes.

## Context pointers

- Read `AGENTS.md`, `ARCHITECTURE.md`, `docs/testing-strategy.md`,
  `docs/runtime-evaluation-audit.md`, `docs/host-embedding-and-hardening.md`,
  and **brief-401's report sections 3 and 4** (`docs/dev/codex/brief-401-*.md`)
  before touching anything. Section 3 is your inventory checklist; section 4 is
  the contract you are implementing.
- Protocol core: `crates/rum-runtime/src/runtime/suspension/core.rs` —
  `SuspensionFrame` at `:17`, `FrameAction` at `:33`, `SuspendedEvaluation` at
  `:10-13`, the driver at `:169-220`.
- Trait siblings and re-export chain: `runtime/suspension/trace.rs:19`,
  `runtime/suspension.rs:11-12`, `runtime/kernel_api.rs:106,112,114`.
- Derive crate: `crates/rum-derive/src/lib.rs:45` and its
  `crates/rum-derive/tests/derive.rs`.
- Ownership: `runtime/model/session.rs:108-111` (the broadly visible
  `pending_suspension: Option<…>` that makes today's raw discards compile),
  `runtime/session/session_state.rs`.
- Existing enforcement precedent to extend, not replace:
  `runtime/tests/frame_root_trace.rs` — `manual_frame_root_trace_implementations_match_sorted_allowlist`
  at `:311`, `suspension_protocol_repository_invariants_hold` at `:337`, and
  `runtime/tests/frame_root_trace/derived_inventory.rs`.
- Stateful families, each with its normal cleanup cited in brief-401 §3:
  `scoping/lexical/block.rs` and `block/resume_frames.rs`,
  `scoping/lexical/module.rs`,
  `kernel_data/control_data/branching_loops/iteration/driver.rs` and its
  `driver/resume_frames.rs`, `branching_loops/iteration/mod.rs`,
  `options/patterns.rs`, `debug_tools/reap.rs`, `debug_tools/monitor.rs`,
  `debug_tools/trace.rs`, `debug_tools/evaluation_data.rs`,
  `debug_tools/resources/constraints.rs`,
  `debug_tools/test_report/evaluation.rs`,
  `kernel_data/control_data/check_quiet.rs`,
  `kernel_data/control_data/cleanup.rs`,
  `kernel_data/control_data/abort_cleanup.rs` and its `resume_frames.rs`,
  `scoping/packages.rs` and `scoping/packages/resume_frames.rs`,
  `session/session_api.rs` (PRNG frame at `:575-591`) and
  `session/session_api/resume_frames.rs`.
- **Your sandbox has no network** — brief-401's run hit
  `Could not resolve host: mirrors.ustc.edu.cn` on its first `cargo`
  invocation. Pass `--offline` to every `cargo` command from the start.
- Acceptance is mechanical: the named tests, the pasted compiler errors, and
  the unchanged-leak probe decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/suspension/core.rs`,
    `runtime/suspension/trace.rs`, `runtime/suspension.rs`,
    `runtime/kernel_api.rs`;
  - `crates/rum-derive/src/lib.rs` and `crates/rum-derive/tests/derive.rs`;
  - `crates/rum-runtime/src/runtime/model/session.rs` and
    `runtime/session/session_state.rs`, for the private suspension-slot owner;
  - **any existing `.rs` file under `crates/rum-runtime/src/runtime/` or
    `crates/rum-stdlib/src/` that already contains `impl SuspensionFrame for`
    or declares a type used by one** — and in such a file only to add the
    abandonment classification, to add or widen a structural restore helper
    that the classification calls, and nothing else;
  - `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs`,
    `runtime/tests/frame_root_trace/derived_inventory.rs`, and focused new or
    existing tests under `crates/rum-runtime/src/runtime/tests/`;
  - a throwaway external-crate proof under `/tmp` (not in the repo).
- Files/areas you MUST NOT touch:
  - `crates/rum-core` public representation, any public serde shape,
    `Cargo.lock`, workspace `Cargo.toml` dependency edges (in particular do not
    add `rum-derive` to `rum-stdlib`);
  - the four production raw-discard callers — `runtime/builtins_input.rs`,
    `runtime/debug_tools/monitor.rs:112-148`,
    `runtime/session/session_helpers.rs:135-162`, and the request boundary in
    `runtime/session/session_requests.rs`. **They are brief-403.** Legacy raw
    clear APIs may stay in place and keep compiling; just do not route them
    through the new protocol and do not delete them yet.
  - ordinary evaluator, dispatch, or matcher logic; `rum-host`; `rum-jupyter`;
    public docs; the generated builtin reference;
  - the brief-400 stash and every unrelated pre-existing worktree edit
    (`.claude/hooks/guard-dispatch.py`, four files under `scripts/`, and the
    untracked `scripts/peak-rss.py`). Leave them exactly as you found them.
- If the work needs a file outside this list, **STOP and report** rather than
  working around it. My MAY lists have been short six times on this project and
  every one of those stop-losses was the right call.

## Deliverable

1. A closed `AbandonReason` enum distinguishing at least: cancelled request,
   helper best-effort discard, checkpoint-rollback discard, and superseded or
   queued-input discard. It is diagnostic/policy data; no caller supplies host
   input through it.
2. A `FrameAbandon` supertrait with one required method taking a restricted
   `AbandonmentContext` — **not** `&mut Session` — and returning a closed
   `FrameAbandonAction`. The context exposes only structural restore, pop, and
   truncate operations. It exposes no `Session::evaluate`, no input queue, no
   input conversion, and no general closure callback.
3. The `PureFrameAbandon` derive, re-exported through `kernel_api`, and applied
   to every pure-computation frame. Every stateful frame writes its impl by
   hand and calls the exact restoration primitive its normal cleanup uses.
4. Structural restoration covering **every** row of brief-401 §3's first table.
   Do not stop at `Block`; the report showed a `Block`-only patch leaves the
   other owners broken.
5. `WithCleanup` as the sole cleanup-evaluating action:
   `FrameAbandonAction::EvaluateCleanup`, constructible only from the
   `WithCleanup` frame family and backed by a repository allowlist gate. The
   driver evaluates that expression under an abandonment guard and then resumes
   **abandonment**, never the ordinary continuation; its result never replaces
   the latched reason. If cleanup itself suspends, take the newly created
   pending owner onto a LIFO abandonment worklist and abandon it too — never
   expose its request, never source an input value.
6. An iterative (not recursive) abandonment driver preserving front-to-back
   inner-to-outer order, including the two-frame payload protocols `Block` and
   the loop iterators use.
7. A private, non-constructible suspension-slot owner replacing raw
   `Option<SuspendedEvaluation>` access, with `take_for_resume` /
   `take_for_abandon` and a monotonically increasing suspension generation.
   Whole-`Session` drop or reset stays legal — all contaminated state dies with
   it.
8. `clear_suspension_state` renamed to a completed-resumption operation that
   asserts its frame queue is already empty, so it can never double-unwind
   after `drive_resuming_frames`.
9. Enforcement, extending `frame_root_trace.rs`:
   - the repository invariant lists every frame carrying the pure derive and
     **fails if any of the stateful families named in Context pointers appears
     in that list** — this catches someone silently marking a stateful frame
     pure, which the compiler cannot catch;
   - it fails if a non-allowlisted frame can construct `EvaluateCleanup`.
10. Direct tests proving, per stateful family, that the saved session field is
    restored, and proving that abandonment runs **no** later continuation, no
    `Monitor` refresh, no `Reap` render, no `Check` fallback, no constraint
    fallback, no trace render, and no `EvaluationData` association.

## Two pieces of evidence I specifically want

- **The compile error.** Delete the pure derive from one arbitrary frame,
  build, and paste the verbatim `error[E0277]` (or whatever it is). Then
  restore it. This is the whole point of the design; I want to see it fire.
- **The crate edge.** Build a throwaway crate under `/tmp` that depends on
  `rum-runtime` and implements `SuspensionFrame` for a local type without the
  classification. Paste the compiler error. brief-389 taught this project that
  a boundary can hold inside the repository and evaporate at the crate edge,
  because a source scan cannot say anything about code that does not exist
  yet. Remove the crate afterwards; nothing lands in the repo.

## rum / Rust pitfalls (read twice)

1. Never call `SuspensionFrame::resume` with a synthetic error while
   abandoning. That is the refuted repair; frames consume errors and run work.
2. Preserve inner-to-outer order. `Block` and the loop iterators use two-frame
   payload protocols; abandoning them out of order restores the wrong depth.
3. The restricted context must not expose `evaluate`, input queues, or a
   general callback. If a frame needs something the context lacks, widen the
   context deliberately and say so in the report — do not reach for `Session`.
4. `Module` abandonment must finish the activation with an error/no-result root
   set so unescaped locals are reclaimed and activation depth truncates. It
   must not run remaining initializers or the body.
5. Package abandonment runs a **non-reporting** rollback: the host discarded
   the work, so do not emit an ordinary package-failure message.
6. Restore exactly once. A frame whose restoration also runs on the normal
   error path must not run twice when both fire.
7. Whole-session drop/reset stays legal; only live-session owner discard is
   forbidden.
8. No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
9. No ordinary-path cost: no allocation, lock, or branch added to
   `evaluate_once`, dispatch, the matcher, `resume_or_suspend!`'s
   non-suspending arm, or `Expr` construction/drop. A vtable method is free; a
   check in the evaluator loop is not.
10. `size_of::<Arc<DynSuspensionFrame>>()` feeds the suspension memory
    accounting at `suspension/core.rs:114`. If your change moves it, say so.

## Verification (MANDATORY — you run it; iterate to green)

Pass `--offline` to every `cargo` command; your sandbox has no network.

```
scripts/run-bounded.py --timeout 300  -- cargo fmt --check
scripts/run-bounded.py --timeout 900  -- cargo test --offline -p rum-runtime suspension_protocol_repository_invariants_hold -- --nocapture
scripts/run-bounded.py --timeout 1500 -- cargo test --offline -p rum-runtime
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace
```

Paste real counts and the verbatim invariant inventory rows.

Then the two enforcement proofs above, each with verbatim compiler output.

Then the **unchanged-leak check**. This brief migrates no caller, so
brief-401's public probe must still leak exactly as it did. Re-run its `Block`
case through public `rum-host` API — a `Block` binding installed before an
`InputString`, the suspension discarded, then a later public eval reading the
symbol — and paste the result. It must still read the inside value. If the
leak has changed or disappeared, you have touched a caller and escaped this
brief's scope: **STOP and report that**.

No timing A/B is required: every path here is cold suspension/error/helper
code. If you find yourself editing ordinary eval, dispatch, or matcher code,
the brief's cost premise is exceeded — stop and report instead of measuring.

## Hard rules

- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave changes in the working tree. **Do not touch the brief-400 stash**, and
  preserve every unrelated pre-existing worktree edit.
- Modify only Scope. If you must go outside it, STOP and write that in the
  report. Do not work around a missing entry.
- Touch nothing outside this repo except the `/tmp` throwaway crate this brief
  grants; remove it when done. No network. No probe artifacts left in the repo.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No vendor engine, oracle harness, compatibility matrix, or
  third-party output.
- **No new `unsafe`.** If you are convinced it is unavoidable, STOP and report.
- Minimal diff. The 370-odd pure classifications should be one added derive
  each — no drive-by reformatting, renames, or comment rewrites.
- Use `scripts/run-bounded.py` for anything long or child-spawning, and paste
  genuine verbatim output. If a run did not reach green, say so — never fake it.

## Report format

1. Per-file change with line ranges and a one-line why; the classification
   counts (pure via derive vs handwritten stateful) and the total against 377.
2. The stateful inventory: each family, the exact restoration primitive its
   abandonment calls, and the normal-cleanup site it mirrors.
3. Verification output, verbatim, including both enforcement proofs and the
   unchanged-leak probe.
4. Ordinary-path cost statement.
5. Stop-loss, risks, open questions — empty if none.
