# Brief 647 — LP06 A2: builtin outcomes and the Catch lifecycle

**Source stage. One stop.** The design round is brief-645; its plan is
`.temp/brief-645/plan.md` and its route/site/cell inventories are under
`.temp/brief-645/`. This brief is the director's rewrite of the A2 draft
(`.temp/brief-645/drafts/LP06-A2-builtins-catch.md`); the draft is raw
material, not authority. A1 landed at `6864588b` (brief-646); this stage
appends to that tree. The B campaign is a separate brief.

## Goal

Adopt the builtin-handler and Catch routes, preserving guest evaluation
exactly. A builtin record observes the handler's actual `Expr` result,
including handler finalizers, before optional mapping, memory validation or
caller validation. The Catch finish record freezes its incoming outcome and
stays open through form resolution, tag matching, handler input and bypass;
the body and total records cover their actual remainders. `ALL` becomes 46
with the three appended Catch keys; the builtin family uses the existing
`AggregateKey::Builtin { symbol, source, arity }` keys — no invented runtime
phase. This stage carries no performance claim; B measures the disabled
path afterwards.

## Context pointers

- `AGENTS.md`, `docs/dev/profiling.md`, `docs/dev/codex/TEMPLATE.md`.
- `.temp/brief-645/plan.md` — the design, especially the A2 attachment and
  the five Catch remainder forms.
- `.temp/brief-645/inventory/current-sites.tsv` — the literal start/finish
  lines and carriers.
- `.temp/brief-645/inventory/oracle-cells.tsv` — the 19 A2 cells, their
  expected records and assertion contracts. Read-only raw material.
- A1 precedent: `docs/dev/codex/brief-646-lp06-a1-loops-functions.md`, and
  `crates/rum-runtime/src/runtime/profiling/control_timing.rs` (the inline
  gate, `Some` destructure, bare token to a cold callee shape).
- The frozen LP01 evidence and every prior brief's `.temp/` evidence are
  read-only.

## Director rulings (decided; do not reopen)

**Q1 — the Incoming queue guard.** In
`crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs:24` the
existing `debug_assert` admits `Policy::Expr` and `Policy::ApplicableExpr`.
Add exactly `Policy::Incoming(_)` to that match. Keep the debug assertion,
the owner/generation/frontier/memory checks and every other line intact. No
removal, no bypass, no new observation mechanism. A2 only.

**Q2 — the builtin identity and its private handle.**
`AggregateKey::Builtin { symbol, source, arity }` already names the three
documented dimensions (`crates/rum-runtime/src/runtime/profiling/logical/record.rs:28`).
Use it with `Policy::Expr`; do not invent a `PhaseKey` or a runtime phase
string, and do not emit a duplicate runtime row. The boxed handle below is
private, charged diagnostic metadata and engine adoption glue, not a
semantic carrier. Keep both existing switches (lexical and builtin timing)
and their current capture and truncation behavior: arity is
`usize` truncated to `u16` at the legacy key, source comes from
`evaluation_profile_current_source_symbol`. Build no key, box, clock or
`Storage` before the relevant enabled gates.

**Q3 — the written table-only exception continues.**
`crates/rum-runtime/src/runtime/profiling/logical/mod.rs` has crossed 35k
under brief-646's written exception. This brief may append the three Catch
key variants and their three table arms only. **Move no existing type or
function in LP06** — a moved inventory identity is a STOP. The future split
(`PhaseKey` and its three tables into `logical/phase.rs`) happens after LP07
closes, in an identity-safe organization stage; restate that boundary in the
report. Do not split anything else here.

**Q4 — the checked/unchecked builtin proof.** The ordinary workspace tests
compile with `test-builtin-memory-growth` (a dev-dependency of
`rum-stdlib`), so they exercise the checked branch; a plain environment
zero cannot. Supply a standalone scratch package under
`.temp/brief-647/harness/` with:
- its own `[workspace]` table, so it is not a member of the live workspace;
- `rum-stdlib` as a **regular** dependency (not a dev-dependency);
- `rum-runtime` enabling **only** `test-continuation-slots` — no
  `test-builtin-memory-growth`, no new production feature;
- an isolated `CARGO_TARGET_DIR` under `.temp/brief-647/harness-target/`.
Print the resolved feature graph for `rum-runtime` and `rum-stdlib` and
prove the checked feature is absent. Then run the same full-key builtin
cases in fresh processes with the lexical switch at 0 and at 1, using the
existing lease/rows APIs. No public feature, no production override, no
masking. The unchecked proof is unpassed until this package runs; say so
plainly if it cannot be made to run.

**Q5 — test-owned output roots stay unchanged.** The canonical roots
(oracle LP01 observation, `.temp/brief-546/certificates`, the fingerprint
review copy, the frontier roots) are test behaviour; do not edit their path
constants. This brief's evidence and scratch go only under
`.temp/brief-647/`.

**Q6 — settled bounds.** 60/600/900/1500 seconds for the local, crate,
oracle and whole-workspace forms, as in brief-646; the same-shape workspace
run was 564 s (gate-644). The harness commands get their own bounds derived
from the same data, not invented round numbers.

**Module layout correction (director).** The draft's
`profiling/timing/builtin.rs` and `profiling/timing/control.rs` would turn
`timing.rs` into a directory module and move every ownership-inventory path
identity — a STOP. Use new sibling files instead:
- `crates/rum-runtime/src/runtime/profiling/control_builtin.rs` — NEW; the
  boxed builtin handle and its start/finish adapter.
- `crates/rum-runtime/src/runtime/profiling/control_timing.rs` — extend the
  existing sibling with the incoming starter/recorder and its frozen token.
`profiling/timing.rs` gets no new directory and no moved identity. The one
`mod` declaration in `profiling/mod.rs` names `control_builtin`.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` — the builtin start
  and `finish_builtin_dispatch` adoption only; preserve the handler,
  capability, memory-validation and mapping statements and both dispatch
  arms.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs` —
  Catch instrumentation only. Preserve every Return/Throw body and every
  continuation/producer statement, especially P0817/P0818 at lines 131 and
  144; do not relabel historical PendingCurrentExecution metadata.
- `crates/rum-runtime/src/runtime/profiling/mod.rs` — one `mod` declaration
  for `control_builtin`; no other change.
- `crates/rum-runtime/src/runtime/profiling/control_builtin.rs` — NEW; the
  private boxed handle and its start/finish adapter.
- `crates/rum-runtime/src/runtime/profiling/control_timing.rs` — add the
  incoming starter, recorder and frozen token beside the A1 forwarded
  helper.
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — append the
  three `PhaseKey` variants and their `ALL`, `name` and `from_name` arms;
  declare the test module if one is needed. No existing type or function
  moves (Q3).
- `crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs` — ONLY the
  `Policy::Incoming(_)` arm of the debug assertion (Q1).
- `crates/rum-runtime/src/runtime/profiling/tests/evaluator.rs` — append the
  three exact site names; `ALL` becomes 46.
- `crates/rum-runtime/src/runtime/profiling/tests/control.rs` — extend the
  A1 native tests with the A2 intermediate-state cells.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/mod.rs`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/control.rs`
- `crates/rum-stdlib/tests/suspension_oracle/support/logical_profile.rs`
- `crates/rum-stdlib/tests/suspension_oracle/manifest.rs` — additive
  coverage only; the frozen 1115/875/240 identity and its fingerprint stay.
- The standing-inventory regeneration outputs named below, under their
  existing paths, only as the replay commands generate them.

Files you MUST NOT touch: `branching_loops/resume_frames.rs`,
`profiling/timing.rs` (no directory conversion, no moved identity),
`eval_dispatch/**` beyond the named start/finish block, `profiling/logical/epoch.rs`,
`profiling/logical/record.rs`, every semantic frame type/field, `Cargo.toml`/
`Cargo.lock` in the live workspace, `docs/**` except this brief, the frozen
LP01 plan/requirement identities, `phase-sites.tsv`, and every prior brief's
evidence under `.temp/`. No new `unsafe`. No new public API, feature, serde
shape or host payload.

## Deliverable

### 1. The three keys

Append, in this order, after `FunctionApplyEvaluate` (index 42), keeping
every prior index and the existing order:

| Index | Variant | Site string |
|---|---|---|
| 43 | `CatchEvalTotal` | `catch.eval.total` |
| 44 | `CatchBodyEvaluate` | `catch.body.evaluate` |
| 45 | `CatchResumeAfterBody` | `catch.resume_after_body` |

`ALL` becomes `[PhaseKey; 46]`. The three names join the exact site-table
assertion in `profiling/tests/evaluator.rs`; the frozen `ALL`/`from_name`
round-trip must stay exact.

### 2. The builtin adoption

At the existing start (`eval_dispatch/mod.rs:184-186`) replace the inner
builtin timing start; keep the outer `builtin_symbol.map(...)` availability
logic. At the existing `finish_builtin_dispatch` entry, before
`resume_typed_or_suspend!` appends `freeze()`, record the adopted boundary
against the actual `produced` result. Replace only the two old boolean
legacy call sites; the legacy recorder must run exactly once per path with
the same value it saw before.

The handle is private and boxed:

```rust
pub(in crate::runtime) struct AdoptedBuiltinTimingToken(Box<BuiltinRecord>);
struct BuiltinRecord {
    legacy: Option<BuiltinTimingToken>,
    call: Option<Call>,
    storage: Option<Storage>,
}
impl Session {
    pub(in crate::runtime) fn builtin_timing_start_adopted(
        &self, symbol: &Symbol, arity: usize,
    ) -> Option<AdoptedBuiltinTimingToken>;
    pub(in crate::runtime) fn record_profile_builtin_timing_adopted_boundary(
        &mut self, token: Option<AdoptedBuiltinTimingToken>,
        result: &Result<Expr, EvalError>,
    );
}
```

The inline wrapper checks the lexical gate and destructures `Some(token)`
before its `#[cold] #[inline(never)]` bare-token callee. The adopted logical
row exists only when the legacy builtin timing gate is also enabled, so the
native `Builtin` family mirrors the legacy producer's gating. On
`InputRequired`, transfer `call.pause()` into the existing producer
frontier before the dispatch frame (the queue insertion path used by
`complete_profile_runtime_path_boundary_slow`). On any other result, finish
the call with the `Expr` completion of the actual result and forward
`result.is_ok()` to the unchanged legacy recorder. A missing spec or
handler yields no builtin invocation; the enclosing stage may independently
report `None`.

The marker must observe the handler result after its finalizer and before
optional mapping or memory validation. `WithCleanup`'s finalizer merges its
saved result before the next profile boundary is observed; the two-pause
cell proves it.

### 3. The Catch adoption

At `nonlocal.rs:81` replace the total start; it is recorded at both existing
exits (invalid arity and the final result). At `:90` replace the body start
and use the adopted `Expr` boundary at the existing record point, preserving
`slot_witness(&body_result)` ordering. At `:119` replace the
`catch.resume_after_body` start with the incoming starter, and record
through the incoming recorder at the existing legacy record point.

Body and total use the existing `Expr` completion (`some = none = 0`). A
body pause carries the total beyond `EvalCatchSuspensionFrame`; a later
form, matcher or handler pause carries it beyond the full remainder. The
finish record freezes the incoming outcome exactly where its legacy
classifier already runs and stays open through tag matching, handler input
and bypass; at the end it completes with that frozen outcome, never with
the actual result. If the actual remainder pauses, transfer the finish
call's pause; the queue guard then sees `Policy::Incoming(_)` (Q1).

Incoming shape:

```rust
impl Session {
    pub(in crate::runtime) fn runtime_path_timing_start_adopted_incoming(
        &self, phase: &'static str, incoming: &Result<Expr, EvalError>,
    ) -> Option<IncomingPathTimingToken>;
    pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted_incoming(
        &mut self, token: Option<IncomingPathTimingToken>,
        legacy_incoming: RuntimeProfileOutcome,
        result: &Result<Expr, EvalError>,
    );
}
```

The starter opens the phase with `AdoptedSite { policy:
Policy::Incoming(frozen) }` where `frozen` is computed from the original
incoming `Result` (`Ok`, `Error`, `Return`, `Throw`, `Cancelled`). The
incoming result at this start is never `InputRequired`; if it is, fault and
STOP — do not invent an outcome. On completion, if the actual result is
`InputRequired`, transfer `Call::pause()`; otherwise finish the call with
the frozen outcome. Never pass `InputRequired` to `complete_phase`'s
default `Error` branch. Preserve the legacy classifier and its record call
semantics; the legacy recorder receives the same frozen outcome.

Preserve all five Catch remainder forms: direct Ok/error, normal Throw
without form, normal Throw with form, bypass without form, bypass with
form. Resolution frames take `Expr`, match frames take
`CompiledMatchOutcome`, their whole remainders return `Expr`. Attach only
after the full remainder, never between matcher and its typed consumer.
Add no start or field to any Catch resume frame. No P0817/P0818 local
legacy loss exists to remove; both incoming Throw vectors and the frozen
requirement identity cells stay SAME.

### 4. Cells and native receipts

Implement the 19 A2 rows of `oracle-cells.tsv` and keep all 13 A1 cells
passing unchanged. The A2 set includes the synchronous builtin Ok, Return,
the no-handler absence, the `WithCleanup` primary/cleanup two-pause and the
finalizer-restored Throw; and the Catch body/handler two-pause, handler
Abort, form pause, tag-condition pause/hit/miss, bypass
form/match/no-form/miss, matcher error, Ok, invalid arity and Return.

Native intermediate snapshots are mandatory, not just final queued/resumed
equality. At Catch's handler pause the body is already one closed Throw,
and finish and total are each still open with `calls = 0`. At successful
completion finish = Throw and total = Ok; at Abort finish = Throw and total
= Error. Nested catches produce two invocations, not one ambiguous
aggregate. Assert the FULL key map (phase/source, or builtin
symbol/source/arity), required-key presence and exact counts, not just
equal empty maps. The manifest adds the 19-cell identity assertion only;
the frozen identities do not move.

### 5. Mutants (each counter assertion kills its own)

Implement the A2 falsifiers and calibrate each red through to its observer.
At least: missing completion; duplicate completion; wrong Some/None carrier;
marker after optional mapping; marker before cleanup finalization; frozen
Throw replaced by the handler's Ok or Error; premature finish at the handler
pause. A mutant may change only the candidate source and must restore
byte-identically; an assertion that cannot kill its own mutant is not
evidence. Keep the existing LP03/04/05 falsifiers and all SAME controls.
I will re-run one at the gate.

## Verification (MANDATORY — you run it; iterate to green or evidence)

Use the A1 SOURCE-* ordered command catalogue in
`.temp/brief-645/inventory/commands.tsv`, with this stage's scratch root and
the Q5 reducers. Replay fingerprints, ownership, LP01, typed-slot receipts
and both generated certificates BEFORE the first acceptance suite after
every edit. Compare rejection counts by kind/file/callee/reason and STOP on
movement. Quote every generated verdict. A red whose only cause is a stale
fingerprint, certificate, position or receipt is an ordering artifact;
never hand-edit an expected cell.

Run fmt, check, the invariant gates, profiling, `rum-runtime` in both
feature modes, `logical_counter_port` (all five child modes), the complete
oracle in both builds, the whole workspace in both builds (with the three
standing switch-only host tests skipped there and run separately with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`), and the six switch-gated host tests
in both builds. No further workspace exclusion.

**Sandbox ZeroMQ policy (same as amendment 3).** The worker sandbox denies
loopback binding. For the two workspace commands only, set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, quote the flag and every skip line, and
change no ZeroMQ source or test. The director gate runs the whole workspace
on the host without the variable, where the nine tests must pass natively.

One cargo at a time, build jobs 8, test threads 8, offline. The dispatcher
already runs this session inside a 16G memory scope; never invoke
`scripts/memory-scope.sh` yourself, and never run a cargo command outside the
dispatch scope (amendment 1). One substantive red reproduction at one
thread; a confirmed cgroup OOM gets one retry at one build job. No
performance campaign, no Callgrind, no timing numbers.

## Checkpointing — protect the scene

Track 46 keys plus the builtin family, the nine LP06 routes, the 19 A2
cells, both builtin dispatch arms, the frozen policy cases, the mutant
kills, the generated inventory changes, compile/test results and the exact
next command. After compilation succeeds, two attempts that move none of
these counters STOP and come back as a question. Preserve every failed
attempt and all reference evidence.

## Hard rules

- No version-control command in any form; no network; no `unsafe`; no
  source outside Scope; no new semantic carrier, frame, continuation
  mechanism, public API or host payload; no guest-visible change; no
  inventory identity move (except the generated positional churn).
- All scratch and evidence under `.temp/brief-647/`; every prior brief's
  evidence is read-only. Never `/tmp`.
- New mechanism, new unknown host payload, a missed second pause, an
  instrumentation fault, a count conflict, or a proposed cell that cannot
  be reached: STOP and ask; do not improvise a repair.
- Do not run the B campaign, `perf-panel`, Callgrind, stack probes or the
  microbench; no measurement is part of this stage.
- Reply to the director in the report only; the tree is the deliverable.

## Report format

Write a `rum-report/1` report at `.temp/brief-647/report-a2.json`: changed
files and lines, every run (failed discovery attempts included), exact
replay verdicts, the rejection comparison, the native intermediate-state
receipts, the builtin checked/unchecked harness with its resolved feature
graph and both switch processes, the mutant receipts (red line, restoration,
green), the size and split assessment (the Q3 boundary), the semantic
result, the unrun performance gates and any remaining question. Check it
with `scripts/report.py check` (without its version-control option) and
return its summary and the full compliance verdict lines.

---

## Amendment 1 — the command envelope is the dispatcher's scope (after the attempt-1 STOP)

**The STOP is correct in kind and is accepted.** The preflight was
`scripts/memory-scope.sh rum-647-preflight-fmt 14G cargo fmt --check`; the
wrapper exited 1 before Cargo with `Failed to connect to user scope bus via
local transport: Operation not permitted`, and the worker stopped with zero
source edits (reported: 1,575 captured files byte-identical). That is my
brief's fault, not the worker's: the Verification paragraph both required
the wrapper and forbade a nested scope, and the sandbox denies the user
scope bus, so the two halves cannot both hold.

**Ruling on Q-ENV: the enclosing dispatcher scope is the command envelope.**
`scripts/dispatch-brief.sh` runs every worker inside its own memory-capped
systemd user scope — `MemoryMax=BRIEF_MEMORY_MAX` (16G default), no swap,
`OOMPolicy=continue` — and its own header states the worker cannot create
scopes of its own. So inside this session:

- Run every cargo command **plain**: no `scripts/memory-scope.sh`, no
  `systemd-run`, no other scope call. A nested scope is expected to fail and
  must not be attempted.
- The scope is evidenced, not assumed: read `memory.max` from
  `/sys/fs/cgroup` (it must read 17179869184 bytes, the dispatcher's 16G) and
  quote it in the compliance section. The dispatch log ends with a
  `# memory-scope:` line giving peak and OOM counts; I read it at the gate.
- Keep `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`,
  `CARGO_NET_OFFLINE=true`, `TMPDIR` under `.temp/brief-647/scratch`, one
  cargo at a time, and `scripts/run-bounded.py` with the Q6 bounds. The
  test-thread and OOM-retry rules are unchanged.

The 14G figure is the director's host instrument: my gate reruns the
acceptance under `memory-scope.sh <unit> 14G` on the host. The worker
envelope stays the dispatcher's 16G; do not change `BRIEF_MEMORY_MAX` and do
not substitute a smaller or larger scope. A cgroup OOM (exit 137 with a
risen `oom_kill`) is still a STOP with one retry at one build job.

**Restated compliance bar.** `cargo-envelope` now reads: PASS when every
cargo command ran inside the dispatcher's scope, with the `memory.max` line
quoted, no nested scope attempted and no unscoped run.

**Rerun.** This is a fresh dispatch after the amendment commit; the stopped
attempt changed nothing, so there is no state to carry forward. Preserve its
report and logs under `.temp/brief-647/` as failed-attempt evidence, then
proceed with the source stage as written. Everything else in this brief
stands.
