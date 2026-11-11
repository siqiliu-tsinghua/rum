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

## Director gate (2026-11-11) — the design is accepted, the STOP is a bar correction, and brief 729 implements the repair

Run: log `.temp/logs/cdx_brief_728.log`, 38,051 lines, sha256
`bfb4e679ba63c0dff2bdb509689a6b809478f2bc3042e93bed5a474b17130626`,
memory-scope peak 2,477,576,192 B, oom 0, oom_kill 0, rc=0; report
`.temp/brief-728/report.json`, 62 runs, 10 PASS / 4 FLAGGED / 3 FAIL /
2 NA, `report.py check` 604 citations (my own run: OK).

**My gate, reproduced myself.** The live tree is 2,318 files with 0
missing / 0 changed / 0 added against the entry manifest (my own walk of
`crates/`, `scripts/`, `docs/` and the top-level files); the 302,550
protected C7-C11/brief-623/726/727 files are unchanged; the seven
immediate proof inputs are exact. `proposed-shape.patch` applies to
`source-base/` and reproduces `source-candidate/` byte-for-byte on the
twelve files in `candidate-files.json`, and `source-base/` equals the
live tree on every file except the six brief-727 harness files. From the
raw records: base == frozen on all four records; candidate == disabled;
the own omission == base exactly; the nine extended class cases are
base-DIFF and candidate-SAME, with each candidate queued observation
byte-equal to the base *resumed* observation (message, rendered text,
span, line and column); the original two-pause record changes exactly
`semantic_queued` and `semantic_same`; both green controls are
byte-identical; the 60 P1566/P1568 cells are SAME/identical/valid with 0
new keys and 0 encoding runs; the 55/55 and 60/60 matrix cells re-derive
from the c12 raw logs; 118 active epoch summaries read `valid=true
faults=0 context_leases=0`.

**The cost, re-parsed by me from all twelve raw Callgrind captures.**
`resume_input_after_text` reads 160.0 Ir / 2 calls per iteration on base
in both draws (80.0 per call), 158.0 (79.0) on candidate1 and 154.0
(77.0) on candidate2, identical within each arm and draw;
`eval_expr_request_from_checkpoint` 412/4, `resume_eval_request_from_checkpoint`
1449/1 and the `EvalError` drop 29/1 exact on every arm;
`with_input_source` has zero entries in every disabled capture;
`EvalError` / `Result<Expr, EvalError>` / `Option<EvalError>` are 136 B
and `Session` 2,080 B in all twelve captures. My own sweep of all 874
captured functions finds no deterministic equal-call increase; the only
other named movements are the standing interner/hash-seed rows whose
within-arm draw spread exceeds the arm difference. The forced rebuild is
a fresh compile (`cache_hit=false`) equal to the base binary
`06a14f13…`.

**Rulings.**

- **R1 — the original two-pause record.** Duty 3's sentence "the
  original two-pause case must stay exactly as frozen" is withdrawn as my
  contradiction: `B726.class.error` is an instance of the class, not a
  control, so repairing it must move its queued arm. The frozen record
  stays byte-identical as historical evidence; the expected post-repair
  record changes exactly `semantic_queued` and `semantic_same`, and that
  delta is the repair's acceptance signature. No special case, no
  comparator change, no frozen rewrite.
- **R2 — the disabled-cost bar is no-increase.** The brief's "identical"
  is corrected to: no changed call-site row may execute more instructions
  per call than base; every other measured named row stays exact; the
  changed function's call count stays identical; and the capture helper
  executes zero times on the disabled success path. A deterministic
  equal-call increase on any named rum row is a STOP. A decrease is
  recorded, never padded to equality. The 80 → 79 and 80 → 77
  measurements pass under this bar. The microbench is NA for this shape:
  no `core-language-microbench.sh` label evaluates `Input`, so the lane
  cannot see the change; if the final shape touches any function
  reachable from a benchmark label, the lane runs.
- **R3 — the public API.** Accept `EvalError::Syntax { error:
  SyntaxError, input_source: Option<String> }` with an explicit
  `From<SyntaxError>` producing `None` and a runtime-private
  `with_input_source(self, String) -> Self` that only fills `None`.
  **Drop `#[source]`.** `#[error(transparent)]` forwards `Error::source()`
  to `SyntaxError::source()`, which is `None` because `SyntaxError`
  carries no source fields; a plain field with `#[error("{error}")]`
  preserves `Error::source()` exactly and keeps `Display` identical.
  Derived `Debug` gains the payload — inherent to carrying it, not a
  contract. The variant-shape change is a public Rust API break; the line
  permits it (user ruling 2026-09-19) and the director records it in
  `CHANGELOG.md` before S10.
- **R4 — no `Other` contract.** No documented host contract or permanent
  assertion requires `Other` for a malformed Input answer; the only
  `Other` assertions cover uncaught throws and completion outcomes. The
  resumed `Syntax` assertions stay unchanged. Queued `Syntax` is
  documented with the repair.
- **R5 — the disclosures.** The mandated shared `CARGO_HOME` cache
  metadata stamp is an explicit exception to the scene fence (Cargo owns
  it; no frozen evidence moved); the canonical probe-build's `--force`
  overwrote the first internal base build log at the same key, so the
  implementation brief requires a unique per-invocation `LOGS` root; the
  matrix re-read ran after candidate assembly but re-derives c12 raw
  evidence and re-pinned nothing.
- **R6 — the class duty.** This repair covers the Input-answer producer.
  `ToExpression` operands, `Session::lower_text`, the outer request
  `lower_syntax_expr`, loader-consumed errors and file-reread provenance
  are named open source owners, not silently dropped; they are carried as
  obligations into continuation 13 and the C2 settlement. `P1566` stays
  Lost until that campaign re-settles it on the installed source.
- **R7 — the STOP is accepted.** Both cost attempts are retained as
  measured; nothing was deleted, weakened or re-pinned by the worker.
- **R8 — brief 729 implements** the design in the corrected shape (no
  `#[source]`): the copied proof on the final source before any live
  edit, the permanent tests, the standing replays, the no-increase cost
  gate and the full acceptance.
