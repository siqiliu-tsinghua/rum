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

## Director gate — stage A1 (2026-11-01): the STOP is a false positive; the bar is amended

I ran the gate (`python3 -I scripts/director-gate.py run 637 --stage a1`,
`.temp/director/gate-637a1/`): **scope PASS** (53 citations verified),
**fmt FAIL** — one call in `eval_core/call.rs:356` needs rustfmt. A1 does not
pass as delivered; the fmt fix, the replays it invalidates and A1's workspace
suite are part of the continuation below.

The loss-ledger STOP is correct under this brief as written, and the bar it
enforced is wrong. I rule it a false positive: **A2 may proceed.**

### Q1, answered: the movement is the queue's source-context restore

`Context::capture` (`crates/rum-runtime/src/runtime/profiling/logical/context.rs:37`)
records `Session::logical_capture_sources()`, and `Context::enter_active`
(`:70`) swaps that stack back in on resume. LP03 A1 inserts the first
boundaries in the tree — LP02 adopted no phase — so for the first time a
resumed frame restores the `EVALUATION_PROFILE_SOURCE_STACK` that the
suspension unwind had emptied. The `runtime|phase|<source>` keys are rendered
from exactly that stack (`profiling/tests/logical_contract.rs:352`), so
nested phases owned by LP04–LP07 are re-attributed **on the resumed arm only**
to the source the queued arm always had. A lost or duplicated legacy event
would move the resumed arm *away* from the queued arm; these cells move it
toward.

### The evidence (my reproduction, `.temp/director/brief-637-repro/`)

Scripts `cmp_ledgers.py`, `cmp2.py`, `cmp3.py`, `distance.py`,
`moved_after_moves.py`; inputs the committed LP01 ledger
(`.temp/brief-637/originals/lp01/`, md5-identical to `HEAD`) and the
regenerated one.

- After applying the six approved producer-position moves to the old rows,
  **`lexical` and `timing` are exactly identical**: 0 old-only, 0 new-only.
  Only `cache` and `all` move — the modes in which the logical epoch exists
  (`executing_epoch()` is `None` without it, `logical/segments.rs:95`), so
  only there is a boundary inserted. The suspension-macro restructuring runs
  in every mode; a movement confined to two modes cannot come from it.
- **The queued arm is untouched**: every `(mode, program, key)` present in
  both ledgers has an identical queued vector — 77,564 vectors, 0 changed.
- **The disagreement mass falls and never rises.** For each `(mode, program)`,
  `M = Σ_keys L1(resumed, queued)` over that program's loss rows (the ledger
  *is* the difference set, so this is the program's whole
  resumed-vs-queued disagreement): 3,184 cells, **610 lower, 2,574 equal,
  0 higher**; `Σ M` 1,364,160 → 725,052. Net rows: cache 21,485 old-only vs
  9,166 new-only; all 24,091 vs 9,515.
- Per-owner old-only rows (cache): LP03 14,615, LP05 4,104, LP06 1,512,
  LP04 1,226, LP07 28. The LP03 promotion and the source re-attribution are
  the same mechanism at different depths.

### Amended loss-ledger bar (supersedes every earlier LP04–LP07 bar in this brief)

`inventory/logical-profile-losses.json` is a derived inventory: it is
re-baselined whenever a stage inserts boundaries. Regenerate it with the pin
procedure, then extend `verify_losses.py` and make it print ALL of these
verdicts **PASS** (keep the existing receipt check as well):

1. **queued-unchanged** — for every `(mode, program, key)` present in both
   the old and the new ledger, the queued vector is identical.
2. **mode-confinement** — after the six approved producer-position moves,
   the `lexical` and `timing` rows are identical old to new.
3. **mass-never-up** — per `(mode, program)`, the L1 disagreement mass over
   its loss rows is `<=` the old ledger's; print the lower/equal/higher
   counts and both totals.
4. **executed-receipts** — 4,375 receipts unchanged (existing).

Any increase under 3 is a STOP. LP04–LP07 cells may move only under 1–3.
Never weaken one of these checks to reach green.

### What the continuation must do

1. `cargo fmt` (only `eval_core/call.rs` is unformatted today), then confirm
   `cargo fmt --check` is clean. Re-run A1's `profiling` suite and its
   workspace (`RUST_TEST_THREADS=8`) on the formatted text.
2. Implement **A2** — the spanning routes `evaluate.total`,
   `evaluate.once_unstable`, `evaluate_call.total`, `evaluate_call.dispatch`
   — and insert the falsifier, exactly per the route table, the shape rules
   and the consumer boundaries above.
3. Run the standing inventory replay **once on the final text**, in the
   brief's order, with the amended bar and the extended `verify_losses.py`.
   The A1 `local_production_acceptance` reds (`stale or omitted instance
   certificate C01`) were the certificate ordering artifact: the certificate
   replay ran after them. My rerun on the delivered tree is green (1 passed,
   rc=0, 33.69 s). Rerun that test in both builds after the replay and quote
   both verdict lines.
4. Run both Stage A verification sets and report `a1` and `a2` separately,
   each with its own runs, checks and pasted verdict lines. A1's report
   stands as delivered except where this continuation replaces its numbers.

## Director gate — continuation 1 (2026-11-01): the layout defect and the measured adopted-start design

The continuation left the tree red: six workspace targets fail
(`logical_profile_lifecycle`, `native_stack_thread_scope`, `rum-runtime --lib`,
`rum-stdlib --lib`, `suspension_oracle`, `typed_slot_census`). Two of the
failures are real and are the stop: the recursive evaluator frames grew.
Measured in the same debug build, symbol by symbol:

```
evaluate_call          0x2b8 -> 0x318   (+96)
evaluate_inner_unstable 0x3e8 -> 0x438  (+80)
evaluate_atom_fast_path 0x1a8 -> 0x1f8  (+80)
```

The `native_stack_thread_scope` fixtures sit on the 1 MiB limit, so this
growth flips `dive[12]` and the 80-deep suspension. **Do not recalibrate
those tests.** Off-by-default state in a hot recursive frame is a hard-line
violation (CLAUDE.md), and the bundle/`Call` local — boxed or not — is that
state. The worker's six layout attempts (boxed, serial, modes, outlined,
bundles, zst-guarded) all leave some growth; none can remove it, because a
local that survives the recursive call always costs a slot in debug.

### The design that measures zero: fold the logical start into the existing timing call

I measured this on the delivered tree (scripts and logs in
`.temp/director/brief-637-repro/`; the probe edits were reverted afterwards):

| variant (evaluate_atom_fast_path) | prologue |
|---|---|
| base function, current build | `0x1a8` |
| + two hot `phase_begin` calls + cold finish/pause | `0x1c8` (+32) |
| + two hot begins + `phase_capture` + cold finish/pause | `0x1e8` (+64) |
| **adopted start (logical open+capture inside the existing start call) + cold finish/pause** | **`0x1a8` (+0)** |

Cold-path calls are free (the `matches!` branch with two pause and two finish
calls measured `0x1a8` on its own). What costs is adding a call *at the entry*
of a recursive function. So the start must ride the call that is already
there.

**Shape (this compiles, passes `cargo test -p rum-runtime --lib profiling`
44/0/2, and measures `0x1a8`):**

```rust
// profiling/timing.rs — beside the legacy start; the legacy function itself
// is untouched.
#[inline]
pub(in crate::runtime) fn runtime_path_timing_start_adopted(
    session: &mut crate::runtime::Session,
    phase: &'static str,
) -> Option<RuntimePathTimingToken> {
    let token = runtime_path_timing_start(phase);
    if token.is_some() {
        if let Some(site) = crate::runtime::profiling::logical::adopted_site(phase) {
            crate::runtime::profiling::logical::open_phase(session, phase, site);
        }
    }
    token
}

// profiling/logical/mod.rs
pub(in crate::runtime) struct AdoptedSite {
    pub policy: Policy,
    pub after_consumer: bool,
    pub after_return: bool,
}

pub(in crate::runtime) fn open_phase(
    session: &mut crate::runtime::Session,
    phase: &'static str,
    site: AdoptedSite,
) {
    let Some(mut call) = session.logical_phase_start(phase, site.policy) else {
        return;
    };
    call.capture_frontier(session, site.after_consumer);
    PHASE_LIFO.with_borrow_mut(|stack| {
        let storage = match executing_epoch() {
            Some(epoch) => match epoch.storage(std::mem::size_of::<PhaseEntry>() + 16) {
                Some(storage) => Some(storage),
                None => { epoch.retire(Retire::DiagnosticCap); None }
            },
            None => None,
        };
        stack.push(PhaseEntry {
            phase, call: Some(call), after_return: site.after_return, _storage: storage,
        });
    });
}
```

- `PhaseEntry { phase: &'static str, call: Option<Call>, after_return: bool,
  _storage: Option<Storage> }`, one TLS `RefCell<Vec<PhaseEntry>>` (LIFO),
  **no `LogicalPhases` local and no `Drop`**.
- Exit helpers are cold calls that pop by phase: `take_phase(phase)` reads the
  top entry, checks its stored phase, pops, and on a mismatch calls
  `epoch.fault()` and returns `None`; then `finish_phase_expr/optional/
  prepared/error/ok` and `phase_pause(session, phase)` call the existing
  `finish_logical_*` / `pause_into`.
- The resume-side completions `finish_logical_stage` and
  `finish_logical_prepared_total` are unchanged and do **not** pop: the pause
  already popped that entry when it converted the record into a boundary.
- `adopted_site(phase)` is the single table: the eleven keys and their
  policies (`Expr`; `OptionalExpr` for `call.stage.*` with
  `after_consumer=true`; `Prepared` for `prepare_arguments.total` with
  `after_consumer=true`; `Expr` with `after_return=true` only for
  `evaluate.once_unstable`). A unit test must assert the table's key set equals
  the brief's adopted list, so a drift is a red test, not a silent lost row.

**The edit at each route** is one line per phase: replace
`runtime_path_timing_start(phase)` with
`runtime_path_timing_start_adopted(self, phase)` (same call count, one more
register argument), delete the bundle start/capture calls, and keep the cold
`pause`/`finish` calls at the exit. Every early return path (the dispatch
loop's `return`s, the prepared-argument `break`) must finish or pause before
it returns; the falsifier's `abandoned=0`/`open=0`/quiescence checks are the
enforcement — there is no `Drop` safety net any more.

### What the continuation must do

1. Revert the `LogicalPhases`/`PHASE_STACK` bundle and apply the adopted-start
   design to every A1 route (inner, atom-fast, head, stage, arg-total,
   arg-evaluate).
2. Before anything else, verify the three prologues are back to
   `0x2b8`/`0x3e8`/`0x1a8` with the same objdump method, and run
   `cargo test -p rum-host --test native_stack_thread_scope` — the two failing
   fixtures must pass unmodified.
3. Then A2 (the spanning routes and the falsifier), the standing replay on the
   final text with the amended loss bar, and both Stage A verification sets,
   reporting a1 and a2 separately.
4. Diagnose each of the other four failing targets before the workspace run;
   the report must name, per target, whether it was a stale inventory cell
   (replayed) or a code defect (fixed), with the verdict line quoted.

The measured probe is reproducible: the base function's `0x1a8` was
re-derived in the same build after reverting, and the profiling suite was
green on the measured variant. If the implemented version does not reproduce
`0x1a8`, stop and report the prologue with the diff, not a recalibrated test.

## Director gate — continuation 2 (2026-11-01): the 16 bytes are the exit strings

Continuation 2 ran 24 minutes, reverted the `LogicalPhases` bundle, applied the
adopted start to the A1 routes, and stopped on the prologue bar: 45/0/2 on the
profiling suite, fixtures untouched, A2 and Stage B not started. The stop is
correct and the bar stands. The measured deltas were `evaluate_call 0x2c8`,
`evaluate_inner_unstable 0x3f8`, `evaluate_atom_fast_path 0x1c8` — exactly +16
on each.

I bisected the increment in the delivered tree, in one build, against the base
function re-derived in the same build:

| variant | `evaluate_atom_fast_path` | `evaluate_call` |
|---|---|---|
| delivered | `0x1c8` | `0x2c8` |
| exit block deleted | `0x1a8` | `0x2b8` |
| exits kept, `u8` argument instead of the phase string | `0x1a8` | — |
| wrapper `#[inline]` instead of `#[inline(always)]` | `0x1c8` | — |

The wrapper, its inlining attribute, and the extra calls are all free. The cost
is the **`&'static str` phase identity at the exit**: a debug build materialises
every such argument as a 16-byte fat pointer in the caller's frame, and the exit
is a point the base function never had one at. Passing a byte-sized key instead
measured `0x1a8` with the exit calls still present.

### Shape: `PhaseKey`, the phase identity as a `Copy` enum

Text survives only where it is consumed (`runtime_path_timing_start`,
`logical_phase_start`); every call edge passes a key.

```rust
// profiling/logical/mod.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum PhaseKey {
    EvaluateTotal,
    EvaluateOnceUnstable,
    EvaluateInnerUnstableTotal,
    EvaluateInnerUnstableSymbol,
    EvaluateInnerUnstableCall,
    EvaluateInnerUnstableAtom,
    EvaluateAtomFastPathTotal,
    EvaluateAtomFastPathStableSymbol,
    EvaluateAtomFastPathUnstableSymbol,
    EvaluateAtomFastPathAtom,
    EvaluateAtomFastPathCall,
    EvaluateCallTotal,
    EvaluateCallHeadEval,
    EvaluateCallResumeAfterHead,
    EvaluateCallDispatch,
    CallStageSpecialForm,
    CallStagePrepareArguments,
    CallStageListable,
    CallStageUpValues,
    CallStageDownValues,
    CallStageSubValues,
    CallStageOrdinaryBuiltin,
    CallStageFunction,
    PrepareArgumentsTotal,
    PrepareArgumentsEvaluateArg,
}

impl PhaseKey {
    pub(in crate::runtime) const ALL: [PhaseKey; 25] = [/* every variant */];

    /// The timing key text: byte-identical to the strings in the tree today,
    /// because the loss ledger, the binding inventory and every oracle cell
    /// are keyed by them.
    pub(in crate::runtime) const fn name(self) -> &'static str {
        /* one arm per variant */
    }

    /// Exhaustive, no `_` arm: a phase cannot exist without its site.
    const fn site(self) -> AdoptedSite {
        /* one arm per variant, with today's assignment */
    }
}
```

- `AdoptedSite { policy, after_consumer, after_return }` and `Policy` are
  unchanged. `site()` keeps the current assignment: `Expr`/false/false for the
  evaluator routes, `Expr`/false/true for `EvaluateOnceUnstable`,
  `OptionalExpr`/true/false for the eight `CallStage*`, `Prepared`/true/false
  for `PrepareArgumentsTotal`.
- `adopted_site(&'static str)` and the `ADOPTED_SITES` string table are
  deleted. The unit test walks `PhaseKey::ALL` and asserts the name list equals
  the brief's adopted list; the compiler now enforces that every key has a
  site.
- `PhaseEntry.phase: &'static str` becomes `key: PhaseKey`; `take_phase(key)`
  compares keys and calls `epoch.fault()` on a mismatch, exactly as today.
- `open_phase(session, key)` resolves `site = key.site()` and calls
  `session.logical_phase_start(key.name(), site.policy)`.
- The exit helpers take `key: PhaseKey`: `phase_pause(session, key)`,
  `finish_phase_expr(key, &result)`, `finish_phase_optional(key, &result)`,
  `finish_phase_prepared(key, &result)`, `finish_phase_error(key, &error)`,
  `finish_phase_ok(key)`.
- `runtime_path_timing_start_adopted(session, key: PhaseKey)` keeps `#[inline]`
  (measured identical to `#[inline(always)]`; do not spend a build on the
  attribute) and calls `runtime_path_timing_start(key.name())`.
- The two string producers become key producers:
  `EvaluateAtomFastPathKind::phase_key(self) -> PhaseKey` and
  `CallDispatchStage::phase_key(self) -> PhaseKey`. Delete
  `call_dispatch_stage_profile_name`; its resume-side caller
  (`resume_suspended_call_dispatch_frame`) passes `key.name()` to
  `finish_logical_stage`. Delete `profile_name()` if no reader is left.
- The finish/pause ordering and the resume-side non-popping completions
  (`finish_logical_stage`, `finish_logical_prepared_total`) are unchanged.

### What continuation 3 must do

1. Convert every site to `PhaseKey` as above. No `&'static str` may remain at
   any `runtime_path_timing_start_adopted`, `phase_pause` or `finish_phase_*`
   edge.
2. Then, before anything else, measure the three prologues with the objdump
   method already in `.temp/brief-637/` and confirm `evaluate_call 0x2b8`,
   `evaluate_inner_unstable 0x3e8`, `evaluate_atom_fast_path 0x1a8`, then run
   `cargo test -p rum-host --test native_stack_thread_scope` **unmodified**. If
   the numbers are not the base, stop again and report the three prologues with
   the diff; do not recalibrate a fixture and do not weaken the bar.
3. Only then A2: wire the five remaining routes (`evaluate.total`,
   `evaluate.once_unstable`, `evaluate_call.total`,
   `evaluate_call.resume_after_head`, `evaluate_call.dispatch`) onto
   `runtime_path_timing_start_adopted`, install the falsifier at
   `crates/rum-host/tests/lp03_phase_rows.rs`, run the standing replay with the
   amended loss bar and both Stage A verification sets, and report `a1` and
   `a2` separately.
4. The per-target diagnosis duty from continuation 1 stands: name, per failing
   target, stale inventory cell vs code defect, with the verdict line quoted.

The bisection is reproducible: I deleted the exit blocks from the delivered
tree (atom-fast, and the head-eval branch in `evaluate_call`), re-derived the
base functions in the same build, and restored both files; the tree is exactly
as continuation 2 left it.

## Director gate — continuation 3 (2026-11-01): ride the record call; make the drop path teardown-safe

Continuation 3 converted the phase identity to `PhaseKey` and stopped on the
bar: `evaluate_call 0x2b8` and `evaluate_atom_fast_path 0x1a8` reproduced
base, but `evaluate_inner_unstable` grew to `0x428` (+64), and the profiling
suite went red (`44/1/2`, then a TLS abort under `--test-threads=1`). Both
findings are right and both have a measured fix.

### The prologue: an added call costs 16, an enum argument costs another 16

Measured on `evaluate_inner_unstable`, one build each, base re-derived:

| variant | prologue |
|---|---|
| base (plain `runtime_path_timing_start`, no exits) | `0x3e8` |
| legacy string start + the exit branches with **no calls** | `0x3e8` |
| legacy string start + 4 exit calls taking `u8` | `0x3f8` |
| legacy string start + 4 exit calls taking `PhaseKey` | `0x408` |
| adopted start `(self, PhaseKey)` + keyed exits (delivered) | `0x428` |

So in a hot frame **every added call costs 16 bytes and every by-value enum
argument costs another 16**, while a `&'static str` constant and a `&mut
Session` argument are free. `evaluate_atom_fast_path` had slack and absorbed
both; `evaluate_inner_unstable` did not.

A zero-growth design therefore cannot *add* a call: it must ride calls that
are already there. Both calls exist:

- the start already is `runtime_path_timing_start(phase)` — the adopted start
  with a **string** argument measured free;
- the exit already is `record_profile_runtime_path_timing(token, outcome)` —
  and it receives the token, which carries the phase.

**Measured shape** (probe edits, then restored): delete every exit call and
replace the record call with `record_profile_runtime_path_timing_adopted(self,
token, &result)`, which records timing and closes the logical phase:

| function | base | this shape |
|---|---|---|
| `evaluate_inner_unstable` | `0x3e8` | `0x3d8` |
| `evaluate_atom_fast_path` | `0x1a8` | `0x1a8` |
| `evaluate_call` (head-eval exits) | `0x2b8` | `0x2a8` |

Passing `outcome` **by value** costs the extra 16 again (`0x1b8` on
atom-fast), so the adopted record takes `&result` and computes the outcome
inside. Passing `&result` to a `#[cold] #[inline(never)]` helper is free.

### Shape

```rust
// profiling/logical/mod.rs — the completion, per result type
pub(in crate::runtime) trait PhaseCompletion {
    fn timing_outcome(result: &Result<Self, EvalError>) -> RuntimeProfileOutcome
    where Self: Sized;
    fn finish_logical(call: Option<Call>, result: &Result<Self, EvalError>)
    where Self: Sized;
}

impl PhaseCompletion for Expr {
    fn timing_outcome(result: &Result<Expr, EvalError>) -> RuntimeProfileOutcome {
        runtime_profile_outcome_for_expr_result(result)
    }
    fn finish_logical(call: Option<Call>, result: &Result<Expr, EvalError>) {
        finish_logical_expr(call, result)
    }
}
// Option<Expr>  -> runtime_profile_outcome_for_optional_result / finish_logical_optional
// PreparedCallArguments -> the shared-args mapping / finish_logical_prepared
```

```rust
// profiling/timing.rs — beside the legacy record, which stays untouched
#[inline]
pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted<T: PhaseCompletion>(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
    result: &Result<T, EvalError>,
) {
    let outcome = T::timing_outcome(result);
    if let Some(phase) = token.as_ref().map(|token| token.key.phase) {
        if let Some(key) = crate::runtime::profiling::logical::PhaseKey::from_name(phase) {
            crate::runtime::profiling::logical::complete_phase(session, key, outcome, |call| {
                T::finish_logical(call, result)
            });
        }
    }
    record_profile_runtime_path_timing(token, outcome);
}
```

- `PhaseKey` stays the internal identity (`PhaseEntry.key`, `take_phase(key)`
  with `epoch.fault()` on a mismatch) and gains `from_name(&'static str) ->
  Option<PhaseKey>` and `const ALL`. The call edges keep **strings**;
  `PhaseKey` never crosses a call boundary. `adopted_site(phase)` becomes
  `PhaseKey::from_name(phase).map(PhaseKey::site)`.
- `complete_phase(session, key, outcome, finish)` takes the entry, and when
  the outcome is `RuntimeProfileOutcome::InputRequired` calls
  `call.pause_into(session, entry.after_return)`, otherwise `finish(Some(call))`.
  A missing entry is a no-op (logical off); a mismatched top is the existing
  `epoch.fault()`.
- The `Ok(None)`-style sites that record a literal outcome get
  `record_profile_runtime_path_timing_adopted_ok(self, token)` (two
  arguments), e.g. the stable-head branch's three closes.
- `phase_pause` and the five `finish_phase_*` helpers are **deleted**; every
  adopted route's plain record call becomes the adopted one. A route is
  converted in pairs — start and record together — because the record is what
  closes the entry: a half-converted route would either leak an entry or fault
  on the wrong top.
- Sites whose deleted finish used a different result expression than their
  record must keep the close at the finish point as a dedicated cold call and
  report that site; do not silently re-point it.

### The teardown defect, and its measured fix (already applied)

`PHASE_LIFO` owns `Call`s; a thread that abandons a suspended evaluation
leaves entries open, and at thread exit their `Segment::drop` calls
`ACTIVE`/`ACTIVE_STATE`, which are already destroyed: `cannot access a Thread
Local Storage value during or after destruction` → panic in a destructor →
SIGABRT. It fired in `lp03_argument_total_spans_a_second_pause` under
`--test-threads=1`.

The fix is in the tree now (`profiling/logical/segments.rs`, five lines): the
drop path must be teardown-safe, not a no-op by accident —

```rust
pub(in crate::runtime) fn active_len() -> usize {
    ACTIVE_STATE.try_with(Cell::get).unwrap_or(0).saturating_sub(1)
}
// Segment::end
let _ = ACTIVE.try_with(|cell| { let mut stack = cell.borrow_mut(); /* body unchanged */ });
```

Measured with it: `cargo test -p rum-runtime --lib profiling` is `45 passed, 0
failed, 2 ignored` both at `--test-threads=1` and at the default eight
threads. Keep this change. Any other TLS reachable from `Call`'s drop path
must be audited the same way (`Epoch::close` is TLS-free; it was checked).

### What continuation 4 must do

1. Apply the record-riding shape above. Delete the exit helpers and every
   entry's `phase_pause`/`finish_phase_*` calls; convert each adopted route's
   record call. Keep the strings at the edges.
2. Then measure the three prologues and require `evaluate_call <= 0x2b8`,
   `evaluate_inner_unstable <= 0x3e8`, `evaluate_atom_fast_path <= 0x1a8`, and
   run `cargo test -p rum-host --test native_stack_thread_scope`
   **unmodified**. If a number is above base, stop and report the three
   prologues with the diff; do not recalibrate a fixture.
3. Run the profiling suite both at `--test-threads=1` and at the default eight
   threads; both must be green.
4. Only then A2, the falsifier, the standing replay with the amended loss bar,
   both Stage A verification sets, and the per-target diagnosis — as in
   continuation 3's instructions.

The probe is reproducible: each variant above was one build in the delivered
tree; the atom-fast and head-eval probes were reverted from
`.temp/director/brief-637-repro/backup/`.

## Director gate — continuation 4 (2026-11-01): A1 is green; the falsifier caught two things

Continuation 4's A1 gate is green and I reproduced it: all three prologues at
or below base, `native_stack_thread_scope` 8/8 unmodified, profiling 45/0/2 at
`--test-threads=1` and at eight threads. A2 wired the five spanning routes and
installed the falsifier verbatim, then stopped on the falsifier's
`evaluate_inner_unstable.symbol` row. That stop opened two findings.

### 1. My falsifier's program was wrong, not the implementation

`f[x_] := x + 1; f[1]` evaluates `f` and `1` on the fast path — a stable symbol
and an atom — so the symbol route never runs and no `evaluate_inner_unstable.
symbol` record can exist. The phase is a real producer
(`logical-profile-required.tsv` row P0509). The probe programs are now ones
that exercise every row:

```rust
("x = 1; f[y_] := y + x; f[2]", false),
("x = 1; f[y_] := y + x; f[InputString[\"p\"]]", true),
```

The `ADOPTED` list and every assertion are unchanged — nothing was dropped. The
completed program passes all ten rows. Both `.temp/brief-637/falsifier/
lp03_phase_rows.rs` and the installed `crates/rum-host/tests/lp03_phase_rows.rs`
carry this text and must stay identical.

### 2. My A2 command was missing the enable switch

Logical profiling only records when `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` is
set; without it the row vector is empty (I measured `[]`, which is also what a
broken implementation looks like). The A2 command is:

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp03_phase_rows
```

Every falsifier run in the report must paste that command and its verdict.

### 3. Behind that row there is a real defect

With the completed program fixed, the *paused* program faults:

```
Epoch::fault                       profiling/logical/epoch.rs:262
segments.rs:131                    segments::restore: !path.is_open()
context.rs:66                      Context::enter_active -> segments::restore
context.rs:63, context.rs:54       Context::enter
```

`logical_profile_test_assert_quiescent()` reports `(0, 0, 0, 0, 1)` — one
fault. It reproduces with the **original** paused program
(`f[x_] := x + 1; f[InputString["p"]]`) too, so it is not caused by the program
change; the verbatim run never reached it because program 1's row check stopped
first.

The rule `segments::restore` states is the one being broken: a captured context
must be consumed before its record's boundary closes. So on the
prepare-arguments pause path either a phase that suspends **finishes** where it
must pause, or the resume-side completion closes the record before the queued
context is entered. Find it with the falsifier as the instrument — the bar is
`faults == 0`, `open == 0`, `abandoned == 0` on both programs — and state which
phase and which ordering was wrong.

### What continuation 5 must do

1. Diagnose and fix finding 3. Do not touch the falsifier's rows, assertions or
   programs; they are the instrument, not the fault.
2. Run `RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test
   lp03_phase_rows` to green, plus the falsifier without the switch to show the
   RED it would produce.
3. Then the rest of A2: the spelling of the five routes is already in; the
   standing replay with the amended loss bar (queued-unchanged,
   mode-confinement, mass-never-up, executed-receipts 4,375), both Stage A
   verification sets, the per-target diagnosis with quoted verdict lines, and
   `a1`/`a2` reported separately. Then Stage B.

## Director gate — continuation 5 (2026-11-01): the stop is right; the carrier cannot cross

Continuation 5 fixed finding 3's instance and stopped on three blockers. I
reproduced what matters; the stop is accepted, but one of its claims is a
ruling question and its workspace evidence is stale.

### What I reproduced

The falsifier is green with the switch (rebuilt from the final tree):

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 cargo test -p rum-host --test lp03_phase_rows
test lp03_adopted_phases_are_exact_in_completed_and_resumed_runs ... ok
```

`logical_native_lifecycle` is red on the **final** tree — the case is the
interleaved moved-session block at `:246-255`:

```
panicked at crates/rum-host/tests/logical_profile_lifecycle.rs:55:5
  left:  [2, 0, 0, 0, 0, 0, 0, 0, 2, 0]
 right:  [2, 2, 0, 0, 0, 0, 0, 0, 0, 0]
rum-profile logical epoch=1074 valid=false reason=InstrumentationFault
```

gdb on that child, breakpoint `Epoch::fault`:

```
#0 Epoch::fault
#1 ProfileBoundary::observe              logical/mod.rs:945
#2 ProfileQueue::observe_boundaries      profile_queue.rs:275
policy = Policy::OptionalExpr            $1 = RecordId(1124)
```

Record 1124 is the `call.stage.ordinary_builtin` record, created at
`resume_call_with_dispatch_plan` and faulted when the session resumes on the
spawned thread. The A2 nested-cleanup stop names the same record.

**The report's workspace evidence is stale.** `final-workspace-default-2.log`
is timestamped 09:25; five sources changed after it — `profile_queue.rs` 09:31,
`logical_profile_lifecycle.rs` 09:32, `resolution.rs` 09:36, `logical/mod.rs`
09:47, `arena.rs` 09:49. `typed-macro-identity-stop` cites that log. From now
on a workspace claim must come from a run made after the last edit, with its
command and its timestamp.

### Why the boundary cannot complete — the source, not a guess

```rust
// suspension/typed/registry.rs:24
impl Codec for Option<Expr> { type Mode = LegacyMode; }
// suspension/typed/registry.rs:328
impl Codec for PreparedCallArguments { type Mode = LegacyMode; }
```

`LegacyMode` encodes to `Slot::Value` (`typed/channel.rs:101`), and
`ProfileBoundary::observe` accepts `Policy::OptionalExpr` only as
`Slot::State(ContinuationState::OptionalExpr(_))` (`logical/mod.rs:934`);
anything else faults (`:944-947`). The flip to `StateMode` is refused until
S54 — `preflip_no_legacy_transport_uses` asserts that refusal.

So a paused record whose policy is `OptionalExpr` or `Prepared` **can never be
satisfied**. Whether it faults is decided only by whether the queue reaches
it. In the falsifier's programs it does not: I traced every observation at
`observe`'s entry — 24, all `Policy::Expr`. In the nested-cleanup and
moved-session cases it does.

### The ruling

1. **A record may pause into the queue only when its carrier crosses the typed
   channel — `Policy::Expr`.** An `OptionalExpr` or `Prepared` record
   terminates at the pause, finished with the `InputRequired` outcome, which
   the counts map to `err` (`error_outcome`, `logical/mod.rs:89-97`). This is
   what `runtime-path-timing` already records: one `input_required` terminal,
   and a new call on resume.
   - `call.stage.*`: finish at the pause. The resumed dispatch already opens a
     fresh stage record at the top of its loop, so the resumed half is the
     second record.
   - `prepare_arguments.total`: the resumed half is the finalizer's
     (`finish_logical_prepared_total`). Choose between a record-riding
     completion and a fresh open on the resume side, and prove the choice with
     the falsifier — the paused program must still show `ok > 0` for that row,
     `open == 0`, `abandoned == 0`.
   - Make the rule structural: a debug assertion or a test that every inserted
     boundary's policy is `Expr`.
2. **Delete what the rule makes dead.** A record that never pauses needs no
   frontier: `captures_at_start()` for `CallStage*`, `AdoptedSite::after_consumer`,
   `Call::frontier`/`_frontier_storage`/`capture_frontier` and
   `logical_frontier_after_consumer` should disappear if nothing else uses
   them. Do not leave `#[allow(dead_code)]` to hide a survivor — the compiler
   is the proof.
3. **The `arena.rs` insertion walk**: prove it with a named check or remove it.
   My control — reverting exactly that hunk, rebuilding, rerunning
   `logical_native_lifecycle` — changed nothing (same `[2,0,…,2,0]`).
4. `logical_native_lifecycle`, `logical_native_nested_pause_counts` and
   `logical_native_source_scope_survives_resume` must pass. The moved-session
   case is the class-wide detector: an epoch fault is a defect, not a bar.
   `pauses_for("native")` is an acceptable scoping of the LP02 counter and
   stays.

### Inventory rulings — mine to make, not the worker's

- **Retention frontier: accepted at 1074 (10/1028/36).** I diffed the dump
  against `gate-state/587d0bff…/frontier.tsv`: 76 new rows, 41 gone. Every new
  row is an unresolved site in the three edited files — 72 RET002
  (method-syntax, closure, trait call), 3 RET003 (macro expansions) and one
  RET001 (`resolution.rs:346`, `Vec::new`, replacing a removed one). RET001
  stays at 10, so no new allocation appears. Regenerate it with the census;
  never hand-edit, and disclose every changed row.
- **Typed-slot M receipt: a move, not a defect — and only if witnessed.** The
  seven macro expansions moved into `#[cold] #[inline(never)]` helpers because
  the hot-frame rule requires it; the receipt records the caller identity, so
  the move shows up there by construction. Accept the seven new identities
  **only if** `witnessed::insertion` covers each by an existing transition;
  otherwise STOP naming the identity and the missing witness. Edit
  `typed-slot-stages.tsv` only: add the new rows, mark the old ones `removed`,
  and leave `exact()`, the counts 629/607/62/14/10/90/134 and
  `shadowing_a_witness_in_a_cold_helper_is_red` untouched.

### What continuation 6 must do

1. Apply the pause rule and delete what it makes dead. Falsifier green with the
   switch, RED without it; `logical_profile_lifecycle` 3/3.
2. Re-run the workspace **after the last edit** and paste the command, its
   timestamp and the failing-test list (empty).
3. Then the rest of A2: the standing replay with the amended loss bar
   (queued-unchanged, mode-confinement, mass-never-up, executed-receipts
   4,375), both Stage A verification sets, the per-target diagnosis with quoted
   verdict lines, and `a1`/`a2` reported separately. Then Stage B.

## Director gate — continuation 6 (2026-11-01): the carrier decides, not the policy

Continuation 6 (kimi, 10:30–11:52, log `.temp/logs/kimi_brief_637.log`, 1518
lines, exit 0, git-attempts 0) applied the pause rule, removed the arena walk
and part of the frontier machinery, and STOPped. I reproduced its reds myself
before treating its report as evidence; one of its three STOPs rests on a
condition I wrote wrong, and I correct it here.

### What I reproduced myself

- **The census is red, and it names the seven helpers.** My run
  (`cargo test -p rum-stdlib --test typed_slot_census --offline -- --exact
  source_baseline_matches_earned_receipt`, 32.5 s) reports
  `M receipt mismatch: expected 603, discovered 603`, seven `missing:` — the
  old inline macro sites in `evaluate_call`,
  `resume_call_with_dispatch_plan` ×2, `resume_suspended_call_with_dispatch_plan`
  ×2, `evaluate_profiled_fixed_point`, `resume_fixed_point_from` — and seven
  `unrecorded:` identities, the cold helpers. The tree cannot be committed.
- **The remaining fault.** gdb, breakpoint on `Epoch::fault` in
  `lp03_phase_rows`: `#1 finish_logical_prepared_total` (`logical/mod.rs:253`)
  ← `#2 FinishPreparedCallArgumentsFinalizer::finish`
  (`eval_call_prepare/resume_frames.rs:75`) ← `#3 adapter::finish` ←
  `#4 drive_frames`. The finalizer reads the TLS ancestry for a
  `prepare_arguments.total` record that is not there.
- **The pause counter.** `logical_native_lifecycle` fails at
  `logical_profile_lifecycle.rs:196` with `left: 2, right: 1`;
  `logical_native_nested_pause_counts` at `:100` with `left: 5, right: 2`.
  `Totals::child` (`logical/totals.rs:17`) adds the child's `pauses` into the
  parent's totals, and `report` writes that into the parent's row, so a phase
  row counts every descendant pause. The LP02 bars were written against
  own-record pauses and are right.

### Correction: my witness condition could never pass

`witnessed::insertion` proves a **call or method** use. `visit_expr_match`
(`typed_slot_census/witnessed.rs:246-262`) matches the site's line/column
against a call's callee span inside an `Err(EvalError::InputRequired { request
})` arm. A `resume_typed_or_suspend!` use site is a macro path (`scan/sites.rs`
`:323` records `mac.span()`); no call starts at its position, so `insertion`
is false for every macro site by construction — which is exactly why `s6.rs:80`,
`s8.rs:102` and `s9.rs:107` exempt `s.name == "resume_typed_or_suspend"` from
it. Continuation 6's probe therefore asked an unsatisfiable predicate of all 92
macro expansions in those seven functions. Its "no witness" STOP is my error,
not its finding. The probe is deleted, correctly, and must not be reinstated.

### Ruling: a cold helper carries the witness; it does not re-enter the macro

The seven helpers take `request` and re-enter `resume_typed_or_suspend!` with a
fabricated `Err(EvalError::InputRequired { request })` and a dead `ok(_) =>
unreachable!()` (e.g. `eval_core/call.rs:35-44`). `slot_witness` is
`PhantomData` (`suspension/typed/witness.rs:11-13`), so the fabricated value
type-checks and proves nothing: the receipt would record a match the helper
never performs.

The census already has the sanctioned shape for this exact extraction — S5's
producer-to-cold-helper route (`witnessed.rs:112-121`) — with seven exemplars
in `kernel_data/control_data/cleanup.rs:257-295`:

```rust
#[cold]
#[inline(never)]
fn suspend_x(
    session: &mut Session,
    /* frame parts */
    witness: SlotWitness<I>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(session, request, Frame { .. })
}
```

Rewrite all seven. The caller keeps its `Err(EvalError::InputRequired {
request })` arm, binds `let witness = slot_witness(&produced);`, and passes
`witness` then `request` as consecutive arguments. Two sites used the macro's
`evaluation_return =>` arm (`resolution.rs`'s `evaluate_profiled_fixed_point`
and `resume_fixed_point_from`); they call
`witness.suspend_evaluation_return(session, request, frame)` and keep that
semantic.

`reaches_insertion` must learn that terminal: today it accepts only method name
`suspend` (`witnessed.rs:125-136`). Add `suspend_evaluation_return` under the
same discipline — receiver is the witness, `args[1]` is the request, three
arguments — and ship its falsifier: a wrong receiver or a wrong request must be
red. The existing three witness forced-reds stay untouched.

The M receipt therefore moves seven rows from the macro family to the call
family: exactly −7 macro and +7 call identities, every other row byte-identical.
My earlier "leave `exact()` and the counts untouched" is amended for this delta
only: the `exact()` totals and the `baseline()` tuple (production
461/50/610/542/343/199, test-only 11/5/16/11/3/8) change by exactly that, and
the report prints both tables before and after. Any other movement is a STOP.

### Ruling: no adopted phase crosses a pause

Continuation 6 kept `complete_phase`'s ride branch (`logical/mod.rs:771-776`): a
`Policy::Expr` record `pause_into`s the queue. That is why the outer chain is
left open — after the resumed run `evaluate.total`, `evaluate.once_unstable`,
`evaluate_inner_unstable.total/.call`, `evaluate_call.total/.resume_after_head/
.dispatch` each read `calls=2, ok=1, open=1`, and quiescence reports
`faults=1`.

The policy says the channel *can* carry a record; it does not say a frame
*does*. The Rust frame that opened these records is gone after the suspension:
the resumed drive re-enters at the deepest typed frame and returns straight
out, so no `complete_phase` runs for the outer phases on resume. The LP02 test
span rides only because it inserts its boundary explicitly
(`logical/mod.rs:297-316`) and the queue observes the resumed completion.

So: **every one of the 25 `PhaseKey` sites closes with `Outcome::Error` on
`InputRequired`; none inserts a queue boundary.** Delete `captures_at_start`,
`Call::capture_frontier`/`pause_into`/`frontier`/`_frontier_storage` and the
`Policy::Expr` branch of `complete_phase`; the compiler must show they are
dead, with no `#[allow(dead_code)]`. The LP02 span keeps its own explicit path.

Each row already carries an `ok` record that is not the paused one, so
`calls == ok + err` and `ok > 0` hold as soon as the pre-pause record closes
with `err`.

`prepare_arguments.total` is `Policy::Prepared` and already finished that way,
so `finish_logical_prepared_total` is dead under the rule: delete the function
and its call at `resume_frames.rs:75`, keep the finalizer's own work.

### The LP04 movement is not yours to accept

The pin stops at `E1.0258.return_boundary.loop-store-reset` with four keys
outside the indexed set: `rule-rhs|request|target`, `rule-rhs|request|-`,
`rule-rhs|fast_clone_empty|-`, `rule-rhs|fast_clone_empty|target` (sites
`substitution/binding_substitution.rs:128,133`). LP04 owns them. Before any
regeneration, attribute each to its producer and consumer with `path:line` and
the call edge that changed; if an attribution depends on anything other than an
LP03-adopted phase, STOP and name it. I accept or reject after reading it.

### What continuation 7 must do

1. The two corrections and the seven-helper rewrite, in one tree.
2. Bars in this order — the derived inventories replay before the acceptance
   suites:
   - `cargo fmt --check` green (red today at `logical/segments.rs:128`);
   - falsifier green with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, RED without;
   - `logical_profile_lifecycle` 3/3 with the `pauses_for("native")` bars
     unchanged (1/2/2 in the first block, 2/4/4 and 2/3 in the nested one;
     the scoping edit already in the tree is mine and stays);
   - `rum-runtime --lib profiling` green, including the new evaluator tests;
   - `typed_slot_census` green with the amended receipt, `exact()` strict, all
     nine forced-reds red;
   - the standing replay with the unchanged loss bar (queued-unchanged,
     mode-confinement, mass-never-up, executed-receipts 4,375) and the LP04
     attribution;
   - workspace green, run after the last edit, with command and timestamp.
3. Staged stops: stop if the falsifier cannot go green without touching an
   LP02 bar; stop if any census count moves beyond the ±7; stop at the LP04
   attribution if it is not attributable to an adopted phase.
