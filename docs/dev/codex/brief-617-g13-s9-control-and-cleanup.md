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
