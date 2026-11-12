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
