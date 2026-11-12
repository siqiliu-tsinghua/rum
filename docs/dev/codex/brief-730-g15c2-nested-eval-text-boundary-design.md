# Brief 730 — G15-C2: design the nested public `eval_text` completion-boundary repair (`P1555`)

Stage: **design only**. `Scope: NONE — no source change`. The director
writes the implementation brief after this design gate.

## Goal

Resolve the loss recorded as `P1555` on the accepted source: an ordinary
native builtin that calls the public `Session::eval_text` loses the inner
call's top-level completion when the inner evaluation suspends on `Input`
and is resumed. The queued path finalizes the inner result (consumes
`Return`, converts terminations, emits the inner uncaught-throw message,
restores the message-list checkpoint); the resumed path does not — the
inner `Return` escapes to the outer request's finalizer and the inner
message pass is skipped. The measured witnesses are exact: `queued 2`
against `resumed 1`, and two `Throw::nocatch` messages against one.

When you are done the design must name exactly one resolution, prove it
in a byte copy, and hand back an implementation brief:

1. **Repair**: one typed owner carries the inner top-level completion
   boundary (its output mark, saved message-list checkpoint and dirty
   flag, and its terminated-evaluation handling mode) across the
   suspension, so the resumed completion runs the same finalization and
   the same error-emission tail as the queued completion, and the
   native call's returned value is the finalized value. The frozen
   30-cell matrix and the six minimized/independent witnesses must
   become SAME with the queued observations as the reference.
2. **Enforced boundary**: a typed refusal at the exact suspension point
   inside the nested `eval_text` boundary that makes the non-resumable
   route fail loudly and identically wherever it can occur, with its
   documented contract, its own tests, and a source proof that no
   in-tree production caller (host request API, script/file loops, CLI)
   changes. A doc-only note that leaves the silent semantic difference
   in place is not acceptable.

The design must decide between them with evidence, not argument, and
must not invent a guest carrier, a fabricated value or a fabricated
suspension.

## Context pointers

- `.temp/g15-c2/c37/top-level-loss-proof.json`: the authenticated
  30-cell matrix, the 16 own armed-hit-caught omissions, the six
  minimized witnesses, the independent public-API confirmation and the
  mechanism text. Every claim in it is reproducible from the raw logs
  it names.
- `.temp/g15-c2/c37/DRAFT-top-level-native-finalization.md`: the
  worker's draft; a proposal, not a mandate.
- `.temp/g15-c2/c37/public_native_confirmation.rs`: the six-case
  public-API reproducer (three DIFF, three SAME).
- `crates/rum-runtime/src/runtime/session/session_eval.rs:200`
  (`eval_text`), `:233-296` (`eval_input_with_terminated_handling`),
  `:298` (`eval_script`), `:375` (`eval_file`), and the finalizer
  helpers at `:21-75`.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:518`
  (`emit_non_loader_eval_message`) and `:542`
  (`emit_top_level_control_flow_messages`).
- `crates/rum-runtime/src/runtime/session/session_requests.rs:283`
  (the outer request resume route that resumes the existing expression
  continuation).
- `crates/rum-runtime/src/runtime/suspension/` (the suspension-frame
  machinery: capture, resume, the typed frame owners used by the
  accepted C2 repairs) and `crates/rum-runtime/src/runtime/eval_call_prepare.rs`
  and the iteration modules for the established typed-frame pattern.
- `docs/host-embedding-and-hardening.md:110-125`: the host contract,
  including the documented script/file non-resumable boundary — keep it
  distinct; do not extend it by argument.
- `crates/rum-host/src/session.rs:436` and `:592`: the documented
  request/resume API and its tests.
- The accepted gate for continuation 37: the same source
  (`production_sha256` `8313fbc1…`, 2,726 files, 0 rebases) and the
  frozen C37 scene (read-only).

## Scope

- Files you MAY modify: **NONE in the live tree**. All work lives under
  `.temp/brief-730/` (byte copies, probes, receipts, report).
- Files you MUST NOT touch: the live `crates/` tree, `scripts/`, `docs/`,
  `TODO.md`, `AGENTS.md`, `Cargo.toml`, `Cargo.lock`, `.git`, and every
  existing `.temp/g15-c2/` scene (all of c13–c37, `brief-623`,
  `probe/` beyond the sanctioned cache, `cargo-home` beyond the
  sanctioned cache).
- No git command in any form. No network. No `unsafe`. No live edit.
- Build only through `scripts/probe-build.py` with a complete assembled
  tree under `.temp/brief-730/`, Rust 1.85.0, offline + locked, `-j 8`,
  one Cargo process at a time, bounded commands, a unique per-invocation
  `LOGS` root. One `--force` sha256 spot check this round under the
  scene environment (`CARGO_HOME=.temp/g15-c2/cargo-home`, the scene
  `TMPDIR`, `RUSTUP_TOOLCHAIN=1.85.0`).
- Keep every attempt, including failed ones, with its raw log.

## Reproduction you must reproduce first

Freeze these from the raw C37 records before designing anything:

1. The 30-cell matrix: 16 SAME (of which two `RestoreSealedReferences.body`
   cells carry `binding_applicable=false` and stay uncredited) and 14
   DIFF, all `valid`, with real records byte-identical to their disabled
   peers and the producer reached on all 30 cells.
2. The two DIFF classes exactly: nine records whose only semantic
   difference is the duplicate `Throw::nocatch` message (queued two,
   resumed one) and five `return_boundary.last-pause-return` records
   whose only difference is the missing inner `Return` finalization
   (queued value 2 vs resumed 1 in the minimal witness; unequal trace
   vectors in the matrix records). Say which fields of the queued
   observation each class loses on the resumed arm.
3. The six minimized/independent cases, including the three controls
   (direct `Input["p"];2` with `Return[1]`, direct `Input["p"]` with
   `Throw[1]`, and `C37NativeText[...];2` with `1`), which must stay
   SAME.
4. The seam instrument: every real/disabled log's 60
   `C37_MESSAGE_RESTORE` reads with `checkpoint_mismatch=0
   dirty_mismatch=0`, and each own mutant log's two reads with exactly
   one mismatch.
5. That no in-tree production caller of `eval_text`/`eval_script`/
   `eval_file` exists inside evaluation (host `session.rs`, the CLI and
   test modules call them from outside a running evaluation), so the
   nested route is a public embedding surface, not a guest-reachable
   one. Quote the call sites.

## Design duties

1. **Inventory the class.** Every public session-evaluation entrypoint
   (`eval_text`, `eval_script`, `eval_file`, `eval_request` /
   `resume_eval_request_with_input`, snapshot restore, and any other
   re-entrant entry), every inner top-level finalization/error-emission
   owner it enters, and every way an inner evaluation can suspend
   (`Input`, `InputString`, depth/limit terminations, cancellation).
   Each row gets a disposition: covered by the chosen resolution, out
   of class with a source proof, or open with its named reason. The
   named instance is not the class.
2. **Decide the boundary with evidence.** State the two candidate
   resolutions, the exact public contract each implies, and the source
   evidence that settles it. The queued path is the reference. A
   design that keeps the nested route supported must show the
   completion boundary survives at least two suspensions, that the
   native call's returned value is the finalized value on both arms,
   and that the outer request is otherwise unchanged. A design that
   refuses must show the exact point of refusal, the typed error, the
   identical behavior of every other entrypoint, and the tests that
   pin it. Name every public API shape change; the director rules on
   it.
3. **Prove it in a copied experiment.** For the repair: a byte-copy
   candidate that makes the frozen 30-cell matrix and the six
   minimized/independent witnesses SAME, with the queued arms
   byte-identical to the C37 queued arms (the reference) and the three
   controls unchanged, plus an own calibrated omission that reddens the
   parity assertion on the same cells and is the proof the repair
   closes it. For the boundary: a byte-copy candidate that fails the
   nested suspension with the typed error at 8 and 1 threads, keeps the
   queued arm and every documented host path unchanged, and whose
   falsifier (the typed refusal removed) restores the silent
   difference. Either way the frozen C37 matrix records stay as
   historical evidence; the expected post-repair/rejection record
   changes exactly the fields the resolution claims, and that delta is
   the acceptance signature.
4. **Cost and shape.** The ordinary top-level success path must not pay
   for the repair or the refusal: the changed functions' disabled-path
   instruction counts must be identical (no-increase bar, never
   padding), and any hot-path change requires the rum-vs-rum
   instruction gate. Report every changed signature and caller; a
   public API change is an explicit decision for the director. No new
   `unsafe`, no second classification surface, no trait-object
   carrier.
5. **Contract decision.** Name every documented host contract or
   existing test that speaks to nested evaluation, resumption or the
   script/file boundary, with `path:line`; state the exact choice
   required. Do not weaken, delete or relabel an assertion to make
   either candidate pass.
6. **Draft the implementation brief.** In `TEMPLATE.md` shape: exact
   staged order, the permanent regression tests, the copied proof
   commands, the standing-inventory replay, the full acceptance
   (runtime suite, workspace, host tests), the disabled-path cost gate,
   the stop-loss, and the exact scope list.

## Verification and the design gate

- `report.py check` on your report; every cited line quoted verbatim.
- The copied experiment's raw records and logs are kept; each claimed
  SAME/DIFF/refusal is reproducible from them.
- The chosen candidate is watched failing on the pre-change copy at the
  named assertion; the omission/falsifier reddens it again.
- No frozen scene, inventory or accepted record is edited, relabelled or
  re-pinned.
- The design states its open decisions; it does not pick a public
  contract change by itself.

## Hard rules

- No git, no network, no `unsafe`, no live-tree edit, no frozen-scene
  edit, no `.temp` file outside `.temp/brief-730/`.
- No inventory edit, no assertion weakening, no Syntax-to-Other
  normalization, no fabricated suspension or value, no test weakening.
- Do not optimize a cold error path at the cost of the success path, and
  do not pay default-path cost for diagnostics.
- If the two resolutions cannot be settled from source and the copied
  experiments, STOP and report the exact evidence gap — do not pick one
  by preference.

## Report format

Return `rum-report/1` with the selected resolution and its exact public
contract, the full entrypoint/finalization inventory, the copied-tree
receipts (pre-change red, candidate green or refusal, own omission
red), the contract decision list, the drafted implementation brief
path, and every open question. No source change is authorized by this
brief.

## Design gate (2026-11-12) — the STOP is well taken: the completion-only repair is proven on the frozen matrix but is not class-complete, the refusal candidate is unproved, and neither cost shape settles the bar; design continuation 1 completes both candidates' evidence

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1cfef-3761-7502-a049-25c43760dccb`), log
`.temp/logs/cdx_brief_730.log` (38,726 lines, 1,893,399 B), memory-scope
peak 3,036,712,960 B with oom 0 and rc 0; 209,392 tokens; 103 run records,
17 retained non-zero exits (the freeze instrument correction, three compile
failures, the expected base/omission REDs, the six class-probe REDs, the
map REDs, and the final-audit parser correction); report 13 checks 7 PASS /
3 FLAGGED / 1 FAIL / 2 NA with two FAIL measurements, and `report.py check`
1,053 citations by my own run; no git or network attempt (no
`.git-attempts`); all 40 reported changes under `.temp/brief-730/`.

**My scope gate.** I walked the live tree with the C37 entry manifest:
2,727 files; the delta against entry-37 is exactly three of my own
post-gate files — `TODO.md`, brief 623 (my continuation-37 acceptance
append) and brief 730 (my dispatch commit). Production is 1,622 files at
`8313fbc1583eec539460dfd2ba1ce655452dd86df6134f3e7808b377d07a025c`,
byte-identical to the entry manifest, 0 added/removed;
`source_rebases=0`. My own seal walk of the C37 scene is 86,737 files,
86,737 read, 0 missing, 0 errors, and exactly the two sealed derived files
my continuation-37 replay refreshed (`attempt-audit.json`,
`integrity-receipt.json`) differ; the scene's newest mtime is 21:17:54,
before the 21:25 dispatch. The worker's own final audit re-read the 1,622
production files and the 22 frozen raw artifacts with 0 changes. Builds ran
through the canonical `scripts/probe-build.py` wrapper with
`CARGO_HOME=.temp/g15-c2/cargo-home`, the brief's own `TMPDIR`,
`RUSTUP_TOOLCHAIN=1.85.0`, offline + locked, `-j 8`, one Cargo at a time, a
fresh `LOGS` root per invocation, one forced rebuild (`cache_hit=false`,
`spot_check=pass`, identical `key`/`tree`/`sha256`
`56aa5622e38e64a3fc1589907b6f37ff1628625939ee211a49e623f241e468cc`).

**Evidence, reproduced by me.** `freeze.py` re-read the C37 raw artifacts
independently (22 files sha-pinned in two places) and re-derived the matrix,
the two DIFF classes (9 duplicate-`nocatch` message records, 5
inner-return-finalization trace records), the 16 own-omission logs (2 seam
reads each, exactly one mismatch), the six minimized cases and the six
independent public cases; the frozen matrix equals the C37 real arm equals
the disabled arm. The base arm then reproduces the frozen 30-cell matrix
byte-for-byte — full raw JSON records, no normalization · real == disabled
== fresh base == omission, 30/30 · and `raw_byte_audit.py` proves the
candidate arms' `queued`/`semantic_queued` fields byte-identical to the
originals. The repair arm is 30/30 five-part SAME; exactly the 14 former
DIFF records change, and only `semantic_resumed`, `resumed`, `semantic_same`
and `native_same` change, field-for-field per the acceptance signature; the
other 16 records are byte-identical; `queued_changes=0`. The omission arm
is 14/14 DIFF over exactly the original cell set with all 30 records
byte-identical to base. The six minimized cases are 6/6 SAME. The public
parity driver runs `resumed == queued` through the public request/resume
API: base is RED at the named assertion (`b730_nested.rs:64`,
`['return-value','throw-messages','return-phase','twice-return']`) at 8 and
1 threads; the repair is GREEN at 8 and 1; the three controls stay SAME on
both arms; the repeated-suspension case (`twice-return`, two real pauses) is
SAME on the repair. The two new class probes are RED identically on base,
repair and the inline shape at 8 and 1 — the inner `Print[99]` prefix is
published on the resumed arm (`prints ["99"]` vs `[]`) and the inner
`$MessageList` reads the outer message on the resumed arm (`1` vs `0`); the
native-map probe shows the queued `List[1]` return transformation lost on
both arms. The cost tables match my own re-parse of the raw perf logs: the
first shape has 7/8 positive whole-process slopes, the inline shape 3/8 with
sign changes across draws, and the unchanged `eval_request` control moves
±0.17/call — neither shape establishes the no-increase bar, and the
changed-function identity was not measured. All 22 frozen artifacts hash
match the live scene and the scene copies.

**The STOP.** Accepted, and well taken under the brief's own hard rule:
the evidence settles a narrower fact (a typed completion owner repairs
`P1555`'s frozen 30 cells and all six public witnesses with the queued
reference byte-identical), but the same public `eval_text` boundary still
loses the inner output ownership and the active message environment on the
resumed arm, the refused alternative has no candidate or falsifier, and
neither cost shape meets the disabled-path bar. Choosing now would be
preference in place of evidence. No public contract change is selected, no
live source or inventory was touched, and the implementation draft is
marked NOT-READY. The three FLAGGED checks (class inventory, native-caller
tail, draft) and the two cost FAILs are the honest gaps, not overclaims.

**Rulings for design continuation 1.** The line stays open and the design
must be completed, not converted into an implementation brief:

1. **Class completeness of the repair.** The two class probes are in the
   nested top-level boundary class, not outside it: on a resumed nested
   evaluation the inner computation must run under the inner message
   environment (checkpoint/dirty installed before the captured frames run,
   not after the finalization tail) and with the inner output owned and
   removed as the queued path's `take_batch_output` does. Design one typed
   owner or owner-scope that carries both across the suspension (naming the
   exact outer-resume surface it must enter and leave), prove it in the byte
   copy: both probes SAME at 8/1, the repeated-suspension case still SAME,
   every queued arm still byte-identical, and every documented host path
   unchanged. If a source-grounded proof shows a piece cannot be covered,
   name the exact blocker and record it as a named loss — do not leave it
   open by argument.
2. **Disabled-path cost, on the standing instrument.** Whole-process perf
   slopes cannot establish changed-function identity. Measure the changed
   functions' ordinary-success-path instruction counts at unchanged call
   counts with the Callgrind identity lane used by the earlier disabled
   gates (base and candidate at one path, one codegen unit), plus the
   rum-vs-rum hot-label/microbench lane if any hot change is claimed. The
   bar is exact no-increase on the disabled path, no padding, and every
   changed signature and caller listed.
3. **The enforced-refusal candidate, built and calibrated.** In the byte
   copy only: the exact refusal point is an actual empty-queue
   `InputRequired` inside a same-Session nested top-level entry under an
   active evaluation; the entry retires its actual captured continuation
   through the ordered abandonment protocol and returns a distinct typed
   non-resumable-boundary failure (no fabricated value, no fabricated
   `InputRequired`). Show at 8/1 that the nested suspension fails with the
   typed error, that the queued arm stays byte-identical to the frozen
   reference, that the top-level host source/request/resume paths, the
   script/file batch boundary and the snapshot-restore terminal path are
   unchanged, and that the falsifier (refusal removed) restores the silent
   difference — the four public parity DIFFs and the two class-probe
   differences return. Name every public surface the typed error would
   change (`EvalError`, `SessionRequestError`, `HostFailureCode`, exhaustive
   matches, host mapping) for my ruling; do not install it live.
4. **The native-caller duty.** A native callback that transforms the value
   after `eval_text` cannot re-run its Rust frame; state the exact public
   contract for that case and pin it with a test, or fold it into the
   refusal surface. Do not claim the transformation survives without
   re-entering native code, and do not fabricate a guest carrier.
5. **Decision rule.** The repair is selectable only if items 1-2 hold and
   the frozen matrix, the six witnesses, the controls and the repeated
   suspension all stay SAME with the queued bytes unchanged; the refusal is
   selectable only if item 3 holds with its falsifier and the public
   surfaces named. The continuation recommends one strictly by evidence; I
   rule and then write the implementation brief. All prior scene evidence
   is immutable; new arms get new names under `.temp/brief-730/`; no live
   edit, no frozen-scene edit, no git, no network, no `unsafe`, no
   inventory edit.

**Recorded, not gated.** The retained instrument corrections: freeze attempt
1 asserted the wrong invariant and was corrected to the C37 fact (25
records native-equal, 5 trace-different), all raw logs kept; three harness
compile failures and the final-audit parser failure are retained with their
transcripts; the two class probes and the map probe are diagnostics, RED on
every arm, not regressions or evidence for unrelated production work; the
implementation draft stays NOT-READY and must not be dispatched; the
per-family `native-value-control` and controls are unchanged. No assertion
was weakened, no record relabelled, no inventory edited.

**Next.** Design continuation 1 of brief 730 as ordered above; then my
ruling between the two proven candidates; then the implementation brief,
its gate, and continuation 38 of brief 623 on the installed source; then
C2's settlement round, CM, `CHANGELOG.md` before S10, G13 S10–S54/F1–F23/R,
E2.

## Design gate (2026-11-12) — continuation 1's STOP is well taken: the repair is eliminated by the native Rust tail, the enforced typed refusal is selected as the resolution, and brief 731 implements it under a staged copied calibration

**Run.** `cdx` (Codex `gpt-6-astra`, reasoning max; session
`01a1d020-3c73-72e1-a589-3d9050a20959`), log
`.temp/logs/cdx_brief_730.log` (27,626 lines, 2,199,817 B); memory-scope
peak 4,065,177,600 B with oom 0 and rc 0; 401,622 tokens; 204 run
records (158 exit 0, 37 exit 101, 9 exit 1; 46 non-zero exits retained);
report 6 PASS / 2 FLAGGED / 2 FAIL / 1 NA checks and 1 PASS / 1 FLAGGED
/ 2 FAIL measurements, `report.py check` 2,319 citations by my own run;
24 canonical builds with 2 retained compile failures and one forced SHA
spot check `5324514d…`; no git or network; 320 changes, all under
`.temp/brief-730/` (the new continuation report included; the run-1
top-level scene has no changed file — only the new
`report-continuation-1.json`).

**My scope gate.** Live 2,727 files; the delta against the C37 entry
manifest is exactly my own three post-gate files (`TODO.md`, brief 623,
brief 730); production is 1,622 files at
`8313fbc1583eec539460dfd2ba1ce655452dd86df6134f3e7808b377d07a025c`,
byte-identical to the C37 entry with 0 added/removed; my seal walk of
the C37 scene is 86,737 files, 86,737 read, 0 missing, 0 errors, and
only the two sealed derived files my continuation-37 replay refreshed
(`attempt-audit.json`, `integrity-receipt.json`) differ; newest scene
mtime 21:17:54, before the 22:18 dispatch. The continuation-1 `frozen`
tree, `frozen-matrix.json`, `frozen-manifest.json` and
`frozen-deltas.json` are byte-identical to run 1's; the seven refusal
and five repair candidate source copies hash-match
`candidate-source-deltas.json` file for file; the scope audit reads
1,622 live production files with 0 changes, 22 frozen raw files with 0
changes, 20 copied trees, 500 copied inventory files byte-identical and
all writes under `.temp/brief-730/`. Builds all ran through the
canonical builder (`scripts/probe-build.py`, `build.py` prints
`canonical=scripts/probe-build.py`), offline+locked, one Cargo at a
time, fresh `LOGS` per invocation.

**Evidence, reproduced by me.** The repair arm is 30/30 five-part SAME
on the frozen matrix with the six minimized cases 6/6 SAME, the queued
`queued`/`semantic_queued` raw bytes 30/30 identical, its own omission
restoring exactly the 16 SAME / 14 DIFF base with all 30 records
byte-identical, and public parity green at 8/1 with the three controls
and the two-real-pause case; the extended environment and output
ownership close both former class probes
(`boundary_output_and_message_scope` green at 8/1). But the repair still
loses the native Rust tail: the `native_return_transformation` assertion
is RED at 8/1 at the named site (`b730_nested.rs:72`, queued `List[1]`
against resumed `1`), and the owner scope cannot re-enter an arbitrary
host Rust frame. Its ordinary-success cost also fails:
`eval_input_with_terminated_handling` 599 → 631 Ir per workload at four
calls, +8/call in both draws, on both shapes. The refusal's public
proof is green at 8/1: 13 entrypoint routes, eight nested cases
including the native map, ordered abandonment, no pending owner after
refusal, fresh requests, the documented host paths, script/file and
snapshot-terminal boundaries, and ordinary guest capture/loading; the
final falsifier removes only the two error-arm calls, keeps the
enum/helper/mappings, and restores the four public parity differences
and both class differences byte-identically to base at 8/1
(historical/parity/boundary/native observations equal to base, ten
queued public cells and runtime host records byte-identical). The
final-shape matrix selector ran zero tests (the copied test file lacked
its `c2_native` module declaration); the earlier-shape matrix is real
evidence only for that shape: 30 cells typed and retired at 8/1, native
queued vectors 30/30 byte-identical, semantic queued vectors 22/30. The
eight semantic differences are exactly one internal symbol ID:
`Global`c37answer` is 692 in the frozen reference and 681 in the
candidate, with identical text and token structure. My own check proves
it: after removing the difference file's `- `/`+ ` markers and
substituting the single `c37answer` 681↔692 symbol pair, all 8/8
records are byte-identical (the native pair is identical before any
substitution, and no other symbol ID differs). The final refusal's
changed-function/neighbor rows are exact or decreasing, but the wider
deterministic equal-call audit still reads +1/call in
`prepare_arguments_for_call_with_unevaluated_restore` (1649 → 1655 at 6
calls), `evaluate_inner_unstable` (865 → 870 at 5) and
`evaluate_inner_unstable'2` (173 → 174 at 1), both draws. The legacy
`ContinuationValue for EvalError` guard is the same invariant already
carried by `EvalError::ContinuationSlotMismatch` ("this wire and guard
are removed at R"); the typed channel forwards the error unchanged, and
the guard is not reached by a supported path. The
reentrant-resume-with-a-pre-existing-owner case remains unproved. The
read-only inventory replay regenerated 68 fingerprint cells with
`installed=false`; the three stale-cell reds are ordering artifacts, not
regressions, and no live inventory changed.

**The STOP.** Accepted and well taken under the brief's own hard rule.
No resolution is selected by preference: the repair is class-incomplete
on a source-grounded inherent limit and fails its cost bar, and the
refusal still has measured proof gaps. The retention of every attempt,
the zero-test selector's non-credit, and the eight strict queued-byte
failures are honest.

**The ruling: the enforced typed refusal is the resolution.** The
repair is not selectable. Its native Rust tail is a named,
source-grounded loss (`B730_NATIVE_RUST_TAIL`): the engine cannot
re-enter an arbitrary native caller's Rust frame after a suspension, and
fabricating a guest carrier is forbidden; a repair that leaves that
silent difference in place is exactly what the brief exists to
eliminate. The refusal is the only class-complete contract: at an
actual empty-queue `InputRequired` inside a same-Session nested
top-level entry under nonzero active evaluation depth, the entry
retires its actual captured continuation through the ordered
abandonment protocol and returns a distinct typed non-resumable
failure; the queued path, the top-level host request/resume, the
script/file batch boundary, the snapshot terminal restore and ordinary
guest capture/loading are unchanged. The native caller's queued
successful transformation remains; on the suspending case it receives
the typed failure and its Rust frame is never replayed. The repair and
`NestedTextCompletion`/`NestedTextEnvironment` evidence stay recorded,
not installed.

**Answers to the worker's questions.** Q1: stage 1 of the implementation
brief closes the exact byte and cost blockers in the copied tree before
any live edit — the corrected final-shape 30-cell matrix, the calibrated
queued-reference predicate, the hot-cost attribution and shape, and the
reentrant-owner probe. Q2: the public contract is one unit variant in
each of `EvalError`, `SessionRequestError` and `HostFailureCode`
(`NestedEvaluationInputBoundary`), their Display arms, the
`session_request_error_from_eval_error` mapping, the host mapping and
the Jupyter name arm; every exhaustive match is updated; the legacy
conversion keeps the existing typed-only structural-fault guard (no
fabricated carrier, no signature change). The API break is allowed
inside the security line and is named for `CHANGELOG.md` before S10.
Q3: the nested entry must abandon only the continuation it actually
captured. A pre-existing request owner is never stolen, overwritten or
abandoned by the nested entry; it remains resumable, and a fresh
request works afterward. A dedicated probe pins this at 8/1; if the
shape cannot distinguish the owners, STOP, never weaken.

**Calibrated queued-reference predicate.** The queued arm is identical
when its semantic tokens are identical after a proven bijective
renaming of internal symbol IDs: same symbol text, same multiplicity,
exactly one such renaming per differing record, and every other token
byte-equal; native vectors remain 30/30 byte-identical. The eight known
`Global`c37answer` 692↔681 records meet this predicate by my own
reproduction. This is the C36 intern-ID class, not a semantic
normalization; the audit records the per-record bijection proof and
never normalizes any other token.

**Recorded, not gated.** The retained instrument corrections:
`final-semantic-audit` attempt 1's parity-summary parse (`IndexError`)
and attempt 2's strict-equality red; `final-public-audit` attempt 1's
marker mismatch and attempt 2's green; the two evidence audits' strict
queued-byte assertions (the same intern-ID class); the matrix-omission
check's expected 16/14 red (the omission restores the base by design);
the hot-gate FAIL and the zero-test final selector (both uncredited, not
green evidence); the read-only replay's three stale derived cells as
ordering artifacts; the microbench's 49 positive labels (FLAGGED, not a
gate); and the 24-build cap consumed and honored. No assertion was
weakened, no record relabelled, no inventory edited, no frozen evidence
or live source changed.

**Next.** Write and dispatch brief 731 (implementation, staged: copied
calibration, install, acceptance), gate it, then continuation 38 of
brief 623 on the installed source; then C2's settlement round, CM,
`CHANGELOG.md` before S10 (including this public enum break and the
earlier `EvalError::Syntax` break), G13 S10–S54/F1–F23/R, E2.
