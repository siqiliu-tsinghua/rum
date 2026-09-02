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

---

# Report — 2026-09-02

## 1. Changes and classification counts

Key changes:

| Files and lines | Change |
|---|---|
| [rum-derive/src/lib.rs](crates/rum-derive/src/lib.rs:53), [derive tests](crates/rum-derive/tests/derive.rs:1) | Added and tested `PureFrameAbandon`, preserving generics and where clauses and rejecting unions. |
| [suspension/core.rs](crates/rum-runtime/src/runtime/suspension/core.rs:11) | Added `AbandonReason`, restricted `AbandonmentContext`, closed `FrameAbandonAction`, mandatory `FrameAbandon`, generation tracking, private slot access, completed-resumption assertion, and iterative LIFO abandonment driver. |
| [suspension.rs](crates/rum-runtime/src/runtime/suspension.rs:11), [kernel_api.rs](crates/rum-runtime/src/runtime/kernel_api.rs:106) | Re-exported protocol types and derive across the runtime/stdlib boundary. |
| [model/session.rs](crates/rum-runtime/src/runtime/model/session.rs:108), [session_state.rs](crates/rum-runtime/src/runtime/session/session_state.rs:660) | Added generation and iterative-abandonment worklist state. |
| [block.rs](crates/rum-runtime/src/runtime/scoping/lexical/block.rs:401), [block frames](crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:22), [module.rs](crates/rum-runtime/src/runtime/scoping/lexical/module.rs:306), [loop driver](crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:424) | Added structural scope-restoration helpers and handwritten frame abandonment. |
| [evaluation_data.rs](crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:242), [monitor.rs](crates/rum-runtime/src/runtime/debug_tools/monitor.rs:165), [reap.rs](crates/rum-runtime/src/runtime/debug_tools/reap.rs:398), [constraints.rs](crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:356), [trace frames](crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:34), [test-report capture](crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:530) | Added capture/scope restoration without rendering, refreshing, fallbacks, or result synthesis. |
| [check_quiet.rs](crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:197), [abort frames](crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:59), [cleanup.rs](crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:170) | Added checkpoint restoration and the two-frame `WithCleanup` allowlist. |
| [options/patterns.rs](crates/rum-runtime/src/runtime/options/patterns.rs:185), [iteration/mod.rs](crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:171), [random frame](crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:26), [package rollback](crates/rum-runtime/src/runtime/scoping/packages.rs:476) | Restored option scope, product budget, PRNG state, and partial package state. |
| [frame_root_trace.rs](crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:68), [derived inventory](crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs:930) | Added exact pure/stateful inventories and closed-action/driver allowlist gates. |
| [suspension_abandonment.rs](crates/rum-runtime/src/runtime/tests/suspension_abandonment.rs:1), [tests/mod.rs](crates/rum-runtime/src/runtime/tests/mod.rs:178) | Added 16 direct tests covering every stateful family and forbidden continuation effects. |

The remaining changes across 120 runtime/stdlib frame-declaration files are mechanical `PureFrameAbandon` derives; stdlib files import the derive’s three protocol names through `kernel_api`. The exact 516 path/type rows are frozen in [derived_inventory.rs](crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs:930).

Counts:

- Pure via derive: **516**
- Handwritten stateful: **29**
- Total: **545**

The brief’s prose says 377, but the current tree—and brief-401’s own pre-task invariant output—contains 545. The implementation classified the full current inventory: `516 + 29 = 545`.

## 2. Stateful restoration inventory

| Family | Abandonment primitive | Normal cleanup mirrored |
|---|---|---|
| `Block` | `restore_single_block_scope` / `restore_block_scope` | `finish_single_symbol_block_scope` and `finish_block_scope` in [block.rs](crates/rum-runtime/src/runtime/scoping/lexical/block.rs:401) |
| `Module` | `finish_module_activation` → `finish_module_scope(depth, Err(Cancelled))` | [finish_module_scope](crates/rum-runtime/src/runtime/scoping/lexical/module.rs:306) |
| Loop iterators | Full symbol restore or dynamic-scope truncate/clear | [loop cleanup](crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:424) |
| `OptionValue` | Pop `option_value_scopes` | Normal pops in [patterns.rs](crates/rum-runtime/src/runtime/options/patterns.rs:11) |
| `Reap` | Pop/discard `reap_scopes` | [resume_reap_after_body](crates/rum-runtime/src/runtime/debug_tools/reap.rs:84) |
| `Monitor` | Pop `monitor_scopes`, no refresh | [finish_monitor](crates/rum-runtime/src/runtime/debug_tools/monitor.rs:80) |
| Constraints | Restore prior deadline/limit | [constraint finishers](crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:225) |
| Trace | `abandon_trace_capture` | Error restoration in [trace.rs](crates/rum-runtime/src/runtime/debug_tools/trace.rs:209) |
| `EvaluationData` | Restore tracing; truncate trace/message/print/error suffixes | [finish_evaluation_data_capture](crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:56) |
| `Check` / `Quiet` | Restore message and handled-error checkpoints | [check/quiet finishers](crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:91) |
| `TestReport` | Restore stream-capture checkpoints | [finish_eval_expr_with_stream_capture](crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:279) |
| `AbortProtect` / `CheckAbort` | Restore depth and exact pending-abort checkpoint | [abort cleanup](crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:271) |
| Iteration product | Restore parent `(budget, consumer)` pair | [with_iteration_product_scope](crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:12) |
| `BlockRandom` | Restore saved `PrngState` | [evaluate_with_random_state_restored](crates/rum-runtime/src/runtime/session/session_api.rs:575) |
| Package install | Non-reporting `rollback_package_install` | Normal failure now calls the same rollback in [packages.rs](crates/rum-runtime/src/runtime/scoping/packages.rs:476) |
| `WithCleanup` | Allowlisted `EvaluateCleanup` action | [resume_with_cleanup_after_primary](crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:80) |

## 3. Verification

All mandatory commands exited 0.

```text
scripts/run-bounded.py --timeout 300 -- cargo fmt --check
```

No output.

```text
scripts/run-bounded.py --timeout 1500 -- cargo test --offline -p rum-runtime
```

```text
test result: ok. 919 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.17s
```

```text
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace
```

Exited 0. Notable verbatim summaries include:

```text
test result: ok. 919 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
test result: ok. 828 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The only warnings were the three pre-existing unused test-builtin helpers.

Invariant output:

```text
named_suspension_frames=545
named_pure_finalizers=60
pure_frame_abandon_derives=516
handwritten_stateful_frame_abandons=29
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:EvalEvaluationDataSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/monitor.rs:EvalMonitorSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/reap.rs:EvalReapSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeMemoryConstrainedWithLimitSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeTimeConstrainedWithLimitSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:EvalExprWithStreamCaptureSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:EvalStructuredTraceCaptureExprSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:EvalAbortProtectSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortWithPropagationSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:WithIterationProductScopeSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalCheckSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalQuietSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:DriveWithCleanupPrimarySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:EvalWithCleanupSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/options/patterns.rs:EvaluateWithOptionValueScopeSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:EvalSingleSymbolBlockSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockInitializersSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeSingleSymbolBlockBodySuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame1
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame2
stateful_frame_abandon=crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:ResumePackageInstallSuspensionFrame
stateful_frame_abandon=crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:EvaluateWithRandomStateRestoredSuspensionFrame
evaluate_cleanup_allowlist=WithCleanup owner frames (2)
legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 920 filtered out; finished in 0.43s
```

Repository compile-enforcement proof after temporarily removing the derive from `EvalUpdateAttributesSuspensionFrame`, then restoring it:

```text
error[E0277]: the trait bound `EvalUpdateAttributesSuspensionFrame: suspension::core::FrameAbandon` is not satisfied
   --> crates/rum-runtime/src/runtime/attributes.rs:234:54
    |
234 | impl crate::runtime::suspension::SuspensionFrame for EvalUpdateAttributesSuspensionFrame {
    |                                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `suspension::core::FrameAbandon` is not implemented for `EvalUpdateAttributesSuspensionFrame`
    |
note: required by a bound in `suspension::core::SuspensionFrame`
   --> crates/rum-runtime/src/runtime/suspension/core.rs:68:45
    |
68  | pub trait SuspensionFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static {
    |                                             ^^^^^^^^^^^^ required by this bound in `SuspensionFrame`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `rum-runtime` (lib) due to 1 previous error
```

External `/tmp` crate-edge proof:

```text
error[E0277]: the trait bound `ExternalFrame: FrameAbandon` is not satisfied
  --> src/main.rs:14:26
   |
14 | impl SuspensionFrame for ExternalFrame {
   |                          ^^^^^^^^^^^^^ the trait `FrameAbandon` is not implemented for `ExternalFrame`
   |
note: required by a bound in `SuspensionFrame`
  --> crates/rum-runtime/src/runtime/suspension/core.rs:68:45
   |
68 | pub trait SuspensionFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static {
   |                                             ^^^^^^^^^^^^ required by this bound in `SuspensionFrame`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `rum-brief402-edge` (bin "rum-brief402-edge") due to 1 previous error
```

Unchanged public `rum-host` leak probe:

```text
queue.block.setup=Success(value="outside")
queue.block.suspend=InputRequired(prompt="old")
queue.block.drain=Success(value="drain")
queue.block.after=Success(value="inside")
```

Both `/tmp` crates were removed afterward.

## 4. Ordinary-path cost

No ordinary evaluator, dispatcher, matcher, expression construction, or drop path gained an allocation, lock, or branch. Pure classifications are compile-time-only; generation work occurs only when an input suspension is installed; the abandonment driver is unreachable from production until brief-403.

`size_of::<Arc<DynSuspensionFrame>>()` remains the suspension accounting unit at [core.rs:175](crates/rum-runtime/src/runtime/suspension/core.rs:175). No timing A/B was required by the brief.

## 5. Stop-loss, risks, and open questions

No stop-loss triggered and no new functional question remains. The legacy raw-clear API and broadly visible pending storage deliberately remain so the forbidden brief-403 callers continue compiling and leaking; new resume/abandon ownership uses the private slot capability.

No `unsafe`, dependency-edge, manifest, lockfile, host, Jupyter, public-doc, or production caller change was made. The pre-existing `.claude`/`scripts` worktree changes were preserved, and no commit, branch, stash, checkout, or git-config operation was performed.

---

## Director's correction to this brief's prose

The brief says 377 frames across 115 files. That was my undercount, not the
worker's: `git grep -c "SuspensionFrame for"` misses every impl header that
wraps as

```rust
impl crate::runtime::suspension::SuspensionFrame
    for EvalFooSuspensionFrame
```

A cross-line regex over the same committed tree returns 546, and the enforced
invariant reports 545 named frames. The design argument the brief makes is
unaffected — it gets stronger with the larger number — but the figure was
wrong and the worker was right to say so against the enforced inventory.
Standing rule, restated: prefer a count some test enforces over one a grep
produces, and cross-check any script-derived count before relying on it.
