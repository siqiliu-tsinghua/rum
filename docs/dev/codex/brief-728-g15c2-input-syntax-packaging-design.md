# Brief 728 — G15-C2: design the queued/resumed Input syntax-error packaging repair

Stage: **design only**. `Scope: NONE — no source change`. The director
writes the implementation brief after this design gate.

## Goal

Specify, prove in a byte copy, and hand back as an implementation brief
one repair for the diagnostic loss recorded as `P1566` on the accepted
source. A malformed `Input[...]` answer produces the same parse failure on
the ordinary queued request and on the resumed request, but the two host
request tails publish different public error variants: the ordinary tail
reports `SessionRequestError::Other`, the resumed peer reports the
structured `SessionRequestError::Syntax` with the same message text.
Continuation 12 minimized that to one input, classified it as a genuine
pre-existing public-observation loss, and rejected the only binding that
would have hidden it.

When you are done the design must: name exactly one way to carry the
original Input source through ordinary queued evaluation to the ordinary
request tail so both peers expose the same structured Syntax diagnostic;
enumerate every `EvalError::Syntax` producer and every source-losing
conversion on those two paths with a disposition each; prove it in a
copied experiment against the frozen red and the green controls; and
carry a drafted implementation brief in `docs/dev/codex/TEMPLATE.md`
shape with its staged acceptance commands.

## Context pointers

- `.temp/g15-c2/c12/malformed-loss-proof.json`: the exact current
  observations, the source packs, the minimal case, the two controls and
  the pre-existing classification.
- `.temp/g15-c2/c12/DRAFT-input-syntax-packaging.md`: the worker's draft;
  treat it as a proposal, not a mandate.
- `crates/rum-runtime/src/runtime/builtins_input.rs:302-315`: where the
  actual Input answer text is parsed.
- `crates/rum-runtime/src/runtime/session/session_requests.rs:244-280`
  (ordinary evaluation and its error tail), `:315-381` (resumed
  packaging), `:536-583` (failure construction).
- `crates/rum-runtime/src/runtime/session/request_errors.rs:28-55`: the
  source-aware Syntax/Other selection.
- `crates/rum-stdlib/tests/suspension_oracle/support/observe.rs`: the
  host observer that renders the public observation.
- `docs/dev/codex/brief-727-g15c2-completed-segment-pause-receipt-install.md`
  and its gate: the accepted source you work on; the 727 completed-segment
  rule and the validity of logical-profile epochs stay untouched.
- `.temp/brief-727/class-records.json`: `B726.class.error`, the original
  two-pause observation (resumed Syntax, queued Other) that brief 727
  kept flagged.
- The frozen C2 scenes (C7-C11) are evidence: read them, never write them.

## Scope

- Files you MAY modify: **NONE in the live tree**. All work lives under
  `.temp/brief-728/` (byte copies, probes, receipts, report).
- Files you MUST NOT touch: the live `crates/` tree, `scripts/`, `docs/`,
  `TODO.md`, `AGENTS.md`, `Cargo.toml`, `Cargo.lock`, `.git`, and every
  existing `.temp/g15-c2/` scene (C7-C11, `brief-623`, `brief-726`,
  `brief-727`, `probe/` beyond the sanctioned cache).
- No git command in any form. No network. No `unsafe`. No live edit.
- Build only through `scripts/probe-build.py` with a complete assembled
  tree under `.temp/brief-728/`, Rust 1.85.0, offline + locked, `-j 8`,
  one Cargo process at a time, bounded commands. One `--force` sha256
  spot check this round, under the scene environment
  (`CARGO_HOME=.temp/g15-c2/cargo-home`, the scene `TMPDIR`,
  `RUSTUP_TOOLCHAIN=1.85.0`).
- Keep every attempt, including failed ones, with its raw log.

## Reproduction you must reproduce first

Freeze, from raw records, the exact current state on the accepted source
before designing anything:

1. The red: `Input["answer"]` answered with the single malformed text
   `(`. Queued observation
   `Other { message: "incomplete input: expected expression at bytes 1..1" }`
   against resumed
   `Syntax(StructuredSourceDiagnostic { message: <same text>, … })`;
   native ten-field rows SAME; logical-profile epochs valid; no faults.
2. The controls, both SAME: `Input["answer"]` answered `83`, and
   `InputString["answer"]` answered `"("` (a string answer, not a
   malformed expression).
3. The original two-pause `B726.class.error` case, byte-equal to the
   accepted brief-727 class record.
4. Confirm the frozen P1566 and P1568 requirement cells and their 55/60
   matrices are otherwise SAME, and that no requirement key or encoding
   run is added by the observation.

## Design duties

1. **Inventory the class.** Enumerate every `EvalError::Syntax` producer
   and every place a Syntax error is converted to a source-less form on
   the ordinary and resumed host-request paths, including Input argument
   parsing, nested Input, parse-depth errors, multiple pending inputs,
   non-ASCII answer locations, cancellation, timeout, abandoned
   requests, and fresh-after-reset requests. Each row gets a disposition:
   carried by the chosen representation, out of class with a source
   proof, or open with its named reason. The named instance is not the
   class.
2. **Choose exactly one representation.** A representation and API that
   carry the original Input source (not the outer program text, not a
   fabricated source) from the parse failure to the ordinary request
   tail, with its ownership, lifetime, cleanup and removal semantics
   stated exactly. Prefer the compiler-enforced shape over a convention.
   State what the representation does when the source is absent, shared,
   nested, or from a request that has already retired.
3. **Prove the parity claim in the copy.** The copied candidate must
   produce whole-observation equality for the red case with the original
   structured diagnostic preserved, byte-equal language/prints/messages/
   trace, valid epochs, and real==disabled instrumentation. Keep an own
   source-provenance omission that reddens the parity assertion on the
   same cells — the proof that the repair is what closes it. The two
   green controls and the original two-pause case must stay exactly as
   frozen.
4. **Cost and shape.** The ordinary success path must not pay for the
   repair: the changed call sites' disabled-path instruction counts must
   be identical, and any hot-path change requires the rum-vs-rum
   instruction gate. Report the exact call-site inventory, every changed
   signature and every caller; a public API change is an explicit
   decision for the director, not a silent one. Do not introduce a
   second classification surface.
5. **Contract decision.** If any documented host contract or existing
   test asserts `Other` for malformed Input answers, name it with
   `path:line` and state the exact choice required (change the contract
   with the repair, or keep `Other` on that path and open a different
   disposition). Do not weaken or delete an assertion to make the
   candidate compile.
6. **Draft the implementation brief.** In `TEMPLATE.md` shape:
   the exact staged order, the permanent regression tests, the copied
   proof commands, the standing-inventory replay, the full acceptance
   (runtime suite, workspace, host tests), the disabled-path cost gate,
   the stop-loss, and the exact scope list.

## Verification and the design gate

- `report.py check` on your report; every cited line quoted verbatim.
- The copied experiment's raw records and logs are kept; each claimed
  SAME/DIFF is reproducible from them.
- The red is watched failing on the pre-repair copy at the parity
  assertion; the candidate is green at the same cells; the omission
  reddens it again.
- No frozen scene, inventory or accepted record is edited, relabelled or
  re-pinned.
- The design states its open decisions; it does not pick a public
  contract change by itself.

## Hard rules

- No git, no network, no `unsafe`, no live-tree edit, no frozen-scene
  edit, no `.temp` file outside `.temp/brief-728/`.
- No classifier masking, comparator weakening, Syntax-to-Other
  normalization, invalid-epoch parity certificate, or test weakening.
- Do not optimize a cold error path at the cost of the success path, and
  do not pay default-path cost for diagnostics.
- Stop-loss: stop after two measured attempts with no progress, or on a
  calibration failure you cannot explain from source — and say which.

## Report format

Return `rum-report/1` with the selected representation and API, the full
producer/conversion inventory, the exact copied-tree red/green/own-omission
receipts, the contract decision list, the drafted implementation brief
path, and every open question. No source change is authorized by this
brief.
