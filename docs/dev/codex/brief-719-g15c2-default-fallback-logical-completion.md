# Brief 719 — G15-C2: the Default fallback logical completion

**Type:** implementation. **Scope:** the direct-default phase's completion
frontier and the fallback continuation's deferred definition boundary, in
`crates/rum-runtime/src/runtime/definitions/application/down_values*` and
the enabled-only logical completion helpers, plus new profiling tests and
the standing inventory replays. No language-visible change; no
default-path cost.

## Goal

Make the logical profiler record the same ten counter fields for the
`Default` direct-lookup/fallback path after input resumption as on the
queued path: the `down_values.direct_default_lookup` phase classifies the
direct lookup's own actual result (`Some`, `None` or the error) at the
frontier where that result is produced — never the fallback's result — and
every definition record opened by guest work inside a resumed fallback is
published and closed at its actual completion, leaving no terminal open
record. Language observations, the suspension sequence, cancellation and
abandonment are unchanged. Brief 623 continuation 4 found this on row
P0248; C2 cannot close before it is repaired.

## Context pointers

- The finding and its minimization: brief 623, "Director continuation 5"
  and "Continuation 4 — execution report"; `.temp/g15-c2/c4/port/loss-evidence.json`
  (36 frozen bindings, 8 minimized cases), `c4/loss-minimization-plans.json`,
  `c4/loss-verdict.log`, raw logs `.temp/g15-c2/logs/c4-structured-real-1.log`
  and `.temp/g15-c2/logs/c4-optional-real-1.log`.
- The source: `crates/rum-runtime/src/runtime/definitions/application/down_values.rs:67-98`
  — `runtime_path_timing_start_adopted("down_values.direct_default_lookup")`
  opens the phase, `record_profile_runtime_path_timing_adopted(..., &direct_default)`
  records the direct result, and the `Err(InputRequired)` arm calls
  `default_fallback::suspend` without forwarding the token;
  `.../down_values/default_fallback.rs:21-49` — `DirectDefaultFallbackFrame::resume`
  consumes the direct `Option` and applies the fallback in one action, so
  the phase is closed later by the suspension machinery with the fallback's
  output.
- The completion machinery: `crates/rum-runtime/src/runtime/profiling/timing.rs:152`
  (`runtime_path_timing_start_adopted`), `:240` (adopted boundary),
  `:670-810` (adopted complete / defer-pause / ok / none);
  `crates/rum-runtime/src/runtime/profiling/logical/mod.rs:1500-1535`
  (`pause_call_stage_at_finish_slow`, `publish_stashed_phases_slow` ->
  `flush_definition_boundaries`); `profiling/logical/completion.rs:22`
  (`flush_definition_boundaries`); `profiling/logical/queue_api.rs:8`
  (`logical_insert_down_values_before_consumer`) and `:41`
  (`logical_frontier`).
- The reference pattern for an adopted phase that pauses: the call-stage
  loop in `crates/rum-runtime/src/runtime/eval_core/call.rs:232-295` — the
  token is recorded as `InputRequired`, the phase is completed or deferred
  through the boundary helpers, and the resumed continuation starts its own
  phases. Follow that shape rather than inventing a transport.
- The campaign instrument (**read-only; copy before use**):
  `.temp/g15-c2/c4/source-structured-real` + `structured-plans.json` for
  the two minimized witnesses and five controls;
  `.temp/g15-c2/c4/source-optional-real` + `optional-plans.json` for the
  36 frozen P0248 bindings; the runner recipes are `c4/structured_batch.py`
  and `c4/optional_batch.py` (copy them into your scratch and point them
  there). The six variables are
  `RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0`,
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, `RUM_PROFILE_BUILTIN_TIMING=1`,
  `RUM_PROFILE_DEFINITION_RULE_TIMING=1`,
  `RUM_PROFILE_RULE_RHS_TIMING=1`, `RUM_PROFILE_EVALUATION_CACHE=1`;
  `C2_ONLY=<id>` selects one case. Build with `scripts/probe-build.py`
  (stable `.temp/probe/active`, content-addressed cache, one `--force`
  sha256 spot-check per round).
- Standing exclusions, restated: the four internal-encoding G15 cases and
  the four invalid G15B cases (G15B.12/.13/.15/.16) are not witnesses; do
  not add an internal-encoding probe; the width4000 labels stay excluded;
  the rejected generated-lock recipe stays rejected (offline, locked
  builds only).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`;
  - `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs`;
  - `crates/rum-runtime/src/runtime/profiling/timing.rs`;
  - `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`;
  - `crates/rum-runtime/src/runtime/profiling/logical/completion.rs`;
  - `crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs`;
    the last five only as far as an existing explicit frontier cannot
    express the correct position, and every use is disclosed;
  - new tests under `crates/rum-runtime/src/runtime/profiling/tests/` and
    test-cfg fixture files strictly required by them;
  - the standing inventory files the maintenance duty requires (canonical
    replay only, never a hand edit).
- Files you MUST NOT touch: public types, serde shapes, host/slot/value
  APIs, anything in `rum-core`, any other runtime module, existing test
  expectations, `Cargo.toml`/`Cargo.lock`, docs. A change needed outside
  this list is a STOP with the file and the reason.
- **Class duty (first deliverable, reported as a table).** Inventory every
  `runtime_path_timing_start_adopted*` call and every definition-rule
  adopted start whose pause path chains additional computation before its
  own completion, and every `defer_definition` boundary with its flush
  sites. One disposition per entry: repaired under this class, already
  correct (named reason), or STOP with the row and the missing ingredient.
  A new call edge that changes when or whether a record completes is in the
  class.
- No new `unsafe` (the crates stay zero-unsafe), no new `#[allow]`, no
  duplicated bodies, no flattened representation, no lazy init, no
  `#[cfg(debug_assertions)]` gate, no silent drop, no invented receipt, no
  new internal-encoding probe, no test expectation weakened to make the
  witness pass. Do not turn an open record into a terminal record during
  serialization; close it at its actual completion.

## Target shape and invariants

The direct-default branch must hand the adopted token into the suspension,
and the resumed frame must complete the phase with the incoming direct
result before any fallback work; a further pause leaves the phase paused,
not closed. The fallback's RHS pause must publish its deferred definition
boundary onto the new pending suspension at the RHS frontier and close it at
its actual completion. My calibrated shape (a different route is acceptable
only if every bar below holds; if no route can hold them without a language,
suspension or cost change, STOP and report):

```rust
// down_values.rs — the InputRequired arm forwards the token into the suspension
Err(EvalError::InputRequired { request }) => {
    return default_fallback::suspend(
        self,
        &identity,
        symbol,
        down_values_cannot_match,
        runtime_profile_default,
        witness,
        request,
    );
}

// default_fallback.rs — the frame completes the phase with the direct lookup's
// actual result, once, before the fallback; a second pause keeps it paused.
fn resume(self, session, outcome, token) -> TypedAction<Option<Expr>> {
    // complete down_values.direct_default_lookup with `outcome` here;
    // never with the fallback's Some.
    match outcome {
        Ok(Some(value)) => TypedAction::complete(Ok(Some(value))),
        Ok(None) => {
            // apply the fallback; if its RHS pauses, publish and join the
            // deferred definition boundary on the new pending suspension,
            // then close it at its actual completion.
        }
        Err(error) => TypedAction::complete(Err(error)),
    }
}
```

Invariants (a change to any of these is a STOP):

- The phase's row is one record per invocation (`calls=1`, one `ok`), and
  its `some`/`none`/error/return/throw slot equals the direct lookup's
  actual result on both paths; it never records the fallback's `Some`.
- Every definition record opened by guest work inside the resumed fallback
  is closed with its actual category at its actual completion; no row ends
  with `open=1`; deferred, retained and abandoned accounting is preserved.
- The language observation is identical on both paths (value, error,
  termination, messages, prints, trace, projections, snapshot); the pause
  sequence and the records are identical; cancellation, abandonment,
  prefix and queue semantics are unchanged.
- Completion is exactly once — no double completion, no second admission of
  an existing phase, no new phase for an existing invocation.
- Profiling off: the path executes no new work; the frame carries no
  off-state on the hot path; the touched functions' disabled-path
  instruction rows are unchanged at identical call counts; no new call edge
  on the disabled path. The cold `#[cold] #[inline(never)]` enabled-body
  convention and the test-cfg admission / flag-read order are preserved.
- A new suspension frame or `FrameRootTrace`/`PureFrameAbandon` type enters
  rum-runtime's closed lists (the derived inventory and the named-frame
  count); add exactly the new types, nothing else. Changing the existing
  `DirectDefaultFallbackFrame`'s fields is not a new type.

## Verification

- **Stage 1 — copied experiment, no live edit.** Copy the live tree into
  your scratch, overlay the retained campaign fixture files, and rebuild
  the probe with `scripts/probe-build.py`. Reproduce the two minimized
  witnesses red on the current source:
  - `C4.loss.decline`: resumed `down_values.direct_default_lookup`
    `[1,1,0,1,0,0,0,0,0,0]`, queued `[1,1,0,0,1,0,0,0,0,0,0]`,
    `semantic_same=true`;
  - `C4.loss.both`: the same row plus `down_value_rhs_eval` Default
    `index: 2` resumed `[0,0,0,0,0,0,0,0,0,1]`, queued
    `[1,1,0,1,0,0,0,0,0,0]`, `semantic_same=true`.

  Then build a copied repair and show both green: all ten fields equal,
  the definition record closed with the queued category, language
  observations identical. The five adjacent controls (`fallback`, `accept`,
  `direct_body`, `outside`, `fallback_twice`) stay SAME, and the 36 frozen
  P0248 bindings turn SAME with real/disabled exact parity. Save tree
  hashes, patches and raw logs. **A copied tree that cannot reach green is
  a STOP**, not a licence to guess.
- **Stage 2 — implementation and shape.** fmt; `check --all-targets`; the
  class inventory table; the new permanent witness (both recipes) red on
  the base tree and green on the repaired tree at 8 and 1 threads; the
  focused profiling set at 8 and 1 (definition, retained-pause,
  synchronous-families, publication, the unregistered-flag-read and lease
  witnesses).
- **Stage 3 — acceptance.**
  - Runtime lib: exactly `1413 passed; 0 failed; 5 ignored; 2 filtered out`
    at 8 and 1 threads, plus exactly the tests you add (each named in the
    report; the suite's own summary line is the bar).
  - Workspace: exactly `4,212 + N` passed / 0 failed / 61 ignored over 63
    binaries at 8 threads, N = the tests you add, with the seven standing
    skips; only the nine native ZeroMQ loopback EPERM failures may appear
    in your sandbox, and their serial same-binary signature must be shown.
  - Stdlib lib `1111/0/53`; the six host targets at 8/1 where gated.
  - Standing replays, canonically: contract fingerprints 68; both
    certificate cells; ownership; rejection census 1,220 (14/1,168/38);
    LP01 five modes 88,795 records with zero semantic differences; typed
    slots 10/0/1; frame-root 15/0; the four-build warning matrix 0/0/3/3.
    Replay before the first acceptance suite; a stale-cell red is an
    ordering artifact, fixed by replay, never reported as a regression.
  - Ledger controls on the campaign inputs: the positive current ledger,
    the P0248 positive case, and the six negative controls stay as the
    continuation-4 record requires.
  - Disabled-path cost gate: Callgrind on a `Default`-headed workload with
    the fourteen `RUM_PROFILE_*` variables at 0; base and candidate built
    at one stable path with one codegen unit; the touched functions' rows
    unchanged at identical call counts (no new instructions on the disabled
    path), and no whole-workload growth beyond the accepted floor. If the
    repair cannot be enabled-only, STOP.
  - Language A/B: rum-vs-rum on the two witnesses and five controls at 8
    and 1 threads, identical observations and pause sequences.
- **Report.** `.temp/brief-719/report.json` (a `rum-report/1` file), one
  entry per run with `cmd`, `log`, `exit`, `attempt`, `tests`; every verdict
  line quoted; the class table, the native/semantic receipts, the exact
  diff, the replay verdicts and the cost rows as `sections`; `changes`
  lists every touched file with line ranges and a one-line why. Final
  message is `python3 -I scripts/report.py summary 719` and nothing else.
  Stop for my gate.

## Hard rules and stop-loss

No git in any form, no network, no new `unsafe`, no new dependencies, no
fabricated receipt, no new internal-encoding probe, no unapproved
production edit, no test change outside your own new files. Keep full raw
output and every failed attempt in `.temp/brief-719/`; copy the campaign
evidence, never modify it. Builds offline and locked; cargo bounded and
serial; the worker runs no timing decision from its sandbox durations.

Stop-loss: after two consecutive executed bounded attempts with no new
witness row, no newly closed tuple, no new source fact or no new cost
measurement, STOP and report. Source-only reviews do not start the clock.
A lifecycle, error, Return, repeat-suspension, cancel or abandonment case
that regresses is a STOP with its raw rows, never a test weakening.
