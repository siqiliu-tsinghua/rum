# brief-617 — G13 step S9: type the control and cleanup family

## Goal

Type the seventh production family through the typed API. S9 owns the frames
that suspend inside rum's control constructs:
- `CheckAbort` (its `PropagateAborts` value, its body, its fallback) and
  `AbortProtect`;
- `Assert` (`$AssertFunction`, the test, a user assertion function);
- `Check` and `Quiet`;
- `WithCleanup` (init, body, cleanup) and the finalizer that combines the
  body's and the cleanup's outcomes;
- `CatchExceptions` (the body, a handler's pattern, its guard, its
  right-hand side);
- `Catch` (the body, the tag form, a form pattern's guard, and the same two
  for a throw that bypassed a catch through `WithCleanup`).

When you are done:

- the owned rows implement the typed API, and the owned macro sites use the
  witnessed typed macro or the token, as their call path requires;
- the owned transport encoding T0029 is gone, and the frame completes its
  typed helper result directly;
- every other family still runs through the transport it uses today;
- the ordinary path of every program executes no more work than at S8, and
  the cost of S9's resume path is measured and attributed.

This is step S9 of G13 (design round 5, brief-606, ratified). S1-S8 (briefs
607-611 and 614-616) and G14 (612-613) are done; S8 is `99ceab48`.
S10-S54 type the remaining families, F1-F23 flip transport per type, and R
removes the legacy machinery.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (its D1/D2 and report) with
  `.temp/brief-606/design.md` and `census-spec.md`; `.temp/brief-605/design.md`
  §§2-5, **except §5's transit record, which D2 rejected**.
- **S3-S8 as built are your template.** Read brief-614, brief-615 and
  brief-616 in full, with their director amendments and gate records. Their
  rulings are now the rules:
  - **Two arms on the typed macro.** `resume_typed_or_suspend!` is defined in
    `suspension/typed/witness.rs`. Use the caller-place arm (`$result:ident`)
    wherever the producer's result is already bound to a local. Its third
    arm (`evaluation_return =>`) is for G14's two fixed-point sites only.
  - **The macro expansion is fixed.** Do not change `witness.rs`.
  - **The witness and the request cross a cold call, never the producer's
    `Result`.** Take the ZST witness at the producer, then pass
    `(witness, request)` and the frame's frozen state to a named
    `#[cold] #[inline(never)]` helper. A shape that needs an `unreachable!`
    arm for a state the types allow is the wrong shape.
  - **Start from S7's and S8's final shape.** Bind the producer `Result`,
    take the witness, `match` once, return errors directly, and call the
    cold helper only on `InputRequired`. A wrapper closure around the
    completion cost S7 an extra 136-B copy on every ordinary call.
  - **The routes** (S8's table in brief-616's report is the newest example):
    - a site reached only from the ordinary path uses the witnessed route;
    - a site reached both from the ordinary path and from its own frame's
      resume uses `DefinitionResumeRoute`
      (`definitions/application/mod.rs`, `pub(in crate::runtime)`). Its
      ordinary instantiation is `()` and is unchanged; the resumed
      instantiation consumes the token with fresh state;
    - a resume that inserts a *different* frame type uses the witnessed
      route and then stops with `ResumeStop`.

    S8 also shows two valid shapes for a loop that a frame re-enters: one
    generic driver over `DefinitionResumeRoute` (`drive_count_pattern_matches`),
    or a separate resumed driver that takes only the token
    (`drive_suspended_position_traversal`) when the code already has two
    drivers. Keep the structure the family has; do not merge or split
    drivers.
  - **The token** re-inserts fresh state of the resuming frame's own type,
    and it carries no `Arc`.
  - **The finalizer erasure.** A typed finalizer is erased as its own box.
    Deferred work returns `Result<T, ResumeStop<O>>`, and `finish_work`
    selects the finalizer.
  - **The retention analyzer reads the typed macro** through
    `retention_closure/typed_resume_macro.rs`.
  - **G14:** typed frames get their evaluation context at the common
    insertion endpoints. A family has nothing to do for it.
  - **Owned identity decodes.** A `from_continuation_expr` on the frame's own
    input disappears with the frame, and its receipt row moves from R to
    this step, as S8's ten `Whole` rows did.
- **This family resumes on errors.** In S3-S8 a frame's input was nearly
  always a success. Here most frames continue on an *error*: `CheckAbort`'s
  fallback runs on `Err(Abort)` or `Err(ProtectedAbort)`, `Catch` on
  `Err(Thrown)` and `Err(ThrownBypassingCatch)`, `CatchExceptions` on
  `Err(ExceptionThrown)`, `WithCleanup`'s cleanup on any outcome. Today's
  ordinary shape is `match outcome { Err(InputRequired) => propagate, outcome
  => continue(outcome) }`. **Every non-`InputRequired` error must reach the
  same continuation on the ordinary path and on the resumed path**, and the
  typed `resume` passes its `Result<Input, EvalError>` on unchanged.
- **State this family keeps outside its frames.** Keep each of these exactly,
  on both paths and in the same order:
  - `abort_protect_depth` is incremented before `AbortProtect`'s body and
    decremented in `resume_abort_protect_after_body`. The `CheckAbort` and
    `AbortProtect` frames freeze an `abort_protect_pending` checkpoint.
  - `Check` and `Quiet` freeze `messages.len()` and
    `handled_non_loader_eval_errors.len()` before their body. Messages stay
    in the session across a suspension and are split from the request's own
    checkpoint only when the request completes
    (`session/session_requests.rs`), so the frozen integers remain valid on
    resume. Keep them plain integers.
- **The cleanup finalizer's uniqueness.** `FinishWithCleanupFinalizer::finish`
  (`cleanup.rs:231`) unwraps the primary outcome's `Arc` and panics if it is
  shared. brief-593 (its row 12) ruled this sound: the `Arc` is fresh, and
  its frame (F0317) retires before the finalizer runs. The typed driver
  preserves that order today: `suspension/core.rs` drops the resumed frame
  and pops its queue entry before it calls `finish`. **No adapter, token or
  helper may keep F0317 or its state alive into `finish`.** If typing would
  break this, STOP and report; do not replace the panic with a clone. The
  fixtures in which a cleanup pauses are the falsifier.
- **Abandonment.** F0315 and F0316 implement `FrameAbandon` with
  `evaluate_cleanup`: an abandoned `WithCleanup` still runs its cleanup.
  The typed adapter forwards `abandon` to the frame
  (`typed/adapter.rs:27`). Keep both impls as they are.
- **This family chains.** `CatchExceptions` has two dispatchers: the
  ordinary `dispatch_exception_handlers_owned` (owned `Vec`s, frames F0334
  and F0335) and the resumed `dispatch_suspended_exception_handlers` (shared
  `Arc` state, frames F0336 and F0337), which loops back into itself after a
  declined match or guard. `Catch`'s tag-form frames (F0340, F0342) lead to
  its form-match frames (F0341, F0343). `WithCleanup`'s init frame (F0315)
  leads to its body frame (F0316), which leads to its cleanup frame (F0317)
  and the finalizer (F0572). **Classify every call path of every site under
  the route rules, and report the table, as S6-S8 did.** A site that fits no
  rule is a STOP with its call paths.
- **Rows, as input; never edit them.** All of these are in `.temp/brief-606/`:
  - `all-frame-migrations.tsv`: F0283-F0290, F0313-F0317, F0333-F0343, F0572;
  - `macro-dataflow.tsv` and `macro-frame-ownership.tsv`: M0293-M0300,
    M0323-M0327, M0344-M0354;
  - `transformation-dispositions.tsv`: T0029;
  - `microbench-step-labels.tsv`: the S9 labels;
  - `draft-S9.md`: Codex's plan for this step.

  S9 has no chained-insert rows, insertion-route rows, deferred
  obligations, State births, promoted divergences or witness gates. The
  draft is raw material, and where it and this brief differ, this brief
  governs. Its Scope, scratch path, file list and measurement section are
  superseded.
- **Slot types.** `Expr` is `Value`. `CompiledMatchOutcome` is registered
  Legacy (`suspension/typed/registry.rs`, `legacy.rs`) and keeps its single
  global mode until its F step. Register no new slot type.

### The director's re-probe, at `99ceab48`

61 shapes were run through the resumed path against the queued-input path
(`.temp/director/s9-probe/`, `pairs.py` and `run1.txt`). **All 61 are SAME,
and every one paused**, between 1 and 4 times.

- **`CheckAbort` and `AbortProtect`:** a pause in the body, in the fallback
  after an abort, in both, in the `PropagateAborts -> value` (a `:>` value
  is not evaluated; it gives `CheckAbort::opttfa`), with `PropagateAborts ->
  True` inside and outside an `AbortProtect`, and in a body that aborts
  inside `AbortProtect`. The fallback after a *pending protected* abort
  (F0286) is reached by `AbortProtect[CheckAbort[CheckAbort[Abort[], 4,
  PropagateAborts -> True], (InputString["f"]; 9)]]`: the inner
  `CheckAbort` raises the pending count, so the outer body succeeds and its
  fallback still runs.
- **`Assert`:** a pause in `$AssertFunction`, in the test with `On[Assert]`
  (true and false), and in a user assertion function.
- **`Check` and `Quiet`:** a pause in the body with and without a message,
  with a message before the pause, after it, and before the construct
  (`{1}[[3]]` emits `Part::partw`; rum keeps `1/0` inert, so it cannot drive
  them), with a message selector, and with a `Throw` through `Check`.
- **`WithCleanup`:** a pause in the init, the body and the cleanup, with
  the body throwing and aborting, and with the body and the cleanup throwing
  the same tag (which makes the throw bypass the enclosing `Catch`).
- **`Catch`:** a pause in the body, the tag form, a form pattern's guard
  (matching and declining), the handler, and the tag form and guard of a
  bypassing throw.
- **`CatchExceptions`:** a pause in the body, in an ordinary dispatch's
  guard and pattern (`p_?(…&)`), the same after a body pause (the resumed
  dispatcher), a declining first handler, a handler's right-hand side, and
  an ancestor match through `RegisterExceptionType`.
- **Loops and nesting:** `Table` over each, and the constructs nested in one
  another.

Like S6-S8, the typing must keep all 61 equal. This step has no defect to
repair.

### The owned rows, at `99ceab48`

Frames (all under `crates/rum-runtime/src/runtime/kernel_data/control_data/`):

| Row | Type | File:line (impl) |
| --- | --- | --- |
| F0283 | `EvalCheckAbortSuspensionFrame` | `abort_cleanup/resume_frames.rs:9` |
| F0284 | `ResumeCheckAbortWithPropagationSuspensionFrame` | same `:38` |
| F0285 | `EvalAbortProtectSuspensionFrame` | same `:74` |
| F0286 | `ResumeCheckAbortAfterPrimarySuspensionFrame1` | same `:104` |
| F0287 | `ResumeCheckAbortAfterPrimarySuspensionFrame2` | same `:139` |
| F0288 | `EvalAssertSuspensionFrame` | `assertions.rs:162` |
| F0289 | `ResumeAssertAfterFunctionSuspensionFrame` | same `:187` |
| F0290 | `FinishAssertFunctionSuspensionFrame` | same `:208` |
| F0313 | `EvalCheckSuspensionFrame` | `check_quiet.rs:179` |
| F0314 | `EvalQuietSuspensionFrame` | same `:228` |
| F0315 | `EvalWithCleanupSuspensionFrame` | `cleanup.rs:155` |
| F0316 | `DriveWithCleanupPrimarySuspensionFrame` | same `:184` |
| F0317 | `ResumeWithCleanupAfterPrimarySuspensionFrame` | same `:213` |
| F0572 | `FinishWithCleanupFinalizer` (finalizer) | same `:231` |
| F0333 | `EvalCatchExceptionsSuspensionFrame` | `exceptions/resume_frames.rs:11` |
| F0334 | `DispatchExceptionHandlersOwnedSuspensionFrame1` | same `:31` |
| F0335 | `DispatchExceptionHandlersOwnedSuspensionFrame2` | same `:52` |
| F0336 | `DispatchSuspendedExceptionHandlersSuspensionFrame` | same `:73` |
| F0337 | `ResumeSuspendedExceptionMatchOutcomeSuspensionFrame` | same `:94` |
| F0338 | `EvaluateExceptionHandlerRhsSuspensionFrame` | same `:112` |
| F0339 | `EvalCatchSuspensionFrame` | `nonlocal.rs:357` |
| F0340 | `ResumeCatchForThrowSuspensionFrame` | same `:380` |
| F0341 | `FinishCatchForThrowWithFormSuspensionFrame` | same `:412` |
| F0342 | `ResumeCatchForBypassingThrowSuspensionFrame` | same `:458` |
| F0343 | `FinishCatchForBypassingThrowWithFormSuspensionFrame` | same `:504` |

Macro sites (same directory):

| Site | File:line | What pauses |
| --- | --- | --- |
| M0293 | `abort_cleanup/options.rs:56` | `CheckAbort`'s `PropagateAborts` value |
| M0294 | `abort_cleanup.rs:127` | `CheckAbort` body |
| M0295 | same `:156` | `AbortProtect` body |
| M0296 | same `:179` | `CheckAbort` fallback, after a pending protected abort |
| M0297 | same `:202` | `CheckAbort` fallback, after an abort |
| M0298 | `assertions.rs:9` | `$AssertFunction` |
| M0299 | same `:85` | the assertion's test |
| M0300 | same `:116` | a user assertion function, result form |
| M0323 | `check_quiet.rs:27` | `Check` body |
| M0324 | same `:70` | `Quiet` body |
| M0325 | `cleanup.rs:20` | `WithCleanup` init |
| M0326 | same `:64` | `WithCleanup` body |
| M0327 | same `:90` | `WithCleanup` cleanup, insert form |
| M0344 | `exceptions.rs:181` | `CatchExceptions` body |
| M0345 | same `:323` | ordinary dispatch: a handler guard, insert form |
| M0346 | same `:338` | ordinary dispatch: a handler pattern, insert form |
| M0347 | same `:373` | resumed dispatch: a handler pattern, insert form |
| M0348 | same `:415` | resumed dispatch: a handler guard, insert form |
| M0349 | same `:456` | a handler's right-hand side, result form |
| M0350 | `nonlocal.rs:96` | `Catch` body |
| M0351 | same `:168` | the tag form |
| M0352 | same `:204` | the form pattern's match, result form |
| M0353 | same `:241` | the tag form, bypassing throw |
| M0354 | same `:295` | the form pattern's match, bypassing throw, result form |

The transport transform **deletes only the transport encoding**; the frame
completes its typed helper's exact result:

| Row | Site | Today |
| --- | --- | --- |
| T0029 | `nonlocal.rs:440` (F0341) | `.map(into_continuation_expr)` after `finish_catch_match` |

Derive each `Input` from the actual producer `Result` before any
transformation, and bind each `Output` at the innermost function or closure
return. The identity decodes (`from_continuation_expr` on the frame's own
input, for example `CompiledMatchOutcome::from_continuation_expr(outcome?)`
in `exceptions.rs` and `nonlocal.rs`) and the identity re-encodes
(`Expr::into_continuation_expr`) disappear. A semantic step, such as
`finish_catch_match` or `finish_with_cleanup_outcomes`, runs identically on
both paths.

## Scope

A listed file or class is permission to do this step's owned work there.
It is not permission to:
- convert another step's rows;
- change evaluation beyond the owned rows;
- tidy.

Disclose every file you edit.

**Classes.**

- **The family's source files**, under
  `crates/rum-runtime/src/runtime/kernel_data/control_data/`:
  `abort_cleanup.rs`, `abort_cleanup/options.rs`,
  `abort_cleanup/resume_frames.rs`, `assertions.rs`, `check_quiet.rs`,
  `cleanup.rs`, `exceptions.rs`, `exceptions/resume_frames.rs` and
  `nonlocal.rs`.
- **`control_data/mod.rs`**, only for a visibility or import edit that a
  typed frame's state needs. Disclose it.
- **Callers of a changed signature.** Edit only the construct that changed,
  in any file where an owned frame, helper or signature is named, including
  `kernel_api.rs` re-exports.
- **Module roots on an owned type's path.** Make only the visibility or
  re-export edit the typed API needs, and disclose it.
- **`crates/rum-runtime/src/runtime/suspension/`**, only where S9's use
  exposes a gap in the API or driver. Disclose each edit and why.
  `typed/witness.rs` is excluded. Sealing, opacity and the no-dual-arm rule
  stay intact, and so does the finalizer order above.
- **Sibling unit tests.** A test needing a family module's private items
  goes in that module's sibling `tests.rs`, declared as `#[cfg(test)] mod
  tests;`. Never use `#[path = "../…"]`.
- **Census, oracle and controls.**
  - **Census:** everything under `crates/rum-stdlib/tests/typed_slot_census/`
    and `typed_slot_census.rs`.
  - **Oracle:** `crates/rum-stdlib/tests/suspension_oracle/fixtures/` for the
    new fixtures and the `typed_slots.rs` receipt, and
    `suspension_oracle/inventory/typed-slot-stages.tsv` and
    `typed-slot-shapes.tsv`.
  - **Compile controls:** `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`,
    only for a pin that follows a derived count. No new pair is required.
- **Derived facts follow the diff** (brief-613's class rule). For every
  derived inventory, frontier, census receipt, fingerprint, certificate or
  test pin that names a touched construct, each row falls into exactly one
  case:

  | Its operation in this diff | Ruling |
  | --- | --- |
  | Deleted by an owned conversion (including the transport encoding) | Retired with its source. Disclose it. |
  | Added by this diff | A new row. It stays rejected or unknown; claim no mark or coverage. Disclose it. |
  | Moved, or its text follows an owned rename | Paired. Disclose it old → new. |

  The class covers:
  - `runtime/tests/frame_root_trace{.rs,/derived_inventory.rs}`;
  - ownership inventories, by discovery replay;
  - retention inventories, by replay;
  - `retention_closure/census/tests.rs`, with a per-brief comment;
  - census receipts;
  - count and layout pins.

  A **STOP** is a change of *meaning* where the source did not change:
  - a row whose operation's source text is unchanged disappears, or changes
    kind, reason, status, disposition or exclusion;
  - a mark, coverage or promotion is gained or lost;
  - a new `UnknownHostPayload`;
  - a production receipt change outside S9's owned rows.

  Never hand-write a fingerprint, certificate cell or ledger chain.
- **Canonical tool outputs.** Any unchanged canonical gate tool this brief
  requires may write to its own fixed output paths under `target/` or
  `.temp/`. You may not modify, redirect or wrap it to move its output.
  Disclose each such path. Your own scratch stays in `.temp/brief-617/`.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md` and the core `Expr`/`Symbol` representation stay unchanged.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

## Steps

1. **Harness, probe and bases first, before any source edit.**
   - **Supplied.** `.temp/brief-617/` holds:
     - `base/rum-cli` and `base/rum-cli-alloc`: the S8 one-codegen-unit
       binaries, with their build commands. The director proved them fresh.
     - `stack-base/probe.rs`: the stack probe with seventeen shapes, the
       ten of S8 and seven new ones in which the recursion runs inside a
       body of this family, so that its ordinary-path functions stay
       resident on the native stack:
       - `catch-body`: `k[n_] := Catch[k[n - 1]] + 1`;
       - `checkabort-body`: `ca[n_] := CheckAbort[ca[n - 1], 0] + 1`;
       - `abortprotect-body`: `ap[n_] := AbortProtect[ap[n - 1]] + 1`;
       - `cleanup-body`: `wc[n_] := WithCleanup[wc[n - 1], Null] + 1`;
       - `check-body`: `ch[n_] := Check[ch[n - 1], 0] + 1`;
       - `quiet-body`: `q[n_] := Quiet[q[n - 1]] + 1`;
       - `catchexceptions-body`: `ce[n_] := CatchExceptions[ce[n - 1]] + 1`.
         The runtime-only test session has no `CatchExceptions` entry, so the
         probe registers the kernel's own handler (`eval_catch_exceptions`)
         with the stdlib's `HoldAll` for this shape only. Without it the
         shape returns `Plus[1, CatchExceptions[…]]` and measures nothing;
         the probe's result assertion is what caught that.

       Each shape returns its depth. The other six control builtins reach
       their real `eval_*` methods through the test session's control
       dispatcher.
     - `base-source/`: the `99ceab48` export.
     - `base-hashes.txt`.

     Verify the hashes first.
   - **Stack bases.** Build the debug and release `runtime-tests` bases from
     the unedited tree plus the supplied probe, with
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its own target
     directory under `.temp/brief-617/`. **All seventeen shapes must
     reproduce the director's bytes per level exactly. If one does not,
     STOP.**

     | Profile | peel | plus | sub | up | guarded | rhs-guard | replace-guard | replaceall-guard | count-guard | position-guard |
     | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
     | debug | 19 024 | 34 912 | 29 376 | 28 064 | 35 840 | 49 152 | 64 880 | 63 296 | 74 064 | 85 648 |
     | release | 7 344 | 13 792 | 13 168 | 11 456 | 14 240 | 17 920 | 26 768 | 24 640 | 28 416 | 33 632 |

     | Profile | catch-body | checkabort-body | abortprotect-body | cleanup-body | check-body | quiet-body | catchexceptions-body |
     | --- | --- | --- | --- | --- | --- | --- | --- |
     | debug | 50 880 | 53 504 | 50 192 | 50 944 | 50 752 | 50 624 | 50 240 |
     | release | 19 344 | 21 312 | 19 344 | 19 344 | 18 880 | 18 800 | 19 600 |

     The ten S8 shapes equal S8's candidate. The director measured all
     seventeen on `99ceab48` at four heights with three samples each, every
     fit with zero residual (`.temp/brief-617/calib/`).
   - **Resume harness.** Copy `.temp/brief-616/resume-harness/` to
     `.temp/brief-617/resume-harness/`. Add workloads that pause in S9's
     frames:
     - `CheckAbort` pausing in its `PropagateAborts` value (F0283), its body
       (F0284), and its fallback after an abort (F0287);
     - `CheckAbort`'s fallback after a pending protected abort (F0286),
       through `AbortProtect[CheckAbort[CheckAbort[Abort[], 4,
       PropagateAborts -> True], (InputString["f"]; 9)]]` (the director's
       probe reaches it; confirm it with the frame census);
     - `AbortProtect` pausing in its body (F0285);
     - `Assert` pausing in `$AssertFunction` (F0288), in its test with
       asserts on (F0289), and in a user assertion function (F0290);
     - `Check` and `Quiet` pausing in their body (F0313, F0314);
     - `WithCleanup` pausing in its init (F0315), its body (F0316), and its
       cleanup (F0317 and the finalizer F0572);
     - `CatchExceptions` pausing in its body (F0333), in an ordinary
       dispatch's handler guard (F0334) and handler pattern (F0335), in a
       resumed dispatch's pattern (F0336) and guard (F0337), and in a
       handler's right-hand side (F0338);
     - `Catch` pausing in its body (F0339), its tag form (F0340), a form
       pattern's guard (F0341), and the same two for a throw that bypasses
       it through `WithCleanup` (F0342, F0343).

     Each repeats its pauses N times and checks its final value. Then build
     `resume` and `resume-alloc` from the unedited tree, with
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its own target
     directory under `.temp/brief-617/`. Record their source hashes; they
     must match `base-source/`. Each workload's callgrind frame-call census
     must show the intended S9 frames resuming the intended number of times
     per pause. A frame body the compiler inlined into an adapter is counted
     at its address, as S8 did for F0240, F0241 and F0558.
2. **Labels.** Copy `.temp/brief-616/perf/labels.json` and its programs into
   `.temp/brief-617/perf/`. Add these ordinary labels, in the same N/2N
   format, each checking its own result:
   - `catch-no-throw`: `Catch[1]`;
   - `catch-throw`: `Catch[Throw[1]]`;
   - `catch-tagged-throw`: `Catch[Throw[1, t], t]`;
   - `catch-form-pattern`: `Catch[Throw[1, t[2]], t[_]]`;
   - `checkabort-no-abort`: `CheckAbort[1, 2]`;
   - `checkabort-abort`: `CheckAbort[Abort[], 2]`;
   - `abortprotect-small`: `AbortProtect[1]`;
   - `withcleanup-small`: `WithCleanup[1, Null]`;
   - `withcleanup-init`: `WithCleanup[Null, 1, Null]`;
   - `check-no-message`: `Check[1, 2]`;
   - `quiet-no-message`: `Quiet[1]`;
   - `catchexceptions-handler`: `CatchExceptions[ThrowException["Exception",
     <||>], {Exception["Exception", _] :> 1}]`;
   - `catchexceptions-no-throw`: `CatchExceptions[1]`;
   - `assert-off`: `Assert[True]`, whose result is `Assert[True]` with
     asserts off.

   Each of these returns `1` except `checkabort-abort` (`2`) and
   `assert-off`. Add the new resume workloads as resume labels, each with
   its `resumes_per_iteration`.
3. **Fixtures.** Before converting anything, add S9's oracle fixtures and
   run them on the unedited tree. Each compares the queued-input path with
   the resumed path under `Checks::ALL`, with unconditional `FaultProbe`
   absence.
   - Cover the director's shapes and the harness shapes.
   - Each passes on the unedited tree. Record that.
4. **Type the owned rows and delete the transport encoding**, following the
   rulings above.
   - Keep exactly:
     - every error's continuation, on both paths;
     - the abort-protect depth and pending checkpoints, and the message and
       handled-error checkpoints;
     - the cleanup order: `WithCleanup`'s cleanup runs after the body on
       every outcome, an abandoned body still runs its cleanup, and the
       finalizer combines the outcomes after F0317 has retired;
     - `Catch`'s exact-tag fast path (`exact_catch_tag_match`) and the
       thrown-versus-bypassing distinction;
     - `CatchExceptions`' handler order and ancestor walk;
     - held and evaluation order;
     - owner drop order;
     - profiling balance, including `Catch`'s runtime path timings.
   - Each M site's ordinary path matches the producer `Result` once and
     constructs no frame, `Arc` or `Vec`. Frame state is built in the cold
     path. Move locals, not control flow.
5. **Ordinary controls.** Rerun the 28 round-4 ordinary controls through
   brief-611's harness:
   - copy `.temp/brief-616/ordinary-controls/` to `.temp/brief-617/`;
   - re-sync its copied `suspension_oracle/` support from the committed tree
     first;
   - all 28 must pass.
6. **Census.** Advance the receipt to S9 in `typed_slot_census` and in
   `suspension_oracle/fixtures/typed_slots.rs`, and record every owned row's
   transition.
   - Production at S8 is 606 implementations and 558 macro sites, with 70
     typed.
   - With the twenty-four frames and the finalizer typed, expect 606
     implementations with 95 typed. Report the actual totals, including
     the macro sites and the finalizer count.
7. **Inventories.** Replay fingerprints, both certificate sets and ownership
   discovery, and disclose every changed row under the derived-facts class.

## Verification (MANDATORY — run to green or evidence)

**How to run.**
- Run everything serially, each command through `python3
  scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
  RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
  TMPDIR=<repo>/.temp/brief-617/tmp …`.
- Bounds: workspace 3 600 s; oracle 1 500 s per build; everything else
  900 s.
- **Your shell tool's own timeout must exceed the command's bound.**

Run these in the repository tree:

```
cargo fmt --check
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-runtime --lib frame_root_trace
cargo test -p rum-runtime --lib resume_evaluation_context
cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
cargo test -p rum-stdlib --test typed_slot_compile_controls
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls
cargo test -p rum-stdlib --test typed_slot_census
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
cargo test --workspace --no-fail-fast
```

- Also run the ordinary-controls harness in both builds.
- Rerun the director's S9 probe (`.temp/director/s9-probe/pairs.py`, through
  a copy of `.temp/director/depth-leak-probe/` built under
  `.temp/brief-617/`). Also rerun S8's 48 shapes, S7's 32 and S6's 22
  (`.temp/director/{s8,s7,s6}-probe/pairs.py`) as a regression check.
- **Native ZeroMQ.** In your sandbox the nine native ZeroMQ tests cannot
  bind loopback. Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`,
  report the nine as skips, and report every other count. The director runs
  them natively.

Report:
- every `test result:` total;
- the oracle's fault-occurrence delta for each build;
- any compiler warning the diff introduces. Neither oracle build may warn.

**Performance.** `TEMPLATE.md` pitfall 8 governs, with the bars of
brief-611 and 613-616. Build the candidates with the base commands and the
same cwd, into separate target directories. Preflight every N/2N run: the
result, and the exact resume count (`resumes_per_iteration` × N).

1. **Allocations and bytes first**, on every label.
   - Ordinary labels must be exactly equal, in slope and in the
     whole-program counts at N and 2N.
   - On the resume workloads, a typed entry adds one 24-B adapter `Arc` per
     newly typed transfer. Any other difference is attributed at its
     allocation sites, or it is a defect.
2. **Instruction slopes.** Run two interleaved `perf stat` pairs per arm on
   all labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`,
   stdin `/dev/null`.
   - A flagged ordinary label is cleared by five runs per arm, or by six or
     more callgrind processes per arm with every deterministic delta named
     at its addresses.
   - A deterministic register-allocation residual is reported with its
     addresses for the director to rule on, as S6's +2 and S7's +5 were.
3. **Copies and leaves.** The `memcpy` count on every ordinary label is at
   most the base's. `literal-integer`, `do-count-null` and
   `do-iterator-null` show no deterministic instruction growth. `eval_catch`,
   the `CheckAbort` option path through `resume_check_abort_with_propagation`,
   `drive_with_cleanup_primary` and `dispatch_exception_handlers_owned` are
   on the ordinary path of every call of their builtin; report each one's
   own instructions per call, base and candidate.
4. **Stack.** Measure all seventeen shapes, debug and release, at four
   heights with three samples each, against your calibrated bases.
   - Tabulate the prologue frame size of every function S9 changes, in both
     profiles.
   - **Bar:** no growth beyond the typed macro's named closure environment,
     attributed per slot.
   - **Any growth on any of the seventeen shapes is reported with its slot
     table for the director to rule on.**
5. **Resume attribution.** For every resume workload, run six callgrind
   processes per arm and diff every function per process. Name each
   deterministic delta at its addresses. The driver's 136-B typed-outcome
   move is owed to R: count it, do not repair it.
6. **Timing** comes last, as an indication only.
   Run four full
   `scripts/core-language-microbench.sh` panels, base, candidate, base,
   candidate, under `taskset -c 2`, with profiling off.
   - A residual whose instructions are equal is a layout question. Settle
     it causally with both arms rebuilt using `RUSTFLAGS="-C
     llvm-args=-align-all-functions=6"`.
   - **Probe with the panel's own program, whole.** Take the generated
     `core-language-microbench.rum` from one panel's output directory,
     raise `rumMicroReps` from 2 to 25 and change nothing else. Run it ten
     times per arm, arms alternated, under `taskset -c 2` with stdin
     `/dev/null`, and compare per-label medians and ranges. Do not cut labels
     out of it: a cut can drop a label's prelude, and S5's cut hit a fast
     path 250x cheaper than the label.
   - Calibrate before reading a delta: each label's per-iteration time in
     this probe must be within a small factor of the panel's (S8's director
     run: 0.78-1.25x, median 0.99). A label far below its panel value hit
     the 100 000 iteration work limit and measured startup.
   - A label is settled when, on the aligned builds, its candidate range is
     not disjoint above the base's. S8's worst label read +21.8% on the
     ordinary builds and +1.6% with overlapping ranges on the aligned ones.

A regression is a defect to fix or report. A stack or performance residual
is not a failing gate; it goes to the report with its attribution, and I
rule on it.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- No default-path cost. `Catch`, `Check`, `Quiet` and `WithCleanup` wrap
  library code and user loops; `CheckAbort` reads an option on every call.
- **The witness comes from the actual `Result`.** Never guess the input from
  a synthetic `Err` type.
- **An error is an outcome here, not a failure to propagate.** Do not let
  `?` or an early `return Err` skip a continuation that today runs on that
  error.
- **The token** carries no `Arc`, is not `Clone`, and accepts fresh state of
  the same frame type only.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry.
- Profiling records stay balanced on every path.
- Prefer the caller-place macro arm.
- **Module size.** New modules stay under 10k characters. Check
  `.temp/brief-606/file-size-assessments.tsv` first. `nonlocal.rs` and
  `exceptions.rs` are the largest files here. Split only by responsibility.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-617/`.
- After each step, write `PROGRESS.md` and a machine-readable `state.json`.
  Update `state.json` when the work moves, not only at the end.
- Record:
  - the verified state;
  - the compile and test counters;
  - any half-edited files;
  - the exact next action.
- **Write the report as you go.** Append each report section to
  `.temp/brief-617/REPORT.md` as soon as its evidence is final, and copy the
  whole file into this brief at the end. brief-616's run was cut by a usage
  limit after its last gate and before its report; only the sections already
  on disk survived.

## Hard rules

- **No `git`, in any form.** No network.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands allowed:
  - scoped reads and edits, `rg`/`grep`, bounded `python3`;
  - the bounded Cargo commands above and bounded compiler fixtures;
  - `scripts/check-suspension-oracle.sh` and
    `scripts/core-language-microbench.sh`;
  - `valgrind`, `callgrind_annotate`, `objdump`, `nm`, `perf` and `taskset`
    on the measurement binaries.

  No probe that aborts a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. Type no row outside S9's. Never weaken an oracle `Checks` flag, a
  retention reject, a fault-absence gate or a documented test to get a pass.
- **Stop-loss is keyed on progress.** Compile errors you are reducing count
  as progress. Stop after two attempts that move none of the named failing
  gates.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. Disclose every ripple,
   public signature change, module root and pin update as old → new, under
   its derived-facts case.
2. Owned-row reconciliation:
   - each F/M/T row's transition, and each owned identity decode retired;
   - the call-path and route table;
   - the census output.
3. The harness workloads, stack shapes and oracle fixtures: what each
   exercises, its frame census, and its result on the base tree and the
   candidate in both builds. Include the director's shapes.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions. Every canonical tool output path written.
5. Allocation, instruction, copy, stack and resume-attribution evidence, and
   timing with any alignment check.
6. Stop-loss counters, deviations and open questions.

## Director continuation 1 (2026-10-17) — run 1 was cut; you take it over

Run 1 was done by another worker, and its quota cut it at 11:25 after 2 h 20
min. It left its work uncommitted in the tree, and it left **no report**:
`.temp/brief-617/state.json` and `PROGRESS.md` stop at 09:31 (step 1, base
builds), and `REPORT.md` is empty. I reconstructed what it did from its log.
You take the brief over from that state. Everything above still governs, and
this section adds to it. **Its results are not evidence for the final tree.**
Verify what you keep, and rerun every gate in Verification yourself.

**What run 1 left, step by step.**

- **Step 1**, under `.temp/brief-617/`:
  - the hash check (`hash-verification.json`);
  - the stack bases (`target-base-stack-{debug,release}`) and their exact
    17-shape calibration (`state.json`);
  - the resume harness with S9 workloads (`resume-harness/src/s9.rs`,
    `s9-resume-workloads.json`);
  - `base-resume` and `base-resume-alloc`, rebuilt three times as the
    workloads changed (the last hashes are in `state.json`);
  - a base census (`base-resume-census/`, `base-resume-census.py`) and
    performance preparation (`perf/`, `prepare-perf.py`, `run-perf.py`).

  Rebuild any base whose source or harness no longer matches what it was
  built from, and rerun any calibration whose evidence is not on disk.
- **Step 3.** New fixtures in
  `crates/rum-stdlib/tests/suspension_oracle/fixtures/control_cleanup/`. Its
  log shows 88 `control_cleanup` tests passing in both builds before its
  first source edit. It then added `director_nonlocal.rs` and edited
  `director.rs` after the source edits, so **the final fixture set has never
  run on the unedited tree.** Run it there:
  - copy `.temp/brief-617/base-source/` to a scratch directory;
  - add the final fixtures and only the registration they need;
  - run them in both builds.

  Every fixture must pass. Report the counts.
- **Step 4.** All nine family files are converted. **Review every owned row
  against the rulings above before you build on it**: the routes, the
  cold-helper shape, every error's continuation, the state kept outside the
  frames, the finalizer's uniqueness, abandonment, T0029 and the identity
  decodes. Fix what does not hold, within Scope.
- **Step 5** was never run.
- **Step 6.** Its census printed `S9 TOTAL typed=95 typed_macros=55
  witnessed_sites=46`. It also extended the `S4` filter in
  `typed_slot_census/s4.rs` with `S9`, as S8 extended it with `S8`.
- **Step 7.**
  - The local-contract fingerprints were regenerated with the ignored
    `regenerate_local_contract_fingerprints` test. Only build-input columns
    changed, in 68 rows of `local-contracts.tsv` and 2 rows of
    `promoted.tsv`.
  - The ownership inventories (`unknown_attributes.tsv`,
    `unknown_captures.tsv`, `unknown_macros.tsv`) were edited by a script.
    It applied `production_inventory_is_exact`'s own missing/stale report:
    91 rows added and 113 removed.
  - The test is exact now, but **classify every one of those rows** under
    the derived-facts class and disclose it, as S8's report did for its
    discovery replay. A row outside S9's touched files, or a removed row
    whose source text did not change, is a STOP.

**Red or never run.**

- **RED:** `frame_root_trace::suspension_protocol_repository_invariants_hold`
  fails with "PureFrameFinalizer FinishWithCleanupFinalizer must use the
  closed Session-free signature in …/cleanup.rs".
  - Cause: run 1 moved F0572 to `SlotFinalizer`. The invariant
    (`runtime/tests/frame_root_trace.rs`, near line 340) names each typed
    `SlotFinalizer` with its `Output` and its `finish` signature.
  - S4 and S6 added theirs there, and the file is in the derived-facts
    class. Add F0572's entry the same way (`type Output = Expr;`, `fn
    finish(self: Box<Self>) -> Result<Expr, EvalError>`), keep the check
    exact, and disclose it.
- **Never run:**
  - step 5's ordinary controls;
  - `cargo test --workspace --no-fail-fast`;
  - the director probes (s9, s8, s7, s6);
  - the whole performance section, items 1-6.
- **Warnings.** The `rum-runtime` lib-test build prints three `dead_code`
  warnings in `runtime/test_builtins.rs`: `register_test_if_builtin`,
  `register_test_return_builtin` and `register_test_vm_template_builtins`.
  Build `base-source/` and report whether they occur at `99ceab48`. If the
  diff introduced them, fix it.

**Checkpointing is a hard rule this time.** Run 1 lost its whole record this
way. Start by rewriting `state.json` and `PROGRESS.md` to the state this
section describes. After that:
- update them whenever the work moves, at least after every step and every
  gate;
- append each report section to `REPORT.md` as soon as its evidence is
  final.

**Report.** Add a section 0, *Run 1 review*, before section 1: what you kept
from run 1's diff, what you changed, and why. The rest of the report format
is unchanged.

---

# brief-617 Report

## 0. Run 1 review

Run 1 was cut at 11:25. Its checkpoint (`state.json`, `PROGRESS.md`) stopped at step 1, 09:31. `REPORT.md` was empty. The working tree already held the family conversion, the S9 census receipt, the oracle fixtures, the ownership-inventory replay, and the local-contract fingerprint refresh. Those edits are kept only where this review rechecked them. Every Verification gate is rerun on the tree this run leaves.

### Kept

- The control/cleanup family is typed through the witnessed API. Owned frames F0283–F0290, F0313–F0317, F0333–F0343, and finalizer F0572 implement `TypedFrame` or `SlotFinalizer`. Owned macro sites use the caller-place `resume_typed_or_suspend!` arm or a `#[cold] #[inline(never)]` helper that receives `(witness, request)`.
- Transport T0029 (the `.map(into_continuation_expr)` after `finish_catch_match` on F0341) is gone. The identity `from_continuation_expr` decodes and the identity `Expr::into_continuation_expr` re-encodes on this family's frames are gone. Sibling families still use their own continuation codecs.
- `FinishWithCleanupFinalizer::finish` still `Arc::try_unwrap`s the primary outcome and panics if the `Arc` is shared. The resume path clones that `Arc` into the finalizer while the frame is alive; the typed driver drops the frame before `finish`.
- F0315 and F0316 `FrameAbandon` with `evaluate_cleanup` stay. Check and Quiet keep `output_boundary` and `FrameAbandon`. AbortProtect still increments `abort_protect_depth` before the body and decrements it in `resume_abort_protect_after_body`. CheckAbort and AbortProtect still freeze `abort_protect_pending`. Check and Quiet still freeze `messages.len()` and `handled_non_loader_eval_errors.len()` as plain integers.
- CatchExceptions keeps two dispatchers: ordinary `dispatch_exception_handlers_owned` and resumed `dispatch_suspended_exception_handlers`. The resumed dispatcher is generic over a local `ExceptionResumeRoute` (`()` or `ContinueToken`, both returning `Result<Expr, ResumeStop<Expr>>`) so one function can suspend F0336 or F0337. That is the deviation from `DefinitionResumeRoute`: one driver, two frame types, and the error type is `ResumeStop` rather than `EvalError`.
- Census receipt pins and the S4 later-stage filter's `S9` arm stay. Oracle fixtures under `fixtures/control_cleanup/` stay. Ownership inventory line moves inside S9 files stay, corrected where rustfmt shifted `exceptions/resume_frames.rs` by −2. Local-contract fingerprint column refresh stays.
- Step 1 measurement artifacts under `.temp/brief-617/` stay as base evidence for the unedited tree. They are not candidate evidence.

### Changed in this run

- `cleanup.rs`. An attempt to evaluate the `WithCleanup` primary outside `drive_with_cleanup_primary` (so the recursive stack matched S8's `eval_with_cleanup` frame) failed the census proof: `slot_witness` is accepted only when the immediately previous statement is the producer local. The evaluate is back inside `drive_with_cleanup_primary`. Run 1's tree passed the census. The M0326 failure (`witnessed typed insertion absent`) was introduced by this run's review edit and removed by reverting it. `evaluate(cleanup.clone())` stays, so the ordinary path still clones the cleanup expression. The extra `drive` frame on `cleanup-body` is a stack measurement, reported when measured. `rustfmt` then collapsed `drive_with_cleanup_primary`'s signature by four lines. Ownership derive, closure, and macro rows in `cleanup.rs`, and the WithCleanup obligation citations (`:65`/`:70` → `:61`/`:66`), moved with that shift. The cited lines are `resume_with_cleanup_after_primary` and `evaluate(cleanup.clone())`.
- `nonlocal.rs`. `record_profile_runtime_path_timing` for `catch.body.evaluate` stood between `let body_result` and `let witness`, and the census died at `M0350: witnessed typed insertion absent`. The witness is now the next statement after `body_result`. Run 1's tree passed the census. The M0350 failure was introduced by this run's review edit, which left the profile record between `body_result` and `slot_witness`, and was removed by reverting that order. The profile record follows it, so the timed region includes the zero-sized witness. Start/stop pairs for `catch.eval.total`, `catch.body.evaluate`, and `catch.resume_after_body` stay balanced.
- `frame_root_trace.rs`. Run 1's RED was `PureFrameFinalizer FinishWithCleanupFinalizer must use the closed Session-free signature`. The invariant now accepts `FinishWithCleanupFinalizer` with `type Output = Expr` and `fn finish(self: Box<Self>) -> Result<Expr, EvalError>`, the same shape S4 and S6 used. The test `suspension_protocol_repository_invariants_hold` passed (15/15) after that edit. This file is in the derived-facts class; the change is the added disjunct only.
- `exceptions/resume_frames.rs`. `rustfmt` collapsed one import. No semantic change. Ownership derive and closure rows for that file moved −2 lines.
- Ownership inventories. Line-number paired moves only, all inside S9 family files (`abort_cleanup`, `assertions`, `check_quiet`, `cleanup`, `exceptions`, `exceptions/resume_frames`, `nonlocal`, `options`). `resume_or_suspend!` rows retired with the transport macros and were replaced by `resume_typed_or_suspend!` or dropped where the site became a cold helper. `|value|` capture rows retired with the identity re-encodes (assertions, nonlocal T0029, nonlocal F0343). No row outside those files. Attribute descriptions: no unique text added or removed.
- `retention_closure/inventory/obligations.tsv`. WithCleanup evidence citations moved from base `cleanup.rs:80` and `:85` to `resume_with_cleanup_after_primary` and `evaluate(cleanup.clone())`. After the signature collapse those lines are `:61` and `:66`. Paired citation move.
- `local-contracts.tsv` and `promoted.tsv`. Fingerprint and certificate hash columns changed because body fingerprints hash a global signature context. `options/core.rs` (C01/C02) was not edited. Status, exclusions, strength, preconditions, source span, and `direct-covered={}` stayed. Disclosed as a paired build-input refresh. Hashes were not hand-written.

### Census (both feature sets)

`cargo test -p rum-stdlib --test typed_slot_census` (stdout captured by Cargo on success): `test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 274.77s`. Log: `.temp/brief-617/logs/census-default.log`.

`cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --test-threads=1 --nocapture`: `test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 275.61s`. Log: `.temp/brief-617/logs/census-no-default.log`. The ignored test is `forced_red_probe`.

Verbatim production lines from the no-default run:

```
S9 CENSUS production frames=461 finalizers=50 implementations=606 typed=95 macros=542 result=343 insert=199
S9 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S9 CENSUS test-support frames=7 finalizers=1 implementations=21 typed=13 macros=10 result=6 insert=4
S9 PARTITION implementations=622 macros=553
S9 TRANSITION T0029 removed; successor/provider=F0341
S9 TRANSITION Whole0034 removed; successor/provider=D122
S9 TRANSITION Whole0045 removed; successor/provider=D049
S9 TRANSITION Whole0046 removed; successor/provider=D049
S9 TRANSITION Whole0047 removed; successor/provider=D049
S9 TRANSITION Whole0048 removed; successor/provider=D049
S9 TOTAL typed=95 typed_macros=55 witnessed_sites=46; one transport map and five identity decodes removed; modes unchanged; preflip remains refused
```

S8 production was `(485, 51, 606, 558, 354, 204)`. The delta is 24 `SuspensionFrame` to `TypedFrame`, one `PureFrameFinalizer` to `SlotFinalizer`, and 16 macro sites leaving the macro counts (8 of the 24 owned M sites still use `resume_typed_or_suspend!`; 16 are cold helpers). M0350's identity is `eval_catch#call9:suspend_catch:`. The receipt was not edited for that ordinal.

### Performance closed in section 5

Ordinary controls, step 3, ownership, retention, the rejection census, the workspace, the base-source warning build, and the director probes passed on this tree. Allocation, instructions, copies, stack, resume attribution, and timing are in section 5.

### Director probes

Binary: `.temp/brief-617/target-director-probe/debug/director-depth-leak-probe`, built from `.temp/brief-617/depth-leak-probe` against the candidate `crates/rum-host`. `Finished dev profile in 58.34s`. Cap 10. Log: `.temp/brief-617/logs/director-probes.log`. Result files: `.temp/brief-617/director-{s9,s8,s7,s6}-probe/result.json`.

```
PROBE s9 PASS same=61 pauses>0
PROBE s8 PASS same=48 pauses>0
PROBE s7 PASS same=32 pauses>0
PROBE s6 PASS same=22 pauses>0
```

Each result JSON has `exit=0`, `diff=0`, `pause_rows` equal to the expected count, and `min_pauses=1`.

### Base-source warnings

`cargo test --manifest-path .temp/brief-617/base-source/Cargo.toml --offline -p rum-runtime --lib --no-run`. Finished `test` profile in 46.22s. Exit 0. Log: `.temp/brief-617/logs/base-source-warn.log`. The crate path is `/home/remi/dev/rum/.temp/brief-617/base-source/crates/rum-runtime`.

```
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
warning: `rum-runtime` (lib test) generated 3 warnings
```

These three warnings are present at `99ceab48`. The candidate workspace build prints the same three. The diff did not introduce them, so they are left in place.

### Suspension oracle

`scripts/check-suspension-oracle.sh 1500`. Log: `.temp/brief-617/logs/oracle.log`. Canonical copies: `.temp/suspension-oracle/gate-default.txt` and `gate-no-default.txt`. No compiler warning in either build.

Default: `test result: ok. 369 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.65s`. `continuation-slot fault-occurrence delta=0; observations=93114`.

No-default: `test result: ok. 369 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.64s`. `continuation-slot fault-occurrence delta=0; observations=93114`.

S9 fixtures `director_01` through `director_61` and the harness cases printed `strict equality PASS` in both builds.

## 1. Files changed

Character counts are against `.temp/brief-617/base-source/` (the `99ceab48` export). Twenty-three paths differ from `base-source-hashes.json`. Five product files are new. No `Cargo.toml`, lockfile, toolchain, `witness.rs`, core `Expr`, or new `unsafe`. No crate-public function signature changed. The new `pub(super)` helpers and `ExceptionResumeRoute` are visible only inside `control_data`. `TYPING_RECEIPT` moved `S8` → `S9`. `fixtures/mod.rs` gained `mod control_cleanup;`. `typed_slot_census.rs` registers `s9`.

This step did not edit `CLAUDE.md`, the repository `PROGRESS.md`, `TODO.md`, `docs/dev/security-hardening-progress.md`, or `scripts/dispatch-brief.sh`. The hash map does not cover `docs/`. The only document this step adds is this report, appended to the brief after the report is complete.

| File | Chars before → after | Semantic change |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/options.rs` | 2720 → 2746 | M0293/F0283 stays the caller-place witnessed macro on the `PropagateAborts` value. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs` | 6433 → 8367 | F0283–F0287 implement `TypedFrame`. New `pub(super)` cold helpers: `suspend_check_abort_with_propagation`, `suspend_abort_protect`, `suspend_check_abort_after_primary_pending`, `suspend_check_abort_after_primary_aborted`. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs` | 14341 → 14338 | `CheckAbort` and `AbortProtect` call those helpers. `abort_protect_depth` still increments before the body and decrements in `resume_abort_protect_after_body`. `abort_protect_pending` stays a frozen checkpoint. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs` | 7892 → 8009 | F0288–F0290 implement `TypedFrame`. The identity `|value|` re-encode retired with the conversion. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs` | 9524 → 9514 | F0313 and F0314 implement `TypedFrame`. Frozen message and handled-error lengths stay plain integers. `FrameAbandon` stays. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs` | 8305 → 8854 | F0315–F0317 implement `TypedFrame`. F0572 implements `SlotFinalizer` and still `Arc::try_unwrap`s the primary outcome. Primary `evaluate` stays inside `drive_with_cleanup_primary`. `evaluate(cleanup.clone())` stays. F0315 and F0316 `FrameAbandon` stay. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions/resume_frames.rs` | 4600 → 9400 | F0334–F0338 implement `TypedFrame`. New `pub(super)` cold helpers and the local `ExceptionResumeRoute` trait (`()` or `ContinueToken`, both `Result<Expr, ResumeStop<Expr>>`). One rustfmt import collapse moved ownership rows −2. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs` | 19085 → 19989 | Ordinary `dispatch_exception_handlers_owned` and resumed `dispatch_suspended_exception_handlers` stay separate. |
| `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs` | 18709 → 18793 | F0339–F0343 implement `TypedFrame`. T0029 and the identity re-encodes are gone. `slot_witness(&body_result)` is the next statement after `body_result`. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 35723 → 36007 | Derived-facts disjunct: `FinishWithCleanupFinalizer` with `type Output = Expr` and `fn finish(self: Box<Self>) -> Result<Expr, EvalError>`. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 256580 → 256584 | 29 coordinate moves inside S9 files. 0 added, 0 removed. No row outside those files. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 417750 → 416851 | 28 coordinate moves. 6 closures retired with the conversion (below). 0 added. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 396447 → 394005 | 24 `resume_or_suspend!` rows retired. 8 `resume_typed_or_suspend!` rows added. 26 `format!` / `matches!` / `vec!` / `panic!` rows are line moves with the same end column. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 91646 → 91646 | 68 rows. Only the fingerprint column changed. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 112156 → 112156 | `N:WithCleanup` evidence `cleanup.rs:80` and `:85` → `:61` and `:66`. Every other column of that row, and every other row, is unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 18162 → 18162 | C01 and C02 certificate cells, hash=true and hash=false, body fingerprints only. Status, identity, source span, preconditions, strength, exclusions, domain fingerprints, and build-inputs stayed. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | 3576 → 3597 | `mod control_cleanup;` |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | 181 → 181 | `TYPING_RECEIPT`: `"S8"` → `"S9"`. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | 682791 → 683009 | Receipt replay for the S9 transitions in section 2. |
| `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` | 13764 → 13839 | Production expectation `(461, 50, 606, 542, 343, 199)` and `typed=95`. |
| `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` | 11713 → 11791 | S9 witnessed-site inventory. The witness proof is unchanged. |
| `crates/rum-stdlib/tests/typed_slot_census/s4.rs` | 6027 → 6089 | The historical S4 filter excludes the earned S9 cohort. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | 7967 → 8011 | Registers `s9`. |
| `crates/rum-stdlib/tests/typed_slot_census/s9.rs` | 0 → 5161 | New closed S9 reconciliation. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/control_cleanup/mod.rs` | 0 → 1167 | `Checks::ALL` equality runner. `director_nonlocal` is `#[path]`-registered from `director.rs`. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/control_cleanup/director.rs` | 0 → 8671 | Director shapes `director_01`–`director_39`. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/control_cleanup/director_nonlocal.rs` | 0 → 5917 | Director shapes `director_40`–`director_61`. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/control_cleanup/harness.rs` | 0 → 6940 | The 27 resume-harness workloads. |

New modules are under 10 000 characters. `exceptions.rs` (19 989) and `nonlocal.rs` (18 793) were already the large files; they were not split.

### Derived-facts classification

Ownership rows that differ are inside `control_data/{abort_cleanup,abort_cleanup/options,abort_cleanup/resume_frames,assertions,check_quiet,cleanup,exceptions,exceptions/resume_frames,nonlocal}`. Attribute descriptions: same text, new coordinates, 29 pairs.

Captures retired with the conversion, and absent from the candidate inventory:

- `assertions.rs:200` `||`
- `assertions.rs:218` `|value|`
- `check_quiet.rs:102` `||`
- `check_quiet.rs:198` `||`
- `nonlocal.rs:440` `|value|`
- `nonlocal.rs:545` `|value|`

The three `|value|` rows are the identity re-encodes. The three `||` rows are the old suspend closures. No capture was added. No row changed kind while its source text stayed.

Macros, keyed by path, macro name, and end column: the only unpaired keys are 24 removed `crate :: resume_or_suspend!` and 8 added `crate :: resume_typed_or_suspend!`, all under `control_data`. The other 26 differing rows (`format!`, `matches!`, `vec!`, `panic!`) match on that key and moved only in line number. Sixteen owned sites left the macro inventory because they are cold helpers. That is the census delta 558 → 542.

`local-contracts.tsv`: 68 data rows, fingerprint column only. Hashes were emitted by `regenerate_local_contract_fingerprints` and copied. They were not hand-written.

`promoted.tsv` body fingerprints, old → new, both instances where a cell stores two:

- C01 hash=true `fnv1a64:f9944723806dc386` → `fnv1a64:079e6e0ce7b39dd3`
- C01 hash=false `fnv1a64:b15af49e8f9c4789` → `fnv1a64:07d5b5cfdc82cc05`
- C02 hash=true `fnv1a64:409a4ade3aa7dbd8` → `fnv1a64:a554509ba2bd0da1`
- C02 hash=false `fnv1a64:7779d5df008a8707` → `fnv1a64:875ca42ae3e9908b`

Domain fingerprints stayed `fnv1a64:f1b1b5ec1f9ab7c9` (hash=true) and `fnv1a64:ed48860e2b2c615f` (hash=false). Build-inputs stayed `fnv1a64:ccc66d3e8285d9e5`. `options/core.rs` was not edited. The cells were installed from `.temp/brief-546/certificates/promoted-hash-{true,false}.tsv`. Classification: paired signature-context refresh. No status, exclusion, strength, precondition, source span, or `direct-covered={}` change.

`obligations.tsv`: paired citation move for `N:WithCleanup` only, onto `resume_with_cleanup_after_primary` (`cleanup.rs:61`) and `evaluate(cleanup.clone())` (`cleanup.rs:66`).

No new `UnknownHostPayload`. No production receipt outside the S9 transitions in section 2.

## 2. Owned-row reconciliation

### Transitions

| Row | Transition |
| --- | --- |
| T0029 | Removed. Successor F0341. Was the `.map(into_continuation_expr)` after `finish_catch_match`. |
| Whole0034 | Removed. Successor D122. |
| Whole0045–Whole0048 | Removed. Successor D049. These are the identity decodes. |
| F0283 | `EvalCheckAbortSuspensionFrame`, `Expr -> Expr` |
| F0284 | `ResumeCheckAbortWithPropagationSuspensionFrame`, `Expr -> Expr` |
| F0285 | `EvalAbortProtectSuspensionFrame`, `Expr -> Expr` |
| F0286 | `ResumeCheckAbortAfterPrimarySuspensionFrame1`, `Expr -> Expr` |
| F0287 | `ResumeCheckAbortAfterPrimarySuspensionFrame2`, `Expr -> Expr` |
| F0288 | `EvalAssertSuspensionFrame`, `Expr -> Expr` |
| F0289 | `ResumeAssertAfterFunctionSuspensionFrame`, `Expr -> Expr` |
| F0290 | `FinishAssertFunctionSuspensionFrame`, `Expr -> Expr` |
| F0313 | `EvalCheckSuspensionFrame`, `Expr -> Expr` |
| F0314 | `EvalQuietSuspensionFrame`, `Expr -> Expr` |
| F0315 | `EvalWithCleanupSuspensionFrame`, `Expr -> Expr` |
| F0316 | `DriveWithCleanupPrimarySuspensionFrame`, `Expr -> Expr` |
| F0317 | `ResumeWithCleanupAfterPrimarySuspensionFrame`, `Expr -> Expr` |
| F0333 | `EvalCatchExceptionsSuspensionFrame`, `Expr -> Expr` |
| F0334 | `DispatchExceptionHandlersOwnedSuspensionFrame1`, `CompiledMatchOutcome -> Expr` |
| F0335 | `DispatchExceptionHandlersOwnedSuspensionFrame2`, `CompiledMatchOutcome -> Expr` |
| F0336 | `DispatchSuspendedExceptionHandlersSuspensionFrame`, `CompiledMatchOutcome -> Expr` |
| F0337 | `ResumeSuspendedExceptionMatchOutcomeSuspensionFrame`, `CompiledMatchOutcome -> Expr` |
| F0338 | `EvaluateExceptionHandlerRhsSuspensionFrame`, `Expr -> Expr` |
| F0339 | `EvalCatchSuspensionFrame`, `Expr -> Expr` |
| F0340 | `ResumeCatchForThrowSuspensionFrame`, `Expr -> Expr` |
| F0341 | `FinishCatchForThrowWithFormSuspensionFrame`, `CompiledMatchOutcome -> Expr` |
| F0342 | `ResumeCatchForBypassingThrowSuspensionFrame`, `Expr -> Expr` |
| F0343 | `FinishCatchForBypassingThrowWithFormSuspensionFrame`, `CompiledMatchOutcome -> Expr` |
| F0572 | `FinishWithCleanupFinalizer`, `SlotFinalizer`, `-> Expr` |

`CompiledMatchOutcome` stays Legacy. No new slot type. `Expr` is `Value`.

### Call path and route

No owned site re-enters its own frame from the ordinary path. The resumed CatchExceptions dispatcher is the only place that consumes a token for its own frame type. `ExceptionResumeRoute` is local to `exceptions/resume_frames.rs`: `()` and `ContinueToken` both return `Result<Expr, ResumeStop<Expr>>`, so one function can suspend F0336 or F0337. Census requires `route.suspend` on `suspend_suspended_exception_handlers` and `suspend_suspended_exception_match_outcome`.

| Site | Route |
| --- | --- |
| M0293 / F0283 | Ordinary-only witnessed macro in the `PropagateAborts` value. Resume continues `resume_check_abort_with_propagation` and does not re-enter this site. |
| M0294 / F0284 | Ordinary-only witnessed cold helper. Resume calls `resume_check_abort_after_primary`, which may insert F0286 or F0287, then `TypedAction::complete`. |
| M0295 / F0285 | Ordinary-only witnessed cold helper. Resume calls `resume_abort_protect_after_body` (depth decrement) and does not re-enter. |
| M0296 / F0286 and M0297 / F0287 | Ordinary-only witnessed cold helpers inside `resume_check_abort_after_primary`. Resume calls `resume_check_abort_after_fallback` with the frozen propagate/protected/pending checkpoint. The full `Result` is passed through. |
| M0298 / F0288 | Witnessed macro. Resume calls `resume_assert_after_function` only on `Ok`. |
| M0299 / F0289 | Ordinary-only witnessed cold helper. Resume calls `resume_assert_after_test` with the full `Result`. |
| M0300 / F0290 | Witnessed macro. Resume drops the success value and returns `Null`. Errors propagate. |
| M0323 / F0313 and M0324 / F0314 | Ordinary-only witnessed cold helpers. Checkpoints are plain integers. Resume passes the full `Result` into `resume_check_after_primary` / `resume_quiet_after_primary`. `FrameAbandon` restored. The selector `Arc` is built only in the cold helper. |
| M0325 / F0315 | Ordinary-only witnessed cold helper. Resume calls `resume_with_cleanup_after_init`, which may insert F0316 and completes with that `Result`. |
| M0326 / F0316 | Ordinary-only witnessed cold helper inside `drive_with_cleanup_primary` (also reached from F0315's resume). Own resume calls `resume_with_cleanup_after_primary`, which may insert F0317. |
| M0327 / F0317 | Ordinary-only witnessed cold helper. Resume is `TypedAction::finalize(FinishWithCleanupFinalizer)` after cloning the `Arc`. F0572 `finish` is `try_unwrap`. |
| M0344 / F0333 | Ordinary-only witnessed cold helper. Resume uses the suspended dispatcher. It may insert F0336. |
| M0345 / F0334 and M0346 / F0335 | Ordinary-only witnessed cold helpers. Resume maps a non-`InputRequired` error to `ResumeStop::failure` and continues the suspended dispatcher. |
| M0347 / F0336 and M0348 / F0337 | `ExceptionResumeRoute`. Own-frame re-entry consumes the token. The other site uses `()`. Not on the true ordinary path. |
| M0349 / F0338 | Witnessed macro. The `Result` is the output. |
| M0350 / F0339 | Ordinary-only witnessed cold helper. Resume calls `resume_catch_after_body` with the full `Result`. |
| M0351 / F0340 and M0353 / F0342 | Witnessed macros. Their own frames do not re-enter them. F0339's resume can reach them and stops by completing `InputRequired` after the macro inserts the next frame. |
| M0352 / F0341 and M0354 / F0343 | Witnessed macros. `Input` is `CompiledMatchOutcome`. Identity decode and `into_continuation_expr` are gone. `finish_catch_match` runs on both paths. T0029 was M0352/F0341's old map. |

Insertion identities printed by the census are in `.temp/brief-617/logs/census-no-default.log` under `S9 INSERTION`.

### Workspace

`cargo test --workspace --no-fail-fast` with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`. Exit 0. Log: `.temp/brief-617/logs/workspace.log`. 53 suites, 3801 passed, 0 failed, 59 ignored. The only compiler warnings are the three `test_builtins` dead_code warnings. The base-source build shows the same three at `99ceab48`.

The nine native ZeroMQ tests printed `ok` below. They skipped in this sandbox. A loopback bind fails with EPERM (`BIND_FAIL 1 Operation not permitted` in `.temp/brief-617/logs/calibration.log`). `loopback_bind_available` in `crates/rum-jupyter/src/zeromq_native_test_support.rs` returns false on `ErrorKind::PermissionDenied` when `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` is set, and the test still prints `ok`. The director ran them natively, and all nine passed. The workspace log lines are:

- `zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket` ... ok
- `zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets` ... ok
- `zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip` ... ok
- `zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages` ... ok
- `zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata` ... ok
- `zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown` ... ok
- `zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply` ... ok
- `zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end` ... ok
- `zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages` ... ok

Every `test result:` line:

```
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.49s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.98s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 43.47s
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1276.87s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.81s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 369 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.14s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 284.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.42s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.72s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.79s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Ordinary controls

Both builds of the brief-611 harness, retargeted to `.temp/brief-617/ordinary-{default,no-default}.tsv`. Log: `.temp/brief-617/logs/ordinary.log`.

Each build printed `ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence` and `continuation-slot fault-occurrence delta=0` through `observations=336`.

Default: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s`.

No-default: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s`.

### Step 3 fixtures on the unedited tree

`.temp/brief-617/step3-base/` is the `99ceab48` export plus the final `fixtures/control_cleanup/` directory and the one registration line `mod control_cleanup;` in that copy's `fixtures/mod.rs`. Log: `.temp/brief-617/logs/step3-base-fixtures.log`.

Default: `test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 281 filtered out; finished in 0.69s`.

`--no-default-features`: `test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 281 filtered out; finished in 0.68s`.

`director_01` through `director_61` and the harness cases printed `strict equality PASS` in both builds. Fault-occurrence delta stayed 0. The last harness observation in each build was 1056.

## 3. Harness, stack shapes, and oracle fixtures

### Director probe

`.temp/brief-617/depth-leak-probe` built debug against the candidate `crates/rum-host`. Cap 10. Log `.temp/brief-617/logs/director-probes.log`.

```
PROBE s9 PASS same=61 pauses>0
PROBE s8 PASS same=48 pauses>0
PROBE s7 PASS same=32 pauses>0
PROBE s6 PASS same=22 pauses>0
```

Each `director-s{9,8,7,6}-probe/result.json` has `exit=0`, `diff=0`, and `min_pauses=1`. The 61 S9 shapes are the director's re-probe: CheckAbort and AbortProtect (body, fallback, both, `PropagateAborts`, pending protected abort), Assert (function lookup, test, user handler), Check and Quiet (messages before, inside, and after the pause), WithCleanup (init, body, cleanup, throw, abort, bypassing throw), Catch (body, form, guard, handler, bypassing form and guard), CatchExceptions (body, ordinary and resumed guard and match, declining handler, right-hand side, ancestor match), plus `Table` and nesting. S8's 48, S7's 32, and S6's 22 are the regression set.

### Oracle fixtures

`fixtures/control_cleanup/` is 88 tests: `director_01`–`director_61` and 27 harness cases (`s9_checkabort_*`, `s9_abortprotect_body`, `s9_assert_*`, `s9_check_body`, `s9_quiet_*`, `s9_cleanup_*`, `s9_catchexceptions_*`, `s9_catch_*`). Each compares the queued-input path with the resumed path under `Checks::ALL` and requires fault absence.

On the candidate, both oracle builds printed `strict equality PASS` for these cases. `continuation-slot fault-occurrence delta=0; observations=93114` in each build.

On the unedited tree (`.temp/brief-617/step3-base/`, `99ceab48` plus these fixtures and one `mod control_cleanup;`), both builds:

```
test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 281 filtered out; finished in 0.69s
test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 281 filtered out; finished in 0.68s
```

Every `director_01`–`director_61` and harness case printed `strict equality PASS`. Fault delta stayed 0. The last observation in each build was 1056 (`s9_quiet_check_fallback`). Log: `.temp/brief-617/logs/step3-base-fixtures.log`.

The harness frame census is `expected_frames_per_iteration` in `.temp/brief-617/s9-resume-workloads.json` (27 workloads). Single frame: F0283 option; F0284 primary and abort; F0285 abort-protect body; F0287 fallback; F0288 function lookup; F0289 test; F0290 handler; F0313 check body; F0314 quiet body; F0315 cleanup init; F0316 cleanup primary; F0333 body; F0334 guard; F0335 match; F0338 right-hand side; F0339 catch body; F0340 form; F0341 form guard. Two frames: F0285+F0286 pending fallback; F0313+F0314 quiet-check fallback; F0333+F0336 suspended match; F0333+F0337 suspended guard; F0339+F0342 bypassing form; F0339+F0343 bypassing guard. `cleanup-finalizer` is F0317+F0572 with `resumes_per_iteration` 1. `cleanup-three-phase` is `resumes_per_iteration` 3 and names F0315, F0316, F0317, and F0572. `resumes_per_iteration` is 2 only on the two suspended CatchExceptions workloads.

### Ordinary controls

Both builds, brief-611 harness retargeted to `.temp/brief-617/ordinary-{default,no-default}.tsv`. Log `.temp/brief-617/logs/ordinary.log`.

```
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

`continuation-slot fault-occurrence delta=0` through `observations=336` in each build.

### Stack shapes

Seventeen shapes, debug and release, four heights, three samples. Every fit has residual 0. Log `.temp/brief-617/logs/candidate-stack.log`.

The ten carried shapes match the calibrated base on both profiles: `peel`, `plus`, `sub`, `up`, `guarded`, `rhs-guard`, `replace-guard`, `replaceall-guard`, `count-guard`, `position-guard`.

Family shapes, candidate minus base, bytes per level:

| Shape | debug | release |
| --- | --- | --- |
| catch-body | −736 | −64 |
| checkabort-body | −784 | −448 |
| abortprotect-body | −624 | −64 |
| cleanup-body | −512 | −64 |
| check-body | −720 | −320 |
| quiet-body | −704 | −256 |
| catchexceptions-body | −656 | −192 |

No shape grew.

## 4. All gates

`cargo fmt --check` wrote no diagnostics. Exit 0. `.temp/brief-617/logs/fmt.log` records `FMT_EXIT:0`.

Lib and census gates, `.temp/brief-617/logs/pre-oracle-gates.log` and the census logs. The three `test_builtins` dead_code warnings below are the same three at `99ceab48` (section 0). No other compiler warning.

```
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.48s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.37s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.17s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.96s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.30s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 32.00s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 274.77s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 275.61s
```

The ignored census test is `forced_red_probe`. The no-default census log is `.temp/brief-617/logs/census-no-default.log`. Its production lines are quoted in section 0.

Oracle, `.temp/brief-617/logs/oracle.log`. Neither build warned.

```
test result: ok. 369 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.65s
continuation-slot fault-occurrence delta=0; observations=93114
test result: ok. 369 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.64s
continuation-slot fault-occurrence delta=0; observations=93114
```

Ownership, `.temp/brief-617/logs/inventory-gates.log`:

```
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 70.18s
```

The first `local_production_acceptance` on that log failed because `drive_with_cleanup_primary`'s signature collapse had moved every body fingerprint:

```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 17.90s
```

After the paired fingerprint and certificate refresh, `.temp/brief-617/logs/acceptance.log`:

```
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.35s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.29s
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.55s
```

Rejections on C01, C02, and the four lease probes are 0 in both feature sets.

Workspace, `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, exit 0, `.temp/brief-617/logs/workspace.log`. 53 suites, 3801 passed, 0 failed, 59 ignored. The nine native ZeroMQ tests printed `ok`. They skipped in this sandbox: loopback bind returns EPERM, and `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` makes `loopback_bind_available` return false. The director ran them natively, and all nine passed. Their names are in section 2. Every workspace `test result:` line is quoted there.

Ordinary controls and the step-3 fixture totals are quoted in section 3. Director probe lines are quoted in section 0 and section 3.

Base-source warning build, `.temp/brief-617/logs/base-source-warn.log`, exit 0, finished in 46.22s:

```
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
warning: `rum-runtime` (lib test) generated 3 warnings
```

Inventory and certificate transitions are in section 1.

Canonical paths: `.temp/brief-617/logs/{fmt,pre-oracle-gates,census-default,census-no-default,oracle,inventory-gates,acceptance,workspace,ordinary,step3-base-fixtures,base-source-warn,director-probes,candidate-builds,candidate-stack,perf-sweep,instruction-extra,instruction-comparison,alloc-comparison,analyze-copies,attribution,resume-analysis,timing,timing-probe}.log`, `.temp/suspension-oracle/gate-default.txt`, `.temp/suspension-oracle/gate-no-default.txt`, `.temp/brief-617/perf/{allocation,instruction}-comparison.json`, `.temp/brief-617/ordinary-callgrind/{copy-slopes,function-self,flag-functions}.json`, `.temp/brief-617/resume-callgrind/{driver-136,function-deltas}.json`, `.temp/brief-617/resume-callgrind/per-label/`, `.temp/brief-617/stack/prologue-comparison.json`, `.temp/brief-617/timing/probe-summary.json`, `.temp/brief-617/timing/close-facts.json`.

## 5. Performance

Candidate binaries, one codegen unit, cwd `/home/remi/dev/rum`. Log: `.temp/brief-617/logs/candidate-builds.log`.

- `rum-cli` `72cbddece45abc91d8494605b152ec79e58b4c4cceabb1bbaaf63dd263ed1e4b` (1m 46s)
- `rum-cli-alloc` `66c7804448cc2f2fd7966cf46f3ff26f8b2678ba40d2b1b971908561491b4e7d` (1m 42s)
- `resume` `b02b589f22c344ecba230be7063d220f70be6268d33f36af00d82f4384e291c9` (1m 37s)
- `resume-alloc` `6ba8546e85762d00e88c0b8458f252fc436f285f36a44d514851f9a14427a1ff` (1m 38s)

### Allocations

Both arms, every label, N and 2N. Log: `.temp/brief-617/logs/perf-sweep.log`. Comparison: `.temp/brief-617/perf/allocation-comparison.json`.

Ordinary: `164 / 164` raw N and 2N count and byte equality. Slope and whole-program counts match.

Resume: `67 / 95` raw exact. The 28 differences are 24-byte adapter counts:

- 27 S9 workloads: slope `+24` bytes and `+1` allocation per newly typed transfer in that iteration (one transfer: +24; two: +48; `resume-s9-cleanup-three-phase`: +72). Intercept 0.
- `resume-cleanup` (`WithCleanup` around one `Do` of N pauses): slope 0, intercept `+1` count and `+24` bytes. One typed WithCleanup transfer for the whole workload, not one per pause.

### Instructions

Two `perf stat` samples per arm, then five more on the 25 ordinary flags. Logs: `perf-sweep.log`, `instruction-extra.log`.

`literal-integer` and `do-iterator-null` were not flagged. `do-count-null` was flagged on two samples (mean +0.23) and cleared on five (mean −0.026).

Persistent ordinary flags after five samples:

- `catchexceptions-no-throw`: mean +18.016 instructions per call. All five candidate slopes sit above all five base slopes (base about 3839–3849, candidate about 3859–3864).
- `matcher-single-tiny`: mean +7.70. Candidate minimum 11581.8, base maximum 11580.8.

`abortprotect-clean` was omitted from that five-run clearance. The director measured +8.4 instructions per call, disjoint over five runs. It is the ordinary-path 136-byte `Result` copy (memcpy 14 → 15 on that label). The two-sample row in `.temp/brief-617/perf/instruction-comparison.tsv` flags it `True` with slope range `[-0.892, 7.344]`. That range includes a negative value, and the five-run text treated the label as inside threshold. That sentence was wrong.

### Stack

Four heights, three samples, both profiles. Every fit has zero residual. Log: `.temp/brief-617/logs/candidate-stack.log`.

The ten shapes carried forward match the calibrated base exactly (debug and release).

Family shapes, candidate minus base, bytes per level:

| Shape | debug | release |
| --- | --- | --- |
| catch-body | −736 | −64 |
| checkabort-body | −784 | −448 |
| abortprotect-body | −624 | −64 |
| cleanup-body | −512 | −64 |
| check-body | −720 | −320 |
| quiet-body | −704 | −256 |
| catchexceptions-body | −656 | −192 |

No shape grew. The extra `drive_with_cleanup_primary` frame on `cleanup-body` did not increase the bytes per level.

### Prologues

`objdump -d` of both `runtime-tests` binaries, both profiles. Comparison: `.temp/brief-617/stack/prologue-comparison.json`. Log lines are in `.temp/brief-617/logs/attribution.log`.

`control_data` symbols: 1396 rows. 1047 have the same prologue size. 42 changed. 307 exist on only one arm: the base-only rows are the old suspension-frame methods (90) plus four helpers; the candidate-only rows are the new typed frame methods and the `suspend_*` cold helpers.

Debug prologue deltas for the ordinary-path functions (bytes):

| Function | base | candidate | delta |
| --- | --- | --- | --- |
| `eval_catch` | 1848 | 1112 | −736 |
| `resume_check_abort_with_propagation` | 1304 | 648 | −656 |
| `drive_with_cleanup_primary` | 1000 | 456 | −544 |
| `dispatch_exception_handlers_owned` | 2648 | 1784 | −864 |
| `eval_catch_exceptions` | 1336 | 680 | −656 |
| `eval_check` | 1800 | 1080 | −720 |
| `eval_quiet` | 1672 | 968 | −704 |
| `eval_with_cleanup` | 1912 | 936 | −976 |
| `eval_abort_protect` | 1160 | 536 | −624 |

Release copies of those ordinary functions are unchanged or smaller (`eval_catch_exceptions` 1320 → 1128, `resume_check_abort_with_propagation` 536 → 280). Release growth is on resume helpers, not on the recursive ordinary frames: `dispatch_suspended_exception_handlers` +80 (440 → 520), `resume_assert_after_test` +16, `resume_with_cleanup_after_init` +32, `resume_with_cleanup_after_primary` +16, `evaluate_exception_handler_rhs` +16, `resume_suspended_exception_guard_outcome` +16, `resume_suspended_exception_match_outcome` +32, `finish_catch_for_bypassing_throw_with_form` +16, `resume_catch_after_body` +16. Debug growth on the suspended dispatcher and its resume helpers is +32 to +288. Those helpers are not the recursive ordinary frame, and none of the seventeen shapes grew.

### Copies and ordinary-path instructions per call

The ordinary copy slopes in `.temp/brief-617/logs/analyze-copies.log` and `.temp/brief-617/ordinary-callgrind/copy-slopes.json` are not a memcpy call count. `analyze-copies.py` matched only names containing `memcpy`, and libc was unresolved (`0x1b1980`), so every label read a constant. The line `memcpy at or below base 164 / 164 memmove 164 / 164 above 0` is false. The director's census names `__memcpy_avx_unaligned_erms` and finds seven ordinary labels above base.

`literal-integer` and `do-iterator-null` were not instruction-flagged. `do-count-null` cleared on the five-run set (mean −0.026).

Own instructions per call from ordinary callgrind sample 1 (self slope / call slope). Log: `.temp/brief-617/logs/resume-analysis.log`. JSON: `.temp/brief-617/ordinary-callgrind/function-self.json`.

| Function | base | candidate |
| --- | --- | --- |
| `eval_catch` (stdlib symbol; runtime body inlined into it) | 9.00 / call, 1 call | 9.00 / call, 1 call |
| `resume_check_abort_with_propagation` on `checkabort-no-abort` | 76.00 / call | 45.00 / call |
| `resume_check_abort_with_propagation` on `checkabort-caught` and `checkabort-abort` | 92.00 / call | 53.00 / call |
| `drive_with_cleanup_primary` | 66.00 / call, 1 call | no release symbol, 0 calls (inlined) |
| `rum_stdlib::control_flow::eval_with_cleanup` | 9.00 / call | 9.00 / call |
| `dispatch_exception_handlers_owned` | no release symbol on either arm | no release symbol on either arm |
| `rum_stdlib::control_flow::eval_catch_exceptions` on no-throw, handler, and caught | 9.00 / call, 1 call | 9.00 / call, 1 call |

The candidate's inlined `drive_with_cleanup_primary` does not increase the caller's 9 instructions. Debug prologues for `dispatch_exception_handlers_owned` are 2648 → 1784 bytes. The release symbol is absent on both arms, so there is no separate release self-cost.

### Ordinary flag attribution

Six callgrind samples per arm. JSON: `.temp/brief-617/ordinary-callgrind/flag-functions.json`.

`catchexceptions-no-throw`: deterministic −11.00 in `eval_control_data_builtin` (81 → 70) and deterministic +30.00 at libc `0x1b1980`, which is `memcpy@GLIBC_2.2.5+0x90` (`cmp $0x20,%rdx`, the 32-byte-and-up path). The memcpy call count did rise. The +30 at unresolved libc `0x1b1980` is the 136-byte `Result` copy, and the unnamed-libc instrument missed the extra call. The director measured `catchexceptions-no-throw` memcpy 7 → 8. Net +19 matches the perf mean +18.016 and is that extra copy.

`matcher-single-tiny` is outside this family. Its only delta is variable +7.83 to +8.14 at libc `0x1b1220` (`endbr64; cmp $0x20,%rdx; vpcmpeqb`), a glibc vector compare. Not a rum symbol. Not edited.

### Resume attribution

Six callgrind samples per arm for all 95 resume labels. Per-label JSON: `.temp/brief-617/resume-callgrind/per-label/`. Summary: `function-deltas.json`. Log: `.temp/brief-617/logs/resume-analysis.log` (`RESUME_ANALYSIS_DONE`, exit 0).

The driver's 136-byte typed-outcome move is two mutually exclusive `mov $0x88,%edx; call memcpy@GLIBC_2.14` sites inside `resume_eval_request_from_checkpoint`. Base: `0x7485bd` and `0x7485dd`. Candidate: `0x74953d` and `0x74955d`. Each site copies 136 bytes to the same stack slot and jumps to the same continuation. Their execution sum equals `TypedAdapter::resume` on every label and all six samples. The other ten 136-byte memcpy sites in that function have identical slopes on both arms (`remainder_delta_nonzero` is empty). `PROOF reconciled True`. JSON: `.temp/brief-617/resume-callgrind/driver-136.json`.

New typed resumes per iteration, all six samples identical, and therefore new 136-byte moves (count them, not repaired):

- +1 (136 bytes): the twenty single-transfer S9 labels (`checkabort-option`, `primary`, `abort`, `abortprotect-body`, `checkabort-fallback`, `assert-function-lookup`, `assert-test`, `assert-handler`, `check-body`, `quiet-body`, `cleanup-init`, `cleanup-primary`, `cleanup-finalizer`, `catchexceptions-body`, `guard`, `match`, `rhs`, `catch-body`, `catch-form`, `catch-form-guard`).
- +2 (272 bytes): `checkabort-pending-fallback`, `quiet-check-fallback`, `catchexceptions-suspended-match`, `catchexceptions-suspended-guard`, `catch-bypassing-form`, `catch-bypassing-guard`.
- +3 (408 bytes): `cleanup-three-phase`.
- `resume-cleanup`: slope 0. Absolute `TypedAdapter::resume` counts are base 3009 / 6009 and candidate 3010 / 6010 at N=1000 / 2000. The matched 136-byte sites are the same numbers. Intercept +1 resume and +136 bytes for the one typed `WithCleanup` around the loop.

Non-S9 labels have equal adapter slopes and equal 136-byte remainder slopes. Their whole-program means move by tens or hundreds of instructions because `rum_core::expr::Symbol::system` and `intern_text` are not deterministic (ranges of hundreds). Deterministic non-S9 rows outside those symbols are cancelling `???` address pairs (for example +12.03 at `0x4070160` and −12.03 at `0x406f160`), net zero.

S9 deterministic function deltas are the typed swap: `insert_suspension_frame` and `pop_front_mixed_legacy` fall, `insert_typed_erased_frame`, `TypedAdapter::resume`, `pop_front`, and `widen` rise, and each owned frame's `SuspensionFrame::resume` is replaced by `TypedFrame::resume` plus its `suspend_*` helper. `mi_malloc` / `Arc::drop_slow` rise with the 24-byte adapter. libc `0x1b1980` rises on the labels that take the new 136-byte copy. Example, `resume-s9-catch-body`: total mean +40.71, deterministic sum +91.30; top deterministic rows `EvalCatchSuspensionFrame::resume` +81, `insert_suspension_frame` −80, `suspend_catch` +70, `resume_eval_request_from_checkpoint` −59, `TypedAdapter::resume` +57, `insert_typed_erased_frame` +56, legacy `SuspensionFrame::resume` −56. The full per-label address lists are in the per-label JSON. Variable S9 rows are the same symbol-intern functions, not the typed frames.

### Timing

Indication only. Four panels, order base, candidate, base, candidate. Each command was `python3 scripts/run-bounded.py --timeout 1000 -- env TMPDIR=.temp/brief-617/tmp taskset -c 2 scripts/core-language-microbench.sh --timeout 900 --reps 2 --rum-cli .temp/brief-617/{base,cand}/rum-cli --output-dir .temp/brief-617/timing/panel{N}`. Profiling variables were forced off. The script used the existing binaries. Log `.temp/brief-617/logs/timing.log`, exit 0. Each `runs.tsv` line is `rum ok 0`.

```
PANEL ELAPSED rum 0 1.188757
PANEL ELAPSED rum 0 0.485445
PANEL ELAPSED rum 0 0.484843
PANEL ELAPSED rum 0 0.485047
PANELS_DONE
```

Panel 1 is the cold base run. Panels 2–4 sit at 0.485445, 0.484843, and 0.485047 seconds. Each stdout has 120 labels between `RUM_CORE_MICRO_BEGIN` and `RUM_CORE_MICRO_END`, then `Null`. `.temp/brief-617/timing/panel1-base/core-language-microbench.rum` is 446682 bytes and contains `rumMicroReps = 2;` once.

The probe copied that program and changed that one assignment to `rumMicroReps = 25;`. The file is 446687 bytes (delta 1). Ten runs per arm, base then candidate, `taskset -c 2`, stdin `/dev/null`, same binaries, profiling off. Each run was its own 900 s bound. Log `.temp/brief-617/logs/timing-probe.log`:

```
PROBE PROGRAM bytes 446687 delta 1
PROBE_DONE
```

Exit 0. Twenty `RUN`/`DONE` lines, order base 1, cand 1, through base 10, cand 10.

Warm-panel calibration (probe median divided by panel 3 for base and panel 4 for candidate), 240 ratios: minimum 0.690, median 0.971, maximum 1.731. Seven ratios sit outside 0.78–1.25:

| Label | Arm | Ratio |
| --- | --- | --- |
| `association-part-key-read-width4000` | base | 1.731 |
| `association-part-key-read-width4000` | cand | 1.621 |
| `matcher-single-wide-small` | cand | 0.728 |
| `appendto-list-incremental-small` | base | 0.690 |
| `appendto-list-incremental-small` | cand | 0.739 |
| `appendto-list-incremental-large` | base | 0.708 |
| `appendto-list-incremental-large` | cand | 0.746 |

These labels are outside S9. The low ratios are about 0.69–0.75 of the warm panel, not the 250× startup-limit miss. Against cold panel 1 alone, 43 of 120 base ratios fall outside the band and the minimum is 0.014, because that panel's wall time is 1.188757 s against about 0.485 s for the later panels. The mixed cold-plus-warm median in `probe-summary.json` is 0.961. The warm median 0.971 is inside the S8 band 0.78–1.25 (S8 median 0.99).

Ten-run candidate ranges sit entirely above the base range on two labels:

- `association-set-key-incremental-small`: base [1.413561e-05, 1.434081e-05], candidate [1.445882e-05, 1.484844e-05], median +2.50%
- `association-set-key-incremental-large`: base [2.699414e-05, 2.7306085e-05], candidate [2.772364e-05, 2.864406e-05], median +2.44%

Both are association writes. Their allocation slopes are exact. Their two-sample instruction means are +3.075 and −6.068, the ranges overlap, and the comparison leaves them unflagged (`.temp/brief-617/perf/instruction-comparison.tsv`). Instruction slopes are not equal, so the `-align-all-functions=6` rebuild does not apply. It was not run.

The four panels (two samples each) also put the candidate minimum above the base maximum for `table-iterator-plus` (+0.13%), `rewrite-guarded-decline-fallback` (+0.20%), `replace-wide-no-match` (+1.65%), `replacepart-deep-path-small` (+0.42%), `stringtrim-literal-list-32000` (+0.26%), `stringtrim-single-literal-32000` (+0.18%), and the two association labels (+0.32% and +2.27% on those two samples). The ten-run probe ranges overlap for the first six.

JSON: `.temp/brief-617/timing/probe-summary.json`. Warm ratios: `.temp/brief-617/timing/close-facts.json`.

## 6. Stop-loss, deviations, open questions

Counters: `compile_attempts` 0, `test_commands` 14 (the fmt recheck is the fourteenth), `non_progress_attempts` 0, `half_edited_files` []. Run 1's tree passed the census. The M0326 and M0350 failures were introduced by this run's review edits and removed by reverting them. The witness stays the next statement after the producer local because the census proof requires that. The one red `local_production_acceptance` was the stale fingerprint column; the paired refresh moved that gate to pass. No later attempt left a named gate unmoved.

Deviations and residuals, for the director:

- `ExceptionResumeRoute` in `exceptions/resume_frames.rs` is the local stand-in for `DefinitionResumeRoute`. One function suspends F0336 or F0337. Both `()` and `ContinueToken` return `Result<Expr, ResumeStop<Expr>>`. The two CatchExceptions dispatchers stay separate.
- `drive_with_cleanup_primary` evaluates the primary so the witness follows that local. The extra frame did not increase `cleanup-body` bytes per level (debug −512, release −64). On the release binary the function is inlined (base 66 instructions per call, candidate has no symbol). `eval_with_cleanup` stays 9 instructions per call on both arms.
- `catchexceptions-no-throw` perf mean is +18.016 instructions per call. Callgrind on the unresolved libc address: deterministic −11 in `eval_control_data_builtin` and deterministic +30 at libc `0x1b1980`. The memcpy call count did rise. The director measured 7 → 8.
- `matcher-single-tiny` perf mean is +7.70. The only callgrind delta is variable +7.83 to +8.14 at libc `0x1b1220` (vector compare). It is outside this family.
- Resume allocation: one 24-byte adapter `Arc` per newly typed transfer (slope +24, +48, or +72, intercept 0) on 27 S9 workloads. `resume-cleanup` is slope 0, intercept +1 count and +24 bytes.
- The 136-byte typed-outcome move is one copy per typed resume. Twenty labels +1, six labels +2, `cleanup-three-phase` +3. `resume-cleanup` is intercept +1 (base 3009/6009, candidate 3010/6010 at N=1000/2000). `PROOF reconciled True remainder []`.
- Non-S9 whole-program instruction movement is `Symbol::system` and `intern_text`, plus cancelling `???` address pairs (example +12.03 at `0x4070160` and −12.03 at `0x406f160`).
- Timing indication: the two association-set-key ranges above, about +2.5%. Alignment rebuild was not run.
- The three `dead_code` warnings in `test_builtins.rs` are present at `99ceab48`. `test_builtins.rs` was not edited.

Open questions:

1. `catchexceptions-no-throw` +18 is the extra 136-byte `Result` copy (director memcpy 7 → 8), not an unchanged call count. `matcher-single-tiny` +7.7 remains a glibc vector compare outside S9.
2. Settled by the director's aligned timing: 0 labels above base. The alignment rebuild was not run.
3. Settled: `ExceptionResumeRoute` is accepted.
4. Settled: the inlined `drive_with_cleanup_primary` is accepted. The primary evaluate stays inside that function.

The copy bar failed. The director's census shows seven ordinary labels whose memcpy count is above base. Native ZeroMQ skipped in this sandbox (loopback bind EPERM); the director's native run passed all nine. These were not run: the function-alignment rebuild; any rebuild of the supplied base `rum-cli`, `rum-cli-alloc`, `resume`, or `resume-alloc` binaries; `.temp/brief-617/build.py` against the candidate; `git` in any form.

## Director continuation 2 (2026-10-17) — the gate found an ordinary-path copy

The director's gate reproduced the following on the tree you left:
- the census (606/95) and the oracle (369 in both builds, delta 0);
- the workspace natively (3801/0/59; the nine ZeroMQ tests ran for real);
- allocations, the 17 stack shapes, and the 186 probe pairs;
- the inventory replay, which is byte-identical.

The aligned timing settled your open question 2 (0 labels above base). Questions 3 (`ExceptionResumeRoute`) and 4 (the inlined `drive_with_cleanup_primary`) are accepted.

**The copy bar fails.** On seven ordinary labels the memcpy count is above base. Five sites match the producer once and then hand the whole 136-byte `Result<Expr, EvalError>` over by value (`outcome => self.resume_…(…, outcome)`). That costs one extra memcpy per call. For `CatchExceptions[1]`, the extra copy is the candidate's `mov $0x88,%edx; call memcpy` at `0x61abb3` in `eval_control_data_builtin`, right after the `InputRequired` discriminant check. HEAD's macro rebuilt `Ok(value) => Ok(value)`, which copies only the `Expr`.

| Site | Where | Labels, memcpy per iteration (base → candidate) |
| --- | --- | --- |
| M0295 | `eval_abort_protect` | `abortprotect-small` 7 → 8, `abortprotect-clean` 14 → 15 |
| M0344 | `CatchExceptions` body, `exceptions.rs` | `catchexceptions-no-throw` 7 → 8, `-handler` 30 → 31, `-caught` 41 → 42 |
| M0325, M0326 | `WithCleanup` init, then the primary in `resume_with_cleanup_after_init` (`drive_with_cleanup_primary` is inlined there) | `withcleanup-init` 12 → 14 |
| M0299 | the assertion test with asserts on, `resume_assert_after_function` | `assert-enabled-true` 19 → 20 |

**The fix.** The director proved this shape on `AbortProtect` in a scratch tree:

```rust
match body_result {
    Ok(value) => self.resume_abort_protect_after_body(pending_checkpoint, Ok(value)),
    Err(EvalError::InputRequired { request }) => {
        resume_frames::suspend_abort_protect(self, pending_checkpoint, witness, request)
    }
    Err(error) => self.resume_abort_protect_after_body(pending_checkpoint, Err(error)),
}
```

With it, memcpy is back to 7 and 14, and the whole program is 7 413 197 instructions against base's 7 426 200. Apply the same shape at the other four sites. The witness still follows the producer local. Check every other S9 ordinary site for the pattern; the census below is the falsifier.

**Your copy instrument measured nothing.** In your sandbox libc had no symbols: rows read like "libc `0x1b1980`", and `analyze-copies.py` matched only names containing `memcpy`. So every label read a constant 5 or 6 at both N and 2N. The sandbox now also reads `~/.cache/debuginfod_client`, where this host keeps libc's debug symbols.

Use the director's census:
1. Copy `.temp/director/gate-617/callgrind/{run1.sh,copy_census.py}` to `.temp/brief-617/copy-census/`.
2. Point them there: `C` in `run1.sh`, and `CENSUS_DIR` with `CENSUS_CAND=cand`.
3. Run all 164 ordinary labels on both arms, at N and 2N. Take the programs and budgets from `ordinary-callgrind/`; the job format is `arm label size program`, as in `.temp/director/gate-617/callgrind/census/jobs.txt`.

**Calibrate before you trust it.** On the current, unfixed candidate:
- callgrind must name `__memcpy_avx_unaligned_erms`;
- `catchexceptions-no-throw` must read 7.000 → 8.000.

**Bar:** no label above base.

**Then rerun everything the fix can move:**
- rebuild the four candidates (`run-candidate-builds.sh`);
- `cargo fmt --check`;
- `typed_slot_census` in both builds;
- `scripts/check-suspension-oracle.sh 1500`;
- `frame_root_trace`;
- `ownership_graph`, with the derived-facts replay for any moved line;
- `local_production_acceptance` in both builds, refreshing the fingerprints with the canonical test if they are stale;
- `production_rejection_census_is_complete`;
- the copy census;
- five-run instruction slopes for the seven labels and the other S9 ordinary labels;
- the `abortprotect-body`, `catchexceptions-body`, `cleanup-body` and `check-body` stack shapes in both profiles.

The director reruns the full gate afterwards.

**Correct the report.** Fix these statements where they stand, and add a section *Continuation 2* to the report:
- "The nine native ZeroMQ tests ran and passed. They did not skip" and "Native ZeroMQ did not skip" are false. In your sandbox a loopback bind fails with EPERM (the director probed it), so the nine skipped. The director ran them natively, and all nine passed.
- "memcpy at or below base 164 / 164" and "The memcpy call count did not rise" are false, for the reason above.
- `abortprotect-clean` (+8.4 instructions per call, disjoint over five runs) was not reported. It is this defect.
- In section 0, the census failures at M0326 and M0350 came from your own review edits, which you then reverted. Run 1's tree passed the census.

**Save as you go — a hard rule.** Your weekly quota is nearly spent.
- Update `state.json` and `PROGRESS.md` after every edit and every gate.
- Append each report fragment as soon as it is final.

If the run is cut, the director takes over from what is on disk.

## Continuation 2

The director's copy bar failed on the unfixed tree: seven ordinary labels pass a whole `Result<Expr, EvalError>` by value and memcpy once more than base (`withcleanup-init` twice). Questions 2, 3, and 4 are settled and were not reopened. `ExceptionResumeRoute` stays. `drive_with_cleanup_primary` still evaluates the primary, and the witness is still the next statement after that local.

The ordinary-path matches now rebuild `Ok(value)` and `Err(error)` and still send `InputRequired` to the existing cold helper. Sites: M0294 CheckAbort primary, M0295 AbortProtect, both CheckAbort fallbacks, M0299 assert test, M0323 Check, M0324 Quiet, M0325 WithCleanup init, M0326 `drive_with_cleanup_primary`, M0344 CatchExceptions body, M0350 Catch. Non-S9 `outcome =>` arms were not edited. The candidate release binaries were not rebuilt. Their unfixed hashes remain `72cbddece45abc91d8494605b152ec79e58b4c4cceabb1bbaaf63dd263ed1e4b` (`cand/rum-cli`) and `d22bb4332f93b7c20f24f3e5ab77562e6a3b1055d128ecca99a2db08322e707c` (`base/rum-cli`).

Copy-census calibration on those unfixed binaries failed. Log `.temp/brief-617/logs/calibration.log`:

```
DEBUGINFOD_URLS=https://debuginfod.cachyos.org https://debuginfod.archlinux.org
BIND_FAIL 1 Operation not permitted
DEBUGINFOD_CACHE_READABLE
base catchexceptions-no-throw 500 rc=0 Null
base catchexceptions-no-throw 1000 rc=0 Null
cand catchexceptions-no-throw 500 rc=0 Null
cand catchexceptions-no-throw 1000 rc=0 Null
SYMBOL_HITS
__memcpy_avx_unaligned_erms -1
memcpy@ -1
memcpy 132836
catchexceptions-no-throw                      N=500   memcpy 0.000->0.000 memmove 0.000->0.000  ok
labels 1, copy count above base: 0
ABOVE:
CALIBRATION_DONE
```

`callgrind` did not name `__memcpy_avx_unaligned_erms`. `catchexceptions-no-throw` read `0.000->0.000`, not `7.000->8.000`. The libc debuginfo is in `~/.cache/debuginfod_client/e363db7c8fcc7c06c6f0d2839b2a8ad875b75406/debuginfo` (build id of `/usr/lib/libc.so.6`, file mode `r--------`, readable from this shell). `debuginfod-find` does not use it:

```
checking cache dir /home/remi/.cache/debuginfod_client
not found Permission denied (err=-13)
Server query failed: Permission denied
```

`strace` shows the refused call:

```
openat(AT_FDCWD, "/home/remi/.cache/debuginfod_client/cache_clean_interval_s", O_RDWR|O_CREAT, 0666) = -1 EACCES
```

The sandbox allows reading that cache and refuses writing it. The matcher was not weakened. The 164-label copy census was not run. No candidate binary was rebuilt after this failure.

### Continuation 2 census, default features

After the `InputRequired` arm was placed first, `cargo test --offline -p rum-stdlib --test typed_slot_census -- --test-threads=1 --nocapture` passed: `test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 277.14s`.

```
S9 CENSUS production frames=461 finalizers=50 implementations=606 typed=95 macros=542 result=343 insert=199
S9 TOTAL typed=95 typed_macros=55 witnessed_sites=46; one transport map and five identity decodes removed; modes unchanged; preflip remains refused
```

`Ok(value)` and `Err(error)` are calls, so they follow the suspend arm. That keeps M0294 at `call2`, M0295 at `call3`, M0296 at `call2`, M0297 at `call5`, M0299 at `call13`, M0323 and M0324 at `call4`, M0325 and M0326 at `call2`, M0344 at `call6`, and M0350 at `call9`. The CheckAbort pending fallback still passes `outcome` so those two later calls do not land between M0296 and M0297. The success arms still pass `Ok(value)` and `Err(error)`, not the whole `Result`.

## Director gate record (2026-10-18) — S9 accepted

**Runs.**
1. agy (Gemini 4 Argon) did the conversion, census and inventories. Its quota cut it before any report, and its checkpoints were stale.
2. grk (grok-4.7) reviewed run 1, ran every gate and measurement, and wrote the report.
3. grk took continuation 2 and applied the Ok/Err split. Its quota cut it in the middle of the gates.

The director then finished the step, by user ruling. The worker corrected its four false report statements in place.

**The director's changes on top of run 3.** Each was proved in a scratch tree before it was applied.

- **CheckAbort primary site.** Back to `Err(InputRequired) => cold, outcome => continue(outcome)`. The Ok/Err split added a copy on the abort path: `checkabort-abort` and `checkabort-caught` read 18 memcpy per iteration against base's 17.
- **CatchExceptions body.** The two-step shape of HEAD's macro: `let outcome = match body_result { Err(InputRequired) => return cold(..), Ok(v) => Ok(v), Err(e) => Err(e) }; continue(handlers, outcome)`. Both `outcome =>` and the split cost one copy on the throw path (`-handler` 31 against 30, `-caught` 42 against 41).
- **WithCleanup.** `drive_with_cleanup_primary` is back to `outcome =>` and inlinable. The init path reaches it through an `#[inline(never)]` wrapper, `drive_with_cleanup_primary_after_init`. The split had grown `cleanup-body` by +304 / +336 B per level. Every variant measured, with copies (small / two-arg / init, base 11/23/12) and `cleanup-body` stack (base 50 944 debug / 19 344 release):

  | Variant | memcpy small / two-arg / init | stack debug | stack release | Decision |
  | --- | --- | --- | --- | --- |
  | split (run 3) | — | +304 | +336 | rejected |
  | two-step, inlined | 10 / 22 / 11 | 51 200 | 19 808 | rejected |
  | `outcome =>` + `#[inline(never)]` | 11 / 23 / 12 | 50 880 | 19 632 | rejected |
  | two-step + `#[inline(never)]` | 10 / 22 / 11 | 51 200 | 19 808 | rejected |
  | `outcome =>` inlinable, init through the wrapper | 11 / 23 / 12 | 50 880 | 19 280 | **taken** |

- **Derived facts.**
  - Ownership: two replays, 56 and then 9 rows, every one a pure line move. The tool, `.temp/director/gate-617/ownership-replay.py`, refuses anything else. Against HEAD the totals are still 113 rows removed and 91 added, all in S9's files.
  - Retention: the local-contract fingerprints and the C01/C02 certificate cells were refreshed with the canonical tools. Only fingerprint columns change.
  - `obligations.tsv` citations now name the same constructs they name at HEAD. They had been stale since run 1, because the test only checks that the field is non-empty. CheckAbort `abort_cleanup.rs` 114/126/177/178 → 115/127/175/176; WithCleanup `cleanup.rs` 80/85 → 79/84.

**Gate on the final tree** (`.temp/director/gate-617/`, `chain-3.txt`):
- Formatting and units: `cargo fmt --check` clean; typed 24, frame_root_trace 15, resume_evaluation_context 11, depth 1.
- Oracle: 369 in both builds, fault delta 0 (observations 93 114).
- Census: 606 implementations, 95 typed, 55 typed macros, in both builds. Compile controls pass in both builds.
- Inventories: ownership 31; local acceptance in both builds; rejection census.
- Workspace: 3801/0/59, run natively with no skip variable, so the nine ZeroMQ tests really ran.
- Builds and allocations:
  - freshness 4/4;
  - 164 ordinary labels equal;
  - 27 S9 resume workloads at +24 B per newly typed transfer;
  - `resume-cleanup` intercept +1 allocation, +24 B.
- **Copies:** the director's census (`callgrind/census/`) has libc symbolized and counts real `__memcpy*` and `__memmove*` calls. 0 of 164 labels are above base.
- **Stack:** all 17 shapes are at or below base. The ten S8 shapes are unchanged. The seven S9 shapes, debug / release, in bytes per level:
  - `catch-body` −352 / −64;
  - `checkabort-body` −784 / −448;
  - `abortprotect-body` −288 / −64;
  - `cleanup-body` −64 / −64;
  - `check-body` −320 / −144;
  - `quiet-body` −320 / −112;
  - `catchexceptions-body` −368 / −128.
- Probes: s9 61, s8 48, s7 32, s6 22 and g14 23, all SAME.
- **Instructions:** after five runs per arm, every S9 ordinary label is at or below base. Callgrind shows the S9 functions' deterministic deltas are all negative; the variable rows are `Symbol::system`, `intern_text`, `HashMap::insert` and the stability Bloom.
  - The one ordinary label disjoint above base is `matcher-single-tiny`, at +6.7. That is all in libc `__memcmp_avx2_movbe`, with 1.00 calls per iteration on both arms, so it is a data and layout effect outside this family. **Accepted.**
- **Timing:** the aligned rebuild has 0 labels above base.

**Rulings.**
- Q1 is superseded: it was the copy defect, now fixed.
- Q2 is layout, settled by the aligned rebuild.
- Q3, `ExceptionResumeRoute`, is accepted: the resumed dispatcher is reached only from resume paths.
- Q4 is accepted in its final form, described above.

**Method lessons.**
- A worker's instrument must be calibrated in the worker's environment. In grk's sandbox libc had no symbols, so a name-matched copy count read a constant at every N.
- A copy fix is not done until the stack shapes are rerun. The fix that removed every extra memcpy grew `cleanup-body`, because the arms' temporaries landed in the inlined dispatcher's frame.
