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

## Stage-1 cost gate (2026-11-13) — the STOP is well taken, the deterministic decrease is ruled not-a-cost, layout2 is selected, and continuation 1 completes the remaining copied proofs

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1d067-5238-7dd1-9d20-b364012ed5f1`), log
`.temp/logs/cdx_brief_731.log` (13,671 lines, 795,144 B); memory-scope
peak 2,920,763,392 B with oom 0 and rc 0; 192,863 tokens; 93 run
records (90 exit 0, three retained identity-red audits) and 28 executed
probe tests; `report.py check 731` 400 citations by my own run; five
canonical builds with one forced SHA spot check `435f3812…` pass; no
git, network or live edit; all 88 changes under `.temp/brief-731/`.

**My scope gate.** Working tree clean at `4ca183a4`; the live tree
compared with `source-base` differs only by the added cost-panel test;
each candidate carries exactly the seven production files as its
production delta; the 92 copied inventory files are byte-identical with
0 installed cells and 0 moved rows; the retained earlier-shape matrix
trees and the C37 scene were read, not edited.

**Evidence, reproduced by me.** I parsed all 16 Callgrind captures
independently: base / pinned refusal / layout1 / layout2 read
`prepare_arguments_for_call_with_unevaluated_restore` 1649/1655/1649/1649,
`evaluate_inner_unstable` 865/870/865/865,
`evaluate_inner_unstable'2` 173/174/173/173,
`finalize_cycle_termination_value` 29/28/28/28 and
`drop_in_place<EvalError>` 23/23/24/23 Ir per workload iteration, with
equal call counts and both draws identical. My own equal-call scan of
all `rum_runtime` rows finds no increase on layout2 in either draw; the
only positive rows outside that scope are std/test/hashbrown rows that
move in one draw only and never at a rum call count (the standing
`default_builtin_dispatch_spec_for_symbol` draw-varying row included).
The disassembly join attributes the finalizer difference to
`je +1 / jmp -1 / jne -1` — a branch inversion on the same path, not
removed work. Sizes stay 136/80/136 B.

**Ruling.** The STOP is well taken in form: the two permitted layout
alternatives are consumed and no unruled increase is shipped. Q1 is
answered yes: a deterministic decrease at equal call counts is not a
default-path cost. The bar's intent is no increase; the finalizer's
`29 -> 28` is accepted and recorded as a decrease, never normalized and
never used to offset an increase. The three hot rows are exact on
layout2, so the original blocker is closed. Layout2 is selected as the
final candidate: it removes every positive `rum_runtime` row, keeps the
sizes and the `EvalError` drop row, and consolidates the new variant
into the existing typed-only legacy guard with the original "this wire
and guard are removed at R" text, which is what item 4 of this brief
requires. The pinned seed's separate guard text is superseded by the
brief's own item 4 and stays recorded, never installed.

**Continuation 1 order (stage 1 only, no live edit).** Complete the
remaining stage-1 obligations on the layout2 source and STOP for my
gate:

1. 1.1 the final-shape matrix. Rebuild the matrix tree from the retained
   C37 real tree
   (`.temp/g15-c2/c37/source-top-level-message-checkpoint-restoration-real`),
   which carries the C37 instrumentation and the `mod c2_native;`
   declaration that the `source-refusal-final` copy lacks; apply the
   final shape's helper and its two `InputRequired` arms to that tree
   without disturbing the C37 probe code and with
   `finalize_top_level_control_flow` otherwise byte-exact. Use the
   original C37 plans and `Checks::ALL`. Every cell produces the typed
   refusal and a retired owner; the queued arm matches the frozen record
   under the calibrated predicate — native 30/30 byte-identical,
   semantic 30/30 after a per-record proof of the single
   `Global`c37answer` 692↔681 bijection, every other token byte-equal.
   If the retained tree cannot produce the matrix without fabricating
   instrumentation or plans, STOP.
2. 1.3 the reentrant pre-existing-owner probe at 8 and 1. If the route
   is unreachable, prove it from source and pin the invariant; if it is
   reachable and the helper abandons, overwrites or steals the
   pre-existing owner, STOP with the exact tuple. Never weaken.
3. 1.4 the legacy contract: the typed channel forwards the new error
   unchanged and no supported path reaches the consolidated guard.
4. 1.5 the falsifier on the exact layout2 source: remove only the two
   helper call arms, keep the enum, helper and mappings; the four public
   parity differences and both class differences return byte-identically
   to base, the ten public queued cells and the runtime host records stay
   byte-identical, and the typed-refusal tests turn red.

The ruled cost rows are not re-measured unless the shape changes; any
shape change is a STOP. Stage 2 remains blocked on my stage-1 gate.

**Recorded, not gated.** The `positive_runtime_rows` field of
`analyze_cost.py` counts differing rows, not positive ones (the
worker's own `layout2-positive-rows.json` is empty and my scan agrees);
the script exits 1 on any difference, including a decrease — the raw
rows are the evidence. The three identity reds are the literal bar
evaluations. The `default_builtin_dispatch_spec_for_symbol` row is the
known draw-varying instrument class and certifies nothing. The
`b730_nested.rs` cost panel and the retained matrix test are distinct
files and must not be interchanged in the continuation.

## Continuation-1 gate (2026-11-13) — the 1.3 STOP is well taken, the refusal becomes an entry-scoped conversion that abandons only its own capture, and continuation 2 rebuilds the revised shape

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1d083-63d3-77f1-b436-a2c4ac8aedc8`), fork of
`01a1d067-5238-7dd1-9d20-b364012ed5f1`; log
`.temp/logs/cdx_brief_731.log` (6,585 lines, 431,074 B); memory-scope
peak 2,796,318,720 B with oom 0 and rc 0; 18 continuation-1 run
records (13 exit 0; three retained exit 1: the first matrix parser,
the ownership wrapper, the first report generator; two exit 101, the
calibrated owner-probe red); `report.py check 731` 288 citations by my
own run; seven of the 24 canonical builds consumed, 17 remain; no
executed git or network; every write under `.temp/brief-731/` except
my own gate scratch.

**My scope gate.** Working tree clean at `bea71c91`; no live edit. The
matrix tree is the retained C37 real tree plus exactly 13 added lines
and 0 removed in `session_eval.rs` — the helper and its two
`InputRequired` arms, nothing else; removing those lines recovers the
C37 bytes. The other six production files in the matrix tree and all
seven in the public tree are byte-identical to `source-layout2`
(session_eval in the public tree included). Both probe binaries match
their build receipts (`db842e4c…` public, `2c3f1eb3…` matrix). The
frozen C37 scene has no file newer than the dispatch; no frozen
artifact was edited.

**Evidence, reproduced by me.** I re-ran the matrix audit; the
regenerated 60-record proof file is byte-identical to the retained one,
and the run reads 30/30 typed refusals with retired owners at both
thread counts, native queued vectors 30/30 byte-exact, semantic 22
byte-exact plus exactly the eight permitted `Global`c37answer`
692↔681 full-token bijections (multiplicity, both directions, no other
normalization), frozen reference unedited, zero zero-test runs. I
re-ran the owner probe at 8 and 1 threads and reproduced the exact
reported tuple: the nested `eval_text` returns
`NestedEvaluationInputBoundary`, its prints contain the owner's early
`owner-cleanup`, `pending=None`, the next resume fails with "no
suspended evaluation is waiting for host input", and a fresh request
returns 42; the overlap control passes at both counts.

**The defect, read from source.** The selected helper classifies only
`InputRequired` plus `evaluation_depth != 0` and then calls
`abandon_pending_suspension`. But depth is not ownership: the resume
driver `resume_eval_request_from_checkpoint`
(`session_requests.rs:283-379`) takes the `Resumable` report, runs the
captured frames, and finalizes through the same shared
`finalize_top_level_eval_value` before `finish_input_required_request`
stores the report again. Inside a nested evaluation that finalizer runs
at nonzero depth, the helper refuses and abandons the request's own
newly captured continuation, the report is never re-stored, the request
cleanup runs early, and the owner is gone. The engine's own invariant —
`SuspensionSlot::install` asserts the owner slot is empty
(`suspension/core.rs:269-285`) — means an entry's own capture exists
only when no stored request covers the slot, so ownership is decidable
from existing state (`Session::pending_request`, `model/session.rs:114`)
without a new field.

**Ruling.** The STOP is well taken. The layout2 enum placement, the
public surfaces and the cost ruling stay; the refusal contract changes
from depth-keyed to entry-scoped and ownership-preserving:

1. The refusal belongs to a nested top-level entry, not to the
   finalizer that request drivers share. The resume driver's own
   finalization is never a refused boundary: its `InputRequired` is the
   request's own next pause and is stored exactly as today. The class
   table must enumerate every call site of both shared finalizers
   (`eval_input_with_terminated_handling` and the script/file entries,
   `eval_expr_request_from_checkpoint`, the resume driver, the cancel
   path, and the loader's `finalize_cycle_termination_value` site) with
   a disposition each.
2. A nested top-level entry never returns a raw `InputRequired`; it
   converts it to `EvalError::NestedEvaluationInputBoundary` in every
   case, exactly as the public contract already rules.
3. The entry abandons only a continuation it captured itself. When the
   pending continuation is covered by a stored request report
   (`pending_request` is `Some`), the entry must not abandon, overwrite
   or take it; the request stays resumable at its actual pause,
   completes later with its own cleanup, and a fresh request works
   afterward.
4. The P1555 class is preserved: removing only the refusal conversions
   still restores the silent differences byte-identically, and the
   matrix cells still exercise the abandonment of an entry's own
   capture.
5. The revised shape must be re-measured on the standing cost lane
   (shape change): the four hot rows, the `EvalError` drop row,
   `finalize_cycle_termination_value`, the three sizes and the
   equal-call positive-row scan, two draws at N=10/20, one codegen
   unit, serial and pinned. No `rum_runtime` row may increase at equal
   calls in either draw; a decrease is recorded, never normalized; any
   increase is a STOP.
6. The class may be closed within the same seven production files; any
   additional production file, or any public surface beyond the ruled
   one, is a STOP.

**Continuation 2 order (stage 1 only, no live edit).**

1. Rebuild the matrix from the retained C37 real tree with the revised
   shape, keeping the C37 instrumentation, plans and `Checks::ALL`, and
   re-run it at 8 and 1: 30/30 typed and retired, native 30/30
   byte-exact, semantic 22 exact plus the eight bijections.
2. The reentrant owner-preservation test becomes the rule-5 witness at
   8 and 1: nested typed refusal, `after` at `owner-second`, the second
   resume completing with its own prints and 42, fresh `6*7` = 42, and
   no pending request at the end. The continuation-1 binary and its
   exact tuple are the calibrated red; the revised binary is the green.
   Add the controls that keep the ownership test from over-refusing: a
   resume that completes inside the nested entry, and a nested entry
   that completes normally while the owner stays pending.
3. 1.4 the legacy contract as written.
4. 1.5 the falsifier on the exact revised source: remove only the
   refusal conversions; the four public parity differences and both
   class differences return byte-identically to base, the ten public
   queued cells and the runtime host records stay byte-identical, and
   the typed-refusal tests turn red.
5. The cost lane of item 5 above on base versus the revised shape.
6. The class table of item 1 above, with the loader site and the
   cancel path dispositioned, and the install-assert invariant cited.
7. No live edit; report stage 1 and STOP for my gate. Stage 2 remains
   blocked.

**Recorded, not gated.** A public `eval_text` that itself suspends
while a request is already suspended hits `SuspensionSlot::install`'s
assert before any finalization; that pre-existing route is outside this
brief's class, is not repaired here, and the class table must classify
it as out of class without reopening it. The depth-only helper and its
superseded readings remain recorded, never installed. The matrix tree
was rebuilt by hand from the C37 real tree and is not a substitute for
the install-time matrix.

## Continuation-2 gate (2026-11-13) — the 1.4 STOP is well taken: the typed error is reachable through the legacy error wire from a resumed Block, and the wire — not the guard — must carry it

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1d09e-87bb-7143-9216-8aef349b0363`), fork of
`01a1d083-63d3-77f1-b436-a2c4ac8aedc8`; log
`.temp/logs/cdx_brief_731.log` (8,190 lines, 525,059 B); memory-scope
peak 3,263,926,272 B with oom 0, oom_kill 0, rc 0; 711,834 tokens;
50 continuation-2 run records (39 exit 0; 8 exit 101 — the two old
latent-assertion reds, the two old calibrated reds, the two new
calibrated reds, the two legacy reds; 3 exit 1 — the two aggregate
wrappers and the retained class-audit metadata-name attempt);
`report.py check 731` 464 citations by my own run; 12 of the 24
canonical builds consumed (seven before, five here), 12 remain; one
prior forced-SHA spot check; no executed git or network; every write
under `.temp/brief-731/` except my own gate scratch.

**My scope gate.** Working tree clean at `d6dfd64d`; no file under
`crates/`, `docs/` or `scripts/` is newer than the dispatch. I re-ran
the scene's scope audit and it reproduces byte-identically: 2,728 live
files unchanged, the 2,728-file C37 real tree unchanged, the 22 frozen
references unchanged, the continuation-1 copied sources unchanged, the
161 copied inventory TSVs byte-identical (0 installed cells, 0 moved
rows), and every copy differs from its origin in exactly the declared
files. The three non-matrix copies carry the same seven production
bytes; each matrix-tree `session_eval.rs` strips back to the C37 real
tree's file and the other six equal the revised copy's; the binaries
match their receipts (`7d1681ff…` matrix, `c2e559f4…` public3,
`34f77959…` runtime).

**Evidence, reproduced by me.** The matrix audit regenerates the
60-record proof byte-identically: 30/30 typed and retired at both
thread counts, native 30/30 byte-exact, semantic 22 exact plus exactly
the eight `Global`c37answer` 692↔681 full-token bijections. The owner
witness is green at 8 and 1 with the exact tuple (nested typed
refusal, `after` at `owner-second`, the resume completing with the four
cumulative prints and 42, fresh 42, no pending request); its four
completion/overlap/cumulative controls are green at 8 and 1; the
continuation-1 binary reproduces the calibrated red bit-for-bit at
both counts (`after=None`, early `owner-cleanup`, the next resume
"no suspended evaluation is waiting for host input", fresh 42). The
typed-channel unit passes. The class table regenerates
byte-identically: six shared-finalizer call sites, the loader and
cancel dispositions, the `install` assertion, the out-of-class raw
capture. The legacy probe reproduces at 8 and 1: the three
non-resumed controls pass and the resumed Block panics at
`suspension/error.rs:272` with the quoted message, identically at both
counts.

**The defect, read from source.** layout2 folds
`NestedEvaluationInputBoundary` into the wire's structural-slot
`unreachable!` arm (`suspension/error.rs:271`). That guard's premise —
no supported path carries a produced error into the wire — is false
for the new variant. A real request suspended inside a `Block` body,
resumed by host input, evaluates the rest of the body; when a native
callback's nested public `eval_text` refuses, the body's `Result` is
`Err(NestedEvaluationInputBoundary)`; `PrepareSingleBlockInitializerFinalizer`
(`scoping/lexical/block/resume_frames.rs:41-45`) serializes the
block's `initializer_outcome` through
`SuspendedSingleBlockScope::into_continuation_expr` and
`result_into_continuation_expr` into `ContinuationValue for EvalError`.
Every produced variant except the structural slot fault has a wire arm
there — `InputRequired` included — so the new variant is on the wrong
side of that line. Depth is not the issue; this is a carrier
reachability fact about a supported public route, and a panic here is
worse than P1555's silent difference. The candidate cannot install in
this shape.

**My calibration of the resolution.** In a scratch copy of the public3
tree I added the zero-argument wire arm in both directions
(`InternalContinuationEvalErrorNestedEvaluationInputBoundary`; the
guard keeps `ContinuationSlotMismatch(_)` alone), rebuilt the
`b730_nested` binary at the scene's stable path (26.65 s, peak
2,327,875,584 B, oom 0, binary `39b50799…`), and ran it: the resumed
Block's panic is gone; twelve of the fourteen legacy cases return the
typed error at 8 and 1 (direct, block-single, block-multi,
block-resumed, block-init-resumed, loop, loop-resumed, cases-level,
cases-level-resumed, position-level, test-report, test-report-resumed);
the two `guest-get` cases are the documented guest file-load boundary:
`Ok` with `$Failed` and a `Get`/`load` message whose text is exactly
the refusal Display ("nested top-level evaluation cannot suspend for
host input"), no pending input — `finish_guest_get`
(`guest_loading/file.rs:146-164`) converts every error other than the
five pass-through variants to `$Failed` plus a message, unchanged by
this brief. The probe's typed-error expectation for that route was
its author's error; its failure at `b731_legacy_probe.rs:56:9` is
retained. The owner suite is green on the wire-arm build at 8 and 1
(5/5).

**Ruling.** The STOP is well taken. layout2's enum placement, the
public surfaces, the seven-file scope and the cost ruling stand; the
Q2 consolidation sentence is revised:

1. The legacy wire carries every produced error, the typed boundary
   included: `suspension/error.rs` gains the zero-argument forward and
   inverse arms for `NestedEvaluationInputBoundary`; the `unreachable!`
   arm returns to `EvalError::ContinuationSlotMismatch(_)` alone with
   its existing text (the wire and guard are removed at R; the text
   now names only the structural slot fault).
2. Wire coverage is a class duty: every `EvalError` variant is
   dispositioned as serialized in both directions or unreachable with
   a source proof. `ContinuationSlotMismatch` is the only permitted
   unreachable; its producer is the structural slot fault
   (`suspension/core/fault.rs:40`), raised as a driver fault. Any
   other variant on the guard is a STOP.
3. The carrier round trip is tested: `into_continuation_expr` →
   `from_continuation_expr` returns the new variant unchanged, and the
   resumed-Block typed route becomes a permanent test in stage 2. Its
   calibrated red is the continuation-2 public3 binary's panic at
   `suspension/error.rs:272`; its green is the same route on the
   wire-arm source.
4. The 14-case legacy probe is corrected only in its two `guest-get`
   expectations: the guest file-load boundary returns `Ok($Failed)`
   with the `Get`/`load` refusal message, no pending input, and a
   fresh request works; the resumed variant first takes a real outer
   pause at `InputString["outer"]`. The twelve in-language cases return
   the typed error. The old probe source, binaries and failures are
   retained; the loader disposition in the class table is confirmed by
   measurement, never changed.
5. No other production file; any additional production file or public
   surface is a STOP. The shape changed again, so the standing cost
   lane is re-run on the exact revised shape (four hot rows, the
   `EvalError` drop row, the finalizer row, the three sizes, the
   equal-call positive scan; two draws at N=10/20, one codegen unit,
   serial and pinned). No `rum_runtime` row may increase at equal
   calls in either draw; a decrease is recorded, never normalized; any
   increase is a STOP.
6. The 1.5 falsifier stays exactly as written, on the exact revised
   source.

**Continuation 3 order (stage 1 only, no live edit).**

1. Reassemble the four copies from the same origins with the wire arm
   in `suspension/error.rs` (the same seven production files; the
   matrix tree from the C37 real tree); rebuild through the canonical
   builder with a fresh `LOGS` root and one `--force` sha256 spot
   check; record every build receipt.
2. Re-run at 8 and 1: the 30-cell matrix (30/30 typed and retired;
   native 30/30 byte-exact; semantic 22 exact plus the eight
   bijections); the owner witness with its four controls and the
   unchanged calibrated red; the typed-channel unit.
3. The 14-case legacy probe: the twelve in-language routes typed at
   8 and 1; the two `guest-get` cases on the corrected expectation at
   8 and 1, quoting the `$Failed` value and the message text and
   asserting the pre-resume outer pause on the resumed variant; the
   old expectation retained as the retained failure.
4. The wire round-trip test on the new variant.
5. 1.5 the falsifier on the exact revised source.
6. The cost lane of item 5 above.
7. The class table with the wire-carrier rows and the
   `ContinuationSlotMismatch` source proof; the six finalizer sites,
   the loader and cancel dispositions unchanged.
8. No live edit; report stage 1 and STOP for my gate. Stage 2 remains
   blocked.

**Recorded, not gated.** The cumulative-output correction (the latent
suffix-only assertion corrected to the exact four-element cumulative
array, the old failure retained, and a direct 1/2/4 control) is
accepted as a normative correction: `docs/guide/evaluation.md:327-331`
and `docs/guide/embedding.md:298-300` make pause and terminal
responses cumulative, and the direct control proves the prefix.
The worker's probe order is preserved, and the cases it never reached
are the continuation-3 duty. My wire-arm build is a director
calibration, not a canonical build, and does not consume the brief's
24. The install-time permanent matrix remains the install gate, not
this copied matrix.

## Continuation-3 gate (2026-11-13) — the stage-1 STOP is cost: the entry conversion adds 122 Ir per workload iteration on the ordinary entry path, and the shape must ride the result inspection the entry already performs

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1d0c0-fc35-70e0-a891-ae1d7da8273e`, fork of
`01a1d09e-87bb-7143-9216-8aef349b0363`); log
`.temp/logs/cdx_brief_731.log` (6,940 lines, 528,997 B); memory-scope
peak 3,287,826,432 B with oom 0, oom_kill 0, rc 0; 1,076,316 tokens;
126 run records (95 exit 0; 28 exit 101 — the designed reds: the
falsifier's base and falsifier arms, the two legacy red arms and the
two owner calibrated reds; 3 exit 1 — two retained legacy-audit
instrument mistakes and the cost audit's STOP exit); `report.py check
731` 1,189 citations by my own run; 18 of the 24 canonical builds
consumed (six here), six remain; one newly requested forced-SHA check,
equal; no executed git or network; 249 changes, all under
`.temp/brief-731/`.

**My scope gate.** Working tree clean at `521754c8`; no live edit. The
scene's scope audit reproduces under my run: the 2,728-file live tree
unchanged, the 2,728-file C37 real tree unchanged, the 22 frozen
references unchanged, all nine prior copied sources unchanged, and the
incoming continuation-2 report byte-identical. Its shape section reads
seven production files, the three shared finalizers byte-exact, one
entry conversion site, one helper caller, no new frames, derives or
`unsafe`; 115 copied inventories byte-identical with 0 installed cells
and 0 moved rows. The one assertion that fails in my shell is the
worker-scope resource check (`memory.max == 17179869184`), exactly as
in continuation 2: my shell is not inside the worker's 16 GiB scope,
and that is an environment assertion, not a scene difference. The
candidate source differs from the continuation-2 copy in exactly four
files: `suspension/error.rs` (the authorized zero-argument forward and
inverse wire arms, the guard narrowed back to
`ContinuationSlotMismatch(_)` alone), `b730_nested.rs` (the cost-panel
test and the new probe modules), `b731_legacy_probe.rs` (the two
corrected guest expectations) and the retained `b731_legacy_probe_old.rs`.
The base tree's production files equal live; the cost panel's function
body is byte-identical between the two cost arms. All eight binaries
match their receipts (base `435f3812…`, public-wire `e7e14dec…`, the
forced public build equal, matrix `55e68ac9…`, runtime `524cba5e…`,
falsifier `79f9bbc6…`, and the two calibrated-red binaries
`public-layout2` and `public-revised3`).

**Evidence, reproduced by me.** I re-ran every audit against the
retained logs and the derived files come back byte-identical
(`matrix-bijection-proof.json`, `owner-proof.json`,
`falsifier-proof.json`, `class-table.json`, `wire-class-table.json`,
`structural-slot-source-proof.json`, `bound-provenance.json`, all five
cost files, `scope-resources.json`, `production-hashes.json`). I then
re-ran the probes themselves at 8 and 1 on the receipted binaries, into
my own scratch: the 30-cell matrix's `REFUSAL_CELL` records are
byte-identical to the scene's at both thread counts; the owner witness,
its four controls and the continuation-1 calibrated red are
byte-identical; the typed channel and the wire round trip are identical;
the 14 legacy cases are 12 byte-identical, the two TestReport records
differing only in their Timing values, with the guest `$Failed` lines
and the resumed outer pause identical; the falsifier's records are
identical with its expected red/green pattern. The class table: 29
`EvalError` variants, 28 serialized in both directions, the new variant
at the zero-arity tag `InternalContinuationEvalErrorNestedEvaluationInputBoundary`
(forward `error.rs:276`, inverse `:628`), `ContinuationSlotMismatch` the
sole `unreachable!` with its driver-fault producer proof; six finalizer
sites, the loader and cancellation dispositions unchanged.

**The defect, measured.** The cost lane reads
`eval_input_with_terminated_handling` 599 → 721 self Ir per workload
iteration at calls = 4 in both draws, and I reproduced those numbers
from the raw captures with my own parser. That is +122 per iteration —
+30.5 per call — on the ordinary success path of every text, script and
file entry. My own dynamic per-instruction capture attributes it: the
added second match over the finalizer's 136-byte `Result` makes the
compiled code materialize the whole result in a fresh stack slot and
re-load its payload on every call (24 executed instructions per call
where base executes 5), and the extra live range spills other values
across the function; `EvalError` is still 136 B, so this is the
expression shape, not the enum layout. I calibrated a cheaper
closure-level expression in a director scratch (the conversion as
`.map_err(...)` on the finalize match's tail; one fresh compile at the
scene's stable builder path, build `c3dir-cheap-1`, binary
`f47f78b2…`, disclosed and not counted against the brief's cap): it
reads 673 per iteration at N = 10/20 — 48 better, still +74. A
closure-local re-expression is therefore not enough.

**Ruling.** The STOP is well taken. The contract is proved on this
shape — matrix, owner, legacy, wire round trip, falsifier and class all
green on my reruns — but a proved feature with a default-path
regression does not install. layout2's enum placement, the public
surfaces, the seven production files and the wire-carrier ruling stand;
the entry conversion's expression must change:

1. The conversion must ride the result inspection the entry already
   performs: the existing outer `Result` match after the closure, whose
   success arm already branches. The ordinary success path must execute
   at most base's instructions; the conversion runs only on the error
   arm. The worker chooses the exact expression; no new production file
   and no public surface.
2. The contract is unchanged: entry-scoped, ownership-preserving
   conversion of a finalizer-produced `InputRequired` before it escapes;
   the wire carries the typed error; `Session::pending_request` remains
   the ownership test. If moving the conversion after the message-list
   checkpoint restore changes any observed behavior, the panels decide,
   and a red panel is a STOP, never a waiver.
3. The bar: the entry row reads at most 599 at calls = 4 in both draws
   (a decrease is recorded, never normalized); the whole equal-call scan
   shows no `rum_runtime` row increase in either draw; the sizes stay
   136/80/136; the four hot rows and the `EvalError` drop stay at their
   scored values; the `RuntimeMessage` drop row is recorded, never
   offset; two draws at N = 10/20, one codegen unit, serial and pinned;
   the base arm reuses the receipted `bin/base` (sha `435f3812…`, its
   production files equal to live) and does not consume a build.
4. Build budget: the canonical-build cap is extended from 24 to 30 (12
   fresh compiles remain from now), because the cost STOP forces a shape
   iteration. Every fresh compile is recorded; a cache hit on a
   byte-identical assembled tree and a reused receipted binary do not
   count; a director calibration does not count.
5. On the repaired shape, re-run the full stage-1 set on the exact final
   binaries: the 30-cell matrix at 8/1; the owner witness with its four
   controls and the continuation-1 calibrated red; the typed channel;
   the 14-case legacy probe; the wire round trip; the falsifier; the
   class table; the cost lane. Then report stage 1 and STOP for my gate.
   Stage 2 remains blocked.

**Erratum (mine).** The continuation-2 gate section and continuation-3
order item 3 said "the twelve in-language routes return the typed
error". The raw evidence reads otherwise: ten uncaptured routes return
the typed error; the two TestReport routes return `Ok(1)` as failed-test
captures under the existing expectation; the two guest `Get` routes
return `$Failed` plus the exact refusal message. My continuation-3
order's "the twelve in-language cases return the typed error" is
withdrawn and corrected to ten typed + two captures + two `$Failed`.
The worker's correction is exactly this: it preserved the TestReport
capture expectation, corrected only the two guest expectations,
retained the old probe source, binary and failure, and quoted the
resumed variant's outer pause. No assertion was weakened.

**Recorded, not gated.** The three exit-1 runs are retained instrument
mistakes, not product signals: the first legacy audit demanded byte
identity of the TestReport Timing fields; the second demanded the
captured Display inside the TestReport traces; the cost audit exits 1
on its STOP verdict by design. The read-only context-inspection
deviation is recorded; the stage-2/3 deferral is correct; my calibration
build and the scene's stable builder path are disclosed. The TestReport
timing variance is an instrument fact: the probe compares those records
structurally, never byte-for-byte, and their raw Timing values stay
unmodified in the retained evidence.

**Continuation 4 order (stage 1 only, no live edit).**

1. Re-express the entry conversion under ruling items 1-2, keeping the
   seven production files and every public surface.
2. Calibrate each candidate expression in scratch with the cost lane
   (the entry row plus the whole equal-call scan, two draws) before
   selecting; record every attempt and its measurement; no shape is
   selected without meeting item 3.
3. Reassemble the four copies from the same origins with the repair;
   rebuild through the canonical builder with fresh `LOGS` roots and one
   `--force` sha256 check; record every receipt and the extended budget.
4. Re-run the full stage-1 set of ruling item 5 on the exact final
   binaries at 8 and 1.
5. No live edit; report stage 1 and STOP for my gate. Stage 2 remains
   blocked.
