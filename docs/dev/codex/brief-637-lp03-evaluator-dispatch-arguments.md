# Brief 637 — LP03: evaluator, dispatch and argument phase records

**Status:** implementation brief, staged. Director-written from the brief-624
design draft; the draft is raw material, this file is the authorization.
**Sequence:** LP01 (`3fd1885c`, contract and oracle), LP02 (`17576575`, the
record engine and queue). Test isolation (briefs 634–636) is done, so the
whole workspace runs at 8 test threads by user ruling 2026-11-01.

## Goal

Adopt the logical profiler's evaluator, fixed-point, head, dispatch, stage and
argument phases — routes `fixed-total`, `fixed-once`, `inner`, `atom-fast`,
`head`, `call-total`, `dispatch-total`, `stage`, `arg-total`, `arg-evaluate`
from `.temp/brief-624/inventory/routes.tsv` — so each logical invocation
produces exactly one record, opened at the operation and closed at the exact
consumer boundary named there, across pause and resume. Also close LP02's
shape item: `Session::enable_pending_profile` unwraps a state the type allows.

Nothing about evaluation changes. Values, errors, traces, output and the
existing (legacy) profiler recorders stay exactly as they are. The disabled
path gains no work beyond the two relaxed loads it already pays.

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md` (pitfalls; the standing inventory
  duty), `docs/dev/profiling.md`.
- `.temp/brief-624/DESIGN.md` — the contract. Read §"Contract" (lines 8–96)
  and §"Disabled-path gate and measurements for every implementation step"
  (265–328). The route tables are
  `.temp/brief-624/inventory/{routes,expanded-paths,path-sites}.tsv`; every
  site in them predates LP02, so the line numbers may sit up to ~15 lines
  late — locate by the phase string and the route rule, never by the line
  number alone.
- The engine you build on: `crates/rum-runtime/src/runtime/profiling/logical/`
  (`mod.rs` `Call`/`ProfileBoundary`, `record.rs` keys and policies,
  `queue_api.rs` `logical_frontier`/`logical_insert_boundary`,
  `segments.rs`, `context.rs`) and
  `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs`
  (`observe_boundaries`, `abandon_boundaries`,
  `crates/rum-runtime/src/runtime/suspension/core/profile_mode.rs`).
- The oracle that decides: `crates/rum-stdlib/tests/suspension_oracle/`
  (`fixtures/logical_profile/mod.rs` — `logical_counter_port` and its child
  modes; `support/logical_profile.rs`; `manifest.rs`), and its inventories
  under `tests/suspension_oracle/inventory/` and `fixtures/logical_profile/`.
- LP02's gate record at the end of `docs/dev/codex/brief-626-*.md` for the
  shape lessons that cost it four continuations.
- My falsifier, already calibrated red on the base tree at
  `.temp/brief-637/falsifier/` (see **Falsifier** below).

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` (the phase
  helpers and any engine-side addition they need; no new module)
- `crates/rum-runtime/src/runtime/resolution.rs`
- `crates/rum-runtime/src/runtime/resolution/resume_frames.rs`
- `crates/rum-runtime/src/runtime/eval_core/mod.rs`
- `crates/rum-runtime/src/runtime/eval_core/call.rs`
- `crates/rum-runtime/src/runtime/eval_call_prepare.rs`
- `crates/rum-runtime/src/runtime/eval_call_prepare/resume_frames.rs`
- `crates/rum-runtime/src/runtime/suspension/core.rs` (the shape item only)
- `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs`
  (the fixed-once completion only)
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs` (new; your
  unit tests)
- `crates/rum-host/tests/lp03_phase_rows.rs` (new; my falsifier, inserted
  verbatim from the .temp copy)
- the standing inventories named under **Inventory replay** below.

Files and areas you MUST NOT touch:

- the legacy `runtime_path_timing_start` / `record_profile_runtime_path_timing`
  calls — they stay, byte for byte, at every LP03 site; `phase-sites.tsv`
  records them and the loss ledger's boundaries are derived from it;
- `Expr`/`Symbol` representation, any public type or serde shape, codecs,
  guest encodings, semantics, error values, trace output, `crates/rum-core`;
- `fixtures/logical_profile/phase-sites.tsv` (no edit at all);
- the identity columns of `logical-profile-required.tsv` (first 16) and its
  frozen fingerprint `fnv1a64:b41ac6a5dbf0e9a4`; row count 1115, binding
  count 875, encoding-owned 240;
- `AGENTS.md`, `TODO.md`, `Cargo.lock`, other briefs, earlier scratch.

## Shape rules (read twice; a departure is a STOP)

1. **One record per logical invocation.** The operation is the one named by
   the instrumented site and its input; a native reentry of the same
   operation is not a second invocation, but a distinct recursive invocation,
   a distinct operand/iteration, or a later dispatch stage is. No record per
   helper call, no record per frame.
2. **Start exactly at the legacy start site** of the same phase string. Use
   the existing enabled gate; the legacy call's `Some` token means the gate
   is on, so piggyback on it (see the helper below) rather than loading the
   gate twice.
3. **Completion only at the route's named boundary**, with the route's own
   carrier: `Expr` for `Policy::Expr`, `Option<Expr>` for the stages,
   `PreparedCallArguments` for `arg-total`. A record must never observe a
   carrier the route does not name (e.g. a child `Ok` standing in for the
   child's error is forbidden).
4. **Pause = `Call::pause()` + `logical_insert_boundary`.** Capture the
   frontier before the operation that can suspend: `logical_frontier()` when
   the boundary observes the next carrier delivered to the frame, and
   `logical_frontier_after_consumer()` when the route's consumer is the frame
   the carrier must pass (the LP02 tests at
   `profiling/tests/queue_boundaries.rs:190-211` show the second form).
   `after_return` is `true` only for `evaluate.once_unstable` (its completion
   must sit after `complete_evaluation_return`'s resource enforcement and
   before the fixed-point continuation); every other route uses `false`.
5. **Finished exactly once.** A record either `finish`es once or is paused
   once and observed once. `Drop` must never find an unfinished record on a
   completed program: after the run,
   `Session::logical_profile_test_assert_quiescent()` must pass and no row
   may have `open` or `abandoned` non-zero.
6. **No default-path cost.** All new work is behind
   `crate::runtime::profiling::lexical_profile_enabled()`, the same two
   relaxed loads the legacy starters pay; the slow path is `#[cold]
   #[inline(never)]`. No TLS lookup, allocation, `Mutex`, `Arc` clone or
   capture construction on the disabled path. Do not add
   `if token.is_some()` branches at call sites.
7. **Keys are aggregate, never identity.** `AggregateKey::Runtime { phase,
   source }` with the phase string exactly as in `phase-sites.tsv`;
   `source` is `crate::runtime::evaluation_profile_current_source_symbol()`.
   Never key on a record id, a thread id or a guest value.
8. **Legacy recorders stay.** LP03 adds the logical layer beside them. The
   legacy output, timing stack and `phase-sites.tsv` do not change.
9. **No semantic edit.** If a change would alter whether, when or with what
   value guest code runs, it is a STOP, not a cost fix.
10. **When the exact consumer boundary cannot be named, STOP and report the
    consumer you traced.** Do not approximate a completion site; a wrong
    boundary is worse than an unadopted route.

## Engine API shape (as code)

Add to `profiling/logical/mod.rs` (names are fixed; adapt internals only):

```rust
impl Session {
    /// The only entry LP03 sites call. Disabled path: one gate, nothing else.
    #[inline]
    pub(in crate::runtime) fn logical_phase_start(
        &self,
        phase: &'static str,
        policy: Policy,
    ) -> Option<Call> {
        if !crate::runtime::profiling::lexical_profile_enabled() {
            return None;
        }
        Self::logical_phase_start_slow(self, phase, policy)
    }

    #[cold]
    #[inline(never)]
    fn logical_phase_start_slow(
        &self,
        phase: &'static str,
        policy: Policy,
    ) -> Option<Call> {
        Call::start(
            self,
            AggregateKey::Runtime {
                phase,
                source: crate::runtime::evaluation_profile_current_source_symbol(),
            },
            policy,
        )
    }
}

/// Finish helper for `Policy::Expr` routes: `Ok(_) -> Outcome::Ok`,
/// `Err(e) -> error_outcome(&e)` (LP02's existing mapping; a paused record
/// never reaches here).
pub(in crate::runtime) fn finish_logical_expr(
    call: Option<Call>,
    result: &Result<Expr, EvalError>,
) {
    if let Some(call) = call {
        call.finish(match result {
            Ok(_) => Outcome::Ok,
            Err(error) => error_outcome(error),
        });
    }
}

/// Finish helper for the stage route: `Some` -> Some, `None` -> None.
pub(in crate::runtime) fn finish_logical_optional(
    call: Option<Call>,
    result: &Result<Option<Expr>, EvalError>,
) {
    if let Some(call) = call {
        call.finish(match result {
            Ok(Some(_)) => Outcome::Some,
            Ok(None) => Outcome::None,
            Err(error) => error_outcome(error),
        });
    }
}
```

At a site, the shape is:

```rust
let runtime_profile = runtime_path_timing_start("evaluate_call.dispatch");
let logical = if runtime_profile.is_some() {
    self.logical_phase_start("evaluate_call.dispatch", Policy::Expr)
} else {
    None
};
let result = /* unchanged body */;
finish_logical_expr(logical, &result);
// legacy record_profile_runtime_path_timing(...) unchanged
```

When the body can suspend, the value that escapes is the suspension; the
record must be paused before it escapes:

```rust
let call = self.logical_phase_start("...", Policy::Expr);
let frontier = if call.is_some() {
    self.logical_frontier()              // or logical_frontier_after_consumer()
} else {
    None
};
let produced = self.operation(...);
let produced = match produced {
    Err(EvalError::InputRequired { request }) => {
        if let Some(boundary) = call.and_then(Call::pause) {
            self.logical_insert_boundary(boundary, frontier, false);
        }
        return Err(EvalError::InputRequired { request });
    }
    other => other,
};
finish_logical_expr(call, &produced);
```

`logical_frontier()` is cheap when off (`executing_epoch()` returns `None`);
keep the call outside the `if` only where the LP02 tests already do, and
never allocate for it.

## Routes

`Policy::Expr` unless the row says otherwise. "Completion" is the exact
consumer boundary; the draft's rule and `routes.tsv`'s
`proposed_completion` say why.

| Route / phase key(s) | Start (current tree) | Completion and pause |
|---|---|---|
| `evaluate.total` | `resolution.rs:324` (`Session::evaluate`) and `:339` (`evaluate_at_root`) | Every return of `evaluate_profiled_fixed_point` (`:354-411`) and `resume_fixed_point_from` (`:414`); one total span across later passes and pauses. |
| `evaluate.once_unstable` | `resolution.rs:381` | After the return's resource enforcement: `complete_evaluation_return` (`suspension/core/evaluation_context.rs:84-87`), before the fixed-point continuation; each resumed pass starts and completes its own record. Pause with `after_return = true`. Preserve the actual error, never a copied earlier `Ok`. |
| `evaluate_inner_unstable.total` / `.symbol` / `.call` / `.atom` | `eval_core/mod.rs:139`, `:142`, `:151`, `:160` | `mod.rs:169-174`, at the end of `evaluate_inner_unstable` before the return consumer; `.symbol`/`.call` pause when their child suspends, `.atom` cannot. |
| `evaluate_atom_fast_path.total` + the kind row (`stable_symbol`/`unstable_symbol`/`atom`/`call`) | `mod.rs:176-179` | `mod.rs:186-193`; only `unstable_symbol`/`call` can pause. |
| `evaluate_call.total` | `eval_core/call.rs:27` | The end of the whole call, including `EvaluateCallSuspensionFrame::resume` and the dispatch frames (`call.rs:377-460`), never at an intermediate frame. |
| `evaluate_call.head_eval` | `call.rs:28` | `call.rs:41`/`:50`, before `resume_call_after_head`; the unstable-head branch pauses. |
| `evaluate_call.resume_after_head` | `call.rs:73` | `call.rs:85-88`, after the dispatch plan run; pauses when dispatch pauses. |
| `evaluate_call.dispatch` | `call.rs:79` and the resumed loop (`call.rs:224-225`'s enclosing function) | After dispatch-loop termination, including declined stages and unevaluated restoration (`call.rs:163-188` and the resumed counterpart). |
| `call.stage.<name>`, policy `Policy::OptionalExpr` | `call.rs:126-127` and `:224-225` | Prepared-arguments success is the stage's `None` (finish `Outcome::None`, never an invented `Some`); an ordinary stage reflects the actual `Some`/`None` before dispatch proceeds (`:170-173`). A stage that suspends pauses; the resumed stage completes at the same rule. |
| `prepare_arguments.total`, policy `Policy::Prepared` | `eval_call_prepare.rs:241` | At `finish_suspended_prepared_call` (`:150`), i.e. after `FinishPreparedCallArgumentsFinalizer` (`eval_call_prepare/resume_frames.rs:66-75`); it spans pauses. |
| `prepare_arguments.evaluate_arg` | `:258` and `:429` | Before accepting the resumed argument; the second-pause route uses the `ContinueToken` at the resumed site. Pauses. |

Rules that are easy to get wrong, so they are named:

- `head` is a *child marker*, not the call: the fast stable/atom head path
  leaves its `call.total` and `dispatch` records outside this marker.
- `evaluate.once_unstable` completes **after** the EvaluationReturn resource
  check; `inner` completes **before** it. Do not swap the two.
- A `stage` success for prepared arguments is `None`, not `Some`, and not the
  `PreparedCallArguments` carrier.
- `arg-total` includes the finalizer; `arg-evaluate` does not include the
  append or the finalizer.
- `evaluate.total` is one span per top-level evaluation; the nested
  `evaluate_inner_unstable` is not another `evaluate.total`.

## The shape item (LP02's gate)

`crates/rum-runtime/src/runtime/suspension/core.rs:505-537`. The caller has
just installed the suspension and already holds it mutably at `:512-515`, yet
`enable_pending_profile` re-borrows `self` and `unwrap`s
`pending_suspension`. Give the cold helper the storage it needs:

```rust
// in begin_input_suspension, after the owner is installed:
if crate::runtime::profiling::lexical_profile_enabled() {
    if let Some(epoch) = crate::runtime::profiling::logical::executing_epoch() {
        self.pending_suspension
            .as_mut()
            .expect("just installed input owner")
            .frames
            .enable_profile(epoch);
    }
}
```

or, equivalently, `fn enable_pending_profile(frames: &mut FrameStorage)` with
the `expect` at the caller, which is the shape the brief requires: no
`unwrap` on a state the type allows, and the `#[cold] #[inline(never)]`
helper retained. The gate must stay where it is (after installation, only
when profiling is enabled); the memory-owner refreshes after it stay
unchanged.

## Inventory replay (standing duty; last step of the edit stage)

The rot guard hashes every `.rs/.py/.toml` under `crates/` (contract
fingerprints and certificate cells), and the LP01 inventories pin producer
files the same way. In this order, after the edits compile:

1. **Retention/contract replay** per `TEMPLATE.md`: regenerate the local
   contract fingerprints and the certificate cells by replaying
   `local_production_acceptance` in both builds, and reconcile the ownership
   rows. Disclose every changed row.
2. **LP01 producer replay.** `cargo test -p rum-stdlib --test
   suspension_oracle -- logical_counter_port` fails naming
   `stale producer <file>` for every edited producer file. Recompute, never
   hand-edit, column 19 of `inventory/logical-profile-required.tsv` for every
   row whose producer file you edited: `fnv1a64` over `[file, file_text]` with
   LP01's own `fingerprint` function (message = 8-byte little-endian length +
   bytes, FNV-1a 64). Update column 17 (and the matching
   `logical-profile-bindings.tsv` column 3) only where a statement actually
   moved, to the line it moved to. The phase strings, the first 16 columns,
   the row count 1115, the binding count 875 and the frozen fingerprint
   `fnv1a64:b41ac6a5dbf0e9a4` must not change. Disclose every changed row.
3. **Loss-ledger replay.** `logical_counter_port` writes
   `.temp/brief-625/actual-<hash>-<mode>.json` and
   `executed-<hash>-<mode>.json` before it asserts. Regenerate
   `inventory/logical-profile-losses.json` from those dumps with the same
   indexing procedure as `.temp/brief-625/pin.py` (read it; copy it into your
   scratch and adapt only paths). The LP03-owned cells must move from losses
   to executed receipts; every cell owned by LP04–LP07 that moves for any
   other reason is a STOP with the old and new records. Never edit the
   indexed JSON by hand.
4. `cargo test -p rum-stdlib --test suspension_oracle` must be green in both
   the default and `--no-default-features` builds.

## Falsifier (director-written, calibrated red)

Insert `.temp/brief-637/falsifier/lp03_phase_rows.rs` verbatim at
`crates/rum-host/tests/lp03_phase_rows.rs`. It asserts, for a completed
program and for one paused on `InputString` and resumed: every adopted phase
row exists, `calls = ok + err`, `abandoned = 0`, `open = 0`, at least one
`call.stage.*` and one `evaluate_atom_fast_path.*` row, the resumed run
records a pause, and `logical_profile_test_assert_quiescent()` holds.

My calibration on the base tree (2026-11-01, 8 threads, log
`.temp/brief-637/falsifier/base-run.log`):

```
test lp03_adopted_phases_are_exact_in_completed_and_resumed_runs ... FAILED
panicked at crates/rum-host/tests/lp03_phase_rows.rs:77:36:
evaluate.total missing for `f[x_] := x + 1; f[1]`: []
```

It must be green after LP03, and it is never weakened to reach green. If it
fails for a phase you did not adopt, that is a STOP, not an edit.

## Verification — Stage A (the shape and its proofs)

Stage A has two stops; report each separately.

**A1 — shape item + leaf routes** (`inner`, `atom-fast`, `head`, `stage`,
`arg-evaluate`, `arg-total`). Run and paste verbatim:

```
cargo fmt --check
cargo test -p rum-runtime --lib profiling --offline  # your unit tests
cargo test --workspace --offline --no-fail-fast      # 8 test threads
```

**A2 — spanning routes** (`fixed-total`, `fixed-once`, `call-total`,
`dispatch-total`) plus the falsifier. Run and paste verbatim:

```
cargo fmt --check
cargo test -p rum-host --test lp03_phase_rows --offline
cargo test -p rum-stdlib --test suspension_oracle --offline -- logical_counter_port
cargo test -p rum-stdlib --test suspension_oracle --offline
cargo test --workspace --offline --no-fail-fast
cargo test --workspace --offline --no-default-features --no-fail-fast
```

The 48 brief-624 correctness programs must stay semantically green with
profiling off and on (values, messages, trace; `.temp/brief-624/correctness/`
holds the fixtures and `summary.tsv` the expected rows; copy any runner into
your scratch rather than writing to brief-624). The oracle's `all` and
`timing` modes are the authority for the adopted cells; every adopted cell
must be SAME and every previously adopted cell must stay green.

## Verification — Stage B (measurement campaign)

Only after A2 is green. Read `.temp/brief-624/DESIGN.md:265-328` and run its
campaign with the director-supplied arms (base tree at `HEAD`, candidate at
your tree, one codegen unit, separate target dirs, offline, serial):

- one-codegen-unit instruction slopes for all 192 ordinary and 121 resume
  labels (`.temp/brief-624/performance/labels.json`) plus your lifecycle
  controls; `perf stat -x, -e cpu_core/instructions/u`, stdin `/dev/null`,
  pinned CPU;
- `scripts/core-language-microbench.sh` rum-vs-rum per-label `timings.tsv`
  on every hot label and same-tier neighbor, profiling OFF;
- the copy/allocation census on all 192 ordinary labels (brief-622 reducer);
- the 22 debug/release stack shapes (44 zero-residual fits);
- the compiler layout gate on every named frame/Entry/Queue/token.

The disabled path must not move: the existing gate is the bar, and any
ordinary-label regression is a defect to attribute or report, never noise.

## Stop conditions and checkpointing

- STOP and report (do not improvise) if: a route's consumer boundary cannot
  be named; a record would need a carrier its route does not name; a change
  would alter guest semantics; the identity columns of the required
  inventory would move; an LP04–LP07 loss cell moves; a new
  `UnknownHostPayload` row appears; or the falsifier fails for a phase you
  did not adopt.
- Write `.temp/brief-637/PROGRESS.md` after every stop and every green stage:
  what is done and verified with raw paths, the exact next action, the tree's
  compile/test state, changed files, and the counters (adopted keys, rows,
  receipts, losses).
- Stop-loss is keyed on progress: two consecutive attempts that advance none
  of the brief's counters (adopted keys, oracle cells, receipts, measurement
  rows) end the stage with a takeover checkpoint.

## Hard rules

- No version-control command in any form; leave the tree for the director.
- No network, no new dependency, no `unsafe`, no default-path cost.
- Keep every legacy recorder and `phase-sites.tsv` untouched.
- Full workspace runs use `RUST_TEST_THREADS=8` (user ruling 2026-11-01) in
  `scripts/memory-scope.sh` at 14G; timing, perf and callgrind runs are
  serial and alone with profiling off.
- Paste real output; say plainly what did not run. The report is
  `rum-report/1` at `.temp/brief-637/report-a1.json`, `report-a2.json`,
  `report-b.json`, and `python3 scripts/report.py check 637 --stage a1`
  (etc.) must pass before each stage's final message.
