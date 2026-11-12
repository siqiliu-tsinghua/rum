# Brief 731 — G15-C2: the enforced typed refusal for the nested top-level evaluation boundary

**Status:** implementation, staged. Design: brief 730 (two design rounds; the
2026-11-12 gate selects the enforced typed refusal and records the repair as
not installed). No live edit is authorized until stage 1 closes at a report
stop.

## Goal

Make same-Session nested top-level evaluation obey one explicit, typed
contract at an actual empty-queue input suspension. The nested entry must
finish synchronously when input is already queued, and when it would actually
suspend for host input under a nonzero active evaluation depth it must retire
its actual captured continuation through the ordered abandonment protocol and
return a distinct typed non-resumable-boundary failure. The queued path, the
top-level host request/resume routes, the documented non-resumable script/file
batch boundary, the snapshot terminal restore path and ordinary guest
capture/loading must remain unchanged. There must be no silent semantic
difference and no fabricated guest carrier. A native caller's queued
successful transformation remains; on an actual suspension it receives the
typed failure and its Rust frame is never replayed.

## Context pointers

- `docs/dev/codex/brief-730-g15c2-nested-eval-text-boundary-design.md`: the
  design, the continuation-1 STOP and the 2026-11-12 gate ruling.
- `.temp/brief-730/continuation-1/`: the copied refusal/repair evidence,
  entrypoint inventory, cost captures and the NOT-READY draft. It is
  immutable; new work gets new names under `.temp/brief-731/`.
- The selected refusal source is
  `.temp/brief-730/continuation-1/source-refusal-final` (seven production
  files, hash-pinned in `candidate-source-deltas.json`); the final public
  proof is in `refusal-final-*` logs and `final-public-audit-2.log`; the
  earlier-shape matrix is in `matrix-refusal-*` logs; the cost campaign is in
  `callgrind2-*`/`callgrind3-*` captures and `cost3-*` results.
- Source of truth for the finalizers:
  `crates/rum-runtime/src/runtime/session/session_eval.rs` (the two
  finalizers and `eval_input_with_terminated_handling`),
  `crates/rum-runtime/src/runtime/suspension/core/abandon.rs` (ordered
  abandonment), `crates/rum-runtime/src/runtime/session/session_requests.rs`
  (`reject_overlapping_evaluation_request`), and
  `crates/rum-runtime/src/runtime/suspension/error.rs` (the legacy
  `ContinuationValue for EvalError` wire).
- `AGENTS.md` and `docs/host-embedding-and-hardening.md`: the operational
  rules and the documented host boundaries.

## Selected contract (hard)

1. **The refusal point.** An actual empty-queue `InputRequired` returned to
   `finalize_top_level_eval_value` or `finalize_cycle_termination_value` while
   `evaluation_depth.get() != 0` is refused. The entry retires its actual
   captured continuation through `abandon_pending_suspension(
   AbandonReason::HelperBestEffortDiscard)` and returns
   `EvalError::NestedEvaluationInputBoundary`. No pending owner remains to
   resume; a fresh request must work afterward.
2. **The helper shape.** Exactly one private helper:

   ```rust
   #[cold]
   #[inline(never)]
   fn refuse_nested_top_level_input(&mut self, error: EvalError) -> EvalError;
   ```

   Its only new callers are the existing `InputRequired` error arms of those
   two finalizers. `finalize_top_level_control_flow` stays byte-exact
   (TestReport also calls it). No signature changes, no new frame, no derive,
   no `unsafe`, no second classification table, no hidden flag.
3. **Public surfaces.** One unit variant in each of:
   - `EvalError::NestedEvaluationInputBoundary`
     (`#[error("nested top-level evaluation cannot suspend for host input")]`);
   - `SessionRequestError::NestedEvaluationInputBoundary`, with its Display
     arm;
   - `HostFailureCode::NestedEvaluationInputBoundary`, with the host mapping
     and the `rum-jupyter` `failure_code_name` arm;
   - `session_request_error_from_eval_error` maps the new `EvalError`;
     every exhaustive match in the workspace is updated. No other public
     signature changes. This API break is allowed inside the security line
     and is named for `CHANGELOG.md` before S10; the worker does not edit
     `CHANGELOG.md`.
4. **The legacy conversion.** `ContinuationValue for EvalError` keeps the
   existing typed-only structural-fault class: the new variant is guarded by
   the same `unreachable!` invariant already carried by
   `EvalError::ContinuationSlotMismatch` ("this wire and guard are removed at
   R"). Do not invent a guest carrier, do not change the trait signature.
   Prove that the typed channel forwards the new error unchanged and that no
   supported public path reaches the guard.
5. **The reentrant-owner rule.** The helper must abandon only the
   continuation actually captured by the entry that produced the error. A
   pre-existing pending request owner is never stolen, overwritten or
   abandoned by the nested entry; it remains resumable, and a fresh request
   works afterward. Pin this with a test at eight and one threads. If the
   shape cannot distinguish the owners, STOP; do not weaken.
6. **The native-caller duty.** A native callback that maps a successful
   `eval_text` value keeps its normal queued transformation; on an actual
   suspension it receives the typed failure and its Rust frame is never
   replayed. Document this in `docs/host-embedding-and-hardening.md` and
   beside the public `Session` contract. Never claim the Rust tail survives.

## Scope

- Files you MAY modify (live, only after the stage-1 report stop):
  - `crates/rum-runtime/src/runtime/session/session_eval.rs`;
  - `crates/rum-runtime/src/runtime/session/session_state.rs`;
  - `crates/rum-runtime/src/runtime/model/requests.rs`;
  - `crates/rum-runtime/src/runtime/session/request_errors.rs`;
  - `crates/rum-runtime/src/runtime/suspension/error.rs`;
  - `crates/rum-host/src/model/outcome.rs`;
  - `crates/rum-jupyter/src/adapter/render.rs`;
  - `crates/rum-stdlib/tests/nested_public_evaluation.rs` (new);
  - `crates/rum-host/tests/adapter_contract.rs`;
  - `crates/rum-host/tests/lifecycle_buildout.rs`;
  - `crates/rum-host/tests/batch_boundary_ownership.rs` only if the named
    ownership cleanup contract requires it (disclose every line);
  - `crates/rum-host/src/session.rs` and
    `docs/host-embedding-and-hardening.md` (the selected contract);
  - the canonical generated cells in the standing inventories under
    `crates/rum-stdlib/.../inventory/*.tsv` only as the canonical replay
    emits them (never by hand; disclose every moved row).
- Files/areas you MUST NOT touch: the repair's `NestedTextCompletion`/
  `NestedTextEnvironment` files or any repair arm; any `.temp/g15-c2/c37`
  scene; any prior brief-730 evidence; `Cargo.lock`, dependencies, `Expr`/
  `Symbol` representation; profiling semantics; unrelated sources; `.git`;
  git config; `CHANGELOG.md`, `PROGRESS.md`, `TODO.md`, `AGENTS.md`,
  `docs/dev/**` other than the two named docs.
- No live source edit until stage 1 is reported green (or STOPs with the
  named blocker) and I gate it.

## Stage 1 — copied calibration (no live edit)

Assemble a fresh base copy of the live tree and a fresh candidate copy with
the selected refusal shape under `.temp/brief-731/` (new names; never
overwrite brief-730 evidence). Build both arms through the canonical builder
(`scripts/probe-build.py` wrapper; one codegen unit; one stable path; one
Cargo at a time; fresh `LOGS` per invocation; one `--force` sha256 spot
check). Then close these exact blockers:

1.1 **The final-shape matrix.** Restore the copied matrix test's missing
`c2_native` module declaration and run the complete 30-cell matrix at eight
and one threads on the exact final refusal shape. Every cell must produce an
actual typed refusal and a retired pending owner, using the original C37
plans and the original queued descriptor runner (`Checks::ALL`). The queued
reference is the frozen C37 record, compared with the calibrated predicate:
native vectors 30/30 byte-identical; semantic tokens identical after a proven
bijective internal-symbol-ID renaming (same symbol text, same multiplicity,
exactly the `Global`c37answer` 692↔681 pair in the known eight records, every
other token byte-equal; record the per-record proof). Never normalize another
token, never relabel a record, never edit the frozen reference.
1.2 **The hot cost.** Rebuild base and final refusal on the final source and
run the Callgrind identity lane (two draws, N=10/20, equal calls, one codegen
unit, same success workload, serial and pinned). Measure
`size_of::<EvalError>()`, `size_of::<SessionRequestError>()` and
`size_of::<Result<Expr, EvalError>>()` on both arms, and disassemble the
three rows `prepare_arguments_for_call_with_unevaluated_restore`,
`evaluate_inner_unstable` and `evaluate_inner_unstable'2` to attribute the
measured +1 Ir/call. Try at most two compiling layout-preserving shapes. The
bar: every changed function and those three hot rows are instruction-identical
at equal call counts on the ordinary success path. If no shape achieves it,
STOP with the size/disassembly proof and the exact residual rows for my
ruling; do not ship an unruled increase.
1.3 **The reentrant-owner case.** Construct a real pre-existing pending-owner
resume with a nested entry and pin rule 5 at eight and one threads: the
pre-existing owner survives untouched and resumes; the nested entry returns
the typed failure (or the existing overlap rejection) without abandoning the
pre-existing owner; a fresh request works afterward. If the route is
unreachable, prove that from source and pin the invariant; do not fabricate a
route.
1.4 **The legacy contract.** Test that the typed channel forwards the new
error unchanged and that no supported path reaches the legacy guard; keep the
guard text in the existing typed-only class.
1.5 **The falsifier on the exact final source.** Remove only the two helper
call arms, retaining the enum, helper and mappings. At eight and one threads
the four public parity differences and the two class differences must return
byte-identically to base; the typed-refusal tests must turn red; the ten
public queued cells and runtime host records must stay byte-identical; no
actual pending input may be invented or replaced.

Stage 1 ends with a `rum-report/1` report and a STOP for my gate. No live
edit in this stage. All attempts, including failed shapes, are retained.

## Stage 2 — install (only after my stage-1 gate)

Apply exactly the selected refusal shape from the stage-1 candidate to the
live tree. The seven production files carry only the helper, the two error-arm
calls, the three unit variants, the Display/mapping/Jupyter arms and the
legacy guard. Add the permanent tests:
`nested_public_evaluation.rs` covers the public contract — raw text/script/
file, source/Expr request, accepted source/Expr/script/file, snapshot batch
and terminal restore, direct source request/resume, nested reentrant resume,
raw lower-level APIs, guest TestReport/Get, no-input Return/Throw/Abort/
malformed source, repeated pauses and queued answers, native mapping, ordered
cleanup exactly once, no pending owner after refusal, and a fresh request
afterward. The host tests pin the adapter contract, the snapshot input
terminal path and the batch ownership cleanup. The two docs state the selected
contract and the native-caller duty. `cargo fmt --check` and
`cargo check --all-targets` are green before any suite.

No other production change. A departure from the selected shape is a STOP.

## Stage 3 — standing replay, acceptance, cost

The canonical standing replay is the last step of the edit stage, before the
first acceptance suite: regenerate the local-contract fingerprints with the
ignored canonical review generator; run `local_production_acceptance` in the
default and `--no-default-features` builds and install only its generated
certificate cells; replay ownership discovery; replay LP01
`logical_counter_port`, `typed_slot_census` and the rejection census. A
stale derived cell is an ordering artifact, not a regression; refresh it by
replay, never by hand. STOP if an existing row's identity, status,
disposition or exclusion would change or disappear, if a new
`UnknownHostPayload` appears, or if the rejection-census counts move without
my ruling (report the keyed comparison). The refusal adds no frame or derive;
a new suspension frame or `FrameRootTrace` type is a STOP.

Acceptance, once: `cargo fmt --check`; `cargo test -p rum-runtime`;
`cargo test -p rum-host`; `cargo test --workspace` with
`--no-fail-fast` at eight threads; the no-default-feature certificate gate;
the nine native ZeroMQ tests remain the documented worker-sandbox skip. Any
red is reproduced at `--test-threads=1` before it is called a defect.

Cost on the final source: the Callgrind identity lane on every changed
function and all affected hot functions, two draws, equal calls, one codegen
unit; the `scripts/core-language-microbench.sh` A/B with all labels and
same-tier neighbors, profiling off, serial; every positive row retained and
reported; one forced SHA spot check. A hot increase that no shape removes is
a STOP for my ruling, not a silent acceptance.

## Falsifier

The calibrated falsifier is the stage-1 removal of only the two helper call
arms. It is retained as `.temp` evidence, never installed. Its red set is the
four public parity differences and the two class differences, byte-identical
to base; the typed-refusal tests turn red. A falsifier that does not restore
those differences is a stop, not a green.

## Must not change / stop-loss

Do not change the queued semantics, the top-level host request/resume routes,
the script/file batch boundary, the snapshot terminal restore, ordinary
guest capture/loading, message order, output ownership, accepted-request
identity, or any `finalize_top_level_control_flow` body. Do not fabricate a
guest carrier, a value, a suspension, a trait-object state, a second
classification table or a hidden flag. Do not weaken an assertion, delete an
attempt, relabel a record or hand-edit an inventory. Do not smuggle any part
of the repair into this brief.

Stop-loss: a compiling shape is progress while it is actively reducing a
named blocker; after two consecutive compiling shape attempts that move none
of the matrix/byte/cost/reentrant counters, STOP and report. Keep the build
cap at 24 canonical builds for the whole brief; one Cargo at a time; one
forced sha256 spot check; Callgrind and microbench runs serial and alone.

## Verification (MANDATORY — run it, paste verbatim lines)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-host
cargo test --workspace --no-fail-fast
```

plus the stage-1 matrix/cost/falsifier commands and the stage-3 canonical
replay commands. Paste the exact verdict lines; a skipped or zero-test run is
not evidence.

## Report format

Write `.temp/brief-731/report.json` (or one staged report per stage) in the
`rum-report/1` schema; every run gets a log under `.temp/brief-731/` and a
`runs` entry, including failed attempts; every verdict line is quoted
verbatim; every changed file is in `changes` with line ranges and a one-line
why; the final message is exactly `python3 scripts/report.py summary 731`.
Run `python3 scripts/report.py check 731` before finishing.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Leave all changes in
  the working tree for my gate.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside the repo, no network, no subagents. The only commands
  are edits within Scope, `cargo`, the release `rum-cli`, the named
  microbench/sampling scripts and `python3` parsing.
- Testing Policy is a hard line: rum's own documented behaviour plus
  rum-vs-rum A/B only; no third-party engine comparisons.
- No new `unsafe`. If it looks unavoidable, STOP.
- All scratch goes under `.temp/brief-731/`; never `/tmp`.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything
  that could run long or spawn children; derive the bound from a prior
  measured run of the same shape, never guess.
- State the concurrency plan: one cargo at a time inside the dispatcher's
  16G scope; `-j 8`; tests at `RUST_TEST_THREADS=8`; Callgrind, timing and
  microbench serial and alone.
- Keep the tree compiling at each checkpoint; write
  `.temp/brief-731/PROGRESS.md` after each milestone with the done/verified
  state, the exact next action and any director decision needed.
